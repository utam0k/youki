# Build your own runtime with libcontainer

youki is also a set of Rust libraries.
Use `libcontainer` when you want container lifecycle management inside your application and control over how the workload executes.

For example, a job runner can own scheduling and policy while `libcontainer` sets up the OCI container.
A containerd shim can supply a WebAssembly executor while reusing the isolation and lifecycle machinery.

## Choose the layer you need

| Layer | What you control |
| --- | --- |
| `youki` executable | Select an OCI runtime through Docker, Podman or containerd |
| `libcontainer` | Create and manage containers from Rust; supply a workload executor |
| `libcgroups` | Manage Linux cgroup resources |
| `liboci-cli` | Reuse OCI runtime command-line structures |

Start with the [crate overview](../user/crates.md) for library references.

## Customize the workload executor

`ContainerBuilder::with_executor` accepts your implementation of the `Executor` trait.
The builder handles the container lifecycle; your executor validates and runs the workload.

This function shows the extension point. Pass your own executor implementation when constructing a container:

~~~rust
use libcontainer::container::builder::ContainerBuilder;
use libcontainer::syscall::syscall::SyscallType;
use libcontainer::workload::Executor;

pub fn builder_with_executor(executor: impl Executor + 'static) -> ContainerBuilder {
    ContainerBuilder::new("custom-workload".to_owned(), SyscallType::default())
        .with_executor(executor)
}
~~~

Continue with `.as_init(bundle_path).build()` to create an init container from an OCI bundle, then start and manage its lifecycle.
The caller supplies the root filesystem, OCI configuration and application-specific policy.

The [Executor trait](https://github.com/youki-dev/youki/blob/main/crates/libcontainer/src/workload/mod.rs) provides:

- `validate`: check whether the executor can handle the OCI spec.
- `exec`: execute the workload inside the configured container environment.
- `setup_envs`: customize environment setup when the workload needs it.

An executor must be cloneable. Validation runs after entering the container's namespaces, cgroups and root filesystem, before the start signal.
Design host-side policy checks before entering that environment.

## Plan process creation explicitly

Container creation crosses fork/clone boundaries.
When embedding `libcontainer` in a multithreaded or async application, keep process creation in a suitable dedicated helper and review what can safely run after cloning.
The [upstream discussion on safe embedding](https://github.com/youki-dev/youki/issues/3027) explains this design consideration.

Use the [libcontainer API documentation](https://docs.rs/libcontainer/) for the crate version you depend on, and check the [library migration guide](https://github.com/youki-dev/youki/blob/main/MigrationGuide.md) when upgrading.

## Projects using this approach

- **runwasi** uses `libcontainer` for an OCI-compatible sandbox for WebAssembly, and for native containers such as sidecars in the same Pod.
- **SpinKube** uses `libcontainer` through runwasi to run Spin applications on Kubernetes.
- **rk8s** uses `libcontainer` and `libcgroups` for Kubernetes-compatible orchestration in Rust.

See [the adopters and their own reports](../community/adopters_and_use_cases.md).
