#!/usr/bin/env python3
"""Prova de mutação da 4.ª onda do doc 121 (§9.19): (3) a ponta e a junta no cartão da forma e (5) os pares dos
impulsos pela grelha e a taça da porta de medição da `=114`.

Cada mutação tem de SANGRAR nas suites que ela nomeia:
  NO     `ph2d-node-motion-shape --lib`              (a declaração, a chave, a leitura e os rótulos)
  CARTAO `ph2d-app-motion --lib cap_and_join`        (o `StrokeSpec`, os rótulos resolvidos, a ida e volta)
  TACA   `ph2d-app-motion --lib a_taca_da_medida`    (a taça contém a grelha; o smoke ao bit)
  BITS   `ph2d-contact --lib os_impulsos_pela_grelha` (os impulsos pela grelha = todos-os-pares, ao bit)
  PLACA  `ph2d-app-motion --lib --ignored a_ponta_e_a_junta_do_cartao` na RTX (a imagem, placa × rota Vello)

Controlos: pré-voo (cada âncora casa o nº esperado de vezes) · corrida LIMPA verde de todas as suites com
população > 0 · mutação que não compila é defeito do arnês · zero testes aborta. Restaura por cópia + touch (o cargo
guarda o build da mutação pelo mtime). ⛔ Sozinho na árvore: nenhuma compilação em paralelo.
Uso: MUTA_SO_ANCORAS=1 só o pré-voo; MUTA_SO=c1,i2 filtra.
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
ESTILO = R + "/crates/ph2d-node-motion-shape/src/stroke_style.rs"
PARAM = R + "/crates/ph2d-node-motion-shape/src/param.rs"
TRACO = R + "/crates/ph2d-app-motion/src/motion_shape_traco.rs"
GEN = R + "/crates/ph2d-app-motion/src/motion_shape_gen.rs"
IMP = R + "/crates/ph2d-contact/src/impulso.rs"
PILHA = R + "/crates/ph2d-app-motion/src/motion_state_pilha_demo.rs"

SUITES = {
    "NO": ["cargo", "test", "-p", "ph2d-node-motion-shape", "--lib"],
    "CARTAO": ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--profile", "smoke", "cap_and_join"],
    "TACA": ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--profile", "smoke", "a_taca_da_medida"],
    "BITS": ["cargo", "test", "-p", "ph2d-contact", "--lib", "os_impulsos_pela_grelha"],
    "PLACA": ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--profile", "smoke", "--", "--ignored",
              "--test-threads=1", "a_ponta_e_a_junta_do_cartao"],
}
RTX = "/usr/share/vulkan/icd.d/nvidia_icd.json"

# (nome, suites, [(ficheiro, âncora, substituição, nº de ocorrências)])
MUTS = [
    ("c1 a ponta redonda chega rente", ["CARTAO", "PLACA"], [(TRACO,
        "        StrokeCap::Round => LineCap::Round,\n", "        StrokeCap::Round => LineCap::Butt,\n", 1)]),
    ("c2 a junta chanfro chega esquadria", ["CARTAO", "PLACA"], [(TRACO,
        "        StrokeJoin::Bevel => LineJoin::Bevel,\n", "        StrokeJoin::Bevel => LineJoin::Miter,\n", 1)]),
    ("c3 o indice da ponta trocado", ["NO"], [(ESTILO,
        "            1 => Self::Round,\n            2 => Self::Square,\n",
        "            1 => Self::Square,\n            2 => Self::Round,\n", 1)]),
    ("c4 a ponta fora da chave", ["NO"], [(PARAM, "    STROKE_CAP,\n    STROKE_JOIN,\n];", "    STROKE_JOIN,\n];", 1)]),
    ("c5 a omissao da ponta redonda", ["NO", "CARTAO"], [(PARAM,
        "        name: STROKE_CAP,\n        default: 0.0,\n", "        name: STROKE_CAP,\n        default: 1.0,\n", 1)]),
    ("c6 o build_shape_path ignora a ponta e a junta", ["CARTAO", "PLACA"], [(GEN,
        "        (spec.cap, spec.join) = mistura::traco::ponta_e_junta(st);\n", "", 1)]),
    ("c7 os rotulos da junta trocados", ["NO"], [(ESTILO,
        '    "node.opts.node_motion_shape.join_labels.1",\n    "node.opts.node_motion_shape.join_labels.2",\n',
        '    "node.opts.node_motion_shape.join_labels.2",\n    "node.opts.node_motion_shape.join_labels.1",\n', 1)]),
    ("i1 o par consigo proprio", ["BITS"], [(IMP, "            if hi <= lo || !ativo[hi] {\n",
        "            if hi < lo || !ativo[hi] {\n", 1)]),
    ("i2 a grelha perde vizinhos", ["BITS"], [(IMP, "            grade.vizinhos_de(lo, &mut viz);\n",
        "            grade.vizinhos_de(lo, &mut viz);\n            viz.retain(|&h| h % 5 != 0);\n", 1)]),
    ("i3 a grelha pelo alcance a metade", ["BITS"], [(IMP,
        "    grade.planeia(p, &ativo, &alcances);\n",
        "    let alcances: Vec<f32> = alcances.iter().map(|a| a * 0.5).collect();\n"
        "    grade.planeia(p, &ativo, &alcances);\n", 1)]),
    ("p1 a taca nao cresce com a pilha", ["TACA"], [(PILHA,
        "k * GAP * 0.75 + (ALTURA - TACA_Y) + 0.2", "k * GAP * 0.25 + (ALTURA - TACA_Y) + 0.2", 1)]),
    ("p2 a cena do smoke muda", ["TACA"], [(PILHA, "lado.map_or(TACA_R, |k|", "lado.map_or(TACA_R * 1.01, |k|", 1)]),
]

BASE_ENV = dict(os.environ, PH2D_GPU="1", PH2D_GPU_ESPERA="1500", VK_ICD_FILENAMES=RTX)


def corrida(suites):
    out, rc, pas, fal = "", 0, 0, 0
    for s in suites:
        p = subprocess.run(["bash", "scripts/ph2d-run.sh"] + SUITES[s], cwd=R, capture_output=True, text=True,
                           env=BASE_ENV)
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
    for nome, _s, passos in MUTS:
        textos = {}
        for f, a, b, n in passos:
            t = textos.setdefault(f, open(f).read())
            if t.count(a) != n:
                print(f"ABORTA: ancora de {nome} casa {t.count(a)} vezes (esperado {n})"); sys.exit(2)
            textos[f] = t.replace(a, b)
    print(f"pre-voo: {len(MUTS)}/{len(MUTS)} mutacoes com todas as ancoras a casar", flush=True)
    if os.environ.get("MUTA_SO_ANCORAS"):
        return
    rc, p, f, ok, out = corrida(list(SUITES))
    if rc != 0 or p == 0 or f or "a_ponta_e_a_junta_do_cartao_desenham_como_a_placa ... ok" not in out:
        print(f"ABORTA: corrida LIMPA nao esta verde (rc {rc}, {p} passed, {f} failed)")
        print(out[-2500:]); sys.exit(2)
    print(f"corrida limpa: verde, {p} passed", flush=True)
    sangrou = 0
    for nome, suites, passos in muts:
        ficheiros = sorted({x[0] for x in passos})
        origs = {f: open(f).read() for f in ficheiros}
        try:
            for f in ficheiros:
                shutil.copy2(f, f + ".muta_bk")
                open(f, "w").write(aplica(origs[f], [x for x in passos if x[0] == f]))
            rc, p, fl, ok, out = corrida(suites)
        finally:
            for f in ficheiros:
                open(f, "w").write(origs[f])
                os.remove(f + ".muta_bk") if os.path.exists(f + ".muta_bk") else None
                os.utime(f, (time.time(), time.time()))
        if not ok:
            v = "NAO COMPILA (defeito do arnes)"
        elif p + fl == 0:
            v = "ZERO testes (defeito do arnes)"
        elif rc != 0 or fl:
            v = "SANGROU"
        else:
            v = "SOBREVIVEU"
        falhos = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
        print(f"{nome} {suites}: {v} ({p} passed, {fl} failed) reprovou: {falhos}", flush=True)
        sangrou += v == "SANGROU"
    for f in (ESTILO, PARAM, TRACO, GEN, IMP, PILHA):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
