import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import AnnualImpact from './AnnualImpact.vue'
import './style.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('AnnualImpact', AnnualImpact)
  }
} satisfies Theme
