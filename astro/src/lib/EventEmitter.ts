export class EventEmitter {
  private listeners: Record<string, Set<() => void>> = {}

  on(event: string, listener: () => void) {
    if (!this.listeners[event]) {
      this.listeners[event] = new Set()
    }
    this.listeners[event].add(listener)
  }

  off(event: string, listener: () => void) {
    this.listeners[event]?.delete(listener)
  }

  emit(event: string) {
    this.listeners[event]?.forEach(listener => listener())
  }
}
