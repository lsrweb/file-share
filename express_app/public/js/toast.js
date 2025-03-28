/**
 * Toast 消息提示类
 * 提供非阻塞式的消息提示功能，支持成功、错误、警告和信息等不同类型的提示
 */
class Toast {
    /**
     * 构造函数，初始化 Toast 容器
     */
    constructor() {
        // 创建 Toast 容器，如果已存在则使用现有容器
        this.container = document.querySelector('.toast-container');
        if (!this.container) {
            this.container = document.createElement('div');
            this.container.className = 'toast-container';
            document.body.appendChild(this.container);
        }
    }

    /**
     * 显示 Toast 消息
     * @param {string} message 消息内容
     * @param {string} type 消息类型: 'success', 'error', 'warning', 'info'
     * @param {number} duration 持续时间（毫秒），默认 3000ms
     */
    show(message, type = 'info', duration = 3000) {
        // 创建 Toast 元素
        const toast = document.createElement('div');
        toast.className = `toast ${type}`;

        // 创建图标
        const icon = document.createElement('span');
        icon.className = 'toast-icon';
        switch (type) {
            case 'success':
                icon.innerHTML = '✓';
                break;
            case 'error':
                icon.innerHTML = '✗';
                break;
            case 'warning':
                icon.innerHTML = '!';
                break;
            default: // info
                icon.innerHTML = 'i';
        }

        // 创建消息区域
        const messageElement = document.createElement('div');
        messageElement.className = 'toast-message';
        messageElement.textContent = message;

        // 将图标和消息添加到 Toast 元素
        toast.appendChild(icon);
        toast.appendChild(messageElement);

        // 将 Toast 添加到容器
        this.container.appendChild(toast);

        // 使用 setTimeout 触发动画
        setTimeout(() => {
            toast.classList.add('show');
        }, 10);

        // 设置自动消失
        setTimeout(() => {
            toast.classList.remove('show');
            // 动画结束后删除元素
            setTimeout(() => {
                if (toast && toast.parentNode) {
                    this.container.removeChild(toast);
                }
            }, 300); // 300ms 是过渡动画持续时间
        }, duration);

        return toast;
    }

    /**
     * 显示成功提示
     * @param {string} message 消息内容
     * @param {number} duration 持续时间（毫秒），默认 3000ms
     */
    success(message, duration = 3000) {
        return this.show(message, 'success', duration);
    }

    /**
     * 显示错误提示
     * @param {string} message 消息内容
     * @param {number} duration 持续时间（毫秒），默认 3000ms
     */
    error(message, duration = 3000) {
        return this.show(message, 'error', duration);
    }

    /**
     * 显示警告提示
     * @param {string} message 消息内容
     * @param {number} duration 持续时间（毫秒），默认 3000ms
     */
    warning(message, duration = 3000) {
        return this.show(message, 'warning', duration);
    }

    /**
     * 显示信息提示
     * @param {string} message 消息内容
     * @param {number} duration 持续时间（毫秒），默认 3000ms
     */
    info(message, duration = 3000) {
        return this.show(message, 'info', duration);
    }
}

// 创建全局 toast 实例
const toast = new Toast();