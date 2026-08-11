#!/usr/bin/env bash
# Coze os matcaps de `crates/ph2d-mesh-render/assets/matcaps/`.
#
# NAO roda em nenhum build: existe para a tabela do LICENSES.md ser conferivel
# em vez de copiada, e para quem acrescentar um matcap nao ter de re-descobrir a
# lei de espaco de cor.
#
#     bash docs/3D/ferramentas/cook_matcaps.sh <dir-com-as-fontes> <dir-de-saida>
#
# A LEI: nada e' quantizado abaixo da precisao da FONTE.
#
#   - Blender (.exr): meio-float LINEAR, com `diffuse` e `specular` em camadas
#     separadas e compressao DWAA. O matcap e' a SOMA das duas (o nosso caminho
#     de matcap nao multiplica por cor de vertice, que e' para o que a separacao
#     serve no Blender). A saida e' um EXR RGB **simples**, meio-float, ZIP:
#     mesma precisao da fonte, e o nosso `ph2d-imageio-exr` o le -- ele recusava
#     o original por DOIS motivos escritos no doc dele (layout de canais alem de
#     RGBA · DWA/DWB), e nenhum dos dois e' sobre precisao.
#
#   - SculptGL (.jpg): 8 bits sRGB autorados. A saida e' PNG, que guarda os
#     MESMOS bytes que o JPEG decodifica -- bit-identico a fonte. Re-transferir
#     ou promover a float aqui nao acrescentaria informacao nenhuma; so' daria
#     um arquivo maior dizendo a mesma coisa.
#
# ⚠️ Por que NAO 8 bits para os do Blender, que foi o primeiro corte desta wave:
# medido, a quantizacao para 8 bits erra ~1 nivel de 255 de volta em LINEAR
# (0,93 no basic_bright, 1,09 no basic_side) contra 0,004 em 16 bits -- 259x.
# Um matcap e' um gradiente liso sobre uma esfera, que e' o caso classico de
# banda visivel. "Cabe em [0,1]" e "8 bits chegam" sao perguntas diferentes, e a
# primeira versao deste script respondeu a primeira achando que respondia a
# segunda.
set -euo pipefail

SRC="${1:-.}"
OUT="${2:-cooked}"
mkdir -p "$OUT"

BLENDER=(basic_bright basic_dark basic_grey basic_side clay_brown clay_green clay_warm red_wax)
SCULPTGL=(skinHazardousarts skinHazardousarts2)

echo "stem            saida    bytes  sha256 da fonte"

for stem in "${BLENDER[@]}"; do
  src="$SRC/$stem.exr"
  [ -f "$src" ] || { echo "  (falta $src)"; continue; }
  # As duas camadas, somadas, com os nomes de canal normalizados para R,G,B --
  # e' o prefixo `diffuse.`/`specular.` que faz o nosso decoder recusar.
  oiiotool "$src" --ch diffuse.R,diffuse.G,diffuse.B -o "$OUT/.d.exr" >/dev/null
  oiiotool "$src" --ch specular.R,specular.G,specular.B -o "$OUT/.s.exr" >/dev/null
  oiiotool "$OUT/.d.exr" "$OUT/.s.exr" --add --chnames R,G,B \
    -d half --compression zip --attrib BlenderMultiChannel "" \
    -o "$OUT/$stem.exr" >/dev/null
  printf "%-14s %-7s %7d  %s\n" "$stem" "exr/half" \
    "$(stat -c%s "$OUT/$stem.exr")" "$(sha256sum "$src" | cut -d' ' -f1)"
done
rm -f "$OUT/.d.exr" "$OUT/.s.exr"

for stem in "${SCULPTGL[@]}"; do
  src="$SRC/$stem.jpg"
  [ -f "$src" ] || { echo "  (falta $src)"; continue; }
  # Sem `-d`: o JPEG decodifica em 8 bits e o PNG guarda esses bytes.
  oiiotool "$src" -o "$OUT/$stem.png" >/dev/null
  printf "%-14s %-7s %7d  %s\n" "$stem" "png/8" \
    "$(stat -c%s "$OUT/$stem.png")" "$(sha256sum "$src" | cut -d' ' -f1)"
done
