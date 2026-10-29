# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (P0..P6g) — 2026-10-28

> Leitor: o agente integrador e a próxima LLM. A linha **fecha e PARA**: não integra, não faz push.
> Lei do módulo: [ADR-0192](../../architecture/decisions/0192-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) · [plano](../02_plano_modulo.md) · [estudo](../01_estudo_aseprite.md) · handoffs de onda: `HANDOFF_CONTINUACAO_line_PixelArt_*` nesta pasta (P0..P6g; cada um tem §«ABERTO»).

## 1. Identidade

- Branch `line/PixelArt`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt`. Merge-base `c3662cd8e4` (= o `main` no fecho, 28/10). **114 commits** (`git log --oneline c3662cd8e4..HEAD | wc -l`); 464 ficheiros.
- O módulo Pixel Art NOVO e autocontido (decisão do dono 25/10: sem Painter; o Aseprite é a espinha das etapas). Ondas P0, P0b, P1, P2, P3, P4, P6, P6b..P6g. **Smoke do dono OK até P6f (27/10); P6g entregue, smoke do dono PENDENTE** (cena 15).

## 2. Crates NOVAS (`git diff --name-status … -- 'crates/*/Cargo.toml' | grep ^A`)

`ph2d-pixel-core` · `ph2d-pixel-io` · `ph2d-pixel-render` · `ph2d-app-pixel` · `ph2d-panel-pixel-tools` · `ph2d-panel-pixel-timeline`. `Cargo.lock`: só estes 6 `+name` (nenhum pacote externo novo).

## 3. Foundational tocado (todo ADITIVO, salvo o que está marcado) — 165 ficheiros, +6841/−638 nos caminhos foundational

`git diff --stat c3662cd8e4..HEAD -- crates/ph2d-editor-core crates/ph2d-ecs crates/ph2d-i18n crates/ph2d-tool-vector crates/ph2d-panel-vector crates/ph2d-app-vec crates/ph2d-app-physics crates/ph2d-app-audio crates/ph2d-app-components crates/ph2d-component-desc crates/ph2d-app-macros crates/ph2d-script crates/ph2d-render crates/ph2d-ui-testkit crates/ph2d-panel-registry-init crates/ph2d-app-registry-init shells/desktop`. `ph2d-editor-core`: 73 ficheiros, +4983/−40.

Pontos que a auditoria verificou:
- **Gizmo de sprite**: 4 zonas de inclinar `GIZMO_SKEW_T/R/B/L = NodeId(974..977)` (`ph2d-editor-core/src/gizmo/hit.rs`), só REGISTADAS pelo Pixel (`paint_sprite_gizmo_with`); quem não as regista não as vê. Teste de colisão de ids: `node_id_collisions` (+6).
- ⚠️ **MUDANÇA VISÍVEL para outras famílias — `flyout_cols`** (`screens/hero/float_bar.rs:132`): muda o layout de TODOS os flyouts (nunca um botão sozinho na última linha; 4 → 1 linha; o grid de formas do Vector/Quadro passa a 5 ou 4 colunas). Testes posicionais que assumam 3 colunas fixas podem mudar na árvore combinada.
- **`vector_tool_active`** (shell) passa a ser falso quando outra família possui o canvas (Vector escondido mas activo com Pixel Sprite seleccionado).
- `PixelPanelInfo`/`PixelOptions` ganharam campos (com `Default`); `ph2d-tool-vector` expõe `shape_handles*`/`shape_thumbs` ao Pixel (formas vivas do Vector, ADR-0192 §12); o catálogo de contornos das formas (`shape_outlines.rs` do `ph2d-panel-vector`) **MOVEU** para `ph2d-tool-vector/src/shape_thumbs.rs` (o Pixel precisa dele; o flyout do Vector passa a chamá-lo de lá) e `nomes_do_motor::forma` virou `pub` (era `pub(crate)`); `BarFace`/barras ganharam os campos `grouped`/`pixel_rail` (default vazio/falso).
- **MOVIDO (a shell encolheu)**: `physics/bridge_tests.rs` (−257) → `ph2d-app-physics`; testes de áudio (`render_loop/audio_overlay.rs`) → `ph2d-app-audio`. `app_state.rs` 920 e `main.rs` 1058 linhas (tecto 600 do aviso do script) já excediam antes: aqui só ENCOLHEM (`the_shell_only_shrinks` ✓).
- Shell: `input_dispatch/despacho_pixel.rs` (novo), `keyboard_palette.rs`, `render_loop/fase_pixel_panel.rs`, `fase_object_add.rs` (+21), `snapshots.rs` (+37), `project*.rs` (§5).

## 4. Contratos congelados (§6): NÃO tocados

```
$ git diff --name-only c3662cd8e4..HEAD | grep -E "ph2d-(nodegraph|tool-registry|tool-api|vector/|imageio)|contract_surface"
(vazio)
$ git diff --stat c3662cd8e4..HEAD -- crates/ph2d-editor-core/src/tool.rs
(vazio)
$ git diff c3662cd8e4..HEAD -- crates | grep -E "^\+.*(impl .*(Tool|NodeOp|OpResolver|RasterEditTool|CanvasPaintTool) for|trait (Tool|NodeOp))"
(vazio)
```
`collision-surface.sh`: `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` «intocado».

## 5. Superfície de colisão (colada de `bash $PWD/scripts/collision-surface.sh`, 28/10)

```
▸ SCHEMAS    PROJECT_SCHEMA 194 (base 193) · tripla do gate (194, 13, 23) (base (193, 13, 23))
             VEC_SCENE_SCHEMA 23 · FLIP_SCHEMA 13 · DOC_VERSION (timeline) 23 — iguais à base
▸ REGISTO    ph2d-ecs 109 (base 108) · ph2d-render espelho 110 (base 109) · ph2d-script espelho 110 (base 109)
▸ ADR        esta linha cria 0192 (último no disco 0192; próximo livre 0193)
▸ Cargo.lock 6 '+name': ph2d-app-pixel, ph2d-panel-pixel-timeline, ph2d-panel-pixel-tools, ph2d-pixel-core, ph2d-pixel-io, ph2d-pixel-render
▸ MARCADORES nenhum
▸ LOC        ✗ 920/600 app_state.rs · ✗ 1058/600 main.rs (pré-existentes; a linha só os encolhe)
```

Formato gravado (ADR-0192): `PROJECT_SCHEMA 193 → 194` (`shells/desktop/src/project_schema.rs`), campo `pixel_sprites: Vec<u8>` no FIM do `ProjectFile` (blob `PH2DPIXEL`, versão própria em `ph2d_pixel_core::codec`); os braços de leitura 193/192/191/190/189 acrescentam zeros (postcard posicional); migrações v95/v128 preenchem `Vec::new()`. Tripla `(194, 13, 23)` em `project_schema_tests.rs` (a tripla NÃO vê este degrau). Componente `PixelSprite` registado por nome (`ph2d::ecs::PixelSprite`) em `ph2d-ecs` `scene/registry.rs`. Gate: `project_pixel_tests`.
⚠️ **O degrau é NÚMERO QUE SOMA entre linhas**: se outra linha integrada antes também subiu o schema ou os registos de componentes, reconte (`python3 scripts/schema-recount.py`; registos nos três sítios) e renumere o do Pixel para o seguinte (o comentário `# 193 → 194` e o braço de leitura acompanham).

## 6. Registos (também somam)

- `ph2d-panel-registry-init`: `EXPECTED_TYPED` +2 (`PixelToolsPanel`, `PixelTimelinePanel`), `quantas_entradas_tem_cada_painel` +1 linha (`pixel_timeline`), teste `paineis_armados`.
- `ph2d-app-registry-init`: `FAMILY` do Pixel (feature `app-pixel`); `every_object_mode_has_a_composed_family` 5 → 6 famílias e par `(K::PixelSprite, ObjectMode::Draw)`.
- Shell: features `panel-pixel-tools`/`panel-pixel-timeline` no `default`; `PANEL_Z_ORDER_FALLBACK`, `screens/hero/panel_ids.rs` (+11), `widget/scrollbar_ids.rs`: `PIXEL_SCROLLBAR_ID = NodeId(852)`, `PIXEL_TIMELINE_SCROLLBAR_ID = NodeId(853)`.
- Greps do integrador para detectar mesmo-símbolo: `NodeId(852)`, `NodeId(853)`, `NodeId(974..977)`, `PROJECT_SCHEMA`, `register_default::<crate::PixelSprite>`.

## 7. O fecho (MEDIDO, 28/10)

- `fmt` ✓ · `clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` ✓.
- `nextest-impacted` (`PH2D_SUITE=fecho`, base `c3662cd8e4`): **19 881 testes — 19 862 verdes, 9 falhas + 10 timeouts**.
  - Das 9, **5 eram determinísticas desta linha**, curadas no commit `0496642618` e re-corridas por nome, verdes: `arch_safe_clamp_only`, `arch_color_space_typed`, `node_id_collisions::every_non_literal_hash_is_named`, `hr12_widgets_a11y`, `toast_reading::census`.
  - As outras 4 falhas + 10 timeouts são de CARGA (loadavg 397/313/226 durante a corrida): `ph2d-timeline` `motion_path_perf`, `ph2d-tool-painter` `mask_tests` e `o_deposito_de_um_pincel_pesado…`, `ph2d-app-board` `fac_live`, timeouts em `ph2d-tool-painter`/`ph2d-panel-painter-layers`. Todas verdes isoladas (a do pincel pesado: 1 de 3 a load ~200, verde a load 21). **Zero ficheiros do diff nessas crates.** Ver [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md).
- `file_loc_caps`, `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`, arch-gates da shell ✓; `censos-da-arvore-combinada.sh` 127/127.
- **Auditoria com 2 lentes (commit `0496642618`)**. Lente A: (1) gesto pendente comia todo Up/Move da app (clique do chrome, pan do botão do meio) → `PixelApp.pressed`; (2) Save MSK com origem fora de i16 dava panic → `write_msk` devolve `Result` (`OutOfRange`); (3) `.msk` sem bloco alocava a caixa do cabeçalho (4,3 G) → máscara 0×0; (4) abrir projecto assentava a moldura solta do anterior no novo → `install` limpa; (5) W/H 65535 alocava ~20 GB → `edit::transform_within` corta à tela; (6) Vector escondido mas activo → `vector_tool_active`; (7) documento órfão em spawn falhado. Lente B: `pixel.hint.axis` sem leitor → `on_axis`; toast do braço 193 citava 191. Testes: `crates/ph2d-app-pixel/tests/it/fecho.rs` (4), `msk` `OutOfRange`, dica do eixo. **Mutações 5/6 sangram**: a guarda `was_editing` é redundante com `solta = None`; a mutação do tecto da moldura alocou 16 GB até o timeout (confirma o defeito).

## 8. Oráculos (números dos handoffs P6/P6g)

Elipse da selecção **715/715** caixas · bilinear do Sprite Size **8463/8463** canais · rotação/inclinação Fast **76/86** célula a célula (resíduo DECLARADO: 36 células de empate em 10 casos) · **RotSprite NÃO resolvido** (lacuna por caso em `ROTSPRITE_GAP`, 8–264 células) · MSK byte a byte contra o LibreSprite 1.3 em **45** casos (`docs/PixelArt/ferramentas/oraculo_msk.py`, fixture 839 KB). Aseprite Trial não salva nem corre scripts: polígono/Contour/Spray sem oráculo executável.

## 9. Smokes (para o Enio, passos em cada handoff de onda)

Cenas `PH2D_PIXEL_SMOKE=1..15` (máximo em `crates/ph2d-app-pixel/src/smoke.rs`; a 15 é a do P6g: inclinar pelo gizmo, flyout 1/2 linhas, MSK).
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=<n> ./target/smoke/ph2d-host-desktop`
2. No main, a partir da raiz: `cargo build --profile smoke -p ph2d-host-desktop` e o mesmo env.
Fotos de cada cena em `docs/PixelArt/handoffs/smoke_*.png`.

## 10. ABERTO

- **Smoke do dono da cena 15 (P6g).**
- Pedidos sem controlo: `ConstantFrameRate`, `TagRange`, `LayerContinuous` (`crates/ph2d-app-pixel/src/quadros.rs:147/166/175`; dívida declarada desde o P4). Selecção de intervalos na timeline; P5 (subtags, grupos de tags, velocidades, Preview flutuante).
- RotSprite fora de 90° não casa o Trial; as 36 células de empate do Fast; elipse com lados > 150×96 não medida e a tabela `{4,8,12,22}` sem explicação; foto da cena 9 com a lei Fast nova por tirar; tecto do Modify (100) e arredondamento %→px não medidos no Trial; `a_altura_de_abertura_de_um_painel_so_encolhe` com vizinhos.
- Etiqueta do Pixel não segue `LengthDisplay` (mede em células); gravar com forma viva não a assenta (P6d §6, ver P6f/P6g se ainda vale).
- Paletas: arrastar-reordenar, Tint/Shade/Tone, Spectrum, rodas, presets (P2 §7); `.ase` import/export = P9; OKLCH/cycling = P13; GPU dos 19 modos de mistura por medir (P3 §6).
- Mutações da LINHA ainda por correr: as listadas em §6 de P6b..P6g (a auditoria correu 6, 5 sangram).
- Licença do Aseprite: recusada pelo dono; oráculos abertos (LibreSprite, ImageMagick, Pillow, GIMP).

## 11. Instruções ao integrador

1. `git rebase main` com o `main` que já tiver GameEngineFuncional/BlenderConstraints/UIUX/TrelloClone (integram ANTES). Confira marcadores `|||||||`.
2. Recontar schema (`python3 scripts/schema-recount.py`) e registos de componentes (ecs/render/script); renumerar o degrau 194 se preciso; reconferir `NodeId(852/853/974..977)` contra os ids das outras linhas e `0192` contra os ADR.
3. `bash $ABS/scripts/collision-surface.sh` em cada worktree (caminho ABSOLUTO); a coluna `base:` é o merge-base.
4. `bash scripts/censos-da-arvore-combinada.sh`, `cargo machete`, `scripts/check-standalone-optional.sh`.
5. A suíte completa ANTES do merge (é do integrador). Atenção a testes posicionais de flyouts (`flyout_cols`, §3) em Vector/Quadro.
6. Smoke e push só por ordem do Enio.
