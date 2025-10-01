// アプリケーションの型定義

export interface RegisteredApp {
  id: string;
  name: string;
  path: string;
  arguments: string;
  description: string;
  enabled: boolean;
  delay: number;
  prevent_duplicate: boolean;
}

export interface UtilityTool {
  id: string;
  name: string;
  description: string;
  command: string;
  icon: string;
}
