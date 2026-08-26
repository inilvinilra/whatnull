import { create } from 'zustand'

const onboardingStorageKey = 'whatnull:onboarding-completed'

const readOnboardingCompleted = () => {
  if (typeof window === 'undefined') return false
  return window.localStorage.getItem(onboardingStorageKey) === 'true'
}

interface AppState {
  sessionState: 'Uninitialized' | 'Loading' | 'AuthenticationRequired' | 'Authenticated' | 'Offline' | 'Reconnecting' | 'Expired' | 'Failed'
  privacyMode: boolean
  activeTab: 'chats' | 'settings' | 'accounts' | 'about'
  onboardingCompleted: boolean
  setSessionState: (state: AppState['sessionState']) => void
  setPrivacyMode: (enabled: boolean) => void
  setActiveTab: (activeTab: AppState['activeTab']) => void
  setOnboardingCompleted: (completed: boolean) => void
}

export const useAppStore = create<AppState>((set) => ({
  sessionState: 'Uninitialized',
  privacyMode: false,
  activeTab: 'chats',
  onboardingCompleted: readOnboardingCompleted(),
  setSessionState: (sessionState) => set({ sessionState }),
  setPrivacyMode: (privacyMode) => set({ privacyMode }),
  setActiveTab: (activeTab) => set({ activeTab }),
  setOnboardingCompleted: (onboardingCompleted) => {
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(onboardingStorageKey, String(onboardingCompleted))
    }
    set({ onboardingCompleted })
  },
}))
