"""Régua de UM passo do oráculo «snippet aninhado» (VS Code), sem ler o fonte — só a foto e a área de transferência.

Uso:
  python3 -I estado.py limpa                       # esvazia a CLIPBOARD (toma-a e sai: o dono passa a None)
  python3 -I estado.py le <foto.png> <y_linha> <x0> <largura_col> <ncols>
Imprime UMA linha:
  sel=«…» (a CLIPBOARD depois do Ctrl+C; <vazia> = nada seleccionado, porque editor.emptySelectionClipboard=false)
  · barra=«Ln a, Col b (n selected)» (OCR da barra de estado)
  · marcas=… o fundo de cada coluna da linha (cor MAIS FREQUENTE na célula, a tinta da letra é minoria),
    em corridas «c0-1:#rrggbb»; a borda do $0 (1 px) não muda a moda — por isso também
  · bordas=… as colunas com uma risca VERTICAL clara (o traço do $0 / cursor) no meio da altura da linha.
"""
import re
import subprocess
import sys
import tkinter
from collections import Counter


def clip_limpa():
    r = tkinter.Tk()
    r.withdraw()
    r.clipboard_clear()
    r.clipboard_append("")
    r.update()
    r.destroy()


def clip_le():
    r = tkinter.Tk()
    r.withdraw()
    try:
        s = r.clipboard_get()
    except tkinter.TclError:
        s = None
    r.destroy()
    return "<vazia>" if not s else s


def barra(foto, tmp):
    subprocess.run(["magick", foto, "-crop", "900x24+700+974", "+repage", "-negate", "-resize", "300%", tmp], check=True)
    t = subprocess.run(["tesseract", tmp, "-", "--psm", "7"], capture_output=True, text=True).stdout
    t = " ".join(t.split())
    m = re.search(r"Ln\s*(\d+)\s*,\s*Col\s*(\d+)(\s*\((\d+)\s*selected\))?", t)
    return (m.group(0) if m else f"? OCR={t!r}")


def marcas(foto, y, x0, cw, n):
    from PIL import Image
    im = Image.open(foto).convert("RGB")
    p = im.load()
    top, bot = y - 8, y + 8
    cols, riscas = [], []
    for c in range(n):
        a, b = int(round(x0 + cw * c)), int(round(x0 + cw * (c + 1)))
        cnt = Counter(p[i, j] for i in range(a, b) for j in range(top, bot + 1))
        cols.append("#%02x%02x%02x" % cnt.most_common(1)[0][0])
    # risca vertical: uma coluna x cujos 19 px da ALTURA INTEIRA da linha (y-10..y+8) têm a MESMA cor e mais clara
    # que o fundo à esquerda (medido: a marca do $0 = 2 px × 19 px #313232 sobre #121314; os parênteses têm 13 px;
    # o cursor = 2 px claros). Imprime a cor e a fronteira de coluna mais próxima (c = antes do carácter c).
    for x in range(int(x0) - 3, int(x0 + cw * n)):
        col = [p[x, j] for j in range(y - 10, y + 9)]
        fundo = sum(p[x - 6, y - 10])
        ss = [sum(q) for q in col]
        if min(ss) - fundo > 40 and max(ss) - min(ss) < 20:
            riscas.append((x, "#%02x%02x%02x" % col[9]))
    runs, i = [], 0
    while i < n:
        k = i
        while k + 1 < n and cols[k + 1] == cols[i]:
            k += 1
        runs.append(f"c{i}" + (f"-{k}" if k > i else "") + f":{cols[i]}")
        i = k + 1
    grupos = []
    for x, c in riscas:
        if grupos and x - grupos[-1][1] == 1:
            grupos[-1][1] = x
        else:
            grupos.append([x, x, c])
    # 1 px = a borda esquerda da caixa de um buraco realçado (o fundo de referência cai fora dela): não é risca
    return " ".join(runs), (" ".join(
        f"x{a}-{b}{c}(fronteira c{round((a + 1 - x0) / cw)})" for a, b, c in grupos if b > a) or "nenhuma")


if __name__ == "__main__":
    if sys.argv[1] == "limpa":
        clip_limpa()
        sys.exit(0)
    foto, y, x0, cw, n = sys.argv[2], int(sys.argv[3]), float(sys.argv[4]), float(sys.argv[5]), int(sys.argv[6])
    s = clip_le().replace("\n", "⏎")
    b = barra(foto, foto + ".ocr.png")
    m, r = marcas(foto, y, x0, cw, n)
    print(f"sel=«{s}» · barra=«{b}» · marcas={m} · bordas={r}")
