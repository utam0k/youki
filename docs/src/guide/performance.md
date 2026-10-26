# Startup overhead and memory footprint

youki is designed for fast container startup and a small runtime memory footprint.
These benefits matter most when containers are created frequently or hosts have limited resources.

Measure the runtime separately from image pulls, application initialization and orchestration.
Changing the OCI runtime does not by itself reduce an application's working set or speed up its computation.

## Published lifecycle benchmark

The project's [README benchmark](https://github.com/youki-dev/youki/blob/main/README.md#-motivation) reports:

| Runtime | Version | Mean lifecycle time |
| --- | --- | --- |
| youki | 0.3.3 | 111.5 ms ± 11.6 ms |
| runc | 1.1.7 | 224.6 ms ± 12.0 ms |
| crun | 1.15 | 47.3 ms ± 2.8 ms |

This is a **historical create → start → force-delete test**, not a result for the latest releases or an end-to-end application benchmark.
In this measurement youki took roughly half the time of runc; crun was faster than both.
The published host used Ubuntu 22.04.4 and Linux 6.5.0-35-generic.

Use the same commands, host and workload to make a comparison relevant to your deployment.

## Compare the current runtimes

First [prepare an OCI bundle](../user/basic_usage.md) whose process exits immediately, for example `/bin/true`.
Install [hyperfine](https://github.com/sharkdp/hyperfine) and pre-pull any images used by your container engine.

Record the environment:

~~~sh
youki --version
runc --version
youki info
uname -a
~~~

For a direct lifecycle comparison, run from the directory containing `tutorial/`:

~~~sh
sudo -v
hyperfine --warmup 10 --min-runs 100 \
  'sudo runc create -b tutorial bench-runc && sudo runc start bench-runc && sudo runc delete -f bench-runc' \
  'sudo youki create -b tutorial bench-youki && sudo youki start bench-youki && sudo youki delete -f bench-youki'
~~~

Both runtimes must use equivalent cgroup settings, OCI config and release builds.
Record failures as well as latency. Repeat with your real workloads and the Docker/containerd path you use in production.

The historical README test drops host filesystem caches before each run.
The example above uses warm caches; do not compare these results directly to that table.

## Memory footprint

Here, footprint means the memory used by the runtime while creating, starting or managing a container.
It is distinct from the memory consumed by the containerized application, container engine and shim.

Rust does not require a garbage collector or a managed language runtime.
To quantify the footprint on your host, use GNU `time` to collect maximum resident set size for equivalent commands:

~~~sh
sudo /usr/bin/time -v -o runc-create.txt \
  runc create -b tutorial memory-runc
sudo runc delete -f memory-runc

sudo /usr/bin/time -v -o youki-create.txt \
  youki create -b tutorial memory-youki
sudo youki delete -f memory-youki
~~~

Read “Maximum resident set size” in each report, repeat the measurement, and record versions and build targets.
This measures the `create` command's peak RSS and its waited-for children, not aggregate memory across every process or the application's steady-state memory.

For a service-level comparison, also measure the container engine, shim and application under equivalent concurrency and limits.
The project does not publish a current memory comparison on this page; the commands above let you obtain one for your environment.

See the [upstream memory measurement discussion](https://github.com/youki-dev/youki/issues/1515) for the distinction between runtime and application memory.
