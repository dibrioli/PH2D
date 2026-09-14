---
name: two-identical-reports-can-have-causes-in-different-layers
description: «não tem undo/redo» duas vezes, dois pincéis, duas camadas — no tecido a shell prendia o foco, na pose a janela do undo saía vazia da crate
metadata:
  type: feedback
---

Medido em 2026-09-14 (`line/sculpt3d`). O dono reportou **a mesma frase** sobre
dois pincéis, com nove dias de diferença:

| report | pincel | causa | camada |
|---|---|---|---|
| 2026-09-05 | tecido | tocar num chip numérico do painel prendia o foco e matava todo atalho | **shell** |
| 2026-09-14 | pose | a janela `touched()` do undo saía **vazia**, e o `close_stroke` devolve cedo sobre ela | **crate** |

No primeiro os dados estavam **certos** na crate (medido no dia: `191` vértices
capturados, repor devolvia a malha ao bit) e o defeito estava a jusante. No
segundo o gesto movia `171` vértices e **não deixava rasto nenhum**, porque o
`capture` — a porta que enche a janela — vive no laço por-vértice por onde
aquele verbo, que resolve a própria região, **nunca passa**.

**Why:** um report do dono descreve o que ele **vê**, e a mesma coisa vista
(«o Ctrl+Z não faz nada») é produzida por causas em camadas diferentes. *Tratar
o segundo pela memória do primeiro manda procurar no sítio errado* — e a memória
é convincente precisamente porque a frase é idêntica.

**How to apply:** ao receber um report que **repete** um anterior, meça a camada
**antes** de repetir a cura: um gate de unidade sobre a lei responde «os dados
estão certos?» em segundos, e a resposta parte o espaço de busca em dois. ⚠️ E
quando um verbo **desvia** do caminho comum (aqui, resolver a própria região em
vez de passar pelo laço por-vértice), ele perde **tudo** o que aquele caminho
fazia de lado — a janela do undo, os censos, os planos por-vértice. *Um desvio é
uma lista de coisas que se deixam de ganhar de graça, e ela não está escrita em
lado nenhum: descobre-se uma de cada vez, por report.*
