use crate::workflow::{error::InputParseError, input::Args};

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

    /// Parsed arguments.
    pub fn args(&self) -> &Args<T> {
        &self.args
    }

    /// Turn [Parsed] structure into owned [Args].
    pub fn into_args(self) -> Args<T> {
        self.args
    }

    /// Parsed function named.
    pub fn fun(&self) -> Option<String> {
        self.fun.as_ref().map(|f| f.to_string())
    }
}

impl<T> From<Parsed<T>> for Args<T> {
    fn from(apply: Parsed<T>) -> Self {
        apply.args
    }
}

/// Interface for [Instruction] implementations, relying on `homestore-core`
/// to implement custom parsing specifics.
///
/// # Example
///
/// ```
/// use homestar_core::{
///     workflow::{
///         input::{Args, Parse}, Ability, Input, Instruction,
///     },
///     Unit,
/// };
/// use libipld::Ipld;
/// use url::Url;
///
/// let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
/// let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();
///
/// let inst = Instruction::unique(
///     resource,
///     Ability::from("wasm/run"),
///     Input::<Unit>::Ipld(Ipld::List(vec![Ipld::Bool(true)]))
/// );
///
/// let parsed = inst.input().parse().unwrap();
///
/// // turn into Args for invocation:
/// let args: Args<Unit> = parsed.try_into().unwrap();
/// ```
///
/// [Instruction]: crate::workflow::Instruction
pub trait Parse<T> {
    /// Function returning [Parsed] structure for execution/invocation.
    ///
    /// Note: meant to come before the `resolve` step
    /// during runtime execution.
    fn parse(&self) -> Result<Parsed<T>, InputParseError<T>>;
}
