#!/bin/bash
# ⭐ W5w — o CUSTO da luz no APP INTEIRO (o número do Inspector), N processos, numa tela VIRTUAL.
#
# uso:
#   bash docs/Luz2D/ferramentas/mede_custo_no_app.sh <largura> <altura> <nivel low|medium|high> <quadros> <saida_dir> [VAR=valor ...]
#   ex.: bash docs/Luz2D/ferramentas/mede_custo_no_app.sh 1920 1080 high 2400 /tmp/x PH2D_LIGHT2D_RENASCE=5
#
# Cada chamada é UM processo do app (cena 5 por omissão: `PH2D_LIGHT2D_SMOKE=5`), com o instrumento
# `PH2D_LIGHT2D_CUSTO=1` (mínimo e mediana por janela de 60 leituras) e as prefs da luz num HOME isolado
# (nível manual, automático desligado). O log fica em <saida_dir>/app.log, com o relógio da placa
# (`nvidia-smi`) a cada 2 s em <saida_dir>/placa.csv.
# A sessão é a do `docs/Components/ferramentas/fotografa_cena.sh` (kwin --virtual, X11 na Xwayland dela,
# HOME isolado): nunca o ecrã do dono. Este roteiro NÃO constrói: recusa um binário mais velho que o código.
set -euo pipefail

LARG="${1:?largura}"
ALT="${2:?altura}"
NIVEL="${3:?nivel}"
QUADROS="${4:?quadros}"
SAIDA="${5:?saida_dir}"
shift 5
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$BIN" ] || { echo "falta o binario: bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }
NOVO="$(find "$RAIZ/crates" "$RAIZ/shells" -name '*.rs' -newer "$BIN" -print -quit 2>/dev/null || true)"
[ -z "$NOVO" ] || { echo "RECUSA: codigo mais novo que o binario (${NOVO#"$RAIZ/"})" >&2; exit 2; }

mkdir -p "$SAIDA"
SAIDA="$(cd "$SAIDA" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/cfg" "$TMP/home/.ph2d"
cat > "$TMP/cfg/kwinrulesrc" <<'EOF'
[General]
count=1
rules=r1

[r1]
Description=maximiza tudo
wmclassmatch=0
types=1
maximizehoriz=true
maximizehorizrule=3
maximizevert=true
maximizevertrule=3
EOF
printf 'light_quality=%s\nlight_auto=0\n' "$NIVEL" > "$TMP/home/.ph2d/prefs.txt"
ENVS="PH2D_LIGHT2D_SMOKE=5 PH2D_LIGHT2D_CUSTO=1 $*"

cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: nao e' a tela virtual" > "$SAIDA/app.log"; exit 3
fi
cd "$RAIZ"
env -u WAYLAND_DISPLAY HOME="$TMP/home" __GL_SHADER_DISK_CACHE_PATH=/home/enio/.cache/nvidia \
  __GL_SHADER_DISK_CACHE_SIZE=17179869184 $ENVS PH2D_EXIT_AFTER_FRAMES=$QUADROS "$BIN" > "$SAIDA/app.log" 2>&1
EOF
if grep -q '`' "$TMP/sessao.sh"; then echo "RECUSA: crase no roteiro gerado" >&2; exit 4; fi
chmod +x "$TMP/sessao.sh"

nvidia-smi --query-gpu=timestamp,pstate,clocks.sm,clocks.mem,power.draw,utilization.gpu,temperature.gpu \
  --format=csv,noheader -l 2 > "$SAIDA/placa.csv" 2>/dev/null &
SMI=$!
XDG_CONFIG_HOME="$TMP/cfg" timeout 600 kwin_wayland --virtual --width "$LARG" --height "$ALT" \
  --xwayland --exit-with-session "$TMP/sessao.sh" > "$TMP/kwin.log" 2>&1 || true
kill "$SMI" 2>/dev/null || true
echo "$ENVS · $NIVEL · ${LARG}×${ALT}" > "$SAIDA/caso.txt"
grep -E '\[luz2d\] (janela|onde)' "$SAIDA/app.log" | tail -n +1 | head -3 || true
