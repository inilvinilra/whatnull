
import { useAppStore } from '../stores/appStore'
import { MessageSquare, Users, Settings, Info, Shield, LogOut } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'

export const Sidebar: React.FC = () => {
  const { activeTab, setActiveTab, privacyMode, setPrivacyMode } = useAppStore()

  const handleQuit = async () => {
    try {
      await invoke('quit_app')
    } catch (e) {
      console.error(e)
    }
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <Shield size={24} style={{ color: 'var(--accent-light)' }} />
        <h1 className="sidebar-logo">WhatNull</h1>
      </div>

      <nav className="sidebar-menu">
        <button
          className={`menu-item ${activeTab === 'chats' ? 'active' : ''}`}
          onClick={() => setActiveTab('chats')}
        >
          <MessageSquare size={18} />
          <span>WhatsApp</span>
        </button>

        <button
          className={`menu-item ${activeTab === 'accounts' ? 'active' : ''}`}
          onClick={() => setActiveTab('accounts')}
        >
          <Users size={18} />
          <span>Accounts</span>
        </button>

        <button
          className={`menu-item ${activeTab === 'settings' ? 'active' : ''}`}
          onClick={() => setActiveTab('settings')}
        >
          <Settings size={18} />
          <span>Settings</span>
        </button>

        <button
          className={`menu-item ${activeTab === 'about' ? 'active' : ''}`}
          onClick={() => setActiveTab('about')}
        >
          <Info size={18} />
          <span>About</span>
        </button>
      </nav>

      <div className="sidebar-footer">
        <button
          className={`menu-item ${privacyMode ? 'active' : ''}`}
          onClick={() => setPrivacyMode(!privacyMode)}
          style={{ width: 'auto', flex: 1 }}
        >
          <Shield size={18} />
          <span>{privacyMode ? 'Protected' : 'Privacy Mode'}</span>
        </button>

        <button className="menu-item" onClick={handleQuit} style={{ padding: '12px' }}>
          <LogOut size={18} />
        </button>
      </div>
    </aside>
  )
}
