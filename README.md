# Fixed Execution Order framework (FEO)

## Cloning the Repository

```sh
git clone https://github.com/eclipse-score/feo.git
cd feo
```


## Rust setup

You can build the Rust code with `bazel`.

### Bazel specific commands

Lint Rust code (clippy)

```sh
bazel build --config=lint-rust //...
```

## Bazel quick-start

The recommended way to run `bazel` is with [`bazelisk`][bazelisk].
On linux, this means downloading the binary from the releases page
and symlinking it to `bazel` somewhere on the `PATH`.

### Bazel examples

Build the whole workspace

```sh
bazel build //...
```

Test the whole workspace

```sh
bazel test //...
```

Query for targets

```sh
bazel query //...
```

Run example rust binary

```sh
bazel run //examples/rust/mini-adas:adas_primary_com_iox2_direct_unix
```

[bazelisk]: https://github.com/bazelbuild/bazelisk

## IDE Setup (rust-analyzer)

Because this project relies exclusively on **Bazel** for Rust, you must generate a `rust-project.json` file for `rust-analyzer` to provide features like code completion and go-to-definition.

Generate the configuration by running:
```sh
bazel run @rules_rust//tools/rust_analyzer:gen_rust_project
```

*Note: If you use VS Code, a build task is included. You can press `Ctrl+Shift+B` (or `Cmd+Shift+B`) and select **Update rust-analyzer config** to regenerate this file anytime you add or modify dependencies in your `BUILD.bazel` files.*

## Profiling

### CPU

[Samply](https://github.com/mstange/samply) is a convenient tool to profile `perf` based. The main goal is to simplify the usage of `perf` and provide a web interface to analyze the results without the need to perform manual steps.

Get your copy of *samply*:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/mstange/samply/releases/download/samply-v0.12.0/samply-installer.sh | sh
```

Profile:

```sh
bazel build -c opt //examples/rust/mini-adas:adas_primary_com_iox2_direct_unix
samply record bazel-bin/examples/rust/mini-adas/adas_primary_com_iox2_direct_unix
# ...
# <ctrl-c>
```
Samply will spawn a webserver on [https://127.0.0.1:3000](https://127.0.0.1:3000) by default. Open and enjoy the results.
The [Firefox Profiler](https://profiler.firefox.com) requires Firefox or Chrome (Safari is not supported).

### Memory

Easiest way to profile memory usage is to use
[bytehound](https://github.com/koute/bytehound). It is a tool that can be used
to profile memory usage of a binary or verify that a binary is allocation free.
It supports Linux only at the moment.

Install *bytehound* with the following commands:

```sh
wget https://github.com/koute/bytehound/releases/download/0.11.0/bytehound-x86_64-unknown-linux-gnu.tgz
tar xzf bytehound-x86_64-unknown-linux-gnu.tgz bytehound libbytehound.so
mv bytehound libbytehound.so $HOME/.cargo/bin
```

Record something with bytehound:

```sh
bazel build //examples/rust/mini-adas:adas_primary_com_iox2_direct_unix
LD_PRELOAD=$HOME/.cargo/bin/libbytehound.so bazel-bin/examples/rust/mini-adas/adas_primary_com_iox2_direct_unix
# ...
# <ctrl-c>
```

Done with recording.  Analyze the recording by

```sh
bytehound server memory-profiling_*.dat
# [2025-01-16T10:15:18Z INFO  server_core] Trying to load "memory-profiling_hello_tracing_1737022463_1792106.dat"...
# [2025-01-16T10:15:18Z INFO  cli_core::loader] Loaded data in 0s 099
# [2025-01-16T10:15:18Z INFO  actix_server::builder] Starting 96 workers
# [2025-01-16T10:15:18Z INFO  actix_server::builder] Starting server on 127.0.0.1:8080
```

Click on the [link](http://127.0.0.1:8080) in the output to open the browser and
see the results. Setup ssh port forwarding if needed when working remote (`ssh -L 8080:localhost:8080 host`).
