# Adopters and use cases

These are public reports from [upstream issue #3366: Gather adopters and use cases](https://github.com/youki-dev/youki/issues/3366).
They include uses of the libraries as well as the youki executable.

## rk8s

[rk8s](https://github.com/rk8s-dev/rk8s) is a lightweight Kubernetes-compatible orchestration system written in Rust.
It implements the Container Runtime Interface and supports single containers, Kubernetes-style Pods and Docker Compose-style applications.

Crates used: `libcontainer`, `libcgroups`.

[Project report](https://github.com/youki-dev/youki/issues/3366#issuecomment-3765275589).

## runwasi

[runwasi](https://github.com/containerd/runwasi) is a containerd shim for running WebAssembly containers with Docker and Kubernetes.
It uses `libcontainer` to build the OCI-compatible sandbox and to run native containers in the same Pod, such as sidecars.

Crate used: `libcontainer`.

[Project report](https://github.com/youki-dev/youki/issues/3366#issuecomment-3862871826).

## SpinKube

[SpinKube](https://www.spinkube.dev/) runs Spin applications on Kubernetes.
It uses `libcontainer` through runwasi.

[Project report](https://github.com/youki-dev/youki/issues/3366#issuecomment-3862871826).

## Build something of your own

See [use cases](../use-cases/index.md) for integration paths, and [customizing libcontainer](../guide/custom-runtime.md) for the executor extension point.

To add a project, comment on [#3366](https://github.com/youki-dev/youki/issues/3366) with its name, link, a short summary and the crates you use.
