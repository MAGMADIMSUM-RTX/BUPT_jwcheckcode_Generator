use dioxus::prelude::*;
// Import Route from its module. Adjust the path as needed based on your project structure.
use crate::Route;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {},
                h2 { "BUPTricks" }
            }
        }
        Outlet::<Route> {}
    }
}
