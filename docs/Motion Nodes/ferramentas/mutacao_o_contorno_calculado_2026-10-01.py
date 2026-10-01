#!/usr/bin/env python3
"""Prova de mutação do contorno calculado (doc 121 §9.5): cada mutação tem de SANGRAR no gate
`o_contorno_calculado_desenha_o_que_o_eixo_desenha`, menos as NOMEADAS (só relógio).

Controlos: pré-voo (cada âncora casa UMA vez) · corrida LIMPA verde · população > 0 (testes que de
facto correram, lidos do `test result:`). Restaura por cópia + touch (o cargo guarda o build da
mutação se o mtime voltar atrás).
Uso: MUTA_SO_ANCORAS=1 para só o pré-voo.
"""
import os, re, shutil, subprocess, sys, time

R = "/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value"
W = R + "/crates/ph2d-shape-gpu/src/contorno.wgsl"
S = R + "/crates/ph2d-shape-gpu/src/shape.wgsl"
RS = R + "/crates/ph2d-shape-gpu/src/contorno.rs"

MUTS = [
    ("M1 o contorno nunca e' escrito", W,
     "    if total > n || total + SEGS_POR_BLOCO <= n {",
     "    if true {", "sangra"),
    ("M2 sem a aresta de ponta da frente", W,
     "        if !faixa1 {\n            aresta(q1, q2, s);\n        }",
     "", "sangra"),
    ("M3 todo bloco lido como encadeado", W,
     "select(0.0, 1.0, corrente)", "1.0", "sangra"),
    ("M4 o telescopio ao contrario", S,
     "s += clamp(ex.x - xy.y, 0.0, 1.0) - clamp(ex.y - xy.y, 0.0, 1.0);",
     "s += clamp(ex.y - xy.y, 0.0, 1.0) - clamp(ex.x - xy.y, 0.0, 1.0);", "sangra"),
    ("M5 a capacidade nao cresce", RS,
     "self.total_visto = self.total_visto.max(u64::from(total));",
     "let _ = total;", "sangra"),
    ("M6 o lado de baixo na corrente da frente (SO RELOGIO, nomeada)", W,
     "aresta_em(q2, q3, s, s);", "aresta_em(q2, q3, s, false);", "sobrevive"),
]

def corrida():
    p = subprocess.run(
        ["cargo", "test", "-q", "--release", "-p", "ph2d-shape-gpu", "--test", "it", "--",
         "--ignored", "--exact", "contorno_calculado::o_contorno_calculado_desenha_o_que_o_eixo_desenha"],
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
            print(f"ABORTA: ancora de {nome} casa {c} vezes"); sys.exit(2)
    print(f"pre-voo: {len(MUTS)}/{len(MUTS)} ancoras casam uma vez")
    if os.environ.get("MUTA_SO_ANCORAS"):
        return
    rc, n, ok, out = corrida()
    if rc != 0 or n == 0:
        print(f"ABORTA: corrida LIMPA nao esta verde (rc {rc}, {n} testes)"); print(out[-2000:]); sys.exit(2)
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
            shutil.copy2(bk, f); os.remove(bk); os.utime(f, (time.time(), time.time()))
        if not ok:
            veredito = "NAO COMPILA (defeito do arnes)"
        elif n == 0:
            veredito = "ZERO testes corridos (defeito do arnes)"
        else:
            veredito = "SANGRA" if rc != 0 else "SOBREVIVE"
        bate = (veredito == "SANGRA") == (esperado == "sangra") and veredito in ("SANGRA", "SOBREVIVE")
        certo += bate
        msg = re.findall(r"panicked at [^\n]*\n[^\n]*", out)
        print(f"{nome}: {veredito} (esperado {esperado}) {'ok' if bate else 'XX'}  {msg[0][:160] if msg else ''}")
    assert open(W).read().count("select(0.0, 1.0, corrente)") == 1, "restauro falhou"
    print(f"placar: {certo} de {len(MUTS)} como esperado")
    sys.exit(0 if certo == len(MUTS) else 1)

main()
