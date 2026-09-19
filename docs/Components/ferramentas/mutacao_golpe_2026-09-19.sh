#!/usr/bin/env bash
# Provas de mutação do SUPLENTE #24 — O SINAL SABE QUEM (a tabela passa a saber de quem veio).
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE, e isso
#      lê-se exactamente como «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes** —
#      uma mutação que não entra lê-se exactamente como uma que sobreviveu.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_golpe_2026-09-19.sh
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

LEI=crates/ph2d-ecs/src/signal_actions.rs
ORIGEM=crates/ph2d-runtime/src/lib.rs
PONTE=crates/ph2d-app-components/src/signal_actions_bridge.rs
FASE=shells/desktop/src/render_loop/fase_tabela_de_accoes.rs
CENA=crates/ph2d-app-components/src/dano_smoke.rs

echo "════ W1 — a CERCA e o OUTRO LADO (ph2d-ecs) ════"

# (1) A cerca fechada deixa de fechar: o tiro num inimigo volta a atingir os dez.
bloco "cerca: Myself -> passa sempre" ph2d-ecs a_cerca_faz_um_golpe "$LEI" 1 \
  "SignalFrom::Myself => quem_falou == Some(reactor)," \
  "SignalFrom::Myself => true,"

# (2) ⚠️ A armadilha SUBTIL: ler «sem sujeito» como «qualquer um». Um sinal anónimo passaria a
#     cerca FECHADA, e o modo de falha é MUDO — a cena só falha quando alguém publica sem sujeito.
bloco "cerca: sem sujeito lido como curinga" ph2d-ecs um_sinal_sem_sujeito "$LEI" 1 \
  "SignalFrom::Myself => quem_falou == Some(reactor)," \
  "SignalFrom::Myself => quem_falou.is_none() || quem_falou == Some(reactor),"

# (3) A linha volta a reagir por NOME e não por DISPARO: dois eventos iguais num quadro dão 1 efeito.
bloco "linha: um efeito por NOME" ph2d-ecs uma_linha_reage_por_disparo "$LEI" 1 \
  "for disparo in fired.iter().filter(|d| d.nome == action.on) {" \
  "for disparo in fired.iter().filter(|d| d.nome == action.on).take(1) {"

# (4) O «outro lado» passa a ser QUEM FALOU — a bala deixa de morrer e o alvo morre duas vezes.
bloco "alvo: Other lido como o falante" ph2d-ecs o_outro_lado_e_quem_bateu "$LEI" 1 \
  "SignalTarget::Other => vivo(world, disparo.outro).into_iter().collect()," \
  "SignalTarget::Other => vivo(world, disparo.quem).into_iter().collect(),"

# (5) O lado que já saiu da cena deixa de ser conferido: um efeito sobre bits reciclados.
bloco "outro lado: sem conferir o mundo" ph2d-ecs um_lado_que_ja_saiu_da_cena "$LEI" 1 \
  "    e.filter(|&e| world.get_entity(e).is_ok())" \
  "    e"

# (6) O verbo que APAGA passa a declarar que lê o `arg` — o painel pintaria um campo morto.
bloco "Destroy: passa a ler o arg" ph2d-ecs o_verbo_que_apaga "$LEI" 1 \
  "SignalVerb::StartTimer | SignalVerb::StopTimer | SignalVerb::AddToCounter" \
  "SignalVerb::StartTimer | SignalVerb::StopTimer | SignalVerb::AddToCounter | SignalVerb::Destroy"

echo "════ W2 — a PORTA de «quem falou» (ph2d-runtime) ════"

# (7) O contacto deixa de ter o outro lado: `Who Hit` passa a apontar a ninguém em todo o app.
bloco "origem: o contacto perde o outro lado" ph2d-runtime so_o_contacto_tem_o_outro_lado "$ORIGEM" 1 \
  "            Self::Contact { other, .. } => Some(*other)," \
  "            Self::Contact { .. } => None,"

# (8) Uma origem COM sujeito passa a responder que não tem — a cerca fechada cala toda a cena.
bloco "origem: o timer perde o sujeito" ph2d-runtime as_tres_origens_sem_sujeito "$ORIGEM" 1 \
  "            Self::Timeline { .. } | Self::Control | Self::Motion { .. } => None," \
  "            Self::Timeline { .. } | Self::Control | Self::Motion { .. } | Self::Timer { .. } => None,"

echo "════ W3 — a PONTE (ph2d-app-components) ════"

# (9) O `Destroy` deixa de respeitar a fronteira e apaga DOCUMENTO — o `Ctrl+Z` herda a remoção.
bloco "Destroy: apaga o documento" ph2d-app-components o_destroy_tira_quem_nasceu "$PONTE" 1 \
  "    if !ph2d_ecs::is_transient(sim.world(), fx.target) {" \
  "    if false {"

# (10) A origem morre na leitura — a forma EXACTA do defeito que esta wave existiu para curar.
bloco "leitura: a origem morre no map" ph2d-app-components a_origem_atravessa_a_leitura "$PONTE" 1 \
  "        de(s.origin.quem())," \
  "        None,"

echo "════ W4 — a FASE do quadro (shells/desktop) ════"

# (11) A fase volta a ler só o NOME: os gates da lei ficam todos verdes e a cena inteira mente.
bloco "fase: volta a ler so' o nome" ph2d-host-desktop a_fase_le_pela_porta "$FASE" 1 \
  "            .map(signal_actions::lido)" \
  "            .map(|s| (s.name.to_string(), None, None))"

# (12) As mortes deixam de chegar ao despachante: o alvo grita, a tabela resolve, e nada sai da cena.
bloco "fase: as mortes nao chegam ao dreno" ph2d-host-desktop a_fase_le_pela_porta "$FASE" 1 \
  "        deaths.extend(r.mortes);" \
  "        let _ = r.mortes;"

echo "════ W5 — a CENA (o que o dono vê) ════"

# (13) A cerca da bala cai: o herói a passar por um alvo mata-o, e dois alvos matam-se um ao outro.
bloco "cena: sem o filtro da bala" ph2d-app-components so_a_bala_passa_a_cerca "$CENA" 1 \
  "            SignalTagFilter(bala.0)," \
  "            SignalTagFilter(0),"

# (14) Os postos encostam-se — a geometria que a 1.ª redacção desta cena violou por sorteio.
bloco "cena: os postos encostam" ph2d-app-components dois_alvos_nunca_se_tocam "$CENA" 1 \
  "const ESPACO: f32 = 2.2;" \
  "const ESPACO: f32 = 0.5;"

# (15) As duas fileiras no MESMO y — o par que uma régua só sobre o `ESPACO` não veria.
bloco "cena: as duas fileiras no mesmo y" ph2d-app-components dois_alvos_nunca_se_tocam "$CENA" 1 \
  "const Y_BAIXO: f32 = -2.2;" \
  "const Y_BAIXO: f32 = 2.2;"

# (16) O CONTROLO da cena perde-se: as duas fileiras passam a ser a mesma experiência.
bloco "cena: o controlo ganha a cerca" ph2d-app-components as_duas_fileiras_diferem "$CENA" 1 \
  "        SignalFrom::Anyone," \
  "        SignalFrom::Myself,"

# (17) O roteiro nomeia um rótulo que a tela não tem — o passo impossível que o dono aprova.
bloco "roteiro: rotulo que nao existe" ph2d-app-components o_roteiro_nomeia_rotulos "$CENA" 1 \
  '`From Myself` nos vermelhos' \
  '`So eu` nos vermelhos'

# (18) As fábricas ficam sem receita: a cena abre VAZIA e as leis acima continuam verdes.
bloco "cena: as receitas por resolver" ph2d-app-components as_duas_fabricas_apontam "$CENA" 1 \
  "    resolver_receitas(world);" \
  "    let _ = &world;"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"
