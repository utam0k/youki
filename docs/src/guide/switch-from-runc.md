# Switching from runc

Start by adding youki alongside runc. Keep your current default runtime while you compare a representative workload.

youki implements the OCI Runtime Specification. Your image and application can stay the same; you change the low-level runtime selected by Docker, Podman or containerd.
Runtime-specific flags, generated defaults and host integration still need to be checked.

## 1. Install alongside runc

[Install a youki release](../user/basic_setup.md) on a Linux host.
Use a distinct executable path such as `/usr/local/bin/youki`; keep the existing `runc` executable and runtime configuration.

~~~sh
/usr/local/bin/youki --version
/usr/local/bin/youki info
runc --version
~~~

Record versions and the host's cgroup configuration so comparisons are useful.

## 2. Select it for one workload

| Tool | Select youki | Guide |
| --- | --- | --- |
| Docker | `docker run --runtime youki …` after registering it | [Docker](./docker.md) |
| Podman | `podman run --runtime /usr/local/bin/youki …` | [Podman](./podman.md) |
| Kubernetes / containerd | Configure a runtime handler and use `runtimeClassName: youki` | [Kubernetes](../user/kubernetes.md) |
| Your own Rust platform | Use `ContainerBuilder` and an `Executor` | [libcontainer](./custom-runtime.md) |

For a local Podman comparison:

~~~sh
podman run --rm --runtime runc docker.io/library/alpine:3.21 echo hello
podman run --rm --runtime /usr/local/bin/youki docker.io/library/alpine:3.21 echo hello
~~~

Use the same image digest for measurements. Rootless operation needs user namespaces, UID/GID mappings and suitable cgroup delegation; see the [Podman guide](./podman.md).

## 3. Check the behavior your application needs

Run your existing tests with each runtime. Include:

- Start, exec, exit status, signals and cleanup.
- Volumes, read-only mounts and the namespaces your application uses.
- CPU, memory and PID limits; the cgroup driver used on your hosts.
- Seccomp, capabilities, AppArmor or SELinux policies that apply to the workload.
- Rootless operation and user mappings, if you use them.

The project runs [runc compatibility tests](../developer/e2e/runc_compatibility_test.md) and [containerd integration tests](../developer/e2e/containerd_integration_test_using_youki.md).
These are useful evidence alongside tests of your own deployment.

## 4. Compare overhead, then expand

Measure both the runtime lifecycle and the end-to-end path through your container engine.
The [performance guide](./performance.md) explains how to compare startup latency and memory without mixing in image pulls.

Expand from a development host to a small set of workloads or dedicated nodes.
Switch a default only after those workloads meet your behavior and performance requirements.

## Returning to runc

For Docker, recreate the test container with `--runtime runc`.
For Podman, select `--runtime runc` or your previous runtime path.
For Kubernetes, recreate the workload using your previous RuntimeClass, or omit `runtimeClassName` when runc is the cluster default.

A running container cannot change runtime in place. Keep the original runtime configuration until all youki-managed workloads are stopped and cleaned up.

## Need behavior beyond a CLI runtime?

Use [libcontainer](./custom-runtime.md) to embed the lifecycle in Rust and provide a custom executor.
[runwasi and rk8s](../community/adopters_and_use_cases.md) show what projects build on that foundation.
