const qrCodeImage = document.getElementById('qrCodeImage');
const qrCodeLoading = document.getElementById('qrCodeLoading');
const qrStatus = document.getElementById('qrStatus');
const generateBtn = document.getElementById('generateBtn');
const copyLinkBtn = document.getElementById('copyLinkBtn');
const backBtn = document.getElementById('backBtn');

// 自动刷新相关变量
let autoRefreshInterval = null;
let isAutoGenerating = false;

// 页面加载时初始化
window.addEventListener('DOMContentLoaded', function() {
    initializeApp();
});

async function initializeApp() {
    // 从URL中提取class_lesson_id
    const pathParts = window.location.pathname.split('/');
    if (pathParts.length >= 4 && pathParts[1] === 'gencode' && pathParts[2] === 'classid') {
        const classLessonId = pathParts[3];
        
        // 自动生成二维码
        await generateQRCode(classLessonId);
        
        // 设置定时刷新（每2秒）
        autoRefreshInterval = setInterval(async () => {
            if (!isAutoGenerating) {
                await generateQRCode(classLessonId);
            }
        }, 2000);
    } else {
        // URL格式不正确，跳转到扫描页面
        updateQRStatus('URL格式错误，正在跳转...', 'error');
        setTimeout(() => {
            window.location.href = '/';
        }, 2000);
    }
}

async function generateQRCode(classLessonId) {
    if (isAutoGenerating) return;
    
    isAutoGenerating = true;
    showLoading();
    
    try {
        const response = await fetch(`/api/qr-data/${classLessonId}`);
        const data = await response.json();
        
        if (response.ok && data.content) {
            // 生成二维码图片
            const qrUrl = `https://api.qrserver.com/v1/create-qr-code/?size=300x300&data=${encodeURIComponent(data.content)}`;
            
            qrCodeImage.src = qrUrl;
            qrCodeImage.style.display = 'block';
            hideLoading();
            updateQRStatus('二维码生成成功', 'success');
        } else {
            hideLoading();
            updateQRStatus(data.message || '生成失败', 'error');
            
            // 如果是过期或无数据，3秒后跳转到扫描页面
            if (data.message && (data.message.includes('过期') || data.message.includes('扫描'))) {
                setTimeout(() => {
                    window.location.href = '/';
                }, 1500);
            }
        }
    } catch (error) {
        console.error('生成二维码失败:', error);
        hideLoading();
        updateQRStatus('网络错误', 'error');
    }
    
    isAutoGenerating = false;
}

function showLoading() {
    qrCodeLoading.style.display = 'block';
    qrCodeImage.style.display = 'none';
}

function hideLoading() {
    qrCodeLoading.style.display = 'none';
}

function updateQRStatus(message, type = 'info') {
    qrStatus.textContent = message;
    qrStatus.className = `qr-status ${type}`;
}

// 复制当前链接功能
async function copyCurrentLink() {
    try {
        const currentUrl = window.location.href;
        
        // 使用现代的 Clipboard API
        if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(currentUrl);
            updateQRStatus('链接已复制到剪贴板', 'success');
        } else {
            // 降级方案：使用传统的复制方法
            const textArea = document.createElement('textarea');
            textArea.value = currentUrl;
            textArea.style.position = 'fixed';
            textArea.style.left = '-999999px';
            textArea.style.top = '-999999px';
            document.body.appendChild(textArea);
            textArea.focus();
            textArea.select();
            
            try {
                document.execCommand('copy');
                updateQRStatus('链接已复制到剪贴板', 'success');
            } catch (err) {
                updateQRStatus('复制失败，请手动复制', 'error');
            }
            
            document.body.removeChild(textArea);
        }
        
        // 3秒后恢复状态显示
        setTimeout(() => {
            updateQRStatus('二维码生成成功', 'success');
        }, 3000);
        
    } catch (err) {
        console.error('复制链接失败:', err);
        updateQRStatus('复制失败，请手动复制', 'error');
        
        // 3秒后恢复状态显示
        setTimeout(() => {
            updateQRStatus('二维码生成成功', 'success');
        }, 3000);
    }
}

// 手动生成按钮事件
generateBtn.addEventListener('click', async () => {
    // 从URL中提取class_lesson_id
    const pathParts = window.location.pathname.split('/');
    if (pathParts.length >= 4 && pathParts[1] === 'gencode' && pathParts[2] === 'classid') {
        const classLessonId = pathParts[3];
        await generateQRCode(classLessonId);
    }
});

// 复制链接按钮事件
copyLinkBtn.addEventListener('click', async () => {
    await copyCurrentLink();
});

// 返回扫描按钮事件
backBtn.addEventListener('click', () => {
    // 清除定时器
    if (autoRefreshInterval) {
        clearInterval(autoRefreshInterval);
        autoRefreshInterval = null;
    }
    
    // 跳转到扫描页面
    window.location.href = '/';
});

// 页面卸载时清除定时器
window.addEventListener('beforeunload', () => {
    if (autoRefreshInterval) {
        clearInterval(autoRefreshInterval);
        autoRefreshInterval = null;
    }
});
