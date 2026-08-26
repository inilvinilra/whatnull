import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Sidebar } from './components/Sidebar'
import { SettingsModal } from './components/SettingsModal'
import { AccountSwitcherModal } from './components/AccountSwitcherModal'
import { PrivacyOverlay } from './components/PrivacyOverlay'
import { Onboarding } from './pages/Onboarding'
import { useAppStore } from './stores/appStore'
import { useConfigStore } from './stores/configStore'

export function App() {
  const { onboardingCompleted, activeTab } = useAppStore()
  const { fetchConfig } = useConfigStore()
  const [isSettingsOpen, setIsSettingsOpen] = useState(false)
  const [isAccountSwitcherOpen, setIsAccountSwitcherOpen] = useState(false)
  const [isManualLocked, setIsManualLocked] = useState(false)
  const [isBlurredByFocus, setIsBlurredByFocus] = useState(false)

  useEffect(() => {
    fetchConfig()
  }, [fetchConfig])

  useEffect(() => {
    const unlistenPromise = listen<boolean>('privacy_blur', (event) => {
      setIsBlurredByFocus(event.payload)
    })

    return () => {
      unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
        e.preventDefault()
        setIsManualLocked((prev) => !prev)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  useEffect(() => {
    const visible =
      onboardingCompleted &&
      !isSettingsOpen &&
      !isAccountSwitcherOpen &&
      !isManualLocked &&
      !isBlurredByFocus

    invoke('set_whatsapp_visible', { visible }).catch(() => {})
  }, [
    onboardingCompleted,
    isSettingsOpen,
    isAccountSwitcherOpen,
    isManualLocked,
    isBlurredByFocus,
  ])

  return (
    <div className="app-container">
      <PrivacyOverlay
        isLockedManual={isManualLocked}
        isBlurredByFocus={isBlurredByFocus}
        onUnlockManual={() => setIsManualLocked(false)}
      />

      <Sidebar
        onOpenSettings={() => setIsSettingsOpen(true)}
        onOpenAccountSwitcher={() => setIsAccountSwitcherOpen(true)}
        onPrivacyLock={() => setIsManualLocked(true)}
      />

      <div className="app-main-content">
        {!onboardingCompleted ? (
          <Onboarding />
        ) : (
          <div className="webview-container">
            {activeTab === 'chats' && (
              <div
                id="webview-placeholder"
                style={{
                  width: '100%',
                  height: '100%',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: 'var(--text-secondary)',
                }}
              >
                Connecting to WhatsApp Web...
              </div>
            )}
          </div>
        )}
      </div>

      {isSettingsOpen && <SettingsModal onClose={() => setIsSettingsOpen(false)} />}
      {isAccountSwitcherOpen && (
        <AccountSwitcherModal onClose={() => setIsAccountSwitcherOpen(false)} />
      )}
    </div>
  )
}

export default App
