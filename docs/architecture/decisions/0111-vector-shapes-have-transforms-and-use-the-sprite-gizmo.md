# ADR-0111 — Formas vetoriais têm `Transform`, e quem as manipula é o gizmo de sprite

- **Status:** aceito (Enio, 2026-07-09)
- **Fecha:** [ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md) §5 (herança de transform, deferida de propósito)
- **Escopo:** pose, parentesco real, gizmo, picking, pill. Modos de ferramenta (Select / Node) são o bloco seguinte (§6).

## Contexto

O ADR-0110 fez de cada path uma entidade. Faltava-lhe **pose**: a geometria era assada
em coordenadas de mundo, e o `Transform` da entidade não era lido por ninguém.
Parentear um path a um sprite aninhava a linha na árvore e herdava visibilidade e
trava — mas mover o sprite não movia o path. Era organização, não parentesco.

Ao mesmo tempo o módulo tinha um **gizmo próprio** (`vec_gizmo.rs`, 492 linhas):
enquadrava a seleção, e ao arrastar **mutava a geometria** — transladava âncoras,
escalava handles, girava pontos. Um segundo motor de transformação, com o seu próprio
undo, o seu próprio pivô, o seu próprio bbox orientado. E ele não sabia compor com um
sprite: não havia como transformar junta uma multi-seleção mista.

## Decisão

**A geometria do path é LOCAL. A pose vive no `Transform` da entidade.**

O afim local→mundo é `parent_world_transform ∘ Transform` — a mesma cadeia de um
sprite, pelo mesmo helper, com o mesmo `libm::sincosf` (HR-5). A shell o publica uma
vez por frame (`vec_transform::build`); o documento continua puro e não conhece ECS.

Identidade ⇒ local **é** mundo. Todo path recém-desenhado nasce assim, e todos os
testes puros seguem valendo sem uma linha alterada. O mapa nem guarda a identidade.

**O gizmo vetorial foi DELETADO.** Quem move, gira e escala uma forma é o **gizmo de
sprite**, escrevendo `Transform`. Isto não é adaptação, é remoção: a tradução é exata.
O gizmo enquadra um sprite como

```text
centro = translation + R·(anchor ⊙ scale)     meia-extensão = half_intrínseco ⊙ scale
```

e uma forma vetorial tem a **mesma** forma se lermos `anchor` como o *centro da bbox
local da curva* e `half_intrínseco` como a *meia-extensão dessa bbox*. Por isso
`opposite_anchor_translation` (que fixa o canto oposto ao escalar) vale sem uma linha
nova — ela depende só desses dois números. `vec_gizmo_view::anchor_half` é a tradução
inteira.

Do que cai fora de graça:

- **Parentesco real.** Mover/girar/escalar um sprite move a forma filha. A cadeia é a
  mesma, computada pelo mesmo `parent_world_transform`.
- **Gizmo global misto.** Uma seleção de sprites e paths transforma junto: o gizmo
  global já itera a seleção escrevendo `Transform` em cada entidade, e agora o path é
  uma dessas entidades.
- **Picking de canvas.** Clicar numa forma a seleciona como um sprite (com
  clique-cíclico entre sobreposições) e o marquee a pega pela bbox de mundo. Formas
  vetoriais desenham por cima dos sprites, então entram na frente da lista.
- **Undo.** Um move de forma é um undo de `Transform`, o mesmo dos sprites — não mais
  um snapshot do documento inteiro.

**O pill do Vector alterna.** Clicar na ferramenta já ativa volta para a default
(move). Vale para os clusters direct-activate (`vector_tools`, `motion_tools`); os
`image_tools` seguem mandados pelo toggle IMG.

## A regra que mantém o pen honesto

> **O que o usuário vê, aponta e encaixa é MUNDO. O que o documento guarda é LOCAL.**

A conversão mora só na fronteira de `PenTool` (`to_world` / `to_local` /
`delta_to_local`), e nenhum cálculo de geometria mudou. Consequências que os testes
travam:

- o raio de captura é world, então uma forma escalada 10× não ganha um alvo 10× maior;
- arrastar uma âncora a leva **sob o cursor**, não ao dobro da distância;
- as setas do teclado andam o mesmo tanto na tela, esteja a forma escalada ou não;
- os alvos de snap são publicados em mundo — o canto encaixa onde ele aparece.

## Operações de geometria assam o frame

Booleana, merge e offset recebem operandos de entidades com poses **diferentes**, e um
resultado só pode viver num frame. Cada operando é assado no mundo (`bake_xform`)
antes da operação; o path resultante nasce world-space e a entidade nova dele nasce na
identidade — a forma aparece exatamente onde as originais estavam.

## Consequências

**Boas.** Um motor de transformação, não dois. 492 linhas a menos. Parentesco que move
de verdade. Gizmo global misto. O botão "Set Center" do painel Vector saiu: o centro de
rotação/escala é o pivô do gizmo, como em qualquer objeto.

**Custos.** O stroke escala com a forma (o contorno de um sprite escalado também
escala). Se um dia se quiser "não escalar traços" à la Illustrator, é uma flag no
render, não uma mudança de modelo.

**Gap conhecido.** `vec_save` / `vec_load` (Ctrl+S/Ctrl+O) serializam só o `VecScene`.
Desde o ADR-0110 nome/visibilidade/parentesco já não iam junto, e agora a **pose**
também não. O caminho certo é o save de cena do ECS, não um segundo formato — ainda
não foi feito, e não regride nada que já funcionasse.

## §6 — O que fica de fora (deliberadamente)

**Modos da ferramenta: Select (seta preta) e Node (seta branca).** Hoje a ferramenta
Vector é a caneta, e sair dela (o pill alterna) dá o comportamento de seleção de
objeto — que é exatamente o que uma seta preta faz. Falta expô-los *dentro* da
ferramenta, como o Illustrator: `DrawMode::Select` (o canvas não roteia para o pen; o
gizmo e o picking mandam) e `DrawMode::Node` (o pen edita âncoras, mas um clique no
vazio não começa um path novo — desseleciona ou box-seleciona).

É UI sobre o que este ADR já entregou, e merece o seu próprio smoke.
