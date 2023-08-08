/// Random value generator for sampling data.
use proptest::{
    collection::vec,
    strategy::{Strategy, ValueTree},
    test_runner::{Config, TestRunner},
};

/// A random value generator (RVG), which, given proptest strategies, will
/// generate random values based on those strategies.
#[derive(Debug, Default)]
pub struct Rvg {
    runner: TestRunner,
}

impl Rvg {
    /// Creates a new RVG with the default random number generator.
    pub fn new() -> Self {
        Rvg {
            runner: TestRunner::new(Config::default()),
        }
    }

    /// Creates a new RVG with a deterministic random number generator,
    /// using the same seed across test runs.
    pub fn deterministic() -> Self {
        Rvg {
            runner: TestRunner::deterministic(),
        }
    }

    /// Samples a value for the given strategy.
    ///
    /// # Example
    ///
    /// ```
    /// use homestar_core::test_utils::Rvg;
    ///
    /// let mut rvg = Rvg::new();
    /// let int = rvg.sample(&(0..100i32));
    /// ```
    pub fn sample<S: Strategy>(&mut self, strategy: &S) -> S::Value {
        strategy
            .new_tree(&mut self.runner)
            .expect("No value can be generated")
            .current()
    }

    /// Samples a vec of some length with a value for the given strategy.
    ///
    /// # Example
    ///
    /// ```
    /// use homestar_core::test_utils::Rvg;
    ///
    /// let mut rvg = Rvg::new();
    /// let ints = rvg.sample_vec(&(0..100i32), 10);
    /// ```
    pub fn sample_vec<S: Strategy>(&mut self, strategy: &S, len: usize) -> Vec<S::Value> {
        vec(strategy, len..=len)
            .new_tree(&mut self.runner)
            .expect("No value can be generated")
            .current()
    }
}
