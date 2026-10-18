#!/usr/bin/env python3
"""C31 — o KDE aplica uma curva de pressão ANTES do tablet-v2? Compara, traço a traço e amostra a amostra, a
pressão que o libinput lê da mesa (`sudo libinput debug-events`, 2 casas) com a que o PH2D recebe do compositor
(`PH2D_PEN_LOG`, 4 casas), gravadas na MESMA sessão (`c31_sessao.sh`).

uso: c31_compara.py <libinput.txt[.zst]> <ph2d_pen.log>

Os traços alinham-se pela ORDEM (o n-ésimo toque de um é o n-ésimo do outro); dentro de um traço, cada amostra
do PH2D casa com a do libinput de tempo mais perto (relógios diferentes: o desvio é o do 1.º toque de cada traço).
Sem curva, |PH2D − libinput| ≤ 0,005 (o arredondamento a 2 casas do libinput). Com curva, a tabela por faixa de
pressão mostra-a — e a recta de ajuste diz se ela é linear (tecto/piso) ou não.
"""
import re
import subprocess
import sys


def linhas(caminho):
    if caminho.endswith(".zst"):
        return subprocess.run(["zstdcat", caminho], capture_output=True, text=True, check=True).stdout.splitlines()
    return open(caminho, encoding="utf8", errors="replace").read().splitlines()


def tracos_libinput(caminho):
    """[(t_s, p)] por toque: entre `TABLET_TOOL_TIP … down` e `… up`."""
    tracos, atual = [], None
    for l in linhas(caminho):
        m = re.search(r"\+(\d+\.\d+)s", l)
        if not m:
            continue
        t = float(m.group(1))
        p = re.search(r"pressure: *([\d.]+)", l)
        if "TABLET_TOOL_TIP" in l and " down" in l:
            atual = []
            tracos.append(atual)
        if atual is not None and p and ("TABLET_TOOL_AXIS" in l or "TABLET_TOOL_TIP" in l):
            atual.append((t, float(p.group(1))))
        if "TABLET_TOOL_TIP" in l and " up" in l:
            atual = None
    return [t for t in tracos if t]


def tracos_ph2d(caminho):
    """[(t_s, p)] por toque: as amostras com fase de contacto (`down`/`move`) até à `up`."""
    tracos, atual = [], None
    for l in linhas(caminho):
        m = re.match(r"(\d+) (\S+) (\S+) [-\d.]+/[-\d.]+ pressure: ([\d.]+)", l)
        if not m:
            continue
        t, fase, p = int(m.group(1)) / 1000.0, m.group(3).lower(), float(m.group(4))
        if fase == "down":
            atual = []
            tracos.append(atual)
        if atual is not None and fase in ("down", "move"):
            atual.append((t, p))
        if fase == "up":
            atual = None
    return [t for t in tracos if t]


def main():
    li, ph = tracos_libinput(sys.argv[1]), tracos_ph2d(sys.argv[2])
    print(f"toques: libinput {len(li)} · PH2D {len(ph)}")
    pares = []
    for a, b in zip(li, ph):
        d = b[0][0] - a[0][0]
        for tb, pb in b:
            ta, pa = min(a, key=lambda s: abs(s[0] + d - tb))
            pares.append((pa, pb))
    if not pares:
        print("nenhum par: grave os dois ficheiros na mesma sessão")
        return
    erro = [abs(pb - pa) for pa, pb in pares]
    erro.sort()
    n = len(erro)
    print(f"amostras {n}: |PH2D − libinput| p50 {erro[n // 2]:.4f} · p95 {erro[int(0.95 * n)]:.4f} · máx {erro[-1]:.4f}")
    print("faixa libinput → PH2D (média)")
    for k in range(10):
        f = [pb for pa, pb in pares if k / 10 <= pa < (k + 1) / 10]
        if f:
            print(f"  {k / 10:.1f}–{(k + 1) / 10:.1f}: {sum(f) / len(f):.3f} ({len(f)})")
    xs, ys = [p for p, _ in pares], [q for _, q in pares]
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    a = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / max(sxx, 1e-12)
    print(f"recta: PH2D = {a:.4f}·libinput + {my - a * mx:.4f}")
    print("VEREDICTO:", "sem curva do KDE (a do libinput chega igual)" if erro[int(0.95 * n)] <= 0.006 else
          "o KDE MUDA a pressão — ver a tabela")


main()
