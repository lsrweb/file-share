const fs = require('fs');
const path = require('path');

const storageFilePath = path.join(__dirname, 'storage.json');

// 初始化存储文件
if (!fs.existsSync(storageFilePath)) {
    fs.writeFileSync(storageFilePath, JSON.stringify({}));
}

const readStorage = () => {
    const data = fs.readFileSync(storageFilePath, 'utf-8');
    return JSON.parse(data);
};

const writeStorage = (data) => {
    fs.writeFileSync(storageFilePath, JSON.stringify(data, null, 2));
};

const getStorageItem = (key) => {
    const storage = readStorage();
    return storage[key];
};

const setStorageItem = (key, value) => {
    const storage = readStorage();
    storage[key] = value;
    writeStorage(storage);
};

const removeStorageItem = (key) => {
    const storage = readStorage();
    delete storage[key];
    writeStorage(storage);
};

module.exports = {
    getStorageItem,
    setStorageItem,
    removeStorageItem,
};