import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Users, Plus, Check, Trash2, X } from 'lucide-react'
import { useConfigStore } from '../stores/configStore'

interface AccountProfile {
  id: string
  display_name: string
  storage_partition: string
  avatar_color: string
  created_at: number
  last_used_at: number
}

interface AccountSwitcherModalProps {
  onClose: () => void
}

export const AccountSwitcherModal: React.FC<AccountSwitcherModalProps> = ({ onClose }) => {
  const { config, fetchConfig } = useConfigStore()
  const [profiles, setProfiles] = useState<AccountProfile[]>([])
  const [newProfileName, setNewProfileName] = useState('')
  const [newProfileColor, setNewProfileColor] = useState('#10b981')
  const [isCreating, setIsCreating] = useState(false)

  const loadProfiles = async () => {
    try {
      const list = await invoke<AccountProfile[]>('list_profiles')
      setProfiles(list)
    } catch (e) {
      console.error(e)
    }
  }

  useEffect(() => {
    // oxlint-disable-next-line react/set-state-in-effect
    loadProfiles()
  }, [])

  const handleSwitch = async (id: string) => {
    try {
      await invoke('switch_profile', { profileId: id })
      await fetchConfig()
      onClose()
    } catch (e) {
      console.error(e)
    }
  }

  const handleCreate = async () => {
    if (!newProfileName.trim()) return
    try {
      await invoke('create_profile', {
        name: newProfileName.trim(),
        avatarColor: newProfileColor,
      })
      setNewProfileName('')
      setIsCreating(false)
      await loadProfiles()
      await fetchConfig()
    } catch (e) {
      console.error(e)
    }
  }

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation()
    if (profiles.length <= 1) return
    try {
      await invoke('delete_profile', { profileId: id })
      await loadProfiles()
      await fetchConfig()
    } catch (err) {
      console.error(err)
    }
  }

  const activeId = config?.accounts.active_profile_id || 'default'

  return (
    <div className="settings-overlay">
      <div className="settings-modal" style={{ maxWidth: '480px' }}>
        <div className="settings-panel" style={{ padding: '24px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <Users size={20} style={{ color: '#10b981' }} />
              <h2 className="settings-panel-title">Account Profiles</h2>
            </div>
            <button onClick={onClose} style={{ color: 'var(--text-secondary)' }}>
              <X size={20} />
            </button>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', marginBottom: '20px' }}>
            {profiles.map((p) => {
              const isCurrent = p.id === activeId
              const title = p.display_name || p.id
              return (
                <div
                  key={p.id}
                  onClick={() => handleSwitch(p.id)}
                  style={{
                    padding: '12px 16px',
                    borderRadius: 'var(--radius-md)',
                    background: isCurrent ? 'rgba(16, 185, 129, 0.08)' : 'var(--bg-tertiary)',
                    border: `1px solid ${isCurrent ? '#10b981' : 'var(--glass-border)'}`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <div
                      style={{
                        width: '32px',
                        height: '32px',
                        borderRadius: '50%',
                        background: p.avatar_color || '#10b981',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: '#fff',
                        fontWeight: 600,
                        fontSize: '14px',
                      }}
                    >
                      {title.charAt(0).toUpperCase()}
                    </div>
                    <div>
                      <h4 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-primary)' }}>{title}</h4>
                      <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                        {isCurrent ? 'Active Session' : 'Click to switch'}
                      </span>
                    </div>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    {isCurrent && <Check size={18} style={{ color: '#10b981' }} />}
                    {profiles.length > 1 && !isCurrent && (
                      <button
                        onClick={(e) => handleDelete(p.id, e)}
                        style={{ color: '#ef4444', padding: '4px', borderRadius: '4px' }}
                      >
                        <Trash2 size={16} />
                      </button>
                    )}
                  </div>
                </div>
              )
            })}
          </div>

          {!isCreating ? (
            <button
              className="btn-primary"
              onClick={() => setIsCreating(true)}
              style={{
                width: '100%',
                padding: '10px',
                fontSize: '14px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '8px',
              }}
            >
              <Plus size={16} />
              <span>Add Account Profile</span>
            </button>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', padding: '16px', background: 'var(--bg-tertiary)', borderRadius: 'var(--radius-md)', border: '1px solid var(--glass-border)' }}>
              <h4 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-primary)' }}>New Profile Details</h4>
              <input
                type="text"
                className="form-select"
                placeholder="Profile Name (e.g. Work, Personal)"
                value={newProfileName}
                onChange={(e) => setNewProfileName(e.target.value)}
              />
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>Badge Color:</span>
                <input
                  type="color"
                  value={newProfileColor}
                  onChange={(e) => setNewProfileColor(e.target.value)}
                  style={{ width: '32px', height: '32px', border: 'none', background: 'none', cursor: 'pointer' }}
                />
              </div>
              <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
                <button className="btn-primary" onClick={handleCreate} style={{ flex: 1, padding: '8px' }}>
                  Create Profile
                </button>
                <button
                  className="btn-primary"
                  onClick={() => setIsCreating(false)}
                  style={{ flex: 1, padding: '8px', background: 'var(--bg-secondary)' }}
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
