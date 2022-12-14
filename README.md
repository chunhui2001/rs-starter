# rs-starter

### install rust development environment
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

### 环境配置: macOS配置基于sublime text4的Rust开发环境(rust-analyzer)
https://www.cnblogs.com/fijiisland/p/15635408.html

通过sublime打开一个随意的rust文件或项目文件夹，快捷键command + shift + p调出命令选项，有两种命令可选：
	-- LSP: Enable Language Server Globally 此命令会让 sublime 只要启动就加载所选的代码分析前端（不建议，会影响 sublime 的冷启动性能）
	-- LSP: Enable Language Server In Project 此命令会让 sublime 在打开当前文件/项目时才加载所选的代码分析前端，重启后需要重新操作一遍

回车后，在下拉菜单中选择 'rust-analyzer' 就完成了全部配置，一切顺利的话界面显示类似下图，左下角会显示rust-analyzer对代码进行索引分析，指针悬停于代码有相应提示：

### 如何使用VSCode配置Rust开发环境(VS Code 安装 Rust 常用插件)
https://blog.csdn.net/inthat/article/details/121519036

### cargo-watch 用于监控项目中的文件变化并运行命令。
$ cargo install cargo-watch
$ cargo install oha


### 创建项目
$ cargo new hello-rocket --bin
$ cd hello-rocket

[dependencies]
rocket = "0.5.0-rc.2"


### crate
https://crate.io/

### Docs.rs
https://docs.rs/
https://docs.rs/rocket-include-static-resources/latest/rocket_include_static_resources/

### Meet Rocket.
https://rocket.rs/

### rocket vs actix-web
https://kerkour.com/rust-web-framework-2022
https://stackshare.io/stackups/actix-vs-rocket

https://livebook.manning.com/book/rust-servers-services-and-apps/chapter-3/v-10/43

### Rust之模块和路径（一）
https://blog.csdn.net/kakadiablo/article/details/115400316

### 打包和分发一个 Rust 工具
http://llever.com/cli-wg-zh/tutorial/packaging.zh.html

https://msfjarvis.dev/posts/building-static-rust-binaries-for-linux/

### Rust 入门指南（crate 管理）
https://zhuanlan.zhihu.com/p/546235064

### Rust日常开发中的倚天屠龙
https://zhuanlan.zhihu.com/p/451494651?utm_id=0

### actix-web
https://actix.rs/
https://github.com/actix/actix-extras
https://blog.logrocket.com/building-rest-api-rust-rhai-actix-web/
https://actix.rs/docs/url-dispatch/

### api doc
https://api.rocket.rs/
https://api.rocket.rs/v0.4/rocket/http/struct.ContentType.html

### Create a blazingly fast REST API in Rust (Part 1/2)
https://hub.qovery.com/guides/tutorial/create-a-blazingly-fast-api-in-rust-part-1/

https://github.com/blurbyte/restful-rust
https://github.com/grizwako/rust-websocket-chat-server
https://github.com/steelx/rust-rocket-chat-app


### rust Configure Logging
https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
https://docs.rs/log4rs/1.2.0/log4rs/
https://docs.rs/log4rs/latest/log4rs/
https://docs.rs/log4rs/latest/log4rs/encode/pattern/
https://github.com/qoollo/rust-log4rs-logstash
https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html
https://medium.com/nikmas-group-rust/advanced-logging-in-rust-with-log4rs-2d712bb322de
https://stackoverflow.com/questions/74053633/actix-web-middleware-logger-output-to-file
https://medium.com/nerd-for-tech/logging-in-rust-e529c241f92e
https://dev.to/chaudharypraveen98/adding-slog-logger-to-actix-web-2332
https://stackoverflow.com/questions/67148186/why-custom-filter-not-working-in-log4rs1-0
https://docs.rs/actix-ip-filter/latest/actix_ip_filter/
https://github.com/jhen0409/actix-ip-filter

https://dev.to/hackmamba/build-a-rest-api-with-rust-and-mongodb-actix-web-version-ei1
https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
https://users.rust-lang.org/t/error-404-unfound-routes-actix-web/46484/3

https://github.com/secretkeysio/jelly-actix-web-starter

### rust-servers-services-and-apps
# 5 Handling errors
https://livebook.manning.com/book/rust-servers-services-and-apps/chapter-5/v-12/40
https://docs.rs/actix-web/0.5.3/actix_web/middleware/struct.ErrorHandlers.html
https://dev.to/chaudharypraveen98/error-handling-in-actix-web-4mm

### time
https://github.com/tailhook/humantime
https://doc.rust-lang.org/stable/std/time/struct.Duration.html
https://crates.io/crates/duration-string
https://docs.rs/human-repr/1.0.1/human_repr/

### Day12:Write web app with actix-web - 100DayOfRust
https://dev.to/0xbf/day11-write-web-app-with-actix-web-100dayofrust-1lkn

### Asyncifying an Actix Web App and Upgrading it to 1.0
https://www.zupzup.org/asyncify-rust-webapp/

### template
https://tera.netlify.app/docs/
https://github.com/Keats/tera

### validator
https://github.com/rambler-digital-solutions/actix-web-validator
https://dev.to/chaudharypraveen98/form-validation-in-rust-404l

https://docs.rs/actix-ratelimit/latest/actix_ratelimit/
https://blog.logrocket.com/the-state-of-rust-http-clients/

### 细说 Rust 错误处理
https://www.modb.pro/db/137984

### Build a Rust speed test using Actix and WebSockets
https://www.koyeb.com/tutorials/build-a-rust-speed-test-using-actix-and-websockets

### WebSockets in Actix Web Full Tutorial — WebSockets & Actors
https://levelup.gitconnected.com/websockets-in-actix-web-full-tutorial-websockets-actors-f7f9484f5086

### How to Build a REST API using Actix Rust to Execute System Commands — A Step-by-Step Guide
https://codeburst.io/how-to-build-a-rest-api-to-execute-system-commands-using-actix-rust-a-step-by-step-guide-e257d5442b16

### Developing High Performance Apache Cassandra™ Applications in Rust (Part 1)
https://www.datastax.com/blog/2021/03/developing-high-performance-apache-cassandra-applications-rust-part-1

### Developing High-Performance Cassandra Applications in Rust (Part 2)
https://medium.com/building-the-open-data-stack/developing-high-performance-cassandra-applications-in-rust-part-2-10448858f29










