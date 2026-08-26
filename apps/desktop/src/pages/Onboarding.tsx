
import { useAppStore } from '../stores/appStore'
import { EyeOff, Key, Database } from 'lucide-react'

export const Onboarding: React.FC = () => {
  const { setOnboardingCompleted } = useAppStore()

  return (
    <div className="onboarding-screen">
      <div className="onboarding-card">
        <h2 className="onboarding-title">Welcome to WhatNull</h2>
        <p className="onboarding-desc">
          WhatNull is an open-source, lightweight, and security-hardened shell for WhatsApp Web, engineered specifically for Linux desktop environments.
        </p>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '16px', width: '100%', textAlign: 'left' }}>
          <div style={{ display: 'flex', gap: '16px', alignItems: 'flex-start' }}>
            <EyeOff size={20} style={{ color: 'var(--accent-light)', flexShrink: 0, marginTop: '2px' }} />
            <div>
              <h4 style={{ fontWeight: 600, fontSize: '15px' }}>Privacy First</h4>
              <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginTop: '4px' }}>
                Zero telemetry, zero usage tracking, and zero crash reporting. Your activity belongs to you.
              </p>
            </div>
          </div>

          <div style={{ display: 'flex', gap: '16px', alignItems: 'flex-start' }}>
            <Key size={20} style={{ color: 'var(--accent-light)', flexShrink: 0, marginTop: '2px' }} />
            <div>
              <h4 style={{ fontWeight: 600, fontSize: '15px' }}>Security Hardening</h4>
              <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginTop: '4px' }}>
                The remote WhatsApp WebView is strictly isolated from local privileged APIs and files.
              </p>
            </div>
          </div>

          <div style={{ display: 'flex', gap: '16px', alignItems: 'flex-start' }}>
            <Database size={20} style={{ color: 'var(--accent-light)', flexShrink: 0, marginTop: '2px' }} />
            <div>
              <h4 style={{ fontWeight: 600, fontSize: '15px' }}>Fully Isolated Storage</h4>
              <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginTop: '4px' }}>
                Sessions are stored in dedicated directories using XDG standards, permitting multi-account isolation.
              </p>
            </div>
          </div>
        </div>

        <button className="btn-primary" onClick={() => setOnboardingCompleted(true)}>
          Open WhatsApp Web
        </button>
      </div>
    </div>
  )
}
