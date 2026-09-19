#!/usr/bin/env bash
# Provas de mutação da W5 do suplente #21 — O RAIO NO CANVAS.
#
# ⚠️ Estas são as da W5 (a LINHA que se publica). As da LEI, das arestas e do painel estão no
# irmão `mutacao_raio_2026-09-19.sh`, e as duas correm-se.
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ CONTROLO sobre o próprio FILTRO — um filtro que casa ZERO testes sai VERDE e lê-se como
#      «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_raio_gizmo_2026-09-19.sh
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; }

muta() { # ficheiro vezes antigo novo
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read()
c = s.count(old)
if c != n:
    sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() { # nome crate filtro [alvos]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  # shellcheck disable=SC2086
  out=$(timeout 900 cargo test -p "$2" ${4:---all-targets} -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome crate filtro ficheiro vezes antigo novo [alvos]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

FASE=crates/ph2d-physics-ecs/src/bridge/ray_sensors.rs
HOLD=crates/ph2d-physics-ecs/src/bridge/hold.rs
REWIND=crates/ph2d-physics-ecs/src/bridge/rewind.rs
SHELL=shells/desktop/src/render_loop/fase_physics_overlay.rs

echo "════ A LINHA É O RAIO QUE FOI LANÇADO ════"

# (1) A ORIGEM deixa de rodar com o objecto — metade da lei local, e a metade que não se vê num
#     plano. O desenho passa a nascer onde o objecto NÃO está.
bloco "desenho: a origem nao roda" ph2d-physics-ecs a_linha_desenhada_e_o_raio \
  "$FASE" 1 \
  "            origem: [px + o[0], py + o[1]]," \
  "            origem: [px + r.origin.x, py + r.origin.y]," \
  "--test it"

# (2) O RUMO deixa de rodar — a mira de uma torreta deixa de seguir a torreta, no canvas.
bloco "desenho: o rumo nao roda" ph2d-physics-ecs a_linha_desenhada_e_o_raio \
  "$FASE" 1 \
  "        let d = gira(r.dir);" \
  "        let d = [r.dir.x, r.dir.y];" \
  "--test it"

# (3) O ALCANCE da marca deixa de ser o autorado — a linha mede outro número que o do painel.
bloco "desenho: o alcance vem de outra fonte" ph2d-physics-ecs a_linha_desenhada_e_o_raio \
  "$FASE" 1 \
  "                    raio.reach,
                    achou.map(|h| h.distance)," \
  "                    raio.reach * 0.5,
                    achou.map(|h| h.distance)," \
  "--test it"

# (4) A DIRECÇÃO publicada deixa de ser unitária: `dir = (1,1)` com `reach = 3` desenha 4,243 m
#     sobre um raio que castou 3 — o overlay a mentir sobre o alcance que se está a afinar.
bloco "desenho: o rumo nao e' unitario" ph2d-physics-ecs a_direccao_da_marca_e_unitaria \
  "$FASE" 1 \
  "            dir: [d[0] / n, d[1] / n]," \
  "            dir: [d[0], d[1]]," \
  "--test it"

# (5) A marca passa a sair só no ACERTO — some da tela justamente o raio cujo alcance é curto.
bloco "desenho: so' quem acerta e' desenhado" ph2d-physics-ecs um_raio_que_nao_ve_nada \
  "$FASE" 1 \
  "                self.ray_marks.push(ProbeMark::ray(" \
  "                if achou.is_none() {
                    continue;
                }
                self.ray_marks.push(ProbeMark::ray(" \
  "--test it"

# (6) A GUARDA da degeneração cai — um `dir` nulo vira um ponto NaN no caminho de Vello.
bloco "desenho: um raio degenerado e' publicado" ph2d-physics-ecs um_raio_degenerado \
  "$FASE" 1 \
  "        if !n.is_finite() || n <= f32::EPSILON || !r.reach.is_finite() || r.reach < 0.0 {" \
  "        if false {" \
  "--test it"

# (7) A PELE volta — o desenho encolhe e esconde metade do número que o artista mexe.
bloco "desenho: a marca desconta pele" ph2d-physics-ecs a_linha_desenhada_e_o_raio \
  "$FASE" 1 \
  "                    0.0,
                ));" \
  "                    raio.reach * 0.5,
                ));" \
  "--test it"

echo "════ O SOLVER DESARMADO ════"

# (8) O `hold` deixa de re-derivar: um `RaySensor` fica INVISÍVEL no caminho de OMISSÃO do app,
#     porque o toggle Physics nasce desmarcado.
bloco "hold: a linha nao segue o corpo" ph2d-physics-ecs com_o_solver_desarmado \
  "$HOLD" 1 \
  "        self.preview_ray_marks(sim);" \
  "        // self.preview_ray_marks(sim);" \
  "--test it"

# (9) O `hold` passa a publicar uma RESPOSTA — «perguntei e não achei» com a física desligada.
bloco "hold: publica Clear em vez de Idle" ph2d-physics-ecs com_o_solver_desarmado \
  "$FASE" 1 \
  "            self.ray_marks.push(ProbeMark::idle_ray(
                ProbeKind::Sensor,
                raio.origem,
                raio.dir,
                raio.reach,
                0.0,
            ));" \
  "            self.ray_marks.push(ProbeMark::ray(
                ProbeKind::Sensor,
                raio.origem,
                raio.dir,
                raio.reach,
                None,
                0.0,
            ));" \
  "--test it"

# (10) O `hold` deixa de apagar a história: a entrada fica de pé e o dreno do shell re-emite o
#      sinal em TODO quadro, para sempre. (A correcção da W2 que a W5 tornou visível.)
bloco "hold: as arestas sobrevivem ao desarmar" ph2d-physics-ecs desarmar_a_fisica_apaga \
  "$HOLD" 1 \
  "        self.discard_ray_history();" \
  "        // self.discard_ray_history();" \
  "--test it"

echo "════ REBOBINAR ════"

# (11) A linha do tique anterior sobrevive ao Reset — o canvas desenha os raios onde os corpos
#      ESTAVAM, porque o laço de replay não casta.
bloco "rebobinar: a linha sobrevive" ph2d-physics-ecs rebobinar_apaga_a_linha \
  "$REWIND" 1 \
  "        self.ray_marks.clear();" \
  "        // self.ray_marks.clear();" \
  "--test it"

echo "════ A COSTURA DA SHELL ════"

# (12) A shell entrega só a lista do PLAYER: compila, apaga do canvas todo `RaySensor` da cena, e
#      deixa os gates de unidade dos dois lados VERDES.
bloco "shell: os raios autorados nao sao entregues" ph2d-host-desktop the_marks_come_from_the_bridge \
  "$SHELL" 1 \
  "        let probes = [physics.player_probe_marks(), physics.ray_marks()].concat();" \
  "        let probes = physics.player_probe_marks().to_vec();" \
  "--test it"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"
