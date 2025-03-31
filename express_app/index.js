const express = require('express');
const path = require('path');
const bodyParser = require('body-parser');
const cookieParser = require('cookie-parser');
const multer = require('multer');
const fs = require('fs');
const crypto = require('crypto');
const http = require('http');
const WebSocket = require('ws');
const os = require('os');
const readline = require('readline'); // 添加readline模块用于命令行交互
const chalk = require('chalk'); // 用于命令行彩色输出，如果未安装请执行 npm install chalk

// 导入工具模块
const { getIpAddress, getIpAddresses, getClientIp } = require('./utils/ipUtil');
const fileDb = require('./utils/fileDb');
const setting = require('./utils/setting');
const memberDb = require('./utils/memberDb'); // 添加成员数据模块

// 初始化 Express 应用
const app = express();
const port = process.env.PORT || 5421;

// 创建 HTTP 服务器（用于同时支持 Express 和 WebSocket）
const server = http.createServer(app);

// 初始化 WebSocket 服务器
const wss = new WebSocket.Server({ server });

// WebSocket 连接管理
const clients = new Set();

// 存储连接客户端的信息
const connectedClients = new Map();

// 设置模板引擎
app.set('view engine', 'ejs');
app.set('views', path.join(__dirname, 'views'));

// 中间件设置
app.use(bodyParser.urlencoded({ extended: false }));
app.use(bodyParser.json());
app.use(cookieParser());
app.use(express.static(path.join(__dirname, 'public')));

// 会话管理
const sessions = new Set();
const systemToken = crypto.createHash('md5').update(Date.now().toString()).digest('hex');
sessions.add(systemToken);

// WebSocket 连接处理
wss.on('connection', (ws, req) => {
    // 获取客户端IP
    let clientIp = req.headers['x-forwarded-for'] ||
        req.connection.remoteAddress ||
        req.socket.remoteAddress;

    // 如果IP是IPv6格式的本地地址，转换为更友好的格式
    if (clientIp.includes('::1') || clientIp.includes('127.0.0.1')) {
        clientIp = 'localhost';
    } else if (clientIp.includes('::ffff:')) {
        clientIp = clientIp.replace('::ffff:', '');
    }

    // 检查是否存在相同IP的连接
    let existingConnection = null;
    for (let [existingWs, info] of connectedClients.entries()) {
        if (info.ip === clientIp) {
            existingConnection = existingWs;
            break;
        }
    }

    // 如果存在相同IP的旧连接，关闭它
    if (existingConnection) {
        console.log(`检测到重复IP连接 ${clientIp}，正在关闭旧连接...`);
        existingConnection.close();
        clients.delete(existingConnection);
        connectedClients.delete(existingConnection);
    }

    console.log(`新客户端连接，IP: ${clientIp}`);

    // 生成唯一客户端ID
    const clientId = crypto.randomBytes(8).toString('hex');

    // 记录客户端信息
    const clientInfo = {
        id: clientId,
        ip: clientIp,
        connectTime: new Date(),
        userAgent: req.headers['user-agent'] || '未知客户端'
    };

    // 保存客户端信息到Map中
    connectedClients.set(ws, clientInfo);

    // 添加新客户端到集合
    clients.add(ws);

    // 更新成员数据库中的状态
    memberDb.updateMemberStatus(clientIp, true);

    console.log(`WebSocket 客户端已连接，IP: ${clientIp}，当前连接数: ${clients.size}`);
    console.log(`当前在线成员: ${Array.from(connectedClients.values()).map(c => c.ip).join(', ')}`);

    // 先向客户端发送其IP信息，确保客户端知道自己是谁
    ws.send(JSON.stringify({
        type: 'clientInfo',
        data: { ip: clientIp }
    }));

    // 发送当前文件列表
    ws.send(JSON.stringify({
        type: 'files',
        data: fileDb.listFiles()
    }));

    // 广播成员列表更新
    broadcastMembers();

    // 监听客户端消息
    ws.on('message', (message) => {
        try {
            const data = JSON.parse(message);
            console.log('收到客户端消息:', data);

            // 这里可以处理其他类型的客户端消息
            if (data.type === 'setName' && data.name) {
                // 允许客户端设置自己的昵称
                const clientInfo = connectedClients.get(ws);
                if (clientInfo) {
                    clientInfo.name = data.name;

                    // 同时更新成员数据库中的名称
                    memberDb.updateMember({
                        ip: clientInfo.ip,
                        name: data.name
                    });

                    // 广播成员列表更新
                    broadcastMembers();
                }
            }
        } catch (error) {
            console.error('解析客户端消息失败:', error);
        }
    });

    // 监听连接断开
    ws.on('close', () => {
        const clientInfo = connectedClients.get(ws);
        if (clientInfo) {
            // 更新成员状态为离线
            memberDb.updateMemberStatus(clientInfo.ip, false);
        }

        clients.delete(ws);
        connectedClients.delete(ws);
        console.log(`WebSocket 客户端已断开，当前连接数: ${clients.size}`);

        // 广播成员列表更新
        broadcastMembers();
    });

    // 处理连接错误
    ws.on('error', (error) => {
        const clientInfo = connectedClients.get(ws);
        if (clientInfo) {
            // 更新成员状态为离线
            memberDb.updateMemberStatus(clientInfo.ip, false);
        }

        console.error('WebSocket 连接错误:', error);
        clients.delete(ws);
        connectedClients.delete(ws);

        // 广播成员列表更新
        broadcastMembers();
    });
});

// 向所有客户端广播成员列表
function broadcastMembers() {
    // 获取所有成员（包括离线成员）
    const allMembers = memberDb.getAllMembers();

    // 将在线客户端信息与持久化成员信息合并
    const onlineIps = Array.from(connectedClients.values()).map(client => client.ip);

    // 添加当前连接时间等实时信息
    const members = allMembers.map(member => {
        const isCurrentlyOnline = onlineIps.includes(member.ip);
        const connectedClient = Array.from(connectedClients.values())
            .find(client => client.ip === member.ip);

        return {
            id: member.id || crypto.createHash('md5').update(member.ip).digest('hex').substring(0, 8),
            ip: member.ip,
            name: member.name || member.ip,
            isOnline: isCurrentlyOnline,
            connectTime: connectedClient ? connectedClient.connectTime : null,
            lastSeen: member.lastSeen,
            lastDisconnectTime: member.lastDisconnectTime
        };
    });

    // 为每个客户端单独发送带有本机标记的成员列表
    for (let [client, info] of connectedClients.entries()) {
        if (client.readyState === WebSocket.OPEN) {
            // 为当前客户端标记 isCurrentClient
            const personalMembers = members.map(member => ({
                ...member,
                isCurrentClient: member.ip === info.ip
            }));

            client.send(JSON.stringify({
                type: 'members',
                data: personalMembers
            }));
        }
    }
}

// 更新原来的广播消息函数，不要对成员列表使用
function broadcastMessage(message) {
    if (message.type === 'members') {
        // 成员列表已由 broadcastMembers 处理
        return;
    }

    const data = JSON.stringify(message);
    clients.forEach((client) => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(data);
        }
    });
}

// 验证中间件
function authMiddleware(req, res, next) {
    const authEnabled = setting.getAuthEnable();

    // 不需要验证的路径
    const publicPaths = ['/login', '/api/login', '/favicon.ico'];
    const isPublicPath = publicPaths.includes(req.path) ||
        req.path.startsWith('/download/') ||
        req.path.startsWith('/static/') ||
        req.path.startsWith('/js/') ||
        req.path.startsWith('/css/');

    if (!authEnabled || isPublicPath) {
        return next();
    }

    // 验证 Token
    const token = req.cookies.token || req.headers.authorization;
    if (sessions.has(token)) {
        return next();
    }

    // API 请求返回 401
    if (req.path.startsWith('/api/')) {
        return res.status(401).json({ error: '未授权' });
    }

    // 网页请求重定向到登录页
    return res.redirect('/login');
}

// 设置文件上传
const storage = multer.diskStorage({
    destination: (req, file, cb) => {
        const uploadPath = setting.getUploadPath();
        cb(null, uploadPath);
    },
    filename: (req, file, cb) => {
        cb(null, file.originalname);
    }
});
const upload = multer({ storage });

// 应用验证中间件
app.use(authMiddleware);

// 主页
app.get('/', (req, res) => {
    const defaultSettings = {
        uploadPath: '',
        port: 5421,
        ip: 'localhost',
        url: 'http://localhost:5421',
        authEnable: false,
        password: '',
        chunkSize: 20
    };

    // 获取客户端IP
    const clientIp = getClientIp(req);
    // 获取服务器IP
    const serverIp = getIpAddress();

    // 判断是否为主服务端
    const isMainServer = clientIp === serverIp ||
        clientIp === '127.0.0.1' ||
        clientIp === 'localhost' ||
        clientIp.includes('::1') ||
        clientIp.includes('::ffff:127.0.0.1');

    res.render('index', {
        title: '局域网共享',
        files: fileDb.listFiles(),
        settings: { ...defaultSettings, ...setting.getSetting() },
        isMainServer: isMainServer // 传递给模板
    });
});

// 登录页面
app.get('/login', (req, res) => {
    res.render('login', { title: '登录' });
});

// 登录 API
app.post('/api/login', (req, res) => {
    const { password } = req.body;
    const correctPassword = setting.getPassword();

    if (password === correctPassword) {
        const token = crypto.createHash('md5').update(password).digest('hex');
        sessions.add(token);
        return res.json({
            success: true,
            token,
            message: '登录成功'
        });
    }

    return res.status(403).json({
        success: false,
        message: '密码错误'
    });
});

// 文件列表 API
app.get('/api/files', (req, res) => {
    const files = fileDb.listFiles();
    res.json({ success: true, data: files });
});

// 上传文件
app.post('/api/upload', upload.single('file'), (req, res) => {
    const file = req.file;
    const clientIp = getClientIp(req);

    if (!file) {
        return res.status(400).json({ success: false, message: '没有文件上传' });
    }

    const fileInfo = fileDb.addFile({
        name: file.originalname,
        path: file.path,
        username: clientIp,
        type: 'file',
        size: file.size,
        uploadTime: Date.now()
    });

    // 广播文件更新消息给所有客户端
    broadcastMessage({
        type: 'fileAdded',
        data: fileInfo
    });

    res.json({ success: true, message: '文件上传成功' });
});

// 上传文本
app.post('/api/text', (req, res) => {
    const { message, contentType } = req.body;
    const clientIp = getClientIp(req);

    if (!message) {
        return res.status(400).json({ success: false, message: '消息不能为空' });
    }

    // 将 contentType 传递给 addText 方法，默认为 'text'
    const textInfo = fileDb.addText(message, clientIp, contentType || 'text');

    // 广播文本更新消息给所有客户端
    broadcastMessage({
        type: 'fileAdded',
        data: textInfo
    });

    res.json({ success: true, message: '文本分享成功' });
});

// 下载文件
app.get('/download/:filename', (req, res) => {
    const { filename } = req.params;

    try {
        const file = fileDb.getFile(filename);

        if (!file) {
            return res.status(404).send('文件不存在');
        }

        if (!fs.existsSync(file.path)) {
            fileDb.removeFile(file);
            return res.status(404).send('文件不存在');
        }

        res.download(file.path, file.name);
    } catch (err) {
        console.error('下载文件出错:', err);
        res.status(500).send('服务器错误');
    }
});

// 设置 API
app.post('/api/settings', (req, res) => {
    try {
        const newSettings = req.body;
        const previousAuthEnabled = setting.getAuthEnable();
        const newAuthEnabled = newSettings.authEnable;

        // 存储当前token，以便判断是否需要使之失效
        const currentToken = req.cookies.token || req.headers.authorization;

        setting.updateSetting(newSettings)
            .then(() => {
                // 如果从未启用状态变为启用状态，清除所有会话（除了系统令牌）
                if (!previousAuthEnabled && newAuthEnabled) {
                    // 只保留系统令牌
                    const systemTokens = new Set([systemToken]);
                    sessions.clear();
                    systemTokens.forEach(token => sessions.add(token));

                    // 发送标志告诉客户端需要重新登录
                    return res.json({
                        success: true,
                        message: '设置已更新',
                        requireRelogin: true
                    });
                }

                res.json({ success: true, message: '设置已更新' });
            })
            .catch(err => {
                res.status(400).json({ success: false, message: err.message });
            });
    } catch (err) {
        res.status(500).json({ success: false, message: '服务器错误' });
    }
});

// 获取可用的IP地址
app.get('/api/ip-addresses', (req, res) => {
    try {
        const addresses = require('./utils/ipUtil').getIpAddresses();
        res.json({
            success: true,
            addresses: addresses || ['localhost']
        });
    } catch (error) {
        console.error('获取IP地址失败:', error);
        res.json({
            success: false,
            addresses: ['localhost']
        });
    }
});

// 获取当前连接的成员列表
app.get('/api/members', (req, res) => {
    try {
        // 获取所有成员（包括离线成员）
        const allMembers = memberDb.getAllMembers();

        // 将在线客户端信息与持久化成员信息合并
        const onlineIps = Array.from(connectedClients.values()).map(client => client.ip);
        const clientIp = getClientIp(req);

        const members = allMembers.map(member => {
            const isCurrentlyOnline = onlineIps.includes(member.ip);
            const connectedClient = Array.from(connectedClients.values())
                .find(client => client.ip === member.ip);

            return {
                id: member.id || crypto.createHash('md5').update(member.ip).digest('hex').substring(0, 8),
                ip: member.ip,
                name: member.name || member.ip,
                isOnline: isCurrentlyOnline,
                connectTime: connectedClient ? connectedClient.connectTime : null,
                lastSeen: member.lastSeen,
                lastDisconnectTime: member.lastDisconnectTime,
                isCurrentClient: member.ip === clientIp // 标记是否为本机
            };
        });

        res.json({
            success: true,
            data: members
        });
    } catch (error) {
        console.error('获取成员列表失败:', error);
        res.json({
            success: false,
            data: []
        });
    }
});

// 更新成员昵称
app.post('/api/member/nickname', (req, res) => {
    try {
        const { nickname } = req.body;
        const clientIp = getClientIp(req);

        if (!nickname || nickname.trim() === '') {
            return res.status(400).json({
                success: false,
                message: '昵称不能为空'
            });
        }

        // 更新昵称
        const updatedMember = memberDb.setMemberNickname(clientIp, nickname);

        if (!updatedMember) {
            return res.status(400).json({
                success: false,
                message: '更新昵称失败'
            });
        }

        // 广播成员列表更新
        broadcastMembers();

        res.json({
            success: true,
            message: '昵称已更新',
            data: updatedMember
        });
    } catch (error) {
        console.error('更新昵称失败:', error);
        res.status(500).json({
            success: false,
            message: '服务器错误'
        });
    }
});

// 删除文件
app.delete('/api/files/:id', (req, res) => {
    const { id } = req.params;

    try {
        const file = fileDb.getFile(id);

        if (!file) {
            return res.status(404).json({ success: false, message: '文件不存在' });
        }

        // 如果是文件类型，检查文件是否存在，如果存在则删除
        if (file.type === 'file' && fs.existsSync(file.path)) {
            try {
                fs.unlinkSync(file.path);
            } catch (err) {
                console.error('删除文件失败:', err);
                // 即使文件删除失败，也继续删除记录
            }
        }

        // 从数据库中删除记录
        fileDb.removeFile(id);

        // 广播文件删除消息给所有客户端
        broadcastMessage({
            type: 'fileRemoved',
            data: id
        });

        res.json({ success: true, message: '删除成功' });
    } catch (err) {
        console.error('删除文件出错:', err);
        res.status(500).json({ success: false, message: '服务器错误' });
    }
});

// 编辑文本 API
app.put('/api/text/:id', (req, res) => {
    const { id } = req.params;
    const { content } = req.body;

    if (!content || !content.trim()) {
        return res.status(400).json({ success: false, message: '内容不能为空' });
    }

    try {
        const file = fileDb.getFile(id);

        if (!file) {
            return res.status(404).json({ success: false, message: '文本不存在' });
        }

        if (file.type !== 'text') {
            return res.status(400).json({ success: false, message: '只能编辑文本类型' });
        }

        // 更新文本内容
        const updatedFile = fileDb.updateText(id, content);

        // 广播文本更新消息给所有客户端
        broadcastMessage({
            type: 'fileUpdated',
            data: updatedFile
        });

        res.json({ success: true, message: '文本更新成功', data: updatedFile });
    } catch (err) {
        console.error('编辑文本出错:', err);
        res.status(500).json({ success: false, message: '服务器错误' });
    }
});

// 捕获未处理的错误
app.use((err, req, res, next) => {
    console.error('服务器错误:', err);
    res.status(500).send('服务器内部错误');
});

// 获取所有网络接口，优先排序本地局域网IP
function getNetworkInterfaces() {
    const interfaces = os.networkInterfaces();
    const result = [];

    for (const name in interfaces) {
        for (const net of interfaces[name]) {
            // 仅获取IPv4地址，排除内部环回地址
            if (net.family === 'IPv4' && !net.internal) {
                result.push({
                    name,
                    address: net.address,
                    // 添加优先级属性：192.168开头的地址优先级最高
                    priority: net.address.startsWith('192.168') ? 1 :
                        (net.address.startsWith('10.') || net.address.startsWith('172.')) ? 2 : 3
                });
            }
        }
    }

    // 按优先级排序结果
    result.sort((a, b) => a.priority - b.priority);

    return result;
}

// API: 获取网络接口列表
app.get('/api/network-interfaces', (req, res) => {
    try {
        const interfaces = getNetworkInterfaces();
        res.json({ success: true, data: interfaces });
    } catch (error) {
        console.error('获取网络接口失败:', error);
        res.status(500).json({ success: false, message: '获取网络接口失败' });
    }
});

// 交互式选择网卡并启动服务器
async function selectNetworkInterfaceAndStartServer() {
    try {
        // 获取所有网络接口
        const interfaces = getNetworkInterfaces();

        if (interfaces.length === 0) {
            console.log('警告: 未找到有效的网络接口，将使用 localhost');
            startServer('localhost');
            return;
        }

        console.log('========================================');
        console.log('  请选择要使用的网卡:');
        console.log('========================================');

        // 显示网络接口列表
        interfaces.forEach((iface, index) => {
            const marker = index === 0 ? '→' : ' ';
            console.log(`${marker} [${index + 1}] ${iface.name} - ${iface.address}`);
        });

        // 添加localhost选项
        console.log(` [${interfaces.length + 1}] localhost - 127.0.0.1`);

        console.log('----------------------------------------');
        console.log('提示: 默认选择排名第一的网卡，按回车确认');
        console.log('      或者输入编号选择其他网卡');
        console.log('========================================');

        // 创建readline接口
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });

        // 监听用户输入
        rl.question('请输入要使用的网卡编号: ', (answer) => {
            let selectedIndex = parseInt(answer) - 1;

            // 验证输入的有效性
            if (isNaN(selectedIndex) || selectedIndex < 0) {
                selectedIndex = 0; // 默认选择第一个
            }

            let selectedAddress;
            if (selectedIndex >= interfaces.length) {
                selectedAddress = 'localhost';
                console.log(`已选择: localhost (127.0.0.1)`);
            } else {
                const selected = interfaces[selectedIndex];
                selectedAddress = selected.address;
                console.log(`已选择: ${selected.name} - ${selected.address}`);
            }

            // 关闭readline接口
            rl.close();

            // 获取当前设置，只更新IP地址
            const currentSettings = setting.getSetting();
            const updatedSettings = {
                ...currentSettings,
                ip: selectedAddress
            };

            // 更新设置中的IP地址
            setting.updateSetting(updatedSettings)
                .then(() => {
                    // 启动服务器
                    startServer(selectedAddress);
                })
                .catch(err => {
                    console.error('更新设置失败:', err);
                    // 即使更新设置失败，仍然启动服务器
                    startServer(selectedAddress);
                });
        });
    } catch (error) {
        console.error('选择网卡时出错:', error);
        startServer('localhost'); // 出错时使用localhost
    }
}

// 启动服务器
function startServer(ipAddress) {
    server.listen(port, '0.0.0.0', () => {
        console.log(chalk.green('========================================'));
        console.log(chalk.green(`  服务器已启动!`));
        console.log(chalk.green('========================================'));
        console.log(chalk.cyan(`访问地址: http://${ipAddress}:${port}`));
        console.log(chalk.cyan(`本机地址: http://localhost:${port}`));
        console.log(chalk.green('========================================'));
    });
}

// 初始化设置
setting.getSetting(); // 初始化设置

// 启动交互式网卡选择
selectNetworkInterfaceAndStartServer();