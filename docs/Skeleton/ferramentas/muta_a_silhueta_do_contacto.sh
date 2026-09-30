#!/usr/bin/env bash
# Prova de mutacao da SILHUETA DO CONTACTO (2026-09-29) — numa dobra forte o desenho
# fiel da pele vectorial sobrepoe-se a si mesmo e sai como a uniao dos membros.
#
# Tres camadas:
#   * a PORTA — `ph2d_vec_boolean::resolve_overlap` / `crosses_itself` (overlap.rs);
#   * o FIO   — `skin_desenho::calcula` + a porta de bisseccao `PH2D_SKIN_CONTACTO`;
#   * a QUINA (2026-09-30, report do dono: «a depender do angulo a quina fica
#     inconsistente. Faca obedecer ao que foi escolhido no painel») — a solda, a limpeza
#     das alcas e o limite do bico UM so' para o ecra, o Outline Stroke e o SVG.
#
# ⚠️ A POPULACAO e' de quem OBSERVA a mutacao, nunca de quem a CONTEM: as mutacoes do
# limite do bico no renderer e no SVG so' sao vistas pelos gates DESSAS crates, e o
# `EXTRA=<ren|apv>` de cada `muta` acrescenta-os a corrida so' onde e' preciso.
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
REN=crates/ph2d-vec-render/src
APV=crates/ph2d-app-vec/src
BK=$(mktemp -d)
cp -r "$BOO" "$BK/boo"; cp -r "$SKL" "$BK/skl"; cp -r "$REN" "$BK/ren"; cp -r "$APV" "$BK/apv"
restore() {
  rm -rf "$BOO" "$SKL" "$REN" "$APV"
  cp -r "$BK/boo" "$BOO"; cp -r "$BK/skl" "$SKL"; cp -r "$BK/ren" "$REN"; cp -r "$BK/apv" "$APV"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$BOO" "$SKL" "$REN" "$APV" -name '*.rs' -exec touch {} +
}
trap restore EXIT
EXTRA=""

corrida() {
  local a b c ra rb rc=0
  a=$(cargo nextest run -p ph2d-vec-boolean --lib -E 'test(overlap)' 2>&1); ra=$?
  b=$(cargo nextest run -p ph2d-skeleton-live --lib -E 'test(skin_desenho::tests)' 2>&1); rb=$?
  c=""
  case "$EXTRA" in
    *ren*) c=$(cargo nextest run -p ph2d-vec-render --lib -E 'test(o_bico_do_traco)' 2>&1) || rc=1 ;;
  esac
  case "$EXTRA" in
    *apv*) c="$c
$(cargo nextest run -p ph2d-app-vec --lib -E 'test(o_svg_leva_o_limite_do_bico)' 2>&1)" || rc=1 ;;
  esac
  printf '%s\n%s\n%s\n' "$a" "$b" "$c"
  [ "$ra" -eq 0 ] && [ "$rb" -eq 0 ] && [ "$rc" -eq 0 ]
}

populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
EXTRA="ren apv"
limpa=$(corrida); rc_limpo=$?
EXTRA=""
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

# ── A QUINA ──────────────────────────────────────────────────────────────
muta "$BOO/overlap.rs" '        .map(|v| solda_os_segmentos_curtos(v, solda))' '        .map(|v| {
            let _ = solda;
            v
        })' \
  'Q1 a silhueta nao passa pela solda'

muta "$BOO/overlap.rs" '        if minusculo {' '        if false && minusculo {' \
  'Q2 nenhum segmento minusculo e soldado'

# ⛔ A limpeza ANTES da solda foi APAGADA: a mutacao que a removia SOBREVIVEU — a solda
# nao decide nada a partir das alcas limpas, e o ruido medido esta' tres ordens abaixo da
# tolerancia. So' a limpeza DEPOIS dela (Q4) e' lei.
muta "$BOO/overlap.rs" '            p2 = p0;' '            let _ = p0;' \
  'Q3 a alca de ENTRADA caida no comeco fica'

muta "$BOO/overlap.rs" '    limpa_as_alcas(&mut verts, tol);
    verts
}' '    verts
}' \
  'Q4 a alca HERDADA da solda fica por limpar'

muta "$BOO/overlap.rs" '            p1 = p3;' '            let _ = p3;' \
  'Q5 a alca que caiu na ponta de LA fica'

muta "$BOO/overlap.rs" '    180.0 - 2.0 * (1.0 / ph2d_vec_scene::MITER_LIMIT)' '    180.0 - 1.0 * (1.0 / ph2d_vec_scene::MITER_LIMIT)' \
  'Q6 a viragem maxima deixa de ser a do limite'

muta "$BOO/expand.rs" '        .with_miter_limit(ph2d_vec_scene::MITER_LIMIT);' '        ;' \
  'Q7 o Outline Stroke volta ao limite 4'

EXTRA=ren muta "$REN/lib.rs" '        .with_miter_limit(ph2d_vec_scene::MITER_LIMIT);' '        ;' \
  'Q8 o ecra volta ao limite 4'

EXTRA=apv muta "$APV/svg_export.rs" '    if matches!(s.join, ph2d_vec_scene::LineJoin::Miter) {' \
  '    if false && matches!(s.join, ph2d_vec_scene::LineJoin::Miter) {' \
  'Q9 o SVG exportado volta ao limite 4'

# ── O CONTROLO (nao pode sangrar) ────────────────────────────────────────
muta "$BOO/overlap.rs" '/// ⭐⭐ **O contorno cruza-se?** — os' '/// ⭐⭐ **O contorno cruza-se (controlo)?** — os' \
  'C0 CONTROLO: mudar um comentario'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam uma vez (zero testes corridos)"
else
  echo "PLACAR: $sangram de $total sangram (a ultima e' o CONTROLO)"
fi
