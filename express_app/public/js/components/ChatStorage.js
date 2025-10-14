/**
 * ChatStorage 聊天数据存储管理类
 * 用于在浏览器中持久化存储聊天相关的数据
 */
class ChatStorage {
    /**
     * 构造函数
     * @param {string} storageKey - 本地存储的键名
     * @param {number} maxHistoryCount - 最大历史记录数量
     */
    constructor(storageKey = 'chatHistory', maxHistoryCount = 30) {
        this.storageKey = storageKey;
        this.maxHistoryCount = maxHistoryCount;
        this.history = this._loadHistory();
    }

    /**
     * 从本地存储加载历史记录
     * @private
     * @returns {Array} 历史记录数组
     */
    _loadHistory() {
        try {
            const historyData = localStorage.getItem(this.storageKey);
            return historyData ? JSON.parse(historyData) : [];
        } catch (error) {
            console.error('加载聊天历史记录失败:', error);
            return [];
        }
    }

    /**
     * 保存历史记录到本地存储
     * @private
     */
    _saveHistory() {
        try {
            localStorage.setItem(this.storageKey, JSON.stringify(this.history));
        } catch (error) {
            console.error('保存聊天历史记录失败:', error);
        }
    }

    /**
     * 添加聊天记录
     * @param {Object} chatData - 聊天数据对象
     */
    addHistory(chatData) {
        // 防止添加重复记录
        const exists = this.history.some(item => 
            JSON.stringify(item) === JSON.stringify(chatData)
        );

        if (!exists) {
            // 添加到历史记录开头
            this.history.unshift(chatData);
            
            // 限制历史记录数量
            if (this.history.length > this.maxHistoryCount) {
                this.history = this.history.slice(0, this.maxHistoryCount);
            }
            
            // 保存到本地存储
            this._saveHistory();
        }
    }

    /**
     * 获取所有历史记录
     * @returns {Array} 历史记录数组
     */
    getAllHistory() {
        return [...this.history];
    }

    /**
     * 清空所有历史记录
     */
    clearHistory() {
        this.history = [];
        this._saveHistory();
    }

    /**
     * 移除指定索引的历史记录
     * @param {number} index - 要移除的记录索引
     */
    removeHistory(index) {
        if (index >= 0 && index < this.history.length) {
            this.history.splice(index, 1);
            this._saveHistory();
        }
    }
}

// 导出 ChatStorage 类，使其可以在其他文件中使用
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ChatStorage;
} else {
    // 浏览器环境
    window.ChatStorage = ChatStorage;
}
