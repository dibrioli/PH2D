---
name: feedback-making-a-thing-exist-gives-answers-where-there-were-none
description: "Fazer uma coisa EXISTIR no quadro não acorda só quem a copia — acorda toda pergunta que antes devolvia «nada» sobre ela, e uma delas pode matar um gesto"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-14T00:11:43.198Z
---

Medido em 2026-09-13 (W6 do plano `docs/Skeleton/03`, report do dono: *«selecionar o osso não é mais
possível»*). A W2 fez a imagem presa ao esqueleto **emitir instância** pela primeira vez. Eu fiz o
censo de *«quem COPIA a instância?»* e *«quem lê o QUAD?»* e curei cinco consumidores — e faltou o
sexto, que não fazia nem uma coisa nem outra: a **caixa do gizmo** pedia um espelho no presente
(`query::<(&SimRef, &GlobalTransform)>`), logo antes ela devolvia `None` e agora devolve uma caixa.

Essa caixa regista `GIZMO_BBOX_INTERIOR` no `hit_index`; o `on_canvas` do despacho exige o índice
**vazio**; ⇒ com a imagem seleccionada, todo press sobre a arte deixou de chegar ao ramo da
ferramenta — e o gesto que vivia por cima dela (posar um osso) **morreu sem um erro**.

**Why:** um censo de *«quem consome X?»* enumera os leitores que já respondiam. Quando X passa a
existir, o que muda são os que respondiam **`None`** — e esses não aparecem em nenhum `grep` por X,
porque eles não o nomeiam: eles nomeiam a pergunta.

**How to apply:** ao fazer uma entidade/componente/registo passar a existir, o censo é *«que
pergunta sobre ela passa a ter resposta onde antes não tinha nenhuma?»* — e a primeira a conferir é
qualquer uma cuja resposta chegue a um **índice de acerto** ou a uma precedência de despacho, porque
aí o sintoma é um gesto que some, não um desenho errado. Ver também
[[feedback-a-copy-of-one-component-silently-drops-its-siblings]] (a espécie irmã, dos que copiam) e
[[feedback-a-wrong-id-in-a-list-is-not-a-visible-defect-until-its-reader-is-read]].
