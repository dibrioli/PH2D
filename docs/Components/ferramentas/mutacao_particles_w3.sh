#!/usr/bin/env bash
# Provas de mutação do TOP-20 #18 (`ParticleEmitter`) — W3, a SECÇÃO DO INSPECTOR.
#
# As quatro condições da UI são independentes, e cada bloco mata UMA:
#   o controlo EXISTE · é pintado e REGISTADO · o clique chega ao BARRAMENTO · a sequência
#   LEVA A ALGUM LADO (o dreno escreve no componente certo).
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE.
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer exactamente o número esperado de
#    vezes** — um `sed` que não casa é um no-op silencioso e imprime «SOBREVIVEU» sobre nada.
# ⚠️ **As provas do PRAZO correm com `timeout`**: sem o prazo o laço do teste nunca acaba, e isso É
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

INSP=crates/ph2d-app-components/src/particles_inspector.rs
PONTE=crates/ph2d-app-components/src/particles_bridge.rs
POP=crates/ph2d-panel-inspector/src/populate_particles.rs
EVT=crates/ph2d-panel-inspector/src/event_particles.rs
PINT=crates/ph2d-panel-inspector/src/sections/particles.rs
SEM=crates/ph2d-panel-inspector/src/sync_particles.rs

# ── O DRENO: a edição chega ao campo certo do componente ─────────────────────
bloco "um par trocado no dreno dos numeros" ph2d-app-components "cada_numero_vai_ao_campo" \
  "$INSP" 1 "N::Speed => cfg.speed = v," "N::Speed => cfg.speed_random = v,"
bloco "um par trocado no dreno dos textos" ph2d-app-components "cada_texto_vai_ao_campo" \
  "$INSP" 1 "T::StopOn => cfg.stop_on.clone_from(t)," "T::StopOn => cfg.restart_on.clone_from(t),"
bloco "escrever o MESMO valor passa a «mudou»" ph2d-app-components "escrever_o_mesmo_valor" \
  "$INSP" 1 "    *cfg != antes
}

/// **Aplica as edições de um quadro.**" "    let _ = antes;
    true
}

/// **Aplica as edições de um quadro.**"
bloco "uma forma fora da lista passa a escrever" ph2d-app-components "uma_forma_fora_da_lista" \
  "$INSP" 1 "let Some(&s) = ph2d_ecs::EmissionShape::ALL.get(*i as usize) else {
                return false;
            };" "let s = *ph2d_ecs::EmissionShape::ALL
                .get(*i as usize)
                .unwrap_or(&ph2d_ecs::EmissionShape::Point);"
bloco "o campo inteiro TRUNCA em vez de arredondar" ph2d-app-components "um_campo_inteiro_arredonda" \
  "$INSP" 1 "    v.round() as u32" "    v as u32"
bloco "as vivas de UM objecto viram o TOTAL" ph2d-app-components "o_relogio_e_as_vivas" \
  "$INSP" 1 "        alive," "        alive: alive + 1,"

# ── O REGISTO: um id que não passa pelo `populate` é MORTO sob o dedo ────────
bloco "as amostras de cor saem do populate" ph2d-panel-inspector "uma_amostra_de_cor_abre" \
  "$POP" 1 "    for id in [crate::ids::INSP_PART_COLOR, crate::ids::INSP_PART_COLOR_END] {
        store.register(id, InteractiveState::Plain);
    }" ""
bloco "os chips de forma saem do populate" ph2d-panel-inspector "um_chip_de_forma_chega" \
  "$POP" 1 "    register_button_ids(store, &crate::ids::INSP_PART_SHAPE);" ""

# ── A PINTURA: o que a secção mostra, e o que ela esconde ───────────────────
bloco "as medidas da forma sao pintadas sempre" ph2d-panel-inspector "as_medidas_da_forma_somem" \
  "$PINT" 1 "    if i.shape != 0 {" "    if true {"
bloco "uma linha de numero deixa de ser pintada" ph2d-panel-inspector "a_seccao_inteira_e_pintada" \
  "$PINT" 1 "    for n in [N::Speed, N::SpeedRandom, N::Angle, N::Spread] {" "    for n in [N::Speed, N::SpeedRandom, N::Angle] {"
bloco "os quatro sinais deixam de ser pintados" ph2d-panel-inspector "a_seccao_inteira_e_pintada" \
  "$PINT" 1 "    for (k, _t) in PARTICLES_TEXTS.into_iter().enumerate() {" "    for (k, _t) in PARTICLES_TEXTS.into_iter().enumerate().take(3) {"

# ── O DESPACHO: o clique chega ao barramento com o endereço certo ───────────
bloco "o chip de forma le a tabela do ESPACO" ph2d-panel-inspector "um_chip_de_forma_chega" \
  "$EVT" 1 "let edit = if let Some(i) = linha(&crate::ids::INSP_PART_SHAPE, id) {" "let edit = if let Some(i) = linha(&crate::ids::INSP_PART_SPACE, id) {"
bloco "a amostra deixa de armar o selector" ph2d-panel-inspector "uma_amostra_de_cor_abre" \
  "$EVT" 1 "            host.store_mut().set_picker_target(Some(id));" ""

# ── A SEMENTE: o instantâneo chega ao painel, e a escolha volta ─────────────
bloco "a caixa deixa de ser semeada do instantaneo" ph2d-panel-inspector "a_caixa_manda_o_contrario" \
  "$SEM" 1 "        if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {" "        if false && let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {"
bloco "a cor escolhida so vai ao barramento quando NAO mudou" ph2d-panel-inspector "a_cor_escolhida_chega" \
  "$SEM" 1 "                && escolhida != gravada" "                && escolhida == gravada"

# ── W4: a CENA de smoke — o que ela ensina, e o que a tornaria mentira ───────
CENA=crates/ph2d-app-components/src/particles_smoke.rs

bloco "uma fonte sem CORPO" ph2d-app-components "quem_emite_tem_corpo" \
  "$CENA" 1 "            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], CORPO_RGBA),
            cfg," "            cfg,"
bloco "a rajada nasce LIGADA" ph2d-app-components "a_rajada_nasce_desligada" \
  "$CENA" 1 "            emitting: false," "            emitting: true,"
bloco "o relogio grita OUTRO nome" ph2d-app-components "a_rajada_nasce_desligada" \
  "$CENA" 1 "            signal: BOOM.to_string()," "            signal: \"outro\".to_string(),"
bloco "uma coluna com outra QUANTIDADE" ph2d-app-components "as_quatro_da_um_partilham" \
  "$CENA" 1 "            size_end: 0.0," "            size_end: 0.0,
            amount: 5,"
bloco "a fila volta a ficar fora do ecra" ph2d-app-components "o_que_nasce_cabe_no_ecra" \
  "$CENA" 1 "const VAO: f32 = 2.4;" "const VAO: f32 = 5.0;"
bloco "o anel volta a ter a rapidez das outras" ph2d-app-components "cada_coluna_fica_na_coluna" \
  "$CENA" 1 "            speed: 0.25," "            speed: 3.2,"
bloco "a rajada volta a abrir 180 graus" ph2d-app-components "o_que_nasce_cabe_no_ecra" \
  "$CENA" 1 "            spread: 30.0," "            spread: 180.0,"
bloco "a rajada abre 45 graus e entra na coluna vizinha" ph2d-app-components "cada_coluna_fica_na_coluna" \
  "$CENA" 1 "            spread: 30.0," "            spread: 45.0,"
bloco "as duas da =2 deixam de ser um CONTROLO" ph2d-app-components "na_dois_a_unica_diferenca" \
  "$CENA" 1 "            ParticleEmitter { space, ..base.clone() }," "            ParticleEmitter {
                space,
                life: if space == ParticleSpace::Local { 2.0 } else { 1.0 },
                ..base.clone()
            },"
bloco "a cena abre sem ninguem escolhido" ph2d-app-components "particles_smoke" \
  "$CENA" 1 "            Montada { nivel: 1, escolhido }" "            Montada { nivel: 1, escolhido: 0 }"

echo
echo "── TOTAL: $TOTAL provas · $FALHAS falha(s)"
[ "$FALHAS" = 0 ]
