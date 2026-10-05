#!/usr/bin/env python3
"""Prova de mutação da cura do doc 121 §9.18 (C): a rota Vello do Motion preenche os polígonos da placa — a porta CPU
da lei (`contorno_cpu.rs`), o traço pela lei (`motion_shape_traco.rs`) e o gancho da porta de lote (`instance.rs`).
Cada mutação tem de SANGRAR no gate `a_rota_vello_traceja_o_pedaco_rente_como_a_placa` (rota Vello × placa, alfa `≤ 1`)
ou nos testes da porta CPU (`ph2d-shape-gpu --lib contorno_cpu`), na placa que ela nomeia.

Controlos: pré-voo (cada âncora casa o nº esperado de vezes) · corrida LIMPA verde (as duas suites, as duas placas) com
população > 0 · mutação que não compila é defeito do arnês · zero testes aborta. Restaura por cópia + touch (o cargo
guarda o build da mutação pelo mtime). ⛔ Sozinho na árvore: nenhuma compilação em paralelo.
Uso: MUTA_SO_ANCORAS=1 só o pré-voo; MUTA_SO=m1,m6 filtra.
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
CPU = R + "/crates/ph2d-shape-gpu/src/contorno_cpu.rs"
TR = R + "/crates/ph2d-app-motion/src/motion_shape_traco.rs"
INS = R + "/crates/ph2d-vec-render/src/instance.rs"

GATE = ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--profile", "smoke", "--", "--ignored",
        "--test-threads=1", "pedaco_rente"]
UNIT = ["cargo", "test", "-p", "ph2d-shape-gpu", "--lib", "--profile", "smoke", "contorno_cpu"]
ICD = {
    "rtx": "/usr/share/vulkan/icd.d/nvidia_icd.json",
    "igpu": "/usr/share/vulkan/icd.d/radeon_icd.json",
}

# (nome, placas, [(ficheiro, âncora, substituição, nº de ocorrências)])
MUTS = [
    ("m1 a cura desligada (a casa traca)", ["rtx"], [(TR,
        "        let forma = ph2d_vec_render::forma_para_a_placa(path)?;\n",
        "        let forma = ph2d_vec_render::forma_para_a_placa(path).filter(|_| false)?;\n", 1)]),
    ("m2 a conforme pelo eixo no ecra", ["rtx"], [(TR, "        if n.conforme {\n", "        if false {\n", 1)]),
    ("m3 a esticada pelas marcas", ["rtx"], [(TR, "        if n.conforme {\n", "        if true {\n", 1)]),
    ("m4 o nivel fixo no mais grosso", ["rtx"], [(TR,
        "        let n = ph2d_shape_gpu::nivel_da_copia(&g.tol, lin);\n",
        "        let n = ph2d_shape_gpu::NivelDaCopia {\n            nivel: 0,\n"
        "            ..ph2d_shape_gpu::nivel_da_copia(&g.tol, lin)\n        };\n", 1)]),
    ("m5 o ajuste do tracejado a 1", ["rtx"], [(CPU,
        "    let ajuste = s.ajuste_do_tracejado();\n", "    let ajuste = 1.0_f32 + 0.0 * s.ajuste_do_tracejado();\n", 1)]),
    ("m6 a emenda do fechado nunca", ["rtx"], [(CPU,
        "            sub.emenda = sub.a_fim < s0 && s0 < sub.a_fim + sub.tr;\n", "            sub.emenda = false;\n", 1)]),
    ("m7 o sentido do quadrilatero trocado", ["rtx"], [(CPU,
        "            self.poligono(&[a, b, c, d]);\n        } else {\n            self.poligono(&[a, d, c, b]);",
        "            self.poligono(&[a, d, c, b]);\n        } else {\n            self.poligono(&[a, b, c, d]);", 1)]),
    ("m8 a faixa nunca serve", ["rtx"], [(CPU,
        "        if (!quina && dot(m, m) > fora * fora) || recuo > recuo_max {\n",
        "        if fora > 0.0 || recuo > recuo_max {\n", 1)]),
    ("m9 o leque grosso", ["rtx"], [(CPU, "const FLECHA: f32 = 0.25;\n", "const FLECHA: f32 = 1.0;\n", 1)]),
    ("m10 o traco sem a ponta de inicio", ["rtx"], [(CPU,
        "            self.tampa(q0, -u, r, (it.ponta >> 6) & 3);\n", "", 1)]),
    ("m11 o arco pela corda", ["rtx"], [(CPU,
        "        c + 8.0 * h * h / (3.0 * c)\n", "        c + 0.0 * h\n", 1)]),
    ("m12 a casa traca por cima da lei", ["rtx"], [(INS, "        if !proprio {\n", "        if !proprio || true {\n", 1)]),
]

BASE_ENV = dict(os.environ, PH2D_GPU="1", PH2D_GPU_ESPERA="1500")


def corrida(placa):
    env = dict(BASE_ENV, VK_ICD_FILENAMES=ICD[placa])
    out, rc, pas, fal = "", 0, 0, 0
    for cmd in (UNIT, GATE):
        p = subprocess.run(["bash", "scripts/ph2d-run.sh"] + cmd, cwd=R, capture_output=True, text=True, env=env)
        out += p.stdout + p.stderr
        rc = rc or p.returncode
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
    for placa in ("rtx", "igpu"):
        rc, p, f, ok, out = corrida(placa)
        if rc != 0 or p == 0 or f or "pedaco_rente_como_a_placa ... ok" not in out:
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
            if not ok:
                v = "NAO COMPILA (defeito do arnes)"
            elif p + fl == 0:
                v = "ZERO testes (defeito do arnes)"
            elif rc != 0 or fl:
                v = "SANGROU"
            else:
                v = "SOBREVIVEU"
            falhos = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
            alfa = re.findall(r"pedaço rente: alfa max (\d+)", out)
            print(f"{nome} [{placa}]: {v} ({p} passed, {fl} failed) reprovou: {falhos} alfa {alfa}", flush=True)
        sangrou += all(r[1] != 0 or r[3] for r in res) and all(r[4] for r in res)
    for f in (CPU, TR, INS):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
