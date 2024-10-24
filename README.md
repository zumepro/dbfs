# dbfs

Absolutely sane project. A working fuse adapter for DB-based filesystem.


## Database structure

### Table relations
<img src="er.jpg">


### Enums
1. `file_types`

| id | description  |
|----|--------------|
| -  | Regular file |
| d  | Directory    |
| l  | Symlink      |

2. `special_bits` (_column `description` omitted_)

| id | setuid | setgid | sticky |
|----|--------|--------|--------|
| 0  | 0      | 0      | 0      |
| 1  | 0      | 0      | 1      |
| 2  | 0      | 1      | 0      |
| 3  | 0      | 1      | 1      |
| 4  | 1      | 0      | 0      |
| 5  | 1      | 0      | 1      |
| 6  | 1      | 1      | 0      |
| 7  | 1      | 1      | 1      |

3. `permissions`

| id | can_read | can_write | can_execute |
|----|----------|-----------|-------------|
| 0  | 0        | 0         | 0           |
| 1  | 0        | 0         | 1           |
| 2  | 0        | 1         | 0           |
| 3  | 0        | 1         | 1           |
| 4  | 1        | 0         | 0           |
| 5  | 1        | 0         | 1           |
| 6  | 1        | 1         | 0           |
| 7  | 1        | 1         | 1           |


## Building
Build/test recipes can be viewed with `just -l`.


## Integration testing
To run integration tests:

1. Install dependencies: `podman`, `perl`, `perl-dbd-mysql`, `just`
```bash
sudo pacman -Syu podman perl perl-dbd-mysql just
```
2. Setup integration testing environment with
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
