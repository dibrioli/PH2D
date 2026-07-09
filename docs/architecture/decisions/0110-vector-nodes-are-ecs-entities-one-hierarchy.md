# ADR-0110 — Nós vetoriais são entidades ECS: uma Hierarquia só

- **Status:** aceito (Enio, 2026-07-09)
- **Supersede parcialmente:** [ADR-0108](0108-vector-reposition-rive-referenced-native-editor-first.md)
  (o trecho que *retirou* a integração ECS de vector-scene)
- **Escopo:** identidade e árvore. Herança de **transform** é bloco seguinte (ver §5).

## Contexto

O módulo Vector (ADR-0108) nasceu como documento puro: `AppGfx.vec_scene`, fora do
ECS. Ao ganhar estrutura (nome / visibilidade / trava / grupos), ele ganhou junto uma
**segunda árvore de objetos** — uma seção "Objects" no painel `ph2d-panel-vector`,
com seus próprios grupos (`VecGroup`), seu próprio parentesco e sua própria lista.

O editor já tem UMA árvore: o painel **Hierarchy**, que é uma projeção do
`bevy_ecs`. Uma linha *é* uma `Entity`. Parentesco é `ChildOf`, ordem de irmão é a
ordem de inserção em `Children`, ordem de raiz é `RootOrder`. Reparent por arrasto,
olho, cadeado, group-lock, rename inline, duplicar, deletar, seleção compartilhada
com o gizmo — tudo já existe, tudo chaveado em `Entity`.

Duas árvores é uma árvore a mais. E o custo real não é a duplicação de código: é que
**um path vetorial não podia ser filho de um sprite**. Sprites são entidades; nós
vetoriais não eram. Sem espaço de identidade comum, não existe parentesco cruzado —
e nenhum painel irmão conserta isso.

## Decisão

**Todo nó vetorial é uma entidade ECS.**

- Cada `VecPath` tem uma entidade que o referencia (`ph2d_ecs::VecPathRef(VecPathId)`).
  O `VecScene` continua dono da **geometria e do estilo**; a entidade é dona da
  **identidade e do lugar na árvore**.
- **`VecGroup` deixa de existir.** Um grupo vetorial vira uma entidade comum com
  filhos — a mesma coisa que um grupo de sprites. Ctrl+G passa a agrupar *qualquer*
  mistura de tipos, que é o ponto.
- `VecPath` perde `name` / `visible` / `locked` / `parent`. Passam a ser `Name` /
  `Visibility` / `Locked` / `ChildOf`, os componentes que o resto do editor já usa.
- A **ordem de z** dos paths passa a ser derivada da árvore: a ordem DFS da
  Hierarquia, primeira linha à frente (convenção Illustrator/Figma). `VecScene.paths`
  segue sendo a pilha de z que o render e a booleana leem — só que agora é uma
  **projeção** da árvore, re-sincronizada a cada frame.
- A seção "Objects" do painel Vector **sai**. A árvore é a Hierarchy.

Tudo o mais — arrastar para reordenar, reparentear, olho, cadeado, rename, duplicar,
deletar, seleção casada com o canvas — vem de graça, porque já opera em `Entity`.

## Por que não um painel irmão

Foi o que existia (e o que o painel de layers do Painter faz). Reusa o mesmo motor
de arrasto e não toca o ECS. Mas não resolve o pedido: sem espaço de identidade
comum, `ChildOf` não cruza a fronteira sprite ↔ vetor. Um path nunca seria filho de
um sprite. O painel irmão é barato exatamente porque não integra.

## Por que isto não contradiz o ADR-0108

O 0108 retirou a integração ECS do vector-scene **antigo** (`ph2d-vector-doc`, o
modelo de contrato congelado, com `VectorOp`, animação e nós de grafo). O que volta
aqui é outra coisa e muito menor: **uma referência de identidade**
(`VecPathRef(u64)`) para que a árvore do editor seja uma só. O documento vetorial
novo (`ph2d-vec-scene`) permanece puro, sem dep de `bevy_ecs`; quem faz a ponte é a
shell. O que o 0108 rejeitou — o modelo de documento vetorial dentro do ECS —
continua rejeitado.

## Consequências

**Boas.** Uma árvore. Grupos heterogêneos. Arrastar-para-reordenar de graça. Olho,
cadeado, rename e seleção com o comportamento que o usuário já conhece do resto do
editor. `ph2d-vec-scene` fica menor (o módulo `structure` encolhe para o que é
geometria: a pilha de z e o recorte de copy/paste).

**Custos.** A shell ganha um ciclo de vida a manter: path criado ⇒ entidade
spawnada; path removido ⇒ entidade despawnada; árvore reordenada ⇒ `paths`
re-projetada. É um único módulo (`shells/desktop/src/vec_entities.rs`) e um
invariante testável — nenhum path sem entidade, nenhuma `VecPathRef` órfã.

**Quebra de save.** `VEC_SCENE_SCHEMA_VERSION` 7 → 8: `VecPath` perde quatro campos
e `VecScene` perde `groups`. Postcard é posicional.

## §4 — A seleção é COMPARTILHADA (duas armadilhas, as duas já pisadas)

`hero.gizmo` não é a seleção do sprite: é a seleção do **editor**. Depois deste ADR
ela carrega sprites, grupos e paths vetoriais no mesmo conjunto de `Entity`. Duas
coisas no shell presumiam o contrário, e as duas quebraram no primeiro smoke
(Enio, 2026-07-09: *"ao selecionar sprites elas são desselecionadas
automaticamente… os handles não aparecem mais"*).

**1. A poda da seleção testava a ausência de `GizmoView`, não a morte.** Uma
`GizmoView` é o bbox do `Sprite`; uma entidade vetorial não tem sprite, logo nunca
tem view. O atalho "sem view = a entidade morreu" expulsava toda entidade vetorial
da seleção **no mesmo frame em que ela entrava**. Hoje a poda pergunta ao mundo se
a entidade existe (`render_loop::gizmo_prune`), e *sem view* significa apenas *não
pinta gizmo de sprite*. Corolário: o gizmo global passou a contar **views**, não
bits — 1 sprite + 1 path não é uma multi-seleção de sprites.

**2. A sincronia de seleção mandava nos bits alheios.** Um `published` só, comparado
contra o conjunto inteiro, lia "um sprite foi selecionado" como "a árvore mexeu no
vetor" — adotava (esvaziando o pen) e no frame seguinte publicava de volta um
conjunto vazio, apagando o sprite. A regra agora é por **prioridade e por
propriedade** (`vec_selection::sync_selection`): a linha do vetor só enxerga o
subconjunto vetorial do gizmo, só o reescreve quando o **pen** mudou com a
ferramenta ativa (clique de canvas = substituir), e ao adotar da árvore **não toca
no gizmo**. Nada mais some sozinho.

Lição durável: um conjunto compartilhado precisa de um dono por elemento. Um flag
"o que eu publiquei" não basta quando um terceiro escreve no mesmo lugar.

## §5 — O que fica de fora (deliberadamente)

**Herança de transform.** Hoje a geometria vetorial é assada em coordenadas de
**mundo**. Parentear um path a um sprite é, nesta rodada, **organizacional**: herda
visibilidade e trava, aparece aninhado, arrasta junto na árvore — mas mover o sprite
pai *não move o path*.

O caminho para fechar isso já está claro e é o bloco seguinte: cada entidade
vetorial ganha `Transform`, a geometria passa a ser assada no espaço **local** dela
(identidade ⇒ exatamente o comportamento de hoje, então nada regride), o render
aplica o `GlobalTransform`, e pen / gizmo / snap convertem o cursor pelo inverso.
Está separado porque toca render, hit-test, gizmo, snap, booleana entre pais
diferentes e geometria de gradiente — cada um com seu smoke.

Até lá, `drain_reparent` já grava um `Transform` local compensatório na entidade
vetorial (ele faz isso para qualquer entidade). É inofensivo: o render vetorial
ignora `Transform` nesta rodada. Quando o bloco seguinte o ligar, o valor já estará
correto.
