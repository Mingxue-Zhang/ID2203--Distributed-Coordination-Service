# ddbb-rust
> A distributed key-value store similar to etcd

How to run
Clone our repo, then execute `cargo build`.
After build, then execute the following command in different terminal.

```bash
cargo run --bin main -- --ip-addr 127.0.0.1:6550 --pid 1 --peer-ids 2 3 --peers-addrs 127.0.0.1:6551 127.0.0.1:6552
cargo run --bin main -- --ip-addr 127.0.0.1:6551 --pid 2 --peer-ids 1 3 --peers-addrs 127.0.0.1:6550 127.0.0.1:6552
cargo run --bin main -- --ip-addr 127.0.0.1:6552 --pid 3 --peer-ids 1 2 --peers-addrs 127.0.0.1:6550 127.0.0.1:6551
```

In our CLI,

- use `write key value` to write a value to the database
- use `read key` to read a value
- use `show` to output the current configuration