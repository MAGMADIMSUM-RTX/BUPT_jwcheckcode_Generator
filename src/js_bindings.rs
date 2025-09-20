use wasm_bindgen::prelude::*;

// Import JavaScript functions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = initQRScanner)]
    pub fn init_qr_scanner(video_id: &str, canvas_id: &str) -> bool;

    #[wasm_bindgen(js_name = startQRScanning)]
    pub async fn start_qr_scanning() -> JsValue;

    #[wasm_bindgen(js_name = stopQRScanning)]
    pub fn stop_qr_scanning();

    #[wasm_bindgen(js_name = getCameraZoomCapabilities)]
    pub async fn get_camera_zoom_capabilities() -> JsValue;

    #[wasm_bindgen(js_name = setCameraZoom)]
    pub async fn set_camera_zoom(zoom: f64) -> JsValue;

    #[wasm_bindgen(js_name = scanQRFromImage)]
    pub async fn scan_qr_from_image(file: web_sys::File) -> JsValue;
}
