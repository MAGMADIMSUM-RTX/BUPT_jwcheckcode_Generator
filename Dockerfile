# syntax=docker/dockerfile:1
FROM ubuntu:latest

WORKDIR /app

# 安装依赖
RUN apt-get update && \
    apt-get install -y curl build-essential pkg-config libssl-dev && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    . "$HOME/.cargo/env" && \
    cargo install dioxus-cli

# 复制源代码
COPY . /app

# 构建项目
RUN . "$HOME/.cargo/env" && dx bundle --release

# 暴露端口
EXPOSE 2233

# 启动服务
CMD ["/root/.cargo/bin/dx", "serve", "--release", "--addr", "127.0.0.1", "--port", "2233"]
