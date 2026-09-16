#!/usr/bin/env bash
# Provas de mutação do TOP-20 #15 (`StateMachine`) — W0..W4.
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **A prova tem CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes faz o
#    `cargo test` sair VERDE, e o script irmão desta linha imprimiu «SOBREVIVEU» sete vezes sobre
#    um produto correcto antes de isso ser curado.
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1")"; }
restaura() { cp "$TMP/$(basename "$1")" "$1"; touch "$1"; }

conta() { # ficheiro padrão esperado
  local n; n=$(grep -c -F -- "$2" "$1")
  if [ "$n" != "$3" ]; then
    echo "  ⛔ ANCORA: '$2' aparece $n vezes em $1 (esperado $3) — a mutação NAO entrou"
    return 1
  fi
}

prova() { # nome  crate  filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum; a prova nao mediu nada"
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

LEI=crates/ph2d-ecs/src/state_machine.rs
PORTA=crates/ph2d-ecs/src/rewind_runtime.rs
PONTE=shells/desktop/src/render_loop/state_machine_tick.rs
INSP=shells/desktop/src/render_loop/inspector_statemachine.rs

# ── W0: a reposição do vivo ao rebobinar ─────────────────────────────────────
guarda "$PORTA"
conta "$PORTA" "rt.0.extend(cfg.0.iter().map(crate::timer::born));" 1 || exit 1
sed -i 's|rt.0.extend(cfg.0.iter().map(crate::timer::born));|rt.0.extend(cfg.0.iter().map(\|_\| TimerState::default()));|' "$PORTA"
sed -i 's|^use crate::{|use crate::TimerState;\nuse crate::{|' "$PORTA"
prova "REBOBINAR e' RENASCER: o autostart nao fica parado" ph2d-ecs "rebobinar_renasce_o_autostart"
restaura "$PORTA"

guarda "$PORTA"
sed -i 's|^        \*rt = FactoryRuntime::default();$|        let _ = \&mut rt;|' "$PORTA"
prova "a FABRICA gasta volta ao principio" ph2d-ecs "uma_fabrica_gasta_volta_ao_principio"
restaura "$PORTA"

guarda "$PORTA"
sed -i 's|            ent.remove::<CameraRuntime>();|            let _ = \&mut ent;|' "$PORTA"
prova "a CAMERA e' APAGADA, nao posta a zero" ph2d-ecs "a_camera_e_apagada_para_renascer"
restaura "$PORTA"

# ── W1: a lei ────────────────────────────────────────────────────────────────
guarda "$LEI"
conta "$LEI" "gasto[i] = true;" 1 || exit 1
sed -i 's|^        gasto\[i\] = true;$|        // MUTADO|' "$LEI"
prova "um SINAL e' GASTO por quem o ouve" ph2d-ecs "um_sinal_e_gasto_por_quem_o_ouve"
restaura "$LEI"

guarda "$LEI"
conta "$LEI" "let achado = m.transitions.iter().find_map(|t| {" 1 || exit 1
sed -i 's|let achado = m.transitions.iter().find_map(\|t\| {|let achado = m.transitions.iter().rev().find_map(\|t\| {|' "$LEI"
prova "a PRIMEIRA transicao satisfeita ganha" ph2d-ecs "a_primeira_transicao_satisfeita_ganha"
restaura "$LEI"

guarda "$LEI"
conta "$LEI" "|| t.to == t.from" 1 || exit 1
sed -i 's^                || t.to == t.from^                || false^' "$LEI"
prova "uma transicao para SI MESMO e' recusada" ph2d-ecs "uma_transicao_para_si_mesmo"
restaura "$LEI"

guarda "$LEI"
conta "$LEI" "        rt.started = true;" 1 || exit 1
sed -i 's|^        rt.started = true;$|        // MUTADO|' "$LEI"
prova "entrar no inicial anuncia-se UMA vez" ph2d-ecs "entrar_no_estado_inicial_anuncia_se"
restaura "$LEI"

# ── W2: a ponte ──────────────────────────────────────────────────────────────
guarda "$PONTE"
conta "$PONTE" "    ensure_runtime(world);" 1 || exit 1
sed -i 's|^    ensure_runtime(world);$|    // MUTADO|' "$PONTE"
prova "o vivo NASCE no primeiro avanco" ph2d-host-desktop "o_vivo_de_uma_maquina_nasce"
restaura "$PONTE"

guarda "$PONTE"
conta "$PONTE" "    quem.sort_unstable_by_key(|(id, _)| *id);" 1 || exit 1
sed -i 's|    quem.sort_unstable_by_key(\|(id, _)\| \*id);|    quem.reverse();|' "$PONTE"
prova "a ORDEM e' a da IDENTIDADE (HR-5)" ph2d-host-desktop "duas_maquinas_anunciam_na_ordem"
restaura "$PONTE"

# ── W3: o painel ─────────────────────────────────────────────────────────────
guarda "$INSP"
conta "$INSP" "            m.transitions.retain(|t| t.from != k && t.to != k);" 1 || exit 1
sed -i 's|            m.transitions.retain(\|t\| t.from != k \&\& t.to != k);|            // MUTADO|' "$INSP"
prova "apagar um estado LEVA as setas que o apontavam" ph2d-host-desktop "apagar_um_estado_leva_as_setas"
restaura "$INSP"

guarda "$INSP"
conta "$INSP" "                if t.from > k {" 1 || exit 1
sed -i 's|^                if t.from > k {$|                if false {|' "$INSP"
prova "…e RECUA as que apontavam depois dele" ph2d-host-desktop "apagar_um_estado_leva_as_setas"
restaura "$INSP"

# ⚠️ **A 1.ª redacção desta mutação não mutava NADA** — ela punha `has_exit: true && m…`, e
# `true && X` é `X`. O gate «sobreviveu» a uma mutação que era um no-op: *uma prova de mutação que
# não muda o programa mede o compilador*. A que fica apaga o predicado do `from`.
guarda "$INSP"
conta "$INSP" ".any(|t| t.from as usize == i && !t.on.is_empty())" 1 || exit 1
sed -i 's|.any(\|t\| t.from as usize == i \&\& !t.on.is_empty())|.any(\|t\| !t.on.is_empty())|' "$INSP"
prova "o BECO e' derivado das setas DESTE estado" ph2d-host-desktop "um_estado_sem_seta_de_saida_e_um_beco"
restaura "$INSP"

# ── W4: a cena ───────────────────────────────────────────────────────────────
SMOKE=crates/ph2d-app-components/src/statemachine_smoke.rs
guarda "$SMOKE"
conta "$SMOKE" "            repeat: true," 1 || exit 1
sed -i 's|^            repeat: true,$|            repeat: false,|' "$SMOKE"
prova "o botao da cena REPETE" ph2d-app-components "o_botao_repete_e_arranca_sozinho"
restaura "$SMOKE"

guarda "$SMOKE"
conta "$SMOKE" 'Name::new("Door (no brain)"),' 1 || exit 1
sed -i 's|Name::new("Door (no brain)"),|Name::new("Door (nao e o controlo)"),|' "$SMOKE"
prova "a cena traz o CONTROLO" ph2d-app-components "a_cena_traz_a_porta_e_o_controlo"
restaura "$SMOKE"

# ── W5: o corpo da porta (report do dono, 2026-09-15: «nao apareceu no painel a
#        seccao state machine») ─────────────────────────────────────────────────
#
# ⚠️ As duas mutações abaixo reproduzem, uma de cada vez, as DUAS metades do defeito: o cérebro
# numa entidade SEM sprite (invisível ao `pick_sprite_at_world`), e a faixa de cor por CIMA do
# corpo (o clique escolhe a placa, que não tem cérebro).
guarda "$SMOKE"
conta "$SMOKE" "        corpo_da_porta(esq)," 1 || exit 1
sed -i 's|^        corpo_da_porta(esq),$|        Transform::from_translation(Vec2::new(0.0, 0.0)),|' "$SMOKE"
prova "o que tem CEREBRO tem CORPO" ph2d-app-components "o_que_tem_cerebro_tem_corpo"
restaura "$SMOKE"

guarda "$SMOKE"
conta "$SMOKE" "    pub(super) const JUNTA_Y: f32 = 0.8;" 1 || exit 1
sed -i 's|    pub(super) const JUNTA_Y: f32 = 0.8;|    pub(super) const JUNTA_Y: f32 = BASE_Y;|' "$SMOKE"
prova "a FAIXA nao cobre o corpo" ph2d-app-components "o_dedo_do_dono_apanha_a_porta"
restaura "$SMOKE"

echo
if [ "$FALHAS" = 0 ]; then echo "TODAS as $TOTAL mutacoes sangraram."; else echo "⛔ $FALHAS de $TOTAL SOBREVIVERAM"; fi
git diff --stat -- crates shells | tail -3
exit "$FALHAS"
