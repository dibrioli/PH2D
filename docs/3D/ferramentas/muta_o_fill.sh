#!/usr/bin/env bash
# Prova de mutacao do `Fill Piece` (2026-09-24) — a peca inteira com a cor do
# pincel, respeitando a mascara, desfeita com UM Ctrl+Z.
#
# A rede cobre TRES camadas, porque o gesto tem tres:
#   * a LEI    — `ph2d_sculpt3d::preenche` (a conta da mascara partilhada com o
#                carimbo, a mistura exacta, a recusa);
#   * a CENA   — `preenche.rs` + o braco do desfazer + o tecto da historia;
#   * o PAINEL — o botao, a tabela de comandos.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmaos (ancora unica ·
# a mutacao compila · N > 0 testes correram · a corrida limpa esta' VERDE).
#
# ⛔ Os gates de produto precisam de ADAPTADOR: chame por
#   PH2D_GPU=1 PH2D_PRAZO=3600 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_fill.sh
# ⭐ `MUTA_SO_ANCORAS=1` e' o pre-voo (segundos, sem um teste); `MUTA_FILTRO=<ERE>`
# corre so' as mutacoes cujo nome casa.
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
SCU=crates/ph2d-sculpt3d/src
APP=crates/ph2d-app-sculpt3d/src
PAN=crates/ph2d-panel-sculpt3d/src
BK=$(mktemp -d)
cp -r "$SCU" "$BK/scu"; cp -r "$APP" "$BK/app"; cp -r "$PAN" "$BK/pan"
restore() {
  rm -rf "$SCU" "$APP" "$PAN"
  cp -r "$BK/scu" "$SCU"; cp -r "$BK/app" "$APP"; cp -r "$BK/pan" "$PAN"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$SCU" "$APP" "$PAN" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() {
  local a b c ra rb rc
  a=$(cargo nextest run -p ph2d-sculpt3d --lib -E 'test(preenche)' 2>&1); ra=$?
  b=$(cargo nextest run -p ph2d-app-sculpt3d --lib --run-ignored all \
        -E 'test(/history_tinta_fina|esta_ligada_nos|tinta_no_produto_tests::fill/)' 2>&1); rb=$?
  c=$(cargo nextest run -p ph2d-panel-sculpt3d --test it \
        -E 'test(/fill|every_command_reaches_the_shell/)' 2>&1); rc=$?
  printf '%s\n%s\n%s\n' "$a" "$b" "$c"
  [ "$ra" -eq 0 ] && [ "$rb" -eq 0 ] && [ "$rc" -eq 0 ]
}

# A populacao honesta e' a que o `nextest` diz ter CORRIDO.
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

# ── A LEI ────────────────────────────────────────────────────────────────
muta "$SCU/preenche.rs" '    crate::mask_ops::free_weight(k)
' '    crate::mask_ops::free_weight(0.0 * k)
' \
  'L1 a mascara deixa de contar: o Fill pinta por cima dela'

# ⚠️ O canal VERDE e nao o vermelho, MEDIDO: com `c = 0,9` a forma ingenua e'
# exacta para todo `a` em [0,1] (a mutacao no canal vermelho SOBREVIVEU a 1.a
# corrida por construcao); com `c = 0,1` ela erra em 783 de 999 valores.
muta "$SCU/preenche.rs" '        antes[1] * fica + cor[1] * keep,' '        antes[1] + (cor[1] - antes[1]) * keep,' \
  'L2 a mistura deixa de ser exacta nas pontas'

muta "$SCU/preenche.rs" '        *f = true;
        keeps.push((' '        let _ = f;
        let _ = ((' \
  'L3 a passagem dos VERTICES some: o vertice solto fica por pintar'

muta "$SCU/preenche.rs" '    if !topo.descreve(mesh.vert_count(), mesh.faces().len())
        ||' '    if false
        ||' \
  'L4 a recusa por CONTAGENS some'

muta "$SCU/preenche.rs" '            .any(|(f, face)| face.verts().len() != topo.cantos_de(f))' \
  '            .any(|(_, _)| false)' \
  'L5 a recusa pela FORMA das faces some'

muta "$SCU/preenche.rs" '        let nova = mistura(*c, cor, keep);
        mudou |= nova != *c;' '        let nova = mistura(*c, cor, keep);
        mudou = true;' \
  'L6 o preenchimento por vertice diz que mudou quando nao mudou'

muta "$SCU/tinta_fina.rs" '                    keep: crate::preenche::keep_da_amostra(w, m),' \
  '                    keep: crate::mask_ops::free_weight(w.iter().zip(m).map(|(a, b)| a * b).sum()),' \
  'L7 o carimbo volta a fazer a conta da mascara por conta propria'

# ── A CENA ───────────────────────────────────────────────────────────────
muta "$APP/preenche.rs" '        let (mudou_plano, fina) = match obj.tinta.as_mut() {' \
  '        let (mudou_plano, fina) = match None::<&mut ph2d_mesh_colors::Tinta> {' \
  'A1 o Fill deixa de pintar o PLANO de tinta fina'

muta "$APP/preenche.rs" '            finas: if mudou_plano { finas_antes } else { None },' \
  '            finas: { let _ = finas_antes; None },' \
  'A2 a entrada grava-se sem o plano de antes'

muta "$APP/undo.rs" '                    let inversa = p.troca(obj.tinta.as_mut())?;' \
  '                    let inversa = p.troca(None)?;' \
  'A3 o desfazer do Fill deixa de trocar o plano'

muta "$APP/undo.rs" '                    Some(c) => obj.stack.mesh_mut().put_colors(c),' \
  '                    Some(_) => {}' \
  'A4 o desfazer do Fill deixa de devolver a cor por vertice'

muta "$APP/preenche.rs" '        if self.stroke.tinta_fina.is_some() {
            return Preenchido::TracoAberto;
        }' '' \
  'A5 o Fill preenche por baixo de um traco aberto'

muta "$APP/history_budget.rs" '                    + finas.as_ref().map_or(0, super::JanelaFina::bytes)' \
  '                    + finas.as_ref().map_or(0, |_| 0)' \
  'A6 o tecto volta a nao contar a janela fina de um traco'

muta "$APP/history_budget.rs" '                    + finas.as_ref().map_or(0, super::PlanoInteiro::bytes)' \
  '                    + finas.as_ref().map_or(0, |_| 0)' \
  'A7 o tecto nao conta o plano inteiro do Fill'

# ── O PAINEL ─────────────────────────────────────────────────────────────
muta "$PAN/event.rs" '    (crate::ids::SCULPT3D_COLOR_FILL, Sculpt3dIntent::ColorFill),
' '' \
  'P1 o botao sai da tabela de comandos: morto sob o dedo'

muta "$PAN/paint/brush_cor.rs" '    let y = super::widgets::command(
        ctx,
        crate::ids::SCULPT3D_COLOR_FILL,' '    let y = super::widgets::readout(
        ctx,' \
  'P2 o botao deixa de ser pintado'

# ── O CONTROLO ───────────────────────────────────────────────────────────
muta "$SCU/preenche.rs" '/// A mistura, exacta nas duas pontas — ver o cabeçalho.' \
  '/// A mistura, exacta nas duas pontas — ver o cabeçalho.
' \
  'C0 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ] || exit 1
  exit 0
fi
if [ -n "$FILTRO" ]; then
  echo "PLACAR PARCIAL (filtro MUTA_FILTRO='$FILTRO'): $sangram de $total sangram"
else
  echo "PLACAR DO FILL: $sangram de $total sangram (o C0 e' o CONTROLO e nao pode)"
fi
[ "$sangram" -ge $((total - 1)) ] || exit 1
