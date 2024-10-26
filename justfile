[group("building")]
run:
    cargo run


[group("building")]
build:
    cargo build --release


[group("testing")]
[doc("Run unit tests")]
test:
    cargo test


_test_db:
    prove db_tests

[group("testing")]
[doc("Run integration tests")]
test_int: _test_db
    cargo test --features integration_testing



_prepare_run_cont:
    podman run --detach --name dbfs_intest_db --rm --env MARIADB_ALLOW_EMPTY_ROOT_PASSWORD=1 -p "127.0.0.1:3306:3306" docker.io/mariadb:latest

_wait_for_container_start:
    @echo "waiting for database socket to open";
    @for i in {10..1}; do echo -n "$i "; sleep 1; done
    @echo

_prepare_setup_db:
    #!/bin/bash
    user_creation="CREATE DATABASE \`dbfs\`; GRANT ALL PRIVILEGES ON \`dbfs\`.* TO 'dbfs'@'%' IDENTIFIED BY 'dbfs'; USE \`dbfs\`;"
    setup_file=$(cat "./sql/testing.sql")
    data_file=$(cat "./sql/dbfs_test.sql")
    quit=$(echo -e "\nEXIT;")
    echo "$user_creation$setup_file$data_file$quit" | podman exec -it dbfs_intest_db mariadb


[group("testing")]
[doc("Prepare integration testing environment")]
prepare: (_prepare_run_cont) _wait_for_container_start _prepare_setup_db


[group("testing")]
[doc("Stop and clean integration testing environment")]
stop:
    podman stop dbfs_intest_db


clean:
    cargo clean
