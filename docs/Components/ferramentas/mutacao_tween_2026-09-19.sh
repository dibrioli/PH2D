#!/usr/bin/env bash
# Provas de mutação do SUPLENTE #22 — O TWEEN (dois números e uma curva, sem grafo nenhum).
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE, e isso
#      lê-se exactamente como «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes** —
#      uma mutação que não entra lê-se exactamente como uma que sobreviveu.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_tween_2026-09-19.sh
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

LEI=crates/ph2d-tween/src/lib.rs
PRESET=crates/ph2d-tween/src/preset.rs
TIMER=crates/ph2d-ecs/src/timer.rs
ECS=crates/ph2d-ecs/src/tween.rs
PONTE=crates/ph2d-app-components/src/tween_bridge.rs
PAINEL=crates/ph2d-app-components/src/tween_inspector.rs
CENA=crates/ph2d-app-components/src/tween_smoke.rs

echo "════ W0 — o FACTO e o ACONTECIMENTO (ph2d-ecs) ════"

# (1) O `finished` do ESTADO morre: um relógio esgotado volta a ler-se como «por arrancar», e o
#     tween que devia FICAR no valor final desaparece. ⚠️ É a wave inteira num campo.
bloco "timer: o facto de ter acabado" ph2d-ecs o_fim_tem_duas_respostas "$TIMER" 1 \
  "            state.finished = true;" \
  "            state.finished = false;"

echo "════ W1 — a LEI (ph2d-tween) ════"

# (2) O `Rewind` deixa de devolver a arte: o pisca fica aceso para sempre.
bloco "lei: Rewind passa a Hold" ph2d-tween o_flash_volta_a_arte "$LEI" 1 \
  "            AoAcabar::Rewind => return None," \
  "            AoAcabar::Rewind => 1.0,"

# (3) ⚠️ A armadilha do ZERO: um relógio POR ARRANCAR e um PARADO NO ZERO leem-se igual, e escrever
#     no primeiro põe o motor a pintar antes de alguém lhe tocar.
bloco "lei: por arrancar passa a escrever" ph2d-tween por_arrancar_ele_nao_escreve "$LEI" 1 \
  "        return None;
    };" \
  "        0.0
    };"

# (4) A ARIDADE deixa de decidir: um canal escalar passa a escrever as quatro componentes, e um
#     `Opacity` levaria a cor do sprite atrás dele.
bloco "lei: a aridade ignorada" ph2d-tween um_canal_escalar_so_escreve "$LEI" 1 \
  ".take(t.canal.aridade())" \
  ".take(4)"

# (5) A curva deixa de ser avaliada: todo tween vira linear, e as onze famílias ficam inertes.
bloco "lei: a curva evaporada" ph2d-tween a_curva_e_a_do_motor "$LEI" 1 \
  "Some(mistura(t, t.easing.eval(u)))" \
  "Some(mistura(t, u))"

echo "════ W2 — o ÍNDICE (ph2d-ecs) ════"

# (6) O tween deixa de ler o relógio DELE e passa a ler o primeiro: dois tweens no mesmo objecto
#     correm em fase, e a quarta coluna da cena deixa de ensinar o que promete.
# ⛔⛔ **Ela SOBREVIVEU a` 1.a corrida, e a culpa era da FIXTURA do gate:** o
#     `o_tween_corre_no_relogio_do_mesmo_indice` tinha UM tween e UM relogio, e com um so'
#     elemento `get(i)` e `first()` sao a mesma coisa. *Uma fixtura com um elemento nao pode
#     testar um indice* — o gate foi renomeado para o que mede, e a lei ficou com o irmao.
bloco "ecs: o indice trocado pelo zero" ph2d-ecs o_indice_e_que_liga "$ECS" 1 \
  "(cfg.0.get(i), rt.0.get(i))" \
  "(cfg.0.first(), rt.0.first())"

echo "════ W3 — a PONTE (ph2d-app-components) ════"

# (7) A silhueta deixa de acender o interruptor: ela vira uma tinta, e o pisca não se vê.
bloco "ponte: a silhueta sem o interruptor" ph2d-app-components a_silhueta_acende "$PONTE" 1 \
  "let fill = matches!(p.canal, Canal::Silhueta);" \
  "let fill = false;"

# (8) ⚠️ O `Transform` escrito INTEIRO: `PositionX` e `PositionY` passam a apagar-se um ao outro, e
#     o modo de falha é o par a compor — nenhum dos dois sozinho o mostra.
bloco "ponte: o campo trocado pelo Transform" ph2d-app-components dois_tweens_de_pose "$PONTE" 1 \
  "                Canal::PositionY => t.translation.y = v[0]," \
  "                Canal::PositionY => t.translation = ph2d_core::Vec2::new(0.0, v[0]),"

# (9) ⭐⭐ O LEDGER cala-se: a corrida passa a ser DOCUMENTO e cada quadro do tween vira um passo
#     de `Ctrl+Z`. ⚠️ É a lei que o `preview_drive` existe para impor, e ela não se vê na tela.
bloco "ponte: o ledger sem declaracao" ph2d-app-components o_fade_nao_entra_no_undo "$PONTE" 1 \
  "        declara(sim, drive, e, era);" \
  "        let _ = (&era, &drive);"

echo "════ W5 — os PRESETS (ph2d-tween) ════"

# (10) O `Flash` passa a FICAR: a silhueta acesa para sempre, que é a queixa que o painel sabe dizer.
bloco "preset: o flash com Hold" ph2d-tween o_flash_volta_a_arte "$PRESET" 1 \
  "                ao_acabar: AoAcabar::Rewind," \
  "                ao_acabar: AoAcabar::Hold,"

# (11) ⭐ A DURAÇÃO do preset deixa de ser escrita no relógio: o pisca dura UM SEGUNDO (o valor de
#      fábrica do timer), oito vezes mais lento — e o dono lê *«o preset não funcionou»*.
bloco "preset: a duracao por escrever" ph2d-app-components um_preset_escreve_tambem "$PAINEL" 1 \
  "                    (t.duration_us != p.duracao_us()).then(|| {
                        t.duration_us = p.duracao_us();
                    })" \
  "                    Some(())"

# (12) Um preset que não move nada — o botão morto com cara de feature.
bloco "preset: o fade-in inerte" ph2d-tween todo_preset_produz_um_tween "$PRESET" 1 \
  "..Tween::linear(Canal::Opacity, 0.0, 1.0)" \
  "..Tween::linear(Canal::Opacity, 1.0, 1.0)"

echo "════ W6 — a CENA (ph2d-app-components) ════"

# (13) Os dois relógios da quarta coluna passam a ter o MESMO período: o quadrado cresce por igual
#      e a coluna deixa de demonstrar a lei do índice — a cena continua a montar, calada.
bloco "cena: a quarta coluna em fase" ph2d-app-components a_coluna_do_crescer "$CENA" 1 \
  '            laco("largura", PERIODO_CURTO_US),' \
  '            laco("largura", PERIODO_US),'

# (14) O CONTROLO da `=2` ganha tweens: os dois lados entram iguais e a cena não compara nada.
bloco "cena: o controlo com tween" ph2d-app-components a_copia_e_o_controlo "$CENA" 1 \
  'let sem = receita(world, "Copia (controlo)", COPIA_CTRL_RGBA, false);' \
  'let sem = receita(world, "Copia (controlo)", COPIA_CTRL_RGBA, true);'

# (15) O relógio de uma coluna passa a FALAR: a tela enche-se de avisos a cada segundo.
bloco "cena: o laco a publicar" ph2d-app-components a_galeria_abre_a_correr "$CENA" 1 \
  "        repeat: true,
        autostart: true,
        signal: String::new()," \
  '        repeat: true,
        autostart: true,
        signal: "tick".to_owned(),'

# (16) A galeria abre PARADA: quatro quadrados imóveis, e o dono lê «o tween não funciona».
bloco "cena: a galeria sem autostart" ph2d-app-components a_galeria_abre_a_correr "$CENA" 1 \
  "        repeat: true,
        autostart: true," \
  "        repeat: true,
        autostart: false,"

# (17) As duas fábricas da `=2` ficam por resolver: a cena abre e nada nasce.
bloco "cena: as receitas por resolver" ph2d-app-components as_duas_fabricas_correm "$CENA" 1 \
  "            resolver_receitas(world);" \
  "            let _ = &world;"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"
