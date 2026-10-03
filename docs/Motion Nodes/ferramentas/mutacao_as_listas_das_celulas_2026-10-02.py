#!/usr/bin/env python3
"""⛔ APOSENTADO em 2026-10-03: as listas saíram com o buffer de acumulação (doc 121 §9.12); as âncoras
deste arnês já não existem. O sucessor é `mutacao_o_buffer_de_acumulacao_2026-10-03.py` (17/17).

Prova de mutação das listas de arestas das células (doc 121 §9.8): cada mutação tem de SANGRAR no
gate de GPU de `ph2d-shape-gpu` (7 testes `--ignored`: paridade com o Vello, contorno par contra par,
fileiras que não cabem, cena que muda, quina exacta, NaN vertical, rota).

Placar de 2026-10-02: 14/14. A M10 sobreviveu à 1.ª corrida (com 6 testes): num passe NOVO a reserva
não escrita é zero e soma nada; quem a mata é `uma_cena_que_muda_nao_le_as_arestas_do_quadro_anterior`,
escrito por causa dela.

Controlos: pré-voo (cada âncora casa o nº esperado de vezes — 1, ou 2 onde a mutação toca as duas
passagens) · corrida LIMPA verde com população > 0 (lida do `test result:`) · mutação que não compila
é defeito do arnês · zero testes aborta. Restaura por cópia + touch (o cargo guarda o build da mutação
pelo mtime). Uso: MUTA_SO_ANCORAS=1 só o pré-voo; MUTA_SO=M1,M5 filtra.
"""
import os, re, shutil, subprocess, sys, time

R = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
C = R + "/crates/ph2d-shape-gpu/src/contorno.wgsl"
S = R + "/crates/ph2d-shape-gpu/src/shape.wgsl"

# (nome, [(ficheiro, âncora, substituição, nº de ocorrências)])
MUTS = [
    ("M1 o fundo somado na celula ka e nao na kb", [(C,
        "q0 + ks.y * REGISTO + a.fam]", "q0 + ks.x * REGISTO + a.fam]", 1)]),
    ("M2 a lista comeca uma celula depois (conta e escreve)", [(C,
        "for (var k = ks.x; k < ks.y; k += 1u) {", "for (var k = ks.x + 1u; k < ks.y; k += 1u) {", 2)]),
    ("M3 a lista acaba uma celula antes (conta e escreve; `k + 1u < ks.y` para nao dar a volta em ks.y = 0)", [(C,
        "for (var k = ks.x; k < ks.y; k += 1u) {", "for (var k = ks.x; k + 1u < ks.y; k += 1u) {", 2)]),
    ("M4 o fragmento ignora SEM_LISTA", [(S,
        "de_sempre = s.w == 0.0;", "de_sempre = false;", 1)]),
    ("M5 cs_lugar_das_listas sem o prefixo do fundo", [(C,
        "fundo += vec3<i32>(", "fundo = vec3<i32>(", 1)]),
    ("M6 o cursor das marcas no inicio da fileira da familia", [(C,
        "atomicStore(&ccelulas_rw[q + 5u], acc + nf);", "atomicStore(&ccelulas_rw[q + 5u], acc);", 1)]),
    ("M7 cs_escreve_listas nao salta SEM_LISTA", [(C,
        "if atomicLoad(&ccelulas_rw[q0 + 3u]) == SEM_LISTA {", "if false {", 1)]),
    ("M8 o fragmento divide por 2 * ESCALA_FIXA", [(S,
        "return vec4<f32>(vec3<f32>(s) / ESCALA_FIXA, 1.0);",
        "return vec4<f32>(vec3<f32>(s) / (2.0 * ESCALA_FIXA), 1.0);", 1)]),
    ("M9 na_fileira com min/max trocados", [(C,
        "return vec3<f32>(y0 - y1, min(e.x, e.z), max(e.x, e.z));",
        "return vec3<f32>(y0 - y1, max(e.x, e.z), min(e.x, e.z));", 1)]),
    ("M10 aresta_de aceita as arestas reservadas e nao escritas", [(C,
        "if bl >= c0.y + c0.z + c0.w {", "if false {", 1)]),
    ("M11 fileiras_da_aresta com floor no maximo", [(C,
        "ceil(max(a.e.y, a.e.w))", "floor(max(a.e.y, a.e.w))", 1)]),
    ("M12 cs_zera nao apaga nada", [(C,
        "atomicStore(&ccelulas_rw[f.registo + k], 0u);", "_ = atomicLoad(&ccelulas_rw[f.registo + k]);", 1)]),
    ("M13 sem a verificacao da capacidade", [(C,
        "if inicio > contas.cap_listas || total > contas.cap_listas - inicio {", "if false {", 1)]),
    ("M14 marcas e contorno trocados em cobertura_de_ecra", [
        (S, "s.y += fixo(contribuicao(e.xy, e.zw, xy));", "s.QQ += fixo(contribuicao(e.xy, e.zw, xy));", 1),
        (S, "s.z += fixo(contribuicao(e.xy, e.zw, xy));", "s.y += fixo(contribuicao(e.xy, e.zw, xy));", 1),
        (S, "s.QQ += fixo(", "s.z += fixo(", 1)]),
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
    # `QQ` é um marcador temporário (a troca do M14 faz-se em três passos); a contagem é por passo.
    for _f, a, b, n in passos:
        assert texto.count(a) == n, f"ancora {a!r}: {texto.count(a)} != {n}"
        texto = texto.replace(a, b)
    return texto


def main():
    so = os.environ.get("MUTA_SO")
    muts = [m for m in MUTS if not so or m[0].split()[0] in so.split(",")]
    for nome, passos in MUTS:
        # pré-voo: cada âncora sobre o texto do ficheiro, passo a passo (encadeado, como na aplicação)
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
    print(f"corrida limpa: verde, {p} passed")
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
