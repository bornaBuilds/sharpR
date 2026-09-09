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

export { formatBytes, formatPercent, percentChange };
