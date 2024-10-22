[group("building")]
run:
    cargo run

[group("building")]
build:
    cargo build --release

[group("testing")]
test:
    cargo test

[group("testing")]
test_int:
    cargo test --features integration_testing

clean:
    cargo clean
