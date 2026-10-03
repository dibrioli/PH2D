#!/usr/bin/env python3
"""Prova de mutação da VARIANTE ENXUTA (doc 121 §9.10) sobre os gates de GPU de ph2d-shape-gpu (`--test
it`) e os de paridade do PRODUTO em ph2d-app-motion (`motion_shape_placa::gpu_tests`).
Controlos: pré-voo das âncoras · corrida LIMPA verde · não compila = defeito do arnês · restauro por
cópia + touch. Uso: MUTA_SO=V1,V5 filtra; MUTA_SO_ANCORAS=1 só o pré-voo.

⚠️ A V9 (o `override` fora do predicado do WGSL) é SOBREVIVENTE ESPERADA: não muda um pixel nem a escolha
da variante — só a `registos_dos_shaders.sh` a vê (os VGPRs da enxuta voltam a `128`)."""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
P = R + "/crates/ph2d-shape-gpu/src/pass.rs"
C = R + "/crates/ph2d-shape-gpu/src/contorno.rs"
E = R + "/crates/ph2d-shape-gpu/src/eixo.rs"
S = R + "/crates/ph2d-shape-gpu/src/shape.wgsl"

ESCOLHA = "self.tracejado = eixo.iter().any(crate::EixoItem::tracejado);"
MUTS = [
    ("V1 escolhe sempre a COMPLETA", [(P, ESCOLHA, "self.tracejado = true;", 1)]),
    ("V2 escolhe sempre a ENXUTA", [(P, ESCOLHA, "self.tracejado = false;", 1)]),
    ("V3 Variantes::de trocada", [
        (C, "            &self.completa\n", "            &self.QQ\n", 1),
        (C, "            &self.enxuta\n", "            &self.completa\n", 1),
        (C, "            &self.QQ\n", "            &self.enxuta\n", 1)]),
    ("V4 as opcoes de compilacao trocadas", [(C,
        "constants: if tracejado { COMPLETA } else { ENXUTA },",
        "constants: if tracejado { ENXUTA } else { COMPLETA },", 1)]),
    ("V5 a contagem sempre na enxuta", [(C,
        "self.conta.de(tracejado)", "self.conta.de(false)", 1)]),
    ("V6 a escrita sempre na enxuta", [(C,
        "self.escreve.de(tracejado)", "self.escreve.de(false)", 1)]),
    ("V7 o desenho sempre na enxuta", [(P,
        "self.pipeline.de(self.tracejado)", "self.pipeline.de(false)", 1)]),
    ("V8 o predicado do Rust aceita todo troco", [(E,
        "self.traco + self.vao > 0.0", "self.traco + self.vao >= 0.0", 1)]),
    ("V9 o override fora do predicado do WGSL (sobrevivente ESPERADA)", [(S,
        "return TRACEJADO && it.traco + it.vao > 0.0;", "return it.traco + it.vao > 0.0;", 1)]),
]

CMD = ["bash", "scripts/ph2d-run.sh", "cargo", "test", "-p", "ph2d-shape-gpu", "--release", "--test", "it",
       "--", "--ignored", "--nocapture"]
CMD2 = ["bash", "scripts/ph2d-run.sh", "cargo", "test", "-p", "ph2d-app-motion", "--release", "--lib",
        "motion_shape_placa::gpu_tests", "--", "--ignored", "--nocapture"]
ENV = dict(os.environ, PH2D_GPU="1", PH2D_GPU_ESPERA="1500")


def corrida():
    p = subprocess.run(CMD, cwd=R, capture_output=True, text=True, env=ENV)
    q = subprocess.run(CMD2, cwd=R, capture_output=True, text=True, env=ENV)
    out = p.stdout + p.stderr + q.stdout + q.stderr
    rc = p.returncode or q.returncode
    pas = fal = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed", out):
        pas += int(m.group(1)); fal += int(m.group(2))
    return rc, pas, fal, "could not compile" not in out, out


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
            if t.count(a) != n:
                print(f"ABORTA: ancora de {nome} casa {t.count(a)} vezes (esperado {n}): {a!r}"); sys.exit(2)
            textos[f] = t.replace(a, b)
    print(f"pre-voo: {len(MUTS)}/{len(MUTS)}", flush=True)
    if os.environ.get("MUTA_SO_ANCORAS"):
        return
    rc, p, f, ok, out = corrida()
    print(f"corrida limpa: rc {rc}, {p} passed, {f} failed", flush=True)
    if rc != 0 or p + f == 0 or f:
        print(out[-3000:]); sys.exit(2)
    sangrou = 0
    for nome, passos in muts:
        ficheiros = sorted({x[0] for x in passos})
        origs = {f: open(f).read() for f in ficheiros}
        try:
            for f in ficheiros:
                shutil.copy2(f, f + ".muta_bk")
                open(f, "w").write(aplica(origs[f], [x for x in passos if x[0] == f]))
            rc, p, fl, ok, out = corrida()
        finally:
            for f in ficheiros:
                open(f, "w").write(origs[f])
                os.remove(f + ".muta_bk")
                os.utime(f, (time.time(), time.time()))
        validou = not re.search(r"(?i)shader validation error|is invalid", out)
        if not ok:
            v = "NAO COMPILA (defeito do arnes)"
        elif not validou:
            v = "SHADER NAO VALIDOU (defeito do arnes)"
        elif p + fl == 0:
            v = "ZERO testes (defeito do arnes)"
        elif rc != 0 or fl:
            v = "SANGROU"; sangrou += 1
        else:
            v = "SOBREVIVEU"
        falhos = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
        print(f"{nome}: {v} ({p} passed, {fl} failed) reprovou: {falhos}", flush=True)
    for f in (P, C, E, S):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
