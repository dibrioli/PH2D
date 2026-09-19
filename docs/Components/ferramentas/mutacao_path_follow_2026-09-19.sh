#!/usr/bin/env bash
# Provas de mutação do SUPLENTE #23 — O SEGUIDOR DE CAMINHO.
#
# Arnês IDÊNTICO ao das waves anteriores desta linha — controlo sobre o próprio FILTRO (um filtro
# que casa ZERO testes imprime `ok` e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora
# não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_path_follow_2026-09-19.sh
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

LEI=crates/ph2d-ecs/src/path_follow.rs
PONTE=crates/ph2d-app-components/src/path_follow_bridge.rs
TWEENB=crates/ph2d-app-components/src/tween_bridge.rs
VOCAB=crates/ph2d-editor-core/src/path_follow_edits.rs
EVENTO=crates/ph2d-panel-inspector/src/event_path_follow.rs
POPULATE=crates/ph2d-panel-inspector/src/populate_path_follow.rs
CENA=crates/ph2d-app-components/src/path_follow_smoke.rs
TWEEN=crates/ph2d-tween/src/lib.rs

echo "════ A LEI — o relogio, a volta e o indice ════"

# (1) ⭐⭐⭐ O seguidor passa a ler o PRIMEIRO relogio em vez do dele. Com UM relogio na fixtura isto
#     seria um no-op — e e' por isso que a fixtura do gate tem DOIS, com instantes diferentes.
bloco "lei: o relogio do indice vira o primeiro" ph2d-ecs o_indice_e_que_liga_o_seguidor_ao_timer \
  "$LEI" 1 \
  "        let (Some(timer), Some(estado)) = (cfg.0.get(i), rt.0.get(i)) else {" \
  "        let (Some(timer), Some(estado)) = (cfg.0.first(), rt.0.first()) else {" \
  "--lib"

# (2) ⛔⛔ A volta vira um `rem_euclid` CRU — o defeito que o gate red-first apanhou: `1,0` passa a
#     `0,0`, e quem acaba TELETRANSPORTA-SE para o principio da pista.
bloco "lei: a volta apanha o proprio fim" ph2d-ecs quem_acaba_descansa_no_fim "$LEI" 1 \
  "    Some(if s > 1.0 { s - 1.0 } else { s })" \
  "    Some(s.rem_euclid(1.0))" \
  "--lib"

# (3) O deslocamento e' ignorado: toda a pista comeca no mesmo sitio.
bloco "lei: o deslocamento nao conta" ph2d-ecs o_deslocamento_da_a_volta "$LEI" 1 \
  "    let s = u + f64::from(pf.deslocamento).rem_euclid(1.0);" \
  "    let s = u;" \
  "--lib"

echo "════ A PONTE — a curva, a normal e o angulo ════"

# (4) ⭐⭐ A TANGENTE e' aplicada como PONTO. Sob uma pose com translacao ela vira uma direccao
#     deslocada, e o objecto aponta para um sitio que nao existe.
bloco "ponte: a tangente como ponto" ph2d-app-components a_pose_da_forma_leva_o_seguidor_com_ela \
  "$PONTE" 1 \
  "        let tangente = pista.afim.apply_vec(tangente);" \
  "        let tangente = pista.afim.apply(tangente);" \
  "--lib"

# (5) O LADO passa a deslocar ao longo da curva em vez de perpendicular a ela — deixa de ser uma
#     faixa e passa a ser um avanco.
bloco "ponte: o lado ao longo da curva" ph2d-app-components o_lado_desloca_perpendicularmente \
  "$PONTE" 1 \
  "                [-t[1] * f64::from(p.lado), t[0] * f64::from(p.lado)]," \
  "                [t[0] * f64::from(p.lado), t[1] * f64::from(p.lado)]," \
  "--lib"

# (6) O ANGULO e' lido como RADIANOS, onde o descritor promete GRAUS.
bloco "ponte: o angulo em radianos" ph2d-app-components o_angulo_e_em_graus "$PONTE" 1 \
  "                    .then(|| (t[1].atan2(t[0]) as f32) + p.angulo.to_radians())," \
  "                    .then(|| (t[1].atan2(t[0]) as f32) + p.angulo)," \
  "--lib"

# (7) ⭐⭐ O seguidor anda na curva AUTORADA e nao na COZIDA — ele passa a andar ao lado do que esta'
#     na tela em toda forma com quina viva, offset ou zig-zag.
bloco "ponte: andar na fonte em vez do cozido" ph2d-app-components o_seguidor_anda_na_curva_cozida \
  "$PONTE" 1 \
  "    let cozido = cena.path(id)?.cooked();" \
  "    let cozido = std::borrow::Cow::Borrowed(cena.path(id)?);" \
  "--lib"

# (8) ⛔⛔ A conversao MUNDO → LOCAL desaparece: um seguidor com pai fica deslocado exactamente pela
#     pose do pai, e a fixtura sem pai nao o veria.
bloco "ponte: mundo escrito num local" ph2d-app-components um_seguidor_com_pai_pousa "$TWEENB" 1 \
  "    let Some(novo) = Transform::inverse_compose(pai, alvo) else {
        return false;
    };" \
  "    let novo = alvo;" \
  "--lib"

echo "════ O VOCABULARIO — a escada das queixas ════"

# (9) A escada responde primeiro pelo RELOGIO: quem nem escreveu o nome e' mandado resolver a
#     metade errada.
bloco "vocabulario: a escada fora de ordem" ph2d-editor-core \
  a_escada_responde_do_especifico_para_o_geral "$VOCAB" 1 \
  "        if self.caminho.trim().is_empty() {
            return Some(PathFollowQueixa::SemNome);
        }" \
  "        let Some(_) = self.duracao_us else {
            return Some(PathFollowQueixa::SemRelogio);
        };
        if self.caminho.trim().is_empty() {
            return Some(PathFollowQueixa::SemNome);
        }" \
  "--lib"

echo "════ A COSTURA — o chip, a caixa e o indice do relogio ════"

# (10) ⭐⭐⭐ Todo chip manda a tag `0`: eles ficam VIVOS sob o dedo e todo clique escreve a primeira
#      opcao — o painel le^-se como «o botao nao faz nada» para tres das quatro familias.
bloco "costura: o chip manda sempre a tag 0" ph2d-panel-inspector seam_path_follow "$EVENTO" 1 \
  "            push(host, bits, PathFollowFieldEdit::Ciclo(t));" \
  "            push(host, bits, PathFollowFieldEdit::Ciclo(0));"

# (11) ⭐⭐⭐ A caixa do ALINHAR sai do `populate`: ela fica PINTADA, hit-registada e MORTA sob o
#      dedo — o clique morre no `is_focusable`, sem um unico erro.
bloco "costura: a caixa do alinhar fora do populate" ph2d-panel-inspector seam_path_follow \
  "$POPULATE" 1 \
  "        (ids::INSP_PF_ALINHA, true)," \
  ""

# (12) As caixas do relogio escrevem sempre no timer `0` — com um seguidor no indice 2, elas mexem
#      no relogio de outra coisa.
bloco "costura: o relogio escrito no indice 0" ph2d-panel-inspector \
  as_caixas_do_relogio_escrevem_no_timer "$EVENTO" 1 \
  "                TimerFieldEdit::Repeat(info.relogio, !info.repeat)," \
  "                TimerFieldEdit::Repeat(0, !info.repeat),"

echo "════ A CENA — o nome que os dois lados leem ════"

# (13) A pista nasce com OUTRO nome: a cena abre com tres seguidores parados e o painel a dizer
#      «no object in the scene has that name» sobre uma forma que esta' a' vista.
bloco "cena: a pista com outro nome" ph2d-app-components a_pista_tem_o_nome "$CENA" 1 \
  "            n.0 = PISTA.to_owned();" \
  "            n.0 = \"Outra\".to_owned();" \
  "--lib"

echo "════ A PORTA EXTRAIDA — a ordem da dobra e da curva ════"

# (14) ⭐⭐ O `andamento` aplica a CURVA antes da DOBRA. A extraccao prometia ser byte-identica, e e'
#      este gate do TWEEN — que existia antes dela — que o afirma.
bloco "porta: a curva antes da dobra" ph2d-tween a_dobra_vem_antes_da_curva "$TWEEN" 1 \
  "    Some(easing.eval(ciclo.dobra(u)))" \
  "    Some(ciclo.dobra(easing.eval(u)))" \
  "--lib"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"
