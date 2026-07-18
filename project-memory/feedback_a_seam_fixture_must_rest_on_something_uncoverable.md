---
name: feedback-a-seam-fixture-must-rest-on-something-uncoverable
description: Fixture que precisa de um caso NEGATIVO (uma costura, um fallback) tem de repousar em algo impossível por ESTRUTURA, não em algo meramente ainda-não-feito
metadata:
  type: feedback
---

Um gate que precisa de um caso **negativo** — uma costura CPU↔GPU, um fallback,
um caminho recusado — costuma ser construído sobre "o nó que ainda não tem
kernel". Isso apodrece do jeito mais irônico possível: **o próprio trabalho de
cobertura apaga o fenômeno que o gate existe para medir**, e a pressão do momento
é enfraquecer o gate para ele voltar a passar.

**Caso real (GPU/M5, 2026-07-18).** Os fixtures de duas costuras usavam
`value.instance_field` porque ele não tinha kernel. Horas depois, na MESMA sessão,
a medição de perf apontou justamente esse nó como o maior imposto de prefixo — dei
kernel a ele, e os dois gates ficaram vermelhos porque o plano passou a ter **zero
fronteiras**. Vermelho pelo melhor motivo possível, e ainda assim vermelho.

**Why:** "sem kernel" é estado de BACKLOG, não propriedade. Um fixture ancorado
nele mede *"o que ainda não fizemos"*, e essa quantidade é projetada para ir a
zero. Já `value.attribute` nomeia sua coluna por **text param** enquanto
`ColumnBinding.column` é `&'static str` — é incobrível por ESTRUTURA. Um fixture
sobre ele mede a costura enquanto a costura existir, e se um dia deixar de ser
verdade **é certo que ele quebre alto**.

**How to apply:**
- Ao escrever um gate que precisa de um caso negativo, pergunte: *"o que torna
  isto impossível — uma decisão de arquitetura, ou uma tarefa não feita?"*. Se for
  tarefa, escolha outra âncora.
- Escreva no fixture **por que aquela âncora é a âncora** (o mecanismo, não o
  nome), senão o próximo agente troca por conveniência.
- Gate vermelho porque a capacidade avançou é **notícia boa**: rebase o fixture,
  nunca afrouxe a asserção. Relacionado: [[feedback_convention_vs_inertia]] e
  [[feedback_documented_decision_chesterton_fence]].
