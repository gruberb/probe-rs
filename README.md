# probe-rs

This service is providing `favicons` based on the given `url` parameter and in addition, is able to resize the returned `ico` based on the passed `?size` query paremter.

### Pre-requirements
Having Rust installed on your system

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation
- `git clone git@github.com:gruberb/probe-rs.git`
- `cd probe-rs`
- `cargo build --release`

### Run

Either

```bash
$ cargo run
```

or with logs
```bash
$ RUST_LOG=info cargo run
```

or the releas binary
```bash
$ ./target/release/probe-rs
```

# Example

Open your browser and query `http://localhost:3000/favicon?url=mozilla.org&size=180`

![alt text](https://github.com/gruberb/probe-rs/blob/main/example.png?raw=true)
