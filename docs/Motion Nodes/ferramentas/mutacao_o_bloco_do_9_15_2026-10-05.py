#!/usr/bin/env python3
"""Prova de mutação dos pedaços do doc 121 §9.15, dobrados no §9.17: o ajuste do tracejado que sai da CONTAGEM (A1a),
o traço adiado de um fechado (A1b), o prefixo das arestas ESCRITAS (B1), a reserva com a junta uma vez por troço (B2),
a capacidade medida antes do 1.º quadro de uma cena nova (c2) e o prefixo do `cs_varre` por subgrupo (D). Cada mutação
tem de SANGRAR nos `14` gates de placa de `ph2d-shape-gpu` (`--test it -- --ignored`), na placa que ela nomeia.

Controlos: pré-voo (cada âncora casa o nº esperado de vezes) · corrida LIMPA verde nas DUAS placas, com população > 0 ·
mutação que não compila é defeito do arnês · zero testes aborta · shader que não valida é marcado. Restaura por cópia
+ touch (o cargo guarda o build da mutação pelo mtime). ⛔ Sozinho na árvore: nenhuma compilação em paralelo.
Uso: MUTA_SO_ANCORAS=1 só o pré-voo; MUTA_SO=m1,m6 filtra.
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
C = R + "/crates/ph2d-shape-gpu/src/contorno.wgsl"
SG = R + "/crates/ph2d-shape-gpu/src/contorno_subgrupo.wgsl"
CR = R + "/crates/ph2d-shape-gpu/src/contorno.rs"
P = R + "/crates/ph2d-shape-gpu/src/pass.rs"

GPU = ["cargo", "test", "-p", "ph2d-shape-gpu", "--profile", "smoke", "--test", "it", "--", "--ignored"]
ICD = {
    "rtx": "/usr/share/vulkan/icd.d/nvidia_icd.json",
    "igpu": "/usr/share/vulkan/icd.d/radeon_icd.json",
}

# (nome, placas, [(ficheiro, âncora, substituição, nº de ocorrências)])
MUTS = [
    ("m1 o ajuste da contagem a 1", ["rtx"], [(C,
        "        ajuste = lim.ajuste;\n", "        ajuste = 1.0;\n", 1)]),
    ("m2 o traco adiado do fechado nunca emitido", ["rtx"], [(C,
        "let p = pedaco(tr, sub, 0.0);\n        if p.valido {", "let p = pedaco(tr, sub, 0.0);\n        if false {", 1)]),
    ("m3 o prefixo das escritas deslocado de um bloco", ["rtx"], [(C,
        "        contagem[e0 + i] = acc;\n", "        contagem[e0 + i] = acc + SEGS_POR_BLOCO;\n", 1)]),
    ("m4 a reserva sem as pontas", ["rtx"], [(C,
        "let por_troco = pecas * (4u + 2u * tampa) + junta;", "let por_troco = pecas * 4u + junta;", 1)]),
    ("m5a o c2 desligado", ["rtx"], [(CR,
        "&& self.ligado && self.mede_no_inicio {", "&& self.ligado && false {", 1)]),
    ("m5b a cena nova sem medir_ja", ["rtx"], [(P,
        "        self.contorno.medir_ja = true;\n", "", 1)]),
    ("m6 o subgrupo sem a correcao da fronteira da celula", ["igpu"], [(SG,
        "    s = select(s, s - antes, inicio > 0u);\n", "", 1)]),
]

BASE_ENV = dict(os.environ, PH2D_GPU="1", PH2D_GPU_ESPERA="1500")


def corrida(placa):
    env = dict(BASE_ENV, VK_ICD_FILENAMES=ICD[placa])
    p = subprocess.run(["bash", "scripts/ph2d-run.sh"] + GPU, cwd=R, capture_output=True, text=True, env=env)
    out = p.stdout + p.stderr
    pas = fal = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed", out):
        pas += int(m.group(1)); fal += int(m.group(2))
    return p.returncode, pas, fal, "could not compile" not in out, out


def aplica(texto, passos):
    for _f, a, b, n in passos:
        assert texto.count(a) == n, f"ancora {a!r}: {texto.count(a)} != {n}"
        texto = texto.replace(a, b)
    return texto


def main():
    so = os.environ.get("MUTA_SO")
    muts = [m for m in MUTS if not so or m[0].split()[0] in so.split(",")]
    for nome, _placas, passos in MUTS:
        textos = {}
        for f, a, b, n in passos:
            t = textos.setdefault(f, open(f).read())
            if t.count(a) != n:
                print(f"ABORTA: ancora de {nome} casa {t.count(a)} vezes (esperado {n})"); sys.exit(2)
            textos[f] = t.replace(a, b)
    print(f"pre-voo: {len(MUTS)}/{len(MUTS)} mutacoes com todas as ancoras a casar", flush=True)
    if os.environ.get("MUTA_SO_ANCORAS"):
        return
    for placa in sorted({p for m in muts for p in m[1]} | {"rtx", "igpu"}):
        rc, p, f, ok, out = corrida(placa)
        if rc != 0 or p + f == 0 or f:
            print(f"ABORTA: corrida LIMPA na {placa} nao esta verde (rc {rc}, {p} passed, {f} failed)")
            print(out[-2500:]); sys.exit(2)
        print(f"corrida limpa: {placa} verde, {p} passed", flush=True)
    sangrou = 0
    for nome, placas, passos in muts:
        ficheiros = sorted({x[0] for x in passos})
        origs = {f: open(f).read() for f in ficheiros}
        res = []
        try:
            for f in ficheiros:
                shutil.copy2(f, f + ".muta_bk")
                open(f, "w").write(aplica(origs[f], [x for x in passos if x[0] == f]))
            for placa in placas:
                res.append((placa,) + corrida(placa))
        finally:
            for f in ficheiros:
                open(f, "w").write(origs[f])
                os.remove(f + ".muta_bk") if os.path.exists(f + ".muta_bk") else None
                os.utime(f, (time.time(), time.time()))
        for placa, rc, p, fl, ok, out in res:
            validou = not re.search(r"(?i)shader.*(error|invalid)|validation error|createShaderModule", out)
            if not ok:
                v = "NAO COMPILA (defeito do arnes)"
            elif p + fl == 0:
                v = "ZERO testes (defeito do arnes)"
            elif rc != 0 or fl:
                v = "SANGROU"
            else:
                v = "SOBREVIVEU"
            falhos = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
            extra = "" if validou else " [shader nao validou]"
            print(f"{nome} [{placa}]: {v} ({p} passed, {fl} failed) reprovou: {falhos}{extra}", flush=True)
        sangrou += all(r[1] != 0 or r[3] for r in res) and all(r[4] for r in res)
    for f in (C, SG, CR, P):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
