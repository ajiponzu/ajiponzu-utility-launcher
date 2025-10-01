import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RegisteredApp } from "../types";
import "./Settings.css";

interface SettingsProps {
  onClose: () => void;
}

export default function Settings({ onClose }: SettingsProps) {
  const [registeredApps, setRegisteredApps] = useState<RegisteredApp[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // アプリ追加・編集用のフォーム
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingApp, setEditingApp] = useState<RegisteredApp | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    path: "",
    arguments: "",
    description: "",
    delay: 0,
    preventDuplicate: false,
    autoStart: false,
  });

  useEffect(() => {
    loadRegisteredApps();
  }, []);

  const loadRegisteredApps = async () => {
    setIsLoading(true);
    try {
      const apps = await invoke<RegisteredApp[]>("get_registered_apps");
      setRegisteredApps(apps);
    } catch (error) {
      console.error("Failed to load registered apps:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const resetForm = () => {
    setFormData({
      name: "",
      path: "",
      arguments: "",
      description: "",
      delay: 0,
      preventDuplicate: false,
      autoStart: false,
    });
    setShowAddForm(false);
    setEditingApp(null);
  };

  const handleAdd = () => {
    console.log("Add button clicked"); // デバッグ用
    resetForm();
    setShowAddForm(true);
  };

  const handleEdit = (app: RegisteredApp) => {
    setFormData({
      name: app.name,
      path: app.path,
      arguments: app.arguments || "",
      description: app.description || "",
      delay: app.delay || 0,
      preventDuplicate: app.prevent_duplicate || false,
      autoStart: app.auto_start || false,
    });
    setEditingApp(app);
    setShowAddForm(true);
  };

  const handleSave = async () => {
    if (!formData.name || !formData.path) {
      alert("名前とパスは必須です");
      return;
    }

    try {
      if (editingApp) {
        // 既存アプリの更新
        await invoke("update_registered_app", {
          id: editingApp.id,
          name: formData.name,
          path: formData.path,
          arguments: formData.arguments,
          description: formData.description,
          delay: formData.delay,
          preventDuplicate: formData.preventDuplicate,
          autoStart: formData.autoStart,
        });
      } else {
        // 新規アプリの追加
        await invoke("add_registered_app", {
          name: formData.name,
          path: formData.path,
          arguments: formData.arguments,
          description: formData.description,
          delay: formData.delay,
          preventDuplicate: formData.preventDuplicate,
          autoStart: formData.autoStart,
        });
      }
      resetForm();
      loadRegisteredApps();
    } catch (error) {
      console.error("Failed to save app:", error);
      alert("保存に失敗しました");
    }
  };

  const handleDelete = async (app: RegisteredApp) => {
    if (confirm(`「${app.name}」を削除しますか？`)) {
      try {
        await invoke("remove_registered_app", { id: app.id });
        loadRegisteredApps();
      } catch (error) {
        console.error("Failed to delete app:", error);
        alert("削除に失敗しました");
      }
    }
  };

  const selectFile = async () => {
    try {
      const path = await invoke<string | null>("open_file_dialog");
      if (path) {
        setFormData({ ...formData, path });
      }
    } catch (error) {
      console.error("Failed to select file:", error);
    }
  };

  return (
    <div className="settings-overlay">
      <div className="settings-modal">
        <div className="settings-header">
          <h2>アプリケーション設定</h2>
          <button className="close-btn" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="settings-content">
          {isLoading ? (
            <div className="loading">読み込み中...</div>
          ) : (
            <>
              {/* アプリリスト */}
              <div className="apps-section">
                <div className="section-header">
                  <h3>登録済みアプリケーション</h3>
                  <button
                    className="add-btn"
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      console.log("Add button clicked");
                      handleAdd();
                    }}
                    type="button"
                  >
                    ➕ 追加
                  </button>
                </div>

                {registeredApps.length === 0 ? (
                  <p className="no-apps">
                    登録されたアプリケーションがありません
                  </p>
                ) : (
                  <div className="apps-list">
                    {registeredApps.map((app) => (
                      <div key={app.id} className="app-card">
                        <div className="app-details">
                          <h4>{app.name}</h4>
                          <p className="app-path">{app.path}</p>
                          {app.arguments && (
                            <p className="app-args">引数: {app.arguments}</p>
                          )}
                          {app.description && (
                            <p className="app-description">{app.description}</p>
                          )}
                          <div className="app-settings">
                            <span
                              className={`status ${
                                app.auto_start ? "enabled" : "disabled"
                              }`}
                            >
                              {app.auto_start ? "自動起動" : "手動起動"}
                            </span>
                            {app.delay && app.delay > 0 && (
                              <span className="delay">遅延: {app.delay}秒</span>
                            )}
                            {app.prevent_duplicate && (
                              <span className="prevent-duplicate">
                                重複起動防止
                              </span>
                            )}
                            {app.auto_start && (
                              <span className="auto-start">自動起動</span>
                            )}
                          </div>
                        </div>
                        <div className="app-actions">
                          <button
                            className="edit-btn"
                            onClick={() => handleEdit(app)}
                          >
                            編集
                          </button>
                          <button
                            className="delete-btn"
                            onClick={() => handleDelete(app)}
                          >
                            削除
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* アプリ追加・編集フォーム */}
              {showAddForm && (
                <div className="form-section">
                  <h3>
                    {editingApp
                      ? "アプリケーション編集"
                      : "アプリケーション追加"}
                  </h3>
                  <div className="form-grid">
                    <div className="form-group">
                      <label>名前 *</label>
                      <input
                        type="text"
                        value={formData.name}
                        onChange={(e) =>
                          setFormData({ ...formData, name: e.target.value })
                        }
                        placeholder="アプリケーション名"
                      />
                    </div>

                    <div className="form-group">
                      <label>実行ファイルパス *</label>
                      <div className="path-input">
                        <input
                          type="text"
                          value={formData.path}
                          onChange={(e) =>
                            setFormData({ ...formData, path: e.target.value })
                          }
                          placeholder="C:\path\to\app.exe"
                        />
                        <button type="button" onClick={selectFile}>
                          参照
                        </button>
                      </div>
                    </div>

                    <div className="form-group">
                      <label>引数</label>
                      <input
                        type="text"
                        value={formData.arguments}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            arguments: e.target.value,
                          })
                        }
                        placeholder="コマンドライン引数 (オプション)"
                      />
                    </div>

                    <div className="form-group">
                      <label>説明</label>
                      <input
                        type="text"
                        value={formData.description}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            description: e.target.value,
                          })
                        }
                        placeholder="アプリケーションの説明 (オプション)"
                      />
                    </div>

                    <div className="form-group">
                      <label>遅延時間 (秒)</label>
                      <input
                        type="number"
                        min="0"
                        value={formData.delay}
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            delay: Number(e.target.value),
                          })
                        }
                      />
                    </div>

                    <div className="form-group checkbox-group">
                      <label>
                        <input
                          type="checkbox"
                          checked={formData.preventDuplicate}
                          onChange={(e) =>
                            setFormData({
                              ...formData,
                              preventDuplicate: e.target.checked,
                            })
                          }
                        />
                        重複起動を防止
                      </label>
                    </div>

                    <div className="form-group checkbox-group">
                      <label>
                        <input
                          type="checkbox"
                          checked={formData.autoStart}
                          onChange={(e) =>
                            setFormData({
                              ...formData,
                              autoStart: e.target.checked,
                            })
                          }
                        />
                        自動起動
                      </label>
                    </div>
                  </div>

                  <div className="form-actions">
                    <button className="save-btn" onClick={handleSave}>
                      {editingApp ? "更新" : "追加"}
                    </button>
                    <button className="cancel-btn" onClick={resetForm}>
                      キャンセル
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
