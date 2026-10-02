#!/usr/bin/env python3
"""Prova de mutação das arestas no ecrã com células e fundo (doc 121 §9.6): cada mutação tem de
SANGRAR nos gates `contorno_calculado` (o par contra o caminho de sempre, com as famílias conformes,
even-odd, marcas e grandes, e a rota), menos as NOMEADAS.

Controlos: pré-voo (cada âncora casa UMA vez) · corrida LIMPA verde · população > 0 (testes que de
facto correram, lidos do `test result:` — nunca do `running N tests`, que conta os `#[ignore]`).
Restaura por cópia + touch (o cargo guarda o build da mutação se o mtime voltar atrás).
Uso: MUTA_SO_ANCORAS=1 para só o pré-voo. Corre com a placa: `PH2D_GPU=1 bash scripts/ph2d-run.sh`.
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
C = R + "/crates/ph2d-shape-gpu/src/contorno.wgsl"
S = R + "/crates/ph2d-shape-gpu/src/shape.wgsl"

MUTS = [
    ("N1 o fundo sem o prefixo ao longo da linha", C,
     "    for (var r = 0u; r < p.linhas; r += 1u) {\n        let linha = mbase + r * p.celulas * registo;\n        var acc",
     "    for (var r = 0u; r < 0u; r += 1u) {\n        let linha = mbase + r * p.celulas * registo;\n        var acc",
     "sangra"),
    ("N2 a mascara so com a primeira palavra", C,
     "        let palavra = 3u + bl / 32u;", "        let palavra = 3u;", "sangra"),
    ("N3 o fundo na celula que o bloco ainda toca", C,
     "                let i = linha + kb * registo + cat;",
     "                let i = linha + max(kb, 1u) * registo - registo + cat;", "sangra"),
    ("N4 o pixel sem o fundo", S,
     "    var s = vec3<f32>(\n        bitcast<f32>(cmascaras[base]),",
     "    var s = vec3<f32>(\n        0.0 * bitcast<f32>(cmascaras[base]),", "sangra"),
    ("N5 todos os blocos no preenchimento", S,
     "            if bi < fim_f {\n                s.x += v;",
     "            if true {\n                s.x += v;", "sangra"),
    ("N6 a corrente de tras nao desce", C,
     "            contorno_rw[para + i] = contorno_rw[de + i];",
     "            contorno_rw[para + i] = contorno_rw[para + i];", "sangra"),
    ("N7 o limite da contagem sem a junta", C,
     "                n += max(4u, arestas_do_leque(r));", "                n += 0u;", "sangra"),
    ("N8 a rota ao contrario", C,
     "    if !p.eixo && tam.x * tam.y < contas.area_minima_conforme {",
     "    if !p.eixo && tam.x * tam.y >= contas.area_minima_conforme {", "sangra"),
    ("N9 a primeira linha de um bloco arredondada para cima", C,
     "        let ra = u32(clamp(floor(lo.y) - p.y0, 0.0, f32(p.linhas)));",
     "        let ra = u32(clamp(ceil(lo.y) - p.y0, 0.0, f32(p.linhas)));", "sangra"),
    ("N10 a celula do pixel uma a direita", S,
     "    let kx = floor((xy.x - bitcast<f32>(c2.x)) / LARGURA_DA_CELULA);",
     "    let kx = floor((xy.x + LARGURA_DA_CELULA - bitcast<f32>(c2.x)) / LARGURA_DA_CELULA);",
     "sangra"),
]


def corrida():
    p = subprocess.run(
        ["cargo", "test", "-q", "--release", "-p", "ph2d-shape-gpu", "--test", "it", "--",
         "--ignored", "contorno_calculado::"],
        cwd=R, capture_output=True, text=True)
    out = p.stdout + p.stderr
    m = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed", out)
    corridos = int(m.group(1)) + int(m.group(2)) if m else 0
    compilou = "could not compile" not in out
    return p.returncode, corridos, compilou, out


def main():
    for nome, f, a, _b, _e in MUTS:
        c = open(f).read().count(a)
        if c != 1:
            print(f"ABORTA: ancora de {nome} casa {c} vezes")
            sys.exit(2)
    print(f"pre-voo: {len(MUTS)}/{len(MUTS)} ancoras casam uma vez")
    if os.environ.get("MUTA_SO_ANCORAS"):
        return
    rc, n, ok, out = corrida()
    if rc != 0 or n == 0:
        print(f"ABORTA: corrida LIMPA nao esta verde (rc {rc}, {n} testes)")
        print(out[-2000:])
        sys.exit(2)
    print(f"corrida limpa: verde, {n} teste(s)")
    certo = 0
    for nome, f, a, b, esperado in MUTS:
        orig = open(f).read()
        bk = f + ".muta_bk"
        shutil.copy2(f, bk)
        try:
            open(f, "w").write(orig.replace(a, b))
            rc, n, ok, out = corrida()
        finally:
            shutil.copy2(bk, f)
            os.remove(bk)
            os.utime(f, (time.time(), time.time()))
        if not ok:
            veredito = "NAO COMPILA (defeito do arnes)"
        elif n == 0:
            veredito = "ZERO testes corridos (defeito do arnes)"
        else:
            veredito = "SANGRA" if rc != 0 else "SOBREVIVE"
        bate = (veredito == "SANGRA") == (esperado == "sangra") and veredito in ("SANGRA", "SOBREVIVE")
        certo += bate
        msg = re.findall(r"panicked at [^\n]*\n[^\n]*", out)
        print(f"{nome}: {veredito} (esperado {esperado}) {'ok' if bate else 'XX'}  {msg[0][:170] if msg else ''}", flush=True)
    for f in (C, S):
        assert not os.path.exists(f + ".muta_bk"), "restauro falhou"
    print(f"placar: {certo} de {len(MUTS)} como esperado")
    sys.exit(0 if certo == len(MUTS) else 1)


main()
