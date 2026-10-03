#!/usr/bin/env bash
# O smoke do dono do ADR-0177: o MESMO documento do Painter (o exemplo `camadas_antes_e_depois` da
# ph2d-tool-painter — manchas translúcidas, uma sombra em Multiplicar, um Desfoque e umas Curvas por
# cima, um quarto do fundo transparente) composto pelo compositor de produção nos DOIS estados:
#   antes  = o merge-base da linha (a lei velha: as camadas juntam-se em LUZ)
#   depois = esta árvore (a lei nova: em tons de ecrã)
# e gravado em PNG: antes.png, depois.png, lado_a_lado.png e diferenca.png (|depois − antes| × 4).
#
#   bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/smoke_camadas_antes_e_depois.sh [pasta] [base]
#
# O «antes» corre num worktree TEMPORÁRIO no merge-base, com o target DELE (partilhar o target entre
# worktrees troca os .rlib — memória `feedback_sharing_a_target_dir_between_worktrees…`); é removido
# no fim, sucesso ou não.
set -euo pipefail
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=$(realpath -m "${1:-$RAIZ/target/smoke_camadas}")
BASE=${2:-$(git -C "$RAIZ" merge-base main HEAD)}
EX=crates/ph2d-tool-painter/examples/camadas_antes_e_depois.rs
mkdir -p "$SAIDA"
cd "$RAIZ"

cargo run -q -p ph2d-tool-painter --profile smoke --example camadas_antes_e_depois -- "$SAIDA/depois.rgba"

VELHO="$RAIZ/target/smoke_camadas_velho"
rm -rf "$VELHO"
git worktree prune
git worktree add -q --detach "$VELHO" "$BASE"
trap 'git -C "$RAIZ" worktree remove --force "$VELHO" 2>/dev/null || rm -rf "$VELHO"; git -C "$RAIZ" worktree prune' EXIT
mkdir -p "$VELHO/crates/ph2d-tool-painter/examples"
cp "$EX" "$VELHO/$EX"
(cd "$VELHO" && cargo run -q -p ph2d-tool-painter --profile smoke --example camadas_antes_e_depois -- "$SAIDA/antes.rgba")

python3 - "$SAIDA" "$BASE" "$(git rev-parse --short HEAD)" <<'PY'
import sys
from PIL import Image, ImageChops, ImageDraw
pasta, base, cabeca = sys.argv[1:4]
W, H = 640, 400
img = {n: Image.frombytes('RGBA', (W, H), open(f'{pasta}/{n}.rgba', 'rb').read()) for n in ('antes', 'depois')}
for n, i in img.items():
    i.save(f'{pasta}/{n}.png')

def sobre_xadrez(i):
    x = Image.new('RGBA', (W, H))
    d = ImageDraw.Draw(x)
    for yy in range(0, H, 16):
        for xx in range(0, W, 16):
            c = 205 if (xx // 16 + yy // 16) % 2 else 245
            d.rectangle([xx, yy, xx + 15, yy + 15], fill=(c, c, c, 255))
    return Image.alpha_composite(x, i)

lado = Image.new('RGBA', (2 * W + 20, H + 30), (255, 255, 255, 255))
lado.paste(sobre_xadrez(img['antes']), (0, 30))
lado.paste(sobre_xadrez(img['depois']), (W + 20, 30))
d = ImageDraw.Draw(lado)
d.text((8, 8), f'ANTES - as camadas juntavam-se em luz ({base[:9]})', fill=(0, 0, 0, 255))
d.text((W + 28, 8), f'DEPOIS - em tons de ecra, como o Photoshop/Krita ({cabeca})', fill=(0, 0, 0, 255))
lado.save(f'{pasta}/lado_a_lado.png')
dif = ImageChops.difference(img['antes'], img['depois']).point(lambda v: min(255, v * 4))
dif.putalpha(255)
dif.save(f'{pasta}/diferenca.png')
a, b = img['antes'].tobytes(), img['depois'].tobytes()
mud = sum(1 for k in range(0, len(a), 4) if a[k:k + 4] != b[k:k + 4])
pior = max(abs(x - y) for x, y in zip(a, b))
print(f'{pasta}: {mud} de {W * H} pixeis mudaram; pior canal {pior} degraus')
PY
