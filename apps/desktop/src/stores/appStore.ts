import { create } from 'zustand'

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
  onboardingCompleted: false,
  setSessionState: (sessionState) => set({ sessionState }),
  setPrivacyMode: (privacyMode) => set({ privacyMode }),
  setActiveTab: (activeTab) => set({ activeTab }),
  setOnboardingCompleted: (onboardingCompleted) => set({ onboardingCompleted }),
}))
