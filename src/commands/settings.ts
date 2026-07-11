import { loadSettings, saveSettings, type AppSettings, DEFAULT_SETTINGS } from '../tauri-commands';

/** Load saved settings from the Tauri app config directory. */
export async function loadAppSettings(): Promise<AppSettings> {
  try {
    const saved = await loadSettings();
    return saved ?? DEFAULT_SETTINGS;
  } catch {
    return DEFAULT_SETTINGS;
  }
}

/** Save settings to the Tauri app config directory. */
export async function saveAppSettings(settings: AppSettings): Promise<void> {
  return saveSettings(settings);
}
