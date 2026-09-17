---
name: feedback-a-new-gesture-inherits-the-enemies-of-the-old-one
description: Um gesto NOVO para o mesmo tipo de manipulação herda todos os inimigos do antigo — e ninguém lhe dá a lista de excepções que foi escrita para o primeiro
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-14T18:27:27.982Z
---

Medido em 2026-09-14 (W8 do plano `docs/Skeleton/03`). Report do dono: *«com a timeline aberta não é
possível transformar os ossos e criar key frames com AutoKey»*.

A timeline aplica o documento ao mundo a cada quadro e salta **o que a mão segura** — uma lista que
existia e estava certa: `hero.gizmo.drag`. Posar um osso é um gesto **próprio** (o gizmo de sprite
não serve: a caixa de um osso é `0×0`, e o código do posar já o escrevia), e ele não publicava nada.
Sonda: a mão punha `rotation = 0,77` e o apply devolvia `0,45` **no quadro seguinte**. O osso voltava
debaixo do dedo, e o AutoKey — que corre DEPOIS do apply — lia `mundo == curva` e não tinha o que
cunhar. *Duas metades do mesmo defeito, um relato só.*

⚠️ E a cura criava um defeito novo sem a **terceira** metade: o `drag_now` do AutoKey também só
conhecia o gizmo, logo cada quadro do arrasto seria uma «edição discreta» com passo de undo próprio —
quarenta passos para dobrar um braço.

**Why:** quem escreve o gesto novo pensa no motor dele (aqui: o esqueleto). As listas de excepção que
protegem uma manipulação viva — o que o apply salta, o que o undo agrupa, o que a captura repõe —
foram escritas para o gesto que existia, **nomeando-o**, e nenhuma sonda pergunta se elas ainda
descrevem a população.

**How to apply:** ao acrescentar um segundo gesto que manipula o mesmo estado, faça o censo de *quem
já nomeia o primeiro* (`grep` pelo campo dele: `gizmo.drag`, `dragging_entity`, …) e responda, em
cada sítio, se o novo pertence ali. E prefira passar o **ESTADO** em vez de um `Option` já resolvido:
quando a porta lê o campo ela própria, esquecer a segunda mão deixa de ser possível num chamador que
nenhum teste alcança. Ver [[feedback-making-a-thing-exist-gives-answers-where-there-were-none]] e
[[feedback-a-gesture-written-in-two-halves-accepts-a-new-variant-in-only-one]].
