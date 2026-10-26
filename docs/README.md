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
- Guide: installation, evaluation alongside runc, Docker, Podman and performance.
- Build on youki: `libcontainer` and custom executors.
- Use cases: concrete integration paths and public reports from upstream issue #3366.
- Developers / Community: existing technical references and project information.

The release updater writes the example `YOUKI_VERSION` in `src/user/basic_setup.md`.
Rust snippet imports use named `// #region` comments in their source files.
