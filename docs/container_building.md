Cloud Run 向けコンテナイメージ化の考慮事項（指針）

本ドキュメントは、本リポジトリのアプリケーションを Google Cloud Run で動作させる前提で、コンテナイメージ化する際の考慮点とサンプル Dockerfile をまとめたものです。

前提
- バックエンド: api_server（Rust）
- フロント: front（Vite などでビルド）
- イメージに含めたいもの:
  - api_server の release ビルド成果物（api_server/target/release/<実行ファイル>）
  - フロントのビルド成果物（front/dist 配下の静的ファイル）
- 重要: これらのビルドは podman build 実行前に人間が完了させておく（コンテナ内ではビルドしない）

---

1. Cloud Run 固有の要件
- PORT 環境変数
  - Cloud Run は PORT（通常 8080）を注入します。アプリは 0.0.0.0:$PORT で待受けが必要。
- 非 root 実行
  - 可能な限り非 root ユーザで実行（例: uid/gid=65532）。
- ライフサイクル/ログ
  - SIGTERM によるグレースフルシャットダウンを推奨。
  - ログは標準出力/標準エラーへ（Cloud Logging へ自動連携）。
- ファイルシステム
  - 書き込みは基本 /tmp のみ（エフェメラル）。それ以外は読み取り専用想定。
- ヘルスチェック
  - /healthz などを提供すると運用が容易。

2. 成果物の配置方針
- 単一コンテナで api_server が静的ファイルを配信する想定。
- 推奨パス例（コンテナ内）
  - バイナリ: /app/server（または /app/<バイナリ名>）
  - 公開ディレクトリ: /app/public（front/dist の中身）
- api_server 側の静的配信ディレクトリを /app/public に向けられるか確認。
  - もし api_server/public 固定で参照する実装なら、podman build 前に front/dist の中身を api_server/public へコピーする運用でも可。

3. 事前ビルド手順（人間が実施）
- フロント
  - cd front
  - pnpm build（または npm/yarn 相当）
  - 成果物: front/dist
- バックエンド（例）
  - cd api_server
  - cargo build --release
  - 成果物: api_server/target/release/<バイナリ名>
- 必要に応じてコピー
  - rsync -a --delete front/dist/ api_server/public/

4. ベースイメージと依存
- glibc 依存がある場合は debian:bookworm-slim 等が無難。
- 完全静的リンク（musl）にできる場合は distroless/static や scratch も選択可。
- OpenSSL など動的ライブラリ依存がある場合、ランタイムに必要なパッケージの追加を検討。

5. サンプル Dockerfile（glibc, Debian slim）

```
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
COPY --chown=appuser:appuser api_server/target/release/ /app/

# フロント成果物を公開ディレクトリへ
COPY --chown=appuser:appuser front/dist/ /app/public/

ENV PORT=8080
EXPOSE 8080

USER appuser:appuser

# バイナリ名に合わせて調整（例: private_start_page）
CMD ["/app/private_start_page"]
```

注意点
- サイズを絞る場合は、COPY を単一バイナリに限定し、余計なファイルを含めない。
- バイナリが PORT を尊重し 0.0.0.0:PORT で待受ける実装であることを確認。

6. サンプル（完全静的 + distroless）

```
# syntax=docker/dockerfile:1.7
FROM gcr.io/distroless/static:latest

WORKDIR /app
COPY api_server/target/x86_64-unknown-linux-musl/release/<BINARY_NAME> /app/server

ENV PORT=8080
USER 65532:65532
EXPOSE 8080

CMD ["/app/server"]
```

ポイント
- 動的ライブラリは使えないため、完全静的リンクが必須。

7. .dockerignore 例

```
**/.git
**/node_modules
**/target
!api_server/target/release/*
front/dist
!front/dist/**
**/.idea
**/.vscode
**/.DS_Store
```

8. ローカル検証（Podman）

```
podman build -t asia-northeast1-docker.pkg.dev/<PROJECT>/<REPO>/private-start-page:latest .

podman run --rm -p 8080:8080 -e PORT=8080 \
  --user 65532:65532 \
  asia-northeast1-docker.pkg.dev/<PROJECT>/<REPO>/private-start-page:latest
```

9. Artifact Registry へ push（例）

```
# 事前に `gcloud auth configure-docker asia-northeast1-docker.pkg.dev`
podman push asia-northeast1-docker.pkg.dev/<PROJECT>/<REPO>/private-start-page:latest
```

10. Cloud Run へデプロイ（例）

```
gcloud run deploy private-start-page \
  --image=asia-northeast1-docker.pkg.dev/<PROJECT>/<REPO>/private-start-page:latest \
  --region=asia-northeast1 \
  --platform=managed \
  --port=8080 \
  --allow-unauthenticated
```

必要に応じて
- 環境変数（--set-env-vars）
- 同時実行（--concurrency）
- メモリ/CPU（--memory, --cpu）
- 最小/最大インスタンス数

11. サーバ実装側チェックリスト
- [ ] PORT を尊重し 0.0.0.0:PORT で待受ける
- [ ] 静的配信ディレクトリを /app/public（または api_server/public）に合わせる
- [ ] ログは stdout/stderr へ
- [ ] SIGTERM で優雅に終了
- [ ] /tmp 以外への書き込み前提を置かない

12. トラブルシュート
- 8080 でローカル待受けできるか確認
- ldd /app/server で動的依存の有無を確認（glibc ベースで満たせるか）
- Cloud Run の起動ログを確認
- 非 root 権限・所有権での静的配信失敗がないか確認

以上。まずは Debian slim ベースで動作確認し、問題なければ distroless/static でのサイズ最適化を検討してください。