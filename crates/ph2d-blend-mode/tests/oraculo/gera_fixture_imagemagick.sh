#!/usr/bin/env bash
# Oráculo CORRIDO (CLAUDE.md §0.9): o ImageMagick compõe a NOSSA grelha de cores em cada um dos 20
# modos e a saída vira `imagemagick_compose.txt`, lida pelo gate `tests/it/oraculo_imagemagick.rs`.
# Nunca se lê o fonte do IM — só se corre o binário instalado.
#
# Entrada: duas imagens PFM (float32, 49×1) com todos os pares (fundo × fonte) da grelha.
# O PFM entra marcado sRGB e NADA o converte (medido: Multiply 0,5·0,5 = 0,25 exacto, Over devolve a
# fonte bit a bit) ⇒ os números são dados crus, a mesma convenção do `apply` (linear, straight).
# Ordem do IM: `magick FUNDO FONTE -compose X -composite` (o 1.º é o destino/backdrop).
set -euo pipefail

aqui="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
saida="$aqui/imagemagick_compose.txt"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# código do nosso BlendMode → operador do IM (`magick -list compose`).
modos=(
  "0 Over" "1 Multiply" "2 Darken" "3 ColorBurn" "4 LinearBurn"
  "5 Lighten" "6 Screen" "7 ColorDodge" "8 LinearDodge" "9 Overlay"
  "10 SoftLight" "11 HardLight" "12 VividLight" "13 LinearLight" "14 Difference"
  "15 Exclusion" "16 Hue" "17 Saturate" "18 Colorize" "19 Luminize"
)

# A grelha NOSSA: cinzas 0 · 0,5 · 1 e quatro cores que põem cada canal em 0,05..0,95.
python3 -I - "$tmp" <<'PY'
import struct, sys
cores = [(0, 0, 0), (1, 1, 1), (0.5, 0.5, 0.5), (0.2, 0.8, 0.5),
         (0.9, 0.1, 0.3), (0.25, 0.4, 0.95), (0.8, 0.65, 0.05)]
def pfm(p, px):
    with open(p, "wb") as f:
        f.write(b"PF\n%d 1\n-1.0\n" % len(px))
        f.write(b"".join(struct.pack("<3f", *c) for c in px))
pares = [(b, s) for b in range(len(cores)) for s in range(len(cores))]
pfm(sys.argv[1] + "/fundo.pfm", [cores[b] for b, _ in pares])
pfm(sys.argv[1] + "/fonte.pfm", [cores[s] for _, s in pares])
with open(sys.argv[1] + "/grelha.txt", "w") as f:
    for i, c in enumerate(cores):
        f.write("cor %d %s\n" % (i, " ".join(repr(float(x)) for x in c)))
PY

for m in "${modos[@]}"; do
  set -- $m
  magick "$tmp/fundo.pfm" "$tmp/fonte.pfm" -compose "$2" -composite \
    -define quantum:format=floating-point -depth 32 "$tmp/out_$1.pfm"
done

{
  echo "# Oráculo ImageMagick para ph2d_blend_mode::apply (modos 0..19), gerado por"
  echo "#   crates/ph2d-blend-mode/tests/oraculo/gera_fixture_imagemagick.sh"
  echo "# Data: 2026-10-11 · $(magick -version | head -1)"
  echo "# Licença (pacman -Qi): imagemagick 7.1.2.32-1.1 = ImageMagick License (permissiva, estilo"
  echo "#   Apache-2.0); o caminho de compose não usa delegados (liblqr GPL-3 só serve -liquid-rescale)."
  echo "#   Corrido como oráculo, nunca linkado nem lido."
  echo "# Comando por modo: magick fundo.pfm fonte.pfm -compose <IM> -composite"
  echo "#   -define quantum:format=floating-point -depth 32 out.pfm   (Q16-HDRI, PFM float32 cru, opaco)"
  echo "# Formato: 'cor <i> r g b' = a grelha; '<IM> <codigo> <i_fundo> <i_fonte> r g b' = resultado."
  cat "$tmp/grelha.txt"
  python3 -I - "$tmp" "${modos[@]}" <<'PY'
import struct, sys
tmp, modos = sys.argv[1], [m.split() for m in sys.argv[2:]]
n = 7
def f(x):
    s = "%.7f" % x
    s = s.rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s
for codigo, nome in modos:
    d = open("%s/out_%s.pfm" % (tmp, codigo), "rb").read()
    head, rest = d.split(b"\n", 3)[:3], d.split(b"\n", 3)[3]
    assert head[1] == b"%d 1" % (n * n), head
    v = struct.unpack("<%df" % (n * n * 3), rest)
    for k in range(n * n):
        print(nome, codigo, k // n, k % n, *(f(c) for c in v[3 * k:3 * k + 3]))
PY
} > "$saida"

echo "escrito: $saida ($(wc -c < "$saida") bytes, $(grep -vc '^#\|^cor' "$saida") linhas de dados)"
