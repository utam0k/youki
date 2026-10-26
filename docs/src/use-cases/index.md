# What can you build with youki?

Use the youki executable with existing container tools, or build an execution platform from its Rust libraries.

## Run existing Linux containers

**For:** teams evaluating OCI runtimes for Docker, Podman or Kubernetes.

Keep the application image and choose youki for a test container or Pod.
Compare compatibility, startup overhead and runtime memory on your hosts before expanding.

Start with [switching from runc](../guide/switch-from-runc.md), then choose [Docker](../guide/docker.md), [Podman](../guide/podman.md) or [Kubernetes](../user/kubernetes.md).

## Launch short-lived jobs

**For:** build workers, task runners and platforms that create containers frequently.

Runtime startup and cleanup become repeated costs when jobs are short.
Use the [performance guide](../guide/performance.md) to measure lifecycle overhead separately from image pulls and application work.

For a Rust-native job runner, [embed libcontainer](../guide/custom-runtime.md) and keep scheduling and application policy in your own code.

## Build a custom runtime in Rust

**For:** platform developers who need a different execution model or direct control of the container lifecycle.

Use `libcontainer` for OCI container setup and supply an `Executor` for workload-specific validation and execution.
Use `libcgroups` when you need resource management as a separate building block.

[rk8s](https://github.com/rk8s-dev/rk8s) is a reported example: a Kubernetes-compatible orchestration system in Rust using `libcontainer` and `libcgroups`.
[Build with libcontainer](../guide/custom-runtime.md).

## Run WebAssembly with container tooling

**For:** platforms integrating Wasm execution with OCI isolation and Kubernetes.

[runwasi](https://github.com/containerd/runwasi) uses `libcontainer` for its OCI-compatible sandbox and native sidecars.
[SpinKube](https://www.spinkube.dev/) uses it through runwasi.

The youki executable also has optional Wasm executors, enabled at build time.
See the [WebAssembly guide](../user/webassembly.md) for feature flags, image requirements and a Podman example.
The library and CLI integration paths are distinct; choose the one your platform needs.

## Public reports and your own use case

These project examples come from [upstream issue #3366](https://github.com/youki-dev/youki/issues/3366).
Read the [adopter reports](../community/adopters_and_use_cases.md) for the source of each example.
Add your project to that issue with a link, a short summary and the crates you use.
