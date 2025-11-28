# syntax=docker/dockerfile:1.7
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG APP_UID=65532
ARG APP_GID=65532
RUN groupadd -g ${APP_GID} appuser \
 && useradd -u ${APP_UID} -g ${APP_GID} -M -s /usr/sbin/nologin appuser \
 && mkdir -p /app/public \
 && chown -R appuser:appuser /app

WORKDIR /app

# 事前ビルド済みの成果物をコピー（必要に応じて単一ファイルに限定）
COPY --chown=appuser:appuser api_server/target/release/ /app/release/

# フロント成果物を公開ディレクトリへ
COPY --chown=appuser:appuser front/dist/ /app/public/

ENV PORT=8080
EXPOSE 8080

ENV APP_DIR_ABS=/app

ENV GITHUB_ACCESS_TOKEN=github_pat_

USER appuser:appuser

CMD ["/app/release/api_server"]