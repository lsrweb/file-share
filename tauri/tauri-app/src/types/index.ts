export interface ServerInfo {
  message_type: string;
  server_address: string;
  server_port: number;
  server_name: string;
  timestamp: number;
}

export interface SharedItem {
  id: string;
  name: string;
  type: 'file' | 'text';
  content?: string;
  path?: string;
  fileType?: string;  // 文件类型: 'image', 'video', 'audio', 'pdf', 'document', 'text', 其他
  mimeType?: string;  // MIME 类型
  size?: number;
  username: string;
  upload_time: number;
  updateTime?: number;
  contentType?: string; // 内容类型: 'markdown', 'plain', 'html', 等
}