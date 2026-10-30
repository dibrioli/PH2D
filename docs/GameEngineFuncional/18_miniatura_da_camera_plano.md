# 18 — A6: a miniatura do que a câmera vê (plano, 30/10)

Plano [17](17_camera_no_canvas_plano.md) A6; handoff [18](handoffs/HANDOFF_CONTINUACAO_2026-10-30_18_CAMERA.md) §ABERTO 1.
Lei: a miniatura é FIEL (sprites, vetor, Flip, Motion, luz 2D) ou não existe. Meia-miniatura de sprites = recusada.
É, na prática, o R1 recusado em [04](04_R0_N0_plano.md):79 («desenhar o mundo 2× por quadro»).

## Medição (30/10, `fotografa_cena.sh` 1930×1040, `PH2D_FLUID_PROFILE=1 PH2D_LIGHT2D_CUSTO=1`, load 3,4)

| Cena | GPU/quadro | Partição |
|---|---|---|
| `PH2D_GAME_SMOKE=4` | 1,30–1,42 ms | `render.vello` 1,13–1,24 (mundo + chrome) · sprite 0,07–0,08 · compositor 0,03–0,05 · tonemap 0,02 |
| `PH2D_LIGHT2D_SMOKE=8` | luz 3,80 ms (min) + 0,53–0,71 | luz: grade 1280×704 da janela 1930×1012; c2 2,32 · miolo 0,61 · preenche 0,49 · fina 0,25 |

Quadro preso ao vsync (16,1 ms; `cpu-encode` 5,9–8,8 ms). Miniatura 320×180 ≈ 1/36 dos píxeis da janela ⇒ a 2.ª
passagem na placa é estimada em < 1 ms com luz (a grade da luz escala com o alvo). **Por medir depois da etapa 3:**
o CPU da 2.ª codificação (vetor + `recolher` da luz) — régua: `cpu-encode` com a miniatura On × Off, intercalado.

## O que a leitura (explorador ×2) estabeleceu

- Câmera e tamanho vêm de `gfx.camera` / `surface.size()` (`present.rs:62`). Aceitam câmera+alvo por parâmetro:
  `render_with_streams` (`renderer_draw.rs:99`, `submit` próprio por chamada ⇒ 2 chamadas sequenciais não colidem
  no `camera_buffer`), Flip `pass::render`, placa do Motion `desenha_o_quadro`, Vello `render_to_intermediate`.
- **Vetor:** codificado com a câmera do EDITOR embutida (`fase_hero_scene.rs:47` `cam_affine` →
  `ph2d_vec_render::dispatch`), na MESMA `VectorScene` que gizmos e chrome (`present_chrome.rs:170-180`); só com
  faixas vai ao HDR. ⇒ `append` com outra afim é recusado (leva o chrome e a câmera errada): 2.ª codificação do
  DOCUMENTO numa `VectorScene` própria com a afim da câmera do jogo.
- **Luz 2D:** `LuzDoQuadro` é estado de UMA vista (`mundo_para_tela`, `alvo_px`, `resolvido`, `vigia`,
  `governador`; `quadro.rs:66-109`, criada em `init.rs:174`) ⇒ 2.ª instância para a miniatura.
- **Vello:** `vello_pass.intermediate` é um alvo só ⇒ `VelloPass` (ou intermediário) próprio.
- **Vazam se reusados:** fantasmas do onion (slot `extra`, HDR), gizmos/moldura/ossos (cena do chrome), grade.
- Retrato do asset (`asset_card_portrait.rs:11-14`) é CPU sobre pixels — esta capacidade NÃO o serve (o handoff 18
  dizia o contrário).

## Desenho: a vista da câmera TROCA-SE com a do editor (decisão técnica, 30/10)

Fidelidade por construção: a miniatura corre **o MESMO código** do present (nenhuma 2.ª cópia da lei de
desenho — [cheiro da lei duplicada](../../project-memory/feedback_a_promise_of_sameness_beside_a_copy_is_the_shape_that_diverges.md)).
Uma `VistaDaCamera` guarda instâncias próprias de tudo o que é **por vista** (depende do tamanho do alvo ou da
câmera) e faz `mem::swap` com os campos do `AppGfx` à volta das fases que desenham o mundo:

| Por vista (trocado) | Porquê |
|---|---|
| `camera`, tamanho | a câmera do jogo e o alvo pequeno |
| `game_rt` · `world_rt` · `tonemap` · `vello_pass` · `motion_fx` | alvos do tamanho da janela (`ensure_size`) |
| `band_doc_scenes` | o documento codificado com a afim da câmera |
| `luz2d` (`LuzDoQuadro`) | estado de uma vista (`alvo_px`, `resolvido`, `vigia`) |
| `flip_render` · `flip_compose` | `ensure_depth`/scratch por tamanho (`composite.rs:160+`) — alternar tamanhos recriaria 2×/quadro |
| `motion_shell.placa` (`PlacaDeFormas`) | camada por tamanho (`motion_shape_placa.rs:225`) e codificada com `cam_affine` |
| `Renderer.clip_stencil` | partilhado; passa a guardar um por tamanho (2 entradas) em vez de recriar |

Partilhado (lido): `present`, `sim`, `renderer` (atlas, pipelines; `submit` por chamada), `motion` (o cook),
`flip` (doc). Fora da miniatura (editor, não jogo): onion, tinta da máscara, fantasmas/peek do Flip, traço em curso,
grade atrás, vidro da receita, lente do split, sonda do drift, gizmos/chrome (a cena do chrome não é desenhada).
A miniatura é sempre **em faixas** (o mundo inteiro acaba no `world_rt` dela) e cola-se no quadro por `band_blit`
depois do compositor, num canto do canvas. Ordem no quadro: a vista da câmera é desenhada **antes** da principal
(cada passe faz o seu `submit`; o `ligar` da luz principal reescreve os receptores depois).

## Etapas (cada uma com o seu gate por nome)

1. **Alvos da vista** (`ph2d-render`): `VistaDoMundo { game_rt, tonemap, vello }` no tamanho da miniatura.
2. **Luz da vista**: 2.ª `LuzDoQuadro` (sem montagem em fundo partilhada), alimentada com a câmera do jogo.
3. **Vetor da vista**: `dispatch` só do documento com a afim da câmera do jogo, sem overlays.
4. **Flip e Motion**: as passagens com a câmera e o tamanho da vista; conferir `FlipRenderer`/`FlipCompose`,
   `motion_fx`/`layer_compositor` (`write_buffer` antes do submit) e `present_frost::lift` (destrutivo?).
5. **Composição** num canto do canvas (blit), moldura fina, só com a câmera seleccionada e Preview On.
6. **Cartão Camera**: «Preview: Off | On» volta (`CameraCanvasView.preview`), gesto do editor como Outside/Editor View.
7. **Prova de fidelidade**: a miniatura × o mesmo mundo desenhado em tamanho cheio com a câmera do jogo (o Play),
   reduzido — por cena de smoke com vetor + Flip + Motion + luz; e a medição On × Off.

Código novo em crate (`ph2d-app-game` / `ph2d-render`); a shell só encolhe (`the_shell_only_shrinks`).

## Feito (30/10) — medido

- Construída como desenhada acima (`vista_da_camera.rs`): o pedido na fase da câmera (a seleccionada, «Preview: On»,
  fora do palco), a codificação no fim das faixas (`fase_vector_bands`, com `preferir` do nível da luz do editor), o
  desenho no início do present (antes do do editor), o retrato `Rgba8Unorm` registado no Vello e pintado na fase da
  selecção. Canto: em baixo à direita, por cima da faixa do HUD (`last_layout.bottom_hud.y`), abaixo da régua; com o
  ponteiro por cima fica só o fio (uma alça da moldura por baixo tem de se ver).
- **O atlas do Vello é persistente** (0.10+): sem `mark_texture_dirty` por quadro a miniatura ficou parada no 1.º
  quadro — que era o quadro SEM luz (a estufa da luz da vista ainda a aquecer). Foi assim que o defeito se mostrou.
- Fidelidade, por foto (`fotos_18/nivel_4_miniatura*.png`, cenas `PH2D_GAME_SMOKE=4` + `PH2D_LIGHT2D_SMOKE=7|8` +
  `PH2D_MOTION_OBJETO_SMOKE=1`): traço do Flip, discos, raios e sombras da luz, a pilha do Motion e o enxame fora da
  moldura — posições conferidas à mão pela escala moldura→caixa (0,382).
- **Custo** (1930×1040, load 18, `PH2D_MINIATURA_CUSTO=1` + `PH2D_FLUID_PROFILE=1`, cena 4 + luz 8 + Motion 1):

| | min | med / típico |
|---|---|---|
| CPU da vista (codificar + desenhar) | 1,62 ms | 2,29 ms |
| placa, `render.miniatura` (span) | 5,58 ms | 6,06 ms |
| … dos quais a luz da vista | 3,29 ms (grade 512×320 para um alvo 366×205; entorno 0,64) | |
| quadro | 16,1 ms (62 fps, vsync) | |

  Leitura a load 31 (descartada, CPU sem fatia): 23 ms / 43 fps. ⇒ o custo é a LUZ da vista, que tem um piso de
  grade acima do tamanho do alvo — aberto, sem limite escrito (§0: ainda não pesa no quadro desta placa).

## ⛔ Fora da miniatura (aberto, com o porquê)

- **Os palcos das macros** (`motion_shell.palcos`): a placa de cada palco é codificada com a câmera do editor e
  tem estado de reprodução por instância — uma 2.ª instância por vista re-simularia. A vista salta-os (`na_vista`).
- **As formas do Motion que a placa NÃO leva** (a rota do Vello: `do_editor = motion_tool_active && !placa.ativa()`
  em `fase_vector_overlays.rs:48`, e a camada filtrada `motion.filtro.alvo_da_cena`): codificadas com o `cam_affine`
  do editor na cena do chrome ou na camada do filtro — na vista faltam (chrome) ou viriam com a câmera errada (filtro).
  Cura: a mesma `motion_shape_gen::encode` com a afim da vista numa cena da vista, colada como faixa.
- **Os palcos** e **estas formas** fecham na próxima janela (a mesma família: o que o Motion codifica com a câmera
  do editor fora da placa) — por estado por vista (`PalcoEmCena.placa` trocada como a do editor) e uma cena da vista.
