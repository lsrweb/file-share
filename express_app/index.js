const express = require('express');
const path = require('path');
const bodyParser = require('body-parser');
const cookieParser = require('cookie-parser');
const multer = require('multer');
const fs = require('fs');
const crypto = require('crypto');
const http = require('http');
const WebSocket = require('ws');

// 导入工具模块
const { getIpAddress } = require('./utils/ipUtil');
const fileDb = require('./utils/fileDb');
const setting = require('./utils/setting');

// 初始化 Express 应用
const app = express();
const port = process.env.PORT || 5421;

// 创建 HTTP 服务器（用于同时支持 Express 和 WebSocket）
const server = http.createServer(app);

// 初始化 WebSocket 服务器
const wss = new WebSocket.Server({ server });

// WebSocket 连接管理
const clients = new Set();

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
wss.on('connection', (ws) => {
    // 添加新客户端到集合
    clients.add(ws);

    console.log(`WebSocket 客户端已连接，当前连接数: ${clients.size}`);

    // 发送当前文件列表
    ws.send(JSON.stringify({
        type: 'files',
        data: fileDb.listFiles()
    }));

    // 监听客户端消息
    ws.on('message', (message) => {
        try {
            const data = JSON.parse(message);
            console.log('收到客户端消息:', data);

            // 这里可以处理其他类型的客户端消息
        } catch (error) {
            console.error('解析客户端消息失败:', error);
        }
    });

    // 监听连接关闭
    ws.on('close', () => {
        clients.delete(ws);
        console.log(`WebSocket 客户端已断开，当前连接数: ${clients.size}`);
    });

    // 处理连接错误
    ws.on('error', (error) => {
        console.error('WebSocket 连接错误:', error);
        clients.delete(ws);
    });
});

// 向所有客户端广播消息
function broadcastMessage(message) {
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

// 获取客户端 IP
function getClientIp(req) {
    const ip = req.headers['x-forwarded-for'] || req.ip || req.connection.remoteAddress;
    return ip.replace(/::ffff:/, '');
}

// 应用验证中间件
app.use(authMiddleware);

// 主页
app.get('/', (req, res) => {
    res.render('index', {
        title: '文件共享系统',
        files: fileDb.listFiles(),
        settings: setting.getSetting()
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

// 获取可用的 IP 地址列表
app.get('/api/ip-addresses', (req, res) => {
    const addresses = require('./utils/ipUtil').getIpAddresses();
    res.json({ success: true, addresses });
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

// 捕获未处理的错误
app.use((err, req, res, next) => {
    console.error('服务器错误:', err);
    res.status(500).send('服务器内部错误');
});

// 初始化设置并启动服务器
setting.getSetting(); // 初始化设置

// 使用 HTTP 服务器而不是直接使用 Express 监听
server.listen(port, () => {
    const ipAddress = getIpAddress();
    console.log(`服务器运行在 http://${ipAddress}:${port}`);
});