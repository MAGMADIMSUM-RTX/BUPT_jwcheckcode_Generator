const video = document.getElementById('video');
const startBtn = document.getElementById('startBtn');
const stopBtn = document.getElementById('stopBtn');
const zoomInBtn = document.getElementById('zoomInBtn');
const zoomOutBtn = document.getElementById('zoomOutBtn');
const zoomLevel = document.getElementById('zoomLevel');
const status = document.getElementById('status');

let codeReader = null;
let scanning = false;
let stream = null;
let currentZoom = 1;
let track = null;

// 页面加载时初始化
window.addEventListener('DOMContentLoaded', function() {
    initializeApp();
});

async function initializeApp() {
    // 检查基本支持
    if (!navigator.mediaDevices || typeof ZXing === 'undefined') {
        updateStatus('浏览器不支持或二维码库未加载', 'error');
        return;
    }
    updateStatus('等待扫描', 'info');
}

function updateStatus(message, type = 'info') {
    status.textContent = message;
    status.className = `status ${type}`;
}

async function startScanning() {
    try {
        updateStatus('正在启动摄像头...', 'info');
        
        if (!navigator.mediaDevices?.getUserMedia) {
            throw new Error('浏览器不支持摄像头访问');
        }
        
        codeReader = new ZXing.BrowserQRCodeReader();
        
        const constraints = {
            video: { 
                facingMode: { ideal: 'environment' },
                width: { ideal: 1280 },
                height: { ideal: 720 }
            }
        };
        
        stream = await navigator.mediaDevices.getUserMedia(constraints);
        track = stream.getVideoTracks()[0];
        
        // 初始化缩放值为设备的最小缩放值
        if (track && track.getCapabilities) {
            const capabilities = track.getCapabilities();
            if (capabilities.zoom) {
                currentZoom = capabilities.zoom.min;
            }
        }
        
        video.srcObject = stream;
        scanning = true;
        startBtn.disabled = true;
        stopBtn.disabled = false;
        
        // 启用缩放控制
        enableZoomControls();
        
        updateStatus('扫描中...', 'info');
        
        codeReader.decodeFromVideoDevice(null, video, async (result, error) => {
            if (result) {
                const qrText = result.getText();
                updateStatus(`扫描成功！正在处理...`, 'success');
                
                try {
                    const response = await fetch('/api/qr-code', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ content: qrText })
                    });
                    
                    const data = await response.json();
                    
                    if (response.ok && data.status === 'success') {
                        // 扫描成功，跳转到生成页面
                        updateStatus('等待跳转...', 'success');
                        setTimeout(() => {
                            window.location.href = data.message; // data.message 包含跳转 URL
                        }, 1000);
                    } else {
                        updateStatus(data.message || '二维码格式不正确', 'error');
                        setTimeout(() => {
                            if (scanning) updateStatus('继续扫描...', 'info');
                        }, 2000);
                    }
                } catch (err) {
                    updateStatus('网络错误', 'error');
                    setTimeout(() => {
                        if (scanning) updateStatus('继续扫描...', 'info');
                    }, 2000);
                }
            }
        });
        
    } catch (err) {
        console.error('摄像头启动失败:', err);
        let errorMessage = '摄像头启动失败';
        
        if (err.name === 'NotAllowedError') {
            errorMessage = '摄像头权限被拒绝';
        } else if (err.name === 'NotFoundError') {
            errorMessage = '未找到摄像头';
        } else if (err.name === 'NotReadableError') {
            errorMessage = '摄像头被占用';
        }
        
        updateStatus(errorMessage, 'error');
        startBtn.disabled = false;
        stopBtn.disabled = true;
    }
}

function stopScanning() {
    scanning = false;
    
    if (codeReader) {
        codeReader.reset();
        codeReader = null;
    }
    
    if (stream) {
        stream.getTracks().forEach(track => track.stop());
        stream = null;
    }
    
    video.srcObject = null;
    track = null;
    
    startBtn.disabled = false;
    stopBtn.disabled = true;
    
    // 禁用缩放控制
    disableZoomControls();
    
    updateStatus('扫描已停止', 'info');
}

function enableZoomControls() {
    if (!track || !track.getCapabilities) return;
    
    const capabilities = track.getCapabilities();
    if (!capabilities.zoom) return;
    
    zoomInBtn.disabled = false;
    zoomOutBtn.disabled = false;
    updateZoomDisplay();
}

function disableZoomControls() {
    zoomInBtn.disabled = true;
    zoomOutBtn.disabled = true;
    currentZoom = 1;
    updateZoomDisplay();
}

function updateZoomDisplay() {
    zoomLevel.textContent = `${currentZoom.toFixed(1)}x`;
}

async function adjustZoom(factor) {
    if (!track || !track.getCapabilities) return;
    
    const capabilities = track.getCapabilities();
    if (!capabilities.zoom) return;
    
    const newZoom = Math.max(capabilities.zoom.min, 
                            Math.min(capabilities.zoom.max, currentZoom * factor));
    
    try {
        await track.applyConstraints({
            advanced: [{ zoom: newZoom }]
        });
        currentZoom = newZoom;
        updateZoomDisplay();
    } catch (err) {
        console.error('缩放调整失败:', err);
    }
}

// 事件监听器
startBtn.addEventListener('click', startScanning);
stopBtn.addEventListener('click', stopScanning);
zoomInBtn.addEventListener('click', () => adjustZoom(1.2));
zoomOutBtn.addEventListener('click', () => adjustZoom(0.8));
