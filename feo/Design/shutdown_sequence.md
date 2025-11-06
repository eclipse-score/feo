# FEO Shutdown Sequence Design

This document outlines the graceful shutdown process for the FEO framework, covering both `direct` and `relayed` signalling modes. The process is designed to be robust, ensuring that all components terminate cleanly.

The shutdown is orchestrated by the `Scheduler` and is divided into two distinct phases.

## Initiation via OS Signal (e.g., Ctrl-C)

The primary agent is designed to handle OS termination signals (like `SIGINT` from Ctrl-C) to ensure a clean exit. This is achieved using a shared `Arc<AtomicBool>` flag.
1.  **Setup**: On startup, the `Primary` agent sets up a signal handler using the `ctrlc` crate.
2.  **Signal Received**: When a termination signal is received, the handler atomically sets the shared boolean flag to `true`.
3.  **Detection**: The `Scheduler`'s main `run` loop checks this flag on every cycle. When it detects the flag is `true`, it breaks its loop.
4.  **Graceful Shutdown**: After breaking the loop, `Scheduler::shutdown_gracefully()` is called, initiating the two-phase shutdown process described below.

## Phase 1: Graceful Activity Shutdown

This phase focuses on stopping the application-level logic (the activities).

1.  **Initiation**: The shutdown process begins when `Scheduler::shutdown_gracefully()` is called. This can be triggered in two main ways:
    -   **Programmatic Shutdown**: The application logic can decide to end execution and call this function (e.g., after a fixed number of cycles, timeouts,....).
    -   **OS Signal (e.g., Ctrl-C)**: The primary agent is designed to handle OS termination signals. When a signal like `SIGINT` is received, a shared `Arc<AtomicBool>` flag is set to `true`. The `Scheduler`'s main `run` loop checks this flag on every cycle, and when it detects the change, it breaks its loop and subsequently calls `shutdown_gracefully()`.

2.  **Identify Active Components**: The scheduler first identifies all activities that have successfully started. An activity is considered "started" if it has sent a `Ready` signal at least once (`ever_ready == true`).

3.  **Send `Shutdown` Signal**: The scheduler sends a `Signal::Shutdown((ActivityId, Timestamp))` to each started activity.
    -   The `Scheduler` also forwards this `Shutdown` signal to all connected `Recorder` agents. This is for logging purposes; the recorder's only action is to write this event to the recording file.

4.  **Worker Response**:
    -   Upon receiving the `Shutdown` signal, a `Worker` thread calls the `shutdown()` method on the corresponding `Activity`.
    -   After the activity's shutdown logic completes, the `Worker` sends a `Signal::Ready((ActivityId, Timestamp))` back to the scheduler. This signal acts as a confirmation that the activity has shut down cleanly.

5.  **Scheduler Waits for Confirmation**: The scheduler waits until it has received a `Ready` signal from every activity it sent a `Shutdown` signal to.
    -   **Crucially, the scheduler does *not* wait for any response from recorders during this phase.** It only tracks acknowledgements from activities.
    -   A timeout is in place to prevent the system from hanging if an activity fails to respond.

## Phase 2: System-Wide Termination

Once all activities are confirmed to be shut down, this phase terminates all agent processes.

1.  **Broadcast `Terminate` Signal**: The scheduler calls `broadcast_terminate()`, which sends a `Signal::Terminate(Timestamp)` to all connected agents.
    -   **Direct Mode**: The `SchedulerConnector` sends the `Terminate` signal directly to every connected worker and recorder socket.
    -   **Relayed Mode**: The `SchedulerConnector` sends the `Terminate` signal to its local `PrimarySendRelay` (for remote agents) and its local workers. The relays are responsible for broadcasting the signal over the network.

2.  **Agent Response**:
    -   When a `Worker` receives the `Terminate` signal, it immediately sends a `Signal::TerminateAck(AgentId)` back to the scheduler.
    -   When a `FileRecorder` receives the `Terminate` signal, it also sends a `Signal::TerminateAck(AgentId)` back.
    -   After sending the `TerminateAck`, both `Worker` and `FileRecorder` threads will **sleep for 100ms**. This brief pause is critical to ensure the `TerminateAck` message has time to be transmitted over the network before the thread and its associated socket are destroyed.
    -   After the sleep, the thread exits (`run()` method returns).

3.  **Scheduler Waits for Final Acknowledgement**: The scheduler waits until it has received a `TerminateAck` from all connected remote agents.
    -   Once all acknowledgements are received (or a timeout occurs), the `scheduler.run()` method returns, signalling the end of its lifecycle.
    -   **Note**: The confirmation is agent-based. The scheduler tracks `AgentId`s, not individual `WorkerId`s. A single `TerminateAck` from any worker on a remote agent is sufficient to confirm that the entire agent is terminating.

## Final Cleanup

1.  **Primary Agent**: Once `scheduler.run()` returns, the main thread of the `Primary` agent proceeds to `join()` all of its local `Worker` threads (and `Relay` threads in relayed mode). This ensures all local threads have exited cleanly. The primary agent process then terminates.

2.  **Secondary Agents**: The `Worker` threads on secondary agents will have already exited upon receiving the `Terminate` signal. This allows the main thread of the `Secondary` agent to `join()` its worker threads and terminate cleanly.

This two-phase process ensures that data processing is stopped gracefully before the underlying processes and communication channels are torn down, preventing data corruption and ensuring a clean system exit.
