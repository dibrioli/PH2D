#!/usr/bin/env bash
# Provas de mutação da CURA do report do dono (19/09):
#   «vc esqueceu de colocar física no jogador. quando eu coloquei a física travou, não consigo dar play»
#
# Duas metades: a CENA do abanão passa a dar um corpo cinemático ao herói, e a SEMENTE faz um corpo
# anexado debaixo de um controlador cinemático nascer `Kinematic` em vez de `Dynamic`.
#
# Arnês IDÊNTICO ao das waves anteriores desta linha — controlo sobre o próprio FILTRO (um filtro
# que casa ZERO testes imprime `ok` e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora
# não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_corpo_do_jogador_2026-09-19.sh
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

CENA=crates/ph2d-app-components/src/shake_smoke.rs
SEED=crates/ph2d-app-physics/src/physics_seed.rs
PORTA=crates/ph2d-physics-ecs/src/bridge/pose_owner.rs

echo "════ A CENA (ph2d-app-components) ════"

# 1) O herói perde o CORPO — é o defeito que o dono reportou, à letra.
bloco "cena: o heroi sem corpo" ph2d-app-components \
  o_heroi_anda_quando_o_dono "$CENA" 1 \
  "            RigidBody {
                kind: BodyKind::Kinematic,
            }," \
  ""

# 2) O corpo do herói é DINÂMICO — o mover fica inerte e a gravidade leva-o.
bloco "cena: o corpo do heroi e dinamico" ph2d-app-components \
  o_heroi_anda_quando_o_dono "$CENA" 1 \
  "                kind: BodyKind::Kinematic," \
  "                kind: BodyKind::Dynamic,"

# 3) O herói perde o COLLIDER — um corpo sem forma não entra no controlador.
bloco "cena: o heroi sem collider" ph2d-app-components \
  o_heroi_anda_quando_o_dono "$CENA" 1 \
  "            Collider {
                shape: ColliderShape::Ball { radius: 0.45 },
                ..Collider::default()
            }," \
  ""

# 10) O herói deixa de ler o TECLADO — a lei fica certa e a ENTREGA morre.
#     ⚠️ É a metade que o canal INTERNO da ponte NÃO mede: com `set_player_input` escrito à mão o
#     gate irmão continua VERDE, que é exactamente a rotura que o TOP-20 #13 pagou por report.
bloco "cena: o heroi nao le o teclado" ph2d-app-components \
  a_porta_do_teclado_alcanca_o_heroi "$CENA" 1 \
  "                direction: DirectionMode::Free," \
  "                direction: DirectionMode::Free,
                default_controls: false,"

echo
echo "════ A SEMENTE (ph2d-app-physics) ════"

# 4) A semente escreve o ponto NEUTRO — é o produto de antes da cura.
bloco "seed: escreve o ponto neutro" ph2d-host-desktop \
  nasce_cinematico "$SEED" 1 \
  "    sim.world_mut().entity_mut(entity).insert(RigidBody {
        kind: BodyKind::Kinematic,
    });" \
  "    sim.world_mut().entity_mut(entity).insert(RigidBody {
        kind: BodyKind::default(),
    });" \
  "--bins"

# 5) A guarda do CONTROLADOR é invertida — todo corpo nasce cinemático.
#    ⚠️ É o CONTROLO que sangra aqui, e não o gate positivo: sem ele a cura leria como
#    «todo corpo nasce cinemático», que é outro produto.
bloco "seed: a guarda do controlador invertida" ph2d-host-desktop \
  um_corpo_num_objecto_comum "$SEED" 1 \
  "    if !ph2d_physics_ecs::controlador_cinematico(sim.world(), entity) {" \
  "    if false {"

# 6) A guarda do PONTO NEUTRO desaparece — a semente rebaixa um corpo autorado.
bloco "seed: sem a guarda do ponto neutro" ph2d-host-desktop \
  a_semente_nao_reescreve "$SEED" 1 \
  "    if corpo.kind != BodyKind::default() {
        return;
    }" \
  "    if false {
        return;
    }"

# 7) A entrada do `RigidBody` sai da tabela — a 1.ª ordem de chegada (o gesto do dono) deixa de
#    ser semeada, e a 2.ª (a paleta) continua a funcionar.
bloco "seed: sem a entrada do RigidBody" ph2d-host-desktop \
  um_corpo_anexado_a_um_mover "$SEED" 1 \
  "    (\"ph2d::physics::RigidBody\", seed_kinematic_controller_body)," \
  ""

# 8) A entrada do `TopDownPlayer` sai da tabela — a 2.ª ordem (a cascata da paleta) morre.
bloco "seed: sem a entrada do TopDownPlayer" ph2d-host-desktop \
  escolher_um_mover_na_paleta "$SEED" 1 \
  "    (
        \"ph2d::physics::TopDownPlayer\",
        seed_kinematic_controller_body,
    )," \
  ""

echo
echo "════ A PORTA (ph2d-physics-ecs) ════"

# 9) A porta deixa de reconhecer o projéctil — a ponte dele volta a ser conduzida pela cena, e a
#    semente entrega-lhe um corpo dinâmico.
#    ⚠️ **É o gate da PONTE que sangra**, e é isso que prova que a porta tem DOIS leitores: uma
#    segunda cópia da lista na semente deixaria este verde.
bloco "porta: o projectil deixa de escrever a propria pose" ph2d-physics-ecs \
  ele_voa_sozinho_para_onde_o_corpo_esta_virado "$PORTA" 1 \
  "    world.get::<TopDownPlayer>(entity).is_some()
        || world
            .get::<crate::components::ProjectileMotion>(entity)
            .is_some()" \
  "    world.get::<TopDownPlayer>(entity).is_some()"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutações SANGRARAM."
else
  echo "⛔ $FALHAS de $TOTAL não sangraram."
fi
exit "$FALHAS"
