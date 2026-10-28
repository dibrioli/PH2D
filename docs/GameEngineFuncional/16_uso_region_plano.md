# Plano GEF 16 — o 4.º uso da imagem: **Region** (a área escolhida num editor próprio)

Pedido do dono (27/10): *«o usuário abre a imagem e com uma seleção retangular com gizmo de sprite seleciona a área
que deseja exibir. Ao abrir a imagem para selecionar a área pode configurar o grid w × h para facilitar o snap da
seleção»*. Decisões do dono (27/10, não reabrir): o botão chama-se **Region** (Still · Repeated · Animated ·
Region) · a grelha configura-se em **colunas × linhas** · a seleção faz-se num **editor próprio**, nunca no canvas.

Antecessor: [plano 15](15_uso_da_imagem_a_vista_plano.md) e o handoff
[`HANDOFF_CONTINUACAO_2026-10-26_15_USO_A_VISTA.md`](handoffs/HANDOFF_CONTINUACAO_2026-10-26_15_USO_A_VISTA.md)
(cada uso guarda a sua configuração · nenhuma troca muda a escala · Repeated conta cópias).

## O que já existe (medido 27/10, não reconstruir)

- `ph2d_ecs::SpriteRegion { rect: [f32;4] px de textura, filter_clip }` (`crates/ph2d-ecs/src/sprite_region.rs:20`),
  registado; desenhado por `region_subrect` (`crates/ph2d-render/src/sprite/subrect.rs:56`). ⚠️ É o par de
  `SpriteSheetRef` (folha empacotada à mão) e entra em `image_choice.rs:100`: usá-lo como marcador do uso é
  ambíguo. Campos X/Y/W/H numéricos em `sections/render_source_regiao.rs:140-190`, sem gizmo.
- `UvTransform { scale, offset }` (UV), já guardado em `ImageUseLook.uv`.
- Editor de animações (`crates/ph2d-app-sprite-anim/src/editor/`): pinta a folha com as células, sem zoom, pan
  nem alças. Gizmo de 8 alças do canvas: `ph2d-editor-core/src/gizmo/hit.rs:89-108` (escala do objecto).
  Recorte com gesto: `ph2d-board-edit/src/crop.rs:74` (do Quadro). Grelha proposta: `ph2d_sprite_sheet::guess_grid`.
  Não há snap de grelha em parte nenhuma.

## D1 — o uso Region

- `InspectorImageUse::Region` / `ImageUse::Region` (4.º de `ALL`); `INSP_IMAGE_USE: [NodeId; 4]`; i18n
  `panel.inspector.image.use_region`. O seletor em 2 × 2 (Still Repeated / Animated Region).
- **Estado:** MEDIR primeiro (§5.0) se `UvTransform` sozinho exprime o recorte no render (scale = w/W, offset = x/W),
  sem `SpriteGrid`. Se sim, o marcador é um componente novo pequeno `ImageRegion { rect_px: [u32; 4], grid: [u32; 2] }`
  (a verdade em px; o `UvTransform` deriva-se) — registado ⇒ `PROJECT_SCHEMA` +1, recontado por
  `scripts/schema-recount.py`, descritor no catálogo `image`, porta das macros. `image_use()` lê-o ANTES de Repeated.
- **Tamanho:** a área à `display_scale` de agora (nenhuma troca muda a escala). Region nova = a imagem INTEIRA
  (rect = 0,0,W,H) e o editor abre-se sozinho.
- **Memória:** slot `region` em `ImageUseMemory` (+ `is_empty`, `forget_looks`; o `ImageUseLook` ganha o
  `ImageRegion`). Trocar a imagem esquece o rect (ele descrevia a outra).
- Sob o seletor, em Region: a linha **«Edit Region»** (botão) e `W × H px` da área (só leitura).

## D2 — o editor de Region (irmão do editor de animações, `ph2d-app-sprite-anim`)

- Abre pelo «Edit Region»: a imagem INTEIRA grande, zoom (roda) e pan (botão do meio / espaço), fundo xadrez.
- No topo: **Columns** e **Rows** (nascem do `guess_grid`; sem proposta 1 × 1) — a grelha pinta-se por cima.
- O retângulo: arrastar dentro move; 8 alças redimensionam; arrastar fora desenha um novo. **Snap às linhas da
  grelha** sempre (com 1 × 1, ao píxel); `Alt` desliga o snap (ao píxel). Nunca sai da imagem; mínimo 1 px.
- **OK** aplica (UM passo de desfazer: rect + tamanho + collider automático, `follow_auto_collider`); **Cancel** /
  `Esc` nada muda. A grelha escolhida fica com o objecto (`ImageRegion.grid`).

## Gates (por nome) e fotos

1. As 12 trocas entre os 4 usos (o caminho que cobre cada par ordenado), 2 voltas, desfazer e reaberto a meio —
   generalizar `cada_troca_ida_e_volta_devolve_o_uso_como_estava` (`image_use_copias_tests.rs`); nenhuma muda a
   escala (`nenhuma_troca_de_uso_muda_a_escala` com Region).
2. Snap: o rect arrastado cai nas linhas de `cols × rows` (folha 800 × 400, 8 × 4 ⇒ múltiplos de 100); `Alt` ao
   píxel; nunca fora da imagem.
3. OK = um passo de desfazer; Cancel não muda nada.
4. O render mostra só a área (o píxel do canto da área no canto do objecto) — gate de render ou de UV.
5. Costura do painel: o 4.º botão chega como `ImageUseEdit::Use(Region)`; «Edit Region» abre o editor.
6. Mutação sobre o snap, a memória do slot e o OK num passo.
Fotos: o seletor 2 × 2; o editor com a grelha 8 × 4 e o retângulo numa célula; o Player na cena só com essa área.

## Smoke do dono (rascunho)

Player → **Region** → abre o editor → **Columns 8 · Rows 4** → arrastar o retângulo sobre um boneco (encaixa
nas linhas) → **OK** → na cena só esse boneco, do mesmo tamanho → **Still** → **Region** → volta a mesma área.
