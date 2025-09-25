use crate::models::SigningCode;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
// use crate::routes::Route;
use crate::js_bindings::*;
use crate::utils::parse_signing_code;
use crate::utils::*;

#[derive(Clone)]
struct ZoomCapability {
    level: f64,
    min: f64,
    max: f64,
    step: f64,
}

// 新增一个统一处理二维码数据的异步函数
async fn handle_qr_code_data(
    qr_data: String,
    mut qr_result: Signal<String>,
    mut signing_code: Signal<Option<SigningCode>>,
    mut invalid_qr_message: Signal<String>,
    mut error_message: Signal<String>,
    mut image_upload_message: Signal<String>,
) {
    if let Some(parsed_code) = parse_signing_code(&qr_data) {
        // 验证二维码是否有效
        let current_time_diff = time_diff_from_now(&parsed_code.create_time);
        
        // 检查二维码是否超过200分钟
        if current_time_diff > 200 {
            invalid_qr_message.set(format!("二维码已过期（超过{}分钟）", current_time_diff));
            image_upload_message.set(String::new());
            let _ = log_scan_result(qr_data.clone()).await;
            return;
        }
        
        // 检查数据库中是否已有相同site_id的记录
        match get_class_data(parsed_code.site_id.clone()).await {
            Ok(Some(existing_data)) => {
                // 如果数据库中存在记录，检查创建时间
                if let Some(existing_time) = existing_data.last_created_time {
                    let existing_time_diff = time_diff_from_now(&existing_time);
                    // 如果当前二维码晚于数据库中的记录，则无效
                    if current_time_diff > existing_time_diff {
                        invalid_qr_message.set(format!("二维码无效（2）"));
                        image_upload_message.set(String::new());
                        let _ = log_scan_result(qr_data.clone()).await;
                        return;
                    }
                }
            }
            Ok(None) => {
                // 数据库中没有记录，继续处理
            }
            Err(e) => {
                error_message.set(format!("数据库查询失败: {:?}", e));
                let _ = log_scan_result(qr_data.clone()).await;
                return;
            }
        }
        
        // 内容正确：保存并跳转
        qr_result.set(qr_data.clone());
        signing_code.set(Some(parsed_code.clone()));
        image_upload_message.set("二维码识别成功，正在跳转...".to_string());
        invalid_qr_message.set(String::new());
        error_message.set(String::new());

        // 异步保存扫码数据
        match save_signing_code(parsed_code.clone()).await {
            Ok(_) => {
                // 保存成功后，从数据库获取id并跳转
                match get_class_id(parsed_code.site_id.clone()).await {
                    Ok(Some(id)) => {
                        let url = format!("/i/{}", id);
                        if let Some(window) = web_sys::window() {
                            if let Err(e) = window.location().set_href(&url) {
                                web_sys::console::error_1(&format!("跳转失败: {:?}", e).into());
                            }
                        }
                    }
                    Ok(None) => {
                        error_message.set("未找到新保存的课程ID".to_string());
                    }
                    Err(e) => {
                        error_message.set(format!("获取课程ID失败: {:?}", e));
                    }
                }
            }
            Err(e) => {
                error_message.set(format!("保存失败: {:?}", e));
            }
        }
    } else {
        // 内容错误
        invalid_qr_message.set(format!("无效格式"));
        image_upload_message.set(String::new());
    }

    // 将扫描结果保存到scanlogs.txt
    let _ = log_scan_result(qr_data.clone()).await;
}

#[component]
pub fn Home() -> Element {
    let mut scanning = use_signal(|| false);
    let qr_result = use_signal(|| String::new());
    let signing_code = use_signal(|| None::<SigningCode>);
    let mut error_message = use_signal(|| String::new());
    let mut invalid_qr_message = use_signal(|| String::new());
    let mut help_message = use_signal(|| String::new());
    let mut zoom: Signal<ZoomCapability> = use_signal(|| ZoomCapability {
        level: 1.0,
        min: 1.0,
        max: 1.0,
        step: 1.0,
    });
    // 图片上传相关状态
    let image_upload_message = use_signal(|| String::new());

    // QR code detection setup - 摄像头扫描
    use_effect(move || {
        let window = web_sys::window().unwrap();
        let closure = Closure::wrap(Box::new({
            let qr_result = qr_result.clone();
            let signing_code = signing_code.clone();
            let mut scanning = scanning.clone();
            let invalid_qr_message = invalid_qr_message.clone();
            let error_message = error_message.clone();
            let image_upload_message = image_upload_message.clone();

            move |event: web_sys::Event| {
                // Use js_sys::Reflect to access custom event properties
                if let Ok(event_obj) = event.dyn_into::<js_sys::Object>() {
                    if let Ok(detail) = js_sys::Reflect::get(&event_obj, &"detail".into()) {
                        if let Ok(detail_obj) = detail.dyn_into::<js_sys::Object>() {
                            if let Ok(data) = js_sys::Reflect::get(&detail_obj, &"data".into()) {
                                if let Some(qr_data) = data.as_string() {
                                    // 验证二维码内容
                                    let qr_data_clone = qr_data.clone();
                                    let qr_result_clone = qr_result.clone();
                                    let signing_code_clone = signing_code.clone();
                                    let invalid_qr_message_clone = invalid_qr_message.clone();
                                    let error_message_clone = error_message.clone();
                                    let image_upload_message_clone = image_upload_message.clone();

                                    spawn_local(async move {
                                        handle_qr_code_data(
                                            qr_data_clone,
                                            qr_result_clone,
                                            signing_code_clone,
                                            invalid_qr_message_clone,
                                            error_message_clone,
                                            image_upload_message_clone,
                                        )
                                        .await;
                                    });

                                    // 停止扫描
                                    scanning.set(false);
                                    stop_qr_scanning();
                                }
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        window
            .add_event_listener_with_callback("qr-code-detected", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    });

    // 监听图片上传结果
    use_effect(move || {
        let window = web_sys::window().unwrap();
        let closure = Closure::wrap(Box::new({
            let qr_result = qr_result.clone();
            let signing_code = signing_code.clone();
            let mut invalid_qr_message = invalid_qr_message.clone();
            let error_message = error_message.clone();
            let mut image_upload_message = image_upload_message.clone();

            move |event: web_sys::Event| {
                if let Ok(event_obj) = event.dyn_into::<js_sys::Object>() {
                    if let Ok(detail) = js_sys::Reflect::get(&event_obj, &"detail".into()) {
                        if let Ok(result_obj) = detail.dyn_into::<js_sys::Object>() {
                            if let Ok(success) =
                                js_sys::Reflect::get(&result_obj, &"success".into())
                            {
                                if success.as_bool() == Some(true) {
                                    // 成功找到二维码
                                    if let Ok(data) =
                                        js_sys::Reflect::get(&result_obj, &"data".into())
                                    {
                                        if let Some(qr_data) = data.as_string() {
                                            // 使用统一的函数处理二维码数据
                                            let qr_data_clone = qr_data.clone();
                                            let qr_result_clone = qr_result.clone();
                                            let signing_code_clone = signing_code.clone();
                                            let invalid_qr_message_clone =
                                                invalid_qr_message.clone();
                                            let error_message_clone = error_message.clone();
                                            let image_upload_message_clone =
                                                image_upload_message.clone();

                                            spawn_local(async move {
                                                handle_qr_code_data(
                                                    qr_data_clone,
                                                    qr_result_clone,
                                                    signing_code_clone,
                                                    invalid_qr_message_clone,
                                                    error_message_clone,
                                                    image_upload_message_clone,
                                                )
                                                .await;
                                            });
                                        }
                                    }
                                } else {
                                    // 未找到二维码或其他错误
                                    if let Ok(error) =
                                        js_sys::Reflect::get(&result_obj, &"error".into())
                                    {
                                        if let Some(error_str) = error.as_string() {
                                            invalid_qr_message.set(error_str);
                                        } else {
                                            invalid_qr_message
                                                .set("图片中未找到二维码".to_string());
                                        }
                                    } else {
                                        invalid_qr_message.set("图片中未找到二维码".to_string());
                                    }
                                    image_upload_message.set(String::new());
                                }
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        window
            .add_event_listener_with_callback("image-qr-result", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    });

    let start_scanning = move |_| {
        scanning.set(true);
        error_message.set(String::new());
        invalid_qr_message.set(String::new());
        help_message.set(String::new()); // 清除帮助消息

        let mut error_message = error_message.clone();
        let mut scanning = scanning.clone();
        let mut zoom = zoom.clone();

        spawn_local(async move {
            // Wait a bit for DOM to update
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 100)
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

            // Initialize QR scanner
            if !init_qr_scanner("qr-video", "qr-canvas") {
                error_message.set("Failed to initialize QR scanner".to_string());
                scanning.set(false);
                return;
            }

            // Start scanning
            let result = start_qr_scanning().await;

            // Handle scanning result
            if let Ok(obj) = result.dyn_into::<js_sys::Object>() {
                if let Ok(success) = js_sys::Reflect::get(&obj, &"success".into()) {
                    if success.as_bool() == Some(true) {
                        error_message.set(String::new());

                        // Get camera zoom capabilities
                        let zoom_caps = get_camera_zoom_capabilities().await;
                        if let Ok(caps_obj) = zoom_caps.dyn_into::<js_sys::Object>() {
                            let mut new_zoom = zoom.read().clone();

                            if let Ok(min_val) = js_sys::Reflect::get(&caps_obj, &"min".into()) {
                                if let Some(min_f64) = min_val.as_f64() {
                                    new_zoom.min = min_f64;
                                }
                            }
                            if let Ok(max_val) = js_sys::Reflect::get(&caps_obj, &"max".into()) {
                                if let Some(max_f64) = max_val.as_f64() {
                                    new_zoom.max = max_f64;
                                }
                            }
                            if let Ok(step_val) = js_sys::Reflect::get(&caps_obj, &"step".into()) {
                                if let Some(step_f64) = step_val.as_f64() {
                                    new_zoom.step = step_f64;
                                }
                            }

                            zoom.set(new_zoom);
                        }
                    } else {
                        if let Ok(error) = js_sys::Reflect::get(&obj, &"error".into()) {
                            if let Some(error_str) = error.as_string() {
                                error_message.set(error_str);
                            }
                        }
                        scanning.set(false);
                    }
                } else {
                    error_message.set("Failed to start QR scanning".to_string());
                    scanning.set(false);
                }
            } else {
                error_message.set("Failed to start QR scanning".to_string());
                scanning.set(false);
            }
        });
    };

    let stop_scanning = move |_| {
        stop_qr_scanning();
        scanning.set(false);
        invalid_qr_message.set(String::new());
        // Reset zoom to default
        zoom.set(ZoomCapability {
            level: 1.0,
            min: 1.0,
            max: 1.0,
            step: 1.0,
        });
    };

    // 页面加载2秒后显示帮助提示
    use_effect(move || {
        let mut help_message = help_message.clone();
        spawn_local(async move {
            // 等待2秒
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 2000)
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

            // 显示帮助提示
            help_message.set("点击\"开始扫描\"按钮开始扫描".to_string());
        });
    });

    // 监听扫描状态，开始扫描2秒后显示第二个帮助提示
    use_effect(move || {
        let scanning = scanning();
        let mut help_message = help_message.clone();

        if scanning {
            spawn_local(async move {
                // 等待2秒
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    web_sys::window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 2000)
                        .unwrap();
                });
                wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

                // 显示扫描提示
                help_message.set("扫描成功后，会有提示".to_string());
            });
        }
    });

    rsx! {
        div { id: "home", class: "home-container",
            div { class: "home-scanner-container",

                p { class: "home-title", "Scan BUPT Checkcode Everywhere" }

                CameraPreview { scanning, zoom }

                ScanControls {
                    scanning,
                    on_start: start_scanning,
                    on_stop: stop_scanning,
                }

                MessageDisplay {
                    qr_result,
                    signing_code,
                    invalid_qr_message,
                    error_message,
                    help_message,
                    image_upload_message,
                }
            }
        }
    }
}

// 缩放控制组件
#[component]
fn ZoomControls(zoom: Signal<ZoomCapability>) -> Element {
    let zoom_capability = zoom();

    if zoom_capability.max <= zoom_capability.min {
        return rsx! {
            div {}
        };
    }

    let handle_zoom_in = move |_| {
        let mut current_zoom = zoom();
        let new_level = (current_zoom.level + current_zoom.step).min(current_zoom.max);
        if new_level != current_zoom.level {
            current_zoom.level = new_level;
            zoom.set(current_zoom);
            spawn_local(async move {
                let _ = set_camera_zoom(new_level).await;
            });
        }
    };

    let handle_zoom_out = move |_| {
        let mut current_zoom = zoom();
        let new_level = (current_zoom.level - current_zoom.step).max(current_zoom.min);
        if new_level != current_zoom.level {
            current_zoom.level = new_level;
            zoom.set(current_zoom);
            spawn_local(async move {
                let _ = set_camera_zoom(new_level).await;
            });
        }
    };

    rsx! {
        div { class: "home-zoom-controls",

            button {
                onclick: handle_zoom_in,
                disabled: zoom_capability.level >= zoom_capability.max,
                class: "home-zoom-button",
                "+"
            }

            div { class: "home-zoom-level", "{zoom_capability.level} x" }

            button {
                onclick: handle_zoom_out,
                disabled: zoom_capability.level <= zoom_capability.min,
                class: "home-zoom-button",
                "−"
            }
        }
    }
}

// 相机预览组件
#[component]
fn CameraPreview(scanning: Signal<bool>, zoom: Signal<ZoomCapability>) -> Element {
    rsx! {
        div { class: if scanning() { "home-camera-preview scanning" } else { "home-camera-preview" },

            video {
                id: "qr-video",
                autoplay: true,
                muted: true,
                playsinline: true,
                class: "home-video",
            }

            canvas { id: "qr-canvas", style: "display: none;" }

            // 扫描框 - 始终显示
            div { class: "home-scanner-corners",

                // 四个角的装饰 - 始终显示
                div { class: "home-scanner-corner top-left" }
                div { class: "home-scanner-corner top-right" }
                div { class: "home-scanner-corner bottom-left" }
                div { class: "home-scanner-corner bottom-right" }

                // 动态扫描线 - 仅在扫描时显示
                if scanning() {
                    div { class: "home-scanner-line" }
                }
            }

            if scanning() {
                ZoomControls { zoom }
            }
        }
    }
}

// 扫描控制按钮组件
#[component]
fn ScanControls(
    scanning: Signal<bool>,
    on_start: EventHandler<()>,
    on_stop: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "home-scanner-controls",

            if !scanning() {
                div {
                    // 添加图片上传按钮
                    div { class: "button-group",
                        input {
                            r#type: "file",
                            id: "image-upload",
                            accept: "image/*",
                            style: "display: none;",
                            oninput: move |_| {
                                let js_code = format!(
                                    r#"
                                                                                    const input = document.getElementById('image-upload');
                                                                                    const file = input.files[0];
                                                                                    if (file) {{
                                                                                        window.handleImageUpload(file);
                                                                                    }}
                                                                                "#,
                                );
                                let _ = js_sys::eval(&js_code);
                            },
                        }
                        button {
                            onclick: move |_| on_start.call(()),
                            class: "home-button-primary",
                            "开始扫描"
                        }

                        button {
                            onclick: move |_| {
                                let js_code = "document.getElementById('image-upload').click();";
                                let _ = js_sys::eval(js_code);
                            },
                            class: "home-button-primary",
                            "上传图片"
                        }
                    }
                }
            } else {
                button {
                    onclick: move |_| on_stop.call(()),
                    class: "home-button-danger",
                    "停止扫描"
                }
            }
        }
    }
}

// 消息显示组件
#[component]
fn MessageDisplay(
    qr_result: Signal<String>,
    signing_code: Signal<Option<SigningCode>>,
    invalid_qr_message: Signal<String>,
    error_message: Signal<String>,
    help_message: Signal<String>,
    image_upload_message: Signal<String>,
) -> Element {
    rsx! {
        div {
            // 图片上传消息
            if !image_upload_message().is_empty() {
                div { class: "home-info-message",
                    h4 { "{image_upload_message()}" }
                }
            }

            // 成功消息
            if !qr_result().is_empty() {
                div { class: "home-success-message",
                    if signing_code().is_some() {
                        div {
                            h4 { "扫描成功，正在跳转..." }
                        }
                    }
                }
            }

            // 无效二维码消息
            if !invalid_qr_message().is_empty() {
                div { class: "home-warning-message",
                    h4 { "{invalid_qr_message()}" }
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
        }
    }
}
