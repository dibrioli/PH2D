#!/usr/bin/env python3
"""Prova de mutação do TRACEJADO no ecrã (doc 121 §9.9) sobre os 9 gates de GPU de ph2d-shape-gpu e
os 6 de paridade do PRODUTO em ph2d-app-motion (`motion_shape_placa::gpu_tests`).
Controlos: pré-voo das âncoras · corrida LIMPA verde · não compila = defeito do arnês · restauro por
cópia + touch. Uso: MUTA_SO=T1,T5 filtra; MUTA_SO_ANCORAS=1 só o pré-voo."""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
C = R + "/crates/ph2d-shape-gpu/src/contorno.wgsl"
S = R + "/crates/ph2d-shape-gpu/src/shape.wgsl"
E = R + "/crates/ph2d-shape-gpu/src/eixo.rs"

MUTS = [
    ("T1 o padrao nao escala com a caneta", [
        (S, "s.tr = it.traco * caneta;", "s.tr = it.traco;", 1),
        (S, "s.per = (it.traco + it.vao) * caneta;", "s.per = (it.traco + it.vao);", 1)]),
    ("T2 o fechado nunca emenda", [(S,
        "s.emenda = s.a_fim < s.tot && s.tot < s.a_fim + s.tr;", "s.emenda = false;", 1)]),
    ("T3 o fechado emenda sempre", [(S,
        "s.emenda = s.a_fim < s.tot && s.tot < s.a_fim + s.tr;", "s.emenda = s.fechado;", 1)]),
    ("T4 o traco nunca liga ao troco de tras", [(S,
        "p.liga0 = a < tr.s0 && tr.tem_ant;", "p.liga0 = false;", 1)]),
    ("T5 o traco nunca liga ao troco da frente", [(S,
        "p.liga1 = b > tr.fim && tr.tem_seg;", "p.liga1 = false;", 1)]),
    ("T6 a faixa sem o recuo dos pedacos (as duas passagens)", [
        (S, "it.limite, p.recuo0);", "it.limite, 1.0e30);", 1),
        (S, "it.limite, p.recuo1);", "it.limite, 1.0e30);", 1),
        (C, "it.limite, p.recuo0);", "it.limite, 1.0e30);", 1),
        (C, "it.limite, p.recuo1);", "it.limite, 1.0e30);", 1)]),
    ("T7 a fase recomeca em cada troco (as duas passagens)", [
        (S, "        s0 = tr.fim;", "        s0 = 0.0;", 1),
        (C, "        s0 = tr.fim;", "        s0 = 0.0;", 1)]),
    ("T8 as pontas de inicio e fim trocadas (as duas passagens)", [
        (S, "(it.ponta >> 6u) & 3u", "(it.ponta >> QQu) & 3u", 1),
        (S, "(it.ponta >> 8u) & 3u", "(it.ponta >> 6u) & 3u", 1),
        (S, "(it.ponta >> QQu) & 3u", "(it.ponta >> 8u) & 3u", 1),
        (C, "emite_tampa(q.xy, -u, r, (it.ponta >> 6u) & 3u);", "emite_tampa(q.xy, -u, r, (it.ponta >> QQu) & 3u);", 1),
        (C, "emite_tampa(q.zw, u, r, (it.ponta >> 8u) & 3u);", "emite_tampa(q.zw, u, r, (it.ponta >> 6u) & 3u);", 1),
        (C, "(it.ponta >> QQu) & 3u);", "(it.ponta >> 8u) & 3u);", 1)]),
    ("T9 sem a ponta do inicio de cada traco (as duas passagens)", [
        (S, "s += tampa_px(q.xy, -u, r, (it.ponta >> 6u) & 3u, xy);", "s += 0.0;", 1),
        (C, "emite_tampa(q.xy, -u, r, (it.ponta >> 6u) & 3u);", "", 1)]),
    ("T10 o arco medido no LOCAL x caneta (a lei que a casa recusa)", [(S,
        "let d = aplica(lin, t, it.b) - aplica(lin, t, it.a);",
        "let d = (it.b - it.a) * sqrt(abs(lin.x * lin.w - lin.z * lin.y));", 1)]),
    ("T15 o arco e' a corda (sem a flecha)", [(S,
        "return c + 8.0 * h * h / (3.0 * c);", "return c;", 1)]),
    ("T11 o pixel a pixel sem a junta dentro de um traco", [(S,
        "s += junta_em(u, b, cf, r, select(2u, it.junta, quina), it.limite, xy);", "s += 0.0;", 1)]),
    ("T12 o calculado sem a ponta do fim de cada traco", [(C,
        "emite_tampa(q.zw, u, r, (it.ponta >> 8u) & 3u);", "", 1)]),
    ("T13 a emenda com o recuo de um troco do meio", [(S,
        "la = sub.tot - sub.a_fim;", "la = tr.s0 - a;", 1)]),
    ("T14 o pixel a pixel salta os blocos tracejados pela caixa", [(S,
        "if (cab.ponta & 1u) == 0u && (", "if (", 1)]),
    ("R1 o sub-caminho fechado nao se marca", [(E,
        "flags |= SUB_INICIO | if s.fechado { SUB_FECHADO } else { 0 };", "flags |= SUB_INICIO;", 1)]),
    ("R2 o primeiro troco conta um troco a menos", [(E,
        'pad = u32::try_from(troços).expect("um sub-caminho com mais de 4 mil milhoes de troços");',
        'pad = u32::try_from(troços - 1).expect("x");', 1)]),
]

CMD = ["bash", "scripts/ph2d-run.sh", "cargo", "test", "-p", "ph2d-shape-gpu", "--release", "--test", "it",
       "--", "--ignored", "--nocapture"]
# E a rota do PRODUTO (o tracejado ajustado da casa, a câmara, o `encode`).
CMD2 = ["bash", "scripts/ph2d-run.sh", "cargo", "test", "-p", "ph2d-app-motion", "--release", "--lib",
        "motion_shape_placa::gpu_tests", "--", "--ignored", "--nocapture"]
ENV = dict(os.environ, PH2D_GPU="1", PH2D_GPU_ESPERA="1500")


def corrida():
    p = subprocess.run(CMD, cwd=R, capture_output=True, text=True, env=ENV)
    q = subprocess.run(CMD2, cwd=R, capture_output=True, text=True, env=ENV)
    out = p.stdout + p.stderr + q.stdout + q.stderr
    p.returncode = p.returncode or q.returncode
    pas = fal = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed", out):
        pas += int(m.group(1)); fal += int(m.group(2))
    return p.returncode, pas, fal, "could not compile" not in out, out


def aplica(texto, passos):
    for _f, a, b, n in passos:
        assert texto.count(a) == n, f"ancora {a!r}: {texto.count(a)} != {n}"
        texto = texto.replace(a, b)
    return texto


def placar(out):
    return [l.strip() for l in out.splitlines() if "PLACAR" in l or "PAR " in l or "esticado:" in l or "produto:" in l or "letras:" in l]


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
    for l in placar(out):
        print("   ", l)
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
        for l in placar(out):
            print("   ", l)
    for f in (C, S, E):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
