#!/usr/bin/env bash
# Provas de mutação da VIGIA DO CONTADOR — a lei, a porta, a ponte, a secção e a cena.
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE, e isso
#      lê-se exactamente como «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes.**
#    o sangue que elas procuram.
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

prova() { # nome crate filtro [timeout_s]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(timeout "${4:-900}" cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  if [ "$rc" = 124 ]; then
    echo "  ✅ sangrou (o teste nao acabou em ${4}s — o prazo e' o que o fazia acabar)"
    return
  fi
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

# um bloco = guarda, muta, prova, restaura. Se a âncora falhar, restaura e conta como falha.
bloco() { # nome crate filtro ficheiro vezes antigo novo [timeout]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-900}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}


ECS=crates/ph2d-ecs/src
FAMILIA=crates/ph2d-app-components/src
PAINEL=crates/ph2d-panel-inspector/src
SHELL=shells/desktop/src

echo "════ A LEI — a aresta, o zero e o «só uma vez» ════"

bloco "a aresta vira NIVEL (dispara por quadro)" ph2d-ecs a_vigia_fala_na_subida \
  "$ECS/counter_watch.rs" 1 \
  'let sobe = tem && !state.held;' \
  'let sobe = tem;'

bloco "um contador AUSENTE passa a ler zero" ph2d-ecs um_contador_que_nao_existe \
  "$ECS/counter_watch.rs" 1 \
  'let tem = soma.is_some_and(|v| row.compare.holds(v, row.value));' \
  'let tem = row.compare.holds(soma.unwrap_or(0), row.value);'

bloco "nascer JA' satisfeita (a travessia do tique 0 some)" ph2d-ecs ja_satisfeita_ao_nascer \
  "$ECS/counter_watch.rs" 1 \
  '        held: false,' \
  '        held: true,'

bloco "o «so' uma vez» deixa de calar a segunda" ph2d-ecs o_so_uma_vez_cala \
  "$ECS/counter_watch.rs" 1 \
  '(row.once && state.fired)' \
  '(false && state.fired)'

bloco "a regra MUDA deixa de seguir o mundo" ph2d-ecs sem_nome_de_sinal_ela_cala_se \
  "$ECS/counter_watch.rs" 1 \
  '    state.held = tem;
    if !sobe' \
  '    if !sobe'

bloco "o reconcile deixa de esquecer um slot removido" ph2d-ecs o_reconcile_cresce_e_encolhe \
  "$ECS/counter_watch.rs" 1 \
  '    rt.0.resize(cfg.0.len(), born());' \
  '    if rt.0.len() < cfg.0.len() { rt.0.resize(cfg.0.len(), born()); }'

bloco "as comparacoes trocam de sentido" ph2d-ecs as_tres_comparacoes \
  "$ECS/counter_watch.rs" 1 \
  'Self::AtMost => valor <= limiar,' \
  'Self::AtMost => valor < limiar,'

echo "════ A PORTA — a soma que o placar e a regra partilham ════"

bloco "a porta le «o primeiro» em vez de SOMAR" ph2d-app-components a_vigia_le_a_soma \
  "$ECS/counter.rs" 1 \
  '            total = total.saturating_add(rt.value);' \
  '            total = rt.value;'

bloco "REBOBINAR deixa de re-armar a aresta" ph2d-ecs rebobinar_re_arma_a_aresta \
  "$ECS/rewind_runtime.rs" 1 \
  '        rt.0.clear();
        rt.0.resize(cfg.0.len(), crate::counter_watch::born());' \
  '        rt.0.resize(cfg.0.len(), crate::counter_watch::born());'

echo "════ A PONTE — o relogio, as orfas e os slots ════"

bloco "a ponte fala com o relogio PARADO" ph2d-app-components parada_ela_nao_fala \
  "$FAMILIA/counter_watch_bridge.rs" 1 \
  '    if !playing || ticks == 0 {' \
  '    if false {'

bloco "os slots so' nascem com o relogio a andar" ph2d-app-components parada_ela_nao_fala \
  "$FAMILIA/counter_watch_bridge.rs" 1 \
  '    // ── Os slots primeiro, e sempre ──────────────────────────────────────────' \
  '    if !playing { return out; }'

echo "════ A SECCAO — o clique, o indice e o CHIP ════"

bloco "a escolha da comparacao manda sempre a regra 0" ph2d-panel-inspector o_chip_da_comparacao_abre \
  "$PAINEL/event_counter_watch.rs" 1 \
  'push(host, bits, E::Compare(sel_u8, u8::try_from(i).unwrap_or(0)));' \
  'push(host, bits, E::Compare(0, u8::try_from(i).unwrap_or(0)));'

bloco "as opcoes do chip saem do populate (mortas sob o dedo)" ph2d-panel-inspector o_chip_da_comparacao_abre \
  "$PAINEL/populate_counter_watch.rs" 1 \
  '    register_button_ids(store, &crate::ids::INSP_WATCH_CMP_OPT);' \
  '    let _ = &crate::ids::INSP_WATCH_CMP_OPT;'

bloco "o rect do chip nao chega ao passe diferido" ph2d-panel-inspector o_chip_da_comparacao_abre \
  "$PAINEL/sections/counter_watch.rs" 1 \
  '        crate::state_popovers::set_pending_watch_dd(Some((row.compare, linha.control)));' \
  '        let _ = row.compare;'

bloco "o [+] deixa de abrir a regra que nasceu" ph2d-panel-inspector os_dois_botoes_da_lista \
  "$PAINEL/event_counter_watch.rs" 1 \
  '            panel.watch_selected = info.rows.len();' \
  '            panel.watch_selected = 0;'

bloco "a caixa manda sempre TRUE em vez do contrario" ph2d-panel-inspector a_caixa_do_so_uma_vez \
  "$PAINEL/event_counter_watch.rs" 1 \
  'push(host, bits, E::Once(sel_u8, !info.rows[sel].once));' \
  'push(host, bits, E::Once(sel_u8, true));'

echo "════ A TRADUCAO e os TECTOS ════"

bloco "a traducao da comparacao troca dois bracos" ph2d-host-desktop o_chip_da_comparacao_cobre \
  "$FAMILIA/counter_watch_inspector.rs" 1 \
  '        1 => Compare::AtLeast,
        2 => Compare::Exactly,' \
  '        1 => Compare::Exactly,
        2 => Compare::AtLeast,'

echo "════ A CENA — o controlo, os nomes e a visibilidade ════"

bloco "os dois contadores passam a SOMAR (mesmo nome)" ph2d-app-components os_dois_contadores_nao_se_somam \
  "$FAMILIA/counter_watch_smoke.rs" 1 \
  'pub const CONTROLO: &str = "vidas_controlo";' \
  'pub const CONTROLO: &str = "vidas";'

bloco "o controlo GANHA a vigia" ph2d-app-components so_o_heroi_tem_a_vigia \
  "$FAMILIA/counter_watch_smoke.rs" 1 \
  '        Name::new("Controlo (sem vigia)"),' \
  '        Name::new("Controlo (sem vigia)"),
        CounterWatch(vec![regra(0, "morri_controlo")]),'

bloco "os dois escutam o MESMO nome de morte" ph2d-app-components o_controlo_tem_a_mesma_fiacao \
  "$FAMILIA/counter_watch_smoke.rs" 1 \
  '            apaga(&format!("{CONTROLO_LUZ}3"), "luz3_controlo"),' \
  '            apaga(&format!("{CONTROLO_LUZ}3"), "luz3"),'

bloco "o heroi perde a Visibility (o Hide fica INERTE)" ph2d-host-desktop o_heroi_some_ao_chegar_a_zero \
  "$FAMILIA/counter_watch_smoke.rs" 1 \
  '        Visibility::visible(),
        Counter {
            name: HEROI.into(),' \
  '        Counter {
            name: HEROI.into(),'

bloco "a vigia sai do laco do quadro" ph2d-host-desktop o_heroi_some_ao_chegar_a_zero \
  "$SHELL/render_loop/counter_watch_chain_tests.rs" 1 \
  '        nomes.extend(f.disparos.into_iter().map(|(_, _, n)| n));' \
  '        let _ = &f.disparos;'

bloco "o morto continua a perder vidas (sem StopTimer)" ph2d-app-components a_morte_do_heroi_para_tambem \
  "$FAMILIA/counter_watch_smoke.rs" 1 \
  '                verb: SignalVerb::StopTimer,' \
  '                verb: SignalVerb::Hide,'

bloco "uma LUZ do heroi fica sem quem a apague" ph2d-app-components as_tres_luzes_do_heroi \
  "$FAMILIA/counter_watch_smoke.rs" 1 \
  '            apaga(&format!("{HEROI_LUZ}2"), "luz2"),' \
  '            '

bloco "duas regras no MESMO limiar (as luzes apagam juntas)" ph2d-app-components as_tres_regras_estao_em_limiares \
  "$FAMILIA/counter_watch_smoke.rs" 1 \
  '            regra(1, "luz2"),' \
  '            regra(2, "luz2"),'

echo
echo "════ $((TOTAL-FALHAS)) de $TOTAL sangraram ════"
[ "$FALHAS" = 0 ] || exit 1
