#!/usr/bin/env python3
"""O VectorCraft como ORÁCULO do kerning (prompt 49, W3).

Da raiz da worktree:
    python3 -I "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc_kerning.py" \\
        > crates/ph2d-vec-text/tests/fixtures/vectorcraft_kerning.tsv

Para cada palavra e peso, o `vectorcraft-cli convert --outline-text` desenha a palavra a
`font-size = unitsPerEm` (1 unidade SVG = 1 unidade de design) e cada PREFIXO dela. Os contornos
que o prefixo k acrescenta ao k-1 são o glifo k; o pen dele é o `xmin` exacto desses contornos
menos o `xmin` do mesmo glifo na fonte (fontTools, só para ler o ficheiro — o oráculo é o VC).
O kerning entre k-1 e k está no prefixo k, e o que vem depois de k não move o pen de k.

Saída: `palavra  wght  pens` — pens em unidades de design, relativos ao 1º glifo, `,`-separados.
"""
import datetime, json, os, re, subprocess, sys, tempfile
from fontTools.ttLib import TTFont
from fontTools.pens.boundsPen import BoundsPen

VC = os.environ.get("VC", os.path.expanduser("~/Apps/vectorcraft/squashfs-root/usr/bin/vectorcraft-cli"))
FONT = "crates/ph2d-text/fonts/InterVariable.ttf"
WORDS = ["AV", "To", "Wa", "fi", "Ty", "LT", "Yo", "AVATAR", "WAVE", "Kerning", "minimum", "Typography"]
# Só 400: a Inter do VC é ESTÁTICA (Regular, SemiBold…) e não a nossa variável — `font-weight=700`
# sai `status: substitute`, resolvido em SemiBold (medido 11/10), e outra fonte não é oráculo.
WEIGHTS = [400]
X0, Y0 = 1000.0, 3000.0
TMP = tempfile.mkdtemp(prefix="oraculo_vc_kern_")
TOK = re.compile(r"[MLCQZmlcqz]|-?\d+(?:\.\d+)?(?:e-?\d+)?")


def contours(d):
    """Os subcaminhos de um `d` absoluto (M/L/C/Q/Z, o que o VC escreve), como segmentos."""
    out, cur, pos, cmd, toks = [], None, None, None, TOK.findall(d)
    i = 0
    while i < len(toks):
        t = toks[i]
        if t.isalpha():
            cmd = t
            i += 1
            if cmd in "Zz":
                cur = None
            continue
        n = {"M": 2, "L": 2, "C": 6, "Q": 4}[cmd.upper()]
        v = [float(x) for x in toks[i:i + n]]
        i += n
        if cmd == "M":
            cur = []
            out.append(cur)
            pos = (v[0], v[1])
            cur.append(("M", [pos]))
            cmd = "L"
        else:
            pts = [(v[j], v[j + 1]) for j in range(0, n, 2)]
            cur.append((cmd, [pos] + pts))
            pos = pts[-1]
    return out


def seg_xmin(kind, pts):
    """`xmin` EXACTO de um segmento (extremos da derivada), não o das alças."""
    xs = [p[0] for p in (pts[0], pts[-1])]
    if kind == "C":
        a, b, c, e = (p[0] for p in pts)
        A, B, C = -a + 3 * b - 3 * c + e, 2 * (a - 2 * b + c), b - a
        roots = []
        if abs(A) > 1e-12:
            disc = B * B - 4 * A * C
            if disc >= 0:
                roots = [(-B + s * disc ** 0.5) / (2 * A) for s in (1, -1)]
        elif abs(B) > 1e-12:
            roots = [-C / B]
        for t in roots:
            if 0 < t < 1:
                u = 1 - t
                xs.append(u ** 3 * a + 3 * u * u * t * b + 3 * u * t * t * c + t ** 3 * e)
    elif kind == "Q":
        a, b, c = (p[0] for p in pts)
        den = a - 2 * b + c
        if abs(den) > 1e-12:
            t = (a - b) / den
            if 0 < t < 1:
                xs.append((1 - t) ** 2 * a + 2 * (1 - t) * t * b + t * t * c)
    return min(xs)


def outline(word, wght, upem):
    src = os.path.join(TMP, f"{wght}_{len(word)}.svg")
    out = src[:-4] + "_o.svg"
    with open(src, "w") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="40000" height="4000" viewBox="0 0 40000 4000">'
                f'<text x="{X0}" y="{Y0}" font-family="Inter" font-weight="{wght}" font-size="{upem}">{word}</text></svg>')
    p = subprocess.run([VC, "convert", src, out, "--outline-text"], capture_output=True, text=True, timeout=120)
    if p.returncode != 0:
        sys.exit(f"vectorcraft-cli falhou em {word!r}: {p.stderr}")
    info = json.loads(subprocess.run([VC, "info", src], capture_output=True, text=True, timeout=120).stdout)
    if any(f["status"] != "exact" for f in info["fonts"]):
        sys.exit(f"{word!r} a {wght}: o VC SUBSTITUIU a fonte ({info['fonts']}) — não é a mesma régua")
    svg = open(out).read()
    cs = []
    for d in re.findall(r'<path\b[^>]*\bd="([^"]*)"', svg):
        cs += contours(d)
    return cs


def main():
    font = TTFont(FONT)
    upem = font["head"].unitsPerEm
    cmap = font.getBestCmap()
    print(f"# origem: {subprocess.run([VC, '--version'], capture_output=True, text=True).stdout.strip() or 'vectorcraft-cli'}"
          f" (storytold, MIT OU Apache-2.0) — {VC}")
    print(f'# comando: python3 -I "docs/Vector Module/ferramentas/vectorcraft_2026-10-11/oraculo_vc_kerning.py"')
    print(f"# fonte: {FONT} ({font['name'].getDebugName(5)}), font-size = unitsPerEm = {upem}; gravado {datetime.date.today()}")
    print("# palavra\twght\tpens (unidades de design, relativos ao 1.º glifo)")
    for wght in WEIGHTS:
        gs = font.getGlyphSet(location={"wght": wght, "opsz": 14})
        lsb = {}
        for ch in set("".join(WORDS)):
            bp = BoundsPen(gs)
            gs[cmap[ord(ch)]].draw(bp)
            lsb[ch] = bp.bounds[0]
        alone = {ch: len(outline(ch, wght, upem)) for ch in set("".join(WORDS))}
        for word in WORDS:
            pens, prev = [], 0
            for k in range(1, len(word) + 1):
                cs = outline(word[:k], wght, upem)
                new = cs[prev:]
                # Um prefixo que não soma os contornos do glifo sozinho trocou glifos (ligadura,
                # alternativa contextual): o pen de k já não é medível por esta régua.
                if len(new) != alone[word[k - 1]]:
                    sys.exit(f"{word[:k]!r}: {len(cs)} contornos, esperava {prev} + {alone[word[k-1]]} (ligadura?)")
                prev = len(cs)
                xmin = min(seg_xmin(kind, pts) for c in new for kind, pts in c)
                pens.append(xmin - lsb[word[k - 1]] - X0)
            print(f"{word}\t{wght}\t" + ",".join(f"{p:g}" for p in pens))


main()
