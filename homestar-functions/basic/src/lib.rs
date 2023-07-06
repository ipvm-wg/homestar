mod example;

#[cfg(feature = "example-add")]
pub use example::add::Component as AddComponent;

#[cfg(feature = "example-test")]
pub use example::test::Component as TestComponent;
