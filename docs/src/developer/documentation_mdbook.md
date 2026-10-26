# Work on this documentation

The site uses [VitePress](https://vitepress.dev/) and is published with GitHub Pages.
Markdown sources live in `docs/src`; navigation and theme settings live in `docs/.vitepress/config.ts`.

## Develop locally

Install Node.js 22 or later, then run:

~~~sh
cd docs
npm ci
npm run dev -- --host 0.0.0.0
~~~

For a production build and local preview:

~~~sh
npm run build
npm run check:links
npm run preview -- --host 0.0.0.0
~~~

The build fails on broken page links.
The link check also checks fragment anchors, navigation and local assets in generated HTML.

## Add or change a page

Write Markdown in `docs/src` and add the navigation entry in `docs/.vitepress/config.ts`.
Keep existing paths when possible so external links continue to work.
This page retains its old filename for that reason.

For source code examples, use VitePress snippet imports:

~~~text
<<< ../../../../tests/contest/contest/src/tests/example/hello_world.rs#get_example_spec
~~~

Named snippets use `// #region name` and `// #endregion name` comments in the Rust source.

## Publish

The documentation workflow builds pull requests and pushes to `main`.
It also publishes the fork's `docs/vitepress` branch for review.
GitHub Pages must use GitHub Actions as its build source.

The output is `docs/.vitepress/dist`.
The base path defaults to `/youki/`; set `DOCS_BASE` when building for another location.
