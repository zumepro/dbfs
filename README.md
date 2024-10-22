# dbfs

Absolutely sane project. A working fuse adapter for DB-based filesystem.


### Database structure
<img src="er.jpg">


### Building
Build/test recipes can be viewed with `just -l`.
Example output:
```text
Available recipes:
    clean

    [building]
    build
    run

    [testing]
    test
    test_int
```


### Integration testing
To run integration tests:

1. Setup integration testing environment with
```bash
just prepare
```
2. Run integration tests
```bash
just test_int
```
3. Stop and clean integration testing environment
```bash
just stop
```

