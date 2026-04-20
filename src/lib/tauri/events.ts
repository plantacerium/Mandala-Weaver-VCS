import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface FileChangeEvent {
  path: string;
}

/**
 * Listens for hot-reload file system changes detected by the Rust watcher.
 */
export async function listenForFileChanges(
  callback: (payload: FileChangeEvent) => void
): Promise<UnlistenFn> {
  return await listen<FileChangeEvent>('mandala://file-changed', (event) => {
    callback(event.payload);
  });
}
