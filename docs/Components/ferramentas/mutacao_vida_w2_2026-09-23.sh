#!/usr/bin/env bash
# Provas de mutação da VIDA E DANO (plano 28, W2) — a PONTE: a vida corrida contra o mundo.
#
# Arnês IDÊNTICO ao da W1 (`mutacao_vida_2026-09-23.sh`): controlo sobre o próprio FILTRO (um
# filtro que casa ZERO testes imprime `ok` e lê-se como «sobreviveu»), `muta` a ABORTAR quando a
# âncora não aparece o número esperado de vezes, rede que restaura no EXIT/INT/TERM/PIPE e o modo
# SECO (`SECO=1`), que confere só as âncoras.
#
# ⚠️ Três famílias de mutação, e cada uma diz qual gate a apanha:
#   · as TRÊS FONTES de um golpe (o contacto sólido · o sensor · o canal do mover, este em TRÊS
#     pontes — foi a 1.ª corrida deste arnês que pediu os gates das outras duas);
#   · a LEI do toque (Began, contínuo, equipa, gasto) e as TRÊS memórias do scrub (o anel, o replay,
#     o renascer);
#   · as SAÍDAS: os factos, os sinais com os nomes do artista, e a porta que a shell drena.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_vida_w2_2026-09-23.sh
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; MUTADO="$1"; MUTADO_TAG="$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; MUTADO=""; }

MUTADO=""
MUTADO_TAG=""
ao_sair() {
  if [ -n "$MUTADO" ]; then
    echo "⚠️  interrompido com $MUTADO mutado — a restaurar"
    cp "$TMP/$(basename "$MUTADO").$MUTADO_TAG" "$MUTADO"; touch "$MUTADO"
  fi
}
trap ao_sair EXIT INT TERM PIPE

if grep -n '^bloco "[^"]*`' "$0" >&2; then
  echo "⛔ CRASE no nome de uma prova (ver as linhas acima): ele desfaz a citacao do ficheiro." >&2
  exit 2
fi

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

prova() { # nome crate filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(timeout 900 cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  # A população honesta é `passed + failed` — o `running N tests` CONTA os ignorados.
  corridos=$(printf '%s' "$out" | grep -oE 'test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed' \
             | grep -oE '[0-9]+ (passed|failed)' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
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

bloco() { # nome crate filtro ficheiro vezes antigo novo
  guarda "$4" b
  if [ "${SECO:-0}" = 1 ]; then
    TOTAL=$((TOTAL+1))
    muta "$4" "$5" "$6" "$7" || FALHAS=$((FALHAS+1))
    restaura "$4" b
    return
  fi
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

K=ph2d-physics-ecs
B=crates/ph2d-physics-ecs/src/bridge
H=$B/health.rs

echo "=== AS TRES FONTES DE UM GOLPE ==="

bloco "a bala nao entrega o que bateu" $K uma_bala_fere_quem_acerta \
  $B/projectile.rs 1 \
  '                self.toques_do_mover
                    .extend(hits.iter().filter_map(|h| h.body.map(|b| (entity, b))));' \
  ''

# ⛔⛔ Estas duas SOBREVIVERIAM sem o gate `os_dois_controladores_ferem_pelo_canal_do_mover`, que
# nasceu da leitura deste arnês: o canal vive em três pontes e só a do projéctil tinha gate.
bloco "o mover de vista de cima nao entrega o que bateu" $K os_dois_controladores \
  $B/topdown.rs 1 \
  '                self.toques_do_mover
                    .extend(hits.iter().filter_map(|h| h.body.map(|b| (entity, b))));' \
  ''

bloco "o mover de plataforma nao entrega o que bateu" $K os_dois_controladores \
  $B/player_kinmove.rs 1 \
  '            self.toques_do_mover
                .extend(hits.iter().filter_map(|h| h.body.map(|b| (m.entity, b))));' \
  ''

bloco "o contacto solido nao conta" $K uma_pedra_que_cai \
  "$H" 1 \
  '                toques.push(((a, a), (b, b)));' \
  ''

bloco "a sobreposicao de sensor nao conta" $K quem_nasce_sobreposto \
  "$H" 1 \
  '            toques.push(((forma, corpo), (dentro, dentro)));' \
  ''

bloco "o canal do mover nao e lido" $K uma_bala_fere_quem_acerta \
  "$H" 1 \
  '                toques.push(((mover, mover), (b, b)));' \
  ''

echo "=== A LEI DO TOQUE ==="

# ⛔ A queixa nº 1 da pesquisa: sem a memória, cada tique de sobreposição é um golpe novo.
bloco "todo tique de toque e um comeco" $K um_toque_fere_uma_vez \
  "$H" 1 \
  '                let comecou = !st.tocando.contains(&fonte);' \
  '                let comecou = true;'

bloco "a memoria do toque nao e guardada" $K um_toque_fere_uma_vez \
  "$H" 1 \
  '            st.tocando = fontes;' \
  ''

bloco "o continuo fere por tique e nao por segundo" $K um_toque_fere_uma_vez \
  "$H" 1 \
  '                    f64::from(dano.amount) * dt' \
  '                    f64::from(dano.amount)'

bloco "a equipa nao e perguntada" $K a_mesma_equipa_nao_fere \
  "$H" 1 \
  '                if !dano.fere(&h) {' \
  '                if false {'

bloco "a bala que some nunca e gasta" $K uma_bala_que_some \
  "$H" 1 \
  '                if comecou && dano.on_hit == OnHit::Vanish {' \
  '                if false {'

# ⛔ A semente mistura a do componente com a identidade; sem ela toda vida sorteia igual.
bloco "a esquiva ignora a semente" $K a_esquiva_e_determinista \
  "$H" 1 \
  '        rng: h.seed ^ id.unwrap_or(0),' \
  '        rng: 0,'

echo "=== AS TRES MEMORIAS DO SCRUB ==="

bloco "o anel nao guarda a vida" $K um_scrub_devolve \
  $B/tape.rs 1 \
  '                health: self.health_state.clone(),' \
  '                health: Default::default(),'

bloco "o anel nao devolve a vida" $K um_scrub_devolve \
  $B/tape.rs 1 \
  '            self.health_state = m.health.clone();' \
  ''

bloco "o replay nao anda as vidas" $K um_scrub_devolve \
  $B/rewind.rs 1 \
  '            self.depois_do_passo(sim, false);' \
  ''

bloco "o replay publica os golpes" $K um_scrub_devolve \
  "$H" 1 \
  '                if publicar {
                    for kind in factos(&antes, &st.vida) {' \
  '                if true {
                    for kind in factos(&antes, &st.vida) {'

echo "=== AS SAIDAS ==="

bloco "um morto morre em todo golpe" $K morre_uma_vez \
  "$H" 1 \
  '    if !antes.morta() && depois.morta() {' \
  '    if depois.morta() {'

bloco "a morte grita o nome do dano" $K a_vida_grita_os_nomes \
  $B/signals.rs 1 \
  '                HealthEventKind::Died => &vida.on_death,' \
  '                HealthEventKind::Died => &vida.on_damage,'

bloco "quem grita e quem bateu" $K a_vida_grita_os_nomes \
  $B/signals.rs 1 \
  '                source: ev.target,
                other: ev.source,' \
  '                source: ev.source,
                other: ev.target,'

# ⛔⛔ A lei que protege o trabalho do artista: sem ela um inimigo de DOCUMENTO sai da cena.
bloco "um objecto de documento sai da cena" $K so_sai_da_cena \
  $B/mortes.rs 1 \
  '            if is_transient(world, entity) && !out.iter().any(|d| d.entity == entity) {' \
  '            if !out.iter().any(|d| d.entity == entity) {'

bloco "a morte da vida nao e anunciada" $K so_sai_da_cena \
  $B/mortes.rs 1 \
  '                junta(ev.target, DeathCause::Killed);' \
  ''

bloco "a morte da vida sai como gasta" $K so_sai_da_cena \
  $B/mortes.rs 1 \
  '                junta(ev.target, DeathCause::Killed);' \
  '                junta(ev.target, DeathCause::Spent);'

# ⛔⛔ Esta SOBREVIVEU à 1.ª corrida (22 de 23): a bala do gate também acaba o VOO ao bater, e o
# `projectile_done` anunciava-a pelo outro ramo. Só um `Vanish` que NÃO voa (a pedra que cai)
# separa os dois — o gate ganhou essa metade, e a 2.ª corrida sangra.
bloco "a bala gasta nao e anunciada" $K so_sai_da_cena \
  $B/mortes.rs 1 \
  '        for &e in self.damage_spent() {
            junta(e, DeathCause::Spent);
        }' \
  ''

echo
echo "=== $((TOTAL-FALHAS)) de $TOTAL sangraram ==="
exit "$FALHAS"
