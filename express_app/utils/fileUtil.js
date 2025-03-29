/**
 * 文件工具函数
 */

const path = require('path');

/**
 * 判断文件是否为图片
 * @param {string} filename 文件名
 * @returns {boolean} 是否为图片
 */
function isImage(filename) {
    if (!filename) return false;
    const ext = path.extname(filename).toLowerCase();
    return ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.webp', '.svg'].includes(ext);
}

/**
 * 判断文件是否为视频
 * @param {string} filename 文件名
 * @returns {boolean} 是否为视频
 */
function isVideo(filename) {
    if (!filename) return false;
    const ext = path.extname(filename).toLowerCase();
    return ['.mp4', '.webm', '.ogg', '.mov', '.avi', '.wmv', '.flv', '.mkv'].includes(ext);
}

/**
 * 判断文件是否为音频
 * @param {string} filename 文件名
 * @returns {boolean} 是否为音频
 */
function isAudio(filename) {
    if (!filename) return false;
    const ext = path.extname(filename).toLowerCase();
    return ['.mp3', '.wav', '.ogg', '.m4a', '.flac', '.aac'].includes(ext);
}

/**
 * 判断文件是否为PDF
 * @param {string} filename 文件名
 * @returns {boolean} 是否为PDF
 */
function isPdf(filename) {
    if (!filename) return false;
    const ext = path.extname(filename).toLowerCase();
    return ext === '.pdf';
}

/**
 * 判断文件是否为文本文件
 * @param {string} filename 文件名
 * @returns {boolean} 是否为文本文件
 */
function isText(filename) {
    if (!filename) return false;
    const ext = path.extname(filename).toLowerCase();
    return ['.txt', '.md', '.json', '.js', '.html', '.css', '.xml', '.csv', '.ts', '.jsx', '.tsx', '.vue', '.py', '.java', '.c', '.cpp', '.h', '.php', '.rb', '.go', '.rs'].includes(ext);
}

/**
 * 获取文件类型
 * @param {string} filename 文件名
 * @returns {string} 文件类型(image, video, audio, pdf, text, other)
 */
function getFileType(filename) {
    if (isImage(filename)) return 'image';
    if (isVideo(filename)) return 'video';
    if (isAudio(filename)) return 'audio';
    if (isPdf(filename)) return 'pdf';
    if (isText(filename)) return 'text';
    return 'other';
}

module.exports = {
    isImage,
    isVideo,
    isAudio,
    isPdf,
    isText,
    getFileType
};