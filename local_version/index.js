const express = require('express');
const IpUtil = require('./utils/IpUtil');
const Setting = require('./utils/Setting');
const Server = require('./utils/Server');
const FileUtil = require('./utils/FileUtil');
const FileDb = require('./utils/FileDb');
const EventDispatcher = require('./utils/EventDispatcher');

// 本地启动入口
const app = express();
const port = 3000;

// 初始化配置
console.log('初始化配置...');
Setting.getSetting();

// 启动服务
if (Server.getServerStatus() === Server.StatusStop) {
    console.log('启动服务...');
    Server.startServer();
}

// 提供简单的 HTTP 接口
app.get('/status', (req, res) => {
    res.json({
        serverStatus: Server.getServerStatus(),
        settings: Setting.getSetting(),
    });
});

app.post('/update-setting', express.json(), (req, res) => {
    updateSetting(req.body)
        .then((msg) => res.json({ success: true, message: msg }))
        .catch((err) => res.status(500).json({ success: false, error: err.message }));
});

app.listen(port, () => {
    console.log(`本地服务已启动，访问 http://localhost:${port}`);
});

// 配置更新
const updateSetting = (setting) => {
    return new Promise((resolve, reject) => {
        let updateUploadPath = Setting.updateUploadPath(setting[Setting.uploadPathKey]);
        let updatePort = Setting.updatePort(setting[Setting.portKey]).then((result) => {
            if (result.message === 'ValueNotChange') {
                console.log("端口未变更");
                return;
            }
            // 端口更新成功后重启服务
            Server.stopServer();
            Server.startServer();
        });
        let password = Setting.updatePassword(setting[Setting.Password]);
        let authEnable = Setting.updateAuthEnable(setting[Setting.AuthEnable]);
        let tusEnable = Setting.updateTusEnable(setting[Setting.tusEnableKey]);
        let chunkSize = Setting.updateChunkSize(setting[Setting.chunkSizeKey]);
        Promise.all([updateUploadPath, updatePort, password, authEnable, tusEnable, chunkSize])
            .then((msg) => {
                resolve(msg);
            })
            .catch((e) => {
                console.log(e);
                reject(e);
            });
    });
};

const openFile = (filename) => {
    let file = FileDb.getFile(filename);
    FileUtil.openFile(file.path);
};

module.exports = {
    updateSetting,
    getSetting: Setting.getSetting,
    startServer: Server.startServer,
    stopServer: Server.stopServer,
    getServerStatus: Server.getServerStatus,
    openFile,
    registryEventListener: EventDispatcher.registryEventListener,
    addText: FileDb.addText,
    addFile: FileDb.addFile,
    removeFile: FileDb.removeFile,
    listFiles: FileDb.listFiles,
    updateIp: Setting.updateIp,
    getUrl: Setting.getUrl,
    getIpAddress: IpUtil.getIpAddress,
    getIpAddresses: IpUtil.getIpAddresses,
    getNetInterfaceNames: IpUtil.getNetInterfaceNames,
};
