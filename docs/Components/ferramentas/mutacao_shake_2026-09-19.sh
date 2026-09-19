#!/usr/bin/env bash
# Provas de mutação do SUPLENTE #25 — O ABANÃO DA VISTA.
#
# Arnês IDÊNTICO ao das waves anteriores desta linha — controlo sobre o próprio FILTRO (um filtro
# que casa ZERO testes imprime `ok` e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora
# não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_shake_2026-09-19.sh
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

LEI=crates/ph2d-shake/src/lib.rs
COMP=crates/ph2d-ecs/src/camera_shake.rs
REWIND=crates/ph2d-ecs/src/rewind_runtime.rs
PONTE=crates/ph2d-app-components/src/shake_bridge.rs
INSP=crates/ph2d-app-components/src/shake_inspector.rs
CENA=crates/ph2d-app-components/src/shake_smoke.rs
EVENTO=crates/ph2d-panel-inspector/src/event_shake.rs
# ⚠️⚠️ **Este caminho MUDOU na mesma jornada e a 1.ª redacção ficou obsoleta:** a catraca
# `the_shell_only_shrinks` mandou a fase da câmera para a crate da família, e o script continuou a
# apontar para `shells/desktop/src/render_loop/camera_2d.rs`, que já não existe. *Uma sonda que
# aponta para um ficheiro que se mudou não fica em silêncio — o `muta` aborta e a prova conta como
# FALHA*, que é a sorte desta história: a forma cara é a que fica verde.
VISTA=crates/ph2d-app-components/src/camera_2d.rs

echo "════ A LEI (ph2d-shake) ════"

# 1) O ruído deixa de INTERPOLAR — ele passa a ser um degrau por célula. ⚠️ A régua é a
#    CONTINUIDADE, e um degrau não a tem: amostrar mais fino deixa de encolher o salto.
bloco "ruido: degrau em vez de interpolar" ph2d-shake \
  o_abanao_e_facto_do_tempo "$LEI" 1 \
  "    a + (b - a) * u" \
  "    let _ = (b, u); a"

# 2) O trauma deixa de ter piso — ele fica NEGATIVO e a vista nunca assenta.
bloco "decai: sem piso a zero" ph2d-shake \
  o_trauma_chega_a_zero "$LEI" 1 \
  "    (trauma - decaimento.max(0.0) * dt.max(0.0)).max(0.0)" \
  "    trauma - decaimento.max(0.0) * dt.max(0.0)"

# 3) O trauma deixa de SATURAR — dez explosões abanam dez vezes mais.
bloco "acumula: sem saturacao" ph2d-shake \
  o_trauma_satura "$LEI" 1 \
  "    (trauma + impulso.max(0.0)).min(TRAUMA_MAX) // CLAMP-OK" \
  "    (trauma + impulso.max(0.0)) // CLAMP-OK"

# 4) A cerca do raio EXTERNO desaparece — para lá dele a atenuação cresce em vez de ser zero.
#
# ⚠️⚠️ **A 1.ª redacção mutava `>=` para `>` e era um NO-OP**: a smoothstep dá **exactamente** `0`
# em `u = 1`, logo trocar a inclusão da borda não muda um bit. *Uma mutação que a aritmética já
# neutraliza lê-se como uma lei sem gate, e não é* — a cerca É load-bearing, mas para `d > fora`
# (onde `1 − u²(3 − 2u)` dispara), e é ESSE regime que a mutação tem de tocar.
bloco "atenuacao: sem a cerca do raio externo" ph2d-shake \
  a_atenuacao_e_um_ao_pe "$LEI" 1 \
  "    if distancia >= fora {
        return 0.0;
    }" \
  "    if false {
        return 0.0;
    }"

# 5) O expoente fica DESFASADO de um — `n = 1` passa a elevar ao quadrado.
bloco "potencia: expoente desfasado" ph2d-shake \
  o_expoente_zero_nao_apaga "$LEI" 1 \
  "    for _ in 1..n {" \
  "    for _ in 0..n {"

# 6) Os DOIS EIXOS partilham a semente — a vista abana numa diagonal só.
bloco "deslocamento: os dois eixos com a mesma semente" ph2d-shake \
  os_dois_eixos_nao_andam_juntos "$LEI" 1 \
  "        k * ruido(lei.semente ^ EIXO_Y, fase)," \
  "        k * ruido(lei.semente, fase),"

echo
echo "════ A PONTE (ph2d-app-components) ════"

# 7) A cerca de quem falou é IGNORADA — dez bombas iguais abanam todas.
bloco "ponte: a cerca de quem falou ignorada" ph2d-app-components \
  dez_bombas_iguais "$PONTE" 1 \
  "                        if !f.de.deixa_passar(falou, *e) {" \
  "                        if false {"

# 8) A DISTÂNCIA deixa de atenuar — perto e longe abanam igual.
bloco "ponte: a distancia nao atenua" ph2d-app-components \
  a_mesma_explosao_abana_menos "$PONTE" 1 \
  "                        let a = ph2d_shake::atenuacao(d, f.dentro, f.fora);" \
  "                        let a = { let _ = d; 1.0_f32 };"

# 9) O relógio do abanão deixa de andar — dois estrondos ficam ENLATADOS.
bloco "ponte: o relogio do abanao congelado" ph2d-app-components \
  dois_estrondos_seguidos "$PONTE" 1 \
  "        rt.t += dt;" \
  "        rt.t += 0.0;"

# 10) A distância sai do OUVINTE e não de quem gritou.
bloco "ponte: a distancia sai do ouvinte" ph2d-app-components \
  a_distancia_sai_de_quem_gritou "$PONTE" 1 \
  "                        let onde = falou
                            .and_then(|q| pose_de(sim, q))
                            .or_else(|| pose_de(sim, *e));" \
  "                        let onde = pose_de(sim, *e);"

# 11) A varredura «existe quem abane?» responde sempre SIM.
bloco "inspector: a cena tem sempre quem abane" ph2d-app-components \
  o_emissor_sabe_se_existe_quem_abane "$INSP" 1 \
  "    mundo.query::<&CameraShake>().iter(mundo).next().is_some()" \
  "    { let _ = mundo; true }"

# 12) O expoente deixa de ser coagido — o painel entrega um `0` à lei.
bloco "inspector: o expoente sem cerca" ph2d-app-components \
  o_expoente_e_coagido "$INSP" 1 \
  "                c.expoente = (*n).clamp(ph2d_shake::EXPOENTE_MIN, ph2d_shake::EXPOENTE_MAX); // CLAMP-OK: a faixa é da lei" \
  "                c.expoente = *n;"

echo
echo "════ REBOBINAR (ph2d-ecs) ════"

# 13) O abanão SAI do censo do rebobinar — a vista treme o resto da corrida anterior.
bloco "rewind: o abanao fora do censo" ph2d-app-components \
  rebobinar_apaga_o_trauma "$REWIND" 1 \
  "        *rt = crate::CameraShakeRuntime::default();" \
  "        let _ = &mut rt;"

echo
echo "════ A VISTA (ph2d-app-components) ════"

# 14) O offset deixa de CHEGAR ao centro da vista — o motor todo fica invisível.
bloco "vista: o offset nao chega ao centro" ph2d-app-components \
  o_abanao_chega_ao_centro_da_vista "$VISTA" 1 \
  "            rt.center[0] + cam.offset[0] + abanao[0]," \
  "            rt.center[0] + cam.offset[0],"

echo
echo "════ O PAINEL (ph2d-panel-inspector) ════"

# 15) O chip do expoente manda o ÍNDICE — a lei recebe um `0`.
bloco "painel: o chip manda o indice e nao o expoente" ph2d-panel-inspector \
  todo_chip_do_expoente "$EVENTO" 1 \
  "        let n = u8::try_from(i).unwrap_or(0) + ph2d_shake::EXPOENTE_MIN;" \
  "        let n = u8::try_from(i).unwrap_or(0);" \
  "--test it"

# 16) O chip da cerca escreve sempre na PRIMEIRA fonte.
bloco "painel: a cerca escreve sempre na primeira fonte" ph2d-panel-inspector \
  todo_chip_da_cerca "$EVENTO" 1 \
  "            push_emitter(host, bits, EE::De(sel_u8, u8::try_from(i).unwrap_or(0)));" \
  "            push_emitter(host, bits, EE::De(0, u8::try_from(i).unwrap_or(0)));" \
  "--test it"

echo
echo "════ A CENA (ph2d-app-components) ════"

# 17) O pátio fica RALO — há sítios onde o abanão é invisível.
bloco "cena: o patio ralo demais" ph2d-app-components \
  o_patio_cobre_mais "$CENA" 1 \
  "pub const POSTE_PASSO: f32 = 5.0;" \
  "pub const POSTE_PASSO: f32 = 20.0;"

# 18) A cerca da cena abre — uma segunda bomba passaria a ouvir o estrondo da primeira.
bloco "cena: a cerca aberta" ph2d-app-components \
  a_fonte_ouve_so_o_proprio "$CENA" 1 \
  "                de: SignalFrom::Myself," \
  "                de: SignalFrom::Anyone,"

# 19) O componente da câmera SAI da cena — nada treme.
bloco "cena: sem quem trema" ph2d-app-components \
  a_cena_tem_quem_grite "$CENA" 1 \
  "        CameraShake::default()," \
  "        Name::new(\"sem abanao\"),"

echo
echo "════════════════════════════════════════════"
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes SANGRAM"
else
  echo "⛔ $FALHAS de $TOTAL NAO sangraram"
fi
exit "$FALHAS"
