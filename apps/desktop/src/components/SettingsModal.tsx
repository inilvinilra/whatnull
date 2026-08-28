import { useEffect, useState } from 'react'
import { useConfigStore } from '../stores/configStore'
import { Bell, Play, RefreshCw, RotateCcw, Settings, Shield, Trash2, X, Zap } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../stores/configStore'

interface SettingsModalProps {
  onClose: () => void
}

export const SettingsModal: React.FC<SettingsModalProps> = ({ onClose }) => {
  const { config, fetchConfig, updateConfig } = useConfigStore()
  const [activeTab, setActiveTab] = useState<'general' | 'notifications' | 'privacy' | 'startup' | 'advanced'>('general')

  useEffect(() => {
    fetchConfig()
  }, [fetchConfig])

  const handleStartupChange = async (enabled: boolean) => {
    try {
      await invoke('set_startup_enabled', { enabled })
      await fetchConfig()
    } catch (e) {
      console.error(e)
    }
  }

  const handleReload = async () => {
    try {
      await invoke('reload_whatsapp')
    } catch (e) {
      console.error(e)
    }
  }

  const handleHardReload = async () => {
    try {
      await invoke('hard_reload_whatsapp')
    } catch (e) {
      console.error(e)
    }
  }

  const handleResetSession = async () => {
    try {
      await invoke('reset_session')
    } catch (e) {
      console.error(e)
    }
  }

  if (!config) return null

  return (
    <div className="settings-overlay">
      <div className="settings-modal">
        <div className="settings-tabs">
          <button
            className={`settings-tab-btn ${activeTab === 'general' ? 'active' : ''}`}
            onClick={() => setActiveTab('general')}
          >
            <Settings size={16} />
            <span>General</span>
          </button>
          <button
            className={`settings-tab-btn ${activeTab === 'notifications' ? 'active' : ''}`}
            onClick={() => setActiveTab('notifications')}
          >
            <Bell size={16} />
            <span>Notifications</span>
          </button>
          <button
            className={`settings-tab-btn ${activeTab === 'privacy' ? 'active' : ''}`}
            onClick={() => setActiveTab('privacy')}
          >
            <Shield size={16} />
            <span>Privacy</span>
          </button>
          <button
            className={`settings-tab-btn ${activeTab === 'startup' ? 'active' : ''}`}
            onClick={() => setActiveTab('startup')}
          >
            <Play size={16} />
            <span>Startup</span>
          </button>
          <button
            className={`settings-tab-btn ${activeTab === 'advanced' ? 'active' : ''}`}
            onClick={() => setActiveTab('advanced')}
          >
            <Zap size={16} />
            <span>Advanced</span>
          </button>
        </div>

        <div className="settings-panel">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <h2 className="settings-panel-title">
              {activeTab.charAt(0).toUpperCase() + activeTab.slice(1)} Settings
            </h2>
            <button onClick={onClose} style={{ color: 'var(--text-secondary)' }}>
              <X size={20} />
            </button>
          </div>

          {activeTab === 'general' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              <div className="form-group">
                <label className="form-label">Close Behavior</label>
                <select
                  className="form-select"
                  value={config.general.close_behavior}
                  onChange={(e) =>
                    updateConfig((cfg) => {
                      cfg.general.close_behavior = e.target.value as AppConfig['general']['close_behavior']
                    })
                  }
                >
                  <option value="Quit">Quit Application</option>
                  <option value="HideToTray">Minimize to System Tray</option>
                  <option value="Ask">Ask Every Time</option>
                </select>
              </div>

              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.general.start_minimized}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.general.start_minimized = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Start Minimized</span>
                </label>
              </div>

              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.general.remember_window_position}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.general.remember_window_position = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Remember Window Size and Position</span>
                </label>
              </div>
            </div>
          )}

          {activeTab === 'notifications' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.notifications.enabled}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.notifications.enabled = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Enable System Notifications</span>
                </label>
              </div>

              <div className="form-group">
                <label className="form-label">Notification Privacy</label>
                <select
                  className="form-select"
                  value={config.notifications.privacy}
                  onChange={(e) =>
                    updateConfig((cfg) => {
                      cfg.notifications.privacy = e.target.value as AppConfig['notifications']['privacy']
                    })
                  }
                  disabled={!config.notifications.enabled}
                >
                  <option value="FullPreview">Full Preview (Sender and message content)</option>
                  <option value="SenderOnly">Sender Only (Hide message content)</option>
                  <option value="Generic">Generic (Hide sender and content)</option>
                  <option value="Disabled">Disabled</option>
                </select>
              </div>
            </div>
          )}

          {activeTab === 'privacy' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.privacy.blur_on_unfocus}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.privacy.blur_on_unfocus = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Blur Window When Unfocused</span>
                </label>
              </div>

              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.privacy.blur_on_minimize}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.privacy.blur_on_minimize = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Blur Window When Minimized</span>
                </label>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', paddingTop: '16px', borderTop: '1px solid var(--glass-border)' }}>
                <h4 className="form-label">Calls and Devices</h4>
                <p style={{ color: 'var(--text-secondary)', fontSize: '13px', lineHeight: 1.5 }}>
                  WhatsApp Web asks for these when you place or answer a call. Requests from any other source are refused.
                </p>

                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.permissions.microphone}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.permissions.microphone = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Allow microphone for voice calls</span>
                </label>

                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.permissions.camera}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.permissions.camera = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Allow camera for video calls</span>
                </label>

                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.permissions.screen_share}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.permissions.screen_share = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Allow screen sharing</span>
                </label>
              </div>

              <div style={{ padding: '16px', background: 'rgba(239, 68, 68, 0.05)', border: '1px solid rgba(239, 68, 68, 0.2)', borderRadius: 'var(--radius-md)', display: 'flex', flexDirection: 'column', gap: '8px' }}>
                <h4 style={{ color: '#ef4444', fontWeight: 600, fontSize: '14px' }}>Privacy Notice</h4>
                <p style={{ color: 'var(--text-secondary)', fontSize: '13px', lineHeight: 1.5 }}>
                  WhatNull collects no telemetry, no usage statistics and no crash reports, and talks to no server of its own.
                </p>
              </div>
            </div>
          )}

          {activeTab === 'startup' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.startup.autostart}
                    onChange={(e) => handleStartupChange(e.target.checked)}
                  />
                  <span className="form-label">Launch WhatNull on system startup</span>
                </label>
              </div>
            </div>
          )}

          {activeTab === 'advanced' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.advanced.hardware_acceleration}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.advanced.hardware_acceleration = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Enable Hardware Acceleration (Requires Restart)</span>
                </label>
              </div>

              <div className="form-group">
                <label className="form-checkbox-group">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={config.advanced.enable_dev_tools}
                    onChange={(e) =>
                      updateConfig((cfg) => {
                        cfg.advanced.enable_dev_tools = e.target.checked
                      })
                    }
                  />
                  <span className="form-label">Enable Web Inspector (Developer Tools)</span>
                </label>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginTop: '12px', paddingTop: '16px', borderTop: '1px solid var(--glass-border)' }}>
                <h4 className="form-label">Session Controls & Recovery</h4>
                <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
                  <button className="btn-primary" style={{ padding: '8px 16px', fontSize: '13px', display: 'flex', alignItems: 'center', gap: '6px' }} onClick={handleReload}>
                    <RefreshCw size={14} />
                    <span>Reload WhatsApp</span>
                  </button>
                  <button className="btn-primary" style={{ padding: '8px 16px', fontSize: '13px', background: 'var(--bg-tertiary)', display: 'flex', alignItems: 'center', gap: '6px' }} onClick={handleHardReload}>
                    <RotateCcw size={14} />
                    <span>Hard Reload</span>
                  </button>
                  <button className="btn-primary" style={{ padding: '8px 16px', fontSize: '13px', background: '#ef4444', display: 'flex', alignItems: 'center', gap: '6px' }} onClick={handleResetSession}>
                    <Trash2 size={14} />
                    <span>Reset Session</span>
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
