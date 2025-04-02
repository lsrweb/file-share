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
  fileType?: string;
  size?: number;
  username: string;
  uploadTime: number;
  updateTime?: number;
}