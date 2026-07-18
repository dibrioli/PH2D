---
name: feedback_a_deferral_notes_bar_may_exceed_the_projects_policy
description: "Nota de trabalho diferido declara um critério de aceitação? Confira contra o que o repo REALMENTE exige — ela pode te prender a uma barra que nada no projeto atinge"
metadata:
  node_type: memory
  type: feedback
---

Quando um trabalho é adiado, quem adia costuma escrever **como ele deveria ser aceito**. Esse critério
é uma *opinião de quem não fez*, não a política do projeto — e pode ser estritamente mais duro que
qualquer coisa que o repo de fato exige.

Aconteceu 2026-07-18 (luz do impasto na GPU). A nota dizia: *"um `LayerOp` novo, reconciliado
**bit-a-bit** contra esta passagem CPU"*. Ambas as metades estavam erradas:

- **bit-a-bit** — o próprio compositor documenta que a saída de runtime **não** é bit-idêntica entre
  backends (um backend pode contrair `a*b+c` em FMA). A política real do projeto são **literais**
  bit-idênticos por gate CPU-only + **épsilon documentado** de runtime (Bloom ≤5, S/H ≤4) contra o
  kernel canônico. Perseguir a barra da nota seria caçar fantasma.
- **`LayerOp`** — a luz é espacialmente não-local e roda uma vez no fim, e o compositor já argumenta
  que não-local pertence ao pass-graph segmentado, não ao laço per-pixel. Como passe pós-composite ela
  não tocou `LayerOp`, `flatten`, validação, segmentação nem bind groups.

**Why:** a nota é escrita no momento de MENOR informação sobre o trabalho — antes de alguém ler o
seam. Ela é excelente como *ponteiro* ("isto falta, aqui dói") e não-confiável como *especificação*.
Aceitá-la como spec ou infla o escopo (uma variante de enum que não era necessária) ou trava o
trabalho numa barra inatingível.

**How to apply:** trate a nota como sintoma, não como projeto. Antes de orçar, verifique cada
afirmação dela contra o que o repo AFIRMA: para "bit-a-bit", leia os gates de paridade que existem e
veja qual tolerância eles aceitam; para "precisa de X", leia se X é mesmo o encaixe. E quando a nota
estiver errada, **corrija a nota** no mesmo commit — senão a próxima LLM a lê e recomeça o erro. Vide
[[feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate]] e
[[feedback_stale_comment_and_dead_code_lie]].
