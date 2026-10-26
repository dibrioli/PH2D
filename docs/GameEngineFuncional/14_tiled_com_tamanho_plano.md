# 14 — Tiled com tamanho: o plano (26/10)

Origem: smoke do dono da onda 13 (26/10). Ele largou «Platform» e viu Scale (40,16); a Platform Low tinha
Scale (0.8,1) e o Sampling mexido. Perguntou: *«porque tanta complexidade?»*. Decisões dele (memória
`project_tiled_blocks_and_canvas_drop_always_creates_2026_10_26`):

1. Largar uma imagem no canvas SEMPRE cria objecto; trocar é só pela ranhura Image. **Feito** (`6c3e8b1011`):
   o céu (Sprite 40×16 por escala) cobria a vista e cada queda retexturava-o.
2. Bloco que repete = **Tiled com tamanho** (Draw Mode Tiled da Unity): Scale 1, Largura/Altura no objecto,
   a imagem repete sozinha.

## O que existe (lido)

- O bloco é `Sprite.size = BLOCO [3, 0,5]` × `Transform.scale` (0,8 , 1) + `UvTransform.scale` (2,4 , 1) escrito
  À MÃO + `TextureRepeat::Enabled` (`smoke.rs` `azulejo`/`arruma`). Dois controlos que têm de combinar.
- O UV é `wrap(quad_uv · UvTransform.scale + offset)` (`ph2d-ecs/src/sampling.rs`), lido em
  `sim_extract_emit::style`. Nada deriva repetições do tamanho.
- O collider nasce de `Sprite.size / 2` (`ph2d-app-physics/src/physics_seed.rs`, só se a forma é a default) e
  a ponte multiplica-o pela escala (`ph2d-physics-ecs/src/scale.rs` `scaled_shape`). Mudar `Sprite.size`
  depois NÃO mexe no collider.
- O 9-Slice (`SliceNine`) já ladrilha por tamanho, mas é a ferramenta da moldura (8 células, bordas); o
  `Tiled` dele foi retirado de propósito (`slice_nine.rs`). ⛔ Não é a casa disto: o dono pediu menos
  controlos, não a grelha 3×3.
- `ScrollRepeat` é o envoltório da paralaxe (ladrilho por onde o fundo dá a volta), outra lei.

## Desenho

**D1 — `TiledDraw { tile: [f32; 2] }`** (ph2d-ecs, componente opcional, gravado): «esta sprite desenha a sua
imagem em ladrilhos de `tile` metros até encher `Sprite.size`». `tile` é o tamanho NATIVO de uma célula da
imagem (`image_facts`), escrito pela porta da imagem; guardado porque o extract não tem a biblioteca.
Ausente = a sprite de sempre (byte-idêntico).

**D2 — O desenho:** no extract, com `TiledDraw`, `uv_xform.scale = size / tile` (o offset do `UvTransform`
continua), e o repeat resolvido `Inherit/Disabled` passa a `Enabled` (`Mirror` fica). `UvTransform.scale` é
ignorado nessa sprite (o Inspector esconde-o). Ficheiro antigo: nenhum tem `TiledDraw` ⇒ igual.

**D3 — A colisão segue o tamanho:** mudar Largura/Altura de uma sprite cujo `ColliderShape::Cuboid` media
`size/2` (o collider «automático», como o *Auto Tiling* do BoxCollider2D da Unity) leva o collider junto, no
MESMO passo de undo. Um collider que o artista mudou à mão fica.

**D4 — As portas:** `choose_library_image` numa sprite com `TiledDraw` guarda o tamanho e reescreve `tile`.
`place_library_image` de uma imagem-azulejo (`FactoryArt.tile`) nasce com `TiledDraw` (tamanho = 1 ladrilho)
⇒ «Platform» largada é o MESMO tipo de objecto da Platform Low. `image_use`: `TiledDraw` ⇒ *repetida*.
`make_animated` tira-o.

**D5 — Inspector (Texture & Atlas):** linha **Draw Mode** `Simple | Tiled` e, em Tiled, **Width / Height**
(m) = `Sprite.size`. Ligar Tiled: `tile` = célula nativa, tamanho fica. Desligar: tira o componente. Sampling
esconde UV Scale numa sprite Tiled.

**D6 — Os níveis do jogo** nascem com Scale 1, `Sprite.size = 2·meia`, `TiledDraw`, collider de `size/2` (a
parede invisível filha do chão recalcula-se sem a escala do pai).

## Gates (red primeiro)

1. `o_bloco_do_nivel_e_tiled_com_escala_1` (jogo): Platform Low/High, chão, paredes: scale 1, `TiledDraw`,
   collider = size/2, mesmos texels que antes na foto do nível 4.
2. `tiled_repete_pelo_tamanho` (extract): size 2,4 × 0,5, tile 1 × 0,5 ⇒ uv scale (2,4 , 1), repeat Enabled.
3. `largar_platform_e_o_mesmo_tipo_da_platform_low` (sprite-anim): `TiledDraw`, size 1 × 0,5.
4. `largura_leva_o_collider_automatico` + `collider_a_mao_fica` (shell/commit), um passo de undo.
5. `trocar_a_imagem_de_um_tiled_guarda_o_tamanho` (sprite-anim).
6. Seam do Inspector: Draw Mode e Width/Height pintam e emitem.
7. Ficheiro antigo (escala + UV) abre e desenha igual (os de 12/13 continuam).

## Smoke `PH2D_GAME_SMOKE=4`

Platform Low: Scale 1,1 · Draw Mode Tiled · Width 2,4. Mudar Width para 4 ⇒ a plataforma cresce, a imagem
repete (não estica) e o boneco anda por cima da parte nova. Arrastar «Platform» para a tela ⇒ objecto Tiled
1 × 0,5; Width 2,4 ⇒ igual à Platform Low. Fotografado.

## Feito (26/10)

- **Lei da queda (`6c3e8b1011`):** `asset_drop::resolve` (app-components) — imagem no canvas = `SpawnImage` SEMPRE; `DropOver` perdeu `is_sprite`; a escolha da shell (`asset_drag_wire`) simplificou. Retexturar só pela ranhura Image do Inspector. Origem: a «Platform» largada caía no Sky (40×16 por escala, parallax 0) e retexturava-o.
- **D1/D2 (`2db59d42b2`):** `ph2d-ecs` `TiledDraw{tile}` + `resolve_uv_xform` (repeats = size/tile, offset do UvTransform mantido) + `resolve_draw_repeat` (o Mirror próprio fica); usado por `sim_extract_emit::style`. Registo 110→111; `PROJECT_SCHEMA` 194→195, 194/193 lidos pelo tipo vivo (braço em `project_load_read.rs`, `PROJECT_SCHEMA_SEM_TILED_DRAW`).
- **D4:** `image_choice` Lei 5 (escolher imagem numa Tiled guarda o tamanho; tile = célula nova × razão antiga); `place_library_image` de azulejo de fábrica (`FactoryArt.tile`) nasce Tiled; `image_use` Tiled = Repeated; `make_animated` tira `TiledDraw`.
- **D6:** níveis 2+: blocos com Sprite 2·meia, escala 1, `TiledDraw`, Collider Cuboid = meia; Invisible Wall escala 1 com collider caixa.
- **D3/D5 (`8ccdb31a1a`):** editor-core `InspectorDraw{Fixed,Simple,Tiled{size}}`, `ImageUseEdit{MakeAnimated,Tiled(bool),Width,Height}` (`EditorAction::ImageUseEdit` substitui `MakeImageAnimated`); sprite-anim `tiled.rs` (`draw_of`, `set_tiled`, `set_tiled_side` MIN 0,01 m; Cuboid que media size/2 segue, o feito à mão fica); panel-inspector `event_image_use.rs` + `paint_draw_rows` (caixa Tiled, Width/Height).
- **Catálogo/canais:** `TiledDraw` em `catalog/core.rs`, campo `tile` escondido NoCard; `INSP_IMAGE_TILED` NaoAnimavel.
- **Gates verdes, por nome:** asset_drop (15), smoke::tests (12), camara_tests (3), image_use (6), image_choice (13), registry (6), project_schema (1), `tiled::` (4), seam_image_use (4), catálogo ordenado, losango/canal. clippy `-D warnings` em 10 crates e fmt limpos. Mutação: desligar o collider-segue pôs `largura_leva_o_collider_automatico` vermelho; restaurado.
- **Fotos:** [`handoffs/fotos_14/`](handoffs/fotos_14/) `nivel_4_{baixa,larga,largada}.png` (`PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=baixa|larga|largada`).
- **Smoke do dono: POR FAZER.** Handoff: [`HANDOFF_CONTINUACAO_2026-10-26_14_TILED.md`](handoffs/HANDOFF_CONTINUACAO_2026-10-26_14_TILED.md).
