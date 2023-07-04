//! Input paramters for [Instruction] execution and means to
//! generally [parse] and [resolve] them.
//!
//! [Instruction]: super::Instruction
//! [parse]: Parse::parse
//! [resolve]: Args::resolve

use crate::workflow::{
    self,
    error::ResolveError,
    pointer::{Await, AwaitResult, ERR_BRANCH, OK_BRANCH, PTR_BRANCH},
    InstructionResult, Pointer,
};
use libipld::{serde::from_ipld, Cid, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::btree_map::BTreeMap, result::Result};

mod parse;
pub use parse::*;

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
    pub fn deferreds(&self) -> Vec<Cid> {
        self.0.iter().fold(vec![], |mut acc, input| {
            if let Input::Deferred(awaited_promise) = input {
                acc.push(awaited_promise.instruction_cid());
                acc
            } else {
                acc
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
    pub fn resolve<F>(self, lookup_fn: F) -> Result<Self, ResolveError>
    where
        F: FnMut(Cid) -> Result<InstructionResult<T>, ResolveError> + Clone,
        Ipld: From<T>,
    {
        let inputs = resolve_args(self.0, lookup_fn);
        Ok(Args(inputs))
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
    InstructionResult<T>: TryFrom<Ipld>,
{
    type Error = workflow::Error<T>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::List(vec) = ipld {
            let args = vec
                .into_iter()
                .fold(Vec::<Input<T>>::new(), |mut acc, ipld| {
                    if let Ok(invocation_result) = InstructionResult::try_from(ipld.to_owned()) {
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
            Err(workflow::Error::not_an_ipld_list())
        }
    }
}

/// Contains parameters expected by the [URI]/[Ability] pair.
///
/// Left to the executor to define the shape of this data, per job.
///
/// [URI]: [url::Url]
/// [Ability]: super::Ability
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Input<T> {
    /// [Ipld] Literals.
    Ipld(Ipld),
    /// Promise-[links] awaiting the output of another [Instruction]'s
    /// invocation, directly.
    ///
    /// [links]: Pointer
    /// [Instruction]: super::Instruction
    Deferred(Await),
    /// General argument, wrapping an [InstructionResult] over a task-specific
    /// implementation's own input type(s).
    Arg(InstructionResult<T>),
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
    pub fn resolve<F>(self, mut lookup_fn: F) -> Input<T>
    where
        F: FnMut(Cid) -> Result<InstructionResult<T>, ResolveError> + Clone,
        Ipld: From<T>,
    {
        match self {
            Input::Ipld(ipld) => {
                if let Ok(await_promise) = Await::try_from(&ipld) {
                    if let Ok(func_ret) = lookup_fn(await_promise.instruction_cid()) {
                        Input::Arg(func_ret)
                    } else {
                        Input::Deferred(await_promise)
                    }
                } else {
                    Input::Ipld(resolve_links(ipld, lookup_fn))
                }
            }
            Input::Arg(ref _arg) => self,
            Input::Deferred(await_promise) => {
                if let Ok(func_ret) = lookup_fn(await_promise.instruction_cid()) {
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
    type Error = workflow::Error<String>;

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
                if let Ok(invocation_result) = InstructionResult::try_from(ipld.to_owned()) {
                    Ok(Input::Arg(invocation_result))
                } else {
                    Ok(Input::Ipld(ipld))
                },
                |(branch, ipld)| {
                    let instruction = Pointer::try_from(ipld)?;
                    Ok(Input::Deferred(Await::new(
                        instruction,
                        AwaitResult::result(branch).ok_or_else(|| {
                            workflow::Error::InvalidDiscriminant(branch.to_string())
                        })?,
                    )))
                },
            )
    }
}

fn resolve_args<T, F>(args: Vec<Input<T>>, lookup_fn: F) -> Vec<Input<T>>
where
    F: FnMut(Cid) -> Result<InstructionResult<T>, ResolveError> + Clone,
    Ipld: From<T>,
{
    let args = args.into_iter().map(|v| v.resolve(lookup_fn.clone()));
    args.collect()
}

/// Resolve [awaited promises] for *only* [Ipld] data, given a lookup function.
///
/// [awaited promises]: Await
pub fn resolve_links<T, F>(ipld: Ipld, mut lookup_fn: F) -> Ipld
where
    F: FnMut(Cid) -> Result<InstructionResult<T>, ResolveError> + Clone,
    Ipld: From<T>,
{
    match ipld {
        Ipld::Map(m) => {
            let btree = m.into_iter().map(|(k, v)| match v {
                Ipld::Link(cid) => {
                    if let Ok(func_ret) = lookup_fn(cid) {
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
                    let resolved = resolve_links(Ipld::Map(m.clone()), lookup_fn.clone());
                    (k, resolved)
                }
                Ipld::List(ref l) => {
                    let resolved = resolve_links(Ipld::List(l.clone()), lookup_fn.clone());
                    (k, resolved)
                }
                _ => (k, v),
            });

            Ipld::Map(btree.collect::<BTreeMap<String, Ipld>>())
        }
        Ipld::List(l) => {
            let list = l.into_iter().map(|v| match v {
                Ipld::Link(cid) => {
                    if let Ok(func_ret) = lookup_fn(cid) {
                        func_ret.into_inner().into()
                    } else {
                        v
                    }
                }
                Ipld::Map(ref m) => resolve_links(Ipld::Map(m.clone()), lookup_fn.clone()),
                Ipld::List(ref l) => resolve_links(Ipld::List(l.clone()), lookup_fn.clone()),
                _ => v,
            });

            Ipld::List(list.collect())
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
        let instruction = test_utils::workflow::instruction::<Unit>();
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
        let input: Input<Ipld> = Input::Arg(InstructionResult::Just(Ipld::Bool(false)));
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
}
