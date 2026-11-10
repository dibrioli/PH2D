#!/usr/bin/env python3
"""Resumo do mede_pen_up_4k.sh: depois de cada mouse-up, quanto a LINHA PRINCIPAL trabalha por fatia de 50 ms, e as
funções (inclusivas, da pilha) que dominam o segundo depois do Up — contra o mesmo segundo ANTES do traço (base).
uso: python3 docs/Painter/ferramentas/mede_pen_up_4k.py target/mede-pen-up/<rotulo>"""
import collections
import subprocess
import sys

base = sys.argv[1]
ups = [int(x) / 1e9 for x in open(base + ".ups").read().split()]
out = subprocess.run(
    ["perf", "script", "-i", base + ".perf", "-F", "pid,tid,time,ip,sym", "--no-demangle"],
    capture_output=True, text=True, errors="replace",
).stdout
amostras = []  # (t, tid, pid, [syms folha→raiz])
cab = None
for linha in out.splitlines():
    if not linha.strip():
        if cab:
            amostras.append((*cab, pilha))
        cab = None
        continue
    p = linha.split()
    if len(p) >= 2 and "/" in p[0] and p[1].endswith(":"):
        if cab:
            amostras.append((*cab, pilha))
        try:
            pid, tid = (int(v) for v in p[0].split("/"))
            t = float(p[1].rstrip(":"))
        except ValueError:
            cab = None
            continue
        cab, pilha = (t, tid, pid), []
        if len(p) >= 4:  # sem cadeia: `ip sym` na mesma linha
            pilha.append(p[3].split("+0x")[0])
    elif cab:
        p = linha.strip().split(None, 1)
        if len(p) == 2:
            pilha.append(p[1].split("+0x")[0])
if cab:
    amostras.append((*cab, pilha))
if not amostras:
    sys.exit("sem amostras")
principal = amostras[0][2]
hz = 997.0
JANELA, FATIA = 1.5, 0.05


def resumo(t0, t1, so_principal=True):
    sel = [a for a in amostras if t0 <= a[0] < t1 and (a[1] == principal or not so_principal)]
    incl = collections.Counter()
    for a in sel:
        for s in set(a[3]):
            incl[s] += 1
    return sel, incl


print(f"linha principal = tid {principal} · {len(amostras)} amostras · {hz:.0f} Hz (1 amostra ≈ 1 ms de CPU)")
total_up = collections.Counter()
for k, u in enumerate(ups):
    sel, incl = resumo(u, u + JANELA)
    fatias = [0] * int(JANELA / FATIA)
    for a in sel:
        fatias[min(int((a[0] - u) / FATIA), len(fatias) - 1)] += 1
    outros = len([a for a in amostras if u <= a[0] < u + JANELA and a[1] != principal])
    print(f"\nUp {k + 1}: principal {len(sel)} ms de CPU em {JANELA * 1000:.0f} ms · outros fios {outros} ms")
    print("  por 50 ms: " + " ".join(f"{v:2d}" for v in fatias))
    total_up.update(incl)
antes = collections.Counter()
for u in ups:
    _, incl = resumo(u + 1.4, u + 2.4)
    antes.update(incl)
print("\nfunções (inclusivo, linha principal), soma dos Ups · a mesma janela 2,4 s antes (parado):")
for s, n in total_up.most_common(45):
    print(f"  {n:6d}  {antes[s]:6d}  {s[:150]}")

# Os quadros LENTOS (PH2D_QUADRO_LENTO_MS) em torno de cada Up, no relógio de parede: o que o perfil de CPU não vê
# (um quadro parado à espera da placa ou de um trinco).
import os
if os.path.exists(base + ".ups_parede"):
    upsp = [int(x) for x in open(base + ".ups_parede").read().split()]
    lentos = []
    for linha in open(base + ".log", errors="replace"):
        if linha.startswith("[quadro-lento]"):
            d = dict(kv.split("=") for kv in linha.split()[1:])
            lentos.append((int(d["parede"]), float(d["intervalo"]), float(d["quadro"])))
    print(f"\nquadros lentos: {len(lentos)} no total")
    for k, u in enumerate(upsp):
        perto = [(round((p - u) / 1e6), i, q) for p, i, q in lentos if -500e6 <= p - u <= 3000e6]
        print(f"  Up {k + 1}: " + (" · ".join(f"+{d}ms {i:.0f}ms(q {q:.0f})" for d, i, q in perto) or "nenhum"))
