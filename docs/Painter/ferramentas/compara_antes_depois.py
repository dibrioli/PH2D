"""B08c: antes → depois por pincel × tela × fundo × fase. argv: ANTES.md DEPOIS_tabela.md"""
import re, sys

def secoes(texto):
    sec, out = None, []
    for l in texto.splitlines():
        if l.startswith("## ") or l.startswith("### "):
            sec = l
        if l.startswith("| ") and not l.startswith("| pincel") and l.count("|") == 13:
            c = [x.strip().replace("**", "").replace(" ✗", "") for x in l.strip("|").split("|")]
            out.append((sec, c))
    return out

def preferida(sec, tela, fundo):
    if tela == "3840x2160" and fundo == "blocos":
        return "continuação —" in sec  # load ~1 (o B08b 4K blocos era load 11)
    if fundo == "branco":
        return sec.startswith("## B08b — o complemento")
    return sec.startswith("## A matriz")

antes = {}
for sec, c in secoes(open(sys.argv[1]).read()):
    k = tuple(c[:4])
    if preferida(sec or "", c[2], c[3]):
        antes[k] = c
depois = {tuple(c[:4]): c for _, c in secoes(open(sys.argv[2]).read())}

def num(s):
    m = re.findall(r"[\d.]+", s)
    return [float(x) for x in m]

def ok(fase, v):
    if fase == "traço":
        return v[-1] <= 16.6
    if fase in ("toque", "soltar", "+1 s", "Ctrl+Z"):
        return v[0] <= 33.3
    if fase == "parado":
        return max(v) <= 5
    if fase == "Wet":
        return v[0] >= 10
FASES = [("toque", 5), ("traço", 6), ("soltar", 7), ("+1 s", 8), ("Ctrl+Z", 9), ("parado", 10), ("Wet", 11)]
print("| pincel | painel | tela | fundo | " + " | ".join(f for f, _ in FASES) + " |")
print("|---|---|---|---|" + "---|" * len(FASES))
tot = {f: [0, 0, 0] for f, _ in FASES}  # passa depois, passava antes, n
for k in sorted(depois, key=lambda k: (k[2], k[3], k[0], k[1])):
    d = depois[k]; a = antes.get(k)
    cel = []
    for f, i in FASES:
        dv = num(d[i])
        if not dv:
            cel.append("—"); continue
        av = num(a[i]) if a else []
        dok = ok(f, dv); aok = ok(f, av) if av else None
        tot[f][2] += 1; tot[f][0] += dok; tot[f][1] += bool(aok)
        s = (a[i] if a else "?") + " → " + d[i] + (" ✓" if dok else " ✗")
        cel.append(s)
    print("| " + " | ".join(k) + " | " + " | ".join(cel) + " |")
print()
print("| fase | passam antes | passam depois | células |")
print("|---|---|---|---|")
for f, _ in FASES:
    print(f"| {f} | {tot[f][1]} | {tot[f][0]} | {tot[f][2]} |")
