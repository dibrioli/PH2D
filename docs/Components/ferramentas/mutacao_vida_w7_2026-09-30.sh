#!/bin/bash
# ⭐ PROVA DE MUTAÇÃO da W7 do plano 28 (§15): a ARENA (`PH2D_VIDA_SMOKE=4`), a queixa do dano que
# não fere, e o gate dos rótulos do tutorial 02.
# Corre-se por `bash scripts/ph2d-run.sh bash <este ficheiro>`.
#
# ⚠️ O molde é o da W6 (`mutacao_vida_w6_2026-09-29.sh`): controlo sobre o próprio filtro (uma
# corrida que casa ZERO testes é defeito do ARNÊS, nunca «sobreviveu»), a agulha casa EXACTAMENTE
# uma vez ou o caso ABORTA, e `MUTA_SO_ANCORAS=1` corre só o pré-voo (o `cargo fmt` reescreve
# âncoras). `MUTA_SECCOES="CENA"` filtra.
set -u
cd "$(dirname "$0")/../../.."

SO_ANCORAS=${MUTA_SO_ANCORAS:-0}
SECCOES=${MUTA_SECCOES:-}
ACTIVA=1
seccao() { # CHAVE título
  if [ -z "$SECCOES" ] || [[ " $SECCOES " == *" $1 "* ]]; then ACTIVA=1; echo "== $2 =="; else ACTIVA=0; fi
}
ancoras_ok=0
ancoras_mortas=0
sangram=0
sobrevivem=0
arnes=0

corre() { # rótulo cargo-args...
  # ⛔⛔ **Tudo `local`, e a razão foi uma corrida que deixou a árvore MUTADA** (2026-09-30): o
  # `f` daqui (as reprovadas) era atribuído ao `local f` do `muta` (o FICHEIRO) pelo escopo
  # dinâmico do bash, e o `cp` de reposição escreveu o backup num ficheiro chamado `1` na raiz —
  # as mutações ficaram presas, cumulativas, com o placar a dizer «sangra» por cima.
  local rot=$1 out r n falhas; shift
  [ "$ACTIVA" = 1 ] || return
  [ "$SO_ANCORAS" = 1 ] && return
  out=$(cargo test "$@" 2>&1)
  r=$(echo "$out" | grep -oE "test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed" | head -1)
  n=$(echo "$r" | awk '{print $4+$6}')
  if [ "${n:-0}" -eq 0 ]; then
    echo "  $rot -> ARNES: zero testes (filtro ou compilacao)"
    echo "$out" | grep -E "^error" | head -3
    arnes=$((arnes+1))
    return
  fi
  falhas=$(echo "$r" | awk '{print $6}')
  if [ "$rot" = CONTROLO ]; then
    echo "  $rot -> $r"
  elif [ "${falhas:-0}" -gt 0 ]; then
    echo "  $rot -> SANGRA ($r)"; sangram=$((sangram+1))
  else
    echo "  $rot -> SOBREVIVEU ($r)"; sobrevivem=$((sobrevivem+1))
  fi
}
muta() { # ficheiro agulha substituto rótulo cargo-args...
  local alvo=$1 a=$2 b=$3 rot=$4; shift 4
  [ "$ACTIVA" = 1 ] || return
  if [ "$SO_ANCORAS" = 1 ]; then
    if python3 - "$alvo" "$a" <<'P'
import sys; p,a=sys.argv[1:3]; s=open(p).read(); sys.exit(0 if s.count(a)==1 else 1)
P
    then ancoras_ok=$((ancoras_ok+1)); else ancoras_mortas=$((ancoras_mortas+1)); echo "  $rot -> ANCORA MORTA"; fi
    return
  fi
  cp "$alvo" /tmp/mut_w7.bak
  if ! python3 - "$alvo" "$a" "$b" <<'P'
import sys; p,a,b=sys.argv[1:4]; s=open(p).read()
assert s.count(a)==1, ("agulha", a, s.count(a)); open(p,'w').write(s.replace(a,b))
P
  then echo "  $rot -> ABORTADO (agulha)"; cp /tmp/mut_w7.bak "$alvo"; touch "$alvo"; arnes=$((arnes+1)); return; fi
  corre "$rot" "$@"
  cp /tmp/mut_w7.bak "$alvo"; touch "$alvo"
}

CENA=(-p ph2d-app-components --lib vida_arena)
EC=(-p ph2d-editor-core --lib vida_edits)
TUT=(-p ph2d-panel-inspector --test it o_tutorial_da_vida)
A=crates/ph2d-app-components/src/vida_arena_smoke.rs
R=crates/ph2d-app-components/src/vida_smoke.rs
Q=crates/ph2d-editor-core/src/vida_edits.rs
N=crates/ph2d-i18n/src/inspector_vida.rs
V=crates/ph2d-ecs/src/signal_actions.rs
# ⭐ **O 4.º CONTROLO: a árvore sai como entrou** — as somas dos ficheiros mutados antes e depois.
# Sem ele a reposição partida de 2026-09-30 leu-se como «19 sangram».
SOMAS_ANTES=$(sha256sum $A $R $Q $N $V)

seccao CENA "a ARENA"
corre CONTROLO "${CENA[@]}"
muta $A "                stable_name_id(HEROI)," "                0," "A1 o morcego nao persegue" "${CENA[@]}"
muta $A "                amount: MORDIDA,
                team: MONSTROS.to_owned(),
                on_hit: OnHit::Vanish," "                amount: MORDIDA,
                team: MONSTROS.to_owned(),
                on_hit: OnHit::Stay," "A2 o morcego que morde fica" "${CENA[@]}"
muta $A "                    radius: LADO_DO_MORCEGO / 2.0,
                },
                ..Collider::default()" "                    radius: LADO_DO_MORCEGO / 2.0,
                },
                is_sensor: true,
                ..Collider::default()" "A3 o morcego sem corpo solido" "${CENA[@]}"
muta $A "                resistances: vec![Resistance {
                    kind: CURA_TIPO.to_owned()," "                resistances: vec![Resistance {
                    kind: String::new()," "A4 o heroi nao absorve a cura" "${CENA[@]}"
muta $A "                amount: CURA,
                team: MONSTROS.to_owned()," "                amount: CURA,
                team: String::new()," "A5 o coracao sem equipa" "${CENA[@]}"
muta $A "                amount: CURA,
                team: MONSTROS.to_owned()," "                amount: CURA,
                team: HEROIS.to_owned()," "A6 o coracao da equipa do heroi" "${CENA[@]}"
muta $A "                        rate: 0.0," "                        rate: 1.0," "A7 a Salamandra sente o fogo" "${CENA[@]}"
muta $A "                        rate: 2.0," "                        rate: 1.0," "A8 o gelo sem o dobro" "${CENA[@]}"
muta $A "            amount: 0.0,
            per_second: true," "            amount: 5.0,
            per_second: true," "A9 a lava golpeia" "${CENA[@]}"
muta $A "            over_time_s: LAVA_DEPOIS_S," "            over_time_s: 0.0," "A10 a lava nao queima" "${CENA[@]}"
muta $A '        linha(RECOMECAR, SignalVerb::RestartRun, ""),' "" "A11 sem a linha do recomeco" "${CENA[@]}"
muta $A "                signal: RECOMECAR.to_owned(),
                autostart: false," "                signal: \"outro\".to_owned(),
                autostart: false," "A12 o relogio diz outro nome" "${CENA[@]}"
muta $A "                on_death: CAIU.to_owned()," "                on_death: String::new()," "A13 o heroi morre calado" "${CENA[@]}"
muta $A "                on_death: VENCEU.to_owned()," "                on_death: String::new()," "A14 a vitoria calada" "${CENA[@]}"
muta $R "    if nivel == 3 || nivel == 4 {" "    if nivel == 3 {" "A15 a arena sem as duas armas" "${CENA[@]}"
muta $R "pub const CENAS: u32 = 4;" "pub const CENAS: u32 = 3;" "A16 CENAS nao conta a arena" "${CENA[@]}"
muta $R "    if nivel == 4 {" "    if nivel == 40 {" "A17 o braço 4 do montar" "${CENA[@]}"

seccao QUEIXA "a QUEIXA do dano que nao fere"
corre CONTROLO "${EC[@]}"
muta $Q "        self.amount > 0.0 || (self.dura() && self.over_time_s > 0.0)" "        self.amount > 0.0" \
  "Q1 a queixa esquece o dano que dura" "${EC[@]}"
muta $Q "        self.amount > 0.0 || (self.dura() && self.over_time_s > 0.0)" "        self.amount > 0.0 || self.dura()" \
  "Q2 a queixa esquece a duracao" "${EC[@]}"

seccao TUT "o gate do TUTORIAL 02"
corre CONTROLO "${TUT[@]}"
muta $N '        "panel.inspector.vida.over_time_s" => "Lasts",' '        "panel.inspector.vida.over_time_s" => "Duration",' \
  "U1 o campo Lasts muda de nome" "${TUT[@]}"
muta $V '            SignalVerb::RestartRun => "ecs.signal_verb.restart_run",' '            SignalVerb::RestartRun => "ecs.signal_verb.restart",' \
  "U2 o verbo deixa de usar a chave" "${TUT[@]}"

echo
if [ "$(sha256sum $A $R $Q $N $V)" != "$SOMAS_ANTES" ]; then
  echo "ARNES: a arvore NAO saiu como entrou — reponha antes de confiar no placar"; exit 2
fi
if [ "$SO_ANCORAS" = 1 ]; then
  echo "ancoras: $ancoras_ok ok · $ancoras_mortas mortas"
  [ "$ancoras_mortas" -eq 0 ]
else
  [ -n "$SECCOES" ] && echo "(PARCIAL: só $SECCOES)"
  echo "placar: $sangram sangram · $sobrevivem sobrevivem · $arnes do arnês"
  [ "$sobrevivem" -eq 0 ] && [ "$arnes" -eq 0 ]
fi
