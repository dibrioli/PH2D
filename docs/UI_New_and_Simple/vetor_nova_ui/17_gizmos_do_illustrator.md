# 17 — Os gizmos das formas como o Illustrator, a caixa com o traço, e afastar a ④ (F11, 11/10)

> O dono (11/10, depois do smoke da estrela da F10): *«Ainda não gostei dos nossos gizmos. Se for
> necessário simplifique. Veja como faz o Illustrator.»* · *«O gizmo de transformação (comum a todas
> as formas) não engloba o stroke. Faça englobar.»* · *«O modal das formas e a barra superior abrem
> muito próximos das formas. Afaste um pouco.»* Prompt: [`16_prompt_F11_…`](16_prompt_F11_gizmos_do_illustrator.md).
> A lei das alças: [`13`](13_alcas_das_formas.md) §1; os parâmetros e o cartão: [`15`](15_mais_parametros_das_formas.md).

## §1 — O oráculo: o Illustrator PELAS FOTOS

Sem build para Linux. Fontes: os GIFs animados do helpx (*Live Shapes*, Wayback 2024-02-07,
`Live-shapes_*.gif`), o Adobe Learn *Live Shapes* (`live-shapes-step{1..10}`), o Tuts+ *What is a Live
Shape* (`cms-108724`, GIFs da estrela viva — Illustrator 2024+), o helpx de out/2025 (texto). Imagens e
quadros extraídos **fora do repo**: `/var/tmp/ph2d_F11_illustrator/` (35 ficheiros; as folhas de quadros
`*-widget_*.png` e `star.png` foram lidas a olho). Tamanhos em múltiplos da alça da caixa (1× = 12 px).

| forma | widget | onde | cara | o que faz | durante o arrasto |
|---|---|---|---|---|---|
| Rectângulo (arredondado) | *corner widget* ×4 | DENTRO do canto, ~10 px em cada eixo; arredondado: no CENTRO do arco | ⊙ aro claro + ponto, ~1× | para o centro arredonda os 4 | ficha «R: 24.56 px»; contorno-prévia; no máximo os arcos ficam VERMELHOS; a caixa **fica** |
| Polígono | *side widget* | borda DIREITA da caixa, acima do meio | ◇ losango oco, ~1× | ↕ conta 3–11 (cursor «+/−») | «Sides: 10»; as alças da caixa SOMEM, fica o contorno; prévia do polígono novo |
| | *corner widget* | UM só, ~20 px para dentro do vértice de cima-direita | ⊙ | arredonda todos | — |
| Elipse | *pie widget* ×2 | FORA do aro, numa haste de ~10 px (às 3h nasce junto da alça do meio-direito) | ⊙ na haste | gira à volta do centro | ficha de 3 linhas «Start/End/Pie Angle»; alças somem, contorno fica |
| Estrela (viva desde 2024) | *points* | borda direita da caixa, ~15 % abaixo do topo | aro com ⌃⌄, ~2,5× | ↕ conta 3–20 | «Points: 7»; alças somem; prévia |
| | R1 / R2 | SOBRE uma ponta / um vale | ○ aro claro, ~1,5× | radial | **círculo TRACEJADO** do raio + «R2: 162.54 px»; a caixa INTEIRA some |
| | ⊙ ×2 | ~20 px para dentro de uma ponta e de um vale | ⊙ | arredonda | não visto |
| Linha | pontas | nas duas pontas | ■ cheio | comprimento + ângulo | «D: 68.78 px / −39.25°» |

Comum: o ponto azul do centro move; não há alça de rotação (perto do canto, fora); duplo-clique num
⊙ abre o painel. **O Affinity** (o produto do dono) NÃO engloba o traço na caixa (fórum Serif 2023,
tópico 195460: só a geometria; sem opção). **O Illustrator** engloba com *Use Preview Bounds*
(Preferências ▸ Geral; também no menu do Align) — W/H e o snap passam a contar o traço. O dono pediu
englobar: segue-se o pedido (§2).

## §2 — A caixa engloba o traço

- ⭐ **A lei** (`ph2d_vec_render::visual_bounds`): a caixa do que a forma PINTA — a geometria ∪ cada
  marca da pilha (`VecPath::paint_stack`): o traço EXPANDIDO pela kurbo (junta, ponta, bico do miter
  tal como saem) sob a caneta redonda do renderer (`w·√|det|`, `stroke_uniform`); o Outer = a metade
  de fora da faixa `2w` (a lei do `aligned_stroke`), o Inner fica dentro; `offset`/`dilate` das
  camadas; as pontas de seta pelo `stroke_plan`. O tracejado não encolhe a caixa.
- A irmã `path_bounds_under` fica CONSERVADORA (o pior miter, 4×½w — recortes de FX): para DESENHAR
  uma caixa ela lê-se como «não abraça» (medido: um rectângulo miter de 10 inflava 20 em vez de 5).
- **Os leitores** (uma pergunta, uma porta): a caixa (`vec_gizmo_view::anchor_half`, `container_view`
  — e o arrasto de escala da shell, que lê a MESMA porta), a ④ e o cartão
  (`bars_publish::selection_on_screen`), o enquadrar (`frame_boxes::path_box`). A caixa intrínseca
  mede o traço JÁ sob a escala do mundo e divide por ela: sob `(3, 1)` a borda fica `0,1·√3` fora nos
  dois eixos (gate `the_gizmo_box_wraps_the_stroke_and_the_action_bar_reads_the_same_box`).
- **Ficam na geometria, de propósito:** o snap (`ph2d_vec_edit::snap` — vértices e caixa da curva, o
  que o artista encaixa; o Illustrator só muda com *Use Preview Bounds*), o pick/marquee, o
  `align`/`distribute`, o readout W/H do Inspector (`path_bbox`, as âncoras).
- `VecPath::all_contours_closed` passou a porta única de «há interior?» (o align da booleana e a caixa).
- ⚠️ **ABERTO — o desvio do arrasto de escala** (medido pela conta, não corrigido): o rácio da escala
  é `cursor/início` medido ao pivô (o canto oposto da caixa VISUAL); o traço não cresce como a
  geometria, então a alça arrastada desvia do cursor `m·(r − √(rₓ·r_y))` (escala livre) ou `m·(r−1)`
  (*Resize Box*, o traço fixo — e aí o canto oposto também anda `m·(r−1)`), `m` = a meia largura
  visual. Zero no arrasto UNIFORME (Shift). 3 px de traço a ×2: 1,5 px. A cura exacta é um rácio que
  desconta a margem (`(agora − m)/(início − m)`) com o pivô na GEOMETRIA — mexe no
  `GizmoDragState`/`transform.rs` partilhado com o sprite e o Flip (≈15 literais de teste).

## §3 — A ④ e o cartão mais longe

Antes: a ④ a `Spacing::Lg` (12 px) da seleção, o cartão a `Sm + alça` (18 px) — na foto da F10 a
④ encostava às alças da caixa (6 px de folga visível). Agora os dois a
`tool_bars_card::selection_gap()` = **alça + `Spacing::Lg` = 24 px** (a ④ por
`float_bar::strip_rects_at`; o Quadro fica na `strip_rects` de 12 px — é outro produto). Com o traço
dentro da caixa (§2) a folga real cresce ainda `½w` × zoom. Gate:
`the_action_bar_and_the_card_float_one_handle_plus_lg_off_the_selection`.

## §4 — Os gizmos: antes × depois

| forma | F10 (pontos verdes iguais) | F11 (o idioma do Illustrator) | foi para o cartão |
|---|---|---|---|
| Rect arredondado | 4 pontos no lado de cima de cada canto | 4 ⊙ | — |
| Polígono | ⇕ · envergadura · canto | ⇕ · ⊙ | envergadura |
| Estrela | ⇕ · envergadura · vale-redondo · R2 · ponta-redonda | ⇕ · ⊙ · ○ (R2) · ⊙ | envergadura |
| Elipse / pizza | furo · início · fim (dentro do aro) | início · fim: ⊙ numa HASTE por fora do aro | furo |
| Espiral | voltas · raio interno | voltas (○) | raio interno |

- **A cara sai da LEI** (`ph2d_tool_vector::shape_handles_face::glyph_of`): `Rounding`/`Apex` ⇒ ⊙,
  `Steps`/`Count` ⇒ ⇕, `Sweep`/`ArcStart` ⇒ ⊙ na haste, o resto ⇒ ○. O render tem o vocabulário dele
  (`HandleFace`), a ponte é um `match` exaustivo em `shape_handles_live::face_of`.
- **Pintura:** aro `shape-handle` sobre miolo `bg-elev` (gate de contraste nos 8 temas), ⊙ = 1× a alça
  da caixa, ○ = 1,3×, ⇕ = 1,6× (o Illustrator ~2,5×: menor para não cobrir a alça de canto da caixa).
- **Durante o arrasto** (o Illustrator): a caixa só CONTORNO (`GizmoStateGroup::outline_only`, sem
  hits), a ficha do número junto do cursor (`handle_readout` — `«Points: 7»`, o nome pela porta do
  painel `nomes_do_motor::campo`, os comprimentos pela `LengthDisplay`), e no raio o círculo
  TRACEJADO pelo ponto agarrado (`radius_ring`).
- **O ⊙ do rectângulo DENTRO do canto** (janela 2): `HandleDesc::in_corner` (só o `round_corner`)
  ⇒ o `handle_grip` desenha-o no CENTRO do arco (`spots::corner_centre`: a quina + o início do arco
  em cada aresta; com raio zero `park_at` em cada eixo, na diagonal). O ponto VERDADEIRO fica no
  início do arco, na aresta, e a lei `Nearest` não muda: a mão diagonal projecta na aresta e o
  centro anda na diagonal com ela. Gates `a_corner_handle_sits_inside_the_corner_at_the_arc_centre`
  (os 4 cantos) e `a_parked_handle_grabbed_and_held_still_changes_nothing` (a mão na diagonal).
  O balão rectangular (`SPEECH_RECT`) fica com o canto estacionado na aresta (o canto dele roda com o
  bico; sem foto do Illustrator).

## §5 — ⛔ Recusas MEDIDAS

| proposta | porque não |
|---|---|
| a caixa pela `path_bounds_under` (já existia) | conservadora: o miter a 4×½w sempre — um rectângulo de traço 10 ganhava 20 de folga |
| caixa visual = a local escalada pelo afim | a caneta é REDONDA sob escala desigual (`√|det|`): sob `(3, 1)` a borda erraria `0,3` contra `0,173` |
| afastar também a barra do Quadro | outro produto (MiroClone, outra linha); `strip_rects` fica, a ④ do vetor usa a `strip_rects_at` |
| ficar na geometria como o Affinity | o pedido do dono é explícito; o Affinity foi lido e registado (§1) |

## §6 — As cenas (por fazer nesta linha — ver o handoff de continuação)

`=15` e `=16` re-fotografar (as caras novas); `=17` nova: uma forma com traço GROSSO e a caixa por
fora dele, a ④ e o cartão à folga nova.
