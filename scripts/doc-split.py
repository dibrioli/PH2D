#!/usr/bin/env python3
"""doc-split.py — corta um doc gigante em VIVO + ARQUIVO, provando que nada se perdeu.

WHY this exists (medido 2026-08-18)
-----------------------------------
O `CLAUDE.md` foi de 1,7 KB a 917 KB por append, e a cura («a narrativa vai para o
handoff») apenas REALOCOU a doença: o `HANDOFF_line_physics.md` chegou a 710 KB,
77% do que o §5 chegou a ser. Medido no repo inteiro: 4,12 MB de história em 206
docs vivos (42%), e 2,15 MB só nos 23 arquivos acima de 80 KB (61%).

O joelho está entre 80 e 110 KB: a `DIRETRIZ.md` (86 KB) é o doc mais lido do repo
e ainda cabe num `Read`; acima disso o `Read` desaparece e o acesso vira raspagem
por shell — o tracker de física teve **1 `Read` para 407 comandos**, e **89% dele
nunca entrou em contexto nenhum**. Uma regra na linha 8.000 não é «difícil de
achar»: ela não é lida por ninguém.

A LEI QUE ESTE SCRIPT IMPÕE
---------------------------
⚠️ **Toda linha do original termina no VIVO ou no ARQUIVO, exatamente uma vez, e
verbatim.** O script reconstrói o original a partir das duas metades e compara o
**sha256**. Se não bater, ele aborta sem escrever nada.

Isto não é zelo: cortar um doc de 11.400 linhas à mão é como o histórico se perde,
e o histórico é o único lugar onde vive *«medido e REJEITADO — não refaça»* (47
ocorrências só no log de perf do Painter). Perder isso custa reconstruir o que já
foi medido e recusado.

USO
---
  python3 scripts/doc-split.py <arquivo.md> --keep 1-260,1680-1885 \\
      --archive docs/archive/tracker-physics-2026-08-18/HANDOFF_line_physics.md

  --dry-run   mostra o plano e os tamanhos, não escreve nada
  --keep      faixas de linhas 1-indexadas, INCLUSIVAS, separadas por vírgula

O cabeçalho novo do doc vivo NÃO é escrito por aqui de propósito: ele é conteúdo
autorado (o roteador do módulo), e escrevê-lo por script daria um roteador genérico.
O script preserva as faixas mantidas; o cabeçalho vem por `Edit` depois.
"""

import argparse
import hashlib
import os
import sys


def parse_ranges(spec: str, n: int) -> list[tuple[int, int]]:
    out = []
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            a, b = part.split("-", 1)
            lo, hi = int(a), int(b)
        else:
            lo = hi = int(part)
        if not (1 <= lo <= hi <= n):
            sys.exit(f"✗ faixa {part} fora de 1..{n}")
        out.append((lo, hi))
    out.sort()
    # ⚠️ sobreposição faria uma linha aparecer nos DOIS lados e a verificação de
    # sha passaria mesmo assim (o remontador usa índices). Barrar aqui.
    for i in range(1, len(out)):
        if out[i][0] <= out[i - 1][1]:
            sys.exit(f"✗ faixas sobrepostas: {out[i-1]} e {out[i]}")
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("arquivo")
    ap.add_argument("--keep", required=True)
    ap.add_argument("--archive", required=True)
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    raw = open(args.arquivo, "rb").read()
    texto = raw.decode("utf-8")
    linhas = texto.splitlines(keepends=True)
    n = len(linhas)

    faixas = parse_ranges(args.keep, n)
    manter = set()
    for lo, hi in faixas:
        manter.update(range(lo, hi + 1))

    vivo = [linhas[i - 1] for i in range(1, n + 1) if i in manter]
    arquivo = [linhas[i - 1] for i in range(1, n + 1) if i not in manter]

    # ---- a prova: as duas metades remontam o original, byte a byte -------------
    remontado = []
    iv = ia = 0
    for i in range(1, n + 1):
        if i in manter:
            remontado.append(vivo[iv])
            iv += 1
        else:
            remontado.append(arquivo[ia])
            ia += 1
    if hashlib.sha256("".join(remontado).encode()).hexdigest() != hashlib.sha256(raw).hexdigest():
        sys.exit("✗ ABORTADO: a remontagem não bate com o original (sha256)")

    b_vivo = len("".join(vivo).encode())
    b_arq = len("".join(arquivo).encode())
    print(f"  original : {len(raw):>9} B · {n:>6} linhas")
    print(f"  VIVO     : {b_vivo:>9} B · {len(vivo):>6} linhas  ({100*b_vivo/len(raw):.1f}%)  faixas {args.keep}")
    print(f"  ARQUIVO  : {b_arq:>9} B · {len(arquivo):>6} linhas  ({100*b_arq/len(raw):.1f}%)")
    print("  sha256   : remontagem IDÊNTICA ao original ✓")

    if args.dry_run:
        print("  (dry-run — nada escrito)")
        return

    os.makedirs(os.path.dirname(args.archive), exist_ok=True)
    rel_vivo = os.path.relpath(args.arquivo, os.path.dirname(args.archive))
    cab = (
        f"# ARQUIVO — {os.path.basename(args.arquivo)} (história, {len(arquivo)} linhas)\n\n"
        f"> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de\n"
        f"> [`{os.path.basename(args.arquivo)}`]({rel_vivo}) em 2026-08-18, **verbatim** — nenhuma\n"
        f"> linha foi editada, e a remontagem das duas metades bate sha256 com o original.\n"
        f">\n"
        f"> Use para responder *\"por que isto ficou assim?\"* — **nunca** para decidir a próxima\n"
        f"> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).\n"
        f">\n"
        f"> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma\n"
        f"> recusa com medição atrás não volta à fila por ter mudado de arquivo.\n"
        f">\n"
        f"> Recorte: linhas fora de `{args.keep}` do original.\n\n"
        f"---\n\n"
    )
    open(args.archive, "w", encoding="utf-8").write(cab + "".join(arquivo))
    open(args.arquivo, "w", encoding="utf-8").write("".join(vivo))
    print(f"  → {args.archive}")
    print(f"  → {args.arquivo}")


if __name__ == "__main__":
    main()
