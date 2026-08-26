import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface AppConfig {
  schema_version: number
  general: {
    close_behavior: 'Quit' | 'HideToTray' | 'Ask'
    start_minimized: boolean
    remember_window_position: boolean
    zoom_level: number
    language: string
  }
  appearance: {
    theme: 'System' | 'Light' | 'Dark'
  }
  privacy: {
    telemetry: boolean
    analytics: boolean
    crash_upload: boolean
    message_logging: boolean
    contact_logging: boolean
    privacy_mode_enabled: boolean
    blur_on_unfocus: boolean
    blur_on_minimize: boolean
    lock_timeout_mins: number
  }
  notifications: {
    enabled: boolean
    privacy: 'FullPreview' | 'SenderOnly' | 'Generic' | 'Disabled'
    dnd_enabled: boolean
  }
  downloads: {
    ask_every_time: boolean
    default_directory: string | null
  }
  startup: {
    autostart: boolean
  }
  accounts: {
    active_profile_id: string
    profiles: Array<{
      id: string
      display_name: string
      storage_partition: string
      avatar_color: string
      created_at: number
      last_used_at: number
    }>
  }
  advanced: {
    hardware_acceleration: boolean
    enable_dev_tools: boolean
  }
}

interface ConfigState {
  config: AppConfig | null
  loading: boolean
  error: string | null
  fetchConfig: () => Promise<void>
  updateConfig: (updater: (config: AppConfig) => void) => Promise<void>
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  config: null,
  loading: false,
  error: null,
  fetchConfig: async () => {
    set({ loading: true, error: null })
    try {
      const config = await invoke<AppConfig>('get_app_config')
      set({ config, loading: false })
    } catch (e: any) {
      set({ error: e.toString(), loading: false })
    }
  },
  updateConfig: async (updater) => {
    const current = get().config
    if (!current) return
    const updated = JSON.parse(JSON.stringify(current)) as AppConfig
    updater(updated)
    set({ config: updated, loading: true })
    try {
      await invoke('update_app_config', { config: updated })
      set({ loading: false })
    } catch (e: any) {
      set({ error: e.toString(), config: current, loading: false })
    }
  },
}))
