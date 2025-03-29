const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const database = require('./database');
const fileUtil = require('./fileUtil');

// 初始化数据库
if (!database.getStorageItem('files')) {
    database.setStorageItem('files', []);
}

/**
 * 生成随机ID
 * @returns {string} 随机ID
 */
function generateId() {
    return crypto.randomBytes(4).toString('hex');
}

/**
 * 获取所有文件
 * @returns {Array} 文件列表
 */
function listFiles() {
    return database.getStorageItem('files') || [];
}

/**
 * 获取指定ID的文件
 * @param {string} id 文件ID
 * @returns {Object|null} 文件对象
 */
function getFile(id) {
    const files = database.getStorageItem('files') || [];
    return files.find(file => file.id === id) || null;
}

/**
 * 添加文件
 * @param {Object} file 文件对象
 * @returns {Object} 添加的文件对象
 */
function addFile(file) {
    const id = generateId();
    const fileObj = {
        ...file,
        id,
        uploadTime: Date.now(),
        fileType: fileUtil.getFileType(file.name) // 自动识别文件类型
    };

    const files = database.getStorageItem('files') || [];
    files.unshift(fileObj);
    database.setStorageItem('files', files);

    return fileObj;
}

/**
 * 添加文本
 * @param {string} content 文本内容
 * @param {string} username 用户名
 * @param {string} contentType 内容类型 ('text' 或 'markdown')
 * @returns {Object} 添加的文本对象
 */
function addText(content, username, contentType = 'text') {
    const id = generateId();
    const textObj = {
        id,
        name: content.length > 30 ? content.substring(0, 30) + '...' : content,
        type: 'text',
        contentType: contentType,
        content,
        username,
        uploadTime: Date.now()
    };

    const files = database.getStorageItem('files') || [];
    files.unshift(textObj);
    database.setStorageItem('files', files);

    return textObj;
}

/**
 * 删除文件
 * @param {string} id 文件ID
 */
function removeFile(id) {
    let files = database.getStorageItem('files') || [];
    files = files.filter(file => file.id !== id);
    database.setStorageItem('files', files);
}

/**
 * 更新文本内容
 * @param {string} id 文本ID
 * @param {string} content 新的文本内容
 * @returns {Object} 更新后的文本对象
 */
function updateText(id, content) {
    const files = database.getStorageItem('files') || [];
    const fileIndex = files.findIndex(file => file.id === id);

    if (fileIndex === -1) {
        throw new Error('文本不存在');
    }

    const file = files[fileIndex];
    if (file.type !== 'text') {
        throw new Error('只能编辑文本类型');
    }

    // 更新文本内容
    const updatedFile = {
        ...file,
        content,
        name: content.length > 30 ? content.substring(0, 30) + '...' : content,
        updateTime: Date.now()  // 添加更新时间
    };

    files[fileIndex] = updatedFile;
    database.setStorageItem('files', files);

    return updatedFile;
}

module.exports = {
    listFiles,
    getFile,
    addFile,
    addText,
    removeFile,
    updateText
};