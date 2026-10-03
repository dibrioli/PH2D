#!/usr/bin/env python3
"""Prova de mutação do buffer de ACUMULAÇÃO das células (doc 121 §9.12): cada mutação tem de SANGRAR no
gate de GPU de `ph2d-shape-gpu` (10 testes `--ignored`: paridade com o Vello, contorno par contra par,
cópias que não cabem nas células, cena que muda, quina exacta, NaN vertical, rota, tracejado).

Substitui a das listas (`mutacao_as_listas_das_celulas_2026-10-02.py`, 14/14): o código que ela mutava
saiu com as listas. Cada mutação dela tem aqui o análogo que ainda faz sentido.

Controlos: pré-voo (cada âncora casa o nº esperado de vezes) · corrida LIMPA verde com população > 0
(lida do `test result:`) · mutação que não compila é defeito do arnês · zero testes aborta · um shader
que não valida é marcado (sangrar por não compilar o WGSL não prova a lei). Restaura por cópia + touch
(o cargo guarda o build da mutação pelo mtime). Uso: MUTA_SO_ANCORAS=1 só o pré-voo; MUTA_SO=A1,A5 filtra.
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
C = R + "/crates/ph2d-shape-gpu/src/contorno.wgsl"
S = R + "/crates/ph2d-shape-gpu/src/shape.wgsl"

# (nome, [(ficheiro, âncora, substituição, nº de ocorrências)])
MUTS = [
    ("A1 o fundo somado uma celula depois da kb", [(C,
        "atomicAdd(&ccelulas_rw[(q0 + kb) * REGISTO + a.fam]",
        "atomicAdd(&ccelulas_rw[(q0 + min(kb + 1u, a.celulas - 1u)) * REGISTO + a.fam]", 1)]),
    ("A2 o primeiro pixel cruzado perdido", [(C,
        "for (var x = u32(max(floor(lo) - a.x0, 0.0));", "for (var x = u32(max(floor(lo) - a.x0 + 1.0, 0.0));", 1)]),
    ("A3 o degrau final ate dy perdido", [(C,
        "ceil(hi) - a.x0 + 1.0);", "ceil(hi) - a.x0);", 1)]),
    ("A4 o prefixo nao recomeca na celula", [(C,
        "if x % PIXELS_DA_CELULA == 0u {", "if false {", 1)]),
    ("A5 cs_fundo sem o prefixo do fundo", [(C,
        "fundo += vec3<i32>(", "fundo = vec3<i32>(", 1)]),
    ("A6 cs_zera nao apaga a acumulacao do preenchimento", [(C,
        "atomicStore(&acumula_rw[a], 0u);", "_ = atomicLoad(&acumula_rw[a]);", 1)]),
    ("A7 cs_zera nao apaga o fundo", [(C,
        "atomicStore(&ccelulas_rw[cel * REGISTO + p], 0u);", "_ = atomicLoad(&ccelulas_rw[cel * REGISTO + p]);", 1)]),
    ("A8 a varredura nao e segmentada pela celula", [(C,
        "if p >= d {", "if li >= d {", 1)]),
    ("A9 cs_varre ignora a regra par-impar", [(C,
        "if atomicLoad(&ccelulas_rw[q + 3u]) != 0u {", "if false {", 1)]),
    ("A10 cs_varre divide o preenchimento por 2 * ESCALA_FIXA", [(C,
        "let af0 = f32(s.x) / ESCALA_FIXA;", "let af0 = f32(s.x) / (2.0 * ESCALA_FIXA);", 1)]),
    ("A11 o recorte a fileira com o minimo trocado pelo maximo", [(C,
        "let lo = clamp(min(xa, xb), f.y, f.z);", "let lo = clamp(max(xa, xb), f.y, f.z);", 1)]),
    ("A12 sem a verificacao da capacidade das celulas", [(C,
        "        || mbase + nmask_reservado > contas.cap_celulas {", "        {", 1)]),
    ("A13 o fragmento le a fileira com o passo errado", [(S,
        "(c1.x + u32(r) * c2.y) * PIXELS_DA_CELULA", "(c1.x + u32(r) * (c2.y + 1u)) * PIXELS_DA_CELULA", 1)]),
    ("A14 preenchimento e contorno trocados na familia da aresta", [(C,
        "a.fam = select(select(2u, 1u, bl < c0.y + c0.z), 0u, bl < c0.y);",
        "a.fam = select(select(0u, 1u, bl < c0.y + c0.z), 2u, bl < c0.y);", 1)]),
    ("A15 cs_escreve nao leva a regra da copia", [(C,
        "p.celulas, p.regra, 0u);", "p.celulas, 0u, 0u);", 1)]),
    ("A16 a kb por floor (o fundo na celula que a aresta ainda cruza)", [(C,
        "ceil((hi - a.x0) / LARGURA_DA_CELULA)", "floor((hi - a.x0) / LARGURA_DA_CELULA)", 1)]),
    ("A17 o deposito sem a diferenca para o pixel anterior", [(C,
        "deposita(q0, a.fam, x, v - ant);", "deposita(q0, a.fam, x, v);", 1)]),
]

CMD = ["bash", "scripts/ph2d-run.sh", "cargo", "test", "-p", "ph2d-shape-gpu", "--release", "--", "--ignored"]
ENV = dict(os.environ, PH2D_GPU="1", PH2D_GPU_ESPERA="1500")


def corrida():
    p = subprocess.run(CMD, cwd=R, capture_output=True, text=True, env=ENV)
    out = p.stdout + p.stderr
    pas = fal = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed", out):
        pas += int(m.group(1)); fal += int(m.group(2))
    compilou = "could not compile" not in out
    return p.returncode, pas, fal, compilou, out


def aplica(texto, passos):
    for _f, a, b, n in passos:
        assert texto.count(a) == n, f"ancora {a!r}: {texto.count(a)} != {n}"
        texto = texto.replace(a, b)
    return texto


def main():
    so = os.environ.get("MUTA_SO")
    muts = [m for m in MUTS if not so or m[0].split()[0] in so.split(",")]
    for nome, passos in MUTS:
        textos = {}
        for f, a, b, n in passos:
            t = textos.setdefault(f, open(f).read())
            c = t.count(a)
            if c != n:
                print(f"ABORTA: ancora de {nome} casa {c} vezes (esperado {n})"); sys.exit(2)
            textos[f] = t.replace(a, b)
    print(f"pre-voo: {len(MUTS)}/{len(MUTS)} mutacoes com todas as ancoras a casar o numero esperado de vezes")
    if os.environ.get("MUTA_SO_ANCORAS"):
        return
    rc, p, f, ok, out = corrida()
    if rc != 0 or p + f == 0 or f:
        print(f"ABORTA: corrida LIMPA nao esta verde (rc {rc}, {p} passed, {f} failed)")
        print(out[-2500:]); sys.exit(2)
    print(f"corrida limpa: verde, {p} passed", flush=True)
    sangrou = 0
    for nome, passos in muts:
        ficheiros = sorted({x[0] for x in passos})
        origs = {f: open(f).read() for f in ficheiros}
        bks = {}
        try:
            for f in ficheiros:
                bks[f] = f + ".muta_bk"; shutil.copy2(f, bks[f])
                open(f, "w").write(aplica(origs[f], [x for x in passos if x[0] == f]))
            rc, p, fl, ok, out = corrida()
        finally:
            for f in ficheiros:
                open(f, "w").write(origs[f])
                os.remove(bks[f]) if os.path.exists(bks[f]) else None
                os.utime(f, (time.time(), time.time()))
        validou = not re.search(r"(?i)shader.*(error|invalid)|validation error|Error matching|createShaderModule", out)
        if not ok:
            v = "NAO COMPILA (defeito do arnes)"
        elif p + fl == 0:
            v = "ZERO testes (defeito do arnes)"
        elif rc != 0 or fl:
            v = "SANGROU"; sangrou += 1
        else:
            v = "SOBREVIVEU"
        falhos = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
        extra = "" if validou else " [shader nao validou]"
        print(f"{nome}: {v} ({p} passed, {fl} failed) reprovou: {falhos}{extra}", flush=True)
        if not falhos and v == "SANGROU":
            print("   cauda:", out[-600:].replace("\n", " | "), flush=True)
    for f in (C, S):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
