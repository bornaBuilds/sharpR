const UNITS = ["B", "KB", "MB", "GB"];
const KILO = 1024;
const WHOLE_NUMBER_THRESHOLD = 100;
const PERCENT = 100;

function formatBytes(bytes: number): string {
  if (bytes < KILO) {
    return `${bytes} ${UNITS[0]}`;
  }
  let value = bytes;
  let unit = 0;
  while (value >= KILO && unit < UNITS.length - 1) {
    value /= KILO;
    unit += 1;
  }
  const digits = value >= WHOLE_NUMBER_THRESHOLD ? 0 : 1;
  return `${value.toFixed(digits)} ${UNITS[unit]}`;
}

/** Signed percentage change from `before` to `after` (negative = smaller). */
function percentChange(before: number, after: number): number {
  if (before === 0) {
    return 0;
  }
  return Math.round(((after - before) / before) * PERCENT);
}

function formatPercent(value: number): string {
  return `${value > 0 ? "+" : ""}${value}%`;
}

const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3600;

function pad2(value: number): string {
  return String(value).padStart(2, "0");
}

/** "0:07", "12:34" or "1:02:03"; an em dash when the duration is unknown. */
function formatDuration(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds) || seconds < 0) {
    return "—";
  }
  const total = Math.round(seconds);
  const hours = Math.floor(total / SECONDS_PER_HOUR);
  const minutes = Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  const secs = pad2(total % SECONDS_PER_MINUTE);
  return hours > 0 ? `${hours}:${pad2(minutes)}:${secs}` : `${minutes}:${secs}`;
}

export { formatBytes, formatDuration, formatPercent, percentChange };
