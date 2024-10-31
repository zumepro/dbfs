[group("running")]
run:
    cargo run


[group("running")]
build:
    cargo build --release --no-default-features


[group("running")]
[doc("Prepare for production run")]
prepare platform:
    @if [[ {{platform}} == "mysql" ]]; then just "./sql/mysql/" prod; fi




[group("testing")]
[doc("Run unit tests")]
test platform:
    @if [[ {{platform}} == "mysql" ]]; then just _test_int_mysql; fi


_test_db_mysql:
    prove db_tests


_test_int_mysql: _test_db_mysql
    cargo test --features integration_testing


[group("testing")]
[doc("Prepare integration testing environment")]
prepare_int platform:
    @if [[ {{platform}} == "mysql" ]]; then just "./sql/mysql/" testing; fi


[group("testing")]
[doc("Stop and clean integration testing environment")]
stop:
    just "./sql/mysql/" stop




[group("running")]
[doc("Clean the directory structure")]
clean:
    cargo clean
