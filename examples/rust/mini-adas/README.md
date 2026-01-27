# Mini ADAS Example

Example of a minimal dummy ADAS activity set.
A recorder can be added to the example and switched on in the primary via a CLI flag.

## Running

You need to run the following commands in separate terminals.

```sh
# Use 400ms cycle time
bazel run //examples/rust/mini-adas:adas_primary -- 400
```

```sh
bazel run //examples/rust/mini-adas:adas_secondary -- 1
```

```sh
bazel run //examples/rust/mini-adas:adas_secondary -- 2
```

If you want to include recording,
you need to pass the recorder's agent ID to the primary
and start the recorder.

```sh
# Use 400ms cycle time
# Wait for recorder with ID 900
bazel run //examples/rust/mini-adas:adas_primary -- 400 900
```

```sh
bazel run //examples/rust/mini-adas:adas_secondary -- 1
```

```sh
bazel run //examples/rust/mini-adas:adas_secondary -- 2
```

```sh
# Start recorder with ID 900
bazel run //examples/rust/mini-adas:adas_recorder -- 900
```

You may also use more than one recorder by specifying multiple recorder agent ids as a dot-separated
list to the primary and then start all the corresponding recorders.


```sh
# Use 400ms cycle time
# Wait for two recorders with IDs 900 and 901
bazel run //examples/rust/mini-adas:adas_primary -- 400 900.901
```

```sh
bazel run //examples/rust/mini-adas:adas_secondary -- 1
```

```sh
bazel run //examples/rust/mini-adas:adas_secondary -- 2
```

```sh
# Start recorder with ID 900
bazel run //examples/rust/mini-adas:adas_recorder -- 900
```

```sh
# Start recorder with ID 901
bazel run //examples/rust/mini-adas:adas_recorder -- 901
```


## Different signalling layer

The easiest way to switch the signalling layer is by changing the crate_features in the `BUILD.bazel`,
make sure to switch it for every target you're using. Then you can just use the commands from above.

Note that for mpsc-only signalling, there can be only a primary process without
any secondaries or recorders, because mpsc does not support inter-process signalling.

## Running tracer

In order to start tracing use:

```sh
bazel run //feo-tracer:feo_tracer -- -o out.dat
```
where `out.dat` is the tracing data output.

You can specify the tracing duration in seconds and log level using:

```sh
bazel run //feo-tracer:feo_tracer -- -d 10 -l INFO -o out.dat
```
