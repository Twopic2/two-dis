# two-dis

A Redis clone for learning Rust, asynchronous networking, and the Redis
Serialization Protocol (RESP). The roadmap follows
[Write a Redis Clone](https://rust-projects-write-a-redis-clone.github.io/).

## Basic setup

### Install a command-line client

On Arch Linux:

```sh
sudo pacman -S valkey
redis-cli --version
```

### Build and start the server

From the repository root:

```sh
cargo build 
cargo run 
```

Keep this terminal open. The server listens on `127.0.0.1:6379`; the address is

### Check the current listener

In a second terminal:

```sh
redis-cli
```

At the current stage, the server terminal prints `accepted new connection`, and
the client reports a closed or reset connection. 

## Debug database changes

Enable database snapshots when starting the server:

```sh
TWO_DIS_DEBUG_DB=1 cargo run
```

Then run commands in another terminal:

```sh
redis-cli SET user:2 Bob
redis-cli SET user:1 Ada
redis-cli SET user:2 Grace
redis-cli DEL user:1 user:2 missing
```

The server prints a snapshot to stderr after each insertion, overwrite, and
deletion attempt. Snapshots show the number of entries and sort keys so changes
are easy to compare. Quotes, line breaks, and NUL bytes are escaped. `DEL` accepts
multiple keys and returns the number of keys removed; missing or repeated keys
do not increase that count. Snapshots are disabled by default.

To try several changes without starting a server:

```sh
cargo run --example debug_database
```

In Rust code or tests, enable automatic snapshots with
`storage.set_debug_logging(true)`, or print the current database with
`eprintln!("{storage:#?}")`. For shared storage, print it while holding its lock:

```rust
eprintln!("{:#?}", &*storage.lock().await);
```

Use `cargo test -- --nocapture` to see snapshots printed during passing tests.

## CPU flamegraphs

On Arch Linux, install the profiling tools if needed:

```sh
sudo pacman -S perf
cargo install flamegraph --locked
```

Stop any existing `cargo run` server, then start profiling from the repository root:

```sh
cargo flame
```

If perf reports a permissions error, run `cargo flame --root` instead and enter
your sudo password when prompted.

The `flame` alias builds an optimized release binary with debug symbols and
samples user-space CPU time. The Linux linker flags support perf stack unwinding
while keeping Tokio console instrumentation enabled.

Wait for the server to print `Redis server listening`, then generate traffic in
another terminal:

```sh
redis-benchmark -h 127.0.0.1 -p 6379 -c 10 -n 100000 PING
```

The benchmark may warn `Could not fetch server CONFIG` because this server does
not implement `CONFIG`; the PING workload still runs.

After the benchmark finishes, press Ctrl+C in the profiling terminal and wait for
FlameGraph to finish generating the graph. Open it in a browser:

```sh
xdg-open target/flamegraph.svg
```

The SVG and raw recording (`target/perf.data`) are kept under the ignored
`target/` directory. See the [FlameGraph documentation](https://github.com/flamegraph-rs/flamegraph)
for more profiling options.

