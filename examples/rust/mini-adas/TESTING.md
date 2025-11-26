# Testing Guide

This document provides instructions for running specific test scenarios to verify core features of the FEO framework.

## Testing Error Handling

The framework includes robust error handling for activity failures. To test these scenarios, the `mini-adas` example can be run with a special feature flag and environment variables to inject failures at runtime.

### Enabling the Test Feature

All test commands must include the `--features test-error-injection` flag to compile the failure injection logic. For non-default signalling modes, `--no-default-features` must also be used, and a `com_` backend like `com_iox2` must be explicitly enabled.

### Test 1: Step Failure

**Requirement**: If an activity fails in the step function, the primary process shall call shutdown for all activities in arbitrary sequence and terminate itself.

This test is triggered by setting the `FAIL_STEP_AFTER=N` environment variable, where `N` is the number of steps to execute before failing.

#### On a Primary Agent

Inject a failure into the `Camera` activity (Activity 0), which runs on the primary agent.

*   **Relayed TCP Mode (Default):**
    ```sh
    # Terminal 1 (Primary Agent)
    FAIL_STEP_AFTER=3 cargo run -p mini-adas --features test-error-injection --bin adas_primary -- 400

    # Terminal 2 (Secondary 1)
    cargo run -p mini-adas --features test-error-injection --bin adas_secondary -- 1

    # Terminal 3 (Secondary 2)
    cargo run -p mini-adas --features test-error-injection --bin adas_secondary -- 2
    ```
*   **Direct TCP Mode:**
    ```sh
    # Terminal 1 (Primary Agent)
    FAIL_STEP_AFTER=3 cargo run -p mini-adas --no-default-features --features "signalling_direct_tcp,com_iox2,test-error-injection" --bin adas_primary -- 400

    # Terminal 2 (Secondary 1)
    cargo run -p mini-adas --no-default-features --features "signalling_direct_tcp,com_iox2,test-error-injection" --bin adas_secondary -- 1

    # Terminal 3 (Secondary 2)
    cargo run -p mini-adas --no-default-features --features "signalling_direct_tcp,com_iox2,test-error-injection" --bin adas_secondary -- 2
    ```

**Expected Result:** The primary agent will log `A failure occurred during step execution... Shutting down.` and orchestrate a system-wide shutdown.

#### On a Remote Agent

Inject a failure into the `NeuralNet` activity (Activity 2), which runs on `adas_secondary -- 1`.

*   **Relayed TCP Mode (Default):**
    ```sh
    # Terminal 1 (Primary Agent)
    cargo run -p mini-adas --features test-error-injection --bin adas_primary -- 400

    # Terminal 2 (Failing Secondary)
    FAIL_STEP_AFTER=3 cargo run -p mini-adas --features test-error-injection --bin adas_secondary -- 1

    # Terminal 3 (Other Secondary)
    cargo run -p mini-adas --features test-error-injection --bin adas_secondary -- 2
    ```

**Expected Result:** The failing secondary agent will log an `Injecting STEP failure` error. The primary agent will receive the remote failure, log `A failure occurred during step execution... Shutting down.`, and orchestrate a system-wide shutdown.

---

### Test 2: Shutdown Failure

**Requirement**: If an activity fails in the shutdown function, the primary process shall shutdown all remaining activities in arbitrary sequence and terminate itself.

This test is triggered by setting the `FAIL_ON_SHUTDOWN=1` environment variable and then stopping the primary agent with `Ctrl-C`.

#### On a Primary Agent

Inject a failure into the `Camera` activity (Activity 0). Run the commands for the primary and secondary agents, let them connect, then press `Ctrl-C` in the primary's terminal.

*   **Relayed TCP Mode (Default):**
    ```sh
    # Terminal 1 (Primary Agent)
    FAIL_ON_SHUTDOWN=1 cargo run -p mini-adas --features test-error-injection --bin adas_primary -- 400

    # (In other terminals, run the secondary agents normally)
    ```

**Expected Result:** The primary agent will log `Activity A0 failed during shutdown: Shutdown. Continuing.` and complete the shutdown process without hanging.

#### On a Remote Agent

Inject a failure into the `NeuralNet` activity (Activity 2). Run the commands, let them connect, then press `Ctrl-C` in the primary's terminal.

*   **Relayed TCP Mode (Default):**
    ```sh
    # Terminal 1 (Primary Agent)
    cargo run -p mini-adas --features test-error-injection --bin adas_primary -- 400

    # Terminal 2 (Failing Secondary)
    FAIL_ON_SHUTDOWN=1 cargo run -p mini-adas --features test-error-injection --bin adas_secondary -- 1
    ```

**Expected Result:** The failing secondary agent will log an `Injecting SHUTDOWN failure` error. The primary agent will log `Activity A2 failed during shutdown: Shutdown. Continuing.` and complete the shutdown process without hanging.

---

### Note on Unix and MPSC Sockets

*   The same tests can be performed for the `relayed_unix` and `direct_unix` backends by replacing the `signalling_direct_tcp` feature flag with `signalling_relayed_unix` or `signalling_direct_unix` in the commands above.
*   For `mpsc` mode, which runs as a single process, simply set the environment variable on the `adas_primary` command. For example:
    ```sh
    FAIL_STEP_AFTER=3 cargo run -p mini-adas --no-default-features --features "signalling_direct_mpsc,com_iox2,test-error-injection" --bin adas_primary -- 400
    ```
