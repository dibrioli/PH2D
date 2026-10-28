# Handoff de continuação — GEF 16: o 4.º uso da imagem, **Region** (27/10)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · base `c3662cd8e4` (o main não
andou). Plano: [`16_uso_region_plano.md`](../16_uso_region_plano.md). Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-26_15_USO_A_VISTA.md`](HANDOFF_CONTINUACAO_2026-10-26_15_USO_A_VISTA.md).

## Estado

**Smoke do dono (onda 16): POR FAZER.** Commits: `9852e2edc6` (D1, o uso) · `26dc1fbf34` (D2, o editor, o
condutor, as fotos) · o fecho (mutação, clippy, este handoff). Fotos do app real em [`fotos_16/`](fotos_16/).

## Medido antes de decidir (D1, §5.0)

O plano pedia medir se `UvTransform` sozinho exprime o recorte. **Exprime no `sprite.wgsl`** (`wrap(quad_uv ·
scale + offset)`, clamp), mas três leitores não o seguem:

| leitor | com `UvTransform` | com `SpriteRegion` |
|---|---|---|
| captura da Luz 2D (`ph2d-app-light2d/src/captura.rs:56`) | salta toda sprite com `uv_xform ≠ identidade` | estreita `atlas_uv`: segue |
| borda do recorte | clamp ao rect da IMAGEM: o bilinear puxa a célula vizinha | `filter_clip` (meio texel) |
| ferramentas de píxeis (`texture_edit::crop_region`), retrato, pele, script, `sheet_bake` | não o leem | leem |

⇒ **a área vive no `SpriteRegion`** (px da imagem, a lei que já tem 6 leitores); o componente novo é só o
**marcador do uso + a grelha de snap**: `ph2d_ecs::ImageRegion { grid: [u32; 2] }`. ⛔ Recusado: `ImageRegion {
rect_px }` com o `UvTransform` derivado (duas verdades do retângulo e a Luz 2D cega).

## O que entrou

- **ecs:** `ImageRegion` registado (ecs `112 → 113`) ⇒ **`PROJECT_SCHEMA 196 → 197`** (`PROJECT_SCHEMA_SEM_REGION
  = 196`, lido com o tipo vivo; tripla `(197, 13, 23)`). `ImageUseMemory.region: Option<RegionLook { size, area:
  Option<SpriteRegion>, cut: ImageRegion }>` no FIM, com `Deserialize` próprio: o blob de 3 lugares (o 196) lê-se —
  o postcard responde ERRO ao fim dos bytes, não `None`, e o `#[serde(default)]` derivado propaga o erro.
  Catálogo `image` (machinery), porta das macros `Gravado`, i18n.
- **Lei (`image_use.rs`):** `ImageUse::Region`, lido ANTES de Repeated. Region nova = a imagem INTEIRA à
  `display_scale` (sem área), grelha do corte da imagem ou da proposta das calhas (sem proposta 1 × 1), e
  **pede o editor** (`RegionEditorRequests`); voltar a uma Region guardada não o pede. `set_region` (o OK): a área
  à MESMA escala (`size = área/px × native × k`), dentro da imagem, ≥ 1 px, collider automático segue.
  `display_scale` conta a fracção da área. **Sair de Region tira a área DEPOIS de o uso novo medir a escala.**
  `ImageFacts.px` (a fábrica: `image_px()`; importada: `w × h`; pintura: `0`). `choose_image` numa Region: a
  escala conta a área, a área da imagem antiga vai-se, o marcador fica.
- **editor-core:** `InspectorImageUse::Region` (`ALL` = 4), `InspectorDraw::Region { px }`,
  `ImageUseEdit::{EditRegion, SetRegion { rect, grid }}`.
- **Painel:** o seletor com 4 (o `SegmentedAdaptive` reflui em 2 × 2 sozinho — foto), «Edit Region…»
  (`INSP_IMAGE_EDIT_REGION`) e «Area: W × H px» (px da IMAGEM: o herói de fábrica é 1536 × 768, células de 192).
- **O editor (`ph2d-app-sprite-anim/src/region/`):** família `RegionDocs` irmã do editor de animações; `frame` é
  chamado de dentro do `editor::frame` (o ppm vem do `HeroScreen`) e `editor::install` instala as duas famílias
  — a shell não cresce por isto. Barra: Columns · Rows · a ajuda · Cancel · OK. Tela: xadrez, imagem, grelha
  (`Text2`), fora da área escurecido (`BgScrim`), área (`Accent`) e 8 alças. Gestos: arrastar dentro move,
  alças redimensionam, arrastar fora desenha, **um clique sem arrastar escolhe a célula** (dentro ou fora da
  área), roda = zoom no ponteiro, botão do meio / espaço = arrasto. Snap às linhas; `Alt` e 1 × 1 ao píxel.
  A grelha vai de 1 ao lado da imagem em px (uma célula tem ≥ 1 px — o tecto é o da imagem, não um número).
  OK (botão, Enter) = `SetRegion` num quadro; Cancel / `Esc` nada; a aba fecha também se a sprite sai de Region.
- **Condutor `PH2D_GAME_SMOKE_USOS`:** passos `region`, `cell:C:L`, `ok` (`PassoDoDono`); a vista centra o Player
  (`CENTRO_DOS_USOS`); o estado do Player no `stderr` mostra a área.

## Defeitos que as fotos apanharam (nenhum gate os via)

1. **O clique numa célula perdia-se** com a área = imagem inteira (tudo é «dentro» ⇒ virava mover). Cura: o
   mover sem mexer a área é um clique. Gate `um_clique_escolhe_a_celula` (caso «dentro da área inteira»).
2. **Region (célula) → Still dava a folha a 1 × 1 m:** a área saía antes de o uso novo medir a escala. Gate
   `sair_de_uma_area_nao_muda_a_escala` visto VERMELHO (`[1,1] → [0.125, 0.25]`), verde com a cura.
3. A grelha em `Border` não se via sobre o xadrez ⇒ `Text2`.

## Gates (por nome)

- `ph2d-app-sprite-anim`: `cada_troca_ida_e_volta_devolve_o_uso_como_estava` (12 trocas, 4 usos, 2 voltas, a
  área dada à 1.ª Region, desfazer e reaberto a meio) · `nenhuma_troca_de_uso_muda_a_escala` (os 4 usos, 5
  objectos × 64 sequências; Region nova sem área nem corte; fora de Region sem marcador nem área) ·
  `image_use_region_tests` (7: célula à mesma escala, render amostra só a área — `region_subrect` →
  `[0.375, 0.25, 0.5, 0.5]` —, nunca fora, editor só em Region, trocar a imagem esquece a área, collider,
  sair de uma área) · `region::tests` (6 de gestos + `ok_aplica_num_quadro_cancel_nao_muda_nada` pelas portas
  reais da aba e o `frame`).
- `ph2d-ecs`: `a_memoria_de_tres_lugares_le_se_sem_region`, `a_region_guardada_volta_pelo_postcard_e_pelo_json`,
  `register_ecs_components_populates_registry` (113).
- `ph2d-panel-inspector`: `seam_image_use` (7, com `edit_region_pede_o_editor`).
- shell: `every_registered_component_*` + `todo_componente_registado_tem_porta_no_mundo` (6),
  `a_schema_bump_anywhere_must_bump_the_project_schema`; `ph2d-i18n` por nome (4); `ph2d-component-desc catalog` (18).

## Mutação, clippy, fmt e censos (fecho da onda)

**Mutação 9/9 sangraram** (agente `mutacao`, árvore reposta byte a byte): M1 `snap` ignora a grelha →
`a_area_desenhada_encaixa…` · M2 o mover-sem-mexer não é clique → `um_clique_escolhe_a_celula` +
`ok_aplica_num_quadro…` · M3 a Region deixada não se guarda → `cada_troca_ida_e_volta…` +
`o_editor_pede_se_so_em_region` · M4 o OK sem `apply_edit` → `ok_aplica_num_quadro…` · M5 a leitura de 3 lugares
propaga o erro → `a_memoria_de_tres_lugares…` · M6 a área sai antes de medir → `sair_de_uma_area…` · M7
`display_scale` sem a fracção → `a_area_escolhida…` + `sair_de_uma_area…` · M8 `clamp_area` mudo →
`a_area_nunca_sai_da_imagem` · M9 o braço «Edit Region» apagado → `edit_region_pede_o_editor`.
**clippy** `PH2D_SUITE=fecho --all-targets -D warnings` limpo nas 9 crates tocadas (cura: `RegionSeed`); **fmt**
limpo; censos `architecture_the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`,
`architecture_panel_loc_cap`, `no_literal_color`, `every_painted_id_is_reachable`, `o_inspector_armado`, app-game
`smoke` (15) verdes. `no_magic_numeric` estava VERMELHO por 19 sítios DESTA linha (a ilustração da galeria,
`a1ab91b4cb`, onda 07–08; `MIN_COPIAS`, `e1186b906a`, onda 15) — anotados `LITERAL-PX-OK` (coordenadas 0..1 da
moldura; cópias, não px); verde.

## Smoke (para o Enio)

Ver a resposta da janela (passos numerados). Fotos: `nivel_4_editor_aberto` (Columns 8 · Rows 4 sobre a folha
inteira) · `nivel_4_editor_celula` (a célula 1:0 escolhida) · `nivel_4_cena_depois_do_ok` (um boneco só, 100 × 100
px, «Area: 192 × 192 px») · `nivel_4_volta_still_region` (Still → Region devolve a mesma célula).

## ABERTO

- Smoke do dono da onda 16.
- «Area» conta px da IMAGEM (1536 × 768 no herói) e o cabeçalho do Inspector conta px do OBJECTO (800 × 400):
  números diferentes na mesma vista — decisão de produto se confundir.
- O editor não tem desfazer próprio (o Ctrl+Z é do projecto e só vê o OK); um gesto errado dentro da aba
  corrige-se com outro gesto ou Cancel.
- Os ABERTOS do handoff 15 continuam (Animation aberta/recolhida, numeração 0/1, bloco antigo sem Width/Height,
  Add «Static Body» sem Tiled, vermelhos herdados).

## PRÓXIMO

O smoke do dono decide. Depois: 12b ou J5 moedas ([plano 03](../03_plano_jogo_de_plataforma.md)), conforme o dono.
