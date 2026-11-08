# Handoff de continuação — Add Object na versão B (07/11)

**Estado: smoke do dono OK (07/11).** Commits na `line/GameEngineFuncional`: `68bcc35b68` (gesto, faixa,
grupos, palavras, ensino, Recent, dicas), `7d175f3d40` (estreito e no acabamento da referência), `1052637aad`
(import morto). Plano: [24_add_object_plano.md](../24_add_object_plano.md). Decisões do dono:
[memória](../../../project-memory/project_add_object_owner_decisions_2026_11_07.md). Não reconstruir.

## O que existe

- **Gesto opt-in** `PaletteModel.select: Option<PaletteSelect>` (`widget/command_palette/selecao.rs`): 1 clique
  destaca, `DoubleClick`/Enter escolhe e fecha, ↑↓ andam pelos visíveis (`interaction/state/palette_select_ops.rs`,
  shell `keyboard_palette.rs`); destaque inicial = Sprite. Add Component, Ctrl+K e biblioteca do Motion: `select: None`,
  inalterados.
- **Faixa da dica** no fundo, à altura da dica MAIS alta (`dock_height`); some quando a busca não acha; o hover não
  pinta dica em modo de seleção.
- **Aspeto compacto** só em modo de seleção: `command_palette/look.rs` (820 px, 3 colunas em alvenaria, contagem à
  direita, filete de 1 px, pílula 22 px, lupa `IconId::Search`); cartão de grupo em `command_palette/grupo.rs`.
- **Catálogo**: `AddGroup {Pictures, Drawing, Game, Empty}`, ordem por `AddEntry::at`; 19 entradas; Physics Body e
  Ambient Light saíram. Entradas e `TEACH` do jogo em `ph2d-game-dicas::add` (catraca de LOC da `ph2d-app-game`).
- **Palavras de busca** `<chave>.keywords` (`ph2d-i18n/src/object_add.rs`, `AddEntry::keywords`); **ensino** da busca
  sem resultado (8 casos) com botão «Start with the …» (`CMD_PALETTE_TEACH_START`).
- **Recent** (3) em `~/.ph2d/sections.txt`, linha `add_recent=` — preferência do editor, NÃO no `.ph2dproj`.
- Fotos: `PH2D_OBJECT_ADD_SMOKE=1` + `PH2D_OBJECT_ADD_HOVER=<chave>` (destaca) / `PH2D_OBJECT_ADD_QUERY=<texto>`.

## Provas

Verdes por nome: editor-core `command_palette` (34), `object_add` (6), `architecture_widget_loc_cap`,
`the_shell_only_shrinks`; desktop `fase_object_add` (13), `the_app_never_reshapes_a_still_screen`,
`um_modal_aberto_tem_o_teclado`; components `component_palette`, `every_game_object`; light2d
`o_menu_add_cria_a_luz_com_o_componente`; registry-init `a_paleta_fecha_ao_escolher_por_cima_de_um_painel`;
game-scenes `jogo_object_add`; i18n gates de chave.
**Mutações:** as 3 do aspeto sangraram (`look_tests.rs`). ⚠️ As do 1.º commit (destaque inicial, duplo clique,
opt-in, altura reservada, filtro de palavras, corte de 3 do Recent) **NÃO foram corridas** — devidas no fecho.

## Aberto

- **Próximo item (dono, 07/11): o Add Component no MESMO padrão, com dicas** (Crítica `#addcomponent`; hoje
  `tips: Vec::new()` em `ph2d-app-components/src/component_palette.rs`; 63 listas de palavras já existem).
- A decidir pelo dono: peso das letras (o modo `CrispHeavy` engrossa ≤ 12 px no app inteiro) e o destaque
  cheio × só contorno da maquete.
- A linha ainda deve o fecho completo (`/pd-linha-fechar`): mutações acima, rodada anterior sem handoff
  (aba Scene etc.), link do CLAUDE.md §5.
