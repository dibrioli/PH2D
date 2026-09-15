#!/usr/bin/env bash
# Provas de mutação do TOP-20 #14 — o PROJÉCTIL (line/components, 2026-09-15).
#
# ⚠️ Elas correm DEPOIS do código: a prova de que os gates não são inertes é ESTA.
#
# ⛔ Um `✗ SOBREVIVEU` é um gate que não afirma o que o doc-comment dele diz. Um `✗ CONTROLO
# inválido` é o FILTRO errado, não o produto.
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

SW=crates/ph2d-sweep/src/lib.rs
BO=crates/ph2d-projectile/src/bounce.rs
PJ=crates/ph2d-projectile/src/lib.rs
BR=crates/ph2d-physics-ecs/src/bridge/projectile.rs
PO=crates/ph2d-physics-ecs/src/bridge/pose_owner.rs
SH=shells/desktop/src/render_loop/inspector_projectile.rs

echo "── A) a folha do ORCAMENTO ───────────────────────────────────────────────"

mutacao "A1 o orcamento deixa de ser |v|·dt" "$SW" \
  'let budget = len(v) * dt;' \
  'let budget = len(v);' \
  ph2d-sweep --lib o_orcamento_do_primeiro_passo

mutacao "A2 o tecto deixa de fechar o plano" "$SW" \
  'if resto < RESTO_MINIMO || step.steps_left == 0 {' \
  'if resto < RESTO_MINIMO {' \
  ph2d-sweep --lib o_tecto_fecha_o_plano

mutacao "A3 o espelho vira uma projeccao" "$SW" \
  'let d = 2.0 * dot(v, n);' \
  'let d = dot(v, n);' \
  ph2d-sweep --lib espelho

echo "── B) a lei do RICOCHETE, contra o corpus do oraculo ─────────────────────"

# ⛔ O defeito que a wave existe para nao ter: o resto do orcamento EVAPORA no toque.
mutacao "B1 o orcamento nao atravessa o salto" "$BO" \
  'let budget = resto * k;' \
  'let budget = resto * k * 0.5;' \
  ph2d-projectile --lib o_orcamento_atravessa

# ⚠️ A perda por salto tem de tocar nas DUAS grandezas com o MESMO numero.
mutacao "B2 a perda so' toca na velocidade" "$BO" \
  'let budget = resto * k;' \
  'let budget = resto;' \
  ph2d-projectile --lib a_perda_por_salto

# ⚠️ E sair da parede nao e' bater nela — sem esta guarda o corpo vai PARA DENTRO do solido.
mutacao "B3 ricocheteia numa superficie de que se afasta" "$BO" \
  'if dot(step.dir, n) >= 0.0 {
        return None;
    }' \
  'if false {
        return None;
    }' \
  ph2d-projectile --lib nao_se_ricocheteia

mutacao "B4 a bounciness deixa de ser presa na porta" "$BO" \
  'let k = law.bounciness.clamp(0.0, 1.0);' \
  'let k = law.bounciness;' \
  ph2d-projectile --lib uma_bounciness_fora_da_faixa

echo "── C) a lei do VOO ──────────────────────────────────────────────────────"

# ⚠️ O nascimento tem de acontecer UMA vez — senao rodar o corpo relanca a bala.
mutacao "C1 a bala e' relancada a cada tique" "$PJ" \
  'if !state.launched {' \
  'if true {' \
  ph2d-projectile --lib o_nascimento_acontece_uma_vez

# ⛔ range=0 e' SEM LIMITE; lido como limite, toda bala morre ao nascer.
mutacao "C2 alcance zero passa a matar" "$PJ" \
  'if law.range > 0.0 && state.travelled >= law.range {' \
  'if state.travelled >= law.range {' \
  ph2d-projectile --lib alcance_zero

# ⛔ max_speed=0 e' SEM TECTO.
mutacao "C3 tecto zero passa a parar a bala" "$PJ" \
  'if law.max_speed > 0.0 {' \
  'if true {' \
  ph2d-projectile --lib max_speed_zero

# ⚠️ Sem velocidade nao ha' direccao de VOO — inventar uma faz a bala arrancar sozinha.
mutacao "C4 a aceleracao inventa uma direccao" "$PJ" \
  'if law.acceleration != 0.0
        && let Some(d) = normalize(state.velocity)' \
  'if law.acceleration != 0.0
        && let Some(d) = normalize(state.velocity).or(Some([1.0, 0.0]))' \
  ph2d-projectile --lib sem_velocidade_a_aceleracao

echo "── D) a PONTE contra a rapier ───────────────────────────────────────────"

# ⛔⛔ O defeito que o handoff do #13 previu e que esta wave pagou a' mesma.
mutacao "D1 o pose_owner esquece o projectil" "$PO" \
  '|| world
            .get::<crate::components::ProjectileMotion>(entity)
            .is_some();' \
  '|| false;' \
  ph2d-physics-ecs --lib bridge::projectile

# ⚠️ O alcance conta o que ANDOU, nao o que foi pedido.
mutacao "D2 o alcance passa a contar o pedido" "$BR" \
  'st.travelled += ph2d_projectile::len(andado);' \
  'st.travelled += ph2d_projectile::len(velocidade) * dt;' \
  ph2d-physics-ecs --lib uma_bala_barrada

# ⚠️ A memoria tem de atravessar os tiques.
mutacao "D3 a memoria do voo nasce fresca a cada tique" "$BR" \
  'let mut st: ProjectileState = self
                .projectile_state
                .get(&entity)' \
  'let mut st: ProjectileState = ProjectileState::default();
            let _unused = self
                .projectile_state
                .get(&entity)' \
  ph2d-physics-ecs --lib a_memoria_do_voo

echo "── E) o PAINEL ──────────────────────────────────────────────────────────"

# ⚠️ As cercas do painel valem TAMBEM no dreno — o campo e' alcancavel por outra rota.
mutacao "E1 a cerca da bounciness sai do dreno" "$SH" \
  'ProjectileFieldEdit::Bounciness(v) => law.bounciness = v.clamp(0.0, 1.0),' \
  'ProjectileFieldEdit::Bounciness(v) => law.bounciness = *v,' \
  ph2d-host-desktop --bins as_cercas_do_painel

# ⚠️ «o nome que ninguem tem» e' uma resposta PROPRIA, nao um vazio.
mutacao "E2 um alvo apagado le-se como sem alvo" "$SH" \
  '    (String::new(), true)
}' \
  '    (String::new(), false)
}' \
  ph2d-host-desktop --bins um_alvo_apagado

# ⚠️ Vazio e' ZERO, e nao o hash de uma string vazia.
mutacao "E3 limpar o alvo passa a escrever um hash" "$SH" \
  'Some(if t.is_empty() { 0 } else { stable_name_id(t) })' \
  'Some(stable_name_id(t))' \
  ph2d-host-desktop --bins limpar_o_alvo

echo "── $N mutações ──────────────────────────────────────────────────────────"
