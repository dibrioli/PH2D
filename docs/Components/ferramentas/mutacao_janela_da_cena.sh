#!/usr/bin/env bash
# Provas de mutação da JANELA DA CENA — a porta única do mapeamento mundo↔tela do chrome.
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ CONTROLO sobre o próprio FILTRO — um filtro que casa ZERO testes sai VERDE, e isso lê-se
#      exactamente como «sobreviveu» (lição paga pela wave do projéctil, #14).
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"; FALHAS=0; TOTAL=0
guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; }

muta() {
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read(); c = s.count(old)
if c != n: sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() {
  TOTAL=$((TOTAL+1)); echo "── $1"
  local out rc corridos
  out=$(timeout "${4:-900}" cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3; FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else echo "  ✅ sangrou (de $corridos teste(s) corridos)"; fi
}

bloco() {
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then prova "$1" "$2" "$3" "${8:-900}"
  else TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1)); fi
  restaura "$4" b
}

SHELL_SRC=shells/desktop/src

echo "════ O CENSO — a janela crua volta a alimentar um aponte ════"

# ⚠️ A ancora e' a linha INTEIRA com o vizinho: `let window_size = gfx.scene_window();` sozinha
#    aparece 3 vezes neste ficheiro, e o harness aborta em vez de mutar as tres (que e' o que
#    faria a prova medir outra coisa).
bloco "uma LIGACAO volta a' janela crua" ph2d-host-desktop todo_aponte_passa_pela_janela \
  "$SHELL_SRC/input_dispatch/despacho_metodos_picks_e_arrastos.rs" 1 \
  '        let window_size = gfx.scene_window();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        let tol = ph2d_app_physics::joint_anchor_drag::SNAP_PX * gfx.camera.height_world' \
  '        let window_size = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        let tol = ph2d_app_physics::joint_anchor_drag::SNAP_PX * gfx.camera.height_world'

bloco "uma chamada INLINE volta a' janela crua" ph2d-host-desktop todo_aponte_passa_pela_janela \
  "$SHELL_SRC/vec_text_reopen.rs" 1 \
  'gfx.camera.screen_to_world((x, y), gfx.scene_window())' \
  'gfx.camera.screen_to_world((x, y), gfx.surface.size())'

bloco "a SEGUNDA FORMA da porta volta a' janela crua" ph2d-host-desktop todo_aponte_passa_pela_janela \
  "$SHELL_SRC/input_dispatch/despacho_clique_pick.rs" 1 \
  'crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size())' \
  'gfx.surface.size()'

echo "════ A GizmoCamera — a forma que leva a janela DENTRO ════"

bloco "um gizmo de arrasto volta a' janela crua" ph2d-host-desktop toda_gizmo_camera_nasce_da_banda \
  "$SHELL_SRC/flip/pose_gizmo_app.rs" 1 \
  '        let size = gfx.scene_window();' \
  '        let size = gfx.surface.size();'

echo "════ A PORTA — a delegacao que carrega a prova da identidade ════"

bloco "a porta refaz a conta em vez de DELEGAR" ph2d-host-desktop a_porta_delega_e_por_isso_herda \
  "$SHELL_SRC/scene_mapping.rs" 1 \
  '        scene_camera_window(split, self.surface.size())' \
  '        let s = self.surface.size();
        ph2d_host::WindowSize::new(s.width, (s.height as f32 * split.t()).floor() as u32)'

echo "════ A REGUA — os dois pontos cegos que ela ja' pagou ════"

bloco "a regua deixa de saltar os COMENTARIOS (acusa quem a documenta)" ph2d-host-desktop todo_aponte_passa_pela_janela \
  "shells/desktop/tests/it/todo_aponte_passa_pela_janela_da_cena.rs" 1 \
  '        let fonte = sem_prosa(&std::fs::read_to_string(p).expect("ler o fonte"));
        let ch: Vec<char> = fonte.chars().collect();' \
  '        let fonte = std::fs::read_to_string(p).expect("ler o fonte");
        let ch: Vec<char> = fonte.chars().collect();'

bloco "a regua perde o PISO de populacao (verde a medir nada)" ph2d-host-desktop todo_aponte_passa_pela_janela \
  "shells/desktop/tests/it/todo_aponte_passa_pela_janela_da_cena.rs" 1 \
  '        while let Some(k) = fonte[de..].find("screen_to_world") {' \
  '        while let Some(k) = fonte[de..].find("screen_to_world_NUNCA") {'

echo
echo "════ $((TOTAL-FALHAS)) de $TOTAL sangraram ════"
[ "$FALHAS" = 0 ] || exit 1
