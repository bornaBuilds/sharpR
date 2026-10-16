import { type MaxResolution, resolutionToCap } from "$lib/video/helpers";
import type {
  ConvertOptions,
  EncodeOptions,
  OutputFormat,
  VideoFormat,
  VideoOptions,
  VideoQuality,
} from "$lib/tauri/types";

const STORAGE_KEY = "sharpr.settings.v1";
const DEFAULT_MAX_DIMENSION = 1920;
const LOSSLESS_QUALITY = 100;

const DEFAULT_QUALITY: Record<OutputFormat, number> = {
  webp: 80,
  avif: 60,
  jpeg: 82,
  png: LOSSLESS_QUALITY,
};

interface FormatSettings {
  format: OutputFormat;
  quality: Record<OutputFormat, number>;
  lossless: boolean;
}

interface ConvertSettings extends FormatSettings {
  maxDimensionEnabled: boolean;
  maxDimension: number;
  recursive: boolean;
}

interface VideoSettings {
  format: VideoFormat;
  quality: VideoQuality;
  maxResolution: MaxResolution;
  removeAudio: boolean;
  recursive: boolean;
}

interface Settings {
  convert: ConvertSettings;
  crop: FormatSettings;
  video: VideoSettings;
  /** Image format for poster frames saved from the video tab. */
  poster: FormatSettings;
}

/** Settings blocks that `FormatControls` can edit. */
type FormatTarget = "convert" | "crop" | "poster";

function defaultFormatSettings(): FormatSettings {
  return { format: "webp", quality: { ...DEFAULT_QUALITY }, lossless: false };
}

function defaults(): Settings {
  return {
    convert: {
      ...defaultFormatSettings(),
      maxDimensionEnabled: false,
      maxDimension: DEFAULT_MAX_DIMENSION,
      recursive: true,
    },
    crop: defaultFormatSettings(),
    video: {
      format: "mp4",
      quality: "balanced",
      maxResolution: "original",
      removeAudio: false,
      recursive: true,
    },
    poster: defaultFormatSettings(),
  };
}

function mergeFormat<T extends FormatSettings>(base: T, saved?: Partial<T>): T {
  return {
    ...base,
    ...saved,
    quality: { ...base.quality, ...saved?.quality },
  };
}

function merge(base: Settings, saved: Partial<Settings>): Settings {
  return {
    convert: mergeFormat(base.convert, saved.convert),
    crop: mergeFormat(base.crop, saved.crop),
    video: { ...base.video, ...saved.video },
    poster: mergeFormat(base.poster, saved.poster),
  };
}

function load(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      return merge(defaults(), JSON.parse(raw) as Partial<Settings>);
    }
  } catch {
    // Corrupt or unavailable storage: fall through to defaults.
  }
  return defaults();
}

function persist(value: Settings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  } catch {
    // Storage unavailable (private mode, quota); settings stay in memory.
  }
}

function toOptions(
  fs: FormatSettings,
  maxDimension: number | null,
): ConvertOptions {
  const lossless = fs.format === "webp" && fs.lossless;
  return {
    format: fs.format,
    quality: lossless ? LOSSLESS_QUALITY : fs.quality[fs.format],
    lossless,
    maxDimension,
  };
}

class SettingsStore {
  value = $state<Settings>(load());

  constructor() {
    $effect.root(() => {
      $effect(() => {
        persist($state.snapshot(this.value));
      });
    });
  }

  get convertOptions(): ConvertOptions {
    const c = this.value.convert;
    return toOptions(c, c.maxDimensionEnabled ? c.maxDimension : null);
  }

  get cropOptions(): ConvertOptions {
    return toOptions(this.value.crop, null);
  }

  get videoOptions(): VideoOptions {
    const v = this.value.video;
    return {
      format: v.format,
      quality: v.quality,
      maxResolution: resolutionToCap(v.maxResolution),
      removeAudio: v.removeAudio,
    };
  }

  get posterOptions(): EncodeOptions {
    const { format, quality, lossless } = toOptions(this.value.poster, null);
    return { format, quality, lossless };
  }

  resetQuality(target: FormatTarget): void {
    const fs = this.value[target];
    fs.quality[fs.format] = DEFAULT_QUALITY[fs.format];
  }
}

const settings = new SettingsStore();

export { DEFAULT_QUALITY, settings };
export type {
  ConvertSettings,
  FormatSettings,
  FormatTarget,
  Settings,
  VideoSettings,
};
