---
name: feedback-works-then-silently-forgets-recook-wipes-authored-state
description: "Estado autorado guardado dentro de geometria DERIVADA some no próximo recook do dono dela — funciona, o usuário confia, e um slider desfaz em silêncio. E um gate sobre o MODO não é um gate sobre a POLÍTICA."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: beb8fcaa-5792-4a3f-ba88-6a15a729732f
---

**"Funciona e depois esquece sozinho" é pior que "não funciona".** O usuário confia, investe
trabalho, e perde tudo sem uma mensagem de erro.

O padrão: você guarda um dado **autorado** (um raio, um peso, uma flag) dentro de uma
estrutura cuja geometria é **derivada** e re-cozida por outro sistema. Funciona — até o dono
daquela estrutura re-cozinhar. Aí o campo some, e **nada fica vermelho**.

**Como aconteceu (Live Corners, 2026-07-13):** `VecVertex.corner_radius` era autorado pela
alça. Mas uma **Live Shape** tem `vec_shape_live::recook_into` reescrevendo `path.verts`
INTEIRO a cada mudança de parâmetro. Desenhe um retângulo com a Shape tool, arredonde a
quina, encoste num slider do painel → o raio evapora.

**Não dá pra consertar preservando o campo no recook:** a *contagem* de vértices é função dos
parâmetros (o slider de lados de um polígono muda quantas quinas existem). Não há para onde
levar o raio da quina que deixou de existir. A cura é **recusar a feature naquele objeto** e
dizer por quê.

**Why:** no PH2D vários sistemas re-cozinham geometria por-frame (Live Shape, conector vivo,
rótulo vivo, motion nodes). Todo dado autorado que more na saída deles é uma bomba-relógio.

**How to apply:**
1. Antes de guardar um campo autorado numa struct de geometria, pergunte: **quem mais
   ESCREVE esta struct inteira?** (`grep` por `\.verts = `, `*x = `, atribuição do agregado).
   Se alguém a substitui em bloco, o seu campo morre lá.
2. **Um gate sobre o MODO não é um gate sobre a POLÍTICA.** Eu gateei "a alça é do modo Node"
   e achei que estava coberto — mas uma *forma viva selecionada dentro do modo Node* é outra
   coisa, e o gate de modo nunca a alcança. Se a regra é "isto não vale para objetos do tipo
   X", **teste objetos do tipo X**, não os modos em que eles costumam aparecer.
3. Quando recusar a feature num objeto, **recuse no hit-test também**, não só no render —
   senão a alça fica invisível e ainda agarrável: um alvo fantasma, o pior tipo.
4. Achei isto **simulando o caminho do smoke no papel**, não por teste. Vale sempre: percorra
   o roteiro que o Enio vai percorrer, passo a passo, antes de declarar a linha fechada.

Relacionadas: [[feedback_stale_comment_and_dead_code_lie]] ·
[[feedback_tool_unit_green_integration_dead]] · [[feedback_ergonomics_verdict_is_a_design_bug]]
