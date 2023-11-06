//! Procedural macros for homestar-runtime testing.

use proc_macro::TokenStream;
use quote::quote;

/// [Attribute macro] used for async tests that require a database to run
/// in parallel.
///
/// This macro will wrap a function in a `tokio::test` attribute automatically
/// and amend the signature to be async.
///
/// # Example
///
/// ```ignore
/// #[homestar_runtime_proc_macro::db_async_test]
/// fn initialize_a_worker() {
///     // Injected by the macro
///     let settings = TestSettings::load();
///
///     let (tx, mut rx) = test_utils::event::setup_channel(settings.clone().node);
///     let db = builder.db();
///     let builder = WorkerBuilder::new(settings.node).with_event_sender(tx.into());
///     let worker = builder.build().await;
///     let running_tasks = Arc::new(DashMap::new());
///     worker.run(running_tasks.clone()).await.unwrap();
///     assert_eq!(running_tasks.len(), 1);
/// }
/// ```
///
/// [Attribute macro]: <https://doc.rust-lang.org/reference/attributes.html#attribute-macros>
#[proc_macro_attribute]
pub fn db_async_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = func.sig.ident;
    let func_name_as_string = func_name.to_string();
    let body = func.block;

    quote! {
        #[allow(clippy::needless_return)]
        #[::tokio::test]
        async fn #func_name() {
            struct TestSettings;
            impl TestSettings {
                fn load() -> crate::Settings {
                    let mut settings = crate::Settings::load().unwrap();
                    settings.node.db.url = Some(format!("{}.db", #func_name_as_string));
                    settings
                }
            }
            #body
        }
    }
    .into()
}

/// [Attribute macro] used for homestar-runtime-related tests that require a
/// database to run in parallel.
///
/// This macro will wrap a function in a `#[test]` attribute automatically and
/// start a homestar-runtime instance with a temporary database.
///
/// # Example
///
/// ```ignore
/// #[homestar_runtime_proc_macro::runner_test]
/// fn spawn_an_rpc_server_and_ping_it() {
///     let TestRunner { runner, settings } = TestRunner::start();
///     let (tx, _rx) = Runner::setup_rpc_channel(1);
///     let rpc_server = rpc::Server::new(settings.node.network(), tx.into());
///     runner.runtime.block_on(rpc_server.spawn()).unwrap();
///     runner.runtime.spawn(async move {
///         let addr = SocketAddr::new(
///             settings.node.network.rpc_host,
///             settings.node.network.rpc_port,
///         );
///         let client = Client::new(addr, context::current()).await.unwrap();
///         let response = client.ping().await.unwrap();
///         assert_eq!(response, "pong".to_string());
///     });
/// }
/// ```
///
/// [Attribute macro]: <https://doc.rust-lang.org/reference/attributes.html#attribute-macros>
#[proc_macro_attribute]
pub fn runner_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = func.sig.ident;
    let func_name_as_string = func_name.to_string();
    let body = func.block;

    quote! {
        #[test]
        fn #func_name() {
            struct TestRunner {
                runner: crate::Runner,
                settings: crate::Settings,
            }
            impl TestRunner {
                fn start() -> TestRunner {
                    let mut settings = crate::Settings::load().unwrap();
                    settings.node.network.webserver_port = ::homestar_core::test_utils::ports::get_port() as u16;
                    settings.node.network.rpc_port = ::homestar_core::test_utils::ports::get_port() as u16;
                    settings.node.network.metrics_port = ::homestar_core::test_utils::ports::get_port() as u16;
                    settings.node.db.url = Some(format!("{}.db", #func_name_as_string));
                    let db = crate::test_utils::db::MemoryDb::setup_connection_pool(&settings.node, None).unwrap();
                    let runner = crate::Runner::start(settings.clone(), db).unwrap();
                    TestRunner { runner, settings }
                }
            }
            #body
        }
    }
    .into()
}
