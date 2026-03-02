# ------------------------------------------------------------------------------
# 构建
# ------------------------------------------------------------------------------

FROM rust AS builder

ARG FEATURES=""

WORKDIR /app

COPY . .

RUN cargo install --path . --features "$FEATURES" --root /app/install

# ------------------------------------------------------------------------------
# 打包
# ------------------------------------------------------------------------------

FROM ubuntu

WORKDIR /app

RUN apt-get update && \
    apt-get install -y tzdata

COPY --from=builder /app/.env.example ./.env
COPY --from=builder /app/config ./config
COPY --from=builder /app/sqlite ./sqlite
COPY --from=builder /app/themes ./themes
COPY --from=builder /app/install/bin/myblog /usr/local/bin/myblog

RUN chmod +x /usr/local/bin/myblog

CMD ["myblog"]
