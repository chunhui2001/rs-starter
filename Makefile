
### 当前 Makefile 文件物理路径
ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

# 仅用于安装二进制包
install:
	@#cargo install sqlx-cli --no-default-features --features postgres
	cargo install

run: clear
	cargo run --bin rs-starter
	@#RUST_BACKTRACE=1 RUST_LOG=actix_web=info cargo run --bin rs-starter
	@#RUST_BACKTRACE=1 RUST_LOG=actix_web=info cargo-watch -x run --bin rs-starter

build:
	@# cargo build --release --target x86_64-unknown-linux-musl
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --release

serve:
	@#./target/debug/rs-starter
	RUST_BACKTRACE=1 RUST_LOG=actix_web=info ./target/release/rs-starter

clear:
	rm -rf src/tmp*
	rm -rf src/*/tmp*

tls:
	@#openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -sha256 -subj "/C=CN/ST=Fujian/L=Xiamen/O=TVlinux/OU=Org/CN=muro.lxd"
	openssl rsa -in key.pem -out nopass.pem
### benchmark
# make load n=10000 p=info
load:
	ab -n 10000 -c 10 "http://127.0.0.1:8000/"
