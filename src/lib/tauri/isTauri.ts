/**
 * Robust detection of the Tauri environment across Tauri v1 and Tauri v2.
 * In Tauri v2, window.__TAURI_INTERNALS__ is always injected even if withGlobalTauri is false.
 */
export function isTauri(): boolean {
  if (typeof window === 'undefined') return false;
  return '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
}
