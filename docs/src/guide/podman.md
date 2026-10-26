# Use youki with Podman

Podman lets you select the OCI runtime by executable path.
This makes it a convenient way to try youki without changing a runtime default.

## Run one container

[Install youki](../user/basic_setup.md) at `/usr/local/bin/youki`, then run:

~~~sh
podman run --rm --runtime /usr/local/bin/youki docker.io/library/hello-world
~~~

To compare a Linux workload:

~~~sh
podman run --rm --runtime runc docker.io/library/alpine:3.21 echo hello
podman run --rm --runtime /usr/local/bin/youki docker.io/library/alpine:3.21 echo hello
~~~

Specify the full path to runc if it is not available by name.
Use the same image digest and host settings for measurements.

## Rootless operation

Podman runs rootless for an unprivileged user.
Your host needs user namespaces, appropriate `/etc/subuid` and `/etc/subgid` mappings, and working cgroup delegation.
Follow Podman's [rootless setup documentation](https://github.com/containers/podman/blob/main/docs/tutorials/rootless_tutorial.md).

If the host is not configured for rootless containers, use the rootful path for an initial comparison:

~~~sh
sudo podman run --rm --runtime /usr/local/bin/youki docker.io/library/hello-world
~~~

Run builds, pulls and containers in the same user context; rootful and rootless Podman use separate image stores.

## Choose the next step

- [Evaluate your application against runc](./switch-from-runc.md).
- [Measure lifecycle time and memory](./performance.md).
- [Run WebAssembly workloads](../user/webassembly.md) with a suitable youki build.
