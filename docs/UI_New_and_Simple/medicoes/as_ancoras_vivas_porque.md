# As âncoras vivas — o porquê (cabeçalho de `shells/desktop/src/layout_live_anchors.rs`, movido verbatim em 27/10)

> Movido da shell pela catraca `the_shell_only_shrinks` (W5.5 `line/BlenderConstraints`, o arranjo do HUD).

**AS ÂNCORAS, vivas** — a segunda metade do passe de layout (plano UI/UX W3).

Módulo FILHO do [`super`], e a razão não é o teto de LOC: é a **lei da porta única**. O plano
diz que um filho de moldura *"ou está num fluxo ou está ancorado, nunca os dois"*, e um segundo
passe teria uma segunda opinião sobre a pose derivada. Aqui o dono da tabela de poses continua
a ser UM ([`super::LayoutLive`]), e este ficheiro é só o outro braço dele.

# O que uma âncora responde, e o fluxo não

O fluxo empilha: ele decide a posição de TODOS os filhos a partir de uma regra do pai. A âncora
é o oposto — o filho está onde o artista o pôs, e a regra diz apenas **o que acontece quando a
moldura muda de tamanho**. É o HUD: a vida no canto de cima, a pontuação colada no canto
oposto, a barra que estica no meio.

# A régua é LOCAL, e o resto é uma razão

[`ph2d_ecs::VecAnchors::delta_local`] devolve o deslocamento em unidades **locais** da moldura.
Para o levar ao mundo este passe usa a **razão entre a caixa de mundo e a caixa local da própria
moldura** — e essa razão traz consigo, de graça, tudo o que estiver entre as duas: a pose do
ancestral, a escala do `Transform`, e até a colocação que o fluxo acabou de dar à moldura, se
ela própria for um item de fluxo.

⚠️ **Ela é exacta enquanto o mapa local→mundo for alinhado aos eixos, e aproximada sob
ROTAÇÃO** — a caixa de mundo de um retângulo rodado é maior que ele. É a mesma aproximação
declarada que a caixa do gizmo carrega ([`ph2d_app_vec::vec_gizmo_view::fold_layout_pose`]), e pela
mesma razão: o que se compara são caixas alinhadas aos eixos, e um retângulo rodado não é uma.

# O neutro é EXACTO, não *"quase zero"*

Moldura do tamanho da régua ⇒ `delta_local` dá `0.0` nos dois eixos por subtracção de iguais ⇒
a caixa-alvo é a caixa de agora ⇒ o afim é a identidade ⇒ o passe **nem paga a cópia da
geometria**. Um documento sem `VecAnchors` é byte-idêntico ao de antes desta wave; um COM
âncoras e sem redimensionamento também.

## A ordem no `recook` (comentário de `layout_live.rs`, movido verbatim em 27/10)

⚠️ **As âncoras DEPOIS do fluxo, e a ordem é load-bearing pela MEDIÇÃO** (plano UI/UX
W3). A âncora lê a caixa da moldura; o fluxo DECIDE essa caixa. Ao contrário, o fluxo
mediria a moldura pela sub-árvore que a âncora acabou de colocar — e a âncora colocou-a
lendo a caixa que o fluxo ainda ia decidir: um laço.

⚠️ E a razão NÃO é a que parece: para uma colocação que é `translate ∘ scale`, as duas
ordens dão o MESMO ponto (`S·(x+d) + t` contra `S·x + t + S·d` — a mesma expressão), e
a primeira versão do gate ficou verde sobre a ordem trocada por causa disso. O que
discrimina é só o tamanho MEDIDO do nó, e é exactamente isso que o gate
`the_anchor_does_not_feed_back_into_the_flows_measurement` afirma.

⭐⭐⭐ **E os canvas de HUD ancoram pela MESMA lei** — ver `anchor_canvases`. A ordem
entre os dois não importa: as populações são disjuntas por construção (um filho tem UM
pai, e ele ou tem `VecFrame` ou tem `UiCanvas`).

## O canvas do HUD ancora como uma moldura (doc de `anchor_canvases`, movido verbatim em 27/10)

⭐⭐⭐ **O CANVAS DO HUD ancora como uma moldura** (TOP-20 #20) — e a lei do filho é a **mesma**:
[`LayoutLive::anchor_kid`], sem uma linha nova.

⚠️ **A SELECÇÃO mora na crate da família** ([`ph2d_app_components::hud_anchors::ancorados`]):
*«quem ancora quem, e contra que rectângulo»* responde-se com o mundo e o mapa, sem cena e sem
device, logo com gates baratos. O que fica aqui é publicar a pose — a maquinaria que o passe das
molduras já tem.

⭐ **O neutro é EXACTO**: no aspecto da própria caixa não há banda, a efectiva É a de referência,
e o `anchor_kid` sai no `is_identity` **sem pagar a cópia da geometria**.
