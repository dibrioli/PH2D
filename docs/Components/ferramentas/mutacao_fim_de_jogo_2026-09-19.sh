#!/usr/bin/env bash
# Provas de mutação do FIM DE JOGO (SignalVerb::RestartRun).
#
# Arnês IDÊNTICO ao das waves anteriores desta linha — controlo sobre o próprio FILTRO (um filtro
# que casa ZERO testes imprime `ok` e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora
# não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_fim_de_jogo_2026-09-19.sh
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
PONTE=crates/ph2d-app-components/src/signal_actions_bridge.rs
LEDGER=crates/ph2d-preview-drive/src/lib.rs
CENA=crates/ph2d-app-components/src/restart_smoke.rs
FASE=shells/desktop/src/render_loop/fase_fabrica_e_morte.rs

echo "════ A LEI (ph2d-ecs) ════"

# 1) O verbo novo entra no MEIO do `ALL` — todo ficheiro gravado muda de verbo, em silêncio.
bloco "lei: o verbo novo no meio da lista" ph2d-ecs \
  a_tag_de_um_verbo_ja_gravado "$LEI" 1 \
  "        SignalVerb::Destroy,
        SignalVerb::RestartRun,
    ];" \
  "        SignalVerb::RestartRun,
        SignalVerb::Destroy,
    ];"

# 2) O verbo passa a ter ALVO — o painel pinta uma escolha que o consumidor deita fora.
bloco "lei: o recomeco passa a ter alvo" ph2d-ecs \
  a_tag_de_um_verbo_ja_gravado "$LEI" 1 \
  "        !matches!(self, SignalVerb::RestartRun)" \
  "        true"

# 3) E o CONTROLO: `uses_target` devolve `false` a toda a gente.
bloco "lei: nenhum verbo tem alvo" ph2d-ecs \
  a_tag_de_um_verbo_ja_gravado "$LEI" 1 \
  "        !matches!(self, SignalVerb::RestartRun)" \
  "        false"

echo
echo "════ A PONTE (ph2d-app-components) ════"

# 4) O verbo deixa de ANUNCIAR — o pedido nunca chega ao dreno.
bloco "ponte: o verbo nao anuncia" ph2d-app-components \
  o_verbo_do_recomeco_anuncia "$PONTE" 1 \
  "                report.recomecar = true;" \
  "                report.recomecar = false;"

# 5) O pedido nasce LIGADO — toda a tabela do app recomecaria a corrida a cada sinal.
#    ⚠️ A 1.ª redacção desta mutação fechava a struct e abria um `impl` — ela COMPILAVA e nao
#    mudava o `Default`, logo lia-se como «SOBREVIVEU» sobre produto correcto. *Uma mutacao que nao
#    muda o produto e uma que sobrevive dao o mesmo relatorio.*
bloco "ponte: o pedido nasce ligado" ph2d-app-components \
  os_outros_verbos_nao_pedem "$PONTE" 1 \
  "    let mut report = ActionReport::default();" \
  "    let mut report = ActionReport { recomecar: true, ..Default::default() };"

echo
echo "════ A CENA (ph2d-app-components) ════"

# 8) As luzes perdem a `Visibility` — o `Hide` fica INERTE e nenhuma luz se apaga.
bloco "cena: as luzes sem Visibility" ph2d-app-components \
  o_jogo_joga_se_recomeca_e_as_luzes_voltam "$CENA" 1 \
  "            Visibility::default()," \
  ""

# 9) A BATIDA desaparece — a ultima luz apaga e reacende no MESMO quadro.
bloco "cena: sem a batida" ph2d-app-components \
  a_corrente_dos_sinais_casa "$CENA" 1 \
  "                linha(MORRI, \"\", SignalVerb::StartTimer, RELOGIO)," \
  ""

# 10) O heroi perde o corpo — as setas nao fazem nada (a licao de 19/09).
bloco "cena: o heroi sem corpo" ph2d-app-components \
  o_heroi_anda_e_nao_cai "$CENA" 1 \
  "            RigidBody {
                kind: BodyKind::Kinematic,
            }," \
  ""

# 14) As tres linhas que ACENDEM as luzes desaparecem — e' o report do dono a' letra: o recomeco
#     corre e fica INVISIVEL.
bloco "cena: as luzes nao voltam a acender" ph2d-app-components \
  o_jogo_joga_se_recomeca_e_as_luzes_voltam "$CENA" 1 \
  "                linha(RECOMECA, &format!(\"{LUZ}1\"), SignalVerb::Show, \"\")," \
  ""

# 15) E o CONTROLO de meio caminho: sem o `Hide` as luzes nunca se apagam, e o gate tem de dizer
#     que a cena nao chegou a perder — nao que o recomeco funcionou.
bloco "cena: as luzes nunca se apagam" ph2d-app-components \
  o_jogo_joga_se_recomeca_e_as_luzes_voltam "$CENA" 1 \
  "                linha(\"luz1\", &format!(\"{LUZ}1\"), SignalVerb::Hide, \"\")," \
  ""

echo
echo "════ A FASE (shells/desktop) ════"

# 11) A CERCA contra o laco desaparece — um recomeco no primeiro tique passa, e o app congela.
bloco "fase: sem a cerca do laco" ph2d-host-desktop \
  a_cerca_recusa_o_laco "$FASE" 1 \
  "    vida > passo_fixo" \
  "    { let _ = (vida, passo_fixo); true }" \
  "--bins"

# 12) A cerca fica COLADA a um numero — ela deixa de ser derivada do passo.
#     ⚠️ E' a metade que separa «um numero medido» de «um numero escolhido».
bloco "fase: a cerca cravada num numero" ph2d-host-desktop \
  a_cerca_le_o_passo "$FASE" 1 \
  "    vida > passo_fixo" \
  "    { let _ = passo_fixo; vida > 1.0 / 60.0 }"

# 13) A fronteira deixa de ser ESTRITA — o primeiro tique passa a contar como corrida.
bloco "fase: a fronteira deixa de ser estrita" ph2d-host-desktop \
  a_cerca_recusa_o_laco "$FASE" 1 \
  "    vida > passo_fixo" \
  "    vida >= passo_fixo" \
  "--bins"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutações SANGRARAM."
else
  echo "⛔ $FALHAS de $TOTAL não sangraram."
fi
exit "$FALHAS"
