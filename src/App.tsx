import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Settings from "./components/Settings";
import { RegisteredApp } from "./types";
import "./App.css";
import "./responsive.css";
import "./app-theme.css";

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [registeredApps, setRegisteredApps] = useState<RegisteredApp[]>([]);
  const [runningApps, setRunningApps] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadRegisteredApps();
    loadRunningApps();
  }, []);

  const loadRegisteredApps = async () => {
    try {
      const apps = await invoke<RegisteredApp[]>("get_registered_apps");
      setRegisteredApps(apps);
    } catch (error) {
      console.error("Failed to load registered apps:", error);
    }
  };

  const loadRunningApps = async () => {
    try {
      const apps = await invoke<RegisteredApp[]>("get_registered_apps");
      const autoStartAppIds = apps
        .filter((app) => app.auto_start)
        .map((app) => app.id);

      // 実行中プロセスと自動起動アプリを合わせる
      const allRunningIds = [...new Set([...autoStartAppIds])];
      setRunningApps(new Set(allRunningIds));
    } catch (error) {
      console.error("Failed to load running apps:", error);
    }
  };

  const handleLaunchApp = async (app: RegisteredApp) => {
    try {
      await invoke("launch_application", {
        appId: app.id,
        path: app.path,
        arguments: app.arguments,
      });

      // 起動後に実行状態を更新
      setRunningApps((prev) => new Set([...prev, app.id]));
    } catch (error) {
      console.error("Failed to launch application:", error);
      alert(`アプリケーションの起動に失敗しました: ${error}`);
    }
  };

  const handleStopApp = async (app: RegisteredApp) => {
    try {
      console.log(`Stopping app: ${app.name} (ID: ${app.id})`);
      await invoke("stop_application", { appId: app.id });

      // 停止後に実行状態を更新
      setRunningApps((prev) => {
        const newSet = new Set(prev);
        newSet.delete(app.id);
        return newSet;
      });

      console.log(`Successfully stopped ${app.name}`);
    } catch (error) {
      console.error("Failed to stop application:", error);
      alert(`アプリケーションの停止に失敗しました: ${error}`);
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
    loadRunningApps(); // 実行状態も更新
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
          <button
            className="hide-btn"
            onClick={hideWindow}
            title="タスクトレイに最小化"
          >
            ➖
          </button>
        </div>
      </div>

      <div className="content">
        {registeredApps.length === 0 ? (
          <div className="no-apps">
            <p>登録されたアプリケーションがありません</p>
            <button
              onClick={() => setShowSettings(true)}
              className="add-app-btn"
            >
              アプリを追加
            </button>
          </div>
        ) : (
          <div className="apps-list">
            {registeredApps.map((app) => {
              const isRunning = runningApps.has(app.id);
              console.log(runningApps);
              console.log(
                `App: ${app.name}, ID: ${app.id}, isRunning: ${isRunning}`
              );
              return (
                <div key={app.id} className="app-item">
                  <div className="app-info">
                    <h3>{app.name}</h3>
                    <p>{app.description}</p>
                  </div>
                  <div className="app-actions">
                    {!isRunning ? (
                      <button
                        onClick={() => handleLaunchApp(app)}
                        className="launch-btn"
                      >
                        起動
                      </button>
                    ) : (
                      <button
                        onClick={() => handleStopApp(app)}
                        className="stop-btn"
                      >
                        停止
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* 設定画面 */}
      {showSettings && <Settings onClose={handleSettingsClose} />}
    </main>
  );
}

export default App;
