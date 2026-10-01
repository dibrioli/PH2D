#!/usr/bin/env bash
# Prova de mutação do RELEVO pousado na peça (a etapa 3b do Painter, W3 —
# docs/3D/29): a luz 2D desligada na tela da vista, a espessura em píxeis na
# janela, a leitura dela, a conversão de píxel para a peça e a lei
# `nova = antes + k·(tela − semente)`, com o CORPO ao lado e a SEMENTE do
# relevo (§6, report de 01/10).
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmãos (âncora única ·
# a mutação compila · N > 0 testes correram · a corrida limpa VERDE), e tem o
# PRÉ-VOO (`MUTA_SO_ANCORAS=1`).
#
# ⚠️ A POPULAÇÃO é de quem OBSERVA: as mutações do Painter são vistas pelos
# gates dele, as da escultura pelos dela — a corrida é as duas crates.
#
# ⛔ Chame-o sempre pela porta de recursos:
#   bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_relevo_na_peca.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
PNT=crates/ph2d-tool-painter/src/tool
SCU=crates/ph2d-sculpt3d/src
BK=$(mktemp -d)
cp -r "$PNT" "$BK/pnt"; cp -r "$SCU" "$BK/scu"
restore() {
  rm -rf "$PNT" "$SCU"
  cp -r "$BK/pnt" "$PNT"; cp -r "$BK/scu" "$SCU"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTAÇÃO.
  find "$PNT" "$SCU" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() {
  cargo nextest run -p ph2d-tool-painter -p ph2d-sculpt3d --lib \
    -E 'test(screen_canvas_relief) | test(tela_relevo)' 2>&1
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

# ── O PAINTER ────────────────────────────────────────────────────────────
muta "$PNT/paint/impasto_light.rs" \
  '        if self.on_screen_canvas() {
            return false;
        }' \
  '' \
  'P1 a luz 2D volta a ser assada na tela da vista'

muta "$PNT/paint/impasto_live.rs" \
  '            * super::impasto_light::DEPTH_UNIT_PX;' \
  '            * 1.0;' \
  'P2 a espessura sai em unidades de profundidade e não em píxeis'

muta "$PNT/paint/impasto_live.rs" \
  '                out.push((c + l) * escala);' \
  '                out.push(c * escala);' \
  'P3 a janela esquece o traço aberto'

muta "$PNT/screen_canvas.rs" \
  'const MARGEM_DO_RELEVO: u32 = 2;' \
  'const MARGEM_DO_RELEVO: u32 = 0;' \
  'P4 a janela da espessura perde a margem da pousada'

# ── A PEÇA ───────────────────────────────────────────────────────────────
muta "$SCU/tela_relevo.rs" \
  '        let (fx, fy) = (x - 0.5, y - 0.5);' \
  '        let (fx, fy) = (x, y);' \
  'S1 a janela lê-se meio píxel desencontrada da cor'

muta "$SCU/tela_relevo.rs" \
  '        let frente = unit([m[3], m[7], m[11]]).or_else(|| unit([m[2], m[6], m[10]]))?;' \
  '        let frente = unit(d)?;' \
  'S2 o píxel mede-se perpendicular ao RAIO e não à imagem'

muta "$SCU/tela_na_malha_pousa.rs" \
  '                    antes[0] + d[0] * k * wpp,' \
  '                    antes[0] + d[0] * k,' \
  'S3 a espessura não se converte de píxel para a peça'

muta "$SCU/tela_na_malha_pousa.rs" \
  '                    antes[0] + d[0] * k * wpp,' \
  '                    d[0] * k * wpp,' \
  'S4 o traço seguinte substitui a espessura em vez de somar'

muta "$SCU/tela_na_malha_pousa.rs" \
  '    if vazia && d == [0.0; 2] && !fina.tocou(idx)' \
  '    if vazia && !fina.tocou(idx)' \
  'S5 uma tela sem cor não leva a espessura'

muta "$SCU/tela_na_malha_pousa.rs" \
  '        [t[0] - sem[0], t[1] - sem[1]]' \
  '        [t[0], t[1]]' \
  'S6 a pousada esquece a SEMENTE: a tela semeada dobra a espessura'

muta "$SCU/tela_na_malha_pousa.rs" \
  '                    (antes[1] + d[1] * k).clamp(0.0, 1.0),' \
  '                    antes[1] + d[1] * k,' \
  'S7 o corpo não satura em 0..1'

muta "$SCU/tela_na_malha_pousa.rs" \
  '                    (antes[1] + d[1] * k).clamp(0.0, 1.0),' \
  '                    antes[1],' \
  'S8 o corpo da tela não chega à peça'

muta "$SCU/tela_semente_relevo.rs" \
  '                out.px[o] = if m > 0.0 { e[0] / m } else { 0.0 };' \
  '                out.px[o] = e[0];' \
  'S9 a semente não converte a altura da peça em píxeis'

# ── A SEMENTE E O CORPO NO PAINTER (report de 01/10) ─────────────────────
muta "$PNT/paint/impasto_screen_relief.rs" \
  '                out.push(f32::from(c.max(l)) / 255.0);' \
  '                out.push(1.0);' \
  'P5 a janela leva corpo cheio onde a espessura transborda a tinta'

muta "$PNT/paint/impasto_screen_relief.rs" \
  '            .insert(id, Arc::new(px.iter().map(|&v| v / escala).collect()));' \
  '            .insert(id, Arc::new(px.to_vec()));' \
  'P6 a semente não desfaz a conversão da janela'

muta "$PNT/paint/impasto_screen_relief.rs" \
  '        self.covers.insert(id, Arc::new(cover.to_vec()));' \
  '' \
  'P7 a semente não leva a cobertura: o relevo semeado sai sem corpo'

# ── O CONTROLO ───────────────────────────────────────────────────────────
muta "$SCU/tela_relevo.rs" \
  '/// O relevo numa janela da tela — a espessura em PÍXEIS e o corpo `0..1`,' \
  '/// O relevo numa janela da tela, a espessura em PÍXEIS e o corpo `0..1`,' \
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
