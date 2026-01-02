# logfmt

A lightweight parser for [logfmt](https://brandur.org/logfmt).

## Usage
```rust
use logfmt::Logfmt;

let line = r#"level=info msg="request completed" duration=42ms"#;

for (key, value) in line.logfmt() {
    println!("{key}: {value}");
}
```

## What is logfmt?

Logfmt is a logging format that encodes data as space-separated key-value pairs:
```
level=info msg="hello world" count=42
```

It's human-readable, easy to parse, and widely used in logging.

## License

MIT
