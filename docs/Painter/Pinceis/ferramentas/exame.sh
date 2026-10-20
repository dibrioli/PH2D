#!/usr/bin/env bash
# exame.sh — a BANCADA do EXAME do pincel (docs/Painter/Pinceis/04_BANCADA.md; o contrato: 00_MISSAO §3).
#
# uso: exame.sh <pincel> [<oraculo-dir>]      a bateria pintada pelo nosso motor, a matriz contra a biblioteca
#                                             (em cache), a régua contra o oráculo e as pranchas (legível e cega)
#      exame.sh --matriz                      só a biblioteca inteira: matriz.csv + vizinhos.md em Pinceis/matriz/
#      exame.sh --custo <pincel> [5,40,200]   ms/quadro a 60 Hz e a memória pico, um processo por tamanho
#
# <pincel>: o nome de um preset de fábrica de qualquer meio ("Graphite 8B") · o caminho de um .ph2d-brush.png · meio:Digital |
#           meio:Watercolor | meio:Impasto | meio:WetPaint (o pincel do meio, receita em código)
# <oraculo-dir>: as fotos `<cena>_<A|B>.png` do oráculo (ou `<prefixo>_<cena>_<A|B>`, ou uma por cena — ver
#           `exame_regua.acha_foto`) · ou um PREFIXO numa pasta de muitos presets:
#           docs/Painter/Pinceis/fixtures/rebelle_bateria/PENCIL/graphite_pencil_8b
# env: PH2D_EXAME_BATERIA (outra bateria: `legado_5gestos.json` só para validar contra as fotos antigas) ·
#      PH2D_EXAME_ESCALA (o diâmetro de cada traço × escala; por omissão, com oráculo, a largura do recto_lento DELE
#      ÷ a nossa a escala 1 — compara-se à largura MEDIDA igual, 05 §4.1) · PH2D_EXAME_EXTRA (2.ª raiz de presets:
#      os que já saíram, para a matriz e a calibração) · PH2D_EXAME_DIR (o trabalho e a cache; /tmp/ph2d_exame —
#      um por família em paralelo) · PH2D_EXAME_PUBLICA=1 (o --matriz com PH2D_EXAME_DIR copia para Pinceis/matriz/) ·
#      PH2D_EXAME_SUFIXO (acrescentado ao nome das pranchas)
set -euo pipefail
RAIZ=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
F=$RAIZ/docs/Painter/Pinceis/ferramentas
P=$RAIZ/docs/Painter/Pinceis
# A bateria do EXAME (a da matriz, sempre) e a da comparação com o oráculo (a mesma, salvo PH2D_EXAME_BATERIA).
BATM=$F/bateria.json
BAT=$(realpath "${PH2D_EXAME_BATERIA:-$BATM}")
NB=$(basename "$BAT" .json)
DIR=${PH2D_EXAME_DIR:-${TMPDIR:-/tmp}/ph2d_exame}
EXTRA=${PH2D_EXAME_EXTRA:-}
[[ -n $EXTRA ]] && EXTRA=$(realpath "$EXTRA")
mkdir -p "$DIR"
py() { python3 -I "$@"; }

# pinta <presets separados por |> <saida> [escala] [bateria] — o teste `exame_bancada::exame_pinta` pela fatia
# da linha (a bateria por omissão é a do EXAME).
pinta() {
  local log="$DIR/pinta.log"
  PH2D_EXAME_PRESETS="$1" PH2D_EXAME_OUT="$2" PH2D_EXAME_BATERIA="${4:-$BATM}" PH2D_EXAME_ESCALA="${3:-1}" \
    PH2D_EXAME_EXTRA="$EXTRA" bash "$RAIZ/scripts/ph2d-run.sh" cargo test -p ph2d-tool-painter --profile smoke \
    --test it exame_bancada::exame_pinta -- --ignored --nocapture --test-threads 1 >"$log" 2>&1 || true
  grep -E "^\[exame\] [0-9]" "$log" || true
  grep -q "test result: ok. 1 passed" "$log" || { tail -30 "$log"; echo "exame.sh: a pintura falhou ($log)"; exit 1; }
}

# A biblioteca inteira pintada a escala 1 pela bateria do EXAME, em cache pela chave (bateria, presets, motor).
biblioteca() {
  local B="$DIR/biblioteca" chave
  chave=$( {
    sha256sum "$BATM"
    find "$RAIZ/assets/brushes" ${EXTRA:+"$EXTRA"} -name '*.ph2d-brush.png' -print0 | sort -z | xargs -0 sha256sum
    git -C "$RAIZ" rev-parse HEAD
    git -C "$RAIZ" diff HEAD -- crates | sha256sum
  } | sha256sum | cut -c1-16)
  if [[ ! -f $B/chave || $(cat "$B/chave") != "$chave" ]]; then
    echo "▸ a biblioteca inteira pela bateria do EXAME (chave $chave)…" >&2
    rm -rf "$B" && mkdir -p "$B"
    pinta "todos" "$B" 1 >&2
    echo "$chave" >"$B/chave"
  fi
  echo "$B"
}

matriz() { # <vista-dir> <saida>
  local saiu=""
  [[ -n $EXTRA ]] && saiu=$(find "$EXTRA" -name '*.ph2d-brush.png' -printf '%f\n' | sed 's/\.ph2d-brush\.png$//' | paste -sd,)
  # A cache da régua no trabalho DESTA corrida (por omissão o `exame_matriz.py` usava `$TMPDIR/ph2d_exame`, partilhada).
  py "$F/exame_matriz.py" "$1" "$2" --bateria "$BATM" --saiu "$saiu" --cache "$DIR/regua_cache"
}

case "${1:-}" in
  "" | -h | --help) sed -n '2,19p' "$0"; exit 0 ;;
  --matriz)
    B=$(biblioteca)
    matriz "$B" "$DIR/matriz"
    # Com um PH2D_EXAME_DIR próprio (as famílias em paralelo) a matriz fica nele: `Pinceis/matriz/` é UM ficheiro do
    # repo, e cada família só vê a sua biblioteca. Publicar lá: sem PH2D_EXAME_DIR, ou PH2D_EXAME_PUBLICA=1.
    if [[ -z ${PH2D_EXAME_DIR:-} || ${PH2D_EXAME_PUBLICA:-} == 1 ]]; then
      mkdir -p "$P/matriz" && cp "$DIR/matriz/"{matriz.csv,matriz_oraculos.csv,vizinhos.md,oraculos.md} "$P/matriz/"
      echo "▸ $P/matriz/{vizinhos.md,oraculos.md,matriz.csv,matriz_oraculos.csv}"
    else
      echo "▸ $DIR/matriz/{vizinhos.md,oraculos.md,matriz.csv,matriz_oraculos.csv} (PH2D_EXAME_PUBLICA=1 copia-a para $P/matriz/)"
    fi
    exit 0 ;;
  --custo)
    PINCEL=${2:?pincel}
    TAMS=${3:-5,40,200}
    J="$DIR/custo.jsonl"
    : >"$J"
    for t in ${TAMS//,/ }; do
      PH2D_EXAME_PRESETS="$PINCEL" PH2D_EXAME_TAMANHO="$t" PH2D_EXAME_BATERIA="$BAT" PH2D_EXAME_EXTRA="$EXTRA" \
        bash "$RAIZ/scripts/ph2d-run.sh" cargo test -p ph2d-tool-painter --profile smoke --test it \
        exame_bancada::exame_custo -- --ignored --nocapture --test-threads 1 2>&1 | grep -o '\[custo\] .*' |
        sed 's/^\[custo\] //' >>"$J"
    done
    py "$F/exame_custo.py" "$J"
    exit 0 ;;
esac

PINCEL=$1
ORACULO=${2:+$(realpath "$2")}
B=$(biblioteca)
# O nosso a escala 1: da biblioteca se for dela, senão pintado à parte.
C="$DIR/candidato"
rm -rf "$C" && mkdir -p "$C"
pinta "$PINCEL" "$C" 1
SLUG=$(ls "$C")
NOME=$(py -c "import json,sys;print(json.load(open(sys.argv[1]))['nome'])" "$C/$SLUG/meta.json")
V="$DIR/vista" && rm -rf "$V" && mkdir -p "$V"
for d in "$B"/*/; do ln -s "$d" "$V/$(basename "$d")"; done
rm -f "$V/$SLUG" && ln -s "$C/$SLUG" "$V/$SLUG"
M="$DIR/matriz_$SLUG"
matriz "$V" "$M" >/dev/null
VIZ=$(py -c "import json,sys;m=json.load(open(sys.argv[1]));print(m['top3'][sys.argv[2]][0][0])" "$M/matriz.json" "$NOME")
DV=$(py -c "import json,sys;m=json.load(open(sys.argv[1]));print(m['top3'][sys.argv[2]][0][1])" "$M/matriz.json" "$NOME")
LIM=$(py -c "import json,sys;print(round(json.load(open(sys.argv[1]))['calibracao']['limiar'],3))" "$M/matriz.json")
VSLUG=$(py -c "import json,sys;m=json.load(open(sys.argv[1]));import os;print(os.path.basename(m['dirs'][sys.argv[2]].rstrip('/')))" "$M/matriz.json" "$VIZ")
echo "▸ $NOME: vizinho mais perto = $VIZ (d $DV; limiar «claramente longe» $LIM)"

NOSSO="$C/$SLUG"
VIZD="$V/$VSLUG"
if [[ $BAT != "$BATM" ]]; then # outra bateria (o legado): o nosso a escala 1 nela, para a escala
  C2="$DIR/candidato_$NB" && rm -rf "$C2" && mkdir -p "$C2"
  pinta "$PINCEL" "$C2" 1 "$BAT"
  NOSSO="$C2/$SLUG"
  VIZD=""
fi
ESCALA=${PH2D_EXAME_ESCALA:-}
if [[ -n $ORACULO && -z $ESCALA ]]; then
  ESCALA=$(py -c "
import json,sys
sys.path.insert(0,sys.argv[1])
from exame_regua import mede, acha_foto
bat=json.load(open(sys.argv[2]))
c=next(c['nome'] for c in bat['cenas'] if any(o.get('nome')=='recto_lento' for o in c['ops']))
o=mede(acha_foto(sys.argv[3],c,'A'),c,bat)['tracos']['recto_lento']['largura']
n=mede(acha_foto(sys.argv[4],c,'A'),c,bat)['tracos']['recto_lento']['largura']
print(round(o/n,3))" "$F" "$BAT" "$ORACULO" "$NOSSO")
  echo "▸ escala = a largura do recto_lento do oráculo ÷ a nossa = $ESCALA"
fi
ESCALA=${ESCALA:-1}
if [[ -z $VIZD ]] || py -c "import sys;sys.exit(0 if abs(float(sys.argv[1])-1)>0.02 else 1)" "$ESCALA"; then
  E="$DIR/escala" && rm -rf "$E" && mkdir -p "$E"
  pinta "$PINCEL|$VIZ" "$E" "$ESCALA" "$BAT"
  NOSSO="$E/$SLUG"
  [[ -d $E/$VSLUG ]] && VIZD="$E/$VSLUG"
fi

SUF=${PH2D_EXAME_SUFIXO:-}
[[ $NB != bateria ]] && SUF="${SUF}_$NB"
mkdir -p "$P/pranchas/cegas" "$P/pranchas/chaves" "$P/exames"
if [[ -n $ORACULO ]]; then
  py "$F/exame_regua.py" compara "$ORACULO" "$NOSSO" --bateria "$BAT" \
    --json "$P/exames/${SLUG}${SUF}_regua.json" --txt "$P/exames/${SLUG}${SUF}_regua.txt"
fi
py "$F/exame_prancha.py" --nosso "$NOSSO" --vizinho "$VIZD" ${ORACULO:+--oraculo "$ORACULO"} --bateria "$BAT" \
  --titulo-nosso "$NOME (escala $ESCALA)" --titulo-vizinho "$VIZ (d $DV, limiar $LIM)" \
  --titulo-oraculo "$(basename "${ORACULO:-sem oráculo}")" \
  --saida "$P/pranchas/${SLUG}${SUF}.png" --cega "$P/pranchas/cegas/${SLUG}${SUF}.png" \
  --chave "$P/pranchas/chaves/${SLUG}${SUF}.json"
awk -F'|' -v n=" $NOME " '$2 == n || $2 == n "(saiu) "' "$M/vizinhos.md"
