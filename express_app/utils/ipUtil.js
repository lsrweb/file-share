const os = require('os');

/**
 * 获取本机的 IPv4 地址
 * @returns {string} 本机的 IPv4 地址
 */
function getIpAddress() {
    const interfaces = os.networkInterfaces();

    for (const interfaceName in interfaces) {
        const interfaceInfo = interfaces[interfaceName];

        for (const info of interfaceInfo) {
            // 只获取 IPv4 地址，并且不是内部地址
            if (info.family === 'IPv4' && !info.internal) {
                return info.address;
            }
        }
    }

    // 默认返回 localhost
    return '127.0.0.1';
}

/**
 * 获取所有可用的网络接口名称
 * @returns {string[]} 网络接口名称列表
 */
function getNetInterfaceNames() {
    const interfaces = os.networkInterfaces();
    return Object.keys(interfaces);
}

/**
 * 获取所有可用的 IP 地址
 * @returns {Array<{name: string, address: string, family: string}>} IP 地址列表
 */
function getIpAddresses() {
    const interfaces = os.networkInterfaces();
    const result = [];

    for (const name in interfaces) {
        for (const info of interfaces[name]) {
            if (info.family === 'IPv4' && !info.internal) {
                result.push({
                    name,
                    address: info.address,
                    family: info.family
                });
            }
        }
    }

    return result;
}

module.exports = {
    getIpAddress,
    getNetInterfaceNames,
    getIpAddresses
};