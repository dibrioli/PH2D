# HANDOFF — line/PainterWatercolor, a BIBLIOTECA de pincéis (23/10)

> Prompt: [`smoke_2026-10-23/PROMPT_NOVA_JANELA_2.md`](../smoke_2026-10-23/PROMPT_NOVA_JANELA_2.md). Linha NÃO integrada.
> Anterior: [PINCÉIS, no main 23/10](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md).

## §1 — O que entrega (uma família: a lista e o ficheiro do preset)

| Pedido | Causa / desenho | Onde |
|---|---|---|
| **A ordem muda sozinha** | MEDIDO: a vista ordenava `(categoria, media != meio de agora)` — escolher o «Watercolor Basic» (meio Aquarela) punha-o no topo da categoria Watercolor e o «Wet Round» (Wet Paint) mandava-o para o fim. A biblioteca em si (`Biblioteca::carregar`, `BTreeMap` + `ordem_do_navegador`) já era determinística. **Cura:** o «meio de agora primeiro» sai; UMA porta de ordem, `Presets::ordem_do_navegador`. | `painter_bridge_presets_biblioteca.rs` |
| **Ordem manual** | O artista arrasta no popover aberto; a ordem vive em `estado.json` (`EstadoDoUtilizador.ordem`, caminhos relativos), ao lado dos favoritos: é dado do ARTISTA sobre a biblioteca, chaveado pelo caminho estável, gravado no mesmo `rename` atómico; não vai no ficheiro do preset (um preset partilhado não leva a posição de quem o tinha). `mesclar_ordem`: o que a manual conhece na ordem dela; o desconhecido (novo, importado, de fábrica nova) entra logo depois do vizinho da ordem por omissão; o apagado cai — sem buracos nem duplicados. O arrasto não sai da secção (Favoritos / categoria). «Reset positions» esvazia a lista. | `ph2d-brush-presets/src/ordem.rs`, `estado.rs` |
| Arrasto | 3.ª família da máquina de linhas do despacho, `PanelRowFamily::PainterPreset` (append-only); `aninha() == false` ⇒ `find_panel_row_drop` parte a linha ao meio (Before/After, sem Inside). O painel publica as faixas VISÍVEIS do popover (`visible_option_bands`) e desenha a barra da queda; a queda vai à ponte por `SelectOption(PAINTER_WETPAINT_PRESET_MOVE, "a␟b␟before|after")`. O popover fica aberto. | `ph2d-editor-core/.../event.rs`, `dispatch/hierarchy.rs`, `ph2d-panel-painter-layers/src/paint_wetpaint_presets_biblioteca.rs` |
| **Apagar** | Só `Origem::Utilizador && !tem_fabrica` (um sobreposto sai pelo Reset, nunca pelo Delete). **Confirmação numa JANELA** (dono, depois do 1.º smoke: «melhor um modal»; era o 2.º clique no botão): o `Modal` da fundação (até aqui sem chamador de produção; ganhou `button_rects`, a mesma conta que o `paint_modal` usa), título com o nome, o corpo diz que não se desfaz; só o «Delete» dela chega ao tool (e só se o pincel dela ainda for o activo); Cancel, o X e o clique fora fecham. Ids `PAINTER_WETPAINT_PRESET_DELETE_DIALOG` (UI-only). Sem desfazer. `EstadoDoUtilizador::apagado` limpa lembrado, favorito, posição e activo de cada meio; o pincel continua na mão. | idem + `painter_bridge_presets_biblioteca.rs` |
| **Exportar / Importar** | Exportar = o ficheiro da BIBLIOTECA do preset activo (mexidas por guardar ficam de fora — «Save as Default» antes), mais `Parametros.library_folder` (a pasta do meio, que o caminho dizia e o ficheiro solto não). Importar (vários ficheiros): valida (`ler`), recusa meio desconhecido com aviso, grava em `<motor>/<library_folder>/<grupo>/<nome livre>` com `Origem::Importado` e sem o `library_folder`, aplica o último. Nomes de fora passam por `nome_seguro` (sem `/`, `\`, `..`). Diálogos `rfd` atrás de `Seletor` (os testes trocam-no). | `painter_bridge_presets_biblioteca.rs`, `modelo.rs` |
| **Censo das imagens** | Shape, Grain, 2.ª ponta (`dual_shape`), mangueira (N imagens): já viajavam. **Flow** da Ragged Edge («Use as Flow»): só o kind viajava — passa a `phTx` `flow` (captura e instala só na Aquarela, o único meio que o lê; noutro meio o `edge_flow` nem vem do ficheiro). **Paper fica fora por lei:** é da obra (`CAMPOS_FORA`, `DA_OBRA`; `set_paper_field` escreve-o em todos os slots). | `ph2d-tool-painter/src/tool/paint/wetpaint_preset.rs` |

UI: a fileira «Preset library» (Import… · Export… · Delete · Reset positions, cada um só quando serve), por baixo das acções. Textos novos em `ph2d-i18n` (`app.painter.presets.{deleted,delete_failed,exported,export_failed,imported,import_failed,import_unknown_media,file_filter}`, `panel.painter_layers.wetpaint.preset_{library,import,export,delete,delete_confirm,reset_order}`).

## §2 — Portões (por nome; regra do dono 23/10: sem suíte)

- `ph2d-app-painter` (`--features panel-painter-layers`), `painter_bridge_presets_biblioteca_tests`: `a_ordem_nao_muda_sozinha` · `a_ordem_manual_persiste_e_o_reset_a_devolve` · `apagar_nunca_toca_a_fabrica_e_o_pincel_fica_na_mao` · `exportar_e_importar_da_o_mesmo_pincel_e_a_mesma_pintura` (uma imagem em cada slot; campos ao bit, imagens ao byte, traço ao píxel; controlo: o de fábrica pinta diferente) · `importar_recusa_o_que_nao_serve_e_diz_porque`.
- `ph2d-panel-painter-layers` `seam_wet_presets`: `a_fileira_da_biblioteca_pede_o_que_diz` · `o_delete_abre_uma_janela_de_confirmacao_e_nunca_aparece_na_fabrica` (Cancel/X/fora não apagam e fecham; o Delete da janela apaga) · `arrastar_no_navegador_pede_a_ordem_nova` (frações 0,25/0,45/0,55/0,75).
- `ph2d-brush-presets` `ordem::tests` (3).
- Ajustados: `seam_wetpaint::every_doc22_wet_click_forwards_through_the_panel` (o Delete pede dois cliques sobre um pincel do artista); `o_navegador_abre_em_qualquer_meio_e_escolher_troca_o_meio` perde a lei revogada do «meio de agora primeiro».
- Vizinhos verdes por nome: `seam_estado_escondido`, `every_word` (painel e app), `censo_dos_controlos`, `architecture_panel_wiring_parity`, `architecture_panel_loc_cap`, `panel_row`, `every_widget_event_variant`, `hierarchy`, `architecture_tool_contract_surface`, `o_preset_nao_herda`, `wetpaint_preset`, `presets_de_fabrica`, `o_pincel`, `painter_bridge`, `o_formato`, `a_biblioteca`, gates do `ph2d-i18n`.
- Mutações: §3. Fotos (olhadas): [`fotos_biblioteca/`](../smoke_2026-10-23/fotos_biblioteca/) — `foto_da_biblioteca` (`#[ignore]`, `PH2D_FOTO_DIR`).

## §3 — Provas de mutação

13/13 sangram (controlos verdes: A 5 · B 38 · C 3; cópia + `cmp`, árvore restaurada):
M1 o «meio de agora primeiro» de volta → `a_ordem_nao_muda_sozinha` · M2 `ordem` com `#[serde(skip)]` → `a_ordem_manual_persiste…` · M3 o desconhecido no FIM → idem + `ordem::tests::o_novo_entra_ao_lado…` · M4 Reset no-op → `a_ordem_manual…` · M5 sem a guarda da fábrica → `apagar_nunca_toca_a_fabrica…` · M6 sem limpar o favorito → idem · M7 sem `library_folder` · M8 sem capturar o Flow · M9 sem instalar o Flow · M13 sem `Origem::Importado` → `exportar_e_importar…` · M10 linhas do popover não registadas · M12 `aninha() == true` → `arrastar_no_navegador…` · M11 Delete no 1.º clique → `o_delete_confirma…` + `every_doc22_wet_click…` (antes da janela); com a janela: o Cancel a apagar → `o_delete_abre_uma_janela…`.

## §3b — Depois do 1.º smoke (23/10): o Stars pequeno e a secção Jitter

- **Stars com degraus em tamanho pequeno** — MEDIDO (H1, minificação, não máscara ampliada): o Stars (Shape Image
  128², contorno claro de 2 texels) ia pelo caminho POR-PÍXEL (Accumulate off), que lia a imagem num ponto por
  píxel; a 10–40 px o contorno era apanhado em uns píxeis e não noutros. Erro contra a área exacta: 0,550/0,425 →
  0,017/0,051 (r 5/10). **Cura:** imagem de Shape reduzida lê-se SEMPRE pela área (`texture/shape.rs`); a máscara do
  cache bake por área e com pirâmide 2×2 (`StampMask::nivel_para`, nunca ampliada nem reduzida mais de 2:1) nos
  quatro leitores (`blit_stamp`, `stamp_ramped`, `blur`, `clone`); o MESMO no carimbo colorido (`ColorStampMask`,
  nível escolhido uma vez por dab nos lotes fundidos `stamp_color/accumulate*.rs`) e no Grain de imagem
  (`grao_de_imagem_pela_area`, `sample_unit_px`). `lei.area` passa a só forçar o caminho por-píxel. Miniatura do
  Stars regenerada (só ela mudou). Portões red-first: `a_estrela_pequena_nao_tem_degraus` (tool),
  `estrela_tests` (Shape, Grain) e `stamp_color::estrela_tests::o_carimbo_colorido_pequeno_nao_tem_degraus`;
  mutações (nível forçado ao topo; área = centro) sangram. Tolerâncias: `tip_kernel_tests` 0 → 1 px de orla
  (a borda parcialmente coberta; o transbordo de 13 px do bug original continua apanhado). ⚠️ Stencil não coberto.
- **Jitter** (dono: «Unit no topo, todos sliders, ordem inteligente»): a ordem é UMA tabela, `FILEIRAS_DO_JITTER`
  (`paint_stroke/jitter_linhas.rs`) — Unit · Count, Count jitter, Particle size · Position, Scatter · Spacing ·
  Scale, Roundness · Rotation, Stroke angle · Flip X, Flip Y · Opacity jitter (agrupadas pelo que sorteiam). As 8
  linhas da marca passam a barra + caixa (`PAINTER_MARCA_CHIPS`, `link_slider_number_mapped` com a faixa de
  `LinhaDaMarca::faixa()` — Scatter 0..2, Particle 0,05..1); o tool continua a receber o valor real. Portão
  `seam_marca::o_cartao_jitter_tem_o_unit_no_topo_e_so_barras` (red-first; arrasta e digita no Scatter); duas
  mutações sangram. Foto `smoke_2026-10-23/fotos_jitter/`.

## §4 — Aberto

- O arrasto entre secções (um favorito para o meio de uma categoria) é recusado em silêncio; o desenho do Procreate também não o faz.
- Exportar leva o ficheiro, não o pincel com mexidas por guardar — se o dono quiser o contrário, é trocar `exportar_para` para `preset_capturar` sobre a base do activo.
- Contrato `PanelEvent=4` intocado (o arrasto usa `SelectOption`).
