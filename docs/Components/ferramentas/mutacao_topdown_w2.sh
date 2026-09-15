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
PT=crates/ph2d-physics-ecs/src/bridge/topdown.rs
PO=crates/ph2d-physics-ecs/src/bridge/pose_owner.rs
TA=crates/ph2d-physics-ecs/src/bridge/tape.rs
CO=crates/ph2d-physics-ecs/src/components/topdown.rs
SH=shells/desktop/src/render_loop/inspector_topdown.rs
SM=crates/ph2d-app-components/src/topdown_smoke.rs
LS=crates/ph2d-editor-core/src/ids/live_sections.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

# ═══ A PONTE ════════════════════════════════════════════════════════════════
# ⭐⭐⭐ O orcamento deixa de viajar entre passos: cada deslize recomeca do sitio
# ORIGINAL, e o corpo anda a projeccao (a lei do platformer) outra vez.
mutacao "M1 o orcamento nao viaja entre deslizes" "$PT" \
  'andado = [andado[0] + got.translation[0], andado[1] + got.translation[1]];' \
  'andado = [got.translation[0], got.translation[1]];' \
  ph2d-physics-ecs --test=it 'topdown_slide'

# ⭐⭐ O eixo VERTICAL deixa de ser lido — o mover fica so' com a horizontal.
mutacao "M2 o eixo vertical nao chega a' lei" "$PT" \
  'let bruto = [entrada.drive, entrada.drive_y];' \
  'let bruto = [entrada.drive, 0.0];' \
  ph2d-physics-ecs --test=it 'topdown_slide'

# ⭐ A CAMADA volta a ser um zero escrito a' mao.
mutacao "M3 a camada e' um literal" "$PT" \
  '.move_character_from(handle, andado, pedido, params, None, layer, &mut hits);' \
  '.move_character_from(handle, andado, pedido, params, None, 9, &mut hits);' \
  ph2d-physics-ecs --test=it 'a_camada_do_cast'

# ⭐ O guarda do CONFLITO: dois movers passam a escrever a pose no mesmo tique.
mutacao "M4 o conflito com o platformer deixa de calar este" "$PT" \
  'if world.get::<PlatformPlayer>(entity).is_some() {
                continue;
            }' \
  'if false {
                continue;
            }' \
  ph2d-physics-ecs --test=it 'com_dois_movers'

# ⭐⭐⭐ A POSSE DA POSE: sem ela o corpo anda no rapier e o `Transform` fica parado.
mutacao "M5 a posse da pose esquece o mover de vista de cima" "$PO" \
  'if kind == BodyKind::Kinematic && world.get::<TopDownPlayer>(entity).is_some() {' \
  'if false && world.get::<TopDownPlayer>(entity).is_some() {' \
  ph2d-physics-ecs --test=it 'topdown_slide'

# ⭐ A FITA passa a alcancar quem desligou os controlos de fabrica.
mutacao "M6 os controlos de fabrica deixam de gatear a fita" "$TA" \
  '|| world
                        .get::<TopDownPlayer>(e)
                        .is_some_and(|c| c.default_controls)' \
  '|| world.get::<TopDownPlayer>(e).is_some()' \
  ph2d-physics-ecs --test=it 'com_os_controlos_de_fabrica'

# ═══ O COMPONENTE ═══════════════════════════════════════════════════════════
# ⭐ O fio de um enum passa a ler outro modo — o defeito que reordenar produz.
mutacao "M7 a traducao componente->lei le' o modo errado" "$CO" \
  'direction: direction::from_wire(self.direction_mode),' \
  'direction: direction::from_wire(self.direction_mode.wrapping_add(1)),' \
  ph2d-physics-ecs --lib 'topdown'

# ═══ O PAINEL (a shell) ═════════════════════════════════════════════════════
# ⭐⭐ A pergunta do corpo vira a forma NEGATIVA, e um corpo estatico passa.
mutacao "M8 o aviso do corpo pergunta != Dynamic" "$SH" \
  'body_is_kinematic: corpo.is_some_and(|b| b.kind == BodyKind::Kinematic),' \
  'body_is_kinematic: corpo.is_some_and(|b| b.kind != BodyKind::Dynamic),' \
  ph2d-host-desktop --bins 'inspector_topdown'

# ⭐ O aviso do CONFLITO some do instantaneo.
mutacao "M9 o instantaneo deixa de acusar o conflito" "$SH" \
  'conflicts_with_platformer: world.get::<PlatformPlayer>(e).is_some(),' \
  'conflicts_with_platformer: false,' \
  ph2d-host-desktop --bins 'inspector_topdown'

# ⭐ A cerca do dreno: zero deslizes passa a ser aceite.
mutacao "M10 o dreno aceita ZERO deslizes" "$SH" \
  'law.max_slides = u8::try_from((*n).clamp(1, 8)).unwrap_or(4);' \
  'law.max_slides = u8::try_from((*n).min(8)).unwrap_or(4);' \
  ph2d-host-desktop --bins 'inspector_topdown'

# ⭐ A traducao painel->lei do viewpoint le' outro modo.
mutacao "M11 a traducao painel->lei troca o viewpoint" "$SH" \
  'InspectorViewpoint::Iso2to1 => Viewpoint::Isometric2to1,' \
  'InspectorViewpoint::Iso2to1 => Viewpoint::Isometric30,' \
  ph2d-host-desktop --bins 'inspector_topdown'

# ═══ A CENA ═════════════════════════════════════════════════════════════════
# ⭐⭐ O CONTROLO da cena 2 passa a ter o mesmo viewpoint do heroi — a cena deixa
# de poder distinguir «o menu funciona» de «ele anda assim de qualquer maneira».
mutacao "M12 o controlo da cena 2 deixa de ser um controlo" "$SM" \
  'viewpoint: Viewpoint::TopDown,
            ..TopDownLaw::default()
        },
    );
}' \
  'viewpoint: Viewpoint::Isometric2to1,
            ..TopDownLaw::default()
        },
    );
}' \
  ph2d-app-components --lib 'topdown_smoke'

# ⭐ O dente contra o qual o passo do smoke manda encostar desaparece.
mutacao "M13 a cena 1 perde o dente" "$SM" \
  'parede(world, "Wall Corner", Vec2::new(1.5, 1.0), Vec2::new(4.0, 0.5));' \
  'let _ = ();' \
  ph2d-app-components --lib 'topdown_smoke'

# ═══ A CATRACA DAS SECCOES VIVAS ════════════════════════════════════════════
# ⭐⭐⭐ O censo que faltava: uma seccao sai da tabela e ele tem de acusar.
# ⚠️ **A 1.ª redacção APAGAVA a linha, e isso não compila** (o tamanho do array é declarado) —
# o que é uma boa notícia e uma má prova: o compilador já guarda a CONTAGEM, e o que este censo
# guarda é a IDENTIDADE. A mutação que o exercita é uma DUPLICAÇÃO, que mantém o tamanho.
mutacao "M14 uma seccao viva sai da tabela (por duplicacao)" "$LS" \
  '    (INSP_LIVE_TOPDOWN_SECTION, INSP_LIVE_TOPDOWN_COLOR),' \
  '    (INSP_LIVE_TAGS_SECTION, INSP_LIVE_TAGS_COLOR),' \
  ph2d-editor-core --test=it 'every_live_section'

echo
echo "$N mutacoes · load $(cut -d' ' -f1 /proc/loadavg)"
