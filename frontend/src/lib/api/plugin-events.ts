import { listen } from '@tauri-apps/api/event'

export interface PluginEventPayload {
  pluginId: string
  eventType: string
  payload: any
}

export interface PluginNotificationPayload {
  pluginId: string
  title: string
  body: string
}

export function onPluginEvent(
  pluginId: string,
  handler: (event: PluginEventPayload) => void,
): Promise<() => void> {
  return listen<PluginEventPayload>(`plugin-event-${pluginId}`, (e) => {
    handler(e.payload)
  })
}

export function onPluginNotification(
  handler: (notification: PluginNotificationPayload) => void,
): Promise<() => void> {
  return listen<PluginNotificationPayload>('plugin-notification', (e) => {
    handler(e.payload)
  })
}
