[group("building")]
run:
    cargo run

[group("building")]
build:
    cargo build --release

[group("testing")]
test:
    cargo test

clean:
    cargo clean
