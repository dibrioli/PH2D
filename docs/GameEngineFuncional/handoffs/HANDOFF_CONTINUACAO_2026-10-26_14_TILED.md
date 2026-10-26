# Handoff de continuação — `line/GameEngineFuncional`, plano 14 (Tiled com tamanho) (26/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`; a onda 14 vai de
`6c3e8b1011` a `8ccdb31a1a`. **Não integrada, não empurrada.** Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-26_13_ISOLAMENTO_POR_USO.md`](HANDOFF_CONTINUACAO_2026-10-26_13_ISOLAMENTO_POR_USO.md).
Plano: [`14_tiled_com_tamanho_plano.md`](../14_tiled_com_tamanho_plano.md) (§«Feito (26/10)» resume).

## Estado

**Smoke do dono (onda 14): POR FAZER.** Origem: smoke da onda 13 (26/10). Largar «Platform» dos Assets
mostrou Scale (40,16): caiu no Sky (Sprite 40×16 por escala, parallax 0, cobre a vista) e a lei da queda
retexturou o sprite sob a mão; a Platform Low tinha Scale (0,8;1) + Sampling (UvTransform 2,4 + Repeat). Dono:
«porque tanta complexidade?». Decisões (memória `project_tiled_blocks_and_canvas_drop_always_creates_2026_10_26`):
(1) imagem largada no canvas SEMPRE cria objecto; retexturar só pela ranhura Image do Inspector; (2) bloco que
repete = «Tiled com tamanho» (Draw Mode Tiled da Unity): Scale 1, Width/Height, a imagem repete-se sozinha.

| sha | O quê |
|---|---|
| `6c3e8b1011` | fix: a queda no canvas cria sempre (`asset_drop.rs::resolve`; `DropOver` perdeu `is_sprite`; `asset_drag_wire` simplificado) |
| `2db59d42b2` | feat D1/D2/D4/D6: `TiledDraw`, níveis com escala 1 |
| `8ccdb31a1a` | feat D3/D5: caixa Tiled e Width/Height no Inspector; collider automático segue o tamanho |
| (este) | docs: handoff + plano com o feito |

## O que entrou

- **`ph2d-ecs`:** componente `TiledDraw{tile}`; `resolve_uv_xform` (repeats = size/tile; offset do `UvTransform`
  mantido) e `resolve_draw_repeat` (Tiled repete; o Mirror próprio fica), usados por
  `sim_extract_emit::style` (shell). Registo de componentes 110→111. `PROJECT_SCHEMA` 194→195; 194/193 lidos
  pelo tipo vivo (braço em `project_load_read.rs`, const `PROJECT_SCHEMA_SEM_TILED_DRAW`). Reconte com
  `python3 scripts/schema-recount.py` na integração.
- **`ph2d-app-components`:** `asset_drop.rs::resolve` — imagem no canvas = `SpawnImage` sempre; `perform_drop`
  agora `pub(crate)`.
- **`ph2d-app-sprite-anim`:** `image_choice` Lei 5 (escolher imagem numa Tiled guarda o tamanho; tile = célula
  nova × razão antiga); `place_library_image`: imagem de fábrica com `FactoryArt.tile` nasce Tiled; `image_use`:
  Tiled = Repeated; `make_animated` remove `TiledDraw`; `tiled.rs`: `draw_of`, `set_tiled` (tile = célula
  nativa, tira Repeat/UV à mão), `set_tiled_side` (MIN 0,01 m; Cuboid que media size/2 segue, o feito à mão
  fica); `image_use::apply_edit`.
- **`ph2d-editor-core`:** `InspectorDraw{Fixed,Simple,Tiled{size}}` em `InspectorSpriteInfo.draw`;
  `ImageUseEdit{MakeAnimated,Tiled(bool),Width,Height}`; `EditorAction::ImageUseEdit` **substitui**
  `MakeImageAnimated`.
- **`ph2d-panel-inspector`:** `event_image_use.rs` (populate/sync/event) + `image_use_rows::paint_draw_rows`
  (caixa Tiled, linhas Width/Height). Catálogo: `TiledDraw` movido para `catalog/core.rs` (ordenado), campo
  `keyed("tile")` escondido NoCard; censo de canais: `INSP_IMAGE_TILED` NaoAnimavel.
- **`ph2d-app-game`:** níveis 2+: blocos Sprite 2·meia, escala 1, `TiledDraw`, Collider Cuboid = meia; Invisible
  Wall escala 1 com collider caixa. Plano da foto: `PlanoDaFoto.edicoes`; a «larga» passa por
  `asset_drop::resolve` com o Sky sob a mão. Fotos [`fotos_14/`](fotos_14/): `nivel_4_baixa` (Platform Low
  Scale 1/1, Tiled ✓, Width 2,400 Height 0,500), `nivel_4_larga` (Width 4: alarga e repete),
  `nivel_4_largada` (Platform sobre o céu: objecto novo «Platform», Scale 1, Tiled, Width 1, Height 0,5; céu
  intacto). Env: `PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=baixa|larga|largada`.

## Superfície que um merge pode partir

- `EditorAction::MakeImageAnimated` **desapareceu** (`match` exaustivo alheio parte; usar `ImageUseEdit`).
- `InspectorSpriteInfo.draw` (campo novo): todo literal de struct em fixtura tem de o somar.
- `DropOver` sem `is_sprite`; `asset_drag_wire` (shell) mudou; `perform_drop` `pub(crate)`.
- Registo de componentes 110→111 e `PROJECT_SCHEMA` 194→195: **soma entre linhas, reconta-se**.
- Catálogo: ordem de `catalog/core.rs` tem gate (`the_catalog_is_sorted_and_unique`).
- Ficheiros partilhados da shell: `sim_extract_emit`, `asset_drag_wire`, `project_load_read.rs`, `fase_bus_*`.
  Conferir com `collision-surface.sh` pelo caminho absoluto contra o merge-base `c3662cd8e4`.

## Prova de fecho (DELTA)

Verdes, POR NOME: asset_drop (15) · smoke::tests (12, correr/saltar nos colliders novos incluídos) ·
camara_tests (3) · image_use (6) · image_choice (13) · registry (6) · project_schema (1) · `tiled::` (4:
`largura_leva_o_collider_automatico`, `collider_a_mao_fica`, `trocar_a_imagem_de_um_tiled_guarda_o_tamanho`,
`ligar_e_desligar_o_tiled`) · seam_image_use (4) · `the_catalog_is_sorted_and_unique` ·
`todo_campo_pintado_com_losango_tem_canal_ou_porque`. clippy `-D warnings` em 10 crates e fmt limpos.
**Mutação:** desligar o collider-segue pôs `largura_leva_o_collider_automatico` vermelho; restaurado.
Suíte completa: só pelo integrador (regra do dono 23/10).

### Vermelhos herdados (NÃO são das ondas 13/14)

Medidos vermelhos já em `6c3e8b1011` E em `f0ddde51ba^` (worktree destacado). Para o fecho da linha / integrador:

- registry-init: `a_carga_de_comandos_de_um_painel_so_encolhe` (inspector 92 vs 89) · `a_altura_de_abertura_de_um_painel_so_encolhe` (inspector 856 vs 831; a onda 14 não o mudou) · `nenhum_corte_novo_entra_sem_ser_nomeado` (corte «None — click to choose») · `a_divida_do_degrau_estreito_so_encolhe` (87 vs 86) · `as_letras_perdidas_no_degrau_estreito_so_encolhem` (85 vs 84).
- editor-core: `architecture_no_dependency_climbs_a_layer` (app-game/app-sprite-anim → app-components/app-physics; sprite-anim dev → app-game) · `architecture_the_foundation_modules_form_a_dag` (`widget/sheet_cell.rs:5` widget→screens) · `todo_campo_sem_coluna_propria_ganha_a_do_losango` (`anim_rules.rs` `rule_rows`) · `nenhuma_isencao_sobrevive_ao_codigo_que_a_explicava` (`anim_card.rs` `character_rows`).
- app-components: `o_cartao_pinta_a_chave_do_catalogo` (animator.playing/autoplay) · `property_names_tests::o_nome_de_codigo_e_o_rotulo` (CharacterAnimations: código «character_animations» vs rótulo «Auto Animate»).

## Smoke (para o Enio)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Hierarchy → **Platform Low**. Inspector: Scale 1/1; em Texture & Atlas, **Tiled** marcado, Width 2,4, Height 0,5.
3. Mude **Width** para 4: a plataforma cresce e a imagem repete-se (não estica). **Run ▸ Play in a Window**: o
   boneco anda por cima da parte nova.
4. Janela **Assets**: arraste **Platform** para o canvas (mesmo sobre o céu): nasce o objecto «Platform», Tiled,
   Width 1; com Width 2,4 fica igual à Platform Low; o céu não muda.
5. Desmarque **Tiled**: a imagem estica. **Ctrl+Z** volta.
6. **File ▸ Save**, feche, abra: igual.
7. Deu errado se: Scale ≠ 1 nos blocos; a imagem estica com Tiled ligado; o boneco cai pela parte alargada; a
   queda muda o céu; algo muda depois de reabrir.

## ABERTO

- Smoke do dono da onda 14 por fazer.
- Menu Add «Static Body» ainda nasce BLOCO 3×0,5 branco sem Tiled (o dono pode querer Tiled).
- «Platform» largada é imagem sem Physics Body (não colide): para colidir, adicionar Physics Body.
- Ficheiros antigos mantêm a forma velha (escala + UV) e desenham igual; não são convertidos.
- Script `sprite.image` num objecto que repete mantém o tamanho (da onda 13).
- Mudar o ppm do projecto não reescreve `TiledDraw.tile`.
- Vermelhos herdados acima (decidir no fecho da linha); suíte completa só antes da integração.
- Da onda 13/12 ainda de pé: 12b Flip/Vector pela mesma porta · imagem importada sem uso esquece o corte (D3) · modelo Godot adiado.

## PRÓXIMO

O smoke do dono decide. Depois: 12b ou J5 moedas ([plano 03](../03_plano_jogo_de_plataforma.md)), conforme o dono.
