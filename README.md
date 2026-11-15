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
currently hardcoded in `src/main.rs`. It prints a message only after a client
connects. Press `Ctrl+C` to stop it.

### Check the current listener

In a second terminal:

```sh
redis-cli
```

At the current stage, the server terminal prints `accepted new connection`, and
the client reports a closed or reset connection. 

