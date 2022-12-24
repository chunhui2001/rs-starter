
### 当前 Makefile 文件物理路径
ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

APP_NAME 	?=rs-starter
zone 		?=UTC

# 仅用于安装二进制包
install:
	@#cargo install sqlx-cli --no-default-features --features postgres
	cargo install

run: clear
	TZ=$(zone) RUST_BACKTRACE=1 RUST_LOG=actix_web=debug cargo run --bin rs-starter

Built1:
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --release

Built2:
	rustup target add x86_64-unknown-linux-musl
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl
	
Build:
	docker run --rm -it -v $(PWD):/dist:rw --name build_$(APP_NAME) chunhui2001/debian11:rust-1.66.0.slim /bin/bash -c 'cd /dist && make -f Makefile Built2' -m 6g

serve: Built1
	RUST_BACKTRACE=1 RUST_LOG=actix_web=info ./target/release/rs-starter

startup:
	cd /dist && RUST_BACKTRACE=1 RUST_LOG=actix_web=info ./target/x86_64-unknown-linux-musl/release/rs-starter

up: down Build
	docker-compose -f docker-compose.yml up -d

down:
	docker rm -f rs-starter

clear:
	rm -rf src/tmp*
	rm -rf src/*/tmp*
	rm -rf target
	#cargo clean

### 生成tls证书
tls:
	@#openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -sha256 -subj "/C=CN/ST=Fujian/L=Xiamen/O=TVlinux/OU=Org/CN=muro.lxd"
	openssl rsa -in key.pem -out nopass.pem

### 测试ssl是否工作正常
sclient:
	openssl s_client -connect 127.0.0.1:8443

### benchmark
# $ cargo install oha
# make load n=10000 p=info
load:
	oha -n 1000 http://127.0.0.1:8000 && reset && oha -n 10000 -c 100 --latency-correction --disable-keepalive http://127.0.0.1:8000

### https://github.com/chunhui2001/wrk
# $ brew install wrk
load2:
	wrk -t12 -c400 -d30s http://127.0.0.1:8000


