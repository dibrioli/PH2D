#!/usr/bin/env bash
# Provas de mutação da ARMA (`WeaponFire` + o leque da `Factory`).
#
# Arnês IDÊNTICO ao das waves anteriores desta linha — controlo sobre o próprio FILTRO (um filtro
# que casa ZERO testes imprime `ok` e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora
# não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_arma_2026-09-19.sh
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

LEI=crates/ph2d-ecs/src/weapon.rs
FABRICA=crates/ph2d-ecs/src/factory.rs
PONTE=crates/ph2d-app-components/src/weapon_bridge.rs
CENA=crates/ph2d-app-components/src/weapon_smoke.rs
EDITS=crates/ph2d-editor-core/src/weapon_edits.rs
POPULATE=crates/ph2d-panel-inspector/src/populate_weapon.rs
SYNC=crates/ph2d-panel-inspector/src/sync_sections.rs
MOTORES=shells/desktop/src/render_loop/motores_do_quadro.rs

echo "════ A LEI (ph2d-ecs) ════"

# 1) O primeiro tiro deixa de ser imediato — a arma nasce em cadencia.
# ⚠️⚠️ **A 1.ª redação punha `1` e era um NO-OP:** a lei faz `saturating_sub(dt_us)` ANTES de
#    olhar o gatilho, e `1 − 16 666` satura em `0` — o tiro saia na mesma. *Uma mutação que nao muda
#    o observavel le^-se exactamente como uma que sobreviveu.*
bloco "lei: o primeiro tiro atrasado" ph2d-ecs \
  o_primeiro_tiro_sai_no_tique_zero "$LEI" 1 \
  "        cooldown_left_us: 0,
        reload_left_us: 0," \
  "        cooldown_left_us: 1_000_000,
        reload_left_us: 0,"

# 2) O tiro nao arma a cadencia — segurar volta a dar 60 por segundo.
bloco "lei: o tiro nao arma a cadencia" ph2d-ecs \
  segurar_o_gatilho_um_segundo_da_a_cadencia "$LEI" 1 \
  "            st.cooldown_left_us = us(cfg.cooldown_ms);" \
  "            st.cooldown_left_us = 0;"

# 3) A recarga SOMA em vez de repor — a armadilha exacta do `Add to Counter`.
bloco "lei: a recarga soma" ph2d-ecs \
  recarregar_com_balas_dentro_nao_soma "$LEI" 1 \
  "            out.municao = mun.cheio;" \
  "            out.municao = mun.tem + mun.cheio;"

# 4) O gatilho dispara DURANTE a recarga — ela vira decoracao.
bloco "lei: dispara a recarregar" ph2d-ecs \
  durante_a_recarga_o_gatilho_fica_mudo "$LEI" 1 \
  "    if pediu_tiro && !st.reloading && st.cooldown_left_us == 0 {" \
  "    if pediu_tiro && st.cooldown_left_us == 0 {"

# 5) A municao pode ir a NEGATIVO — o defeito que a composicao de hoje tem.
bloco "lei: a municao vai a negativo" ph2d-ecs \
  o_pente_acaba_e_a_arma_para "$LEI" 1 \
  "        if !mun.existe || out.municao > 0 {" \
  "        if true {"

# 6) Pedir recarga a meio REINICIA a que corre — a arma nunca fica pronta.
bloco "lei: a recarga reinicia-se" ph2d-ecs \
  pedir_recarga_a_meio_nao_reinicia "$LEI" 1 \
  "    if pediu_recarga && !st.reloading && cfg.reload_ms > 0 && precisa {" \
  "    if pediu_recarga && cfg.reload_ms > 0 && precisa {"

# 7) `existe = false` deixa de ser municao INFINITA.
bloco "lei: sem contador a arma fica seca" ph2d-ecs \
  sem_contador_a_municao_e_infinita "$LEI" 1 \
  "        if !mun.existe || out.municao > 0 {" \
  "        if out.municao > 0 {"

echo
echo "════ O LEQUE (a fabrica) ════"

# 8) O leque TOCA no gerador mesmo a zero — a sequencia desloca-se em silencio.
# ⚠️ **A 1.ª redação levava asteriscos de markdown na âncora** e casou ZERO vezes — o arnês
#    abortou em voz alta (`⛔ ANCORA`), que é exactamente o que ele existe para fazer.
bloco "leque: o gerador anda com leque zero" ph2d-ecs \
  com_leque_zero_o_gerador_nem_e_tocado "$FABRICA" 1 \
  "                    Some(m) if f.spread_deg != 0.0 => {" \
  "                    Some(m) if true => {"

# 9) O desvio sai do laco — a rajada inteira torta em vez de aberta.
bloco "leque: o desvio e' por RAJADA" ph2d-ecs \
  o_desvio_e_por_copia "$FABRICA" 1 \
  "                    Some(m + estado.meio() * f.spread_deg.to_radians())" \
  "                    Some(m + 0.4 * f.spread_deg.to_radians())"

echo
echo "════ A PONTE (ph2d-app-components) ════"

# 10) A ponte soma TODOS os contadores com o nome — as duas armas partilham municao.
bloco "ponte: o pente e' a soma global" ph2d-app-components \
  cada_arma_gasta_o_proprio_pente "$PONTE" 1 \
  "                (Some(c), Some(rt)) if c.name.trim() == alvo => Municao {" \
  "                (Some(c), Some(rt)) if !alvo.is_empty() && c.name.trim() != alvo => Municao {"

# 11) A cerca do RELOGIO desaparece — escrever num campo do editor gasta municao.
bloco "ponte: sem a cerca do relogio" ph2d-app-components \
  com_o_relogio_parado_a_arma_nao_dispara "$PONTE" 1 \
  "    if !playing {
        return out;
    }" \
  ""

# 12) A ordem deixa de ser a da IDENTIDADE — duas maquinas divergem.
bloco "ponte: a ordem nao e' a da identidade" ph2d-app-components \
  a_ordem_dos_disparos_e_a_da_identidade "$PONTE" 1 \
  "    armas.sort_by_key(|a| a.2);" \
  "    armas.sort_by_key(|a| u64::MAX - a.2);"

echo
echo "════ O PAINEL ════"

# 13) Um id sai do `populate` — pintado, hit-registado e MORTO sob o dedo.
bloco "painel: um id fora do populate" ph2d-panel-inspector \
  todo_campo_esta_vivo_sob_o_dedo "$POPULATE" 1 \
  "        ids::INSP_WEAPON_ON_FIRE," \
  ""

# 14) A semente da seccao deixa de correr — o painel mostra os valores de FABRICA.
bloco "painel: sem a semente" ph2d-panel-inspector \
  os_campos_mostram_o_que_o_objecto_tem "$SYNC" 1 \
  "    crate::sync_weapon::sync(host, inspector_state, entity_changed);" \
  ""

# 15) A queixa devolve `None` sem gatilho — o painel cala-se sobre uma arma inerte.
bloco "painel: a queixa cala-se sem gatilho" ph2d-editor-core \
  a_queixa_vai_da_mais_especifica_para_a_mais_geral "$EDITS" 1 \
  "        if self.on_signal.trim().is_empty() {
            return Some(WeaponQueixa::SemGatilho);
        }" \
  ""

# 16) O tecto do painel deixa de ser o da lei.
bloco "painel: o tecto diverge da lei" ph2d-app-components \
  o_tecto_do_painel_e_o_tecto_da_lei "$EDITS" 1 \
  "pub const WEAPON_MAX_MS_UI: f64 = 60_000.0;" \
  "pub const WEAPON_MAX_MS_UI: f64 = 30_000.0;"

echo
echo "════ A CENA ════"

# 17) As tres colunas EMPILHAM — a cena deixa de ensinar.
bloco "cena: as tres colunas empilhadas" ph2d-app-components \
  cada_peca_cabe_na_banda_visivel "$CENA" 1 \
  "const COLUNA_X: [f32; 3] = [-4.0, 0.0, 4.0];" \
  "const COLUNA_X: [f32; 3] = [0.0, 0.0, 0.0];"

# 18) As torretas voltam a `-0,8` — o CONTROLO POSITIVO da cura que a FOTO pediu.
bloco "cena: as torretas cortadas em baixo (a foto)" ph2d-app-components \
  cada_peca_cabe_na_banda_visivel "$CENA" 1 \
  "const TORRETA_Y: f32 = -0.4;" \
  "const TORRETA_Y: f32 = -0.8;"

# 19) O CONTROLO ganha uma arma — a cena perde o lado que a torna legivel.
bloco "cena: o controlo ganha arma" ph2d-app-components \
  o_controlo_nao_tem_arma "$CENA" 1 \
  "            on_signal: GATILHO.to_owned(),
            burst: 1,
            aim_from_spawner: true,
            ..Factory::default()
        },
        bala_ctrl," \
  "            on_signal: TIRO.to_owned(),
            burst: 1,
            aim_from_spawner: true,
            ..Factory::default()
        },
        bala_ctrl,"

# 20) O leque da cacadeira vai a zero — cinco chumbos empilhados.
# ⛔⛔ **A 1.ª corrida SOBREVIVEU, e o defeito era do GATE:** ele comparava o campo da cena com a
#    MESMA const que a cena lê, logo mexer na const movia os dois lados. *Um gate auto-referente
#    afirma que a cena concorda consigo mesma, nunca que ela ensina alguma coisa.* ⇒ ele passa a
#    medir uma PROPRIEDADE derivada (o leque tem de separar dois chumbos por mais de uma largura de
#    bala no fim do voo).
bloco "cena: a cacadeira sem leque" ph2d-app-components \
  a_cacadeira_tem_rajada_e_leque "$CENA" 1 \
  "pub const LEQUE_GRAUS: f32 = 30.0;" \
  "pub const LEQUE_GRAUS: f32 = 0.0;"

# 21) Os dois pentes passam a ter o MESMO nome — as armas partilham municao.
bloco "cena: um pente para as duas" ph2d-app-components \
  os_dois_pentes_sao_contadores_diferentes "$CENA" 1 \
  "pub const CARTUCHAS: &str = \"shells\";" \
  "pub const CARTUCHAS: &str = \"ammo\";"

# 22) O gatilho passa a `Press` — a cadencia deixa de ser observavel.
bloco "cena: o gatilho por toque" ph2d-app-components \
  as_tres_colunas_ouvem_o_mesmo_gatilho "$CENA" 1 \
  "        edge: ActionEdge::Hold," \
  "        edge: ActionEdge::Press,"

echo
echo "════ A SHELL ════"

# 23) A arma corre ANTES do gatilho — cada tiro chega um quadro atrasado.
bloco "shell: a arma antes do gatilho" ph2d-host-desktop \
  a_arma_corre_depois_do_gatilho "$MOTORES" 1 \
  "    gatilhos(sim, signals, relogio, accoes);" \
  ""

# 24) O motor das armas sai do quadro — o componente fica INERTE.
bloco "shell: as armas fora do quadro" ph2d-host-desktop \
  as_armas_disparam_antes_de_a_tabela "$MOTORES" 1 \
  "    armas(sim, signals, &mut leitores.weapon, relogio);" \
  ""

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutações SANGRARAM."
else
  echo "⛔ $FALHAS de $TOTAL não sangraram."
fi
exit "$FALHAS"
