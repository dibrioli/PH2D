#!/usr/bin/env python3
"""archive-index.py — gera o README de uma pasta de arquivo, derivado do que há nela.

WHY: `docs/archive/estado-2026-08-18/README.md` provou a forma (aviso de que aquilo
NÃO é o estado atual + tabela do que foi recortado + o porquê medido). Mas escrito à
mão ele diverge da pasta no primeiro corte novo — e uma pasta de arquivo cresce a cada
jornada que corta um doc. Este script deriva a tabela do disco.

⚠️ Ele NÃO escreve o "porquê": esse é conteúdo autorado e muda por pasta. O script
preserva tudo acima do marcador `<!-- INDICE-DERIVADO -->` e regenera só abaixo dele.

USO
  python3 scripts/archive-index.py docs/archive/docs-2026-08-18
  python3 scripts/archive-index.py docs/archive/docs-2026-08-18 --check
"""

import os
import subprocess
import sys

MARCA = "<!-- INDICE-DERIVADO -->"


def humano(n: int) -> str:
    return f"{n/1000:.0f} KB" if n < 1_000_000 else f"{n/1_000_000:.2f} MB"


def main() -> None:
    if len(sys.argv) < 2:
        sys.exit("uso: archive-index.py <pasta-de-arquivo> [--check]")
    pasta = sys.argv[1].rstrip("/")
    check = "--check" in sys.argv
    raiz = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True
    ).stdout.strip()
    os.chdir(raiz)
    if not os.path.isdir(pasta):
        sys.exit(f"✗ {pasta} não existe")

    itens = []
    for dirpath, _, arquivos in os.walk(pasta):
        for a in sorted(arquivos):
            # ⚠️ só o README.md da RAIZ da pasta é o índice; um `UI_Bugs/README.md`
            # arquivado é CONTEÚDO. Excluir por nome escondia 3 de 21 arquivos, e o
            # índice de um arquivo que não lista tudo é pior que não existir.
            if not a.endswith(".md") or (a == "README.md" and dirpath.rstrip("/") == pasta):
                continue
            p = os.path.join(dirpath, a)
            n = os.path.getsize(p)
            # o doc VIVO correspondente: procura o mesmo basename fora de archive/
            vivo = subprocess.run(
                ["git", "ls-files", f"*/{a}"], capture_output=True, text=True
            ).stdout.split("\n")
            vivo = [v for v in vivo if v and "archive/" not in v and v.endswith("/" + a)]
            itens.append((os.path.relpath(p, pasta), n, vivo[0] if len(vivo) == 1 else ""))
    itens.sort(key=lambda x: -x[1])

    linhas = [MARCA, "", "## O que está nesta pasta", "",
              "| arquivo (história, verbatim) | tamanho | o doc VIVO de onde saiu |",
              "|---|---:|---|"]
    for rel, n, vivo in itens:
        # ⚠️ os dois lados escapam o espaço. `docs/Vector Module/` e `docs/Motion Nodes/`
        # existem, e um link com espaço literal resolve nuns renderizadores e noutros não —
        # escapar um lado e o outro não é como a inconsistência entra sem ninguém notar.
        alvo = (
            f"[`{os.path.basename(vivo)}`]({os.path.relpath(vivo, pasta).replace(' ', '%20')})"
            if vivo
            else "—"
        )
        linhas.append(f"| [`{rel}`]({rel.replace(' ', '%20')}) | {humano(n)} | {alvo} |")
    total = sum(n for _, n, _ in itens)
    linhas += ["", f"**{len(itens)} arquivos · {humano(total)}** de história fora do caminho quente.", "",
               "> ⚠️ Cada recorte foi feito por `python3 scripts/doc-split.py`, que **aborta se as duas",
               "> metades não remontarem o original byte-a-byte (sha256)**. Nenhuma linha foi editada.",
               ">",
               "> ⛔ As **recusas medidas** que viviam aqui continuam alcançáveis: cada doc vivo leva no",
               "> fim uma tabela `⛔ Recusas MEDIDAS` com o link para a linha exata neste arquivo.",
               "> *Arquivar sem indexar as recusas seria apagá-las.*", ""]
    novo_indice = "\n".join(linhas)

    readme = os.path.join(pasta, "README.md")
    cabecalho = ""
    if os.path.exists(readme):
        atual = open(readme, encoding="utf-8").read()
        cabecalho = atual.split(MARCA)[0] if MARCA in atual else atual.rstrip() + "\n\n"
    if not cabecalho.strip():
        cabecalho = (
            f"# ARQUIVO — {os.path.basename(pasta)}\n\n"
            "> ⚠️ **Isto NÃO é o estado atual de nada.** É história recortada de docs vivos, **verbatim**.\n"
            "> O estado vivo está no doc de origem e no [`CLAUDE.md §5`](../../../CLAUDE.md).\n"
            "> Use para responder *\"por que isto ficou assim?\"* — nunca para decidir a próxima ação.\n\n"
        )
    conteudo = cabecalho + novo_indice

    if check:
        igual = os.path.exists(readme) and open(readme, encoding="utf-8").read() == conteudo
        print(f"{'✓' if igual else '✗'} {readme} {'em dia' if igual else 'desatualizado'}")
        sys.exit(0 if igual else 1)

    open(readme, "w", encoding="utf-8").write(conteudo)
    print(f"✓ {readme} — {len(itens)} arquivos · {humano(total)}")


if __name__ == "__main__":
    main()
