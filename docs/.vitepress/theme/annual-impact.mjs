export const PUBLISHED_TIMINGS = Object.freeze({
  baselineMs: 224.6,
  youkiMs: 111.5
})

function numeric(value) {
  if (typeof value !== 'number' && typeof value !== 'string') return NaN
  if (typeof value === 'string' && value.trim() === '') return NaN
  return Number(value)
}

export function annualImpact(count, baseline, youki) {
  const containers = numeric(count)
  const baselineMs = numeric(baseline)
  const youkiMs = numeric(youki)
  if (!Number.isSafeInteger(containers) || containers < 0 ||
      !Number.isFinite(baselineMs) || baselineMs < 0 ||
      !Number.isFinite(youkiMs) || youkiMs < 0) return null

  const baselineSeconds = containers * baselineMs / 1000
  const youkiSeconds = containers * youkiMs / 1000
  const deltaSeconds = containers * (baselineMs - youkiMs) / 1000
  if (![baselineSeconds, youkiSeconds, deltaSeconds].every(Number.isFinite)) return null
  return { containers, baselineMs, youkiMs, baselineSeconds, youkiSeconds, deltaSeconds }
}
