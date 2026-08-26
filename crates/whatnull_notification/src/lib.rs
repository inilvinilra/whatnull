use whatnull_config::NotificationsConfig;
use whatnull_types::{NotificationPayload, NotificationPrivacy};

pub struct NotificationFilter;

impl NotificationFilter {
    pub fn process(
        payload: &NotificationPayload,
        config: &NotificationsConfig,
    ) -> Option<NotificationPayload> {
        if !config.enabled || config.dnd_enabled {
            return None;
        }

        match config.privacy {
            NotificationPrivacy::Disabled => None,
            NotificationPrivacy::FullPreview => Some(payload.clone()),
            NotificationPrivacy::SenderOnly => Some(NotificationPayload {
                title: payload.title.clone(),
                body: "New message".to_string(),
                icon: payload.icon.clone(),
                tag: payload.tag.clone(),
            }),
            NotificationPrivacy::Generic => Some(NotificationPayload {
                title: "WhatNull".to_string(),
                body: "New message".to_string(),
                icon: payload.icon.clone(),
                tag: payload.tag.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload() -> NotificationPayload {
        NotificationPayload {
            title: "Alice".to_string(),
            body: "Meeting tomorrow at 10 AM".to_string(),
            icon: None,
            tag: Some("chat-123".to_string()),
        }
    }

    #[test]
    fn test_full_preview() {
        let config = NotificationsConfig {
            enabled: true,
            privacy: NotificationPrivacy::FullPreview,
            dnd_enabled: false,
        };
        let result = NotificationFilter::process(&sample_payload(), &config);
        assert!(result.is_some());
        let p = result.unwrap();
        assert_eq!(p.title, "Alice");
        assert_eq!(p.body, "Meeting tomorrow at 10 AM");
    }

    #[test]
    fn test_sender_only() {
        let config = NotificationsConfig {
            enabled: true,
            privacy: NotificationPrivacy::SenderOnly,
            dnd_enabled: false,
        };
        let result = NotificationFilter::process(&sample_payload(), &config);
        assert!(result.is_some());
        let p = result.unwrap();
        assert_eq!(p.title, "Alice");
        assert_eq!(p.body, "New message");
    }

    #[test]
    fn test_generic() {
        let config = NotificationsConfig {
            enabled: true,
            privacy: NotificationPrivacy::Generic,
            dnd_enabled: false,
        };
        let result = NotificationFilter::process(&sample_payload(), &config);
        assert!(result.is_some());
        let p = result.unwrap();
        assert_eq!(p.title, "WhatNull");
        assert_eq!(p.body, "New message");
    }

    #[test]
    fn test_disabled_or_dnd() {
        let config_disabled = NotificationsConfig {
            enabled: false,
            privacy: NotificationPrivacy::FullPreview,
            dnd_enabled: false,
        };
        assert!(NotificationFilter::process(&sample_payload(), &config_disabled).is_none());

        let config_dnd = NotificationsConfig {
            enabled: true,
            privacy: NotificationPrivacy::FullPreview,
            dnd_enabled: true,
        };
        assert!(NotificationFilter::process(&sample_payload(), &config_dnd).is_none());

        let config_priv_disabled = NotificationsConfig {
            enabled: true,
            privacy: NotificationPrivacy::Disabled,
            dnd_enabled: false,
        };
        assert!(NotificationFilter::process(&sample_payload(), &config_priv_disabled).is_none());
    }
}
