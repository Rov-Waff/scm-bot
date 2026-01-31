# 使用多阶段构建
FROM rust:bookworm AS builder

# 创建工作目录
WORKDIR /usr/src/app

# 复制Cargo文件以利用Docker缓存
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建release版本
RUN cargo build --release

# 最终阶段
FROM debian:bookworm-slim

# 安装运行时依赖（如果需要）
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非root用户
RUN groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app

# 从builder阶段复制可执行文件
COPY --from=builder /usr/src/app/target/release/scm-bot /app/

# 设置文件权限
RUN chown -R appuser:appuser /app

# 切换到非root用户
USER appuser

# 运行应用
CMD ["./scm-bot"]
