const fs = require('fs');
const path = require('path');

// 存储文件的路径
const storageFilePath = path.join(__dirname, 'storage.json');
console.log(storageFilePath);

// 初始化存储文件
if (!fs.existsSync(storageFilePath)) {
    fs.writeFileSync(storageFilePath, JSON.stringify({}));
}

/**
 * 读取存储的数据
 * @returns {Object} 存储的数据
 */
const readStorage = () => {
    try {
        const data = fs.readFileSync(storageFilePath, 'utf-8');
        return JSON.parse(data);
    } catch (err) {
        console.error('读取存储数据失败:', err);
        return {};
    }
};

/**
 * 写入数据到存储文件
 * @param {Object} data 要存储的数据
 */
const writeStorage = (data) => {
    try {
        fs.writeFileSync(storageFilePath, JSON.stringify(data, null, 2));
    } catch (err) {
        console.error('写入存储数据失败:', err);
    }
};

/**
 * 获取存储的项目
 * @param {string} key 键
 * @param {any} defaultValue 默认值
 * @returns {any} 存储的项目或默认值
 */
const getStorageItem = (key, defaultValue) => {
    const storage = readStorage();
    return storage[key] !== undefined ? storage[key] : defaultValue;
};

/**
 * 设置存储的项目
 * @param {string} key 键
 * @param {any} value 值
 */
const setStorageItem = (key, value) => {
    const storage = readStorage();
    storage[key] = value;
    writeStorage(storage);
};

/**
 * 删除存储的项目
 * @param {string} key 键
 */
const removeStorageItem = (key) => {
    const storage = readStorage();
    delete storage[key];
    writeStorage(storage);
};

module.exports = {
    getStorageItem,
    setStorageItem,
    removeStorageItem
};
