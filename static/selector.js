// 页面元素
const loadingIndicator = document.getElementById('loadingIndicator');
const classList = document.getElementById('classList');
const emptyState = document.getElementById('emptyState');
const refreshBtn = document.getElementById('refreshBtn');
const scanBtn = document.getElementById('scanBtn');

// 页面加载时初始化
window.addEventListener('DOMContentLoaded', function() {
    loadAllData(); // 统一加载所有数据
});

// 加载所有课程数据
async function loadAllData() {
    showLoading();
    
    try {
        // 只从数据库加载所有课程
        const response = await fetch('/api/all-courses');
        const data = await response.json();
        
        if (response.ok && data.status === 'success' && data.courses && data.courses.length > 0) {
            // 将数据库课程转换为统一格式
            const courses = data.courses.map(course => ({
                class_lesson_id: course.class_lesson_id,
                class_name: course.lesson_name,
                is_expired: course.is_expired,
                scan_timestamp: course.last_create_time || '暂无记录'
            }));
            
            console.log(`加载了 ${courses.length} 个课程`);
            displayCourseList(courses);
        } else {
            showEmptyState();
        }
        
    } catch (error) {
        console.error('加载数据失败:', error);
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
function displayCourseList(courses) {
    loadingIndicator.style.display = 'none';
    emptyState.style.display = 'none';
    classList.style.display = 'block';
    
    // 清空现有内容
    classList.innerHTML = '';
    
    // 按课程名称排序
    courses.sort((a, b) => {
        const nameA = a.class_name || 'unknown';
        const nameB = b.class_name || 'unknown';
        return nameA.localeCompare(nameB);
    });
    
    // 为每个课程创建列表项
    courses.forEach(course => {
        const courseItem = createCourseItem(course);
        classList.appendChild(courseItem);
    });
}

// 创建课程项
function createCourseItem(course) {
    const isExpired = course.is_expired;
    
    const courseItem = document.createElement('div');
    courseItem.className = `class-item ${isExpired ? 'expired' : ''}`;
    
    // 确定状态文字和样式
    let statusText = isExpired ? '已过期' : '可用';
    let statusClass = isExpired ? 'status-expired' : 'status-active';
    
    // 确定详细信息
    let detailsHtml = '';
    if (isExpired) {
        // 过期课程显示最后更新时间
        if (course.scan_timestamp && course.scan_timestamp !== '暂无记录') {
            detailsHtml = `最后更新: ${formatDateTime(course.scan_timestamp)}`;
        } else {
            detailsHtml = '暂无扫描记录';
        }
    } else {
        // 可用课程显示剩余有效时间
        const remainingSeconds = calculateRemainingTime(course.scan_timestamp);
        if (remainingSeconds !== null && remainingSeconds > 0) {
            detailsHtml = `剩余有效时间: ${formatRemainingTime(remainingSeconds)}`;
        } else if (course.scan_timestamp && course.scan_timestamp !== '暂无记录') {
            detailsHtml = `最后更新: ${formatDateTime(course.scan_timestamp)}`;
        } else {
            detailsHtml = '暂无扫描记录';
        }
    }
    
    courseItem.innerHTML = `
        <div class="class-info">
            <div class="class-id">${course.class_name || 'unknown'}</div>
            <div class="class-details">
                ${detailsHtml}
            </div>
        </div>
        <div class="class-status ${statusClass}">
            ${statusText}
        </div>
    `;
    
    // 添加点击事件 - 所有课程都可以直接跳转到生成页面
    if (!isExpired) {
        courseItem.addEventListener('click', () => {
            navigateToGenerator(course.class_lesson_id);
        });
    }
    
    return courseItem;
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

// 课程过期时间（分钟）
const COURSE_EXPIRY_MINUTES = 20;

// 计算剩余有效时间
function calculateRemainingTime(lastCreateTime) {
    if (!lastCreateTime || lastCreateTime === '暂无记录') {
        return null;
    }
    
    try {
        // 解析时间戳，处理多种格式
        let createTime;
        if (lastCreateTime.includes('T')) {
            // ISO 格式，假设为北京时间
            if (lastCreateTime.includes('+') || lastCreateTime.includes('Z')) {
                // 已有时区信息
                createTime = new Date(lastCreateTime);
            } else {
                // 没有时区信息，假设为北京时间 (UTC+8)
                createTime = new Date(lastCreateTime + '+08:00');
            }
        } else {
            // "YYYY-MM-DD HH:MM:SS" 格式，假设为北京时间
            createTime = new Date(lastCreateTime.replace(' ', 'T') + '+08:00');
        }
        
        // 当前时间
        const now = new Date();
        const validUntil = new Date(createTime.getTime() + COURSE_EXPIRY_MINUTES * 60 * 1000);
        
        const remainingMs = validUntil.getTime() - now.getTime();
        
        if (remainingMs <= 0) {
            return 0; // 已过期
        }
        
        return Math.floor(remainingMs / 1000); // 返回剩余秒数
    } catch (error) {
        console.error('解析时间失败:', error);
        return null;
    }
}

// 格式化剩余时间
function formatRemainingTime(seconds) {
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

// 选择课程 - 直接跳转到生成页面
function selectCourse(classLessonId) {
    window.location.href = `/gencode/classid/${classLessonId}`;
}

// 事件监听器
refreshBtn.addEventListener('click', () => {
    loadAllData();
});

scanBtn.addEventListener('click', () => {
    window.location.href = '/';
});

// 自动刷新（每30秒）
setInterval(() => {
    loadAllData();
}, 30000);
