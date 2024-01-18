//! Input paramters for [Instruction] execution and means to
//! generally [parse] and [resolve] them.
//!
//! [Instruction]: super::Instruction
//! [parse]: Parse::parse
//! [resolve]: Args::resolve

use crate::{
    error::ResolveError,
    pointer::{Await, AwaitResult, ERR_BRANCH, OK_BRANCH, PTR_BRANCH},
    task, Error, Pointer,
};
use async_recursion::async_recursion;
use futures::{future, future::BoxFuture};
use libipld::{serde::from_ipld, Cid, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::btree_map::BTreeMap, result::Result, sync::Arc};

mod parse;
pub use parse::{Parse, Parsed};

/// A list of ordered [Input] arguments/parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Args<T>(Vec<Input<T>>);

impl<T> Args<T>
where
    T: std::fmt::Debug,
{
    /// Create an [Args] [Vec]-type.
    pub fn new(args: Vec<Input<T>>) -> Self {
        Self(args)
    }

    /// Return wrapped [Vec] of [inputs].
    ///
    /// [inputs]: Input
    pub fn into_inner(self) -> Vec<Input<T>> {
        self.0
    }

    /// Return refeerence to a wrapped [Vec] of [inputs].
    ///
    /// [inputs]: Input
    pub fn inner(&self) -> &Vec<Input<T>> {
        &self.0
    }

    /// Return *only* deferred/awaited inputs.
    pub fn deferreds(&self) -> impl Iterator<Item = Cid> + '_ {
        self.0.iter().filter_map(|input| {
            if let Input::Deferred(awaited_promise) = input {
                Some(awaited_promise.instruction_cid())
            } else {
                None
            }
        })
    }

    /// Return *only* [Ipld::Link] [Cid]s.
    pub fn links(&self) -> impl Iterator<Item = Cid> + '_ {
        self.0.iter().filter_map(|input| {
            if let Input::Ipld(Ipld::Link(link)) = input {
                Some(link.to_owned())
            } else {
                None
            }
        })
    }

    /// Resolve [awaited promises] of [inputs] into task-specific [Input::Arg]'s,
    /// given a successful lookup function; otherwise, return [Input::Deferred]
    /// for unresolved promises, or just return [Input::Ipld],
    /// [resolving Ipld links] if the lookup function expected [Ipld] input data.
    ///
    /// [awaited promises]: Await
    /// [inputs]: Input
    /// [resolving Ipld links]: resolve_links
    pub async fn resolve<'a, F>(self, lookup_fn: F) -> Result<Self, ResolveError>
    where
        F: Fn(Cid) -> BoxFuture<'a, Result<task::Result<T>, ResolveError>> + Clone + Send + Sync,
        Ipld: From<T>,
    {
        let inputs = resolve_args(self.0, lookup_fn);
        Ok(Args(inputs.await))
    }
}

impl<T> From<Args<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(args: Args<T>) -> Self {
        let args = args.0.into_iter().map(|v| v.into());
        Ipld::List(args.collect())
    }
}

impl<T> TryFrom<Ipld> for Args<T>
where
    task::Result<T>: TryFrom<Ipld>,
{
    type Error = Error<T>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::List(vec) = ipld {
            let args = vec
                .into_iter()
                .fold(Vec::<Input<T>>::new(), |mut acc, ipld| {
                    if let Ok(invocation_result) = task::Result::try_from(ipld.to_owned()) {
                        acc.push(Input::Arg(invocation_result));
                    } else if let Ok(await_result) = Await::try_from(ipld.to_owned()) {
                        acc.push(Input::Deferred(await_result));
                    } else {
                        acc.push(Input::Ipld(ipld))
                    }

                    acc
                });
            Ok(Args(args))
        } else {
            Err(Error::not_an_ipld_list())
        }
    }
}

/// Contains parameters expected by the [URI]/[Ability] pair.
///
/// Left to the executor to define the shape of this data, per job.
///
/// [URI]: [url::Url]
/// [Ability]: super::Ability
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Input<T> {
    /// [Ipld] Literals.
    Ipld(Ipld),
    /// Promise-[links] awaiting the output of another [Instruction]'s
    /// invocation, directly.
    ///
    /// [links]: Pointer
    /// [Instruction]: super::Instruction
    Deferred(Await),
    /// General argument, wrapping an [task::Result] over a task-specific
    /// implementation's own input type(s).
    Arg(task::Result<T>),
}

impl<T> Input<T> {
    /// Resolve [awaited promise] of an [Input] into a task-specific
    /// [Input::Arg], given a successful lookup function; otherwise, return
    /// [Input::Deferred] for an unresolved promise, or just return
    /// [Input::Ipld], [resolving Ipld links] if the lookup function expected
    /// [Ipld] input data.
    ///
    /// [awaited promises]: Await
    /// [inputs]: Input
    /// [resolving Ipld links]: resolve_links
    pub async fn resolve<'a, F>(self, lookup_fn: F) -> Input<T>
    where
        F: Fn(Cid) -> BoxFuture<'a, Result<task::Result<T>, ResolveError>> + Clone + Send + Sync,
        Ipld: From<T>,
    {
        match self {
            Input::Ipld(ipld) => {
                if let Ok(await_promise) = Await::try_from(&ipld) {
                    if let Ok(func_ret) = lookup_fn(await_promise.instruction_cid()).await {
                        Input::Arg(func_ret)
                    } else {
                        Input::Deferred(await_promise)
                    }
                } else {
                    Input::Ipld(resolve_links(ipld, lookup_fn.into()).await)
                }
            }
            Input::Arg(ref _arg) => self,
            Input::Deferred(await_promise) => {
                if let Ok(func_ret) = lookup_fn(await_promise.instruction_cid()).await {
                    Input::Arg(func_ret)
                } else {
                    Input::Deferred(await_promise)
                }
            }
        }
    }
}

impl<T> From<Input<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(input: Input<T>) -> Self {
        match input {
            Input::Ipld(ipld) => ipld,
            Input::Deferred(promise) => Await::into(promise),
            Input::Arg(arg) => arg.into(),
        }
    }
}

impl<T> From<Await> for Input<T> {
    fn from(await_promise: Await) -> Self {
        Input::Deferred(await_promise)
    }
}

impl<T> TryFrom<Ipld> for Input<T>
where
    T: From<Ipld>,
{
    type Error = Error<String>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let Ok(map) = from_ipld::<BTreeMap<String, Ipld>>(ipld.to_owned()) else {
            if let Ok(invocation_result) = ipld.to_owned().try_into() {
                return Ok(Input::Arg(invocation_result));
            } else {
                return Ok(Input::Ipld(ipld));
            }
        };

        map.get_key_value(OK_BRANCH)
            .or_else(|| map.get_key_value(ERR_BRANCH))
            .or_else(|| map.get_key_value(PTR_BRANCH))
            .map_or(
                if let Ok(invocation_result) = task::Result::try_from(ipld.to_owned()) {
                    Ok(Input::Arg(invocation_result))
                } else {
                    Ok(Input::Ipld(ipld))
                },
                |(branch, ipld)| {
                    let instruction = Pointer::try_from(ipld)?;
                    Ok(Input::Deferred(Await::new(
                        instruction,
                        AwaitResult::result(branch)
                            .ok_or_else(|| Error::InvalidDiscriminant(branch.to_string()))?,
                    )))
                },
            )
    }
}

async fn resolve_args<'a, T, F>(args: Vec<Input<T>>, lookup_fn: F) -> Vec<Input<T>>
where
    F: Fn(Cid) -> BoxFuture<'a, Result<task::Result<T>, ResolveError>> + Clone + Send + Sync,
    Ipld: From<T>,
{
    let args = args.into_iter().map(|v| v.resolve(lookup_fn.clone()));
    future::join_all(args).await.into_iter().collect()
}

/// Resolve [awaited promises] for *only* [Ipld] data, given a lookup function.
///
/// [awaited promises]: Await
#[async_recursion]
pub async fn resolve_links<'a, T, F>(ipld: Ipld, lookup_fn: Arc<F>) -> Ipld
where
    F: Fn(Cid) -> BoxFuture<'a, Result<task::Result<T>, ResolveError>> + Clone + Sync + Send,
    Ipld: From<T>,
{
    match ipld {
        Ipld::Map(m) => {
            let futures = m.into_iter().map(|(k, v)| async {
                match v {
                    Ipld::Link(cid) => {
                        let mut f = Arc::clone(&lookup_fn);
                        if let Ok(func_ret) = Arc::make_mut(&mut f)(cid).await {
                            if k.eq(PTR_BRANCH) {
                                (k, func_ret.into())
                            } else {
                                (k, func_ret.into_inner().into())
                            }
                        } else {
                            (k, v)
                        }
                    }
                    Ipld::Map(ref m) => {
                        let resolved = resolve_links(Ipld::Map(m.clone()), lookup_fn.clone()).await;
                        (k, resolved)
                    }
                    Ipld::List(ref l) => {
                        let resolved =
                            resolve_links(Ipld::List(l.clone()), lookup_fn.clone()).await;
                        (k, resolved)
                    }
                    _ => (k, v),
                }
            });
            let resolved_results = future::join_all(futures).await;
            Ipld::Map(
                resolved_results
                    .into_iter()
                    .collect::<BTreeMap<String, Ipld>>(),
            )
        }
        Ipld::List(l) => {
            let futures = l.into_iter().map(|v| async {
                match v {
                    Ipld::Link(cid) => {
                        let mut f = Arc::clone(&lookup_fn);
                        if let Ok(func_ret) = Arc::make_mut(&mut f)(cid).await {
                            func_ret.into_inner().into()
                        } else {
                            v
                        }
                    }
                    Ipld::Map(ref m) => {
                        resolve_links(Ipld::Map(m.clone()), lookup_fn.clone()).await
                    }
                    Ipld::List(ref l) => {
                        resolve_links(Ipld::List(l.clone()), lookup_fn.clone()).await
                    }
                    _ => v,
                }
            });
            let resolved_results = future::join_all(futures).await;
            Ipld::List(resolved_results)
        }
        Ipld::Link(link) => {
            let mut f = Arc::clone(&lookup_fn);
            if let Ok(func_ret) = Arc::make_mut(&mut f)(link).await {
                func_ret.into_inner().into()
            } else {
                Ipld::Link(link)
            }
        }
        _ => ipld,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils, Unit};

    #[test]
    fn input_ipld_ipld_rountrip() {
        let input: Input<Unit> = Input::Ipld(Ipld::List(vec![Ipld::Bool(true)]));
        let ipld = Ipld::from(input.clone());

        assert_eq!(ipld, Ipld::List(vec![Ipld::Bool(true)]));
        assert_eq!(input, ipld.try_into().unwrap());
    }

    #[test]
    fn input_deferred_ipld_rountrip() {
        let instruction = test_utils::instruction::<Unit>();
        let ptr: Pointer = instruction.try_into().unwrap();
        let input: Input<Unit> = Input::Deferred(Await::new(ptr.clone(), AwaitResult::Ptr));
        let ipld = Ipld::from(input.clone());

        assert_eq!(
            ipld,
            Ipld::Map(BTreeMap::from([(PTR_BRANCH.into(), Ipld::Link(ptr.cid()))]))
        );
        assert_eq!(input, ipld.try_into().unwrap());
    }

    #[test]
    fn input_arg_ipld_rountrip() {
        let input: Input<Ipld> = Input::Arg(task::Result::Just(Ipld::Bool(false)));
        let ipld = Ipld::from(input.clone());

        assert_eq!(
            ipld,
            Ipld::List(vec![Ipld::String("just".into()), Ipld::Bool(false)])
        );
        assert_eq!(input, ipld.try_into().unwrap());
    }

    #[test]
    fn args_ipld_rountrip() {
        let input: Input<Unit> = Input::Ipld(Ipld::Bool(true));
        let args = Args::new(vec![input]);
        let ipld = Ipld::from(args.clone());

        assert_eq!(ipld, Ipld::List(vec![Ipld::Bool(true)]));
        assert_eq!(args, ipld.try_into().unwrap());
    }

    #[test]
    fn ser_de_ipld() {
        let input: Input<Unit> = Input::Ipld(Ipld::Bool(true));
        let ser = serde_json::to_string(&input).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(input, de);
    }

    #[test]
    fn ser_de_arg_ipld() {
        let input: Input<Ipld> = Input::Arg(task::Result::Just(Ipld::Bool(false)));
        let ser = serde_json::to_string(&input).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(input, de);
    }
}
