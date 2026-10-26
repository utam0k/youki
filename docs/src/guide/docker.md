# Use youki with Docker

Register youki as an additional runtime, then choose it per container.
You can keep runc as Docker's default.

## Install and register

[Install youki](../user/basic_setup.md) at `/usr/local/bin/youki`.

Merge the following entry into `/etc/docker/daemon.json`.
Preserve existing keys and runtimes; create the file if it does not exist.

~~~json
{
  "runtimes": {
    "youki": {
      "path": "/usr/local/bin/youki"
    }
  }
}
~~~

On a systemd host, validate and reload Docker's configuration:

~~~sh
sudo dockerd --validate --config-file=/etc/docker/daemon.json
sudo systemctl reload docker
docker info --format '{{json .Runtimes}}'
~~~

The runtimes list should include `youki`.
If your Docker version or service configuration requires a restart, schedule it for an appropriate time.

See Docker's [runtime configuration reference](https://docs.docker.com/reference/cli/dockerd/#configure-the-default-container-runtime).

## Run and compare

~~~sh
docker run --rm --runtime youki hello-world
docker run --rm --runtime runc hello-world
~~~

Then use the same application image with both runtimes.
Check signals, volumes, resource limits and security settings, and [compare startup overhead](./performance.md).

## Return to runc

Recreate the test container with `--runtime runc`.
Containers that omit `--runtime` continue using the configured default.

Stop and remove youki-managed containers before removing its runtime registration or executable.
