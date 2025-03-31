const path = require('path');
const fs = require('fs');

/**
 * 格式化文件大小的辅助函数
 * @param {number} size - 文件大小（字节）
 * @returns {string} 格式化后的大小字符串
 */
function formatFileSize(size) {
    if (!size) return '未知';

    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
    if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(2)} MB`;
    return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/**
 * 根据文件名获取MIME类型
 * @param {string} filename - 文件名
 * @returns {string} MIME类型
 */
function getMimeType(filename) {
    const extension = path.extname(filename).toLowerCase();

    const mimeTypes = {
        // 图片
        '.jpg': 'image/jpeg',
        '.jpeg': 'image/jpeg',
        '.png': 'image/png',
        '.gif': 'image/gif',
        '.bmp': 'image/bmp',
        '.svg': 'image/svg+xml',
        '.webp': 'image/webp',

        // 视频
        '.mp4': 'video/mp4',
        '.webm': 'video/webm',
        '.avi': 'video/x-msvideo',
        '.mov': 'video/quicktime',
        '.wmv': 'video/x-ms-wmv',
        '.flv': 'video/x-flv',
        '.mkv': 'video/x-matroska',

        // 音频
        '.mp3': 'audio/mpeg',
        '.wav': 'audio/wav',
        '.ogg': 'audio/ogg',
        '.m4a': 'audio/mp4',
        '.aac': 'audio/aac',
        '.flac': 'audio/flac',

        // 文档
        '.pdf': 'application/pdf',
        '.doc': 'application/msword',
        '.docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        '.xls': 'application/vnd.ms-excel',
        '.xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        '.ppt': 'application/vnd.ms-powerpoint',
        '.pptx': 'application/vnd.openxmlformats-officedocument.presentationml.presentation',

        // 文本
        '.txt': 'text/plain',
        '.html': 'text/html',
        '.css': 'text/css',
        '.js': 'text/javascript',
        '.json': 'application/json',
        '.xml': 'application/xml',
        '.md': 'text/markdown',

        // 压缩
        '.zip': 'application/zip',
        '.rar': 'application/x-rar-compressed',
        '.7z': 'application/x-7z-compressed',
        '.tar': 'application/x-tar',
        '.gz': 'application/gzip'
    };

    return mimeTypes[extension] || 'application/octet-stream';
}

/**
 * 确定文件的预览类型
 * @param {object} file - 文件对象
 * @returns {object} 包含预览信息的对象
 */
function getPreviewInfo(file) {
    // 如果是文本类型而非文件
    if (file.type !== 'file') {
        return {
            previewType: 'text',
            mimeType: 'text/plain'
        };
    }

    const mimeType = getMimeType(file.name);
    let previewType = 'unsupported';
    let fileContent = null;

    // 图片类型
    if (mimeType.startsWith('image/')) {
        previewType = 'image';
    }
    // 视频类型
    else if (mimeType.startsWith('video/')) {
        previewType = 'video';
    }
    // 音频类型
    else if (mimeType.startsWith('audio/')) {
        previewType = 'audio';
    }
    // PDF文件
    else if (mimeType === 'application/pdf') {
        previewType = 'pdf';
    }
    // 文本文件
    else if (mimeType.startsWith('text/') ||
        ['.txt', '.json', '.js', '.css', '.html', '.xml', '.md', '.log'].some(ext =>
            file.name.toLowerCase().endsWith(ext))) {
        previewType = 'text';
        try {
            if (fs.existsSync(file.path)) {
                fileContent = fs.readFileSync(file.path, 'utf8');
            }
        } catch (error) {
            console.error('读取文本文件失败:', error);
            previewType = 'unsupported';
        }
    }
    // Office文件 (Word, Excel, PowerPoint)
    else if (/\.(doc|docx|xls|xlsx|ppt|pptx)$/i.test(file.name)) {
        previewType = 'office';
    }

    return {
        previewType,
        mimeType,
        fileContent
    };
}

module.exports = {
    formatFileSize,
    getMimeType,
    getPreviewInfo
};