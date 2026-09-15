#!/usr/bin/env bash
# Provas de mutação da W1 da FÁBRICA e do CICLO DE VIDA (line/components, 2026-09-14) —
# a porta em LOTE, as duas leis de morte e a lei da fábrica.
#
# ⚠️ Elas correm DEPOIS do código: a prova de que os gates não são inertes é ESTA, e por isso ela
# cobre as asserções que CARREGAM a lei — não uma amostra.
#
# ⛔ Um `✗ SOBREVIVEU` é um gate que não afirma o que o doc-comment dele diz. Um `✗ CONTROLO
# inválido` é o FILTRO errado, não o produto.
#
# Para cada mutação: (1) CONTROLO — o filtro corre >= 1 teste e passa na árvore limpa; (2) muta, com
# a âncora a ocorrer EXACTAMENTE uma vez; (3) corre; (4) restaura do backup e dá `touch`.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
export LC_ALL=C
# ⛔ **`S` é a pasta dos LOGS.** Um alias de ficheiro chamado `S` no corpo faz o arnês escrever
# `…/tags.rs/1-controlo.log` e TODOS os controlos saem «inválidos» de uma vez — falha alto, mas
# o sintoma não aponta para a causa. Os aliases do corpo usam nomes de DUAS letras.
S=${PH2D_MUT_DIR:-$(mktemp -d)}
mkdir -p "$S/bak"
echo "logs em $S"
declare -a BACKED=()

restore_all() {
  for f in "${BACKED[@]:-}"; do
    [ -n "$f" ] || continue
    local b="$S/bak/$(echo "$f" | tr '/' '_')"
    [ -f "$b" ] && cp "$b" "$f" && touch "$f"
  done
}
trap restore_all EXIT INT TERM

backup() {
  local f=$1 b="$S/bak/$(echo "$1" | tr '/' '_')"
  if [ ! -f "$b" ]; then cp "$f" "$b"; BACKED+=("$f"); fi
}

replace() {
  python3 - "$1" "$2" "$3" <<'PY'
import sys
p, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p, encoding="utf-8").read()
n = s.count(old)
if n != 1:
    print(f"ANCORA {n}x em {p}: {old!r}")
    sys.exit(3)
open(p, "w", encoding="utf-8").write(s.replace(old, new))
PY
}

corre() { # crate alvo filtro log -> "passed failed" ou "ERRO"
  local crate=$1 target=$2 filter=$3 log=$4
  cargo test -q -p "$crate" $target -- "$filter" >"$log" 2>&1
  local p f
  p=$(grep -oE '[0-9]+ passed' "$log" | awk '{s+=$1} END {print s+0}')
  f=$(grep -oE '[0-9]+ failed' "$log" | awk '{s+=$1} END {print s+0}')
  if ! grep -q 'test result' "$log"; then echo "ERRO"; else echo "$p $f"; fi
}

N=0
mutacao() { # nome ficheiro velho novo crate alvo filtro
  local nome=$1 file=$2 old=$3 new=$4 crate=$5 target=$6 filter=$7
  N=$((N+1))
  local ctl mut
  ctl=$(corre "$crate" "$target" "$filter" "$S/$N-controlo.log")
  if [ "$ctl" = "ERRO" ] || [ "${ctl%% *}" -lt 1 ] || [ "${ctl##* }" -ne 0 ]; then
    echo "✗ $nome — CONTROLO inválido ($ctl) · filtro «$filter»"; return
  fi
  backup "$file"
  if ! replace "$file" "$old" "$new"; then
    echo "✗ $nome — âncora não casou"; return
  fi
  mut=$(corre "$crate" "$target" "$filter" "$S/$N-mutante.log")
  cp "$S/bak/$(echo "$file" | tr '/' '_')" "$file" && touch "$file"
  if [ "$mut" = "ERRO" ]; then
    echo "? $nome — o mutante NÃO COMPILOU (ver $S/$N-mutante.log)"
  elif [ "${mut##* }" -ge 1 ]; then
    echo "✓ $nome — SANGROU (controlo $ctl · mutante $mut)"
  else
    echo "✗ $nome — SOBREVIVEU (controlo $ctl · mutante $mut)"
  fi
}


# ⚠️ **Algumas leis têm DOIS guardas e nenhum é observável sozinho** — o outro tapa o buraco do
# primeiro, e uma mutação de uma agulha só devolve «SOBREVIVEU» sobre uma lei que está certa. Esta
# variante muta os dois de uma vez, que é a redacção que de facto apaga a lei.
mutacao2() { # nome f1 old1 new1 f2 old2 new2 crate alvo filtro
  local nome=$1 f1=$2 o1=$3 n1=$4 f2=$5 o2=$6 n2=$7 crate=$8 target=$9 filter=${10}
  N=$((N+1))
  local ctl mut
  ctl=$(corre "$crate" "$target" "$filter" "$S/$N-controlo.log")
  if [ "$ctl" = "ERRO" ] || [ "${ctl%% *}" -lt 1 ] || [ "${ctl##* }" -ne 0 ]; then
    echo "✗ $nome — CONTROLO inválido ($ctl) · filtro «$filter»"; return
  fi
  backup "$f1"; backup "$f2"
  if ! replace "$f1" "$o1" "$n1" || ! replace "$f2" "$o2" "$n2"; then
    echo "✗ $nome — âncora não casou"
    cp "$S/bak/$(echo "$f1" | tr '/' '_')" "$f1" && touch "$f1"
    cp "$S/bak/$(echo "$f2" | tr '/' '_')" "$f2" && touch "$f2"
    return
  fi
  mut=$(corre "$crate" "$target" "$filter" "$S/$N-mutante.log")
  cp "$S/bak/$(echo "$f1" | tr '/' '_')" "$f1" && touch "$f1"
  cp "$S/bak/$(echo "$f2" | tr '/' '_')" "$f2" && touch "$f2"
  if [ "$mut" = "ERRO" ]; then
    echo "? $nome — o mutante NÃO COMPILOU (ver $S/$N-mutante.log)"
  elif [ "${mut##* }" -ge 1 ]; then
    echo "✓ $nome — SANGROU (controlo $ctl · mutante $mut)"
  else
    echo "✗ $nome — SOBREVIVEU (controlo $ctl · mutante $mut)"
  fi
}

IN=crates/ph2d-ecs/src/instantiate.rs
LF=crates/ph2d-ecs/src/lifetime.rs
FC=crates/ph2d-ecs/src/factory.rs
SL=crates/ph2d-topdown/src/slide.rs
DR=crates/ph2d-topdown/src/direction.rs
VP=crates/ph2d-topdown/src/viewpoint.rs
IT=crates/ph2d-topdown/src/intent.rs
RT=crates/ph2d-topdown/src/rotation.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

# ═══ M1 — a lei que a casa TEM hoje, posta no lugar da que a wave traz ═══════
# ⭐⭐⭐ É a prova RED-FIRST desta wave, e o mutante não é um absurdo: ele é
# EXACTAMENTE o `move_character` de hoje (medido: razão |d|/tangencial = 1,000 em
# todos os ângulos). Se o corpus do oráculo não sangrar com ele, o corpus não
# está a medir a única coisa que esta crate existe para trazer.
mutacao "M1 o orcamento vira PROJECCAO (a lei de hoje)" "$SL" \
  'budget: resto,' \
  'budget: resto * (1.0 - cos_incidencia * cos_incidencia).sqrt(),' \
  ph2d-topdown --test=it 'o_corpus'

# ═══ M2 — o limiar deixa de cortar ══════════════════════════════════════════
mutacao "M2 o limiar de incidencia nao corta" "$SL" \
  'if cos_incidencia >= cos_limiar {
        return None;
    }' \
  'if cos_incidencia > 2.0 {
        return None;
    }' \
  ph2d-topdown --test=it 'o_corpus'

# ═══ M3 — o tecto de deslizes nao desce ═════════════════════════════════════
mutacao "M3 o tecto de deslizes nao desce" "$SL" \
  'slides_left: step.slides_left - 1,' \
  'slides_left: step.slides_left,' \
  ph2d-topdown --lib 'o_tecto_desce'

# ═══ M4 — a tangente deixa de tirar a componente normal ═════════════════════
mutacao "M4 a tangente nao tira a componente normal" "$SL" \
  'let t = [
        step.dir[0] - n[0] * dot(step.dir, n),
        step.dir[1] - n[1] * dot(step.dir, n),
    ];' \
  'let t = [step.dir[0], step.dir[1]];' \
  ph2d-topdown --lib 'a_tangente_conserva'

# ═══ M5 — a diagonal volta a ser mais rapida ════════════════════════════════
mutacao "M5 o comprimento deixa de ser cortado a 1" "$DR" \
  'let comprimento = len(bruto).min(1.0);' \
  'let comprimento = len(bruto);' \
  ph2d-topdown --lib 'a_diagonal_nao_e_mais_rapida'

# ═══ M6 — a quantizacao corre no espaco do ECRA (a ORDEM invertida) ═════════
# ⚠️ A âncora é a porta única da ordem, no `lib.rs`.
mutacao "M6 a ordem inverte-se (quantizar DEPOIS de reprojectar)" crates/ph2d-topdown/src/lib.rs \
  '    let quantizado = direction::quantize(raw, law.direction);
    viewpoint::reproject(quantizado, law.viewpoint, law.viewpoint_angle_deg)' \
  '    let reprojectado = viewpoint::reproject(raw, law.viewpoint, law.viewpoint_angle_deg);
    direction::quantize(reprojectado, law.direction)' \
  ph2d-topdown --lib 'a_quantizacao_corre_no_espaco'

# ═══ M7 — a identidade do viewpoint deixa de ser ao bit ════════════════════
mutacao "M7 a vista de cima passa a ser um caso do geral" "$VP" \
  '    if matches!(view, Viewpoint::TopDown) {' \
  '    if matches!(view, Viewpoint::Custom) && false {' \
  ph2d-topdown --lib 'identidade_AO_BIT'

# ═══ M8 — zero de rampa passa a ser PARADO ═════════════════════════════════
mutacao "M8 zero de rampa deixa de ser instantaneo" "$IT" \
  'if !(taxa.is_finite() && taxa > 0.0) {
        return ate;
    }' \
  'if !(taxa.is_finite() && taxa > 0.0) {
        return de;
    }' \
  ph2d-topdown --lib 'zero_de_rampa'

# ═══ M9 — a rotacao vira pelo lado LONGO ═══════════════════════════════════
mutacao "M9 a rotacao deixa de dobrar o arco" "$RT" \
  'let delta = arco_curto(alvo - atual);' \
  'let delta = alvo - atual;' \
  ph2d-topdown --lib 'lado_CURTO'

# ═══ M10 — parado, ele volta a apontar para leste ══════════════════════════
mutacao "M10 sem intencao ele salta para o angulo zero" "$RT" \
  'let Some(dir) = normalize(dir_mundo) else {
        return atual;
    };' \
  'let dir = normalize(dir_mundo).unwrap_or([1.0, 0.0]);' \
  ph2d-topdown --lib 'parado_ele_fica'

echo
echo "$N mutacoes · load $(cut -d' ' -f1 /proc/loadavg)"
