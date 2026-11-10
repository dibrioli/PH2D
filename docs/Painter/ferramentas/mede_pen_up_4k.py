#!/usr/bin/env python3
"""Resumo do mede_pen_up_4k.sh: depois de cada mouse-up, quanto a LINHA PRINCIPAL trabalha por fatia de 50 ms, e as
funções (inclusivas, da pilha) que dominam o segundo depois do Up — contra o mesmo segundo ANTES do traço (base).
uso: python3 docs/Painter/ferramentas/mede_pen_up_4k.py target/mede-pen-up/<rotulo>"""
import collections
import subprocess
import sys


def matriz(pasta, saida=None):
    """`--matriz <pasta> [saida]`: a tabela da matriz do .sh (uma linha por lançamento) e, com `saida`, o jsonl e os
    quadros crus (`<saida>/quadros/<rotulo>.csv`: t desde o mousedown, intervalo, quadro, em ms)."""
    import glob
    import json
    import os

    tck = os.sysconf("SC_CLK_TCK")
    lim_quadro, lim_extra = 1000 / 60, 2 * 1000 / 60
    linhas = []
    for meta_f in sorted(glob.glob(os.path.join(pasta, "*.meta"))):
        tag = meta_f[:-5]
        meta = json.loads(open(meta_f).read())
        marcas = {}
        for linha in open(tag + ".marcas"):
            p = linha.split()
            if len(p) >= 2:
                marcas[p[0]] = (int(p[1]), int(p[2]) if len(p) > 2 else None)
        placa = open(tag + ".placa").read().strip() if os.path.exists(tag + ".placa") else ""
        quadros, agua, ultimo = [], [], 0
        for linha in open(tag + ".log", errors="replace"):
            if linha.startswith("[quadro-lento]"):
                d = dict(kv.split("=") for kv in linha.split()[1:])
                ultimo = int(d["parede"])
                quadros.append((ultimo, float(d["intervalo"]), float(d["quadro"])))
            elif "TAXA DA AGUA" in linha and ultimo:
                p = linha.split("TAXA DA AGUA")[1].split()
                agua.append((ultimo, float(p[0]), float(p[5].rstrip("s)"))))
        r = dict(meta, rotulo=os.path.basename(tag), placa=placa)
        if "parado1_fim" not in marcas or not quadros:
            r["erro"] = "sem janela ou sem quadros"
            linhas.append(r)
            continue

        def janela(a, b):
            return [q[1] for q in quadros if a < q[0] <= b]

        def cpu(a, b):
            (ta, ca), (tb, cb) = marcas[a], marcas[b]
            return round(100 * (cb - ca) / tck / ((tb - ta) / 1e9), 1)

        def pct(v, f):
            v = sorted(v)
            return round(v[min(len(v) - 1, int(f * len(v)))], 1) if v else None

        down, ti, tf, up = (marcas[k][0] for k in ("down", "traco_ini", "traco_fim", "up"))
        cz = marcas["ctrlz"][0]
        # O 1.º dab sai com o 1.º MOVIMENTO (o mousedown parado não pinta): o toque vai até 150 ms depois dele.
        traco = janela(ti + 150_000_000, tf)

        def pico(a, b):
            q = max(((i, t) for t, i, _ in quadros if a < t <= b), default=(0, a))
            return round(q[0], 1), round((q[1] - a) / 1e6)

        r.update(
            base_p50=pct(janela(marcas["parado0_ini"][0], marcas["parado0_fim"][0]), 0.5),
            toque=round(max(janela(down, ti + 150_000_000), default=0), 1),
            soltar_1s_em_ms=pico(up, up + 1_000_000_000)[1],
            ctrlz_em_ms=pico(cz, cz + 1_000_000_000)[1],
            traco_p50=pct(traco, 0.5),
            traco_p95=pct(traco, 0.95),
            traco_n=len(traco),
            soltar=round(max(janela(up, up + 200_000_000), default=0), 1),
            soltar_1s=round(max(janela(up, up + 1_000_000_000), default=0), 1),
            ctrlz=round(max(janela(cz, cz + 1_000_000_000), default=0), 1),
            parado_cpu=cpu("parado0_ini", "parado0_fim"),
            parado_cpu_depois=cpu("parado1_ini", "parado1_fim"),
            cpu_3s_depois_do_up=cpu("up", "wet_fim"),
        )
        # A taxa da água: o resumo [frame] (janela de 120 quadros) cujo fim fica mais perto de up + 3 s.
        perto = [a for a in agua if up + 1_500_000_000 < a[0] <= up + 6_000_000_000]
        if perto:
            a = min(perto, key=lambda a: abs(a[0] - up - 3_000_000_000))
            r["agua_hz"], r["agua_janela_s"] = a[1], a[2]
        linhas.append(r)
        if saida:
            os.makedirs(os.path.join(saida, "quadros"), exist_ok=True)
            with open(os.path.join(saida, "quadros", r["rotulo"] + ".csv"), "w") as f:
                f.write("t_desde_down_ms,intervalo_ms,quadro_ms\n")
                for t, i, q in quadros:
                    if down - 1_000_000_000 <= t <= marcas["parado1_ini"][0]:
                        f.write(f"{(t - down) / 1e6:.1f},{i:.1f},{q:.1f}\n")
    if saida:
        with open(os.path.join(saida, "matriz.jsonl"), "w") as f:
            for r in linhas:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")

    def marca(v, ok):
        return "—" if v is None else (f"{v}" if ok else f"**{v} ✗**")

    print("| pincel | painel | tela | fundo | base ms | toque ms | traço p50/p95 ms | soltar ms | +1 s máx ms | Ctrl+Z ms "
          "| parado CPU % (antes/depois) | Wet img/s |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for r in linhas:
        if "erro" in r:
            print(f"| {r['pincel']} | {r['painel']} | {r['tela']} | {r['fundo']} | {r['erro']} |||||||")
            continue
        wet = r.get("agua_hz") if r["painel"] == "wet" else None
        print(
            f"| {r['pincel']} | {r['painel']} | {r['tela']} | {r['fundo']} | {r['base_p50']} "
            f"| {marca(r['toque'], r['toque'] <= lim_extra)} "
            f"| {r['traco_p50']} / {marca(r['traco_p95'], (r['traco_p95'] or 0) <= lim_quadro)} "
            f"| {marca(r['soltar'], r['soltar'] <= lim_extra)} | {marca(r['soltar_1s'], r['soltar_1s'] <= lim_extra)} "
            f"| {marca(r['ctrlz'], r['ctrlz'] <= lim_extra)} "
            f"| {marca(r['parado_cpu'], r['parado_cpu'] <= 5)} / {marca(r['parado_cpu_depois'], r['parado_cpu_depois'] <= 5)} "
            f"| {'—' if wet is None else marca(wet, wet >= 10)} |"
        )


if len(sys.argv) > 1 and sys.argv[1] == "--matriz":
    matriz(sys.argv[2], sys.argv[3] if len(sys.argv) > 3 else None)
    sys.exit(0)

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
