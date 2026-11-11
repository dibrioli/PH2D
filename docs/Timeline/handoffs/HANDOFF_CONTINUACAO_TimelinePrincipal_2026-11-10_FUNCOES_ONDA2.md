# Continuação — `line/TimelinePrincipal`, plano 17 (funções novas), onda 2 (itens 5–7 + cura do Opacity) — IMPLEMENTADA, NÃO FECHADA

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commits da onda sobre `5d5c0616ab`: `8acfdb5040` (fix do smoke da onda 1),
  `f52404c449` (parte 1), `0b7c8d058c` (parte 2), `b2438d11b1` (parte 3, HEAD antes deste handoff).
- Plano: [`17_plano_funcoes_novas.md`](../17_plano_funcoes_novas.md) — «Onda 2 — desenho medido» e «Onda 2 — respostas de produto» (dono 10/11 «escolha e siga»; ele pode vetar).
  Onda anterior: [`..._FUNCOES_ONDA1.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-10_FUNCOES_ONDA1.md).
- Estado: código escrito, **sem smoke do dono** e com testes por correr (lista abaixo). Maquetes: `../fotos/maquete_B1.png`, `_B2`, `_B4`, `_C3`, `_C4`.
  Foto já feita: `../fotos/27_chip_para_faixa.png` + `27_lado_a_lado_chip.png` (a refazer, ver A FAZER 4).

## O que a onda fez

### Cura do Opacity da Sprite (`f52404c449`, `0b7c8d058c`)
- Numa Sprite o canal Opacity conduz `Sprite::opacity` (o slider «Opacity (%)»); o alfa do Tint vai com o canal do Tint (`TINTA_FIM = TINTA + 4`).
  Saiu o canal OBJETO 11 «Sprite Opacity» (**nunca reusar** o número). `SpriteAlpha`/preview/K da shell passam a `opacity`.
- **`PROJECT_SCHEMA` 231 → 232, triplo (232, 13, 29)** — o integrador reconta (`python3 scripts/schema-recount.py`).
- `Sprite::cascaded_opacity` (`ph2d-render`): a opacidade do pai apaga a subárvore, como o tint; o extract da shell emite-a. Pixel a 0,5 na GPU (teste em
  `ph2d-render/tests/it/a_mistura_do_device_chega_ao_pixel.rs`). Costura Timeline → `Sprite::opacity`.
- Constraints: `Prop::Opacidade` pela porta única `write_opacity`/`read_opacity` (sprite e vetor); cenas de smoke 9 e 13 de volta à Opacity (na parte 1 tinham
  ido ao alfa do Tint); o cartão diz «Opacity».
- Portão `o_canal_diz_o_nome_do_campo_que_o_grava` (`nome_do_canal` público): Tint/Self Tint/Corner/Silhouette nomeados pelas chaves do Inspector
  («Silhouette (Tint Fill)», «Per-corner Tint»). Flip nomeado «Flip image H/V» pela chave do Inspector (`nome_do_cartao`; portão sem pendentes).
- `ph2d-app-components` ficou sob o teto (história verbatim em `docs/archive/integracao-jornadas/COMENTARIOS_CORTADOS_line_TimelinePrincipal_2026-11-10.md`).

### Item 5 — Keys | Curves nos cartões
- `TimelinePanelState.vista {Keys, Curves}` (sessão, não grava, não desfaz), ids `TIMELINE_VIEW_KEYS/_CURVES`, i18n `panel.timeline.view.{keys,curves}`.
  `crates/ph2d-panel-timeline/src/curvas_vista.rs`. Segmentado na barra só na aba Keys; grafo grande pelo `paint_track`; curva escolhida cheia, secundárias
  normalizadas 0–1 tracejadas no eixo direito (`TimelineTinta::CurvaSecundaria`); olho por linha esconde a curva; o twirl por linha da Keys FICA (resposta 1).
  Curves volta a Keys ao mudar de aba/clipe/abrir a lista larga; 0/1 por `format_number`; dica do olho. `PH2D_CARDS=0`: sem botão.
- Pisos OKLab 0,388 / 0,433. Testes: `curvas_vista_tests`, `tests/it/curves_view_seam.rs` (incl. `mudar_de_clipe_ou_abrir_a_lista_volta_a_keys`).

### Item 6 — chip → faixa
- `TimelineHitKind::SourceChip{opcao}` (variante nova, append-only, em `ph2d-editor-core` `interaction/types.rs`) no lugar do `Button`; clique parado continua a escolher.
  `chip_drag.rs`: fantasma tracejado na faixa sob o ponteiro, encaixe ao quadro, largada = `AddStrip` (função partilhada com o «+»). Abaixo da última faixa:
  `BeginEdit/AddLane/AddStrip/EndEdit` num só Ctrl+Z (resposta 4). Esc cancela. Ciclo do caminho aberto recusado (`add_strip_to`). Pisos OKLab do fantasma.
- Testes: `chip_drag_tests`, `tests/it/chip_para_faixa.rs`.

### Item 7 — chave no canvas
- `ph2d-app-motion/src/motion_key_points.rs` (shell líquido 0): caminho tracejado e losango por chave de Position X/Y na aba Keys; `MotionPathGrab::Key`;
  inversa por `Transform::inverse_compose` do pai posado; `upsert_key` cria o eixo que falta (resposta 3); caminho de antes durante o arrasto via
  `TimelineHistory::pending`; agulha parada; 1 undo. Só sem caminho Path. Testes: `motion_key_points_tests`, `motion_path`.
- Despacho: `despacho_metodos_modos_e_alcas.rs`, `fase_canvas_overlays.rs` (shell).

## Superfície de colisão (`git diff --stat 5d5c0616ab..HEAD`, 107 ficheiros)
- Crates: `ph2d-panel-timeline` (`chip_drag*`, `curvas_vista*`, `faixa_marcadores*`, `barra_clips`, `transport_*`, `tracks*`, `graph_paint`, `interact`, `state*`, `event`, `ids/*`, `lib`, `tests/it/*`),
  `ph2d-timeline` (`apply*`, `pose`, `history`), `ph2d-editor-core` (`constraint_edits`, `interaction/{timeline_cursor,types}`), `ph2d-app-components`, `ph2d-app-constraints`,
  `ph2d-constraints`, `ph2d-ecs` (`canal_componente*`, `canal_objeto*`), `ph2d-render` (`sprite/component*`), `ph2d-preview-drive`, `ph2d-timeline-preview`, `ph2d-tween`,
  `ph2d-tokens` (`chrome`, `timeline_tinta`), `ph2d-i18n` (`canais`, `constraints`, `timeline`), `ph2d-panel-inspector` (`seam_constraints`), `ph2d-app-motion` (`motion_key_points*`, `lib`, `motion_path_overlay`).
- Shell: `project_schema*`, `render_loop/{fase_canvas_overlays,fase_timeline_drain,sim_extract,sim_extract_emit,timeline_bridge_keys}`, `timeline_preview_tests`, `despacho_metodos_modos_e_alcas`.
- Cargo.toml de `ph2d-app-components`, `ph2d-app-constraints`, `ph2d-app-motion`; `Cargo.lock` (+1).
- Docs: plano 17, `04_color_tint_canais.md` (Sprite_projeto), fotos e `COMENTARIOS_CORTADOS`.
- O que um merge pode partir: a escada do `PROJECT_SCHEMA` (232) e o registo de canais (canal OBJETO 11 removido); `TimelineHitKind` (variante nova); quem escreve `Prop::Opacidade`; i18n `canais.rs` (-5 linhas).

## A FAZER na próxima janela (por esta ordem)
1. Testes por nome do que não correu (o build estava partido pelo chip a meio quando o acabamento do Curves/Flip terminou): `o_canal_diz_o_nome_do_campo_que_o_grava`,
   `marcas` (ph2d-app-components), `the_channel_says_the_inspectors_word`, `canal_componente` (ph2d-ecs), `seam_constraints` (ph2d-panel-inspector),
   `curves_view_seam` (incl. `mudar_de_clipe_ou_abrir_a_lista_volta_a_keys`), `curvas_vista`, `graph`, `chip_para_faixa`, `chip_drag`, `motion_key_points`, `motion_path`,
   os HR-15/censos de chaves, `panel_loc_cap`, `workspace_file_loc_cap`, `no_orphan_source_file`, `the_shell_only_shrinks`; `kill_w5` (ph2d-app-constraints, `--ignored`, perfil smoke).
   Use `bash scripts/cargo-test-narrow.sh <crate> <nome>`.
2. Mutações por fazer: `nome_do_cartao` → `None` (portão do Flip fica vermelho); `volta_a_keys` só na lista (teste da troca de clipe fica vermelho).
3. fmt + clippy `-D warnings` 1× nas crates tocadas (`ph2d-panel-timeline`, `ph2d-timeline`, `ph2d-editor-core`, `ph2d-app-components`, `ph2d-app-constraints`, `ph2d-constraints`,
   `ph2d-ecs`, `ph2d-render`, `ph2d-preview-drive`, `ph2d-timeline-preview`, `ph2d-tween`, `ph2d-tokens`, `ph2d-i18n`, `ph2d-panel-inspector`, `ph2d-app-motion`, `ph2d-host-desktop`).
   O clippy da shell não correu depois do item 7.
4. Fotos (patch local FOTO-HACK, revertido; `docs/Components/ferramentas/fotografa_cena.sh`, nunca `spectacle`) e lado a lado: `26_curves.png` + `26_lado_a_lado_curves.png` (maquete_B2),
   `28_chave_no_canvas.png` + `28_lado_a_lado_c4.png` (maquete_C4); refazer `27_chip_para_faixa` numa faixa VAZIA («Upper body») — a atual cai sobre a 2.ª «Run & jump».
5. `/pd-onda-fechar` e o smoke do dono (abaixo).

## Abertos conhecidos
- Dica «Drop: a new instance of …, 2 s long» da B4: não feita.
- Botão segmentado 28×~89 contra a maquete 25,6×100,4 (métrica partilhada com o Pixel; não mexido).
- Chaves do grafo grande com o desenho das âncoras, não os losangos de 9 px.
- Flip H/V como um grupo com fotografia de preview (versão maior): não feita.
- Custo por quadro das amostras do caminho no canvas (4/quadro, ≤ 2048, percorre a cadeia do pai): NÃO medido.
- Tempo remapeado nos pontos do canvas = tempo da chave (como o onion).
- Ciclos fundos no chip só são recusados pelo documento.

## Smoke do dono (passos numerados; rascunho)
Comando base: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && <VAR>=<n> ./target/smoke/ph2d-host-desktop`
1. **Curves.** `PH2D_ANIMACAO_SMOKE=1`. Aba Keys, clique «Curves»: grafo grande, curva escolhida cheia com números à esquerda, as outras tracejadas azuis com 0/1 à direita.
   Arraste um ponto: a chave muda e 1 Ctrl+Z desfaz. O olho esconde uma curva. «Keys» volta. Errado se: o botão aparece fora da aba Keys, o arrasto precisa de 2 Ctrl+Z, ou a vista continua em Curves ao trocar de clipe.
2. **Chave no canvas.** Mesma cena, objeto selecionado, aba Keys: caminho tracejado branco e um losango por chave. Arraste um losango: objeto e chave movem, o caminho de antes fica fraco, a agulha não mexe; 1 Ctrl+Z desfaz.
   Errado se: a agulha salta, o objeto não acompanha o ponteiro, ou aparecem pontos em Arrange.
3. **Chip → faixa.** `PH2D_NEST_SMOKE=4`, aba Arrange, arraste o chip «Idle pack» para a faixa «Upper body»: peça tracejada durante o arrasto, ao soltar nasce a peça. Solte abaixo de «FX»: nasce faixa nova (1 Ctrl+Z desfaz as duas). Esc cancela.
   Errado se: clique parado deixa de escolher o chip, soltar fora cria algo, ou o Ctrl+Z deixa a faixa vazia.
4. **Opacity.** Numa Sprite, a curva «Opacity» da Timeline mexe no «Opacity (%)» do Inspector (e o losango dele acende); apagar o pai apaga os filhos. `PH2D_CONSTRAINT_SMOKE=13`: o cartão diz «Opacity».
   Errado se: o Inspector não mexe, os filhos continuam visíveis, ou o cartão diz «Tint».
5. **Flip.** Chave no Flip H/V: o canal aparece nomeado «Flip image H/V». Errado se: o nome é o interno.
