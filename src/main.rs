use dioxus::prelude::*;
mod components;
use crate::components::{Home, Navbar, Code, PageNotFound};
mod models;
// mod routes;

use crate::utils::{db, signing_code::*, time::*, api::*};
mod utils;
mod js_bindings;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const QR_SCANNER_JS: Asset = asset!("/assets/qr-scanner.js");
const QR_JS: Asset = asset!("/assets/jsQR.js");

fn main() {
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