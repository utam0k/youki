# Run an OCI container directly

Use the youki CLI when you want to work with an OCI bundle rather than a container engine.
For image pulls, networking and everyday container management, start with [Docker](../guide/docker.md) or [Podman](../guide/podman.md).

## Prepare a bundle

An OCI bundle contains a root filesystem and `config.json`.
This example uses Docker to export a BusyBox root filesystem, and Python 3 to edit the JSON.

~~~sh
mkdir -p tutorial/rootfs
container_id=$(docker create busybox:1.37)
docker export "$container_id" | tar -C tutorial/rootfs -xf -
docker rm "$container_id"
cd tutorial
youki spec
~~~

Configure a process that runs long enough to inspect, with no attached terminal:

~~~sh
python3 - <<'PY'
import json
from pathlib import Path

path = Path("config.json")
spec = json.loads(path.read_text())
spec["process"]["terminal"] = False
spec["process"]["args"] = ["sleep", "30"]
path.write_text(json.dumps(spec, indent=2) + "\n")
PY
~~~

## Create, start and inspect

Run these commands from the `tutorial` directory:

~~~sh
sudo youki create --bundle . tutorial_container
sudo youki state tutorial_container
sudo youki start tutorial_container
sudo youki list
~~~

`create` prepares the container. `start` begins the configured process.
After inspection, stop and remove the container:

~~~sh
sudo youki delete --force tutorial_container
~~~

For a benchmark bundle, change `process.args` to `["/bin/true"]` instead.
See [measuring performance](../guide/performance.md).

## Rootless containers

Rootless operation needs a host configured for user namespaces, UID/GID mappings and cgroup delegation.
Generate the corresponding OCI configuration with:

~~~sh
youki spec --rootless
~~~

Set the desired process in `config.json`, then use `youki create`, `state`, `start`, `list` and `delete` without `sudo`.
Use a new container ID and the same user for the entire lifecycle.

For a higher-level rootless workflow, see [Podman](../guide/podman.md).

## Logging

Release builds default to the `error` log level; debug builds default to `debug`.
Use the global `--log-level` option to select a level, such as `debug` or `trace`.

The `--debug` flag is available for compatibility with runc and crun.
An explicit `--log-level` takes precedence.
