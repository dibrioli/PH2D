#!/usr/bin/env python3
"""Prova de mutação do bloco do doc 121 §9.14: a porta `redesenha` e o halo do `fx.glow` pela rota do quadro. Cada mutação tem de SANGRAR
nos gates que a nomeiam — a GPU de `ph2d-shape-gpu` (`--ignored`), os gates puros do halo
(`ph2d-app-motion --lib motion_glow_layer`) ou os de costura da shell (`present_placa`).

Controlos: pré-voo (cada âncora casa o nº esperado de vezes) · corrida LIMPA verde de cada comando com
população > 0 · mutação que não compila é defeito do arnês · zero testes aborta · shader que não valida
é marcado. Restaura por cópia + touch (o cargo guarda o build da mutação pelo mtime).
Uso: MUTA_SO_ANCORAS=1 só o pré-voo; MUTA_SO=R1,H2 filtra. (O passe de GRUPO e o D1/D2 do
`cs_varre` foram RECUSADOS pela rodada do §9.14 e saíram do código com as mutações deles.)
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
P = R + "/crates/ph2d-shape-gpu/src/pass.rs"
H = R + "/crates/ph2d-app-motion/src/motion_glow_layer.rs"
PR = R + "/shells/desktop/src/render_loop/present.rs"
FX = R + "/shells/desktop/src/render_loop/present_fx.rs"

GPU = ["cargo", "test", "-p", "ph2d-shape-gpu", "--release", "--", "--ignored"]
HALO = ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "motion_glow_layer"]
SHELL = ["cargo", "test", "-p", "ph2d-host-desktop", "--bin", "ph2d-host-desktop", "placa_tests"]

# (nome, comando, [(ficheiro, âncora, substituição, nº de ocorrências)])
MUTS = [
    ("R1 o redesenho mudo", GPU, [(P,
        "self.passe_de_desenho(gpu, encoder, target, wgpu::LoadOp::Load, Some((bg, *count)));",
        "self.passe_de_desenho(gpu, encoder, target, wgpu::LoadOp::Load, None);", 1)]),
    ("H1 o halo do dispositivo le as listas da CPU", HALO, [(H,
        "    if motion.gpu_live {\n        // A MESMA pergunta", "    if false {\n        // A MESMA pergunta", 1)]),
    ("H2 as formas da placa tambem pelo tile", HALO, [(H,
        "let vetores: &[VectorInstance] = if formas_da_placa {", "let vetores: &[VectorInstance] = if false {", 1)]),
    ("H3 a camada deixa de se desenhar antes do halo", SHELL, [(PR,
        "        let _ = placa.desenha(surface.gpu(), tamanho, geos, buffer);\n", "", 1)]),
    ("H4 o halo sem o buffer do dispositivo", SHELL, [(FX,
        "            &halo.cpu,\n            halo.placa,", "            &halo.cpu,\n            None,", 1)]),
    ("H5 o halo sem o redesenho das formas", SHELL, [(FX,
        "if halo.formas_da_placa && !g.formas.redesenha_em(", "if false && !g.formas.redesenha_em(", 1)]),
]

ENV = dict(os.environ, PH2D_GPU="1", PH2D_GPU_ESPERA="1500")


def corrida(cmd):
    p = subprocess.run(["bash", "scripts/ph2d-run.sh"] + cmd, cwd=R, capture_output=True, text=True, env=ENV)
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
    for nome, _cmd, passos in MUTS:
        textos = {}
        for f, a, b, n in passos:
            t = textos.setdefault(f, open(f).read())
            if t.count(a) != n:
                print(f"ABORTA: ancora de {nome} casa {t.count(a)} vezes (esperado {n})"); sys.exit(2)
            textos[f] = t.replace(a, b)
    print(f"pre-voo: {len(MUTS)}/{len(MUTS)} mutacoes com todas as ancoras a casar")
    if os.environ.get("MUTA_SO_ANCORAS"):
        return
    for cmd in {tuple(m[1]) for m in muts}:
        rc, p, f, ok, out = corrida(list(cmd))
        if rc != 0 or p + f == 0 or f:
            print(f"ABORTA: corrida LIMPA de {' '.join(cmd)} nao esta verde (rc {rc}, {p} passed, {f} failed)")
            print(out[-2500:]); sys.exit(2)
        print(f"corrida limpa: {' '.join(cmd[3:6])} verde, {p} passed", flush=True)
    sangrou = 0
    for nome, cmd, passos in muts:
        ficheiros = sorted({x[0] for x in passos})
        origs = {f: open(f).read() for f in ficheiros}
        try:
            for f in ficheiros:
                shutil.copy2(f, f + ".muta_bk")
                open(f, "w").write(aplica(origs[f], [x for x in passos if x[0] == f]))
            rc, p, fl, ok, out = corrida(cmd)
        finally:
            for f in ficheiros:
                open(f, "w").write(origs[f])
                os.remove(f + ".muta_bk") if os.path.exists(f + ".muta_bk") else None
                os.utime(f, (time.time(), time.time()))
        validou = not re.search(r"(?i)shader.*(error|invalid)|validation error|createShaderModule", out)
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
    for f in (P, H, PR, FX):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
