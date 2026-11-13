#!/usr/bin/env python3
"""Resumo do mede_pen_up_4k.sh: depois de cada mouse-up, quanto a LINHA PRINCIPAL trabalha por fatia de 50 ms, e as
funções (inclusivas, da pilha) que dominam o segundo depois do Up — contra o mesmo segundo ANTES do traço (base).
uso: python3 docs/Painter/ferramentas/mede_pen_up_4k.py target/mede-pen-up/<rotulo>"""
import collections
import subprocess
import sys

CAMADA = ("camada_aba", "camada_mais", "camada_troca", "camada_volta", "camada_dup", "camada_apaga", "camada_grupo")


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
                d = dict(kv.split("=", 1) for kv in linha.split()[1:] if "=" in kv)
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
        # B08.1: o controlo (SIGSTOP de 250 ms) e trocar de camada (o maior intervalo no segundo depois de cada clique).
        for k in ("controlo",) + CAMADA:
            if k in marcas:
                r[k] = round(max(janela(marcas[k][0], marcas[k][0] + 1_000_000_000), default=0), 1)
        for k in [f"{t}_{f}" for t in ("nova1", "nova2", "baixo1", "volta1") for f in ("toque", "traco", "up")]:
            if k in marcas and k + "_fim" in marcas:  # NOVA=1: o pior quadro de cada fase do traço depois da camada nova
                r[k] = round(max(janela(marcas[k][0], marcas[k + "_fim"][0] + 200_000_000), default=0), 1)
        if "luz" in marcas:  # o slider de luz do Impasto: o pior quadro do arrasto até 1 s depois, e a mediana do arrasto
            r["luz"] = round(max(janela(marcas["luz"][0], marcas["luz_fim"][0]), default=0), 1)
            r["luz_p50"] = pct(janela(marcas["luz"][0], marcas["luz_solta"][0]), 0.5)
        for k, fim in (("abrir", "reabrir"), ("reabrir", "abrir_fim")):  # abrir doc novo / voltar ao guardado
            if k in marcas and fim in marcas:
                r[k] = round(max(janela(marcas[k][0], marcas[fim][0]), default=0), 1)
        if "controlo" in marcas:  # o mesmo critério no segundo ANTES (sem trava): tem de reprovar, senão não distingue
            r["controlo_sem"] = round(max(janela(marcas["controlo"][0] - 1_000_000_000, marcas["controlo"][0]), default=0), 1)
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
    # A régua só vale se viu o controlo: 250 ms de SIGSTOP têm de aparecer como um intervalo de 250 ms a 250 + 2 quadros.
    com = [r for r in linhas if "controlo" in r]
    if com:
        print("\n| pincel | painel | tela | fundo | controlo sem/com trava de 250 ms | aba Layers ms | + camada ms "
              "| trocar ms | voltar ms | duplicar ms | apagar ms | grupo ms | luz máx / p50 ms | abrir doc / voltar ms |")
        print("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")

        def regua(c):
            return c if 250 <= c <= 250 + 2 * lim_extra else f"**{c} ✗ régua**"

        for r in com:
            print(
                f"| {r['pincel']} | {r['painel']} | {r['tela']} | {r['fundo']} "
                f"| {regua(r['controlo_sem'])} / {regua(r['controlo'])} "
                + " ".join(
                    f"| {marca(r[k], r[k] <= lim_extra) if k in r else '—'}"
                    for k in CAMADA
                )
                + (f" | {marca(r['luz'], r['luz'] <= lim_extra)} / {r['luz_p50']}" if "luz" in r else " | —")
                + (f" | {marca(r['abrir'], r['abrir'] <= lim_extra)} / {marca(r['reabrir'], r['reabrir'] <= lim_extra)}"
                   if "abrir" in r and "reabrir" in r else " | —")
                + " |"
            )
    nova = [r for r in linhas if "nova1_toque" in r]
    if nova:  # NOVA=1: toque/traço/soltar de cada traço depois de «+ camada» (o pior quadro da fase + 200 ms)
        print("\n| rotulo | nova 1.º toque/traço/up | nova 2.º | camada de baixo 1.º | nova de volta 1.º |")
        print("|---|---|---|---|---|")
        for r in nova:
            print(f"| {r['rotulo']} | " + " | ".join(
                " / ".join(str(r.get(f"{t}_{f}", "—")) for f in ("toque", "traco", "up"))
                for t in ("nova1", "nova2", "baixo1", "volta1")) + " |")


def perfil(base, top=15):
    """`--perf <pasta/rotulo>` (o modo `perf` do .sh): cada [quadro-lento] passa ao relógio do perf (mono = parede −
    deslocamento das marcas) e recebe as funções FOLHA das amostras dentro do seu intervalo, por fio; as fases
    `parado0`/`parado1` recebem o mesmo, por quadro (amostras ÷ quadros desenhados na janela)."""
    marcas = {}
    for linha in open(base + ".marcas"):
        p = linha.split()
        marcas[p[0]] = (int(p[1]), int(p[2]), int(p[3]) if len(p) > 3 else None)
    desl = [w - m for m, w, _ in marcas.values()]
    d = sorted(desl)[len(desl) // 2]
    # Com -F 997 o perf ajusta o PERÍODO (ciclos por amostra) quadro a quadro: contar amostras pesa errado entre
    # janelas; cada amostra pesa os seus ciclos (`period`).
    out = subprocess.run(
        ["perf", "script", "-i", base + ".perf", "-F", "comm,tid,time,period,ip,sym,dso"],
        capture_output=True, text=True, errors="replace",
    ).stdout
    am = []  # (t_ns mono, comm, sym, ciclos)
    for linha in out.splitlines():
        cab, _, resto = linha.partition(": ")
        p = cab.split()
        try:
            t = int(round(float(p[-1]) * 1e9))
        except (ValueError, IndexError):
            continue
        comm = " ".join(p[:-2]) + "/" + p[-2]
        r = resto.split(None, 2)
        if len(r) < 2:
            continue
        sym = (r[2] if len(r) > 2 else "").rsplit(" (", 1)
        nome = sym[0].strip() or "?"
        dso = sym[1].rstrip(")").rsplit("/", 1)[-1] if len(sym) > 1 else "?"
        am.append((t, comm, nome if nome not in ("[unknown]", "?") else f"[{dso}]", int(r[0])))
    lentos = []
    for linha in open(base + ".log", errors="replace"):
        if linha.startswith("[quadro-lento]"):
            q = dict(kv.split("=") for kv in linha.split()[1:])
            fim = int(q["parede"]) - d
            lentos.append((fim - int(float(q["intervalo"]) * 1e6), fim, float(q["intervalo"])))

    def tabela(a, b, rot, quadros=None):
        sel = [x for x in am if a <= x[0] < b]
        fios, folha = collections.Counter(), collections.Counter()
        for x in sel:
            fios[x[1]] += x[3]
            folha[x[2]] += x[3]
        n = sum(fios.values()) or 1
        pq = f" · {n / 1e6 / quadros:.1f} M ciclos/quadro em {quadros} quadros" if quadros else ""
        print(f"\n### {rot}: {(b - a) / 1e6:.0f} ms · {len(sel)} amostras · {n / 1e6:.0f} M ciclos{pq}")
        princ = f"{am[0][1]}"
        print(f"fios: {len(fios)} com amostras · principal {princ} {fios[princ] * 100 / n:.0f} % · "
              + " · ".join(f"{c} {v * 100 / n:.0f} %" for c, v in fios.most_common(4)))
        import re

        crate = collections.Counter()
        for s, v in folha.items():
            m = re.search(r"\b(ph2d_[a-z0-9_]+)", s) or re.match(r"[<&\s]*([a-z_0-9]+)::", s)
            crate[m.group(1) if m else s] += v
        print("por crate (a 1.ª nomeada na folha): "
              + " · ".join(f"{c} {v * 100 / n:.1f} %" for c, v in crate.most_common(12)))
        print("| % | função folha |\n|---|---|")
        for s, v in folha.most_common(top):
            print(f"| {v * 100 / n:.1f} | `{s[:140]}` |")

    nomes = {"down": "mousedown", "up": "mouse-up", "ctrlz": "Ctrl+Z"}
    for a, b, iv in lentos:
        perto = min(nomes, key=lambda k: abs(b - marcas[k][0]))
        dt = (b - marcas[perto][0]) / 1e6
        tabela(a, b, f"quadro lento {iv:.0f} ms, acaba {dt:+.0f} ms depois do {nomes[perto]}")
    paredes = []
    for linha in open(base + ".log", errors="replace"):
        if linha.startswith("[frame] total="):
            paredes.append(float(linha.split("total=")[1].split("ms")[0]))
    p50 = sorted(paredes)[len(paredes) // 2] if paredes else None
    for fase in ("parado0", "parado1"):
        a, b = marcas[fase + "_ini"][0], marcas[fase + "_fim"][0]
        cpu = marcas[fase + "_fim"][2] - marcas[fase + "_ini"][2]
        tck = os.sysconf("SC_CLK_TCK")
        print(f"\n{fase}: CPU do processo {100 * cpu / tck / ((b - a) / 1e9):.0f} % de um núcleo")
        tabela(a, b, f"{fase} (sem entrada)", int((b - a) / 1e6 / p50) if p50 else None)


def lote(pasta):
    """`--lote <SAIDA>` (o modo `lote` do .sh): o óleo 2880 por rodada (img/s e óptica ms, as 6 rodadas da fase
    «corre») e os seis traços de cada wet6 (pen-down = o maior intervalo até 150 ms depois do 1.º movimento; depois de
    soltar: o maior intervalo em 3 s e as imagens compostas / passos da água por segundo dos `[frame]` desse tempo)."""
    import glob
    import json
    import os
    import re

    print("| corrida | carga | rodada: img/s · óptica ms (fase corre) |\n|---|---|---|")
    for d in sorted(glob.glob(os.path.join(pasta, "lote", "*_*"))):
        f = os.path.join(d, "oleo_2880.log")
        if not os.path.exists(f):
            continue
        rod = [re.search(r"imagens ([\d.]+)/s óptica ([\d.]+)/", l) for l in open(f, errors="replace")
               if l.startswith("[fps] rodada") and " corre " in l]
        carga = " → ".join(l.split()[0] for l in open(os.path.join(d, "carga_oleo")))
        print(f"| {os.path.basename(d)} | {carga} | " + " · ".join(f"{m[1]}/{m[2]}" for m in rod if m) + " |")
    # O `composite xN` do [frame] é o do fio da tela: com o compositor fora do fio (omissão) lê 0 — não é imagens/s.
    print("\n| corrida | pincel | teclas `]` | carga | pen-down ms ×6 | soltar máx 3 s ms ×6 "
          "| passos água/s ×6 | Ctrl+Z ms |\n|---|---|---|---|---|---|---|---|")
    for meta_f in sorted(glob.glob(os.path.join(pasta, "wet6", "*.meta"))):
        tag = meta_f[:-5]
        meta = json.loads(open(meta_f).read())
        marcas = {p[0]: int(p[1]) for p in (l.split() for l in open(tag + ".marcas")) if len(p) >= 2}
        quadros, frames, ultimo = [], [], 0
        for linha in open(tag + ".log", errors="replace"):
            if linha.startswith("[quadro-lento]"):
                q = dict(kv.split("=") for kv in linha.split()[1:])
                ultimo = int(q["parede"])
                quadros.append((ultimo, float(q["intervalo"])))
            elif "composite media" in linha and ultimo:
                m = re.search(r"composite media [\d.]+ms pico [\d.]+ms x(\d+)", linha)
                frames.append([ultimo, int(m[1]) if m else 0, None, None])
            elif "TAXA DA AGUA" in linha and frames:
                m = re.search(r"\((\d+) passos em ([\d.]+)s\)", linha)
                frames[-1][2:] = [int(m[1]), float(m[2])] if m else [None, None]
        pd, so, ci, pa = [], [], [], []
        for t in range(1, 7):
            if f"fim{t}" not in marcas:
                break
            tr, up = marcas[f"traco{t}"], marcas[f"up{t}"]
            pd.append(max((i for p, i in quadros if marcas[f"down{t}"] < p <= tr + 150_000_000), default=0))
            so.append(max((i for p, i in quadros if up < p <= marcas[f"fim{t}"]), default=0))
            fr = [f for f in frames if up < f[0] <= marcas[f"fim{t}"] + 500_000_000 and f[3]]
            pa.append(sum(f[2] for f in fr) / max(sum(f[3] for f in fr), 1e-9) if fr else None)
        # CTRLZ=n (B12a-cont2): o maior intervalo de quadro no segundo depois de cada Ctrl+Z.
        cz = [max((i for p, i in quadros if marcas[f"cz{k}"] < p <= marcas[f"cz{k}"] + 1_000_000_000), default=0)
              for k in range(1, 20) if f"czfim{k}" in marcas]
        fmt = lambda v: " · ".join("—" if x is None else f"{x:.1f}" for x in v)
        print(f"| {os.path.basename(tag).split('_')[0]}_{os.path.basename(tag).split('_')[1]} | {meta['pincel']} "
              f"| {meta['teclas']} | {meta['carga']} | {fmt(pd)} | {fmt(so)} | {fmt(pa)} | {fmt(cz) or '—'} |")


def amostras(perf):
    """(fio, ns mono, ciclos) de cada amostra cycles:u do `perf record` (CICLOS=1); fio = «principal» (tid = pid),
    o nome do fio (`ph2d-wet-otica`, `ph2d-wet-sim`, `Vello`…) ou «rayon» (os sem nome herdam o do processo)."""
    out = subprocess.run(["perf", "script", "-i", perf, "-F", "pid,tid,comm,time,period", "--ns"],
                         capture_output=True, text=True).stdout
    r, nomes = [], {}
    for l in out.splitlines():
        p = l.split()
        if len(p) < 4 or not p[-2].endswith(":") or "/" not in p[-3]:
            continue
        pid, tid = (int(x) for x in p[-3].split("/"))
        nome = " ".join(p[:-3])
        if pid == tid:
            nomes[pid] = nome
        r.append((pid, tid, nome, int(round(float(p[-2][:-1]) * 1e9)), int(p[-1])))
    # O nome do processo é o do BINÁRIO (`antes`, `head_…`): o fio sem nome próprio tem-no também.
    return [("principal" if pid == tid else ("rayon" if nome == nomes.get(pid) else nome), t, c)
            for pid, tid, nome, t, c in r]


def wet6fn(pasta, top=14):
    """`--wet6fn <SAIDA>` (B12a-cont2, CICLOS=1 com STRIP=none): por corrida, a linha PRINCIPAL no pen-down (toque → 1.º
    movimento + 150 ms) e no segundo depois de cada Ctrl+Z: amostras (-F 997 ⇒ ≈ ms de CPU) contra o maior intervalo de
    quadro (parede); a diferença é espera fora da CPU (o `bring_home` no `recv`). E as funções-folha (sem -g) da principal."""
    import collections
    import glob
    import os

    for meta_f in sorted(glob.glob(os.path.join(pasta, "wet6", "*.meta"))):
        tag = meta_f[:-5]
        if not os.path.exists(tag + ".perf"):
            continue
        linhas = [l.split() for l in open(tag + ".marcas")]
        rel = next((p for p in linhas if p[0] == "relogio"), None)
        off = int(rel[1]) - int(rel[2])
        m = {p[0]: int(p[1]) - off for p in linhas if len(p) >= 2 and p[0] != "relogio"}
        quadros = []
        for linha in open(tag + ".log", errors="replace"):
            if linha.startswith("[quadro-lento]"):
                kv = dict(x.split("=") for x in linha.split()[1:])
                quadros.append((int(kv["parede"]) - off, float(kv["intervalo"])))
        out = subprocess.run(["perf", "script", "-i", tag + ".perf", "-F", "pid,tid,time,ip,sym", "--ns"],
                             capture_output=True, text=True).stdout
        am = []
        for l in out.splitlines():
            p = l.split(None, 2)
            if len(p) < 2 or "/" not in p[0] or not p[1].endswith(":"):
                continue
            p = p + [""] if len(p) == 2 else p
            pid, tid = (int(x) for x in p[0].split("/"))
            if pid == tid:
                am.append((int(round(float(p[1][:-1]) * 1e9)), (p[2].split(None, 1) + ["?"])[1].strip() if p[2].strip() else "?"))
        jan = {"pen-down": [(m[f"down{t}"], m[f"traco{t}"] + 150_000_000) for t in range(1, 7) if f"fim{t}" in m],
               "Ctrl+Z": [(m[f"cz{k}"], m[f"cz{k}"] + 1_000_000_000) for k in range(1, 20) if f"czfim{k}" in m]}
        print(f"\n### {os.path.basename(tag)}")
        for nome, js in jan.items():
            if not js:
                continue
            cpu = [sum(1 for t, _ in am if a < t <= b) for a, b in js]
            par = [max((i for p, i in quadros if a < p <= b), default=0) for a, b in js]
            print(f"{nome}: maior intervalo ms {' · '.join(f'{x:.0f}' for x in par)} · CPU principal ≈ ms "
                  f"{' · '.join(map(str, cpu))}")
            c = collections.Counter(s for t, s in am if any(a < t <= b for a, b in js))
            for sym, n in c.most_common(top):
                print(f"  {n:5d}  {sym[:140]}")


def ciclos(pasta):
    """`--ciclos <SAIDA>` (o lote com CICLOS=1): o veredito com load > 5. Óleo: G ciclos por fio na cena inteira e
    M ciclos da principal por quadro. wet6: M ciclos da principal no pen-down (toque → 1.º movimento + 150 ms) e,
    nos 3 s depois de soltar, G ciclos de todos os fios, do `ph2d-wet-otica` e M ciclos do `ph2d-wet-sim` por passo
    da água (`TAXA DA AGUA`)."""
    import glob
    import json
    import os
    import re

    print("| corrida | carga | G ciclos: principal · rayon · otica · sim · todos | M ciclos/quadro principal |")
    print("|---|---|---|---|")
    for d in sorted(glob.glob(os.path.join(pasta, "lote", "*_*"))):
        f = os.path.join(d, "oleo_2880.perf")
        if not os.path.exists(f):
            continue
        a = amostras(f)
        g = lambda fio: sum(c for x, _, c in a if fio is None or x == fio) / 1e9
        quadros = sum(int(m[1]) for m in (re.search(r"quadros +(\d+)", l) for l in
                      open(os.path.join(d, "oleo_2880.log"), errors="replace") if l.startswith("[fps] rodada")) if m)
        carga = " → ".join(l.split()[0] for l in open(os.path.join(d, "carga_oleo")))
        print(f"| {os.path.basename(d)} | {carga} | {g('principal'):.1f} · {g('rayon'):.1f} · "
              f"{g('ph2d-wet-otica'):.1f} · {g('ph2d-wet-sim'):.1f} · {g(None):.1f} | "
              f"{g('principal') * 1e3 / max(quadros, 1):.1f} ({quadros} q) |")
    print("\n| corrida | pincel | `]` | carga | pen-down M ciclos principal ×6 | soltar 3 s G todos ×6 "
          "| soltar 3 s G otica ×6 | M ciclos sim/passo ×6 |\n|---|---|---|---|---|---|---|---|")
    gestos = []
    for meta_f in sorted(glob.glob(os.path.join(pasta, "wet6", "*.meta"))):
        tag = meta_f[:-5]
        if not os.path.exists(tag + ".perf"):
            continue
        meta = json.loads(open(meta_f).read())
        linhas = [l.split() for l in open(tag + ".marcas")]
        rel = next((p for p in linhas if p[0] == "relogio"), None)
        if not rel:
            continue
        off = int(rel[1]) - int(rel[2])
        m = {p[0]: int(p[1]) - off for p in linhas if len(p) >= 2 and p[0] != "relogio"}
        a = amostras(tag + ".perf")
        janela = lambda t0, t1, fio=None: sum(c for x, t, c in a if t0 < t <= t1 and (fio is None or x == fio))
        passos, ultimo, quadros = [], 0, []
        for linha in open(tag + ".log", errors="replace"):
            if linha.startswith("[quadro-lento]"):
                kv = dict(x.split("=") for x in linha.split()[1:])
                ultimo = int(kv["parede"]) - off
                quadros.append((ultimo, float(kv["intervalo"])))
            elif "TAXA DA AGUA" in linha and ultimo:
                q = re.search(r"\((\d+) passos em", linha)
                if q:
                    passos.append((ultimo, int(q[1])))
        pd, so, ot, sp = [], [], [], []
        for t in range(1, 7):
            if f"fim{t}" not in m:
                break
            pd.append(janela(m[f"down{t}"], m[f"traco{t}"] + 150_000_000, "principal") / 1e6)
            so.append(janela(m[f"up{t}"], m[f"fim{t}"]) / 1e9)
            ot.append(janela(m[f"up{t}"], m[f"fim{t}"], "ph2d-wet-otica") / 1e9)
            n = sum(k for p, k in passos if m[f"up{t}"] < p <= m[f"fim{t}"])
            sp.append(janela(m[f"up{t}"], m[f"fim{t}"], "ph2d-wet-sim") / 1e6 / n if n else None)
        fmt = lambda v, k=1: " · ".join("—" if x is None else f"{x:.{k}f}" for x in v)
        b = os.path.basename(tag).split("_")
        print(f"| {b[0]}_{b[1]} | {meta['pincel']} | {meta['teclas']} | {meta['carga']} | {fmt(pd, 0)} | {fmt(so, 2)} "
              f"| {fmt(ot, 2)} | {fmt(sp)} |")
        # O GESTO (B14.2, PASSOS longos): pincel na tela do 1.º movimento ao soltar — G ciclos/s do rayon (onde a
        # óptica corre) e de todos os fios, quadros/s e o maior intervalo de quadro (ms).
        for t in range(1, 7):
            if f"up{t}" not in m:
                break
            t0, t1 = m[f"traco{t}"], m[f"up{t}"]
            s = (t1 - t0) / 1e9
            qs = [iv for p, iv in quadros if t0 < p <= t1]
            gestos.append(f"| {b[0]}_{b[1]} | {meta['pincel']} | {meta['carga']} | {t} | {s:.2f} | "
                          f"{janela(t0, t1, 'rayon') / 1e9 / s:.2f} | {janela(t0, t1) / 1e9 / s:.2f} | "
                          f"{len(qs) / s:.1f} | {max(qs, default=0):.0f} |")
    if gestos:
        print("\n| corrida | pincel | carga | traço | gesto s | G ciclos/s rayon | G ciclos/s todos | quadros/s "
              "| maior intervalo ms |\n|---|---|---|---|---|---|---|---|---|")
        print("\n".join(gestos))


def otica(pasta):
    """`--otica <SAIDA>/wet6` (B14b-cont2): por corrida e por gesto, as imagens da óptica do log da cena
    (`[otica] gesto k: …`), imagens/s, a luz dos factos (Simples/total) e, com o perf, G ciclos do rayon por imagem
    (traço k → soltar k + 3 s: a óptica corre no rayon) e a CASA no gesto (principal + rayon por tique de factos).
    O veredito é a contagem, não o relógio."""
    import glob
    import json
    import os
    import re

    print("| corrida | luz | carga | gesto: imagens · /s · Simples/factos | repouso: imagens · /s "
          "| G ciclos rayon por imagem (gesto+3 s) |\n|---|---|---|---|---|---|")
    for meta_f in sorted(glob.glob(os.path.join(pasta, "*.meta"))):
        tag = meta_f[:-5]
        meta = json.loads(open(meta_f).read())
        if "luz" not in meta:
            continue
        fases = {"gesto": [], "repouso": []}
        for l in open(tag + ".log", errors="replace"):
            r = re.match(r"\[otica\] (gesto|repouso) (\d+): (\d+) imagens em ([\d.]+) s \(([\d.]+)/s\) · luz Simples "
                         r"(\d+)/(\d+)", l)
            if r:
                fases[r[1]].append((int(r[3]), float(r[5]), int(r[6]), int(r[7])))
        # Os OLEO_ROLA cliques da roda abrem uma «fase de gesto» curta antes dos traços: ficam os últimos TRACOS.
        ups = sum(1 for p in open(tag + ".marcas") if p.startswith("up"))
        fases = {f: v[-ups:] if ups else v for f, v in fases.items()}
        cic = []
        if os.path.exists(tag + ".perf"):
            linhas = [p.split() for p in open(tag + ".marcas")]
            rel = next(p for p in linhas if p[0] == "relogio")
            off = int(rel[1]) - int(rel[2])
            m = {p[0]: int(p[1]) - off for p in linhas if len(p) >= 2 and p[0] != "relogio"}
            a = amostras(tag + ".perf")
            for k, g in enumerate(fases["gesto"], 1):
                if f"fim{k}" in m:
                    c = sum(c for x, t, c in a if x == "rayon" and m[f"traco{k}"] < t <= m[f"fim{k}"]) / 1e9
                    n = g[0] + (fases["repouso"][k - 1][0] if k <= len(fases["repouso"]) else 0)
                    gp = sum(c for x, t, c in a if x == "principal" and m[f"traco{k}"] < t <= m[f"up{k}"]) / 1e9
                    gr = sum(c for x, t, c in a if x == "rayon" and m[f"traco{k}"] < t <= m[f"up{k}"]) / 1e9
                    # A CASA (o composite do traço) por tique de factos: principal + rayon, M ciclos.
                    pf = (gp + gr) * 1e3 / max(g[3], 1)
                    cic.append(f"{c / max(n, 1):.2f} (gesto: principal {gp:.1f} · rayon {gr:.1f} G · "
                               f"casa {pf:.0f} M/facto)")
        g = " / ".join(f"{n} · {s:.1f} · {si}/{f}" for n, s, si, f in fases["gesto"])
        rp = " / ".join(f"{n} · {s:.1f}" for n, s, _, _ in fases["repouso"])
        print(f"| {os.path.basename(tag).split('_')[0]} | {meta['luz']} | {meta['carga']} | {g} | {rp} "
              f"| {' · '.join(cic) or '—'} |")


if len(sys.argv) > 1 and sys.argv[1] == "--otica":
    otica(sys.argv[2])
    sys.exit(0)

if len(sys.argv) > 1 and sys.argv[1] == "--lote":
    lote(sys.argv[2])
    sys.exit(0)

if len(sys.argv) > 1 and sys.argv[1] == "--wet6fn":
    wet6fn(sys.argv[2])
    sys.exit(0)
if len(sys.argv) > 1 and sys.argv[1] == "--ciclos":
    ciclos(sys.argv[2])
    sys.exit(0)

if len(sys.argv) > 1 and sys.argv[1] == "--perf":
    import os

    perfil(sys.argv[2])
    sys.exit(0)

def faixas(pasta, saida):
    """`--faixas <pasta> <saida>` (B08.5): o par DENTRO (A) × FORA (B) da faixa segura de cada caso, rodada a rodada,
    com a carga de cada lançamento; o MÍNIMO entre rodadas quando há mais de uma."""
    import contextlib
    import io
    import json
    import os
    import re

    with contextlib.redirect_stdout(io.StringIO()):
        matriz(pasta, saida)
    casos = {}
    for linha in open(os.path.join(saida, "matriz.jsonl")):
        r = json.loads(linha)
        m = re.fullmatch(r"(.+)([AB])(\d+)", r.get("rot", ""))
        if m:
            casos.setdefault(m.group(1), {}).setdefault(m.group(2), []).append(r)
    campos = ("toque", "traco_p50", "traco_p95", "soltar_1s", "ctrlz", "parado_cpu_depois", "agua_hz")
    print("| caso | variante | pincel | toque ms | traço p50/p95 ms | +1 s máx ms | Ctrl+Z ms | CPU parado depois % "
          "| Wet img/s | controlo ms | carga |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for caso, vs in casos.items():
        for v in "AB":
            rs = [r for r in vs.get(v, []) if "erro" not in r]
            if not rs:
                print(f"| {caso} | {v} | — | sem janela ou sem quadros |||||||")
                continue
            mn = {k: min((r[k] for r in rs if r.get(k) is not None), default=None) for k in campos}
            if mn["agua_hz"] is not None:
                mn["agua_hz"] = max(r.get("agua_hz") or 0 for r in rs)
            var = " ".join(x for x in (rs[0].get("knobs") and "knobs " + rs[0]["knobs"],
                                       rs[0].get("teclas", "0") != "0" and "`]`×" + rs[0]["teclas"],
                                       rs[0].get("relevo") and "relief " + rs[0]["relevo"]) if x) or "fábrica"
            print(f"| {caso} | {v}: {var} | {rs[0]['pincel']} | {mn['toque']} | {mn['traco_p50']} / {mn['traco_p95']} "
                  f"| {mn['soltar_1s']} | {mn['ctrlz']} | {mn['parado_cpu_depois']} | {mn['agua_hz'] or '—'} "
                  f"| {min(r.get('controlo', 0) for r in rs)} | {' · '.join(r['carga'].split()[0] for r in rs)} |")


if len(sys.argv) > 1 and sys.argv[1] == "--faixas":
    faixas(sys.argv[2], sys.argv[3])
    sys.exit(0)

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
            d = dict(kv.split("=", 1) for kv in linha.split()[1:] if "=" in kv)
            lentos.append((int(d["parede"]), float(d["intervalo"]), float(d["quadro"])))
    print(f"\nquadros lentos: {len(lentos)} no total")
    for k, u in enumerate(upsp):
        perto = [(round((p - u) / 1e6), i, q) for p, i, q in lentos if -500e6 <= p - u <= 3000e6]
        print(f"  Up {k + 1}: " + (" · ".join(f"+{d}ms {i:.0f}ms(q {q:.0f})" for d, i, q in perto) or "nenhum"))
