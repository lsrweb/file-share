const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const database = require('./database');

// 文件数据库的键名
const FILE_DB_KEY = 'fileDb';

/**
 * 生成唯一的文件ID
 * @param {string} name 文件名
 * @returns {string} 文件ID
 */
function generateFileId(name) {
    const timestamp = Date.now().toString();
    const random = Math.random().toString();
    const hash = crypto.createHash('md5').update(name + timestamp + random).digest('hex');
    return hash.substring(0, 8);
}

/**
 * 获取文件数据库
 * @returns {Object} 文件数据库对象
 */
function getFileDb() {
    const fileDbStr = database.getStorageItem(FILE_DB_KEY, '{}');
    return typeof fileDbStr === 'string' ? JSON.parse(fileDbStr) : fileDbStr;
}

/**
 * 保存文件数据库
 * @param {Object} fileDb 文件数据库对象
 */
function saveFileDb(fileDb) {
    database.setStorageItem(FILE_DB_KEY, typeof fileDb === 'string' ? fileDb : JSON.stringify(fileDb));
}

/**
 * 添加文件到数据库
 * @param {Object} file 文件对象，包含 name、path、username 等属性
 * @returns {Object} 添加后的文件对象
 */
function addFile(file) {
    const fileDb = getFileDb();
    const id = generateFileId(file.name);

    const fileInfo = {
        id,
        name: file.name,
        path: file.path,
        username: file.username || 'anonymous',
        type: 'file',
        size: file.size || 0,
        uploadTime: Date.now(),
    };

    fileDb[id] = fileInfo;
    saveFileDb(fileDb);

    return fileInfo;
}

/**
 * 添加文本到数据库
 * @param {string} text 文本内容
 * @param {string} username 用户名
 * @param {string} contentType 文本类型，可以是 'text' 或 'markdown'
 * @returns {Object} 添加后的文本对象
 */
function addText(text, username = 'anonymous', contentType = 'text') {
    const fileDb = getFileDb();
    const id = generateFileId(text);

    // 为 Markdown 内容创建更有意义的名称
    let displayName = text.length > 20 ? text.substring(0, 20) + '...' : text;
    if (contentType === 'markdown') {
        // 尝试从 Markdown 中提取标题作为显示名称
        const titleMatch = text.match(/^#\s+(.+)$/m);
        if (titleMatch && titleMatch[1]) {
            displayName = titleMatch[1];
        } else {
            displayName = 'Markdown 笔记';
        }
    }

    const textInfo = {
        id,
        name: displayName,
        type: 'text',
        content: text,
        contentType: contentType,  // 新增字段，表示文本的类型
        username,
        uploadTime: Date.now()
    };

    fileDb[id] = textInfo;
    saveFileDb(fileDb);

    return textInfo;
}

/**
 * 获取指定ID的文件
 * @param {string} id 文件ID
 * @returns {Object|null} 文件对象，不存在则返回 null
 */
function getFile(id) {
    const fileDb = getFileDb();
    return fileDb[id] || null;
}

/**
 * 移除文件
 * @param {string|Object} file 文件ID或文件对象
 * @returns {boolean} 是否成功移除
 */
function removeFile(file) {
    const fileDb = getFileDb();
    const id = typeof file === 'string' ? file : file.id;

    if (!fileDb[id]) {
        return false;
    }

    delete fileDb[id];
    saveFileDb(fileDb);

    return true;
}

/**
 * 列出所有文件
 * @returns {Array} 文件对象数组
 */
function listFiles() {
    const fileDb = getFileDb();
    return Object.values(fileDb).sort((a, b) => b.uploadTime - a.uploadTime);
}

module.exports = {
    addFile,
    addText,
    getFile,
    removeFile,
    listFiles
};