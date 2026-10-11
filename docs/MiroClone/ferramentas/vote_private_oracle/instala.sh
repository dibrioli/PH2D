#!/usr/bin/env bash
# Instala o oráculo Scrumlr em ~/Apps/scrumlr-oracle, SEM root e SEM docker (o que foi feito em 11/10).
# Postgres portátil (zonky), nats-server de release, Go de go.dev, scrumlr-server compilado da tag v5.4.2.
# O build pesado passa pela fatia da linha (scripts/ph2d-run.sh).
set -euo pipefail
O="${SCRUMLR_ORACLE:-$HOME/Apps/scrumlr-oracle}"
REPO="$(cd "$(dirname "$0")/../../../.." && pwd)"
mkdir -p "$O/dl" && cd "$O/dl"
curl -sSLO https://go.dev/dl/go1.26.9.linux-amd64.tar.gz
curl -sSLO https://github.com/nats-io/nats-server/releases/download/v2.14.4/nats-server-v2.14.4-linux-amd64.tar.gz
curl -sSLO https://repo1.maven.org/maven2/io/zonky/test/postgres/embedded-postgres-binaries-linux-amd64/18.4.0/embedded-postgres-binaries-linux-amd64-18.4.0.jar
sha256sum -c - <<'EOF'
42d158b4d8f7b61ac0a830567c940a86098fb7aac52e467a5ebec03ef5cc2f8d  go1.26.9.linux-amd64.tar.gz
20f9d6a199560f243610908bcccea2e27e9f47213242d1c609ca46d1d73e91ea  nats-server-v2.14.4-linux-amd64.tar.gz
401d4e69baba9072d3772607710df0d47b39a7632c6b01b2a5f5d85a9514e212  embedded-postgres-binaries-linux-amd64-18.4.0.jar
EOF
cd "$O"
tar xzf dl/go1.26.9.linux-amd64.tar.gz
tar xzf dl/nats-server-v2.14.4-linux-amd64.tar.gz
mkdir -p pg && (cd pg && unzip -qo ../dl/embedded-postgres-binaries-linux-amd64-18.4.0.jar && tar xJf postgres-linux-x86_64.txz)
[ -d src ] || git clone --depth 1 --branch v5.4.2 https://github.com/inovex/scrumlr.io.git src
test "$(git -C src rev-parse HEAD)" = 6bae213129563eea946ec58e7a38d068dc53e0c3
cd src/server/src
PH2D_PRAZO=1200 bash "$REPO/scripts/ph2d-run.sh" env PATH="$O/go/bin:$PATH" GOPATH="$O/gopath" GOCACHE="$O/gocache" \
  GOTOOLCHAIN=local go build -o "$O/scrumlr-server" .
