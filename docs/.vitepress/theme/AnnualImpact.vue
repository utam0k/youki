<script setup lang="ts">
import { computed, ref } from 'vue'
import { withBase } from 'vitepress'
import { annualImpact, PUBLISHED_TIMINGS } from './annual-impact.mjs'

const containers = ref<number | string>(1_000_000)
const baselineMs = ref<number | string>(PUBLISHED_TIMINGS.baselineMs)
const youkiMs = ref<number | string>(PUBLISHED_TIMINGS.youkiMs)
const presets = [
  { label: '10k', count: 10_000 },
  { label: '100k', count: 100_000 },
  { label: '1m', count: 1_000_000 },
  { label: '10m', count: 10_000_000 }
]
const number = new Intl.NumberFormat('en-US', { maximumFractionDigits: 1 })
const impact = computed(() => annualImpact(containers.value, baselineMs.value, youkiMs.value))
const isExample = computed(() =>
  Number(baselineMs.value) === PUBLISHED_TIMINGS.baselineMs &&
  Number(youkiMs.value) === PUBLISHED_TIMINGS.youkiMs
)

function duration(seconds: number) {
  const magnitude = Math.abs(seconds)
  const [value, unit] = magnitude >= 3600 ? [magnitude / 3600, 'hour'] :
    magnitude >= 60 ? [magnitude / 60, 'minute'] :
    magnitude >= 1 || magnitude === 0 ? [magnitude, 'second'] : [magnitude * 1000, 'millisecond']
  const formatted = number.format(value)
  return formatted + ' ' + unit + (formatted === '1' ? '' : 's')
}

const bars = computed(() => {
  if (!impact.value) return []
  const maximum = Math.max(impact.value.baselineSeconds, impact.value.youkiSeconds) || 1
  return [
    { label: 'Reference runtime', seconds: impact.value.baselineSeconds, width: impact.value.baselineSeconds / maximum * 100, kind: 'baseline' },
    { label: 'youki', seconds: impact.value.youkiSeconds, width: impact.value.youkiSeconds / maximum * 100, kind: 'youki' }
  ]
})
const chartLabel = computed(() => bars.value.map(bar => bar.label + ': ' + duration(bar.seconds)).join('; '))
</script>

<template>
  <section class="annual-impact" aria-label="Annual runtime impact calculator">
    <label for="annual-container-count" class="question">How many containers do you start each year?</label>
    <p id="annual-container-hint" class="hint">Include replicas, CI jobs and short-lived tasks.</p>
    <input id="annual-container-count" v-model.number="containers" type="number" min="0" step="1" inputmode="numeric" aria-describedby="annual-container-hint">
    <div class="presets" aria-label="Example yearly container counts">
      <button v-for="preset in presets" :key="preset.count" type="button" :aria-pressed="Number(containers) === preset.count" @click="containers = preset.count">{{ preset.label }}</button>
    </div>

    <details class="timings">
      <summary>Adjust the runtime timings</summary>
      <div class="timing-fields">
        <label for="annual-baseline-ms">Reference runtime (ms per container)
          <input id="annual-baseline-ms" v-model.number="baselineMs" type="number" min="0" step="any" inputmode="decimal">
        </label>
        <label for="annual-youki-ms">youki (ms per container)
          <input id="annual-youki-ms" v-model.number="youkiMs" type="number" min="0" step="any" inputmode="decimal">
        </label>
      </div>
      <p class="hint">Use equivalent lifecycle measurements from the same host and workload.</p>
    </details>

    <div v-if="impact">
      <output class="result" aria-live="polite">
        <strong data-impact-total>{{ duration(impact.deltaSeconds) }}</strong>
        <span v-if="impact.deltaSeconds > 0">less aggregate runtime time per year</span>
        <span v-else-if="impact.deltaSeconds < 0">more aggregate runtime time per year</span>
        <span v-else>of aggregate time difference per year</span>
      </output>
      <p class="formula">{{ number.format(Math.abs(impact.baselineMs - impact.youkiMs)) }} ms × {{ number.format(impact.containers) }} containers</p>
      <div class="chart" role="img" :aria-label="'Annual aggregate runtime time. ' + chartLabel">
        <p class="chart-title">Annual aggregate runtime time</p>
        <div v-for="bar in bars" :key="bar.kind" class="chart-row">
          <div class="chart-label"><span>{{ bar.label }}</span><span :data-impact-value="bar.kind">{{ duration(bar.seconds) }}</span></div>
          <div class="bar-track"><div class="bar" :class="bar.kind" :style="{ width: bar.width + '%' }"></div></div>
        </div>
      </div>
    </div>
    <p v-else class="input-error" role="alert">Enter a non-negative whole number of containers and non-negative runtime timings.</p>

    <p v-if="isExample" class="source">Historical example: youki 0.3.3 and runc 1.1.7, measured across create → start → force-delete. <a :href="withBase('/guide/performance.html#published-lifecycle-benchmark')">Versions and test conditions</a>.</p>
    <p v-else class="source">Calculated from your runtime timings. <a :href="withBase('/guide/performance.html')">How to measure them</a>.</p>
    <p class="scope">Totals add runtime time across containers, including work performed in parallel. They do not predict deployment wall-clock time, application speed or infrastructure cost.</p>
  </section>
</template>

<style scoped>
.annual-impact { padding: 24px; border: 1px solid var(--vp-c-divider); border-radius: 6px; }
.question { display: block; font-size: 18px; font-weight: 600; }
.hint, .formula, .source, .scope { color: var(--vp-c-text-2); font-size: 14px; line-height: 1.6; }
.hint { margin: 6px 0 12px; }
input { display: block; width: 100%; min-height: 44px; padding: 8px 12px; border: 1px solid var(--vp-c-divider); border-radius: 4px; background: var(--vp-c-bg); color: var(--vp-c-text-1); font: inherit; }
input:focus { outline: 2px solid var(--vp-c-brand-1); outline-offset: 2px; }
#annual-container-count { max-width: 320px; font-size: 22px; font-weight: 600; }
.presets { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 12px; }
.presets button { padding: 6px 16px; border: 1px solid var(--vp-c-divider); border-radius: 4px; font-size: 14px; }
.presets button[aria-pressed="true"] { border-color: var(--vp-c-brand-1); background: var(--vp-c-brand-soft); color: var(--vp-c-brand-1); }
.presets button:focus-visible, summary:focus-visible { outline: 2px solid var(--vp-c-brand-1); outline-offset: 2px; }
.timings { margin-top: 20px; }
summary { cursor: pointer; font-size: 14px; font-weight: 600; }
.timing-fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; margin-top: 16px; }
.timing-fields label { font-size: 14px; }
.timing-fields input { margin-top: 6px; }
.result { display: block; margin-top: 28px; }
.result strong { display: block; color: var(--vp-c-brand-1); font-size: 40px; line-height: 1.2; }
.result span { display: block; margin-top: 8px; }
.formula { margin: 8px 0 24px; }
.chart-title { margin: 0 0 16px; font-size: 14px; font-weight: 600; }
.chart-row + .chart-row { margin-top: 16px; }
.chart-label { display: flex; justify-content: space-between; gap: 12px; margin-bottom: 6px; font-size: 14px; }
.bar-track { overflow: hidden; height: 20px; border-radius: 3px; background: var(--vp-c-bg-soft); }
.bar { height: 100%; background: var(--vp-c-text-3); }
.bar.youki { background: var(--vp-c-brand-1); }
.source { margin: 24px 0 8px; }
.scope { margin: 0; }
.input-error { margin: 24px 0; font-weight: 600; }
@media (max-width: 639px) {
  .annual-impact { padding: 16px; }
  .timing-fields { grid-template-columns: 1fr; }
  .result strong { font-size: 32px; }
}
</style>
