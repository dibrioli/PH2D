# `CLAUDE.md §5` — a história até 2026-08-18 (arquivo)

> **O que é esta pasta.** O texto do `CLAUDE.md` §5 **verbatim**, como estava em 2026-08-18,
> recortado por módulo. Nada foi editado — só recortado; a reconstrução dos 12 arquivos abaixo
> é **byte-idêntica** ao original (conferida por sha256 no dia do corte).
>
> ⚠️ **Isto NÃO é o estado atual de nada.** O **estado vivo** é o [`CLAUDE.md §5`](../../../CLAUDE.md);
> o **mecanismo** de cada wave é o handoff dela (`docs/<Módulo>/handoffs/`, com índice cronológico
> no `README.md` de cada pasta). Use este arquivo para responder ***"por que isto ficou assim?"***
> — nunca para decidir a próxima ação.

## Por que ele existe

O §5 crescia por **acréscimo**: cada linha que fechava anexava a narrativa inteira da jornada.
Medido em 2026-08-18:

| | |
|---|---|
| `CLAUDE.md` | **917.594 bytes** em 326 commits (era **1.751** em 2026-05-08) |
| §5 | **868.076 bytes — 94,6% do arquivo** |
| maior bullet único | **155.725 bytes** (a entrada de Física, ~50 k tokens numa linha de lista) |
| contexto inicial de um agente | **466 k tokens — ~47% da janela de 1 M**, antes da 1ª palavra do Enio |

E o custo era pago **por todos, sempre**: o `CLAUDE.md` é injetado por inteiro em todo agente,
todo subagente e toda worktree — e **a compactação não o alcança**, porque ele é re-injetado
inteiro em cada janela nova.

Depois do corte: `CLAUDE.md` **40.955 bytes**, §5 **24.737**. A regra que impede a volta está na
[`DIRETRIZ §1.5.9 item 8`](../../IntegracaoMultiAgente/DIRETRIZ.md).

## Os arquivos

| Arquivo | Módulo |
|---|---|
| [motion-nodes.md](motion-nodes.md) | Motion Nodes (208 KB) |
| [painter.md](painter.md) | Painter — layers, pintura, impasto, sculpt do relevo, wet paint (282 KB) |
| [physics.md](physics.md) | Física global — o motor rígido (162 KB) |
| [vector.md](vector.md) | Vector Module (82 KB) |
| [sculpt3d.md](sculpt3d.md) | 3D / Sculpt (58 KB) |
| [timeline.md](timeline.md) | Timeline (40 KB) |
| [audio.md](audio.md) | Áudio (26 KB) |
| [flip.md](flip.md) | Flip (24 KB) |
| [editor-shell.md](editor-shell.md) | Editor/shell — undo, persistência, Sprite Inspector, KTX2 |
| [runtime.md](runtime.md) | Runtime — a saída de sinais |
| [watercolor-removido.md](watercolor-removido.md) | Watercolor/fluid/wash — removidos (histórico) |
| [planos-de-nos.md](planos-de-nos.md) | Planos de nós (waves / carry-overs) |

⚠️ **O que está aqui e em lugar nenhum mais:** as lições **cross-line** escritas durante as
integrações (colisões de número, merges limpos-porém-quebrados) e as entradas
**⛔ MEDIDO E REJEITADO**. Antes de reconstruir qualquer coisa que uma lista aberta do §5
mencione, procure-a aqui: *o resultado honesto de uma varredura é às vezes «isto já foi
tentado, medido e recusado»*.
