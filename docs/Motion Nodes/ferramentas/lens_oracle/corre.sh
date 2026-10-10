#!/usr/bin/env bash
# OS ORÁCULOS da lente e dos filtros (ciclo 20, doc 129 §5.1) — corre o ImageMagick e o ffmpeg
# instalados (licença conferida no ARTEFACTO pelo pacman) sobre entradas NOSSAS (`entrada.py`,
# `cubo.py`) e grava, com cabeçalho, em `crates/ph2d-lens/tests/fixtures/`. Correr de novo é mudar
# os gates.
#
#   bash "docs/Motion Nodes/ferramentas/lens_oracle/corre.sh"
#
# ⛔ O ffmpeg é GPL-3.0: CORRE-SE, nunca se lê (CLAUDE.md §0.9). O ImageMagick é permissivo.
# ⚠️ As entradas viajam ao ffmpeg em BRUTO de 16 bits: lido de um PNG, o caminho de conversão dele
# desvia `0,0046` (medido 10/10) e a fixture acusaria a lei por um erro do leitor.
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(git -C "$AQUI" rev-parse --show-toplevel)"
SAIDA="$RAIZ/crates/ph2d-lens/tests/fixtures"
mkdir -p "$SAIDA"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

pac() { LC_ALL=C pacman -Qi "$1" | sed -n "s/^$2 *: *//p"; }
IM_LIC="$(pac imagemagick Licenses)"; IM_VER="$(pac imagemagick Version)"
FF_LIC="$(pac ffmpeg Licenses)"; FF_VER="$(pac ffmpeg Version)"
[ "$IM_LIC" = "ImageMagick" ] || { echo "licenca inesperada do imagemagick: $IM_LIC" >&2; exit 1; }
[ "$FF_LIC" = "GPL-3.0-only" ] || { echo "licenca inesperada do ffmpeg: $FF_LIC" >&2; exit 1; }
IM="imagemagick $IM_VER (pacman) · licenca $IM_LIC"
FF="ffmpeg $FF_VER (pacman) · licenca $FF_LIC — corrido, nunca lido"
DATA="$(date -I)"
PASSO=4

python3 -I "$AQUI/entrada.py" cena "$TMP/cena.png"
python3 -I "$AQUI/entrada.py" branco "$TMP/branco.png"
python3 -I "$AQUI/cubo.py" "$SAIDA/look.cube"
magick "$TMP/cena.png" -endian MSB -depth 16 rgb:"$TMP/cena.raw"
amostra() { python3 -I "$AQUI/amostra.py" "$@"; }

amostra "$TMP/cena.raw" be 3 $PASSO "$SAIDA/entrada_cena.txt" \
  "a ENTRADA (entrada.py cena, 192x128) — o teste confere a formula Rust contra ela" "data: $DATA"

# A VINHETA: elipse de semi-eixos (W/2-ix, H/2-iy), borda borrada por gaussiana sigma, fundo preto.
# ⚠️ O centro do -vignette e o pixel de INDICE (96,64): em coordenadas continuas (96.5, 64.5).
for cfg in "8 24 16" "16 24 16" "4 40 10" "20 8 30"; do
  read -r s ix iy <<<"$cfg"
  magick "$TMP/branco.png" -background black -vignette "0x$s+$ix+$iy" -endian MSB -depth 16 \
    gray:"$TMP/v.raw"
  amostra "$TMP/v.raw" be 1 $PASSO "$SAIDA/vinheta_s${s}_${ix}_${iy}.txt" \
    "oraculo: $IM" "comando: magick branco.png -background black -vignette 0x$s+$ix+$iy" \
    "SIGMA $s INSET $ix $iy | x y mascara" "data: $DATA"
done

# A ABERRAÇÃO: cada canal escalado a partir do centro (96,64 continuo), bilinear, borda REPLICADA.
ab() {
  local nome="$1" r="$2" b="$3" desc="$4"
  magick "$TMP/cena.png" -separate \
    \( -clone 0 -virtual-pixel edge -filter point -interpolate bilinear -distort SRT "$r" \) \
    \( -clone 2 -virtual-pixel edge -filter point -interpolate bilinear -distort SRT "$b" \) \
    -delete 0,2 -swap 0,1 -combine -endian MSB -depth 16 rgb:"$TMP/ab.raw"
  amostra "$TMP/ab.raw" be 3 $PASSO "$SAIDA/$nome.txt" "oraculo: $IM" \
    "comando: -separate; R: -distort SRT '$r'; B: -distort SRT '$b'; -virtual-pixel edge -filter point -interpolate bilinear" \
    "$desc | x y r g b" "data: $DATA"
}
ab aberracao_radial "96,64 1.04 0" "96,64 0.96 0" "RADIAL escala R 1.04 B 0.96"
ab aberracao_linear "96,64 1 0 98.5,62.5" "96,64 1 0 93.5,65.5" "LINEAR R desloca (+2.5,-1.5) B (-2.5,+1.5)"

# A LUT: o mesmo .cube nas duas interpolações (a LUT mistura canais: as duas DIFEREM, 0,0036).
for m in tetrahedral trilinear; do
  ffmpeg -v error -y -f rawvideo -pix_fmt rgb48be -s 192x128 -i "$TMP/cena.raw" \
    -vf "lut3d=file=$SAIDA/look.cube:interp=$m" -f rawvideo -pix_fmt rgb48le "$TMP/lut.raw"
  amostra "$TMP/lut.raw" le 3 $PASSO "$SAIDA/lut_$m.txt" "oraculo: $FF" \
    "comando: ffmpeg -f rawvideo -pix_fmt rgb48be -i cena.raw -vf lut3d=file=look.cube:interp=$m" \
    "LUT look.cube ($m) sobre os valores CODIFICADOS | x y r g b" "data: $DATA"
done

# O BLUR (para o fx.filter, W3): gaussiana sigma 3 em luz LINEAR (o FxStackPass borra linear).
magick "$TMP/cena.png" -colorspace RGB -gaussian-blur 0x3 -colorspace sRGB -endian MSB -depth 16 \
  rgb:"$TMP/blur.raw"
amostra "$TMP/blur.raw" be 3 $PASSO "$SAIDA/blur_linear_s3.txt" "oraculo: $IM" \
  "comando: magick cena.png -colorspace RGB -gaussian-blur 0x3 -colorspace sRGB" \
  "BLUR sigma 3 px em luz linear, borda replicada | x y r g b" "data: $DATA"

ls -la "$SAIDA"
