# syntax=docker/dockerfile:1.7
FROM gcr.io/distroless/cc

ARG APP_UID=65532
ARG APP_GID=65532

WORKDIR /app

# 事前ビルド済みの成果物をコピー
COPY --chown=${APP_UID}:${APP_GID}  api_server/target/release/api_server /app/api_server

# フロント成果物を公開ディレクトリへ
COPY --chown=${APP_UID}:${APP_GID}  api_server/public/ /app/public/

ENV APP_DIR_ABS=/app \
    PORT=8080 \
    GITHUB_ACCESS_TOKEN=github_pat_

USER ${APP_UID}:${APP_GID}

EXPOSE 8080
CMD ["/app/api_server"]