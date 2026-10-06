#!/usr/bin/env python3
"""Prova de mutação da 5.ª e da 6.ª ondas do doc 121 (§9.20–§9.21): o contacto das formas do Motion pelo motor da casa.

Cada mutação tem de SANGRAR nas suites que ela nomeia:
  MUNDO  `ph2d-contact-world` (lib + tests/it)      (o mundo: renascer, recibo, ângulo, recém-nascidos, bytes)
  PASSO  `ph2d-node-sim-step --lib`                  (o passo: a declaração consumida, a velocidade, o rolamento)
  TACA   `ph2d-node-sim-collide --lib mundo_tests`   (o aperto de mão do `sim.collide`)
  RECUO  `ph2d-app-motion --lib recuo_tests`         (o recuo seguido de Play ao bit, as voltas do Loop)
  RAMPA  `ph2d-app-motion --lib material_demo`       (a bola da `=115` trava na rampa)

Controlos: pré-voo (cada âncora casa o nº esperado de vezes) · corrida LIMPA verde de todas as suites com
população > 0 · mutação que não compila é defeito do arnês · zero testes aborta. Restaura por cópia + touch (o cargo
guarda o build da mutação pelo mtime). ⛔ Sozinho na árvore: nenhuma compilação em paralelo.
Uso: MUTA_SO_ANCORAS=1 só o pré-voo; MUTA_SO=m1,m2 filtra.
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
PONTO = R + "/crates/ph2d-nodegraph/src/cook_checkpoint.rs"
MUNDO = R + "/crates/ph2d-contact-world/src/lib.rs"
PECA = R + "/crates/ph2d-contact-world/src/peca.rs"
ROLAR = R + "/crates/ph2d-contact-world/src/rolar.rs"
COLIDE = R + "/crates/ph2d-node-sim-collide/src/mundo.rs"
PASSO = R + "/crates/ph2d-node-sim-step/src/lib.rs"
PILHA = R + "/crates/ph2d-app-motion/src/motion_state_pilha_demo.rs"

SUITES = {
    "MUNDO": ["cargo", "test", "-p", "ph2d-contact-world"],
    "PASSO": ["cargo", "test", "-p", "ph2d-node-sim-step", "--lib"],
    "TACA": ["cargo", "test", "-p", "ph2d-node-sim-collide", "--lib", "mundo_tests"],
    "RECUO": ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--release", "recuo_tests"],
    "RAMPA": ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--release", "material_demo"],
    "DISCOS": ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--release", "a_pile_of_discs_with_rolling_settles"],
    "DURA": ["cargo", "test", "-p", "ph2d-app-motion", "--lib", "--release", "the_pile_stops_before_the_fall_restarts"],
}

# (nome, suites, [(ficheiro, âncora, substituição, nº de ocorrências)])
MUTS = [
    ("m1 o ponto de recuo sem a memoria", ["RECUO"], [(PONTO,
        "            memo: self\n                .memo\n                .iter()\n"
        "                .map(|(k, m)| (*k, Arc::from(m.clone_box())))\n                .collect(),\n",
        "            memo: BTreeMap::new(),\n", 1)]),
    ("m2 o restauro nao repoe a memoria", ["RECUO"], [(PONTO,
        "        self.memo = cp.memo.iter().map(|(k, m)| (*k, m.clone_box())).collect();\n", "", 1)]),
    ("m3 o mundo nunca renasce", ["MUNDO", "RECUO"], [(MUNDO,
        "    let continua = mundo.as_ref().is_some_and(|m| {",
        "    let continua = mundo.is_some() || mundo.as_ref().is_some_and(|m| {", 1)]),
    ("m4 o sim.collide ignora o recibo", ["TACA"], [(COLIDE,
        "        Some(c) if obstaculo::tem_recibo(s, chave) =>",
        "        Some(c) if false && obstaculo::tem_recibo(s, chave) =>", 1)]),
    ("m5 o passo nao consome a declaracao", ["PASSO"], [(PASSO,
        "            || ph2d_contact::obstaculo::e_da_porta(name);", "            || false;", 1)]),
    ("m6 a estabilizacao da omissao", ["PASSO"], [(MUNDO,
        "pub const ESTABILIZACOES: usize = 4;", "pub const ESTABILIZACOES: usize = 1;", 1)]),
    ("m7 a trava estatica nunca fecha", ["RAMPA"], [(MUNDO,
        "                    c.presa = true;\n", "                    c.presa = false;\n", 1)]),
    ("m8 o angulo do rapier sem acumular", ["MUNDO"], [(MUNDO,
        "                estado.rot_antes[i] + delta * GRAUS\n",
        "                b.rotation().angle() * GRAUS\n", 1)]),
    ("m9 o recem-nascido anda no 1.o tique", ["MUNDO"], [(MUNDO,
        "        if pedido.dt[i] < dt && m.corpos[i].is_none() {",
        "        if false && pedido.dt[i] < dt && m.corpos[i].is_none() {", 1)]),
    ("m10 os bytes do mundo sem o factor", ["MUNDO"], [(MUNDO,
        "const FACTOR: usize = 3;", "const FACTOR: usize = 1;", 1)]),
    ("m11 o obstaculo apagado fica no mundo", ["MUNDO"], [(MUNDO,
        "    for k in saem {\n", "    for k in saem.iter().copied().filter(|_| false) {\n", 1)]),
    ("m12 o travao a rolar de sinal trocado", ["PASSO"], [(ROLAR,
        "    if j.is_finite() { -j / dt } else { 0.0 }", "    if j.is_finite() { j / dt } else { 0.0 }", 1)]),
    ("m13 o atrito do par pela media", ["PASSO"], [(PECA,
        "        Some(m) => (m.atrito, m.salto, CoefficientCombineRule::GeometricMean),",
        "        Some(m) => (m.atrito, m.salto, CoefficientCombineRule::Max),", 1)]),
    # A 6.ª onda (06/10, a continuação): a trava destranca ao 2.º excesso, e o mundo some sem colisor.
    ("m14 a trava destranca ao 1.o excesso", ["DISCOS"], [(ROLAR,
        "pub(crate) const EXCESSOS_PARA_SOLTAR: u8 = 2;", "pub(crate) const EXCESSOS_PARA_SOLTAR: u8 = 1;", 1)]),
    ("m15 o mundo fica sem colisor", ["RECUO"], [(MUNDO,
        "        *mundo = None;\n        return None;\n", "        return None;\n", 1)]),
    # O report de 06/10 («a animação não dura o suficiente»): a queda dura o que a pilha leva a assentar.
    ("m16 a queda de 256 dura os 3 s do smoke", ["DURA"], [(PILHA,
        "        Some(k) if k >= 16.0 => 8.0,", "        Some(k) if k >= 16.0 => DURACAO,", 1)]),
]


def corrida(suites):
    out, rc, pas, fal = "", 0, 0, 0
    for s in suites:
        p = subprocess.run(["bash", "scripts/ph2d-run.sh"] + SUITES[s], cwd=R, capture_output=True, text=True)
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
    if rc != 0 or p == 0 or f:
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
    for f in (PONTO, MUNDO, PECA, ROLAR, COLIDE, PASSO, PILHA):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {sangrou} de {len(muts)} sangraram")


main()
