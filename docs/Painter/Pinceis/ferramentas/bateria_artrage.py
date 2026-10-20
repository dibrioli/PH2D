#!/usr/bin/env python3
"""A BATERIA do EXAME (`bateria.json`) no ArtRage 6 Demo, pela porta de SCRIPT do próprio app (`.arscript`).

O ArtRage toca scripts de texto UTF-16 (File > Play Script…, também no Demo; GRAVAR é que o Demo recusa): cada
traço é um `<StrokeEvent>` com os pontos EXACTOS e a pressão (`Pr:`) — a porta mais fiel que há, sem rato nem
caneta simulados. O formato foi lido na página oficial «Example Script Code» (artrage.com/manuals/scripts/) e
confirmado por tentativa no próprio programa (o que está aqui é o que ele aceitou e pintou):

  · cabeçalho `Script Feature Flags: 0x000001034` ⇒ pintura NOVA do tamanho do cabeçalho (1000×640) + diálogo;
  · `CID_ToolSelect ToolID` escolhe a ferramenta; `Reset Current Tool` repõe os valores de fábrica dela;
  · `ToolPreset … Tool Data: { … }` carrega um preset: os bytes do bloco 5 do `.prs` da instalação, cada byte b
    escrito como o carácter U+2500+b (a codificação dos blocos binários dos scripts). Lido do prefixo em tempo de
    corrida; ⛔ nunca copiado para o repo;
  · `SetForeColour ParamType: Pixel Value: { 0xFFRRGGBB }` (ARGB) — a cor;
  · `CID_SetSpecificToolSize ParamType: real Value: { s }` — o tamanho; px = k·s com k MEDIDO por ferramenta
    (`calibra`, a tabela vai no README da fixture);
  · `seca` = `Add New Layer` — o ArtRage não tem tempo nem secagem: a tinta numa camada nova não molha nem
    arrasta a de baixo (é a «tinta seca» dele); `simula` não existe (nada evolui) ⇒ foto B ≡ foto A, só se tira A.

uso:
  bateria_artrage.py gera  <bateria.json> <cena> <saida.arscript> --tool 4900 [--preset "Oil Brush/Thick Gloss"]
                           [--base-tool 4900 --base-preset …] [--k 146]
  bateria_artrage.py calibra <saida.arscript> --tool 4900 [--preset …]   (6 verticais s=0,02…0,64 a pressão 1)
  bateria_artrage.py mede  <foto_da_calibracao.png>                     (largura de cada vertical, px)
"""
import argparse
import json
import struct
from pathlib import Path

PRESETS = Path.home() / "Apps/artrage/wineprefix/drive_c/Program Files/Ambient Design/ArtRage 6 Demo/Resources/Content/Tool Presets"
NOMES = {4900: "Oil Brush", 4901: "Pencil", 4902: "Crayon", 4903: "Chalk", 4904: "Felt Pen", 4905: "Palette Knife",
         4908: "Paint Tube", 4913: "Glitter", 4914: "Airbrush", 4915: "Paint Roller", 4916: "Watercolor",
         4917: "Ink Pen", 4918: "Gloop Pen", 4920: "Sticker Spray", 4925: "Custom Brush"}
PERIODO_S = 0.008          # um evento a cada 8 ms (125 Hz, uma mesa digitalizadora comum): o lento = 375 px/s
CAL = [0.02, 0.04, 0.08, 0.16, 0.32, 0.64]


def cabecalho(nome, w, h, flags="0x000001034"):
    return ["<Version>", "    ArtRage Version: ArtRage 3 6", "    ArtRage Build: 6.0.0", "    Professional Edition: Yes",
            "    Script Version: 1", "</Version>", "<Header>", f'    Painting Name: "{nome}"', f"    Painting Width: {w}",
            f"    Painting Height: {h}", "    Painting DPI: 72", f"    Mask Edge Map Width: {w}",
            f"    Mask Edge Map Height: {h}", '    Author Name: ""', f'    Script Name: "{nome}"', '    Comment: ""',
            '    Script Type: ""', f"    Script Feature Flags: {flags}", "</Header>", "<Events>"]


def _pt(x, y, p, extra=""):
    return f"Loc: ({x}, {y})\tPr: {p}\tTi: 1\tRo: 0\tFw: 1\tBt: 0{extra}"


def traco(pts):
    """Um StrokeEvent com os pontos exactos [[x, y, pressão], …] (o molde do script gravado pelo próprio app).

    Pressão 1,0 EXACTA corta o traço do Crayon (s=0,32) aos ~80 pontos e a do Felt Pen (s=0,64); 0,9 e 0,999
    pintam-no inteiro (medido 19/10, `t_cut6`): o ArtRage lê Pr=1 como rato. ⇒ teto 0,999."""
    pts = [[x, y, min(p, 0.999)] for x, y, p in pts]
    x0, y0, p0 = pts[0]
    s = ["<StrokeEvent>", "\t<StrokeHeader>", "\t\tWait: 0.000s\t" + _pt(x0, y0, p0) + "\tRv: NO\tIv: NO\t",
         "\t\tYes\t", "\t\t0x000000000, 0x000000000\t", "\t\tCount:\t3"]
    s += ["\t\t\t" + _pt(x0, y0, p0)] * 3 + ["\t\t" + _pt(0, 0, 1) + "\t"] * 2
    s += ["\t\t" + _pt(x0, y0, p0, "\tDr: (1, 0)\tHd: (0, 1)") + "\t"] * 2 + ["\t</StrokeHeader>"]
    s += [f"\tWait: {PERIODO_S:.3f}s\t" + _pt(x, y, f"{p:.3f}") + "\tRv: NO\tIv: NO" for x, y, p in pts[1:]]
    return s + ["</StrokeEvent>"]


def comando(cid, tipo=None, valor=None):
    s = f"Wait: 0.000s\tEvType: Command\tCommandID: {cid}"
    return s + (f"\tParamType: {tipo}\tValue: {{ {valor} }}" if tipo else "")


def _blocos(b):
    i, out = 0, {}
    while i + 12 <= len(b):
        t, n, _ = struct.unpack_from("<III", b, i)
        out[t] = b[i + 12:i + 12 + n]
        i += 12 + n
    return out


def pincel(tool, preset=None):
    """Escolhe a ferramenta (valores de fábrica) e, se pedido, carrega o preset do `.prs` instalado."""
    nome = NOMES.get(tool, "Tool")
    s = [comando("CID_ToolSelect", "ToolID", f"{tool} ({nome})"), comando("Reset Current Tool")]
    if preset:
        dados = _blocos((PRESETS / f"{preset}.prs").read_bytes())[5]
        assert struct.unpack_from("<I", dados, 0)[0] == tool, f"{preset} não é da ferramenta {tool}"
        txt = "".join(chr(0x2500 + c) for c in dados)
        s += [f"Wait: 0.000s\tEvType: Command\tCommandID: ToolPreset\tToolID: {tool} ({nome})\tTool Data: {{"]
        s += ["\t\t\t\t" + txt[i:i + 64] for i in range(0, len(txt), 64)]
        s += ["    }    // End of tool preset binary data."]
    return s


def argb(rgb):
    return "0xFF%02X%02X%02X" % tuple(rgb)


def tamanho(px, k):
    """px → valor do ArtRage. `k` = «146» (linear, px = k·s) ou «1=5,4=17,16=41» (tabela s=px medida,
    interpolada; acima do último par fica no último s: o teto da ferramenta)."""
    if "=" not in k:
        return px / float(k)
    pares = sorted((float(b), float(a)) for a, b in (t.split("=") for t in k.split(",")))
    if px <= pares[0][0]:
        return pares[0][1] * px / pares[0][0]
    for (p0, s0), (p1, s1) in zip(pares, pares[1:]):
        if px <= p1:
            return s0 + (s1 - s0) * (px - p0) / (p1 - p0)
    return pares[-1][1]


def gera(bat, cena, tool, preset, base_tool, base_preset, k):
    w, h = bat["tela"]
    ops = next(c for c in bat["cenas"] if c["nome"] == cena)["ops"]
    out = cabecalho(f"{cena}", w, h)
    for op in ops:
        o = op["op"]
        if o == "foto":
            break                                    # ArtRage não tem tempo: B ≡ A, a cena acaba na foto A
        if o == "pincel":
            out += pincel(tool, preset) if op["quem"] == "preset" else pincel(base_tool, base_preset)
        elif o == "cor":
            out.append(comando("SetForeColour", "Pixel", argb(op["rgb"])))
        elif o == "traco":
            out.append(comando("CID_SetSpecificToolSize", "real", f"{tamanho(op['size_px'], k):.5f}"))
            out += traco(op["pts"])
        elif o == "seca":
            out.append(comando("Add New Layer"))
        elif o == "simula":
            pass
        else:
            raise SystemExit(f"op desconhecida: {o}")
    return out + ["</Events>"]


def calibra(tool, preset):
    out = cabecalho("calibra", 1000, 640) + pincel(tool, preset) + [comando("SetForeColour", "Pixel", "0xFF1765B3")]
    for i, s in enumerate(CAL):
        x = 90 + i * 165
        out += [comando("CID_SetSpecificToolSize", "real", f"{s}")]
        out += traco([[x, 120 + j * 3, 1.0] for j in range(134)])
    return out + ["</Events>"]


def mede(png):
    import numpy as np
    from PIL import Image
    a = np.asarray(Image.open(png).convert("RGB")).astype(float)
    papel = np.median(a[5:40, :, :].reshape(-1, 3), axis=0)
    tinta = np.abs(a - papel).sum(2)[250:450].mean(0)
    res = []
    for i, s in enumerate(CAL):
        x = 90 + i * 165
        seg = tinta[max(0, x - 82):x + 82]
        larg = int((seg > 0.25 * seg.max()).sum()) if seg.max() > 8 else 0
        res.append({"s": s, "px": larg})
    return res


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("modo", choices=["gera", "calibra", "mede"])
    ap.add_argument("args", nargs="+")
    ap.add_argument("--tool", type=int)
    ap.add_argument("--preset")
    ap.add_argument("--base-tool", type=int)
    ap.add_argument("--base-preset")
    ap.add_argument("--k", default="149")
    a = ap.parse_args()
    if a.modo == "mede":
        print(json.dumps(mede(a.args[0])))
        return
    if a.modo == "calibra":
        linhas, saida = calibra(a.tool, a.preset), a.args[0]
    else:
        bat = json.loads(Path(a.args[0]).read_text())
        bt = a.base_tool or a.tool
        bp = a.base_preset if a.base_tool else (a.base_preset or a.preset)
        linhas, saida = gera(bat, a.args[1], a.tool, a.preset, bt, bp, a.k), a.args[2]
    Path(saida).write_text("\r\n".join(linhas) + "\r\n", encoding="utf-16")


if __name__ == "__main__":
    main()
