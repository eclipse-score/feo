# Mini ADAS Example

Example of a minimal dummy ADAS activity set.
A recorder can be added to the example and switched on in the primary via a CLI flag.

## Running

You need to run the following commands in separate terminals.

```sh
# Use 400ms cycle time
bazelisk run //examples/rust/mini-adas:adas_primary -- 400
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary -- 1
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary -- 2
```

## Using middleware COM for data exchange

In order to use mw com instead of feo-com for data exchange use the following bazel targets instead:

```sh
# Use 400ms cycle time
bazelisk run //examples/rust/mini-adas:adas_primary_mw_com -- 400
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary_mw_com -- 1
```

```sh
bazelisk run //examples/rust/mini-adas:adas_secondary_mw_com -- 2
```

## Different signalling layer

The easiest way to switch the signalling layer is by changing the crate_features in the `BUILD.bazel`,
make sure to switch it for every target you're using. Then you can just use the commands from above.

Note that for mpsc-only signalling, there can be only a primary process without
any secondaries or recorders, because mpsc does not support inter-process signalling.

## Running tracer

In order to start tracing use:

```sh
bazel run //src/feo-tracer:feo_tracer -- -o out.dat
```
where `out.dat` is the tracing data output.

You can specify the tracing duration in seconds and log level using:

```sh
bazel run //src/feo-tracer:feo_tracer -- -d 10 -l INFO -o out.dat
```
