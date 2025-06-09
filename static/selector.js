// 页面元素
const loadingIndicator = document.getElementById('loadingIndicator');
const classList = document.getElementById('classList');
const emptyState = document.getElementById('emptyState');
const refreshBtn = document.getElementById('refreshBtn');
const scanBtn = document.getElementById('scanBtn');

// 页面加载时初始化
window.addEventListener('DOMContentLoaded', function() {
    loadClassList();
});

// 加载课程列表
async function loadClassList() {
    showLoading();
    
    try {
        const response = await fetch('/api/class-list');
        const data = await response.json();
        
        if (response.ok && data.classes && data.classes.length > 0) {
            displayClassList(data.classes);
        } else {
            showEmptyState();
        }
    } catch (error) {
        console.error('加载课程列表失败:', error);
        showEmptyState();
    }
}

// 显示加载状态
function showLoading() {
    loadingIndicator.style.display = 'block';
    classList.style.display = 'none';
    emptyState.style.display = 'none';
}

// 显示空状态
function showEmptyState() {
    loadingIndicator.style.display = 'none';
    classList.style.display = 'none';
    emptyState.style.display = 'block';
}

// 显示课程列表
function displayClassList(classes) {
    loadingIndicator.style.display = 'none';
    emptyState.style.display = 'none';
    classList.style.display = 'block';
    
    // 清空现有内容
    classList.innerHTML = '';
    
    // 为每个课程创建列表项
    classes.forEach(classData => {
        const classItem = createClassItem(classData);
        classList.appendChild(classItem);
    });
}

// 创建课程列表项
function createClassItem(classData) {
    const isExpired = classData.is_expired;
    const timeRemaining = classData.time_remaining;
    
    const classItem = document.createElement('div');
    classItem.className = `class-item ${isExpired ? 'expired' : ''}`;
    
    classItem.innerHTML = `
        <div class="class-info">
            <div class="class-id">${classData.class_name}</div>
            <div class="class-details">
                课程ID: ${classData.class_lesson_id}<br>
                ${isExpired ? '已过期' : `剩余有效时间: ${formatTimeRemaining(timeRemaining)}`}
            </div>
        </div>
        <div class="class-status ${isExpired ? 'status-expired' : 'status-active'}">
            ${isExpired ? '已过期' : '有效'}
        </div>
    `;
    
    // 添加点击事件
    if (!isExpired) {
        classItem.addEventListener('click', () => {
            navigateToGenerator(classData.class_lesson_id);
        });
    }
    
    return classItem;
}

// 格式化日期时间
function formatDateTime(timestamp) {
    const date = new Date(timestamp);
    return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
    });
}

// 格式化剩余时间
function formatTimeRemaining(seconds) {
    if (seconds <= 0) return '已过期';
    
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    
    if (minutes > 0) {
        return `${minutes}分${remainingSeconds}秒`;
    } else {
        return `${remainingSeconds}秒`;
    }
}

// 跳转到二维码生成页面
function navigateToGenerator(classLessonId) {
    window.location.href = `/gencode/classid/${classLessonId}`;
}

// 事件监听器
refreshBtn.addEventListener('click', () => {
    loadClassList();
});

scanBtn.addEventListener('click', () => {
    window.location.href = '/';
});

// 自动刷新（每30秒）
setInterval(() => {
    loadClassList();
}, 30000);
