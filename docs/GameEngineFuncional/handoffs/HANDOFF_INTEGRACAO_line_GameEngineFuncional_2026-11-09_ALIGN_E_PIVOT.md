# HANDOFF de integração — `line/GameEngineFuncional`, 09/11 (GEF 27 · 28 · 29: Pivot por folha, Align, CellOffsets, Pivot no editor de animações)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, HEAD `e8a44ba528`, **32 commits** sobre `f254e42bf1` = merge-base = ponta do `main` (conferido 09/11; `--ff-only` serve). Nada integrado, nenhum push. **Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram** (regra do dono 23/10): são do integrador. `git diff --shortstat main..HEAD`: 177 files, +7 831 −708. O handoff anterior ([08/11, várias folhas](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-08_VARIAS_FOLHAS.md)) JÁ está no `main`.

## 1. Ondas (mecanismo nos docs linkados)

| Onda | O que | Docs |
|---|---|---|
| GEF 27 | Pivot de qualquer folha mantém o desenho · Ctrl = encaixe · Trim/Padding/Make Square por quadro · imagem escolhida substitui a folha à vista · Edit mostra a folha inteira · animação nova nasce vazia · toast `ShownSwap` · canvas usa o quadro | [27](../27_ferramentas_na_folha_visivel.md) |
| fix hierarquia | um clique (`605f6c4261`) | — |
| GEF 28 A–D | `CellOffsets` (deslocamento por célula) + Scale/Move sheet · **Align** no editor de animações (4 quinas, silhueta de referência, conta-gotas, quadro próprio, By feet / By centre) · meio texel do átlas (`uv_exact`) · filtro de imagem do app nos painéis · cena 5 «Misaligned Sheets» | [28](../28_alinhar_folhas_estudo.md) · [A](HANDOFF_CONTINUACAO_2026-11-09_ALIGN_ONDA_A.md) · [B](HANDOFF_CONTINUACAO_2026-11-09_ALIGN_ONDA_B.md) · [C+D](HANDOFF_CONTINUACAO_2026-11-09_ALIGN_ONDAS_C_D.md) |
| GEF 29 | Pivot do Animated Sprite mora no editor: grade 5×5 sobre a união de todos os quadros de todas as animações, By hand, **Follow** ao vivo (adiado ao largar numa escala pela quina), subquadro. Campo Pivot do Inspector = botão «Edit in Animations» (`AnimFieldEdit::OpenEditorAlign`); a ferramenta Pivot do canvas recusa um Animated com toast | [29](../29_pivot_no_editor_pedido.md) |
| fecho `e8a44ba528` | auditoria de 2 lentes: 11 achados curados (lista: `git show -s e8a44ba528`) + 3 gates de mutação | — |

## 2. Foundational tocado (`git diff --stat main..HEAD -- crates/ph2d-ecs crates/ph2d-render crates/ph2d-editor-core`: 33 files, +690 −217)

| Onde | O que | Natureza |
|---|---|---|
| `ph2d-ecs` | componente `CellOffsets` + `ExtraSheet.offsets` (registado); `SpriteAnimFrames` `Some(vec![])` = «toca nada» | aditivo; **muda semântica** só de um valor que antes não existia |
| `ph2d-editor-core` | `ImageUseEdit::{ScaleSheet, MoveSheet, NudgeCell, MovePivot}` (+ `ShowSheet` e afins do GEF 26–27), `AnimFieldEdit::OpenEditorAlign`, `set_ui_image_filter`/`ui_image_quality` (atómicos) | aditivo |
| `ph2d-editor-core` | **`WidgetEvent::LongPress` REMOVIDO** (sem consumidor) | **NÃO aditivo**: linha que o use não compila |
| `ph2d-render` | `AtlasRegion::uv_exact`, `TextureAtlas::region_uv_exact`, `sprite::sprite_cell_uv` (escolha de UV e meio texel saem da shell) | aditivo; comportamento muda em 2 casos de borda: o meio texel do `filter_clip` mede-se na textura AMOSTRADA (antes a de `spr.source`; só difere com imagem de script sobre átlas ou chave da lei sobre Individual) |
| outras | `ph2d-anim-sheets` (a única `Cargo.toml` tocada, sem crate nova), `ph2d-anim-editor`, `ph2d-panel-inspector`, `ph2d-app-sprite-anim`, `ph2d-i18n`, `shells/desktop` (22 files, +614 −129; ver §5) | consumidores / composição |

## 3. Contratos, schema, registo

**§6 intocado.** Prova (re-corrida 09/11): `git diff -U0 main..HEAD -- '*.rs' | grep -E '^[+-].*\b(NodeOp|OpResolver|NodeManifest|RasterEditTool|CanvasPaintTool|PanelEvent|VectorOp|ColorProfile|trait Tool\b|impl Tool\b)'` → **vazio** (0 linhas). `node.rs`/`tool.rs` intocados no `collision-surface`; gates `architecture_contract_surface` / tool / vector / imageio verdes. Sem ADR (próximo livre 0200).

**`PROJECT_SCHEMA` 225 (`main`) → 226: +1**, nos três sítios (escada, constante em `shells/desktop/src/project_schema.rs`, tripla `(226, 13, 29)` em `project_schema_tests.rs`). VEC 29, FLIP 13, timeline 25 inalterados. **Reconte pelo `main` do dia**: `python3 scripts/schema-recount.py`.

**Registo** (`ph2d-ecs`, `registry_tests.rs`): `main` 119 → linha **120** (DELTA +1, `CellOffsets`). Espelhos de `ph2d-render`/`ph2d-script`: a sonda do `collision-surface.sh` está CEGA (✗); confira à mão.

## 4. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional/scripts/collision-surface.sh`, 09/11)

```
merge-base f254e42bf1 · 32 commit(s) · 177 arquivo(s)
PROJECT_SCHEMA 226 (base: 225) · tripla (226, 13, 29) (base: (225, 13, 29)) · VEC 29=29 · FLIP 13=13 · DOC 25=25
REGISTRO ph2d-ecs 120 (base: 119) · espelhos render/script: SONDA CEGA
CONTRATO node.rs / tool.rs: intocado · ADR: nenhum
Cargo.lock: nenhum '+name' novo · marcadores de conflito: nenhum
LOC: nenhum arquivo da linha passa do teto
```
Rode-a pelo caminho ABSOLUTO em cada worktree; `base:` é o merge-base. Arquivos quentes: `shells/desktop/src/project_schema{,_tests}.rs` · `crates/ph2d-ecs/src/scene/registry{,_tests}.rs` · `crates/ph2d-editor-core` (`WidgetEvent`, `AnimFieldEdit`, `ImageUseEdit`) · `crates/ph2d-panel-inspector/src/cards_sprite*.rs` · `crates/ph2d-i18n` · `CLAUDE.md` §5 (Game Engine) · `.config/testes-lentos.tsv` (regenerar).

## 5. Medido e provado (fecho, em `e8a44ba528`)

- **fmt** limpo; **clippy `--workspace --all-targets -D warnings`** limpo; depois da última cura, de novo nas crates tocadas: limpo.
- **LOC:** `file_loc_caps` 14/14 em `ph2d-host-desktop` (it); `workspace_src_files_under_loc_cap` verde após cortes (`align.rs` 708→640 via `silhueta.rs`; `component_catalog.rs` 701→700); `the_shell_only_shrinks` verde. **Shell cresceu:** `git diff --shortstat main..HEAD -- shells/desktop/src` = 22 files, +614 −129; catraca verde (ver ABERTO 1).
- **Testes por nome:** `project_schema` 39/39; `game-scenes` `editor_animacoes` 85/85; (`editor_animacoes`+`sprite_`+`folhas`+`animado`) 185/185 antes das últimas curas; `ph2d-panel-inspector` 103/103; `ph2d-sprite-edit` pivot 31/31; `ph2d-anim-sheets` 34/34; `ph2d-render` 134 lib + 25 it; hierarquia 3/3.
- **Mutação:** 12 mutações, 9 sangraram à 1.ª; os 3 sobreviventes ganharam gate, cada um visto vermelho sob a mutação: `o_follow_guarda_o_ponto_exato_e_nao_a_medida_torta`, `a_ferramenta_pivot_recusa_na_animada_e_age_na_parada`, `the_extract_cuts_every_atlas_sheet_frame_on_its_own_texels`.
- **Janela real:** Follow + arrasto da folha e 1 Ctrl+Z devolvem folha e âncoras exatas (`[animado-prova] pivô depois da escala` == `pivô 1 Ctrl+Z`). A sonda achou deriva 0,5096 vs 0,5 → cura: guardar a fração exata da grade.

## 6. Riscos de integração

- `WidgetEvent::LongPress` removido: `git grep -n LongPress` nas outras linhas antes de fundir.
- Schema e registo somam entre linhas: reconte (`schema-recount.py`; `registry_tests.rs` à mão para os espelhos).
- `PH2D_GAME_SMOKE=5` mudou («Misaligned Sheets» na fila de cima): fotos antigas da cena 5 não valem.
- O ficheiro `docs/GameEngineFuncional/fotos_29/animado_pivot_segue.{png,log}` é regravado a cada foto da cena (ruído esperado).

## 7. SMOKE do dono

Antes: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build -p ph2d-host-desktop --profile smoke`.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`. Clique um Animated Sprite; no cartão **Animation** o campo **Pivot** é um botão **Edit in Animations**. Errado: campo numérico solto.
2. No editor de animações: grade 5×5 do Pivot; **By hand**; **Follow** liga e o Pivot acompanha o arrasto da folha; **Ctrl+Z** devolve folha e Pivot de uma vez.
3. **Align**: quatro quinas, silhueta de referência, conta-gotas, **By feet** / **By centre**; a cena «Misaligned Sheets» fica alinhada e **Ctrl+Z** desfaz.
4. Com a ferramenta **Pivot** do canvas num Animated: aparece um aviso e nada anda (numa imagem parada, age).

Fotos (uma por vez): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE_FOTO=<nome> bash docs/Components/ferramentas/fotografa_cena.sh PH2D_GAME_SMOKE=5 1930 1040 <out.png> 10`. Nomes GEF 29: `animado_pivot`, `animado_pivot_pes`, `animado_pivot_desfaz`, `animado_pivot_mao`, `animado_pivot_segue`, `animado_pivot_segue_desfaz`, `animado_folhas_pivo` (toast da recusa; espera `2`). GEF 28: `animado_align`, `animado_align_escala`, `animado_align_desfaz`, `animado_align_pes`, `animado_align_pes_desfaz`, `animado_desalinhadas`, `animado_desalinhadas_acertadas`, `animado_idle_q<k>`; filtro: `PH2D_GAME_SMOKE_FILTRO=pixel`. Fotos em `docs/GameEngineFuncional/fotos_28/` e `fotos_29/` (maquete `maquete_pivot_25.png`). **Smoke do dono:** GEF 28 tudo OK; GEF 29 Pivot/Follow aprovado por iteração em 09/11 («ótimo»). As curas do fecho (`e8a44ba528`) têm teste, não foram vistas por ele.

## 8. ABERTO

1. **Shell:** os drivers de smoke em `shells/desktop/src/janela_do_jogo_animado.rs` (`alinha_do_animado`, `pivo_do_animado`, `relato_do_pivo`, `Mutex` estático) são encanamento de teste da família; devem MOVER para uma crate da família (catraca verde, mas é crescimento).
2. Depois de Ctrl+Z, `ShownSheet` volta a 0 (o editor mostra «idle»).
3. O slot «INSERT HERE» cobre a borda direita do último quadro.
4. «Delete empty animations» mantém animações de 0 quadros (deliberado). O «+» do cartão Animation não seleciona a nova.
5. `ph2d-app-macros tutorial_painter::outra_tela_diz_o_tamanho_gravado`: estouro de pilha, alheio à linha; conferir no `main`.
6. Decisões do dono: o botão Pivot do Inspector não mostra o nome do ponto (uma lei só, no editor); o rótulo do nome do ponto saiu; a 360 px os textos das abas elidem (geometria com gate).
7. O Follow adiado após escala pela quina cai no quadro de soltar; o agrupamento do undo foi medido no arrasto da folha (agrupa), não à parte na escala.
8. Restam strings «Cmd+Z» em `shell_media.rs` (outros toasts), anteriores a esta linha.
9. Herdado de [27 §3](../27_ferramentas_na_folha_visivel.md): itens (3), (4) e (6) do handoff de 08/11 e a Fila ([23](../23_refundacao_plano.md)); `line/GameEngineFuncional02` é separada, a ordem é do integrador.

## ⛔ Recusas MEDIDAS

Pivot de um Animated no canvas (recusa com toast: o Pivot mora no editor) · rótulo com o nome do ponto (dono 09/11) · Follow reaplicado durante a escala pela quina (realimenta o gesto: medido, adia-se ao largar) · limiar próprio do Follow (usa o `world_moves` de 1e-3 m do `move_origin`).
