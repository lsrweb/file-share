const database = require('./database');

// 初始化数据库
if (!database.getStorageItem('members')) {
    database.setStorageItem('members', {});
}

/**
 * 获取所有成员
 * @returns {Array} 成员列表
 */
function getAllMembers() {
    const membersMap = database.getStorageItem('members', {});
    return Object.values(membersMap);
}

/**
 * 获取单个成员
 * @param {string} ip 成员IP
 * @returns {Object|null} 成员信息
 */
function getMember(ip) {
    const membersMap = database.getStorageItem('members', {});
    return membersMap[ip] || null;
}

/**
 * 添加或更新成员
 * @param {Object} member 成员信息
 */
function updateMember(member) {
    if (!member || !member.ip) return;
    
    const membersMap = database.getStorageItem('members', {});
    membersMap[member.ip] = {
        ...membersMap[member.ip],
        ...member
    };
    
    database.setStorageItem('members', membersMap);
}

/**
 * 更新成员在线状态
 * @param {string} ip 成员IP
 * @param {boolean} isOnline 是否在线
 */
function updateMemberStatus(ip, isOnline = true) {
    const membersMap = database.getStorageItem('members', {});
    const member = membersMap[ip] || {
        ip,
        name: ip,
        firstSeen: Date.now()
    };
    
    member.isOnline = isOnline;
    member.lastSeen = Date.now();
    
    if (isOnline) {
        member.lastConnectTime = Date.now();
    } else {
        member.lastDisconnectTime = Date.now();
    }
    
    membersMap[ip] = member;
    database.setStorageItem('members', membersMap);
}

/**
 * 设置成员昵称
 * @param {string} ip 成员IP
 * @param {string} nickname 新昵称
 * @returns {Object} 更新后的成员信息
 */
function setMemberNickname(ip, nickname) {
    if (!ip || !nickname) return null;
    
    const membersMap = database.getStorageItem('members', {});
    const member = membersMap[ip] || {
        ip,
        firstSeen: Date.now(),
        lastSeen: Date.now()
    };
    
    member.name = nickname;
    membersMap[ip] = member;
    database.setStorageItem('members', membersMap);
    
    return member;
}

module.exports = {
    getAllMembers,
    getMember,
    updateMember,
    updateMemberStatus,
    setMemberNickname
};
