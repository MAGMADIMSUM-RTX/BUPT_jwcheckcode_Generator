const qrCodeImage = document.getElementById('qrCodeImage');
const qrCodeImageOld = document.getElementById('qrCodeImageOld');
const qrCodeLoading = document.getElementById('qrCodeLoading');
const qrStatus = document.getElementById('qrStatus');
const generateBtn = document.getElementById('generateBtn');
const copyLinkBtn = document.getElementById('copyLinkBtn');
const backBtn = document.getElementById('backBtn');
const courseName = document.getElementById('courseName');
const courseId = document.getElementById('courseId');

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
        
        // 加载课程名称
        await loadCourseName(classLessonId);
        
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

// 加载课程名称
async function loadCourseName(classLessonId) {
    try {
        const response = await fetch(`/api/class-name/${classLessonId}`);
        const data = await response.json();
        
        if (response.ok && data.class_name) {
            courseName.textContent = data.class_name;
            // courseId.textContent = `课程ID: ${classLessonId}`;
        } else {
            courseName.textContent = `课程${classLessonId}`;
        }
    } catch (error) {
        console.error('获取课程名称失败:', error);
        courseName.textContent = `课程${classLessonId}`;
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
            
            // 加载新图像
            const newImage = new Image();
            newImage.onload = function() {
                // 只有在新图像加载成功后才进行切换
                // 将当前图像移动到旧图像位置（如果存在）
                if (qrCodeImage.src && qrCodeImage.style.display !== 'none') {
                    qrCodeImageOld.src = qrCodeImage.src;
                    qrCodeImageOld.style.display = 'block';
                    qrCodeImageOld.style.opacity = '0.8';
                }
                
                // 显示新图像
                qrCodeImage.src = qrUrl;
                qrCodeImage.style.display = 'block';
                qrCodeImage.style.opacity = '1';
                hideLoading();
                updateQRStatus('二维码生成成功', 'success');
                
                // 500毫秒后开始淡出旧图像，1秒后完全隐藏
                setTimeout(() => {
                    if (qrCodeImageOld.style.display !== 'none') {
                        qrCodeImageOld.style.opacity = '0';
                        setTimeout(() => {
                            qrCodeImageOld.style.display = 'none';
                        }, 300);
                    }
                }, 500);
            };
            newImage.onerror = function() {
                hideLoading();
                updateQRStatus('图像加载失败', 'error');
            };
            newImage.src = qrUrl;
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
    // 如果当前已有二维码图片，则保持显示，不显示loading文字
    if (qrCodeImage.src && qrCodeImage.style.display !== 'none') {
        // 保持当前二维码显示，不显示loading
        qrCodeLoading.style.display = 'none';
    } else {
        // 只有在没有二维码图片时才显示loading文字
        qrCodeLoading.style.display = 'block';
        qrCodeImage.style.display = 'none';
    }
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
