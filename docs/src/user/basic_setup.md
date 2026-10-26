# Install and try youki

youki runs Linux containers on Linux. On macOS or Windows, use a Linux VM.
The examples below install the executable alongside runc and leave your default runtime unchanged.

## Install a release

Choose an archive for your architecture from the [youki releases](https://github.com/youki-dev/youki/releases).
The musl builds are static; GNU builds can require your distribution's libseccomp package.

<!--youki release begin-->
~~~sh
YOUKI_VERSION="0.7.0"
~~~
<!--youki release end-->

Run the following in a temporary working directory:

~~~sh
curl -fL "https://github.com/youki-dev/youki/releases/download/v${YOUKI_VERSION}/youki-${YOUKI_VERSION}-$(uname -m)-musl.tar.gz" -o youki.tar.gz
tar -xzf youki.tar.gz youki
sudo install -m 0755 youki /usr/local/bin/youki
/usr/local/bin/youki --version
/usr/local/bin/youki info
~~~

The source build currently requires Linux kernel 5.3 or later.
Available features also depend on kernel configuration, cgroup delegation and the build you install.
Use `youki info` to inspect the host.

## Run your first container

With Podman installed:

~~~sh
podman run --rm --runtime /usr/local/bin/youki docker.io/library/hello-world
~~~

This uses rootless Podman when run as an unprivileged user.
See the [Podman guide](../guide/podman.md) for user mappings and cgroup setup.
For a rootful trial, run the same command with `sudo`.

Using Docker instead? Follow the [Docker guide](../guide/docker.md) to register youki, then run:

~~~sh
docker run --rm --runtime youki hello-world
~~~

## Choose your path

- [Coming from runc](../guide/switch-from-runc.md): compare behavior and overhead before rollout.
- [Kubernetes](./kubernetes.md): select youki through a RuntimeClass.
- [Standalone OCI containers](./basic_usage.md): create and manage an OCI bundle directly.
- [Build a custom runtime](../guide/custom-runtime.md): use libcontainer from Rust.
- [WebAssembly](./webassembly.md): build an optional Wasm executor.

## Build from source

Install [Rust with rustup](https://www.rust-lang.org/tools/install) and [just](https://github.com/casey/just#installation).
The repository's `rust-toolchain.toml` selects the Rust toolchain.

On Debian or Ubuntu:

~~~sh
sudo apt-get install \
  pkg-config libsystemd-dev build-essential libelf-dev \
  libseccomp-dev libclang-dev libssl-dev
~~~

On Fedora, CentOS or RHEL:

~~~sh
sudo dnf install \
  pkg-config systemd-devel gcc elfutils-libelf-devel \
  libseccomp-devel clang-devel openssl-devel
~~~

Clone and build a release binary:

~~~sh
git clone https://github.com/youki-dev/youki.git
cd youki
just youki-release
./youki --version
sudo install -m 0755 ./youki /usr/local/bin/youki
~~~

### Cross compilation

With [cross-rs](https://github.com/cross-rs/cross) and its container-engine dependencies installed:

~~~sh
CARGO=cross TARGET=musl just youki-release
~~~

See `Cross.toml` in the repository for supported targets.

### Developing from another operating system

The repository includes a Vagrant-based Linux development environment:

~~~sh
vagrant up default
vagrant ssh default
~~~

Inside the VM, build from the youki checkout.
