#!/usr/bin/env bash
# Provas de mutação do suplente #21 — O RAIO (a LEI, as arestas, o rebobinar e o PAINEL).
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ CONTROLO sobre o próprio FILTRO — um filtro que casa ZERO testes sai VERDE e lê-se como
#      «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_raio_2026-09-19.sh
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
REWIND=crates/ph2d-physics-ecs/src/bridge/rewind.rs
SINAIS=crates/ph2d-physics-ecs/src/bridge/signals.rs
VOCAB=crates/ph2d-editor-core/src/ray_edits.rs
PONTE=crates/ph2d-app-components/src/ray_inspector.rs

echo "════ A LEI — ordem · métrica · direcção ════"

# (1) O raio devolve o mais LONGE: a coluna ORDEM, que a composição não tem.
bloco "lei: o mais longe em vez do mais perto" ph2d-physics-ecs um_raio_ve_o_mais_perto \
  "$FASE" 1 \
  "let Some(h) = self.world.cast_ray(origem, d, r.reach, excluir, r.layer) else {" \
  "let Some(h) = self.world.cast_ray(origem, d, r.reach * 0.01, excluir, r.layer) else {" \
  "--test it"

# (2) A DIRECÇÃO deixa de contar — o defeito que uma FORMA tem por construção.
bloco "lei: o raio deixa de apontar" ph2d-physics-ecs um_raio_nao_ve_o_que_esta_atras \
  "$FASE" 1 \
  "let d = gira(r.dir);" \
  "let d = gira(ph2d_core::Vec2::new(r.dir.x.abs(), r.dir.y));" \
  "--test it"

# (3) A pose deixa de rodar a ORIGEM: metade da lei local, e a metade que não se vê num plano.
bloco "lei: a origem nao roda com o objecto" ph2d-physics-ecs um_raio_roda_com_o_objecto \
  "$FASE" 1 \
  "                let o = gira(r.origin);" \
  "                let o = [r.origin.x, r.origin.y];" \
  "--test it"

# (4) O corpo dono deixa de ser excluído — «um personagem acha-se no chão para sempre».
bloco "lei: o raio ve-se a si mesmo" ph2d-physics-ecs um_raio_nao_se_ve_a_si_mesmo \
  "$FASE" 1 \
  "                            Some(b.handle)," \
  "                            None," \
  "--test it"

echo "════ AS ARESTAS ════"

# (5) Trocar de alvo deixa de ser uma saída e uma entrada: a porta reage ao 1.º e ignora os outros.
bloco "aresta: trocar de alvo fica calado" ph2d-physics-ecs trocar_de_alvo_e_uma_saida \
  "$FASE" 1 \
  "                Some(anterior) if anterior.body == h.body => {}" \
  "                Some(_) => {}" \
  "--test it"

# (6) A SAÍDA nunca soa — quem escuta «perdi de vista» fica à espera para sempre.
bloco "aresta: a saida nunca soa" ph2d-physics-ecs o_raio_grita_ao_ver \
  "$FASE" 1 \
  "                self.ray_exits.push((e, anterior.body));
            }" \
  "                let _ = anterior;
            }" \
  "--test it"

# (7) A cerca por TAG deixa de valer no braço dos raios — o filtro que custou zero linhas.
bloco "sinais: o filtro de tag salta os raios" ph2d-physics-ecs a_cerca_de_tag_vale_para_o_raio \
  "$SINAIS" 1 \
  "                if let Some(name) = nome(source).filter(|_| passa(source, other)) {" \
  "                if let Some(name) = nome(source) {" \
  "--test it"

echo "════ O REBOBINAR — o QUARTO mapa desta família ════"

# (8) O mapa sobrevive ao Reset: a 2.ª corrida começa com o raio a «já ver», e a entrada não soa.
bloco "rebobinar: o mapa do raio sobrevive" ph2d-physics-ecs rebobinar_faz_o_raio \
  "$REWIND" 1 \
  "        self.ray_hits.clear();" \
  "        // self.ray_hits.clear();" \
  "--test it"

echo "════ O PAINEL — a queixa, e a ordem dela ════"

# (9) A queixa mais GERAL passa à frente da mais específica: dizer «não vê nada» a quem tem a
#     direcção a zero é mandá-lo resolver a metade errada.
bloco "painel: a queixa troca de ordem" ph2d-editor-core a_queixa_vai_da_mais_especifica \
  "$VOCAB" 1 \
  "        if self.dir_x == 0.0 && self.dir_y == 0.0 {
            return Some(RayQueixa::SemDireccao);
        }" \
  "        if self.sees.is_empty() {
            return Some(RayQueixa::NaoVeNada);
        }" \
  "--lib"

# (10) Um nome só de espaços passa a contar como voz — a regra do `SignalOnHit`, ao contrário.
bloco "painel: espacos contam como nome" ph2d-editor-core a_queixa_vai_da_mais_especifica \
  "$VOCAB" 1 \
  "        if self.on_enter.trim().is_empty() && self.on_exit.trim().is_empty() {" \
  "        if self.on_enter.is_empty() && self.on_exit.is_empty() {" \
  "--lib"

echo "════ A PONTE do painel ════"

# (11) A secção deixa de existir para quem não tem os DOIS componentes — um raio sem voz é um raio
#      legítimo, e esconder-lhe a secção inteira é o defeito.
bloco "ponte: exige os dois componentes" ph2d-app-components um_raio_sem_voz_tem_seccao \
  "$PONTE" 1 \
  "    let s = world.get::<RaySignals>(e);" \
  "    let s = Some(world.get::<RaySignals>(e)?);" \
  "--lib"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"
