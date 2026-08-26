import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { SettingsModal } from './components/SettingsModal'
import { AccountSwitcherModal } from './components/AccountSwitcherModal'
import { PrivacyOverlay } from './components/PrivacyOverlay'
import { Onboarding } from './pages/Onboarding'
import { useAppStore } from './stores/appStore'
import { useConfigStore } from './stores/configStore'

type ShellAction = 'openSettings' | 'openAccounts' | 'toggleLock'

export function App() {
  const { onboardingCompleted } = useAppStore()
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
    const unlistenPromise = listen<ShellAction>('shell_action', (event) => {
      switch (event.payload) {
        case 'openSettings':
          setIsAccountSwitcherOpen(false)
          setIsSettingsOpen(true)
          break
        case 'openAccounts':
          setIsSettingsOpen(false)
          setIsAccountSwitcherOpen(true)
          break
        case 'toggleLock':
          setIsManualLocked((prev) => !prev)
          break
      }
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
    const overlay =
      !onboardingCompleted ||
      isSettingsOpen ||
      isAccountSwitcherOpen ||
      isManualLocked ||
      isBlurredByFocus

    invoke('set_overlay_visible', { visible: overlay }).catch(() => {})
  }, [
    onboardingCompleted,
    isSettingsOpen,
    isAccountSwitcherOpen,
    isManualLocked,
    isBlurredByFocus,
  ])

  return (
    <div className={`app-container ${!onboardingCompleted ? 'overlay-mode' : ''}`}>
      <PrivacyOverlay
        isLockedManual={isManualLocked}
        isBlurredByFocus={isBlurredByFocus}
        onUnlockManual={() => setIsManualLocked(false)}
      />

      {!onboardingCompleted && <Onboarding />}

      {isSettingsOpen && <SettingsModal onClose={() => setIsSettingsOpen(false)} />}
      {isAccountSwitcherOpen && (
        <AccountSwitcherModal onClose={() => setIsAccountSwitcherOpen(false)} />
      )}
    </div>
  )
}

export default App
