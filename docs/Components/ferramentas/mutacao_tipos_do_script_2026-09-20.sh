#!/usr/bin/env bash
# Provas de mutação dos DOIS tipos que faltavam às propriedades de script — `vec2` e `color`.
#
# ⚠️ O arnês é o do irmão `mutacao_script_2026-09-16.sh`, com os TRÊS controlos que esta casa já
#    pagou: a âncora tem de casar o número EXACTO de vezes (uma troca que não casa é um no-op
#    silencioso que imprime «SOBREVIVEU»), a mutação tem de COMPILAR, e o filtro tem de casar mais
#    de zero testes (um filtro vazio sai VERDE e lê-se como sobrevivência).
#
# ⚠️ Corra-o PELA PORTA: `bash scripts/ph2d-run.sh bash docs/Components/ferramentas/<este>.sh`
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

prova() { # nome crate filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(timeout 900 cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO ou NAO COMPILA — '$3':"
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

bloco() { # nome crate filtro ficheiro vezes antigo novo
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

LEI=crates/ph2d-script/src/props.rs
VAL=crates/ph2d-script/src/valores.rs
CENA=crates/ph2d-script/src/scene.rs
POP=crates/ph2d-panel-inspector/src/populate_script.rs
PINT=crates/ph2d-panel-inspector/src/sections/script.rs
SEM=crates/ph2d-panel-inspector/src/sync_script.rs
INSP=crates/ph2d-app-components/src/script_inspector.rs
SMK=crates/ph2d-app-components/src/script_smoke.rs

# ── A LEI ────────────────────────────────────────────────────────────────────
bloco "a faixa de uma COR e' recusada" ph2d-script "a_faixa_vale_para_uma_posicao" \
  "$LEI" 1 "ScriptValueKind::Number | ScriptValueKind::Vec2" \
           "ScriptValueKind::Number | ScriptValueKind::Vec2 | ScriptValueKind::Color"
bloco "um canal de cor fora de 0..=1 recusa" ph2d-script "um_canal_de_cor_fora" \
  "$LEI" 1 "&& c.iter().any(|v| !(0.0..=1.0).contains(v))" "&& c.iter().any(|v| *v < -1e9)"
bloco "a finitude passa pela PORTA e alcanca todo tipo" ph2d-script "nenhum_componente_de_nenhum" \
  "$LEI" 1 "Self::Vec2(v) => v.as_slice()," "Self::Vec2(_) => &[],"

# ── A FORMA (uma porta, dois chamadores) ─────────────────────────────────────
bloco "a ida e volta da tabela" ph2d-script "o_que_a_casa_escreve_a_casa_le" \
  "$VAL" 1 't.set("y", *y)?;' 't.set("z", *y)?;'
bloco "uma string numerica nao e' um numero" ph2d-script "uma_string_numerica" \
  "$VAL" 1 "        Value::Number(n) => Some(n)," "        Value::Number(n) => Some(n),
        Value::String(s) => s.to_str().ok().and_then(|t| t.parse().ok()),"
bloco "a tabela sem marca nao e' um valor" ph2d-script "uma_tabela_sem_marca" \
  "$VAL" 1 "    let marca: String = t.get(MARCA).ok()?;" \
           "    let marca: String = t.get(MARCA).unwrap_or_else(|_| VEC2.to_owned());"
bloco "o PORTAO-COROA: a forma do artista e' a da casa" ph2d-script "a_forma_que_o_artista" \
  "$VAL" 1 't.set("r", *r)?;' 't.set("red", *r)?;'
bloco "o to_lua da cena passa pela PORTA" ph2d-script "uma_posicao_e_uma_cor_chegam_ao_script" \
  "$CENA" 1 "    if let Some(t) = crate::valores::tabela_de(lua, v)? {" \
            "    if let Some(t) = None::<Value> {"

# ── O PAINEL ─────────────────────────────────────────────────────────────────
bloco "os dois eixos estao no populate" ph2d-panel-inspector "a_posicao_tem_dois_campos" \
  "$POP" 1 ".chain(crate::ids::INSP_SCRIPT_VEC2_X)" ".chain([])"
bloco "a amostra e' uma amostra de SELECTOR" ph2d-panel-inspector "a_amostra_abre_o_selector" \
  "$POP" 1 "        store.register_picker_swatch(id);" "        let _ = id;"
bloco "os dois eixos partilham a fileira" ph2d-panel-inspector "a_posicao_tem_dois_campos" \
  "$PINT" 1 "                let x = ctrl.x + (meia + gap_eixos) * k as f32;" \
            "                let x = ctrl.x + (meia + gap_eixos) * 0.0;"
bloco "a amostra le' a cor gravada" ph2d-panel-inspector "a_amostra_abre_o_selector" \
  "$SEM" 1 "            host.store_mut().set_widget_color(id, gravado);" "            let _ = gravado;"

# ── A PONTE ──────────────────────────────────────────────────────────────────
bloco "o SetVec2 chega ao documento" ph2d-app-components "a_posicao_e_a_cor_chegam_ao_documento" \
  "$INSP" 1 "        E::SetVec2(n, p) => posto(n, ScriptValue::Vec2(*p)).map(|v| Mexe::Poe(n.clone(), v))," \
            "        E::SetVec2(n, _) => { let _ = n; None }"
# ⚠️ **A âncora abaixo foi REESCRITA pelo `cargo fmt`** depois de a 1.ª redacção deste roteiro a
#    ter copiado da linha única que eu escrevera — e uma âncora que não casa lê-se exactamente
#    como uma mutação que SOBREVIVEU. O controlo da contagem é o que a apanhou.
bloco "o passo le' o eixo com casas decimais" ph2d-app-components "o_passo_de_uma_posicao" \
  "$INSP" 1 "                        if v[0].fract() == 0.0 {
                            v[1]
                        } else {
                            v[0]
                        }" \
            "                        v[0]"

# ── A CENA ───────────────────────────────────────────────────────────────────
bloco "o default da direccao e' a reta para cima" ph2d-app-components "o_ficheiro_da_cena_declara_a_posicao" \
  "$SMK" 1 'ph2d.property("direction", ph2d.vec2(0, 1)' 'ph2d.property("direction", ph2d.vec2(1, 0)'
bloco "a cor de fabrica e' branca" ph2d-app-components "o_ficheiro_da_cena_declara_a_posicao" \
  "$SMK" 1 'ph2d.property("tint", ph2d.color(1, 1, 1))' 'ph2d.property("tint", ph2d.color(0.5, 0.5, 0.5))'

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL sangraram"
else
  echo "⛔ $FALHAS de $TOTAL NAO sangraram"
fi
exit "$FALHAS"
