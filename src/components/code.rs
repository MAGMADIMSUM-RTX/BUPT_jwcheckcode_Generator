use crate::components::PageNotFound;
use crate::models::ClassData;
use crate::utils::*;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

// 复制到剪贴板的函数
fn copy_to_clipboard(text: &str) {
    let text = text.to_string();
    spawn_local(async move {
        // 简单的复制方案：显示文本内容供用户手动复制
        // web_sys::console::log_1(&format!("请复制以下内容: {}", text).into());

        // 尝试使用现代剪贴板 API
        if let Some(window) = web_sys::window() {
            if let Ok(clipboard) = js_sys::Reflect::get(&window.navigator(), &"clipboard".into()) {
                if !clipboard.is_undefined() {
                    let clipboard_obj = clipboard.dyn_into::<js_sys::Object>().unwrap();
                    if let Ok(write_text) =
                        js_sys::Reflect::get(&clipboard_obj, &"writeText".into())
                    {
                        if let Ok(write_text_fn) = write_text.dyn_into::<js_sys::Function>() {
                            let args = js_sys::Array::new();
                            args.push(&text.into());
                            if let Ok(_) = write_text_fn.apply(&clipboard_obj, &args) {
                                web_sys::console::log_1(&"复制成功".into());
                                return;
                            }
                        }
                    }
                }
            }
        }

        web_sys::console::log_1(&"复制功能不可用，请手动复制控制台中的内容".into());
    });
}

#[component]
pub fn Code(code_gen_option: String, id: String) -> Element {
    let class_data = use_signal(|| None::<ClassData>);
    let error_message = use_signal(|| String::new());
    let loading = use_signal(|| true); // 初始状态为加载中

    let mut help_message = use_signal(|| String::new());

    // 根据 code_gen_option 决定如何处理 id
    let site_id = match code_gen_option.as_str() {
        "id" => id.clone(),
        "name" => id.clone(),
        other => {
            return rsx! {
                PageNotFound { segments: vec![other.to_string(), id.clone()] }
            };
        }
    };

    // 加载数据
    use_effect(move || {
        let site_id = site_id.clone();
        let mut class_data = class_data.clone();
        let mut error_message = error_message.clone();
        let mut loading = loading.clone();
        // let mut

        spawn_local(async move {
            loading.set(true);
            match get_class_data(site_id).await {
                Ok(Some(data)) => {
                    class_data.set(Some(data));
                    error_message.set(String::new());
                }
                Ok(None) => {
                    error_message.set("未找到指定的课程数据".to_string());
                }
                Err(e) => {
                    error_message.set(format!("加载数据失败: {:?}", e));
                }
            }
            loading.set(false);
        });
    });

    let mut img_src = use_resource(move || {
        let class_data = class_data();
        async move {
            if let Some(data) = class_data {
                let qr_data = format_signing_code(&data);
                // 简单的 URL 编码，替换特殊字符
                let encoded_data = qr_data
                    .replace("|", "%7C")
                    .replace("&", "%26")
                    .replace("=", "%3D");
                format!(
                    "https://api.2dcode.biz/v1/create-qr-code?data={}&size=256x256",
                    encoded_data
                )
            } else {
                "".to_string()
            }
        }
    });

    // 每2秒自动刷新二维码
    use_effect(move || {
        let mut img_src = img_src.clone();
        spawn_local(async move {
            loop {
                // 等待2秒
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    web_sys::window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 2000)
                        .unwrap();
                });
                wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

                // 刷新二维码
                img_src.restart();
            }
        });
    });

    rsx! {
        div { id: "code", class: "home-container",
            div { class: "home-scanner-container",
                h1 { class: "home-title", "CHECK NOW!" }

                if loading() {
                    div { class: "home-warning-message",
                        p { "正在加载数据..." }
                    }
                } else if let Some(_data) = class_data() {
                    div {
                        // 显示课程信息
                        // div { class: "course-info",
                        //     p { "课程: {data.class_name}" }
                        //     p { "班级: {data.classes}" }
                        //     if let Some(last_time) = data.last_created_time.as_ref() {
                        //         p { "上次创建: {last_time}" }
                        //     }
                        //     p {
                        //         if data.is_expired {
                        //             "状态: 已过期"
                        //         } else {
                        //             "状态: 有效"
                        //         }
                        //     }
                        // }

                        // 二维码显示区域
                        div { class: "qr-code-container",
                            if let Some(src) = img_src() {
                                if !src.is_empty() {
                                    img {
                                        src: "{src}",
                                        class: "qr-code-image",
                                        alt: "签到二维码",
                                    }
                                } else {
                                    div { class: "home-success-message",
                                        p { "正在生成二维码..." }
                                    }
                                }
                            } else {
                                div { class: "home-success-message",
                                    p { "正在生成二维码..." }
                                }
                            }
                        }

                        // 按钮组
                        div { class: "button-group",
                            button {
                                onclick: move |_| img_src.restart(),
                                class: "home-button-primary",
                                "手动刷新"
                            }
                            button {
                                onclick: move |_| {
                                    if let Some(window) = web_sys::window() {
                                        if let Some(location) = window.location().href().ok() {
                                            copy_to_clipboard(&location);
                                        }
                                    }
                                    help_message
                                        .set(
                                            "链接已复制到剪贴板，可以通过该链接访问此页面。"
                                                .to_string(),
                                        );
                                },
                                class: "home-button-primary",
                                "复制链接"
                            }
                                                // button {
                        //     onclick: move |_| {
                        //         if let Some(window) = web_sys::window() {
                        //             let _ = window.location().set_href("/");
                        //         }
                        //     },
                        //     class: "home-button-primary",
                        //     "返回扫码"
                        // }
                        }
                    }
                } else if !error_message().is_empty() {
                    div { class: "home-error-message",
                        h3 { "错误:" }
                        p { "{error_message()}" }
                        button {
                            onclick: move |_| {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/");
                                }
                            },
                            class: "home-error-close-button",
                            "返回扫码"
                        }
                    }
                } else {
                    div { class: "home-warning-message",
                        p { "未找到课程数据" }
                        button {
                            onclick: move |_| {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/");
                                }
                            },
                            class: "home-button-primary",
                            "返回扫码"
                        }
                    }
                }
            }
            MessageDisplay { error_message, help_message }
        }
    }
}

#[component]
fn MessageDisplay(error_message: Signal<String>, help_message: Signal<String>) -> Element {
    rsx! {
        if !help_message().is_empty() {
            div { class: "home-help-message",
                h3 { "帮助:" }
                p { "{help_message()}" }
                button {
                    onclick: move |_| help_message.set(String::new()),
                    class: "home-help-close-button",
                    "关闭"
                }
            }
        }

        // 错误消息
        if !error_message().is_empty() {
            div { class: "home-error-message",
                h3 { "错误:" }
                p { "{error_message()}" }
                button {
                    onclick: move |_| error_message.set(String::new()),
                    class: "home-error-close-button",
                    "关闭"
                }
            }
        }
    }
}
