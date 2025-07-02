use dioxus::prelude::*;
use crate::Route;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn PageNotFound(segments: Vec<String>) -> Element {
    // Auto redirect after 1 second
    use_effect(move || {
        spawn_local(async move {
            // Wait 1 second
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1000)
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

            // Navigate to home using window.location
            let window = web_sys::window().unwrap();
            let location = window.location();
            let _ = location.set_href("/");
        });
    });

    rsx! {
        div {
            id: "page-not-found",
            style: "width: 100vw; height: 100vh; max-width: 350px; margin: 0 auto; display: flex; flex-direction: column; justify-content: center; align-items: center; text-align: center; padding: 20px;",
            h1 { "404 - Page Not Found" }

            p { {format!("The page /{} you are looking for does not exist.", segments.join("/"))} }

            p { "You'll be redirected to the home page in 1 second..." }
            Link {
                to: Route::Home {},
                style: "color: #91a4d2; text-decoration: none; margin-top: 20px; padding: 10px 20px; border: 1px solid #91a4d2; border-radius: 5px;",
                "Go to Home Now"
            }
        }
    }
}
