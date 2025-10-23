use dioxus::prelude::*;
use crate::Route;
// use wasm_bindgen_futures::spawn_local;

#[component]
pub fn SystemUpdating() -> Element {
    rsx! {
        div {
            id: "page-not-found",
            style: "width: 100vw; height: 100vh; max-width: 350px; margin: 0 auto; display: flex; flex-direction: column; justify-content: center; align-items: center; text-align: center; padding: 20px;",
            h1 { "This Website is Reparing now. Please Wait..." }

            // p { {format!("This Website is Reparing now. Please Wait...")} }

        }
    }
}
