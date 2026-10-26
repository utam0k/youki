# youki documentation

The site uses VitePress 1.x with Markdown in `src/`.
Navigation lives in `.vitepress/config.ts` and the small theme customization in `.vitepress/theme/`.

## Local development

Use Node.js 22 or later:

```sh
cd docs
npm ci
npm run dev
```

Before publishing:

```sh
npm run build
npm run check:links
```

The link checker validates generated page paths, assets and fragment anchors.
The site retains the existing `.html` URLs. Existing developer and community pages remain available.

Vite is overridden to patched 6.4.4 while VitePress stays on stable 1.x.
The Vue plugin supports Vite 6; build and link checks verify this combination.
Commit `package-lock.json` when updating dependencies.

## Publish the fork

1. Work on `docs/vitepress`.
2. Set the repository's **Settings → Pages → Source** to **GitHub Actions**.
3. Push the branch; the Documentation workflow builds and publishes it to GitHub Pages.
4. The deployment job links to the published site.

The workflow also builds documentation pull requests and publishes `main`.
The `docs/vitepress` preview deployment is restricted to `utam0k/youki`.

The default base path is `/youki/`. Set `DOCS_BASE=/` to build for a root domain.
The workflow passes the current source branch through `DOCS_BRANCH` for edit links.

## Content structure

- Home: startup overhead, footprint, Rust memory safety, and customization.
- Guide: installation, adoption with existing tools, Docker, Podman and performance.
- Build on youki: `libcontainer` and custom executors.
- Use cases: concrete integration paths and public reports from upstream issue #3366.
- Developers / Community: existing technical references and project information.

The release updater writes the example `YOUKI_VERSION` in `src/user/basic_setup.md`.
Rust snippet imports use named `// #region` comments in their source files.

## Brand assets and diagrams

The youki icon and the CNCF light/dark wordmarks are unmodified SVGs from [cncf/artwork](https://github.com/cncf/artwork/tree/b4b5989332c822ef09d5b38a512e2b39c63e32f2):

- `projects/youki/icon/color/youki-icon-color.svg`
- `other/cncf/horizontal/color/cncf-color.svg`
- `other/cncf/horizontal/white/cncf-white.svg`

They live in `src/public/branding/`. CNCF project status links to the [official youki project page](https://www.cncf.io/projects/youki/).
The shared footer is visible on documentation pages as well as the homepage.

The runtime diagrams in `src/assets/runtime-flow*.svg` explain the CLI and library integration paths.
The mobile version keeps labels readable on narrow screens.

## Annual impact calculator

The home and performance pages share `AnnualImpact.vue`.
The initial values come from the historical README lifecycle benchmark; users can replace both timings.
The result is aggregate runtime time across containers, not end-to-end deployment latency or a cost forecast.

Run the numerical checks with `npm run test:impact`.
The calculator retains negative differences when a user's youki timing is slower, and rejects invalid inputs.
The adoption guide lives at `guide/adopt-youki.md`; the previous URL remains as a compatibility page.
