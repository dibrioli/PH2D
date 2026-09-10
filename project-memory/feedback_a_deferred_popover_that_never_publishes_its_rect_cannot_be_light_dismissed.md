---
name: feedback-a-deferred-popover-that-never-publishes-its-rect-cannot-be-light-dismissed
description: "Popover pintado num passe DIFERIDO nunca fecha ao clique de fora se ninguém chamar set_dropdown_popover — os 4 seletores do Inspector viveram assim, e nenhum gate os via"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6a6caccb-4d8f-423e-885d-d18bb2df8b6f
  modified: 2026-09-09T21:08:57.041Z
---

Um `Dropdown` cujo popover é pintado num **passe diferido** (para ficar acima das secções
seguintes) fica **impossível de fechar ao clique de fora** se ninguém publicar o rect do painel no
store. A lei do light-dismiss vive no `dispatch::pointer_down` e lê `store.dropdown_popover()`; ela
é alimentada por `WidgetStore::set_dropdown_popover(id, rect)`, que **quem pinta** normalmente
chama — e um passe diferido costuma receber só `&HitIndex`/`&WidgetStore`, sem o `&mut`.

Medido 2026-09-09 no `ph2d-panel-inspector`: **os QUATRO seletores** dele (Sampling · Sorting Layer
· Rides Parent Anchor · o verbo novo do SignalActions) nunca publicavam. Os outros nove painéis do
app publicam no sítio onde pintam.

**Why:** nenhum gate desta casa pergunta *o que a pintura publicou no store* — o
`architecture_panel_wiring_parity` mede focalizabilidade e os `seam_*` medem se o clique chega à
ferramenta. Um seletor preso aberto passa em todos eles, e o sintoma que o dono reporta é *«a lista
fica por cima de tudo»*, que se lê como defeito de layout.

**How to apply:** ao pintar um popover fora do sítio da chamada, o passe **deposita** (`(id, rect)`
num slot do painel) e o primeiro ponto com `&mut WidgetStore` depois da pintura **publica**. E
escreva o gate na forma `store().dropdown_popover()` devolve o dono certo e cada linha da lista cai
DENTRO do painel publicado — sem a segunda metade, um clique numa opção é lido como «clicou fora».
Relacionado: [[feedback_painted_is_not_populated_paint_gate]] ·
[[reference_topic_ui_seam_discipline]] · [[feedback_alive_reachable_and_in_the_wrong_place_are_three_questions]]

## E a IRMÃ, achada no mesmo painel um report depois

O mesmo passe diferido pendurava a lista **sempre ABAIXO do chip** (`popover_rect`) enquanto o
resto do app usava o `popover_rect_clamped`. O sintoma do dono: *«o dropdown está abrindo fora da
tela para baixo, não se adapta à posição do widget»*.

⚠️ **E o clamp sozinho é MEIA-CURA:** quando a lista não cabe de nenhum lado ele **encolhe o
painel**, e sem rolagem as linhas de baixo ficam desenhadas fora dele. As três coisas — virar,
encolher, rolar — são **uma** porta, e a régua tem de medir a FIXTURA antes do produto (*esta cena
transbordaria sem o clamp?*), senão o gate passa **por caber**.

⚠️ **A região do clamp é a banda de chrome, nunca a janela:** dada a janela inteira, *«o lado com
mais espaço»* é quase sempre para cima, e a lista nasce colada à borda de topo, sobre uma faixa
onde não há painel nenhum. E a janela do gate tem de ser a do **alvo** (1280×720): a 1600×900 a
mesma cena não produz o fenómeno.
