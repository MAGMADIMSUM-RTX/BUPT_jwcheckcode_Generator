use dioxus::prelude::*;
mod components;
use crate::components::{Code, Home, Navbar, PageNotFound};
mod models;
// mod routes;

use crate::utils::{api::*, db, signing_code::*, time::*};
mod js_bindings;
mod utils;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const QR_SCANNER_JS: Asset = asset!("/assets/qr-scanner.js");
const QR_JS: Asset = asset!("/assets/jsQR.js");

fn main() {
    #[cfg(not(feature = "server"))]
    {
        // 从环境变量中获取服务器地址
        let server_url: &'static str = match std::env::var("SERVER_URL") {
            Ok(url) => Box::leak(url.into_boxed_str()),
            Err(_) => "https://server.buptricks.top",
        };
        server_fn::client::set_server_url(server_url);
    }
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        document::Script { src: QR_JS }
        document::Script { src: QR_SCANNER_JS }
        Router::<Route> {}
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
    #[route("/")]
    Home {},
    #[route("/:code_gen_option/:id")]
    Code { code_gen_option: String, id: String},
}

// #[cfg(feature = "server")]
// async fn launch_server() {
//     // Connect to dioxus' logging infrastructure
//     dioxus::logger::initialize_default();

//     // Connect to the IP and PORT env vars passed by the Dioxus CLI (or your dockerfile)
//     let socket_addr = dioxus::cli_config::fullstack_address_or_localhost();

//     // Build a custom axum router
//     let router = axum::Router::new()
//         .serve_dioxus_application(ServeConfigBuilder::new(), App)
//         .into_make_service();

//     // And launch it!
//     let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
//     axum::serve(listener, router).await.unwrap();
// }
