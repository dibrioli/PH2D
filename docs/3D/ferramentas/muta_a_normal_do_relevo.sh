#!/usr/bin/env bash
# Prova de mutação da NORMAL DO RELEVO (docs/3D/29 §7–§9): sem derivadas de
# ecrã, e com a INCLINAÇÃO POR AMOSTRA (§9, 02/10) — o gradiente de cada célula
# levado às amostras pela área, interpolado pelos pesos da cor, mantido por
# pedaços durante o traço e ao esculpir. A luz lê-o pelo gradiente de superfície.
#
# ⚠️ Re-escrito em 02/10: as N1–N4 de 01/10 mutavam a lei POR CÉLULA do
#    `tinta.wgsl`, que deixou de existir (o pré-voo apanhou as quatro mortas).
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmãos (âncora única ·
# a mutação compila · N > 0 testes correram · a corrida limpa VERDE), e tem o
# PRÉ-VOO (`MUTA_SO_ANCORAS=1`).
#
# ⚠️ A POPULAÇÃO é de quem OBSERVA: a paridade e os gates de pixel são de PLACA
# (`--run-ignored all`), o censo do WGSL corre sem ela, e a LEI da inclinação é
# gateada sem placa na `ph2d-mesh-colors` — a corrida são as duas crates.
#
# ⛔ Chame-o sempre pela porta de recursos, COM a placa:
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_normal_do_relevo.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
REN=crates/ph2d-mesh-render/src
COL=crates/ph2d-mesh-colors/src
BK=$(mktemp -d)
cp -r "$REN" "$BK/ren"
cp -r "$COL" "$BK/col"
restore() {
  rm -rf "$REN" "$COL"
  cp -r "$BK/ren" "$REN"
  cp -r "$BK/col" "$COL"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTAÇÃO.
  find "$REN" "$COL" -type f -exec touch {} +
}
trap restore EXIT

corrida() {
  cargo nextest run -p ph2d-mesh-render -p ph2d-mesh-colors --run-ignored all \
    -E 'test(tinta) | test(relevo) | test(derivadas_de_ecra) | test(inclinacao) | test(esculpir) | test(paralelo) | test(pedacos) | test(vertices) | test(altura)' 2>&1
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
  '        o.g + tinta_inclinacao(i) * w,' \
  '        o.g + tinta_inclinacao(i),' \
  'N1 a inclinação das amostras soma-se SEM os pesos da cor'

muta "$REN/shaders/tinta.wgsl" \
  '    return vec3<f32>(tinta_inclinacoes[b], tinta_inclinacoes[b + 1u], tinta_inclinacoes[b + 2u]);' \
  '    return vec3<f32>(tinta_inclinacoes[b], tinta_inclinacoes[b + 1u], tinta_inclinacoes[b + 1u]);' \
  'N2 a placa lê o z da inclinação no endereço do y'

# ── A LEI (ph2d-mesh-colors) ─────────────────────────────────────────────
muta "$COL/inclinacao.rs" \
  '                        celula([ch(i, j + 1), ch(i + 1, j), ch(i + 1, j + 1)], -1.0);' \
  '                        celula([ch(i, j + 1), ch(i + 1, j), ch(i + 1, j + 1)], 1.0);' \
  'N3 a célula INVERTIDA do triângulo dá o gradiente com o sinal trocado'

muta "$COL/inclinacao.rs" \
  '        let (gu1, gv1) = (gb1, escala(ga1, -1.0));' \
  '        let (gu1, gv1) = (escala(ga1, -1.0), gb1);' \
  'N4 a metade (a,c,d) do quad troca o u com o v'

muta "$COL/inclinacao.rs" \
  '                    if k >= 1 && (!so_borda || i == 0 || j == 0 || k == 1) {' \
  '                    if k >= 1 && (!so_borda || i == 0 || j == 0 || k == 2) {' \
  'N8 a borda da face vizinha esquece as células invertidas do lado k'

muta "$COL/inclinacao.rs" \
  '        self.espalha(tinta, cantos_de, pos, alt, &anel, tocadas.len(), Some(ep));' \
  '        self.espalha(tinta, cantos_de, pos, alt, &anel[..tocadas.len()], tocadas.len(), Some(ep));' \
  'N9 a atualização esquece o ANEL (a média da fronteira mente)'

muta "$COL/inclinacao.rs" \
  '                .filter(|&i| alt[i as usize][ALTURA] != 0.0)' \
  '                .filter(|&i| alt[i as usize][ALTURA] > 0.0)' \
  'N10 o plano esparso esquece as alturas NEGATIVAS'

# ── AS PORTAS DE SUBIDA (tinta_gpu) ──────────────────────────────────────
muta "$REN/tinta_gpu.rs" \
  '            if foto != agora {' \
  '            if foto != agora && false {' \
  'N11 a subida inteira não vê os vértices que o esculpir moveu'

muta "$REN/tinta_gpu.rs" \
  '                .anota_alturas(sujas, tinta.relevo().unwrap_or(&[]));' \
  '                .anota_alturas(sujas, tinta.relevo().unwrap_or(&[]));
            mudadas.clear();' \
  'N12 o incremental do traço não sobe as inclinações que refez'

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
