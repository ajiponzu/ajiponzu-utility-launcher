import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  async function hideWindow() {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  }

  return (
    <main className="app-container">
      <div className="header">
        <h1>🚀 Ajiponzu Utility Launcher</h1>
        <button className="hide-btn" onClick={hideWindow} title="タスクトレイに最小化">
          ➖
        </button>
      </div>
      
      <div className="content">
        <p>シンプルなタスクトレイアプリケーション</p>
      </div>
    </main>
  );
}

export default App;
