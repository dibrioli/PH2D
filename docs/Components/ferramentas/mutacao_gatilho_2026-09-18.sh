#!/usr/bin/env bash
# Provas de mutação do SUPLENTE #24 — O GATILHO (a mão do artista vira um sinal).
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE, e isso
#      lê-se exactamente como «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes** —
#      uma mutação que não entra lê-se exactamente como uma que sobreviveu.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_gatilho_2026-09-18.sh
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

LEI=crates/ph2d-ecs/src/signal_on_action.rs
FAB=crates/ph2d-ecs/src/factory.rs
PONTE=crates/ph2d-app-components/src/factory_bridge.rs
SHELL=shells/desktop/src/render_loop/motores_do_quadro.rs
INSP=crates/ph2d-app-components/src/action_trigger_inspector.rs
CENA=crates/ph2d-app-components/src/trigger_smoke.rs
PONTE_T=crates/ph2d-app-components/src/trigger_bridge.rs
SECCAO_T=crates/ph2d-panel-inspector/src/sections/action_trigger.rs
PROLOGO=shells/desktop/src/components_scenes.rs

echo "════ W1 — a LEI (ph2d-ecs) ════"

# (1) `Press` é a BORDA. Com `pressed` ele vira um `Hold` e uma pistola passa a metralhadora.
bloco "Press: a borda -> segurado" ph2d-ecs press_fala_uma_vez_por_toque "$LEI" 1 \
  "ActionEdge::Press => a.just_pressed," "ActionEdge::Press => a.pressed,"

# (2) `Hold` fala em TODO quadro. Com a borda ele vira um `Press` e o lança-chamas dá um tiro só.
bloco "Hold: segurado -> a borda" ph2d-ecs press_fala_uma_vez_por_toque "$LEI" 1 \
  "ActionEdge::Hold => a.pressed," "ActionEdge::Hold => a.just_pressed,"

# (3) `Release` é a OUTRA borda — trocá-la pela primeira faz a linha falar ao contrário.
bloco "Release: largar -> premir" ph2d-ecs press_fala_uma_vez_por_toque "$LEI" 1 \
  "ActionEdge::Release => a.just_released," "ActionEdge::Release => a.just_pressed,"

# (4) O nome de sinal VAZIO cala a linha — sem isso um gatilho a meio publica um sinal sem nome.
bloco "sinal vazio: calado -> fala" ph2d-ecs uma_linha_sem_nome_de_sinal_fica_calada "$LEI" 1 \
  "if row.signal.trim().is_empty() {" "if false {"

# (5) A ordem é a da IDENTIDADE — sem ela é a iteração do mundo, indefinida entre arquétipos.
bloco "ordem: identidade -> iteracao" ph2d-ecs os_disparos_saem_pela_ordem "$LEI" 1 \
  "map_or(u64::MAX, |s| s.0)" "map_or(u64::MAX, |_| 0)"

# (6) Cada linha lê a PRÓPRIA acção — ler outro campo põe todas a ouvir a mesma tecla.
bloco "cada linha le a sua accao" ph2d-ecs cada_linha_le_a_propria_accao "$LEI" 1 \
  "fala(row, lida(&row.action))" "fala(row, lida(&row.signal))"

# (7) ⭐ A AUSÊNCIA de estado é a lei desta wave, e o censo que a declara tem de morrer se alguém
#     escrever um `*Runtime` aqui.
#     ⚠️ **A âncora é a linha DEPOIS da struct e não a struct:** inserir antes dela rouba-lhe o
#     `#[derive(Component, …)]`, e a mutação deixa de COMPILAR — que se lê como filtro vazio, não
#     como sobrevivência.
bloco "o estado que nao pode existir" ph2d-ecs o_gatilho_nao_guarda_estado "$LEI" 1 \
  "impl SimComponent for SignalOnAction {}" \
  "impl SimComponent for SignalOnAction {}
#[derive(Copy, Clone, Debug, Default)]
pub struct SignalOnActionRuntime;"

echo
echo "════ W2 — a MIRA (a cópia sai apontada) ════"

# (8) Sem a escrita, a cópia fica com a rotação do MOLDE — o buraco que a wave abriu para fechar.
bloco "a mira nao chega a' copia" ph2d-app-components a_copia_sai_apontada "$PONTE" 1 \
  "if let Some(aim) = pedido.aim {" "if let Some(aim) = pedido.aim.filter(|_| false) {"

# (9) ⭐ E o CONTROLO: escrever SEMPRE partiria toda fábrica que já existe.
bloco "a mira escreve sempre" ph2d-app-components a_copia_sai_apontada "$FAB" 1 \
  "                .aim_from_spawner" "                .aim_from_spawner
                .then_some(true)
                .unwrap_or(true)"

echo
echo "════ W3 — a SECÇÃO (o Inspector) ════"

# (10) A tradução da aresta fecha nos DOIS sentidos.
bloco "aresta: ida-e-volta" ph2d-app-components a_traducao_da_aresta_fecha "$INSP" 1 \
  "2 => ActionEdge::Hold," "2 => ActionEdge::Release,"

# (11) Um nome VAZIO não é um órfão (é uma linha por preencher); um desconhecido é.
#      ⚠️ **A âncora mudou de FORMA em 2026-09-18** (o booleano virou três estados) e o arnês
#      ABORTOU em vez de mutar nada — que é exactamente para o que a contagem existe.
bloco "orfao: o vazio conta" ph2d-app-components um_nome_vazio_nao_conta "$INSP" 1 \
  "            no_mapa: if r.action.trim().is_empty() {
                NoMapa::Ligada" \
  "            no_mapa: if r.action.trim().is_empty() {
                NoMapa::Desconhecida"

# (12) Reescrever o MESMO valor não suja o mundo — senão todo quadro de painel vira passo de undo.
bloco "escrita: sem guarda de igualdade" ph2d-app-components reescrever_o_mesmo_valor "$INSP" 1 \
  "            if row.signal == *s {" "            if false {"

# (13) O tecto do modelo é honrado pela PONTE — sem ele o painel aceita o que não mostra.
bloco "tecto: a ponte deixa passar" ph2d-app-components o_tecto_do_modelo_e_honrado "$INSP" 1 \
  "if cfg.0.len() >= ACTION_TRIGGERS_MAX {" "if false {"

echo
echo "════ W4 — a CENA (o smoke do dono) ════"

# (14) As duas armas diferem SÓ na mira — igualá-las apaga o controlo.
bloco "cena: as duas com mira" ph2d-app-components as_duas_armas_diferem_so_na_mira "$CENA" 1 \
  "arma(world, torreta, bala_ctrl, false);" "arma(world, torreta, bala_ctrl, true);"

# (15) A torreta está RODADA — no neutro, «herda a rotação da fábrica» e «fica com a do molde» dão
#      a MESMA imagem, e o controlo não controla nada.
bloco "cena: a torreta no neutro" ph2d-app-components a_torreta_esta_rodada "$CENA" 1 \
  "pose.rotation = std::f32::consts::FRAC_PI_2;" "pose.rotation = 0.0;"

# (16) O CHÃO nasce primeiro (a ordem das raízes é a de CRIAÇÃO desde a cura de 15/09).
bloco "cena: o chao nasce tarde" ph2d-app-components o_chao_nasce_antes_de_tudo "$CENA" 1 \
  "        Name::new(\"Ground\")," "        Name::new(\"Chao tardio\"),"

echo
echo "════ W5 — a FIAÇÃO (a shell) ════"

# (17) ⭐⭐ A CERCA DO RELÓGIO — sem ela, cada tecla escrita num campo publica um sinal.
#      ⚠️ **Mudou de crate em 2026-09-18**, com a lei: ela é do COMPONENTE (o molde da irmã das
#      vigias), e o que ficou na shell é a chamada.
bloco "relogio: a cerca sai" ph2d-app-components com_o_relogio_parado "$PONTE_T" 1 \
  "    if !playing {
        return Vec::new();
    }" "    if false {
        return Vec::new();
    }"

# (18) `just_pressed` e `pressed` só se lêem iguais no INSTANTE em que se carrega — a paragem
#      «segurar» do percurso é a que os separa.
#      ⚠️ **Mudou de crate em 2026-09-18:** a varredura é PURA e saiu da shell para a família
#      (`trigger_bridge`), pela catraca `the_shell_only_shrinks` — e as duas respostas coincidem.
bloco "amostra: just_pressed -> pressed" ph2d-app-components as_amostras_saem_do_mapa "$PONTE_T" 1 \
  "just_pressed: input.just_pressed(&a.name)," "just_pressed: input.pressed(&a.name),"

# (19) ⭐ `just_released` e `!pressed` só se lêem iguais no tique em que se larga — a paragem
#      «ficar solto» é a que os separa, e é o defeito mudo que o doc do produto já nomeia.
bloco "amostra: just_released -> !pressed" ph2d-app-components as_amostras_saem_do_mapa "$PONTE_T" 1 \
  "just_released: input.just_released(&a.name)," "just_released: !input.pressed(&a.name),"

echo
echo "════ W7 — a TECLA (report do dono) ════"

# (20) ⭐⭐ O ESPAÇO de volta: ele É o Play/Pause do transporte, e o gate tem de o dizer.
bloco "tecla: volta ao ESPACO" ph2d-host-desktop a_tecla_do_gatilho_nao_e_reclamada "$CENA" 1 \
  "pub const TECLA: u32 = 0x51;" "pub const TECLA: u32 = 0x20;" "--test it"

# (21) ⭐ E o `P`, que é o menu radial do canvas E o Probe do grafo — a outra metade da régua.
bloco "tecla: o P do menu radial" ph2d-host-desktop a_tecla_do_gatilho_nao_e_reclamada "$CENA" 1 \
  "pub const TECLA: u32 = 0x51;" "pub const TECLA: u32 = 0x50;" "--test it"

echo
echo "════ W8 — o SEGUNDO silêncio (a acção sem tecla) ════"

# (22) ⭐⭐ Sem a distinção, uma acção POR LIGAR lê-se como ligada — e o painel diz que está tudo bem.
bloco "sem tecla: lida como ligada" ph2d-app-components uma_accao_sem_tecla_existe "$PONTE_T" 1 \
  "if a.bindings.is_empty() {" "if false {"

# (23) ⭐ E a DESCONHECIDA não pode ser engolida pelo estado do meio (a cura de cada uma é outra).
bloco "desconhecida: lida como sem tecla" ph2d-app-components uma_accao_sem_tecla_existe "$PONTE_T" 1 \
  ".map_or(NoMapa::Desconhecida, |a| {" ".map_or(NoMapa::SemTecla, |a| {"

# (24) ⭐⭐⭐ E o aviso tem de chegar a PIXEL — um braço que não pinte deixa os outros dois verdes.
bloco "aviso: o braco nao pinta" ph2d-panel-inspector o_aviso_da_accao_sem_tecla "$SECCAO_T" 1 \
  "    } else if row.no_mapa == NoMapa::SemTecla {" "    } else if false {"

# (25) ⭐ E a cena tem de CRIAR a acção sem tecla — nenhuma de fábrica serve de exemplo.
bloco "cena: sem a accao por ligar" ph2d-host-desktop o_prologo_deixa_uma_accao "$PROLOGO" 1 \
  "                .create(ph2d_app_components::trigger_smoke::ACCAO_SEM_TECLA);" \
  "                .create(\"outra\");" "--test it"

echo
echo "════ $((TOTAL-FALHAS))/$TOTAL sangraram ════"
[ "$FALHAS" = 0 ] || exit 1
