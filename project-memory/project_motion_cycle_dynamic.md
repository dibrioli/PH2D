---
name: project_motion_cycle_dynamic
description: A DINÂMICA DOS CICLOS do Motion Nodes (Enio, 2026-09-05, vale até terminarmos) — um grupo de nós por ciclo, escolhido já com o tutorial em mente; redesenho + upgrade; PDF de tutorial; O SMOKE É O TUTORIAL; params no cartão e o painel lateral SAI
metadata:
  type: project
---

**Ordem do Enio, 2026-09-05 — vale até o módulo terminar** (protocolo inteiro:
`docs/Motion Nodes/103_dinamica_dos_ciclos.md`):

A cada ciclo: (1) escolher um GRUPO de nós **já pensando no tutorial**; (2) auditar contra o
estado da arte; (3) **redesenhar o cartão** (params NO cartão, beleza do Mini Cavalry);
(4) **upgrade** dos nós (poder + device); (5) **medir**; (6) escrever o **tutorial em PDF** em
`docs/Motion Nodes/tutoriais/`; (7) **o SMOKE É o tutorial** — o Enio segue o PDF do princípio
ao fim.

⭐ **Duas decisões de produto dele, no mesmo dia:** os params dos nós vivem **no cartão** (como
no Blender) e **o painel lateral sai**; e *«somos uma game engine, precisamos de performance»* —
todo nó de um ciclo diz onde corre (device/CPU) e porquê.

⚠️ **A minha recusa medida contra «params no cartão» respondia a OUTRA pergunta e está
REVOGADA** (doc 101 §4 → doc 103 §4): eu media *altura do cartão ÷ altura do ecrã* (48–80 % do
iPad mini) quando a pergunta era *o que custa área PERMANENTE* — o painel de params ocupa
`chrome/inspector-w = 304 px` **absolutos** (22,3 % / 25,5 % / **26,8 %** da largura nos três
tablets, em todo quadro), e um cartão alto custa altura só onde se trabalha, num canvas
infinito que dobra e tem zoom. *A decisão dele é a melhor das duas.* Ver
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]].

**PDF (ferramenta MEDIDA em 2026-09-05):** não há `typst`/`pandoc`/`weasyprint`/`wkhtmltopdf`/
`xelatex`/`libreoffice` nesta máquina; há **`google-chrome-stable`** ⇒ o caminho é
**HTML → Chrome headless → PDF**, por `bash scripts/tutorial-pdf.sh <fonte.html>`.

**How to apply:** ao assumir a linha do Motion, o ciclo aberto é o primeiro sem ✅ na §5 do doc
103 (o **ciclo 1 é «ARRANJO»**, 10 nós, tutorial *«Do primeiro objecto ao milhão»*, e é o único
que carrega o substrato do cartão). Um ciclo que pára no passo 4 está **meio feito** —
[[feedback_perfection_no_deferrals]].
