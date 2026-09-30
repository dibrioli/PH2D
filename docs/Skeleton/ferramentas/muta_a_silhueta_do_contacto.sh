#!/usr/bin/env bash
# Prova de mutacao da SILHUETA DO CONTACTO (2026-09-29) — numa dobra forte o desenho
# fiel da pele vectorial sobrepoe-se a si mesmo e sai como a uniao dos membros.
#
# Duas camadas:
#   * a PORTA — `ph2d_vec_boolean::resolve_overlap` / `crosses_itself` (overlap.rs);
#   * o FIO   — `skin_desenho::calcula` + a porta de bisseccao `PH2D_SKIN_CONTACTO`.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmaos (ancora unica ·
# a mutacao compila · N > 0 testes correram · a corrida limpa esta' VERDE).
#   bash scripts/ph2d-run.sh bash docs/Skeleton/ferramentas/muta_a_silhueta_do_contacto.sh
# ⭐ `MUTA_SO_ANCORAS=1` e' o pre-voo; `MUTA_FILTRO=<ERE>` corre so' as que casam.
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
BOO=crates/ph2d-vec-boolean/src
SKL=crates/ph2d-skeleton-live/src
BK=$(mktemp -d)
cp -r "$BOO" "$BK/boo"; cp -r "$SKL" "$BK/skl"
restore() {
  rm -rf "$BOO" "$SKL"
  cp -r "$BK/boo" "$BOO"; cp -r "$BK/skl" "$SKL"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$BOO" "$SKL" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() {
  local a b ra rb
  a=$(cargo nextest run -p ph2d-vec-boolean --lib -E 'test(overlap)' 2>&1); ra=$?
  b=$(cargo nextest run -p ph2d-skeleton-live --lib -E 'test(skin_desenho::tests)' 2>&1); rb=$?
  printf '%s\n%s\n' "$a" "$b"
  [ "$ra" -eq 0 ] && [ "$rb" -eq 0 ]
}

populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
limpa=$(corrida); rc_limpo=$?
verde=$(printf '%s' "$limpa" | populacao)
echo "VERDE antes: $verde testes correram"
[ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
if [ "$rc_limpo" -ne 0 ]; then
  echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar tirado daqui e' fabricado."
  printf '%s' "$limpa" | grep -E '^ *(FAIL|Summary)' | tail -8 | sed 's/^/      | /'
  exit 2
fi
fi

sangram=0; total=0
muta() { # ficheiro  ancora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  if [ -n "$FILTRO" ] && ! printf '%s' "$nome" | grep -Eq "$FILTRO"; then return; fi
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a ancora casou $n vezes (esperado 1)"; return; fi
  python3 -c '
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,"w").write(s.replace(a, b, 1))
' "$f" "$agulha" "$subst"
  touch "$f"
  local out rc corridos
  out=$(corrida); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutacao nao compila"
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — as ultimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A PORTA ──────────────────────────────────────────────────────────────
muta "$BOO/overlap.rs" '    if !crosses_itself(&bez) {
        return None;
    }' '    if false && !crosses_itself(&bez) {
        return None;
    }' \
  'P1 a porta corre SEMPRE, com ou sem contacto'

muta "$BOO/overlap.rs" '    out.verts = outer;' '    let _ = outer;' \
  'P2 a silhueta devolve o contorno de antes'

muta "$BOO/overlap.rs" '    out.subpaths = resto;' '    let _ = resto;' \
  'P3 as ilhas da uniao perdem-se'

muta "$BOO/overlap.rs" '    d1 * d2 < 0.0 && d3 * d4 < 0.0' '    d1 * d2 <= 0.0 && d3 * d4 <= 0.0' \
  'P4 um TOQUE passa a contar como cruzamento'

# ⛔ O salto dos VIZINHOS foi APAGADO: a mutacao que o removia SOBREVIVEU, porque um
# extremo partilhado da' um teste de lado ZERO EXACTO e o teste estrito ja' o recusa.

muta "$BOO/overlap.rs" '    if !path.closed || path.subpaths.iter().any(|c| !c.closed) {' \
  '    if path.subpaths.iter().any(|c| !c.closed) {' \
  'P6 um caminho ABERTO ganha silhueta'

# ── O FIO ────────────────────────────────────────────────────────────────
muta "$SKL/skin_desenho.rs" '            if leis.contacto {
                ph2d_vec_boolean' '            if false && leis.contacto {
                ph2d_vec_boolean' \
  'F1 o desenho da pele nunca passa pela porta'

muta "$SKL/skin_desenho.rs" '    valor != Some("0")' '    valor == Some("1")' \
  'F2 o contacto nasce DESLIGADO'

# ── O CONTROLO (nao pode sangrar) ────────────────────────────────────────
muta "$BOO/overlap.rs" '/// ⭐⭐ **O contorno cruza-se?** — os' '/// ⭐⭐ **O contorno cruza-se (controlo)?** — os' \
  'C0 CONTROLO: mudar um comentario'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam uma vez (zero testes corridos)"
else
  echo "PLACAR: $sangram de $total sangram (a ultima e' o CONTROLO)"
fi
