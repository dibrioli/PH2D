# Sobreposições para olho humano + medida de cobertura do contorno da Godot contra a máscara.
#     python3 -I docs/GameEngineFuncional/oraculo_contorno/sobrepoe.py <contorno_godot.json> <pasta_das_alfas_da_uniao>
# Painéis: limiar 0.5, epsilon 0.5 | 2 | 4. Linha branca = contorno; cores = peças convexas.
import json, os, sys
import numpy as np
from PIL import Image, ImageDraw

AQUI = os.path.dirname(os.path.abspath(sys.argv[0]))
d = json.load(open(sys.argv[1]))
ALFAS = sys.argv[2]
K = 5
CORES = [(230, 25, 75), (60, 180, 75), (0, 130, 200), (245, 130, 48), (145, 30, 180), (70, 240, 240), (240, 50, 230), (210, 245, 60)]

def imagem(caso):
    if caso.startswith("heroi"):
        return Image.open(os.path.join(ALFAS, "_alfa_" + caso + ".png")).convert("RGBA")
    return Image.open(os.path.join(AQUI, "entradas", caso + ".png")).convert("RGBA")

def cobertura(img, g):
    a = np.asarray(img)[..., 3] >= 128
    # Centro de cada píxel dentro do polígono (par-ímpar, vectorizado): sem o viés de borda do ImageDraw.
    h, w = a.shape
    cy, cx = np.mgrid[0:h, 0:w] + 0.5
    poli = np.zeros_like(a)
    for p in g["lista"]:
        v = np.array(p["pontos"], float)
        dentro = np.zeros_like(a)
        for (x0, y0), (x1, y1) in zip(v, np.roll(v, -1, axis=0)):
            cruza = (y0 > cy) != (y1 > cy)
            with np.errstate(divide="ignore", invalid="ignore"):
                xi = x0 + (cy - y0) * (x1 - x0) / (y1 - y0)
            dentro ^= cruza & (cx < xi)
        poli |= dentro
    return int((a & ~poli).sum()), int((poli & ~a).sum()), int(a.sum())

for c in d["casos"]:
    img = imagem(c["caso"])
    for g in c["grade"]:
        fora, vazio, tot = cobertura(img, g)
        print(f'{c["caso"]:24s} a={g["limiar_alfa"]} e={g["epsilon"]:3} opacos={tot:5d} fora_do_contorno={fora:4d} ({100*fora/tot:.1f}%) vazio_coberto={vazio:4d} ({100*vazio/tot:.1f}%)')

for nome in ["heroi_uniao_corrida_8", "sint_anel", "sint_moeda_ilha"]:
    c = next(x for x in d["casos"] if x["caso"] == nome)
    img = imagem(nome)
    w, h = img.size
    painel = []
    for g in [x for x in c["grade"] if x["limiar_alfa"] == 0.5]:
        fundo = Image.new("RGBA", (w * K, h * K), (40, 40, 48, 255))
        fundo.alpha_composite(img.resize((w * K, h * K), Image.NEAREST))
        cam = Image.new("RGBA", fundo.size, (0, 0, 0, 0))
        dr = ImageDraw.Draw(cam)
        i = 0
        for p in g["lista"]:
            for q in p["pecas"]:
                cor = CORES[i % len(CORES)]
                dr.polygon([(x * K, y * K) for x, y in q], fill=cor + (90,), outline=cor + (255,))
                i += 1
        for p in g["lista"]:
            pts = [(x * K, y * K) for x, y in p["pontos"]]
            dr.line(pts + [pts[0]], fill=(255, 255, 255, 255), width=2)
            for x, y in pts:
                dr.ellipse([x - 3, y - 3, x + 3, y + 3], fill=(255, 255, 255, 255))
        fundo.alpha_composite(cam)
        ImageDraw.Draw(fundo).text((6, 6), f'eps {g["epsilon"]}: {g["vertices_total"]} v, {g["pecas_convexas_total"]} pecas', fill=(255, 255, 0, 255))
        painel.append(fundo)
    out = Image.new("RGBA", (w * K * 3 + 20, h * K), (0, 0, 0, 255))
    for j, p in enumerate(painel):
        out.paste(p, (j * (w * K + 10), 0))
    out.convert("RGB").save(os.path.join(AQUI, "sobreposicao_" + nome + ".png"))
