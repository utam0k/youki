# Why youki? {#youki}

youki is a fast, lightweight implementation of the [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec), written in Rust.
Use it as the runtime behind your container tools, or use its libraries to build an execution platform of your own.

## Motivation

Container startup is a repeated cost. It matters for short-lived jobs, development environments and platforms that launch containers frequently.
youki focuses on low runtime overhead while keeping the OCI interface familiar.

- **Fast startup:** the project's published lifecycle benchmark shows lower overhead than runc in that test. See the [versions, conditions and a way to reproduce a comparison](./guide/performance.md).
- **Small memory footprint:** Rust lets the runtime operate without a garbage-collected language runtime. Measure runtime memory separately from your application's memory; the [performance guide](./guide/performance.md#memory-footprint) explains how.
- **Reusable isolation:** `libcontainer` exposes container creation and lifecycle management to Rust programs. A custom `Executor` lets you choose what runs inside the container.

## Rust and security

A container runtime processes configuration and sets up namespaces, mounts, capabilities and process boundaries.
Rust's memory safety helps prevent classes of memory corruption bugs in this code.
That is a useful foundation as vulnerability discovery and exploit development become increasingly automated.

This advantage belongs to the runtime implementation: runc is also implemented in a memory-safe language, Go.
Rust adds a different systems-programming model, with ownership and no garbage collector, for implementing process and syscall-heavy code.

Container isolation still depends on the Linux kernel, correct configuration, dependencies and the runtime's unsafe code.
Use supported releases and follow the project's [security policy](https://github.com/youki-dev/youki/security/policy) for vulnerability reporting.

## Familiar tools, an extensible core

| Use youki as a runtime | Build with libcontainer |
| --- | --- |
| Keep your OCI images and container tooling | Integrate container lifecycle management into a Rust application |
| Select youki per container or per Pod | Supply your own workload executor |
| Compare behavior with runc before rolling out | Reuse Linux isolation setup for a specialized runtime |
| [Evaluate alongside runc](./guide/switch-from-runc.md) | [Build a custom runtime](./guide/custom-runtime.md) |

See [real projects using youki and its crates](./community/adopters_and_use_cases.md).

## About the name

youki is pronounced /joʊki/, or “yoh-key”.
It comes from the Japanese word for “container”; it can also mean cheerful or merry.
