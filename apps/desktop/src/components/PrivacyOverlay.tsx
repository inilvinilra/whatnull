import { Shield, Lock, EyeOff } from 'lucide-react'

interface PrivacyOverlayProps {
  isLockedManual: boolean
  isBlurredByFocus: boolean
  onUnlockManual: () => void
}

export const PrivacyOverlay: React.FC<PrivacyOverlayProps> = ({
  isLockedManual,
  isBlurredByFocus,
  onUnlockManual,
}) => {
  const active = isLockedManual || isBlurredByFocus

  if (!active) return null

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 9999,
        background: 'rgba(9, 14, 23, 0.85)',
        backdropFilter: 'blur(24px)',
        WebkitBackdropFilter: 'blur(24px)',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '20px',
        userSelect: 'none',
      }}
    >
      <div
        style={{
          width: '72px',
          height: '72px',
          borderRadius: '50%',
          background: 'rgba(16, 185, 129, 0.1)',
          border: '1px solid rgba(16, 185, 129, 0.3)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#10b981',
        }}
      >
        {isLockedManual ? <Lock size={32} /> : <EyeOff size={32} />}
      </div>

      <div style={{ textAlign: 'center' }}>
        <h2 style={{ fontSize: '20px', fontWeight: 600, color: 'var(--text-primary)', marginBottom: '6px' }}>
          {isLockedManual ? 'WhatNull Locked' : 'Privacy Protection Active'}
        </h2>
        <p style={{ fontSize: '14px', color: 'var(--text-secondary)', maxWidth: '320px', lineHeight: 1.5 }}>
          {isLockedManual
            ? 'Screen is locked. Click unlock to restore your WhatsApp session.'
            : 'Window is unfocused. Content is hidden for your security.'}
        </p>
      </div>

      {isLockedManual && (
        <button
          className="btn-primary"
          onClick={onUnlockManual}
          style={{
            padding: '10px 24px',
            fontSize: '14px',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          }}
        >
          <Shield size={16} />
          <span>Unlock Session</span>
        </button>
      )}
    </div>
  )
}
