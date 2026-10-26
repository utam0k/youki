import test from 'node:test'
import assert from 'node:assert/strict'
import { annualImpact, PUBLISHED_TIMINGS } from '../.vitepress/theme/annual-impact.mjs'

test('published lifecycle example converts one million containers to aggregate hours', () => {
  const result = annualImpact(1_000_000, PUBLISHED_TIMINGS.baselineMs, PUBLISHED_TIMINGS.youkiMs)
  assert.ok(Math.abs(result.baselineSeconds / 3600 - 62.3888888889) < 1e-8)
  assert.ok(Math.abs(result.youkiSeconds / 3600 - 30.9722222222) < 1e-8)
  assert.ok(Math.abs(result.deltaSeconds / 3600 - 31.4166666667) < 1e-8)
})

test('zero starts and equal timings have zero difference', () => {
  assert.equal(annualImpact(0, 224.6, 111.5).deltaSeconds, 0)
  assert.equal(annualImpact(1000, 20, 20).deltaSeconds, 0)
  assert.equal(annualImpact(1000, 0, 0).baselineSeconds, 0)
})

test('slower youki measurements retain the additional time', () => {
  assert.equal(annualImpact(1000, 10, 20).deltaSeconds, -10)
  assert.equal(annualImpact(1000, 0, 20).deltaSeconds, -20)
})

test('fractional millisecond timings and larger volumes scale correctly', () => {
  const small = annualImpact(1000, 1.1, 0.1)
  const large = annualImpact(10000, 1.1, 0.1)
  assert.equal(small.deltaSeconds, 1)
  assert.equal(large.deltaSeconds, 10)
})

test('blank, fractional counts and invalid timings do not become estimates', () => {
  for (const count of ['', ' ', -1, 1.5, Infinity, null, true, Number.MAX_SAFE_INTEGER + 1]) {
    assert.equal(annualImpact(count, 10, 5), null)
  }
  for (const timing of ['', -1, Infinity, NaN, null, 'not a number']) {
    assert.equal(annualImpact(1000, timing, 5), null)
    assert.equal(annualImpact(1000, 10, timing), null)
  }
  assert.equal(annualImpact(1000, Number.MAX_VALUE, 1), null)
})
