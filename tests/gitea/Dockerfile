FROM gitea/runner:3.4.2 AS runner
FROM rust:1.94-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl git jq nodejs \
    && rm -rf /var/lib/apt/lists/*

COPY --from=runner /usr/local/bin/gitea-runner /usr/local/bin/gitea-runner
WORKDIR /runner
