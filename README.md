## Example `no_std` Rust lib/kernel split

To build the kernel, run:
```
cargo build -Zbuild-std=core --target=x86_64-none-uk.json --release
```

To run all of the tests, run:
```
cargo test --release -- --nocapture
```
Note that the QEMU tests require the kernel to be built before being run.