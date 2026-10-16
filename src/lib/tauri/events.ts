import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface FileChangeEvent {
  path: string;
  ring_expanded?: number | null;
}

/**
 * Listens for hot-reload file system changes detected and indexed by the Rust watcher.
 */
export async function listenForFileChanges(
  callback: (payload: FileChangeEvent) => void
): Promise<UnlistenFn> {
  return await listen<FileChangeEvent>('mandala:file-changed', (event) => {
    callback(event.payload);
  });
}
