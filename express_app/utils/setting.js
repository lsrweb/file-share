const path = require('path');
const fs = require('fs');
const os = require('os');
const database = require('./database');
const { getIpAddress } = require('./ipUtil');

// 设置键名
const UPLOAD_PATH_KEY = 'uploadPath';
const PORT_KEY = 'port';
const IP_KEY = 'ip';
const AUTH_ENABLE_KEY = 'authEnable';
const PASSWORD_KEY = 'password';
const CHUNK_SIZE_KEY = 'chunkSize';

// 当前 IP 地址
let curIp = getIpAddress();

/**
 * 获取上传路径的默认值 - 用户的下载文件夹
 * @returns {string} 默认上传路径
 */
function getDefaultUploadPath() {
    return path.join(os.homedir(), 'Downloads');
}

/**
 * 获取上传路径
 * @returns {string} 上传路径
 */
function getUploadPath() {
    return database.getStorageItem(UPLOAD_PATH_KEY, getDefaultUploadPath());
}

/**
 * 更新上传路径
 * @param {string} uploadPath 新的上传路径
 * @returns {Promise} 更新结果的 Promise
 */
function updateUploadPath(uploadPath) {
    return new Promise((resolve, reject) => {
        if (!uploadPath) {
            return reject({ success: false, message: '更新上传路径失败，路径为空' });
        }

        // 值没变，不更新
        if (getUploadPath() === uploadPath) {
            return resolve({ success: true, message: 'ValueNotChange' });
        }

        // 检查路径是否存在
        if (!fs.existsSync(uploadPath)) {
            return reject({ success: false, message: '文件夹不存在' });
        }

        // 检查是否为文件夹
        if (!fs.lstatSync(uploadPath).isDirectory()) {
            return reject({ success: false, message: '上传路径必须为文件夹' });
        }

        // 更新路径
        database.setStorageItem(UPLOAD_PATH_KEY, uploadPath);
        resolve({ success: true, message: '修改成功' });
    });
}

/**
 * 获取端口号
 * @returns {number} 端口号
 */
function getPort() {
    const port = database.getStorageItem(PORT_KEY, 5421);
    if (!port) {
        database.setStorageItem(PORT_KEY, 5421);
        return 5421;
    }
    return port;
}

/**
 * 更新端口号
 * @param {number} port 新的端口号
 * @returns {Promise} 更新结果的 Promise
 */
function updatePort(port) {
    return new Promise((resolve, reject) => {
        if (!port) {
            return reject({ success: false, message: '更新端口失败，端口为空' });
        }

        // 值没变，不更新
        if (getPort() === port) {
            return resolve({ success: true, message: 'ValueNotChange' });
        }

        // 更新端口
        database.setStorageItem(PORT_KEY, port);
        resolve({ success: true, message: '修改成功' });
    });
}

/**
 * 获取服务 URL
 * @returns {string} 服务 URL
 */
function getUrl() {
    const ip = getIp();
    const port = getPort();
    return `http://${ip}:${port}`;
}

/**
 * 获取 IP 地址
 * @returns {string} IP 地址
 */
function getIp() {
    return curIp;
}

/**
 * 更新 IP 地址
 * @param {string} ip 新的 IP 地址
 * @returns {Promise} 更新结果的 Promise
 */
function updateIp(ip) {
    return new Promise((resolve, reject) => {
        if (!ip) {
            return reject({ success: false, message: '更新地址失败，地址为空' });
        }

        // 值没变，不更新
        if (getIp() === ip) {
            return resolve({ success: true, message: 'ValueNotChange' });
        }

        // 更新 IP
        curIp = ip;
        resolve({ success: true, message: '修改成功' });
    });
}

/**
 * 获取是否启用身份验证
 * @returns {boolean} 是否启用身份验证
 */
function getAuthEnable() {
    return database.getStorageItem(AUTH_ENABLE_KEY, false);
}

/**
 * 更新是否启用身份验证
 * @param {boolean} value 是否启用身份验证
 * @returns {Promise} 更新结果的 Promise
 */
function updateAuthEnable(value) {
    return new Promise((resolve, reject) => {
        if (value == null) {
            return reject({ success: false, message: '更新失败，值为空' });
        }

        // 更新设置
        database.setStorageItem(AUTH_ENABLE_KEY, value);
        resolve({ success: true, message: '修改成功' });
    });
}

/**
 * 获取密码
 * @returns {string} 密码
 */
function getPassword() {
    return database.getStorageItem(PASSWORD_KEY, 'password');
}

/**
 * 更新密码
 * @param {string} password 新的密码
 * @returns {Promise} 更新结果的 Promise
 */
function updatePassword(password) {
    return new Promise((resolve, reject) => {
        if (password == null) {
            return reject({ success: false, message: '更新失败，值为空' });
        }

        // 更新密码
        database.setStorageItem(PASSWORD_KEY, password);
        resolve({ success: true, message: '修改成功' });
    });
}

/**
 * 获取分片大小
 * @returns {number} 分片大小（MB）
 */
function getChunkSize() {
    return database.getStorageItem(CHUNK_SIZE_KEY, 20);
}

/**
 * 更新分片大小
 * @param {number} chunkSize 新的分片大小（MB）
 * @returns {Promise} 更新结果的 Promise
 */
function updateChunkSize(chunkSize) {
    const isNumber = (value) => {
        if (typeof value === 'number') {
            return true;
        }
        if (typeof value === 'string') {
            return !!value && !isNaN(value);
        }
        return false;
    };

    return new Promise((resolve, reject) => {
        if (!chunkSize) {
            return reject({ success: false, message: '更新分片大小失败，值为空' });
        }

        if (!isNumber(chunkSize)) {
            return reject({ success: false, message: '更新分片大小失败，值不是数字' });
        }

        if (chunkSize <= 0) {
            return reject({ success: false, message: '更新分片大小失败，值应该大于0' });
        }

        // 值没变，不更新
        if (getChunkSize() === chunkSize) {
            return resolve({ success: true, message: 'ValueNotChange' });
        }

        // 更新分片大小
        database.setStorageItem(CHUNK_SIZE_KEY, chunkSize);
        resolve({ success: true, message: '修改成功' });
    });
}

/**
 * 获取所有设置
 * @returns {Object} 所有设置
 */
function getSetting() {
    return {
        uploadPath: getUploadPath(),
        port: getPort(),
        ip: getIp(),
        url: getUrl(),
        authEnable: getAuthEnable(),
        password: getPassword(),
        chunkSize: getChunkSize()
    };
}

/**
 * 更新设置
 * @param {Object} setting 新的设置
 * @returns {Promise} 更新结果的 Promise
 */
function updateSetting(setting) {
    return new Promise((resolve, reject) => {
        const promises = [
            updateUploadPath(setting[UPLOAD_PATH_KEY]).catch(e => e),
            updatePort(setting[PORT_KEY]).catch(e => e),
            updateIp(setting[IP_KEY]).catch(e => e),
            updateAuthEnable(setting[AUTH_ENABLE_KEY]).catch(e => e),
            updatePassword(setting[PASSWORD_KEY]).catch(e => e),
            updateChunkSize(setting[CHUNK_SIZE_KEY]).catch(e => e)
        ];

        Promise.all(promises)
            .then(results => {
                // 检查是否有错误
                const errors = results.filter(r => r && r.success === false);
                if (errors.length > 0) {
                    reject({ success: false, messages: errors.map(e => e.message) });
                } else {
                    resolve({ success: true, message: '设置更新成功' });
                }
            })
            .catch(err => {
                reject({ success: false, message: err.message || '设置更新失败' });
            });
    });
}

module.exports = {
    UPLOAD_PATH_KEY,
    PORT_KEY,
    AUTH_ENABLE_KEY,
    PASSWORD_KEY,
    CHUNK_SIZE_KEY,
    getUploadPath,
    updateUploadPath,
    getPort,
    updatePort,
    getSetting,
    updateSetting,
    getUrl,
    getIp,
    updateIp,
    updateAuthEnable,
    getAuthEnable,
    updatePassword,
    getPassword,
    getChunkSize,
    updateChunkSize
};