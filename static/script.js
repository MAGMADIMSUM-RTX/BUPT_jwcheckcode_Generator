const video = document.getElementById('video');
const startBtn = document.getElementById('startBtn');
const stopBtn = document.getElementById('stopBtn');
const zoomInBtn = document.getElementById('zoomInBtn');
const zoomOutBtn = document.getElementById('zoomOutBtn');
const zoomLevel = document.getElementById('zoomLevel');
const status = document.getElementById('status');

// 二维码生成相关的变量和元素
const qrCodeImage = document.getElementById('qrCodeImage');
const qrCodeLoading = document.getElementById('qrCodeLoading');
const qrStatus = document.getElementById('qrStatus');
const generateBtn = document.getElementById('generateBtn');

// 自动刷新相关变量
let autoRefreshInterval = null;
let isAutoGenerating = false;

// 标签页相关元素
const tabBtns = document.querySelectorAll('.tab-btn');
const tabContents = document.querySelectorAll('.tab-content');

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
    updateStatus('点击开始扫描', 'info');
    
    // 初始化标签页
    initTabs();
    
    // 检查二维码状态并决定默认标签页
    await checkQRStatusAndSetDefaultTab();
}

// 检查二维码状态并设置默认标签页
async function checkQRStatusAndSetDefaultTab() {
    try {
        const response = await fetch('/api/qr-data');
        const data = await response.json();
        
        if (response.ok) {
            // 如果能成功获取二维码数据，说明有有效的扫描记录，默认显示生成器页面
            switchTab('generator');
        } else {
            // 如果获取失败（未扫描或已过期），默认显示扫描器页面
            switchTab('scanner');
        }
    } catch (error) {
        // 网络错误或其他错误，默认显示扫描器页面
        switchTab('scanner');
    }
}

// 标签页切换功能
function initTabs() {
    tabBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            const targetTab = btn.dataset.tab;
            switchTab(targetTab);
        });
    });
}

function switchTab(tabName) {
    // 切换按钮状态
    tabBtns.forEach(btn => {
        btn.classList.remove('active');
        if (btn.dataset.tab === tabName) {
            btn.classList.add('active');
        }
    });
    
    // 切换内容显示
    tabContents.forEach(content => {
        content.classList.remove('active');
        if (content.id === `${tabName}-tab`) {
            content.classList.add('active');
        }
    });
    
    // 如果切换到生成器页面，开始自动生成
    if (tabName === 'generator') {
        startAutoGeneration();
    } else {
        // 如果切换到其他页面，停止自动生成
        stopAutoGeneration();
    }
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
                updateStatus(`扫描成功！`, 'success');
                
                try {
                    const response = await fetch('/api/qr-code', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ content: qrText })
                    });
                    
                    const data = await response.json();
                    updateStatus(response.ok ? data.message : '发送失败', response.ok ? 'success' : 'error');
                } catch (err) {
                    updateStatus('网络错误', 'error');
                }
                
                setTimeout(() => {
                    if (scanning) updateStatus('继续扫描...', 'info');
                }, 2000);
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
    
    track = null;
    currentZoom = 1; // 重置为默认值
    video.srcObject = null;
    startBtn.disabled = false;
    stopBtn.disabled = true;
    
    // 禁用缩放控制
    disableZoomControls();
    
    updateStatus('已停止扫描', 'info');
}

// 缩放功能
function enableZoomControls() {
    if (!track || !track.getCapabilities) return;
    
    const capabilities = track.getCapabilities();
    if (capabilities.zoom) {
        console.log(`缩放范围: ${capabilities.zoom.min}x - ${capabilities.zoom.max}x`);
        updateZoomButtons();
        updateZoomDisplay();
    }
}

function updateZoomButtons() {
    if (!track || !track.getCapabilities) return;
    
    const capabilities = track.getCapabilities();
    if (capabilities.zoom) {
        const minZoom = capabilities.zoom.min;
        const maxZoom = capabilities.zoom.max;
        
        // 根据当前缩放级别和设备能力启用/禁用按钮
        zoomOutBtn.disabled = (currentZoom <= minZoom);
        zoomInBtn.disabled = (currentZoom >= maxZoom);
    }
}

function disableZoomControls() {
    zoomInBtn.disabled = true;
    zoomOutBtn.disabled = true;
    zoomLevel.textContent = '1x';
}

async function zoomIn() {
    if (!track || !track.getCapabilities) return;
    
    const capabilities = track.getCapabilities();
    if (capabilities.zoom) {
        const maxZoom = capabilities.zoom.max;
        const step = 1.0; // 固定步长为1倍
        currentZoom = Math.min(maxZoom, currentZoom + step);
        
        try {
            await track.applyConstraints({
                advanced: [{ zoom: currentZoom }]
            });
            updateZoomDisplay();
        } catch (err) {
            console.error('缩放失败:', err);
        }
    }
}

async function zoomOut() {
    if (!track || !track.getCapabilities) return;
    
    const capabilities = track.getCapabilities();
    if (capabilities.zoom) {
        const minZoom = capabilities.zoom.min;
        const step = 1.0; // 固定步长为1倍
        currentZoom = Math.max(minZoom, currentZoom - step);
        
        try {
            await track.applyConstraints({
                advanced: [{ zoom: currentZoom }]
            });
            updateZoomDisplay();
        } catch (err) {
            console.error('缩放失败:', err);
        }
    }
}

function updateZoomDisplay() {
    zoomLevel.textContent = `${currentZoom.toFixed(1)}x`;
    updateZoomButtons(); // 同时更新按钮状态
}

// 获取二维码数据
async function fetchQRData() {
    try {
        const response = await fetch('/api/qr-data');
        const data = await response.json();
        
        if (!response.ok) {
            // 如果服务器返回错误（如未扫描过二维码或已过期）
            throw new Error(data.message || `HTTP error! status: ${response.status}`);
        }
        
        return data.content;
    } catch (error) {
        console.error('获取二维码数据失败:', error);
        throw error; // 重新抛出错误让调用者处理
    }
}

// 生成二维码
async function generateQRCode() {
    if (!isAutoGenerating) {
        generateBtn.disabled = true;
    }
    qrCodeLoading.style.display = 'block';
    qrCodeImage.style.display = 'none';
    qrStatus.textContent = '正在获取数据...';
    
    try {
        const content = await fetchQRData();
        if (content) {
            const qrUrl = `https://api.2dcode.biz/v1/create-qr-code?data=${encodeURIComponent(content)}&size=128x128`;
            
            qrCodeImage.onload = () => {
                qrCodeLoading.style.display = 'none';
                qrCodeImage.style.display = 'block';
                qrStatus.textContent = isAutoGenerating ? '自动刷新中...' : '二维码已生成';
                if (!isAutoGenerating) {
                    generateBtn.disabled = false;
                }
            };
            
            qrCodeImage.onerror = () => {
                qrCodeLoading.style.display = 'none';
                qrStatus.textContent = '二维码生成失败';
                if (!isAutoGenerating) {
                    generateBtn.disabled = false;
                }
            };
            
            qrCodeImage.src = qrUrl;
        }
    } catch (error) {
        qrCodeLoading.style.display = 'none';
        qrStatus.textContent = error.message || '获取数据失败';
        if (!isAutoGenerating) {
            generateBtn.disabled = false;
        }
        // 如果是自动生成模式且出现错误（如过期），停止自动生成
        if (isAutoGenerating && (error.message.includes('过期') || error.message.includes('重新扫描'))) {
            stopAutoGeneration();
        }
    }
}

// 开始自动生成
function startAutoGeneration() {
    if (isAutoGenerating) return;
    
    isAutoGenerating = true;
    generateBtn.disabled = true;
    generateBtn.textContent = '自动生成中...';
    
    // 立即生成一次
    generateQRCode();
    
    // 设置每1秒自动刷新
    autoRefreshInterval = setInterval(() => {
        if (isAutoGenerating) {
            generateQRCode();
        }
    }, 2000);
}

// 停止自动生成
function stopAutoGeneration() {
    if (!isAutoGenerating) return;
    
    isAutoGenerating = false;
    generateBtn.disabled = false;
    generateBtn.textContent = '手动生成二维码';
    
    if (autoRefreshInterval) {
        clearInterval(autoRefreshInterval);
        autoRefreshInterval = null;
    }
    
    // 隐藏二维码并重置状态
    qrCodeImage.style.display = 'none';
    qrCodeLoading.style.display = 'block';
    qrCodeLoading.textContent = '自动生成中...';
    qrStatus.textContent = '进入此页面将自动生成二维码';
}

// 事件监听器
startBtn.addEventListener('click', startScanning);
stopBtn.addEventListener('click', stopScanning);
zoomInBtn.addEventListener('click', zoomIn);
zoomOutBtn.addEventListener('click', zoomOut);
generateBtn.addEventListener('click', generateQRCode);

// 页面卸载时清理资源
window.addEventListener('beforeunload', () => {
    if (scanning) stopScanning();
    stopAutoGeneration();
});
