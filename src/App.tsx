import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Settings from "./components/Settings";
import { RegisteredApp } from "./types";
import "./App.css";

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [registeredApps, setRegisteredApps] = useState<RegisteredApp[]>([]);

  useEffect(() => {
    loadRegisteredApps();
  }, []);

  const loadRegisteredApps = async () => {
    try {
      const apps = await invoke<RegisteredApp[]>('get_registered_apps');
      setRegisteredApps(apps);
    } catch (error) {
      console.error('Failed to load registered apps:', error);
    }
  };

  const handleLaunchApp = async (app: RegisteredApp) => {
    try {
      await invoke("launch_application", { 
        appId: app.id, 
        path: app.path, 
        arguments: app.arguments 
      });
    } catch (error) {
      console.error("Failed to launch application:", error);
    }
  };

  async function hideWindow() {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  }

  const handleSettingsClose = () => {
    setShowSettings(false);
    loadRegisteredApps(); // 設定画面を閉じたら再読み込み
  };

  return (
    <main className="app-container">
      <div className="header">
        <h1>🚀 Ajiponzu Utility Launcher</h1>
        <div className="header-buttons">
          <button 
            className="settings-btn" 
            onClick={() => setShowSettings(true)} 
            title="設定"
          >
            ⚙️
          </button>
          <button className="hide-btn" onClick={hideWindow} title="タスクトレイに最小化">
            ➖
          </button>
        </div>
      </div>
      
      <div className="content">
        {registeredApps.length === 0 ? (
          <div className="no-apps">
            <p>登録されたアプリケーションがありません</p>
            <button onClick={() => setShowSettings(true)} className="add-app-btn">
              アプリを追加
            </button>
          </div>
        ) : (
          <div className="apps-list">
            {registeredApps.map((app) => (
              <div key={app.id} className="app-item">
                <div className="app-info">
                  <h3>{app.name}</h3>
                  <p>{app.description}</p>
                </div>
                <button 
                  onClick={() => handleLaunchApp(app)}
                  className="launch-btn"
                >
                  起動
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 設定画面 */}
      {showSettings && <Settings onClose={handleSettingsClose} />}
    </main>
  );
}

export default App;
