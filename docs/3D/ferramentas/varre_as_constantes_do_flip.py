#!/usr/bin/env python3
"""Varre as TRÊS constantes do flip direccional do pente de topologia.

    python3 docs/3D/ferramentas/varre_as_constantes_do_flip.py [chao|tres]

É o instrumento que escolheu o `CHAO_DO_ALINHAMENTO` em 2026-09-18, depois de o
dono reprovar o smoke com foto (*«pouca ou nenhuma diferença»*). As tabelas que
ele imprime são as que vivem no doc-comment daquela constante e no §80 do
handoff de 17/09 — e ele existe para elas poderem ser RE-MEDIDAS, que é a única
maneira de uma tabela não envelhecer.

⚠️ **Ele MUTA a árvore**: escreve a constante, corre a sonda, restaura — sempre,
mesmo em erro. Cada célula tem `assert` de contagem sobre a agulha, e a corrida
aborta se a sonda não imprimir o que se espera dela.

⛔ **As leituras são de FORMA, não de relógio** (contagens de arestas e de
triângulos), logo não são membros da família de flakes sob fan-out — mas corra-o
com a máquina calma na mesma, porque ele compila cinco vezes.

Modos:
  `chao`  (omissão) — varre o chão de qualidade nos quatro rumos da cena `=49`
                      e, a seguir, na chapa da bancada da lei.
  `tres`            — varre as três constantes (rondas · ganho · chão) num rumo,
                      que é a corrida que mostra QUAL delas é a alavanca.
"""

import math
import pathlib
import re
import subprocess
import sys

RAIZ = pathlib.Path(__file__).resolve().parents[3]
FLIP = RAIZ / "crates/ph2d-mesh/src/dyntopo_flip.rs"

ANCORAS = {
    "rondas": "pub(crate) const MAX_ROUNDS: usize = ",
    "ganho": "const GANHO_DO_ALINHAMENTO: f32 = ",
    "chao": "const CHAO_DO_ALINHAMENTO: f32 = ",
}


def escreve(fonte: str, chave: str, valor: str) -> str:
    """Troca UMA constante, com `assert` de contagem — ver o cabeçalho."""
    ancora = ANCORAS[chave]
    linhas = [i for i, l in enumerate(fonte.splitlines()) if l.startswith(ancora)]
    assert len(linhas) == 1, f"{chave}: {len(linhas)} declaracoes, esperava 1"
    saida = fonte.splitlines()
    saida[linhas[0]] = f"{ancora}{valor};"
    return "\n".join(saida) + "\n"


def corre(pacote: str, sonda: str, linhas_esperadas: int, marca: str):
    r = subprocess.run(
        ["bash", "scripts/ph2d-run.sh", "cargo", "test", "-p", pacote,
         "--release", "--lib", sonda, "--", "--ignored", "--nocapture"],
        cwd=RAIZ, capture_output=True, text=True,
    )
    saida = r.stdout + r.stderr
    assert "test result: ok" in saida, saida[-3000:]
    colhidas = [l for l in saida.splitlines() if marca in l]
    assert len(colhidas) == linhas_esperadas, (
        f"a sonda devolveu {len(colhidas)} linhas, esperava {linhas_esperadas}:\n"
        + saida[-3000:]
    )
    return colhidas


def varre_o_chao(fonte: str) -> None:
    for g in (24, 20, 18, 16, 14, 12, 10, 8):
        FLIP.write_text(escreve(fonte, "chao", f"{-math.cos(math.radians(g)):.6f}"))
        print(f"--- chao {g}°  (a bola da cena =49, quatro rumos)")
        for l in corre("ph2d-app-sculpt3d", "diag_o_chao_por_rumo", 4, "| 0.0|"):
            print("   ", l.split("| 1.0|")[0].split("| 0.0|")[0].strip(),
                  "|", l.split("| 1.0|")[1].strip())
        sys.stdout.flush()
    for g in (24, 18, 16, 14, 12):
        FLIP.write_text(escreve(fonte, "chao", f"{-math.cos(math.radians(g)):.6f}"))
        linhas = corre("ph2d-sculpt3d", "diag_a_escada_do_pente", 13, "pente ")
        # ⚠️ Só os NOVE primeiros degraus: acima de `pente = 1` o `k` satura e a
        # pista não os produz. *Uma janela medida fora do alcance do botão é uma
        # janela sobre outro programa.*
        angs = [float(re.search(r"pior angulo\s+([-\d.]+)", l).group(1)) for l in linhas[:9]]
        q = re.search(r"Q\s+([+-][\d.]+)", linhas[8]).group(1)
        print(f"--- chao {g:>2}°  (a chapa da bancada)  Q no tecto {q}  "
              f"pior angulo MINIMO do curso {min(angs):6.2f}°")
        sys.stdout.flush()


def varre_as_tres(fonte: str) -> None:
    for nome, rondas, ganho, chao in [
        ("hoje", "3", "0.20", 16),
        ("rondas 8", "8", "0.20", 16),
        ("rondas 20", "20", "0.20", 16),
        ("ganho 0,05", "3", "0.05", 16),
        ("chao 12°", "3", "0.20", 12),
        ("chao 8°", "3", "0.20", 8),
    ]:
        s = escreve(fonte, "rondas", rondas)
        s = escreve(s, "ganho", ganho)
        s = escreve(s, "chao", f"{-math.cos(math.radians(chao)):.6f}")
        FLIP.write_text(s)
        linha = corre("ph2d-app-sculpt3d", "diag_o_chao_por_rumo", 4, "| 0.0|")[0]
        print(f"{nome:<12} {linha.split('| 1.0|')[1].strip()}")
        sys.stdout.flush()


if __name__ == "__main__":
    modo = sys.argv[1] if len(sys.argv) > 1 else "chao"
    original = FLIP.read_text()
    try:
        (varre_as_tres if modo == "tres" else varre_o_chao)(original)
    finally:
        FLIP.write_text(original)
        assert FLIP.read_text() == original, "a arvore NAO foi restaurada"
        print("restaurado")
