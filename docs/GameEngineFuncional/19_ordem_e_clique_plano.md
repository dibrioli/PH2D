# Plano GEF 19 — a Hierarchy manda no desenho e no clique; ícones especiais; Z-index em todo objecto

Pedido do dono (01/11, depois do smoke aprovado da 18c). Decisões (não reabrir) em
[memória](../../project-memory/project_hierarchy_order_rules_drawing_and_click_2026_11_01.md):
desenho **e** clique pela Hierarchy (mais acima = desenha por baixo) em TODAS as famílias · gizmos especiais
(câmera, lâmpada, macro) ganham o clique · ciclo de sobrepostos = **um clique por degrau**, arrastar move o
selecionado · ícone próprio da Macro · Z-index no Inspector de todo tipo · fundos (parallax) conferidos.

## O que a leitura estabeleceu (4 exploradores, 01/11)

- **Desenho:** uma ordem total só para sprites + vetor — `compute_sort_ranks_into` (`ph2d-ecs/src/sort_key.rs:174`),
  formas inscritas em `sim_extract.rs` (`vector_participant` :196, `build_frame_order` :209), faixas em
  `ph2d-render/src/draw_bands.rs` (`Family {Sprite, Vector}`, `MAX_BANDS = 16`), presente em `present_bands.rs`.
  **Fora da ordem:** Flip (um passe, `present_flip_fx` `present.rs:378`, no `game_rt` antes das faixas, ordem do
  `FlipDoc`) · Motion (placa por FONTE colada por cima de tudo, `present_chrome.rs:54-80`; palcos idem) · Pixel
  Sprites (vetor na cena do CHROME, `fase_selection_highlight.rs:106`, ordem por `StableId`, `mode.rs:30`).
- **Clique:** `pick_objects_at` (`hover_highlight.rs:83`) devolve BLOCOS por família: lâmpada/câmera saem cedo
  (o ciclo nunca chega à arte por baixo do ícone) › vetor (z certo) › Flip (por id) › sprites (ordem de archetype,
  ignora o rank) › pixel › anel do vazio. Sprite com rank acima de uma forma perde o 1.º clique.
- **Ciclo** (`despacho_clique_pick.rs:448`): contagem par mantém, ímpar avança ⇒ dois cliques por degrau.
- **Trava:** só o Flip a respeita no pick (`is_locked_for_edit`); sprites, pixel, luz, câmera não.
- **Macro:** entidade `Name + Transform + Macro`, sem desenho nem pick no canvas. Câmera: glifo `IconId::Camera`
  (`moldura_tinta.rs`), «Icon Size» do projecto (`camera_icon_px`, 44 px).
- **Z-index:** a §7 Ordering pinta-se para toda entidade com `Transform` (gate
  `the_ordering_section_belongs_to_every_object`); o código não a filtra por tipo ⇒ o sumiço que o dono vê
  mede-se no app (foto por tipo), não se adivinha.
- **Parallax:** (b) a repetição de uma Background Layer ESCALADA desalinha — `ScrollRepeat.tile` em metros sem a
  escala (`object_add.rs:105`, `parallax_bridge.rs:188`); (a) o que o editor mostra fora do Play: o código conduz
  a paralaxe pela câmera do jogo em todo quadro (`fase_paralaxe.rs:41`) — confirmar por foto; (c) limites do nível
  1 coincidem com as paredes (medido no código, sem gate).

## Desenho (decisão técnica)

**Uma lei, dois leitores:** a `FrameOrder` passa a guardar a ENTIDADE de cada rank e a família de cada uma
(Sprite · Vector · Flip · Motion; Pixel = participante Vector). O presente desenha faixa a faixa; o pick ordena
os acertos pela MESMA posição de empilhamento (`FrameOrder::stack_of`), inclusive no modo degradado. Ícones
especiais à frente da lista (sem retorno cedo — o ciclo chega à arte por baixo).

## Etapas (cada uma com gate por nome; uma foto antes/depois por etapa de desenho)

- **E1 — o clique pela ordem do desenho** + ícones à frente sem cortar a lista + trava em todas as famílias +
  um clique por degrau (Down sobre o selecionado fica pendente: arrastar move-o, soltar sem mexer desce).
- **E2 — Pixel Sprites na ordem** (participante Vector, desenhados na cena da faixa, não no chrome).
- **E3 — Motion na ordem**: a entidade `MotionGenerator` ganha rank; a placa parte-se por rank pelo `fonte` das
  cópias e cola-se entre faixas; palcos e miniatura (`VistaDoMotion`/`VistaDosPalcos`) pelo mesmo caminho.
- **E4 — Flip na ordem**: `pass::render` por subconjunto de objectos, faixa `Family::Flip` no padrão dos sprites
  (HDR → tonemap → acumulador); fantasmas/peek só em autoria.
- **E5 — Z-index em todo tipo**: foto do Inspector por tipo do Add; curar onde faltar; gate por tipo.
- **E6 — ícone da Macro** (glifo próprio, mesmo «Icon Size», mesmo pick à frente) + ícone por TIPO na Hierarchy
  se `type_icon` (`hier_rows.rs:121`) não estiver ligado no caminho real.
- **E7 — fundos**: (b) a repetição segue a escala; (a) foto e decisão; (c) gate dos limites do nível 1.

Byte-idêntico sem intercalação continua lei (as cenas cuja ordem já coincide com a de hoje não mudam um byte).
`MAX_BANDS` só se move com medição (§0).

## Estado (01/11)

| etapa | commit | estado |
|---|---|---|
| E1 clique pela pilha do desenho, ícones à frente, trava, 1 clique/degrau | `7d703c2d79` | feito |
| E2 Pixel Sprites na ordem (`Family::Pixel`) | `c2d460075d` | feito (sem foto) |
| E3 Motion na ordem — degrau 1: rank + `Family::Motion`, ainda desenhado por cima (`motion_por_cima`) | `06494dff4e` | degrau 2 (placa por faixa) em curso |
| E4 Flip na ordem | — | por fazer, depois da E3 |
| E5 Z-index em todo tipo (gate por tipo do Add) | — | em curso |
| E6 ícone da Macro + smoke `PH2D_OBJECT_ADD_PICK` | `4bf8ac8efe` | feito (deps novas em ph2d-app-macros ⇒ crate inteira no fecho) |
| E7b repetição de camada escalada | `26c6e5c2b3` | feito |
| E7a o que o editor mostra da paralaxe · E7c gate dos limites do nível 1 | — | por fazer |
