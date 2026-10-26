import { defineConfig } from 'vitepress'

const base = process.env.DOCS_BASE || '/youki/'
const repository = process.env.GITHUB_REPOSITORY || 'utam0k/youki'
const branch = process.env.DOCS_BRANCH || 'docs/vitepress'
const item = (text: string, link: string) => ({ text, link })

const guide = [
  { text: 'Start here', items: [
    item('Why youki?', '/youki'),
    item('Install and try youki', '/user/basic_setup'),
    item('Coming from runc', '/guide/switch-from-runc'),
    item('Performance and footprint', '/guide/performance')
  ] },
  { text: 'Run containers', items: [
    item('Docker', '/guide/docker'),
    item('Podman', '/guide/podman'),
    item('Kubernetes / containerd', '/user/kubernetes'),
    item('Standalone OCI containers', '/user/basic_usage'),
    item('WebAssembly', '/user/webassembly')
  ] },
  { text: 'Build on youki', items: [
    item('Customize with libcontainer', '/guide/custom-runtime'),
    item('Use cases', '/use-cases/'),
    item('Adopters', '/community/adopters_and_use_cases'),
    { text: 'Rust crate reference', collapsed: true, items: [
      item('Overview', '/user/crates'),
      item('libcontainer', '/user/libcontainer'),
      item('libcgroups', '/user/libcgroups'),
      item('liboci-cli', '/user/liboci_cli'),
      item('libseccomp', '/user/libseccomp')
    ] }
  ] }
]

const developer = [
  { text: 'Contribute', items: [
    item('Developer overview', '/developer/introduction'),
    item('Basics', '/developer/basics'),
    item('Conventions', '/developer/unwritten_rules'),
    item('Good places to start', '/developer/good_places_to_start'),
    item('Repository structure', '/developer/repo_structure'),
    item('Debugging', '/developer/debugging'),
    item('Work on the docs', '/developer/documentation_mdbook')
  ] },
  { text: 'Implementation', collapsed: true, items: [
    item('Crate overview', '/developer/crate_specific_information'),
    ...['libcontainer', 'libcgroups', 'liboci_cli', 'libseccomp', 'youki']
      .map(name => item(name.replace('_', '-'), '/developer/' + name))
  ] },
  { text: 'Integration tests', items: [
    item('Test overview', '/developer/e2e/e2e_tests'),
    item('Contest', '/developer/e2e/rust_oci_test'),
    item('integration_test', '/developer/e2e/integration_test'),
    item('test_framework', '/developer/e2e/test_framework'),
    item('runtimetest', '/developer/e2e/runtimetest'),
    item('containerd', '/developer/e2e/containerd_integration_test_using_youki'),
    item('runtime-tools', '/developer/e2e/runtime_tools'),
    item('runc compatibility', '/developer/e2e/runc_compatibility_test'),
    item('Kubernetes', '/developer/e2e/kubernetes_test')
  ] }
]

export default defineConfig({
  lang: 'en-US',
  title: 'youki',
  description: 'A fast, lightweight OCI runtime written in Rust. Run containers or build your own runtime with libcontainer.',
  base,
  srcDir: 'src',
  cleanUrls: false,
  lastUpdated: true,
  themeConfig: {
    siteTitle: 'youki',
    nav: [
      item('Guide', '/user/basic_setup'),
      item('Use cases', '/use-cases/'),
      item('Developers', '/developer/introduction'),
      item('Community', '/community/introduction')
    ],
    sidebar: {
      '/developer/': developer,
      '/community/': [{ text: 'Community', items: [
        item('Overview', '/community/introduction'),
        item('Adopters and use cases', '/community/adopters_and_use_cases'),
        item('Contributing', '/community/contributing'),
        item('Maintainers', '/community/maintainer'),
        item('Governance', '/community/governance'),
        item('Chat', '/community/chat')
      ] }],
      '/': guide
    },
    search: { provider: 'local' },
    socialLinks: [{ icon: 'github', link: 'https://github.com/youki-dev/youki' }],
    editLink: {
      pattern: 'https://github.com/' + repository + '/edit/' + branch + '/docs/src/:path',
      text: 'Edit this page'
    },
    outline: [2, 3],
    footer: {
      message: 'A CNCF Sandbox project. Apache-2.0 licensed. <a href="https://lfprojects.org/policies/">Project policies</a>.',
      copyright: 'Copyright Contributors to youki, a Series of LF Projects, LLC.'
    }
  }
})
