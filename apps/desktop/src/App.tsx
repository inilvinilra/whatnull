import { useEffect } from 'react'
import { useAppStore } from './stores/appStore'
import { useConfigStore } from './stores/configStore'
import { Sidebar } from './components/Sidebar'
import { SettingsModal } from './components/SettingsModal'
import { Onboarding } from './pages/Onboarding'
import { Shield, Plus } from 'lucide-react'
import { listen } from '@tauri-apps/api/event'

export default function App() {
  const {
    sessionState,
    setSessionState,
    privacyMode,
    setPrivacyMode,
    activeTab,
    setActiveTab,
    onboardingCompleted,
  } = useAppStore()
  const { fetchConfig } = useConfigStore()

  useEffect(() => {
    fetchConfig()

    const unlistenPromise = listen<string>('session_state_changed', (event) => {
      setSessionState(event.payload as any)
    })

    return () => {
      unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  if (!onboardingCompleted) {
    return <Onboarding />
  }

  return (
    <div className="app-container">
      <Sidebar />

      <main className="main-content">
        {activeTab === 'chats' && (
          <div className="webview-placeholder">
            <Shield size={48} style={{ color: 'var(--accent-light)' }} />
            <h2 style={{ fontSize: '20px', fontWeight: 600 }}>WhatsApp Web Secure Workspace</h2>
            <p style={{ color: 'var(--text-secondary)', fontSize: '14px', maxWidth: '360px', textAlign: 'center' }}>
              The isolated remote WhatsApp Web instance runs within a secure WebView sandbox overlay.
            </p>
            <div style={{ padding: '8px 16px', background: 'rgba(13, 148, 136, 0.1)', border: '1px solid rgba(13, 148, 136, 0.2)', borderRadius: '20px', fontSize: '13px', color: 'var(--accent-light)', fontWeight: 500 }}>
              Status: {sessionState}
            </div>
          </div>
        )}

        {activeTab === 'accounts' && (
          <div style={{ padding: '40px', display: 'flex', flexDirection: 'column', gap: '24px', maxWidth: '800px' }}>
            <h2 style={{ fontSize: '24px', fontWeight: 700 }}>Account Profiles</h2>
            <p style={{ color: 'var(--text-secondary)' }}>
              Manage and switch between separate isolated WhatsApp accounts. Each account is completely separated using profile partition sandboxing.
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))', gap: '16px', marginTop: '16px' }}>
              <div style={{ padding: '20px', background: 'var(--glass-bg)', border: '1px solid var(--accent-color)', borderRadius: 'var(--radius-lg)', display: 'flex', flexDirection: 'column', gap: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <div style={{ width: '40px', height: '40px', borderRadius: '50%', background: 'rgba(13, 148, 136, 0.15)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--accent-light)', fontWeight: 'bold' }}>
                    D
                  </div>
                  <div>
                    <h4 style={{ fontWeight: 600 }}>Default Profile</h4>
                    <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>Active Session</span>
                  </div>
                </div>
              </div>

              <button style={{ padding: '20px', background: 'transparent', border: '1px dashed var(--glass-border)', borderRadius: 'var(--radius-lg)', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: '12px', color: 'var(--text-secondary)' }}>
                <Plus size={20} />
                <span style={{ fontSize: '14px', fontWeight: 500 }}>Add New Profile</span>
              </button>
            </div>
          </div>
        )}

        {activeTab === 'about' && (
          <div style={{ padding: '40px', display: 'flex', flexDirection: 'column', gap: '24px', maxWidth: '640px' }}>
            <h2 style={{ fontSize: '24px', fontWeight: 700 }}>About WhatNull</h2>
            <p style={{ color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              WhatNull is a secure, privacy-focused, and resource-efficient WhatsApp client. It leverages Rust (Tauri 2) to build a native desktop wrapper around WhatsApp Web with strict sandboxing.
            </p>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', padding: '16px', background: 'rgba(255, 255, 255, 0.02)', borderRadius: 'var(--radius-md)', border: '1px solid var(--glass-border)' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '14px' }}>
                <span style={{ color: 'var(--text-secondary)' }}>Version</span>
                <span style={{ fontWeight: 500 }}>0.1.0 (Milestone 1)</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '14px' }}>
                <span style={{ color: 'var(--text-secondary)' }}>Platform</span>
                <span style={{ fontWeight: 500 }}>Linux Desktop</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '14px' }}>
                <span style={{ color: 'var(--text-secondary)' }}>License</span>
                <span style={{ fontWeight: 500 }}>MIT License</span>
              </div>
            </div>

            <div style={{ padding: '16px', background: 'rgba(13, 148, 136, 0.05)', borderRadius: 'var(--radius-md)', border: '1px solid rgba(13, 148, 136, 0.1)', fontSize: '13px', color: 'var(--text-secondary)', lineHeight: 1.5 }}>
              <strong>Brand Disclaimer:</strong> WhatNull is an independent open-source project. It is not affiliated with, endorsed by, sponsored by, or officially connected to WhatsApp or Meta.
            </div>
          </div>
        )}

        {privacyMode && (
          <div className="privacy-overlay" onClick={() => setPrivacyMode(false)}>
            <Shield size={48} style={{ color: 'var(--accent-light)' }} />
            <h2 className="privacy-title">Privacy Mode Active</h2>
            <p className="privacy-subtitle">Your screen is hidden. Click anywhere to unlock.</p>
          </div>
        )}

        {activeTab === 'settings' && (
          <SettingsModal onClose={() => setActiveTab('chats')} />
        )}
      </main>
    </div>
  )
}
