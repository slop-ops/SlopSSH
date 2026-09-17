export interface ToastMessage {
  id: string
  text: string
  type: 'info' | 'success' | 'warning' | 'error'
  durationMs: number
}

let toasts = $state<ToastMessage[]>([])

export function getToasts(): ToastMessage[] {
  return toasts
}

export function showToast(
  text: string,
  type: 'info' | 'success' | 'warning' | 'error' = 'info',
  durationMs: number = 3500,
) {
  const id = crypto.randomUUID()
  const newToast: ToastMessage = { id, text, type, durationMs }
  toasts = [...toasts, newToast]

  if (durationMs > 0) {
    setTimeout(() => {
      dismissToast(id)
    }, durationMs)
  }
}

export function dismissToast(id: string) {
  toasts = toasts.filter((t) => t.id !== id)
}
