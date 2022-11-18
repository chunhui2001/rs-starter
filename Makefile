
### 当前 Makefile 文件物理路径
ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

run:
	cargo run

build:
	@# cargo build --release --target x86_64-unknown-linux-musl
	cargo build --release

start:
	@#./target/debug/rs-starter
	./target/release/rs-starter

### benchmark
# make load n=10000 p=info
load:
	ab -n 100000 -c 10 "http://127.0.0.1:8000/"
