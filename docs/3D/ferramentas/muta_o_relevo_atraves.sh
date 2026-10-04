#!/usr/bin/env bash
# Prova de mutação do RELEVO ATRAVÉS DOS AJUSTES DE VIZINHANÇA (docs/3D/30 §20): o plano da dobra,
# a dobra através (âmbito, opacidade, máscara, o corpo, a nitidez), o 2D que a lê, a peça que a
# deixa por dobrar e a dobra na placa, os leitores da CPU e a frase do painel.
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmãos (âncora única · a mutação compila ·
# N > 0 testes correram · a corrida limpa VERDE), e tem o PRÉ-VOO (`MUTA_SO_ANCORAS=1`).
#
# ⚠️ A POPULAÇÃO: os gates sem placa do Painter (`relief_through`, `impasto_body::blur`), da peça
# (`o_relevo_por_dobrar`, `relevo_borrado`), do painel (`nota_do_relevo`) e os da placa da peça
# (`relevo_atraves`, perfil `smoke`).
#
# ⛔ Chame-o sempre pela porta de recursos, com a placa:
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_relevo_atraves.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
PNT=crates/ph2d-tool-painter/src
SCU=crates/ph2d-app-sculpt3d/src
REN=crates/ph2d-render/src
PAN=crates/ph2d-panel-painter-layers/src
BK=$(mktemp -d)
cp -r "$PNT" "$BK/pnt"; cp -r "$SCU" "$BK/scu"; cp -r "$REN" "$BK/ren"; cp -r "$PAN" "$BK/pan"
restore() {
  rm -rf "$PNT" "$SCU" "$REN" "$PAN"
  cp -r "$BK/pnt" "$PNT"; cp -r "$BK/scu" "$SCU"; cp -r "$BK/ren" "$REN"; cp -r "$BK/pan" "$PAN"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTAÇÃO.
  find "$PNT" "$SCU" "$REN" "$PAN" -type f -exec touch {} +
}
trap restore EXIT

corrida() {
  cargo test -p ph2d-tool-painter --lib -- relief_through impasto_body::blur 2>&1
  cargo test -p ph2d-panel-painter-layers --lib -- nota_do_relevo 2>&1
  cargo test -p ph2d-app-sculpt3d --lib -- o_relevo_por_dobrar relevo_borrado 2>&1
  cargo test -p ph2d-app-sculpt3d --profile smoke --lib -- --ignored --test-threads=1 \
    relevo_atraves --skip diag_ 2>&1
}
populacao() { grep -oP '\K[0-9]+(?= passed)' | awk '{s+=$1}END{print s+0}'; }
falhou() { grep -q 'test result: FAILED'; }

if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida)
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes passaram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa não correu teste nenhum"; exit 2; }
  if printf '%s' "$limpa" | falhou; then
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
  local out corridos
  out=$(corrida)
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutação não compila"
  else
    corridos=$(printf '%s' "$out" | grep -oP '\K[0-9]+(?= (passed|failed))' | awk '{s+=$1}END{print s+0}')
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — as últimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif printf '%s' "$out" | falhou; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A PORTA (ph2d-tool-painter) ──────────────────────────────────────────
muta "$PNT/layers/relief_through.rs" \
  '                    out.push(ReliefStep::Filter {
                        id,' \
  '                    let _nada = (ReliefStep::Filter {
                        id,' \
  'T1 nenhum ajuste entra no plano (o relevo nunca borra)'

muta "$PNT/layers/relief_through.rs" \
  '                antes: Some(h.clone()),' \
  '                antes: None,' \
  'T2 um ajuste dentro de um grupo borra também o relevo de fora'

muta "$PNT/layers/relief_through.rs" \
  '                    let t = opacity * m' \
  '                    let t = m' \
  'T3 a opacidade do ajuste não pesa o relevo'

muta "$PNT/layers/relief_through.rs" \
  '.map(|x| if inv { 1.0 - x } else { x })' \
  '.map(|x| x)' \
  'T4 a máscara invertida não inverte'

muta "$PNT/layers/relief_through.rs" \
  '        ReliefFilter::Blur { radius } => nb.blur2(radius, h, corpo),' \
  '        ReliefFilter::Blur { radius } => nb.blur1(radius, h),' \
  'T5 o desfoque não borra o corpo (a luz corta o relevo borrado na borda da tinta)'

muta "$PNT/layers/relief_through.rs" \
  '                *x = b + amount * (b - *x);' \
  '                *x = b - amount * (b - *x);' \
  'T6 a nitidez amacia em vez de afiar'

muta "$PNT/tool/paint/relief_fields.rs" \
  '            Some((f, _)) => f[i],' \
  '            Some(_) => self.fold_at(i),' \
  'T7 a luz do 2D ignora a dobra materializada'

# ── A PEÇA ───────────────────────────────────────────────────────────────
muta "$SCU/pilha_da_peca_relevo.rs" \
  '            .filter(|s| !matches!(*s, ReliefStep::Layer { id, .. } if !amostras.has(id)))' \
  '            .filter(|s| !matches!(*s, ReliefStep::Layer { id, .. } if !amostras.has(id)))
            .filter(|s| !matches!(s, ReliefStep::Filter { .. }))' \
  'T8 o raio não entra na assinatura: arrastá-lo não redobra'

muta "$SCU/slots.rs" \
  '                .is_some_and(crate::pilha_da_peca::PilhaDaPeca::relevo_por_dobrar)' \
  '                .is_some_and(|_| false)' \
  'T9 o sync_mesh não dobra o relevo por dobrar'

muta "$SCU/tinta_da_peca_pilha.rs" \
  '    *relevo_sujo |= pilha.relevo_atraves();' \
  '' \
  'T10 o traço por baixo de um desfoque não sobe o relevo'

muta "$SCU/tinta_da_peca_pilha.rs" \
  '        pilha.relevo_em_dia(peca);' \
  '' \
  'T11 o em_dia não dobra o relevo por dobrar'

muta "$SCU/tinta_da_peca_pilha.rs" \
  '            if (p.atrasada() || p.relevo_por_dobrar())' \
  '            if p.atrasada()' \
  'T12 o para_ler lê o relevo velho'

# ── A PLACA (ph2d-render) ────────────────────────────────────────────────
muta "$REN/layer_compositor/surface.rs" \
  '            gpu.queue.write_buffer(y, 0, bytemuck::cast_slice(c));' \
  '' \
  'T13 o calor do campo começa sem o próprio campo'

# ── O PAINEL ─────────────────────────────────────────────────────────────
muta "$PAN/paint_adjust.rs" \
  '    if ph2d_tool_painter::relief_effect(params.kind()) != ph2d_tool_painter::ReliefEffect::Tone {' \
  '    if ph2d_tool_painter::relief_effect(params.kind()) == ph2d_tool_painter::ReliefEffect::Tone {' \
  'T14 a frase do tom vai para o efeito errado'

# ── O CONTROLO ───────────────────────────────────────────────────────────
muta "$PNT/layers/relief_through.rs" \
  '/// O que um ajuste faz ao relevo por baixo dele.' \
  '/// O que um ajuste faz ao relevo por baixo dele!' \
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
