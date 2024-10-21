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
test_inet: _test_inet_sockets

_test_inet_sockets:
    #!/bin/bash
    python ./testing/echo_socket.py &
    pid_echo=$!
    sleep .1s
    cargo test --features inet_testing
    kill "$pid_echo"

clean:
    cargo clean
