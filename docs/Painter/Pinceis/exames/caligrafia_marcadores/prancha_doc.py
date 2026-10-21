"""prancha_doc.py <saida.png> <chave.json> <semente> <nome=dir> … — a prancha CEGA de um pincel sem oráculo: por cena
(gestos, tamanhos, sobre, tremor) as fotos A de cada pincel numa ordem sorteada, rotuladas só 1 2 3."""
import json, random, sys
from PIL import Image, ImageDraw
saida, chave, semente = sys.argv[1], sys.argv[2], int(sys.argv[3])
pinceis = [a.split("=", 1) for a in sys.argv[4:]]
rnd = random.Random(semente)
cenas = ["gestos", "tamanhos", "sobre", "tremor"]
W, H, T = 500, 320, 22
F = Image.new("RGB", (W * len(pinceis) + 10 * (len(pinceis) - 1), (H + T) * len(cenas)), "white")
d = ImageDraw.Draw(F)
k = {}
for i, c in enumerate(cenas):
    ordem = pinceis[:]
    rnd.shuffle(ordem)
    k[c] = {}
    for j, (nome, pasta) in enumerate(ordem):
        im = Image.open(f"{pasta}/{c}_A.ppm").convert("RGB").resize((W, H))
        x, y = j * (W + 10), i * (H + T)
        F.paste(im, (x, y + T))
        d.rectangle([x, y + T, x + W - 1, y + T + H - 1], outline=(150, 150, 150))
        d.text((x + 4, y + 4), f"{c} · {j + 1}", fill=(0, 0, 0))
        k[c][str(j + 1)] = nome
F.save(saida)
json.dump(k, open(chave, "w"), indent=1)
