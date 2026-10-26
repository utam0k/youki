---
layout: home
title: A fast, lightweight OCI runtime in Rust
description: Run existing containers with youki, a fast and lightweight OCI runtime written in Rust, or customize your own runtime with libcontainer.
hero:
  name: youki
  text: A fast, lightweight OCI runtime.
  tagline: Written in Rust. Run existing containers with your usual tools, or build your own runtime with libcontainer.
  actions:
    - theme: brand
      text: Try youki
      link: /user/basic_setup
    - theme: alt
      text: Coming from runc?
      link: /guide/switch-from-runc
features:
  - title: Fast startup
    details: Reduce the overhead of creating and starting containers. Compare youki and runc on the same workload.
    link: /guide/performance
    linkText: See performance and measurements
  - title: Small memory footprint
    details: Keep runtime memory overhead lean when launching containers, including short-lived jobs.
    link: /guide/performance#memory-footprint
    linkText: Understand the footprint
  - title: Rust at the isolation boundary
    details: Memory safety helps prevent classes of bugs in code that sets up container isolation.
    link: /youki#rust-and-security
    linkText: Why Rust matters
  - title: Yours to customize
    details: Use libcontainer to control the container lifecycle and plug in your own workload executor.
    link: /guide/custom-runtime
    linkText: Build with libcontainer
---

## Keep your images. Choose your runtime.

youki implements the [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec).
Docker, Podman and containerd handle images and orchestration; youki creates and runs the container.
You can evaluate it alongside runc, one workload at a time.

After [installing youki](/user/basic_setup), try it with Podman:

~~~sh
podman run --rm --runtime /usr/local/bin/youki docker.io/library/hello-world
~~~

Using Docker? [Register youki as an additional runtime](/guide/docker), then select it with `--runtime youki`.

## What do you want to build?

| Your goal | Start here |
| --- | --- |
| Evaluate an alternative to runc without changing the default | [Switching from runc](/guide/switch-from-runc) |
| Run Linux containers with your existing tools | [Docker](/guide/docker), [Podman](/guide/podman), [Kubernetes](/user/kubernetes) |
| Build a Rust container runtime or job runner | [Customize with libcontainer](/guide/custom-runtime) |
| Execute WebAssembly workloads in an OCI sandbox | [WebAssembly](/user/webassembly) |
| Compare startup overhead and memory on your hosts | [Performance guide](/guide/performance) |

## Built on youki

[runwasi](https://github.com/containerd/runwasi) uses `libcontainer` for its OCI-compatible sandbox and native sidecars.
[SpinKube](https://www.spinkube.dev/) uses it through runwasi.
[rk8s](https://github.com/rk8s-dev/rk8s) builds Kubernetes-compatible orchestration in Rust.

These are [publicly reported use cases](https://github.com/youki-dev/youki/issues/3366), covering both the runtime and its libraries.
[Explore the use cases](/use-cases/) or [share yours](https://github.com/youki-dev/youki/issues/3366).

youki is a [CNCF Sandbox project](https://www.cncf.io/projects/youki/).
