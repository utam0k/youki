---
layout: home
title: Fast container starts and a Rust core you can customize
description: Start containers with a fast, lightweight OCI runtime written in Rust. Keep your existing tools, or build your own execution platform with libcontainer.
hero:
  name: youki
  text: Fast container starts. A Rust core you can customize.
  tagline: A lightweight OCI runtime for your existing container tools, with libcontainer for building your own execution platform.
  image:
    src: /branding/youki.svg
    alt: youki
  actions:
    - theme: brand
      text: Try youki
      link: /user/basic_setup
    - theme: alt
      text: Build with libcontainer
      link: /guide/custom-runtime
features:
  - title: Fast startup
    details: Reduce the runtime work of creating and starting containers. Useful for CI workers, batch jobs and other frequently launched workloads.
    link: /guide/performance
    linkText: See measurements and conditions
  - title: Customize in Rust
    details: Embed libcontainer, control the container lifecycle and supply your own workload executor.
    link: /guide/custom-runtime
    linkText: Build your execution platform
  - title: Small memory footprint
    details: Keep runtime overhead lean. Measure the runtime's memory separately from your application's working set.
    link: /guide/performance#memory-footprint
    linkText: Explore the footprint
  - title: Rust at the isolation boundary
    details: Rust's memory safety helps prevent classes of bugs in code that prepares container isolation.
    link: /youki#rust-and-security
    linkText: Why Rust matters
---

## Try youki with your existing tools

Keep your container images and the tools you already use.
Docker, Podman and containerd handle images and orchestration; youki implements the [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec) to create and run the container.

After [installing youki](/user/basic_setup), run your first container with Podman:

~~~sh
podman run --rm --runtime /usr/local/bin/youki docker.io/library/hello-world
~~~

Use the [Docker guide](/guide/docker) or [Kubernetes guide](/user/kubernetes) for those environments.
[Adopt youki](/guide/adopt-youki) walks through adding it alongside your current runtime and trying one workload at a time.

## Build your own runtime in Rust

Keep scheduling and application policy in your own code.
Use `libcontainer` to create and manage the container, and provide an `Executor` that validates and runs your workload.

The extension point is explicit:

~~~rust
ContainerBuilder::new("my-workload".to_owned(), SyscallType::default())
    .with_executor(my_executor);
~~~

[Build with libcontainer](/guide/custom-runtime) for the builder setup, executor contract and container lifecycle.
Explore the [Rust crates](/user/crates) for the building blocks you need.

<picture class="runtime-diagram">
  <source media="(max-width: 639px)" srcset="./assets/runtime-flow-mobile.svg" width="480" height="600">
  <img src="./assets/runtime-flow.svg" alt="Existing tools select the youki CLI. Your Rust runtime can embed libcontainer and supply a custom Executor. Both paths use the same Rust core to set up isolation with the Linux kernel." width="1100" height="360" loading="lazy">
</picture>

## Built on the same Rust core

| Project | What it builds with youki's libraries |
| --- | --- |
| [runwasi](https://github.com/containerd/runwasi) | An OCI-compatible sandbox for WebAssembly and native sidecars, using `libcontainer`. |
| [SpinKube](https://www.spinkube.dev/) | Spin applications on Kubernetes, using `libcontainer` through runwasi. |
| [rk8s](https://github.com/rk8s-dev/rk8s) | Kubernetes-compatible orchestration in Rust, using `libcontainer` and `libcgroups`. |

These are [public reports from the projects](https://github.com/youki-dev/youki/issues/3366).
Read the [adopter reports](/community/adopters_and_use_cases) or explore [use cases](/use-cases/) for your own platform.

## Fast startup, measured on your workload

Container setup is a repeated cost for short-lived jobs and platforms that launch containers frequently.
youki focuses on low runtime overhead at that boundary.

The [performance guide](/guide/performance) includes published lifecycle measurements with their versions and test conditions, plus steps to compare lifecycle time and memory on your host.

Once you have comparable timings, [estimate their yearly impact](/guide/performance#estimate-the-yearly-impact) using your own container count.

## Project and community

youki is a [CNCF Sandbox project](https://www.cncf.io/projects/youki/).
Meet the [maintainers](/community/maintainer), read the [governance](/community/governance), and see how to [contribute](/community/contributing).
The project's [security policy](https://github.com/youki-dev/youki/security/policy) explains vulnerability reporting.
