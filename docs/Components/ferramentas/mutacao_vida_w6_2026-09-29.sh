#!/bin/bash
# ⭐ PROVA DE MUTAÇÃO da W6 do plano 28 (§14): os TIPOS de dano — a lei da taxa e das aflições, a
# ponte (a aflição entra com o golpe, os pulsos, a morte), a chave do tipo, o Inspector (instantâneo
# e dreno), o painel (a lista, os botões, o editor da linha aberta) e a cena `=3`.
# Corre-se por `bash scripts/ph2d-run.sh bash <este ficheiro>`.
#
# ⚠️ Controlo sobre o próprio filtro: uma corrida que casa ZERO testes é acusada como defeito do
# ARNÊS, nunca lida como «sobreviveu» (e uma mutação que não compila cai no mesmo braço). A agulha
# tem de casar EXACTAMENTE uma vez, senão o caso ABORTA. ⭐ `MUTA_SO_ANCORAS=1` corre só o pré-voo
# das âncoras (segundos, sem compilar) — o `cargo fmt` reescreve uma âncora e ela passa a casar zero.
set -u
cd "$(dirname "$0")/../../.."

SO_ANCORAS=${MUTA_SO_ANCORAS:-0}
# ⚠️ `MUTA_SECCOES="LEI FIS"` corre só essas secções — as 38 mutações inteiras passam o prazo de
# 30 min da fatia, e um arnês morto a meio deixa a árvore MUTADA. O sumário diz que é parcial.
SECCOES=${MUTA_SECCOES:-}
ACTIVA=1
seccao() { # CHAVE título
  if [ -z "$SECCOES" ] || [[ " $SECCOES " == *" $1 "* ]]; then ACTIVA=1; echo "== $2 =="; else ACTIVA=0; fi
}
ancoras_ok=0
ancoras_mortas=0

corre() { # rótulo cargo-args...
  local rot=$1; shift
  [ "$ACTIVA" = 1 ] || return
  [ "$SO_ANCORAS" = 1 ] && return
  out=$(cargo test "$@" 2>&1)
  r=$(echo "$out" | grep -oE "test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed" | head -1)
  n=$(echo "$r" | awk '{print $4+$6}')
  if [ "${n:-0}" -eq 0 ]; then
    echo "  $rot -> ARNES: zero testes (filtro ou compilacao)"
    echo "$out" | grep -E "^error" | head -3
    return
  fi
  echo "  $rot -> $r"
}
muta() { # ficheiro agulha substituto rótulo cargo-args...
  local f=$1 a=$2 b=$3 rot=$4; shift 4
  [ "$ACTIVA" = 1 ] || return
  if [ "$SO_ANCORAS" = 1 ]; then
    if python3 - "$f" "$a" <<'P'
import sys; p,a=sys.argv[1:3]; s=open(p).read(); sys.exit(0 if s.count(a)==1 else 1)
P
    then ancoras_ok=$((ancoras_ok+1)); else ancoras_mortas=$((ancoras_mortas+1)); echo "  $rot -> ANCORA MORTA"; fi
    return
  fi
  cp "$f" /tmp/mut_w6.bak
  if ! python3 - "$f" "$a" "$b" <<'P'
import sys; p,a,b=sys.argv[1:4]; s=open(p).read()
assert s.count(a)==1, ("agulha", a, s.count(a)); open(p,'w').write(s.replace(a,b))
P
  then echo "  $rot -> ABORTADO (agulha)"; cp /tmp/mut_w6.bak "$f"; touch "$f"; return; fi
  corre "$rot" "$@"
  cp /tmp/mut_w6.bak "$f"; touch "$f"
}

LEI=(-p ph2d-health --lib tipos)
FIS=(-p ph2d-physics-ecs --test it tipos_de_dano)
INSP=(-p ph2d-app-components --lib vida_inspector)
PAINEL=(-p ph2d-panel-inspector --test it a_lista_de_resistencias)
CENA=(-p ph2d-app-components --lib vida_tipos_smoke)
SHELL=(-p ph2d-host-desktop --test it a_vida_nao_aceita_mais_resistencias)
T=crates/ph2d-health/src/tipos.rs
L=crates/ph2d-health/src/lib.rs
F=crates/ph2d-physics-ecs/src/bridge/health.rs
C=crates/ph2d-physics-ecs/src/components/health.rs
I=crates/ph2d-app-components/src/vida_inspector.rs
E=crates/ph2d-panel-inspector/src/event_vida.rs
S=crates/ph2d-panel-inspector/src/sync_vida.rs
P=crates/ph2d-panel-inspector/src/populate_vida.rs
R=crates/ph2d-panel-inspector/src/sections/vida_resist.rs
V=crates/ph2d-app-components/src/vida_tipos_smoke.rs

seccao LEI "a LEI da taxa e das aflicoes"
corre CONTROLO "${LEI[@]}"
muta $T "        if self.mult.is_finite() && self.mult > 0.0 {" "        if self.mult.is_finite() {" \
  "T1 uma taxa negativa passa" "${LEI[@]}"
muta $T "            a.resta_s = a.resta_s.max(dur_s);" "            a.resta_s = dur_s;" \
  "T2 reaplicar encurta" "${LEI[@]}"
muta $T "            a.por_s = a.por_s.max(por_s);
            return;" "            a.por_s = a.por_s.max(por_s);
            a.ate_pulso_s = a.intervalo_s;
            return;" "T3 reaplicar adia o pulso" "${LEI[@]}"
muta $T "                if acabou || a.intervalo_s <= 0.0 || a.ate_pulso_s <= CHEGOU_S {" \
  "                if a.intervalo_s <= 0.0 || a.ate_pulso_s <= CHEGOU_S {" \
  "T4 o ultimo pulso perde o resto" "${LEI[@]}"
muta $T "        if let Some(a) = self.0.iter_mut().find(|a| a.tipo == tipo) {" \
  "        if let Some(a) = self.0.iter_mut().find(|_| false) {" "T5 dois venenos somam-se" "${LEI[@]}"
muta $L "        let d = dano * taxa.fator();" "        let d = (dano - cfg.armadura_fixa).max(0.0) * taxa.fator();" \
  "L1 o pulso passa pela armadura" "${LEI[@]}"
muta $L "        self.leva(cfg, regras, d, usa_escudo, false);" "        self.leva(cfg, regras, d, usa_escudo, true);" \
  "L2 o pulso arma a invencibilidade" "${LEI[@]}"
muta $L "            d *= taxa.fator();" "" "L3 o golpe ignora a taxa" "${LEI[@]}"
muta $L "        if taxa.absorve {
            self.cura(cfg, regras, dano * taxa.fator());
            return;
        }
        if self.invencivel(cfg) {
            return;
        }" "        if self.invencivel(cfg) {
            return;
        }
        if taxa.absorve {
            self.cura(cfg, regras, dano * taxa.fator());
            return;
        }" "L4 a invencibilidade trava o absorver" "${LEI[@]}"

seccao FIS "a PONTE"
corre CONTROLO "${FIS[@]}"
C0='                if !barrado && !fs.contains(&HealthEventKind::Dodged) && dano.over_time_per_s > 0.0'
muta $F "$C0" '                if !fs.contains(&HealthEventKind::Dodged) && dano.over_time_per_s > 0.0' \
  "F1 a invencibilidade deixa a aflicao" "${FIS[@]}"
muta $F "$C0" '                if !barrado && dano.over_time_per_s > 0.0' "F2 a esquiva deixa a aflicao" "${FIS[@]}"
muta $F "                st.aflicoes.limpa();" "" "F3 a morte nao cura" "${FIS[@]}"
muta $F "            for p in st.aflicoes.anda(dt) {" "            for p in st.aflicoes.anda(0.0) {" \
  "F4 as aflicoes nunca pulsam" "${FIS[@]}"
muta $F "                    .pulso(&cfg, Regras::CASA, p.pontos, h.taxa(&p.tipo), true);" \
  "                    .pulso(&cfg, Regras::CASA, p.pontos, ph2d_health::Taxa::NEUTRA, true);" \
  "F5 o pulso ignora a resistencia" "${FIS[@]}"
muta $F "                let taxa = h.taxa(&dano.kind);" "                let taxa = ph2d_health::Taxa::NEUTRA;" \
  "F6 o golpe ignora a resistencia" "${FIS[@]}"
muta $F "$C0" '                if comecou && !barrado && !fs.contains(&HealthEventKind::Dodged) && dano.over_time_per_s > 0.0' \
  "F7 a lava so' queima no comeco do toque" "${FIS[@]}"
seccao CHAVE "a CHAVE do tipo"
muta $C "    signal_name(kind).map(ph2d_label_fold::fold)" "    signal_name(kind).map(str::to_owned)" \
  "C1 o tipo nao se dobra" "${FIS[@]}"
muta $C "            .find(|r| kind_key(&r.kind).as_deref() == Some(chave.as_str()))" \
  "            .rfind(|r| kind_key(&r.kind).as_deref() == Some(chave.as_str()))" \
  "C2 a ultima linha ganha" "${FIS[@]}"

seccao INSP "o INSTANTANEO e o DRENO do Inspector"
corre CONTROLO "${INSP[@]}"
muta $I "            if h.resistances.len() >= RESISTANCES_MAX {" "            if false {" \
  "I1 juntar passa o tecto" "${INSP[@]}"
muta $I "            let repetida = chave.as_ref().is_some_and(|c| vistas.contains(c));" \
  "            let repetida = false;" "I2 nada e' repetido" "${INSP[@]}"
muta $I "            let chave = kind_key(&r.kind);" "            let chave = Some(r.kind.clone());" \
  "I3 a repetida nao usa a dobra" "${INSP[@]}"
muta $I "            Some(r) => r.rate = positivo(*v)," "            Some(r) => r.rate = *v," \
  "I4 uma taxa negativa grava-se" "${INSP[@]}"
muta $I "            Some(r) => r.kind = t.trim().to_string()," "            Some(r) => r.kind = t.clone()," \
  "I5 o tipo nao se apara" "${INSP[@]}"
muta $I "            if usize::from(*i) >= h.resistances.len() {
                return false;
            }
            h.resistances.remove(usize::from(*i));" "            h.resistances.remove(usize::from(*i));" \
  "I6 tirar fora da tabela" "${INSP[@]}"

seccao PAINEL "o PAINEL"
corre CONTROLO "${PAINEL[@]}"
muta $S "    let aberta = resistencia_aberta(&info, inspector_state.resist_selected);" \
  "    let aberta = resistencia_aberta(&info, 0);" "P1 a semente fica na 1.a linha" "${PAINEL[@]}"
muta $S "    aberta.hash(&mut h);" "" "P2 a assinatura sem a aberta (duas linhas iguais)" \
  "${PAINEL[@]}"
muta $E "        panel.resist_selected = n;" "        panel.resist_selected = 0;" "P3 o + nao abre a nova" \
  "${PAINEL[@]}"
muta $E "        panel.resist_selected = i;
        return Some(None);" "        panel.resist_selected = i;
        return Some(Some(E::AddResistance));" "P4 abrir uma linha vai ao barramento" "${PAINEL[@]}"
muta $E "        return Some(Some(E::RemoveResistance(u8::try_from(k).ok()?)));" \
  "        return Some(Some(E::RemoveResistance(0)));" "P5 o x tira a 1.a linha" "${PAINEL[@]}"
muta $E "        ids::INSP_VIDA_RESIST_RATE => E::ResistanceRate(u8::try_from(aberta?).ok()?, f)," \
  "        ids::INSP_VIDA_RESIST_RATE => E::ResistanceRate(0, f)," "P6 a taxa escreve na 1.a linha" \
  "${PAINEL[@]}"
muta $P "    super::populate::register_button_ids(store, &ids::INSP_VIDA_RESIST_ROW);" "" \
  "P7 as linhas mortas sob o dedo" "${PAINEL[@]}"
muta $R "        h.resistances.len() < ids::INSP_VIDA_RESIST_ROW.len()," "        true," \
  "P8 o + fica no tecto" "${PAINEL[@]}"

seccao SHELL "o TECTO amarrado aos ids (shell)"
corre CONTROLO "${SHELL[@]}"
muta $C "pub const RESISTANCES_MAX: usize = 8;" "pub const RESISTANCES_MAX: usize = 9;" \
  "S1 o modelo aceita mais do que o painel pinta" "${SHELL[@]}"

seccao CENA "a CENA"
corre CONTROLO "${CENA[@]}"
muta $V "        resistencias: &[(FOGO, 0.0, false), (GELO, 2.0, false)]," "        resistencias: &[]," \
  "V1 a salamandra sem resistencias" "${CENA[@]}"
muta $V "    arma(world, heroi, \"Arma de gelo\", SINAL_GELO, bala_gelo);" \
  "    arma(world, heroi, \"Arma de gelo\", SINAL_GELO, bala_fogo);" "V2 a arma de gelo atira fogo" "${CENA[@]}"
muta $V "                over_time_per_s: if queima { QUEIMA_POR_S } else { 0.0 }," \
  "                over_time_per_s: 0.0," "V3 a bala de fogo sem queimadura" "${CENA[@]}"
muta $V "        comeca: VIDA / 2.0," "        comeca: VIDA," "V4 o elemental nasce cheio" "${CENA[@]}"

[ -n "$SECCOES" ] && echo "PARCIAL: so' as seccoes [$SECCOES]"
if [ "$SO_ANCORAS" = 1 ]; then
  echo "PRE-VOO: $ancoras_ok ancoras vivas, $ancoras_mortas mortas"
  [ "$ancoras_mortas" -eq 0 ]
fi
