---
name: feedback-a-ruler-that-stops-at-world-space-approves-a-broken-click
description: Uma régua que mede a resolução em MUNDO aprova um clique partido no ECRÃ — e o chrome da cena desenha numa BANDA, não na janela
metadata:
  type: feedback
---

Medido em 2026-09-17 (`line/components`, TOP-20 #20), report do dono: *«passo 4 não funciona»* — o
botão do HUD não somava pontos. A auto-conferência da cena dizia **SIM**: ela media *que forma está
sob um PONTO DO MUNDO, e de quem ela é*. **O clique do artista começa no ECRÃ**, e era ali que a
corrente estava partida.

Varrendo o ecrã pelo mesmo `path_at` do produto:

| | caixa de ecrã do botão |
|---|---|
| **DESENHADO** (medido na foto) | `x 879..1050 · y 402..462` |
| **ALCANÇÁVEL** pelo dedo | `x 800..1128 · y 728..848` |

`~340 px` de diferença — dentro do painel da timeline, onde `on_canvas` é `false` e o ramo do clique
nem chega a correr.

**Causa:** com um painel a partir o centro (`CenterSplit::Horizontal{t}`) a cena renderiza numa
**BANDA** (`1930×556` numa janela de `1930×1012`) e a projecção MUDA; a porta `vec_world_at` mapeava
o cursor contra a **janela**. A aritmética fecha antes do código: `10` unidades de mundo em `556 px`
são `55,6 px/unidade`, e o centro do botão (`y = −2,778`) cai em `432` — onde a foto o mostra.

**Why:** a lei **já estava escrita** (o doc do `ph2d_app_motion::field_gizmo::scene_window_wh`: *«todo
mapeamento mundo↔tela do chrome da cena TEM de usar isto»*, com um gate vizinho chamado
`the_cursor_is_mapped_through_the_scene_viewport_not_the_window`) — e a porta do vector não a usava.
O censo mede **72** chamadas de `screen_to_world` em **33** ficheiros da shell contra **19** sítios
que já passam pela banda: *a dívida é do CHROME e atinge seis gestos vectoriais*, não do HUD.

**How to apply:**
- Uma régua de costura que pára no espaço do MUNDO afirma metade. **Varra o ECRÃ** e compare a caixa
  alcançável com a caixa PINTADA (a foto) — as duas juntas são a corrente inteira.
- Antes de culpar a lei de uma feature nova, pergunte se o espaço em que o gesto entra é o mesmo em
  que ele é desenhado. Sob split, `surface.size()` **não** é a janela da cena.
- E [[feedback_a_smoke_for_the_owner_explains_what_each_thing_on_screen_is]]: a foto é o único
  oráculo do lado PINTADO, porque nenhuma sonda de dentro do app o conhece.
