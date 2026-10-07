#!/bin/bash
# ORÁCULO do rolamento: o Box2D 3.1.1 (MIT) sobre as NOSSAS entradas — doc 121 §9.24.
#
# PASSO 1 (a triagem da licença, CLAUDE.md §0.9): o artefacto é o pacote `extra/box2d 3.1.1-2` do Arch
# (`usr/share/licenses/box2d/LICENSE`: MIT). Se a biblioteca não estiver instalada, o pacote é baixado do
# espelho do sistema para `target/oraculo_box2d/` e a ASSINATURA conferida contra o chaveiro do pacman
# (uma cópia; nada de root, nada instalado no sistema).
#
# uso: bash corre.sh <pasta com pilha_XX.txt (a sonda `exporta_a_pilha_para_o_oraculo`)> [saida]
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(cd "$AQUI/../../../.." && pwd)"
PASTA="${1:?pasta com pilha_XX.txt}"
SAIDA="${2:-$RAIZ/target/prova/onda9/oraculo/box2d.txt}"
D="$RAIZ/target/oraculo_box2d"
mkdir -p "$D"
if [ -f /usr/include/box2d/box2d.h ]; then
  INC=/usr/include; LIB=/usr/lib
else
  PKG=box2d-3.1.1-2-x86_64.pkg.tar.zst
  if [ ! -d "$D/x/usr/include/box2d" ]; then
    URL="$(pacman -Sp box2d | head -1)"
    case "$URL" in */$PKG) ;; *) echo "o espelho oferece outra versao: $URL" >&2; exit 2 ;; esac
    curl -sSfL -o "$D/$PKG" "$URL"
    curl -sSfL -o "$D/$PKG.sig" "$URL.sig"
    rm -rf "$D/kr"; cp -r /etc/pacman.d/gnupg "$D/kr" 2>/dev/null || true
    LANG=C gpg --homedir "$D/kr" --verify "$D/$PKG.sig" "$D/$PKG" 2>&1 | grep -q '^gpg: Good signature' \
      || { echo "ASSINATURA do pacote nao confere" >&2; rm -rf "$D/kr"; exit 2; }
    rm -rf "$D/kr" "$D/x"; mkdir -p "$D/x"; tar -C "$D/x" -xf "$D/$PKG"
  fi
  INC="$D/x/usr/include"; LIB="$D/x/usr/lib"
fi
gcc -O2 -std=gnu17 -I"$INC" "$AQUI/main.c" "$LIB/libbox2d.so.3.1.1" -lm -Wl,-rpath,"$LIB" -o "$D/oraculo"
{
  echo "# oraculo Box2D 3.1.1 (MIT) — doc 121 §9.24; entradas: ${PASTA#"$RAIZ/"}; escala do rr ${ESCALA:-1}; $(date -I)"
  "$D/oraculo" rampa
  # A escala do rr: o limiar da rampa diz se a unidade leva o raio (ver o README do §9.24).
  ESCALA="${ESCALA:-1}"
  "$D/oraculo" pilha "$PASTA" "$ESCALA" 0 0.05 0.1 0.15 0.25
} | tee "$SAIDA"
