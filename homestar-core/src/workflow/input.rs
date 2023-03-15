//! Input paramters for [Task] execution and means to
//! generally [parse] and [resolve] them.
//!
//! [Task]: super::Task
//! [parse]: Parse::parse
//! [resolve]: Args::resolve

use super::{
    pointer::{Await, AwaitResult, InvokedTaskPointer, ERR_BRANCH, OK_BRANCH, PTR_BRANCH},
    InvocationResult,
};
use anyhow::anyhow;
use libipld::{serde::from_ipld, Cid, Ipld};
use std::{
    collections::{btree_map::BTreeMap, HashMap},
    result::Result,
};

/// Generic link, cid => T [HashMap] for storing
/// invoked, raw values in-memory and using them to
/// resolve other steps within a runtime's workflow.
pub type LinkMap<T> = HashMap<Cid, T>;

/// Parsed [Args] consisting of [Inputs] for execution flows, as well as an
/// optional function name/definition.
///
/// TODO: Extend via enumeration for singular objects/values.
///
/// [Inputs]: super::Input
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed<T> {
    args: Args<T>,
    fun: Option<String>,
}

impl<T> Parsed<T> {
    /// Initiate [Parsed] data structure with only [Args].
    pub fn with(args: Args<T>) -> Self {
        Parsed { args, fun: None }
    }

    /// Initiate [Parsed] data structure with a function name and
    /// [Args].
    pub fn with_fn(fun: String, args: Args<T>) -> Self {
        Parsed {
            args,
            fun: Some(fun),
        }
    }
}

impl<T> From<Parsed<T>> for Args<T> {
    fn from(apply: Parsed<T>) -> Self {
        apply.args
    }
}

/// Interface for [Task] implementations, relying on `core`
/// to implement for custom parsing specifics.
///
/// # Example
///
/// ```
/// use homestar_core::{
///     workflow::{
///         input::{Args, Parse}, Ability, Input, Task,
///     },
///     Unit,
/// };
/// use libipld::Ipld;
/// use url::Url;
///
/// let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
/// let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();
///
/// let task = Task::unique(
///     resource,
///     Ability::from("wasm/run"),
///     Input::<Unit>::Ipld(Ipld::List(vec![Ipld::Bool(true)]))
/// );
///
/// let parsed = task.input().parse().unwrap();
///
/// // turn into Args for invocation:
/// let args: Args<Unit> = parsed.try_into().unwrap();
/// ```
///
/// [Task]: super::Task
pub trait Parse<T> {
    /// Function returning [Parsed] structure for execution/invocation.
    ///
    /// Note: meant to come before the `resolve` step
    /// during runtime execution.
    fn parse(&self) -> anyhow::Result<Parsed<T>>;
}

/// A list of ordered [Input] arguments/parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Args<T>(Vec<Input<T>>);

impl<T> Args<T> {
    /// Create an [Args] [Vec]-type.
    pub fn new(args: Vec<Input<T>>) -> Self {
        Args(args)
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

    /// Resolve [awaited promises] of [inputs] into task-specific [Input::Arg]'s,
    /// given a successful lookup function; otherwise, return [Input::Deferred]
    /// for unresolved promises, or just return [Input::Ipld],
    /// [resolving Ipld links] if the lookup function expected [Ipld] input data.
    ///
    /// [awaited promises]: Await
    /// [inputs]: Input
    /// [resolving Ipld links]: resolve_links
    pub fn resolve<F>(self, lookup_fn: F) -> anyhow::Result<Self>
    where
        F: Fn(Cid) -> anyhow::Result<InvocationResult<T>> + Clone,
        Ipld: From<T>,
        T: Clone,
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
    InvocationResult<T>: TryFrom<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, anyhow::Error> {
        if let Ipld::List(vec) = ipld {
            let args = vec
                .into_iter()
                .fold(Vec::<Input<T>>::new(), |mut acc, ipld| {
                    if let Ok(invocation_result) = InvocationResult::try_from(ipld.to_owned()) {
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
            Err(anyhow!("unexpected conversion type"))
        }
    }
}

/// Contains parameters expected by the [URI]/[Ability] pair.
///
/// Left to the executor to define the shape of this data, per job.
///
/// [URI]: [url::Url]
/// [Ability]: super::Ability
#[derive(Clone, Debug, PartialEq)]
pub enum Input<T> {
    /// [Ipld] Literals.
    Ipld(Ipld),
    /// Promise-[links] awaiting the output of another [Task]'s invocation,
    /// directly.
    ///
    /// [links]: InvokedTaskPointer
    /// [Task]: super::Task
    Deferred(Await),
    /// General argument, wrapping an [InvocationResult] over a task-specific
    /// implementation's own input type(s).
    Arg(InvocationResult<T>),
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
    pub fn resolve<F>(self, lookup_fn: F) -> Input<T>
    where
        F: Fn(Cid) -> anyhow::Result<InvocationResult<T>> + Clone,
        Ipld: From<T>,
    {
        match self {
            Input::Ipld(ipld) => {
                if let Ok(await_promise) = Await::try_from(&ipld) {
                    if let Ok(func_ret) = lookup_fn(await_promise.task_cid()) {
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
                if let Ok(func_ret) = lookup_fn(await_promise.task_cid()) {
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
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let Ok(map) = from_ipld::<BTreeMap<String, Ipld>>(ipld.to_owned()) else {
            if let Ok(invocation_result) = ipld.to_owned().try_into() {
                return Ok(Input::Arg(invocation_result))
            } else {
                return Ok(Input::Ipld(ipld))
            }
        };

        map.get_key_value(OK_BRANCH)
            .or_else(|| map.get_key_value(ERR_BRANCH))
            .or_else(|| map.get_key_value(PTR_BRANCH))
            .map_or(
                if let Ok(invocation_result) = InvocationResult::try_from(ipld.to_owned()) {
                    Ok(Input::Arg(invocation_result))
                } else {
                    Ok(Input::Ipld(ipld))
                },
                |(branch, ipld)| {
                    let invoked_task = InvokedTaskPointer::try_from(ipld)?;
                    Ok(Input::Deferred(Await::new(
                        invoked_task,
                        AwaitResult::result(branch)
                            .ok_or_else(|| anyhow!("wrong branch name: {branch}"))?,
                    )))
                },
            )
    }
}

fn resolve_args<T, F>(args: Vec<Input<T>>, lookup_fn: F) -> Vec<Input<T>>
where
    F: Fn(Cid) -> anyhow::Result<InvocationResult<T>> + Clone,
    Ipld: From<T>,
{
    let args = args.into_iter().map(|v| v.resolve(lookup_fn.clone()));
    args.collect()
}

/// Resolve [awaited promises] for *only* [Ipld] data, given a lookup function.
///
/// [awaited promises]: Await
pub fn resolve_links<T, F>(ipld: Ipld, lookup_fn: F) -> Ipld
where
    F: Fn(Cid) -> anyhow::Result<InvocationResult<T>> + Clone,
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
    use crate::{
        workflow::{Ability, Nonce, Task},
        Unit,
    };
    use url::Url;

    fn task<'a, T>() -> Task<'a, T> {
        let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
        let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();
        let nonce = Nonce::generate();

        Task::new(
            resource,
            Ability::from("wasm/run"),
            Input::Ipld(Ipld::List(vec![Ipld::Integer(88)])),
            Some(nonce),
        )
    }

    #[test]
    fn input_ipld_ipld_rountrip() {
        let input: Input<Unit> = Input::Ipld(Ipld::List(vec![Ipld::Bool(true)]));
        let ipld = Ipld::from(input.clone());

        assert_eq!(ipld, Ipld::List(vec![Ipld::Bool(true)]));
        assert_eq!(input, ipld.try_into().unwrap());
    }

    #[test]
    fn input_deferred_ipld_rountrip() {
        let task: Task<'_, Unit> = task();
        let ptr: InvokedTaskPointer = task.try_into().unwrap();
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
        let input: Input<Ipld> = Input::Arg(InvocationResult::Just(Ipld::Bool(false)));
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
