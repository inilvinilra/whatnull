import React from 'react'
import { Shield, Settings, Users, Lock } from 'lucide-react'

interface SidebarProps {
  onOpenSettings: () => void
  onOpenAccountSwitcher: () => void
  onPrivacyLock: () => void
}

export const Sidebar: React.FC<SidebarProps> = ({
  onOpenSettings,
  onOpenAccountSwitcher,
  onPrivacyLock,
}) => {
  return (
    <div className="app-sidebar">
      <div className="sidebar-logo">
        <div className="sidebar-logo-icon">
          <Shield size={18} />
        </div>
      </div>

      <div style={{ flex: 1 }} />

      <div className="sidebar-actions">
        <button
          className="sidebar-btn"
          title="Privacy Lock (Ctrl+L)"
          onClick={onPrivacyLock}
        >
          <Lock size={18} />
        </button>

        <button
          className="sidebar-btn"
          title="Switch Account Profile"
          onClick={onOpenAccountSwitcher}
        >
          <Users size={18} />
        </button>

        <button
          className="sidebar-btn"
          title="Settings"
          onClick={onOpenSettings}
        >
          <Settings size={18} />
        </button>
      </div>
    </div>
  )
}
