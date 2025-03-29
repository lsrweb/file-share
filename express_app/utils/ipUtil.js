const os = require('os');

function _normalizeFamily(family) {
    return family ? family.toLowerCase() : 'ipv4';
}

function isLoopback(addr) {
    return /^(::f{4}:)?127\.([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})/
        .test(addr) ||
        /^fe80::1$/.test(addr) ||
        /^::1$/.test(addr) ||
        /^::$/.test(addr);
}

function loopback(family) {
    family = _normalizeFamily(family);

    if (family !== 'ipv4' && family !== 'ipv6') {
        throw new Error('family must be ipv4 or ipv6');
    }

    return family === 'ipv4' ? '127.0.0.1' : 'fe80::1';
}

// 获取特定网络接口的所有IP地址
function getIpAddresses(netInterfaceName, ipFamily = "ipv4") {
    const interfaces = os.networkInterfaces();

    // 如果指定了网络接口名称
    if (netInterfaceName) {
        if (!interfaces[netInterfaceName]) {
            return [];
        }

        return interfaces[netInterfaceName].filter(function (ipAddress) {
            const { family, address } = ipAddress;
            return family.toLowerCase() === ipFamily.toLowerCase() && !isLoopback(address);
        });
    }

    // 如果没有指定网络接口名称，返回所有可用的IP地址
    const addresses = ['localhost']; // 始终包含localhost

    for (const devName in interfaces) {
        const iface = interfaces[devName];
        for (let i = 0; i < iface.length; i++) {
            const alias = iface[i];
            if (alias.family.toLowerCase() === ipFamily.toLowerCase() && !isLoopback(alias.address)) {
                addresses.push(alias.address);
            }
        }
    }

    return addresses;
}

// 获取所有可用的网络接口名称
function getNetInterfaceNames(ipFamily = "ipv4") {
    const interfaces = os.networkInterfaces();
    const ipNames = Object.keys(interfaces).filter(function (name) {
        return getIpAddresses(name, ipFamily).length > 0;
    });

    // 过滤掉虚拟接口
    const finalNames = ipNames.filter(function (name) {
        return !/(loopback|vmware|internal|lo|vEthernet)/gi.test(name);
    });

    return finalNames.length === 0 ? (ipNames.length === 0 ? [] : ipNames) : finalNames;
}

// 获取首选IP地址
function getIpAddress(idx = 0, ipFamily = 'ipv4') {
    const names = getNetInterfaceNames(ipFamily);
    if (!names.length) {
        return loopback(ipFamily);
    }

    idx = (idx % names.length);
    const ipAddresses = getIpAddresses(names[idx], ipFamily);
    return ipAddresses.length > 0 ? ipAddresses[0].address : loopback(ipFamily);
}

// 获取客户端IP地址
function getClientIp(req) {
    if (!req) return '未知IP';

    // 检查各种可能的请求头和属性
    const ip = req.headers['x-forwarded-for'] ||
        req.headers['x-real-ip'] ||
        req.connection?.remoteAddress ||
        req.socket?.remoteAddress ||
        req.ip;

    if (!ip) return '未知IP';

    // 处理IPv4映射的IPv6地址 (::ffff:192.168.1.1)
    return ip.replace(/^::ffff:/, '');
}

module.exports = {
    getIpAddress,
    getIpAddresses,
    getNetInterfaceNames,
    getClientIp
};