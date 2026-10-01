#!/usr/bin/env bash
# Prova de mutação da NORMAL DO RELEVO sem derivadas de ecrã (docs/3D/29 §7,
# report do dono de 01/10: «de cima parece bom, inclinado aparece artefato de
# relevo»). O gradiente da altura sai EXACTO das leituras da retícula e passa
# para o objecto pela geometria do triângulo; a luz lê-o pelo gradiente de
# superfície.
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmãos (âncora única ·
# a mutação compila · N > 0 testes correram · a corrida limpa VERDE), e tem o
# PRÉ-VOO (`MUTA_SO_ANCORAS=1`).
#
# ⚠️ A POPULAÇÃO é de quem OBSERVA: a paridade e os gates de pixel são de PLACA
# (`--run-ignored all`), e o censo do WGSL corre sem ela — a corrida é a crate
# do renderizador com os dois.
#
# ⛔ Chame-o sempre pela porta de recursos, COM a placa:
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_normal_do_relevo.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
REN=crates/ph2d-mesh-render/src
BK=$(mktemp -d)
cp -r "$REN" "$BK/ren"
restore() {
  rm -rf "$REN"
  cp -r "$BK/ren" "$REN"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTAÇÃO.
  find "$REN" -type f -exec touch {} +
}
trap restore EXIT

corrida() {
  cargo nextest run -p ph2d-mesh-render --run-ignored all \
    -E 'test(tinta) | test(relevo) | test(derivadas_de_ecra)' 2>&1
}
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida); rc_limpo=$?
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes correram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa não correu teste nenhum"; exit 2; }
  if [ "$rc_limpo" -ne 0 ]; then
    echo "ABORTO: a corrida limpa está VERMELHA -- um placar tirado daqui é fabricado."
    exit 2
  fi
fi

sangram=0; total=0
muta() { # ficheiro  âncora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  if [ -n "$FILTRO" ] && ! printf '%s' "$nome" | grep -Eq "$FILTRO"; then return; fi
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ÂNCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a âncora casou $n vezes (esperado 1)"; return; fi
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
    echo "  ABORTO [$nome]: a mutação não compila"
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — as últimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A LEITURA (o gémeo em WGSL) ─────────────────────────────────────────
muta "$REN/shaders/tinta.wgsl" \
  '        out.g = -lf * hs;' \
  '        out.g = lf * hs;' \
  'N1 o sub-triângulo INVERTIDO lê o gradiente com o sinal trocado'

muta "$REN/shaders/tinta.wgsl" \
  '        lf * ((1.0 - fv) * (hb - ha) + fv * (hd - he)),' \
  '        ((1.0 - fv) * (hb - ha) + fv * (hd - he)),' \
  'N2 a derivada da bilinear esquece que a célula mede 1/L'

muta "$REN/shaders/tinta.wgsl" \
  '        gu = gb;' \
  '        gu = -ga;' \
  'N3 a metade (a,c,d) do quad lê o u da outra metade'

muta "$REN/shaders/tinta.wgsl" \
  '    let gc = -ga - gb;' \
  '    let gc = -ga;' \
  'N4 o gradiente da 3.ª baricêntrica esquece a 2.ª'

# ── A LUZ ────────────────────────────────────────────────────────────────
muta "$REN/fonte.rs" \
  '    let gv = (m * vec4<f32>(g, 0.0)).xyz / escala;' \
  '    let gv = (m * vec4<f32>(g, 0.0)).xyz / escala + 0.0 * dpdx(in.opos);' \
  'N5 a normal volta a tirar uma derivada de ECRÃ'

# ⚠️ N6/N7 re-ancoradas em 01/10 (§8 do doc 29): a lei saiu do `fonte.rs` para
#    o `tinta_inclina` do `tinta.wgsl`, e as âncoras antigas casavam ZERO — o
#    pré-voo apanhou-as. O `corpo` lê-se agora numa linha só (`let c`).
muta "$REN/shaders/tinta.wgsl" \
  '    let c = clamp(corpo, 0.0, 1.0);' \
  '    let c = 1.0;' \
  'N6 a espessura SEM tinta volta a acender (o anel do report anterior)'

muta "$REN/shaders/tinta.wgsl" \
  '    let nb = n - c * gs;' \
  '    let nb = n + c * gs;' \
  'N7 o relevo inclina a normal para o lado CONTRÁRIO'

# ── O CONTROLO ───────────────────────────────────────────────────────────
muta "$REN/fonte.rs" \
  '// ⭐⭐⭐ **A NORMAL INCLINADA PELO RELEVO** — o *gradiente de superfície*' \
  '// ⭐⭐⭐ **A NORMAL INCLINADA PELO RELEVO**: o *gradiente de superfície*' \
  'C1 CONTROLO: uma mutação INERTE (um comentário) não pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRÉ-VOO: $sangram de $total âncoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ] || exit 1
  exit 0
fi
if [ -n "$FILTRO" ]; then
  echo "PLACAR PARCIAL (filtro MUTA_FILTRO='$FILTRO'): $sangram de $total sangram"
else
  echo "PLACAR: $sangram de $total sangram (o C1 é o CONTROLO e não pode)"
fi
