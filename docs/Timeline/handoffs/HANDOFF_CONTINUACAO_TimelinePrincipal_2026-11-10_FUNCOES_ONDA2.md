# Continuação — `line/TimelinePrincipal`, plano 17 (funções novas), onda 2 (itens 5–7 + cura do Opacity) — FECHADA, à espera do smoke do dono

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commits da onda sobre `5d5c0616ab`: `8acfdb5040` (fix do smoke da onda 1),
  `f52404c449` (parte 1), `0b7c8d058c` (parte 2), `b2438d11b1` (parte 3), `bd99e53549` (acabamento do Curves + fotos).
- Plano: [`17_plano_funcoes_novas.md`](../17_plano_funcoes_novas.md) — «Onda 2 — desenho medido» e «Onda 2 — respostas de produto» (dono 10/11 «escolha e siga»; ele pode vetar).
  Onda anterior: [`..._FUNCOES_ONDA1.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-10_FUNCOES_ONDA1.md).
- Estado: código e testes por nome verdes (secção «Fecho»), **sem smoke do dono**. Maquetes: `../fotos/maquete_B1.png`, `_B2`, `_B4`, `_C3`, `_C4`.

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

## Fecho (10/11)

### Testes por nome — todos verdes
- ph2d-app-components: `o_canal_diz_o_nome_do_campo_que_o_grava` (1), `marcas` (13).
- ph2d-ecs: `canal_componente` (15). ph2d-panel-inspector: `seam_constraints` (14).
- ph2d-panel-timeline: `the_channel_says_the_inspectors_word` (em `tracks_tests.rs`), `curves_view_seam` (5), `curvas_vista` (2), `graph` (27), `buffer` (6),
  `tracks` (11), `chip_para_faixa` (4), `chip_drag` (2).
- ph2d-app-motion: `motion_key_points` (5), `motion_path` (19). ph2d-render: `a_mistura_do_device_chega_ao_pixel` (3, não é `--ignored`). ph2d-tokens: `timeline_tinta` (2).
- ph2d-editor-core: hr15, `architecture_panel_loc_cap`, `architecture_workspace_file_loc_cap`, `architecture_no_orphan_source_file`,
  `architecture_the_shell_only_shrinks`, `no_magic_numeric`, `no_literal_color`.
- Shell: `project_schema`, `timeline_preview` (8), `animacao` (1). ph2d-app-constraints: `kill_w5` (`--ignored`, perfil smoke, 2).

### Mutações
- `nome_do_cartao` → `None` sangra (`marcas_inspector_escritor_tests.rs:132`). `volta_a_keys` só pela lista sangra (`curves_view_seam.rs:211`). Ambas restauradas.

### fmt e clippy
- clippy `-D warnings` das 16 crates da onda, e de novo ph2d-panel-timeline / ph2d-tokens / ph2d-host-desktop depois do acabamento: limpo. fmt limpo.

### Acabamento pela foto (`bd99e53549`)
Defeitos que a foto 26 contra a B2 achou:
- (a) A escala esquerda do grafo grande saía elidida («-3…» para «-320px»). `paint_track`/`paint_frame` recebem `label_w: Option<f32>`; `None` = escala DENTRO da banda
  com a largura medida pelo texto (`label_column_width`); o grafo por linha passa `Some(label_w)`.
- (b) O chip Store saía «St…» (largura fixa 38 contra o respiro de `label_budget`). Agora: o mínimo 38 ou a palavra mais larga de Store/Swap + 2×Md; no grafo grande
  os chips ficam à esquerda do eixo direito 0–1 (`curvas_vista::eixo_direito_w`).
- (c) A curva escolhida era azul como as secundárias. `TimelineTinta::CurvaPrincipal` (base KanbanPurpleBold, amostra `#B79CFF` da B2); piso OKLab sobre Bg0 medido
  0,420 (Candy), o piso fica 0,33; legenda igual. O grafo por linha continua no `TimelineCurve`.
- Cena `PH2D_ANIMACAO_SMOKE=1`: o clipe «pular» ganhou Position X (-3 → -1) para o caminho no canvas ser um arco (era uma linha vertical).

### Fotos
FOTO-HACK local revertido (`PH2D_FOTO_CURVES` em `paint.rs`, `PH2D_FOTO_CHIP` em `chip_drag::alveja`, `PH2D_FOTO_PULAR` em `animacao_smoke.rs`).
- `../fotos/26_curves.png` + `26_lado_a_lado_curves.png` (B2).
- `../fotos/28_chave_no_canvas.png` + `28_lado_a_lado_c4.png` (C4).
- `../fotos/27_chip_para_faixa.png` + `27_lado_a_lado_chip.png` (B4, agora na faixa VAZIA «Upper body»).

### Diferenças que ficam contra as maquetes (não defeitos)
- O caminho no canvas sai em retas porque as chaves da cena são lineares (a C4 desenha chaves com easing).
- A C4 realça a chave sob a agulha (anel azul): não feito. A dica «Drop: …» da B4 continua aberta.

## Superfície de colisão (`git diff --stat 5d5c0616ab..HEAD`)
- Crates: `ph2d-panel-timeline` (`chip_drag*`, `curvas_vista*`, `faixa_marcadores*`, `barra_clips`, `transport_*`, `tracks*`, `graph_paint`, `graph_buffer.rs`, `interact`, `state*`, `event`, `ids/*`, `lib`, `tests/it/*`),
  `ph2d-timeline` (`apply*`, `pose`, `history`), `ph2d-editor-core` (`constraint_edits`, `interaction/{timeline_cursor,types}`), `ph2d-app-components`, `ph2d-app-constraints`,
  `ph2d-constraints`, `ph2d-ecs` (`canal_componente*`, `canal_objeto*`), `ph2d-render` (`sprite/component*`), `ph2d-preview-drive`, `ph2d-timeline-preview`, `ph2d-tween`,
  `ph2d-tokens` (`chrome`, `timeline_tinta.rs`: variante nova `CurvaPrincipal`, append-only), `ph2d-i18n` (`canais`, `constraints`, `timeline`), `ph2d-panel-inspector` (`seam_constraints`),
  `ph2d-app-motion` (`motion_key_points*`, `lib`, `motion_path_overlay`).
- Assinatura mudada, crate-interna: `graph_paint::paint_track` / `paint_frame` com `label_w: Option<f32>`.
- Shell: `project_schema*`, `render_loop/{fase_canvas_overlays,fase_timeline_drain,sim_extract,sim_extract_emit,timeline_bridge_keys}`, `timeline_preview_tests`, `despacho_metodos_modos_e_alcas`, `animacao_smoke.rs` (cena «pular»).
- Cargo.toml de `ph2d-app-components`, `ph2d-app-constraints`, `ph2d-app-motion`; `Cargo.lock` (+1).
- Docs: plano 17, `04_color_tint_canais.md` (Sprite_projeto), fotos e `COMENTARIOS_CORTADOS`.
- O que um merge pode partir: a escada do `PROJECT_SCHEMA` (232) e o registo de canais (canal OBJETO 11 removido); `TimelineHitKind` (variante nova); `TimelineTinta` (variante nova);
  quem escreve `Prop::Opacidade`; quem chama `paint_track`; i18n `canais.rs` (-5 linhas).

## Abertos conhecidos
- Realce da chave sob a agulha no canvas (anel da C4).
- Dica «Drop: a new instance of …, 2 s long» da B4: não feita.
- Botão segmentado 28×~89 contra a maquete 25,6×100,4 (métrica partilhada com o Pixel; não mexido).
- Chaves do grafo grande com o desenho das âncoras, não os losangos de 9 px.
- Flip H/V como um grupo com fotografia de preview (versão maior): não feita.
- Custo por quadro das amostras do caminho no canvas (4/quadro, ≤ 2048, percorre a cadeia do pai): NÃO medido.
- Tempo remapeado nos pontos do canvas = tempo da chave (como o onion).
- Ciclos fundos no chip só são recusados pelo documento.

## Smoke do dono
Comando base: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && <VAR>=<n> ./target/smoke/ph2d-host-desktop`
1. **Curves** — `PH2D_ANIMACAO_SMOKE=1`. Na Timeline clique o clipe «pular», depois «Curves» (ao lado de «Keys», à direita da barra). Tem de aparecer: grafo grande,
   Position X em lilás com os números inteiros à esquerda, Position Y tracejada azul com 0/1 à direita, botão «Store» inteiro. Arraste um ponto da curva lilás: a chave muda;
   1 Ctrl+Z desfaz. O olho ao lado de «Position Y (px)» esconde a tracejada. Clique «andar»: volta a Keys.
   Errado se: números cortados com «…», 2 Ctrl+Z para desfazer, ou continuar em Curves ao trocar de clipe.
2. **Chave no canvas** — mesma cena, «pular» escolhido, aba Keys: no canvas um arco tracejado branco com 3 losangos. Arraste o losango do topo: o Herói vai junto e a chave muda;
   a agulha não mexe; 1 Ctrl+Z desfaz. Errado se: a agulha salta, o Herói não acompanha, ou os losangos aparecem na aba Arrange.
3. **Chip → faixa** — `PH2D_NEST_SMOKE=4`, aba Arrange. Arraste o chip «Idle pack» até a faixa «Upper body», depois do «swing»: aparece uma peça tracejada; solte e ela nasce.
   Arraste de novo e solte abaixo de «FX»: nasce uma faixa nova com a peça; 1 Ctrl+Z desfaz as duas. Esc durante o arrasto cancela.
   Errado se: um clique parado no chip deixa de o escolher, soltar fora da área cria algo, ou o Ctrl+Z deixa uma faixa vazia.
4. **Opacity** — numa Sprite, a curva «Opacity» da Timeline mexe no «Opacity (%)» do Inspector (e o losango dele acende); apagar o pai apaga os filhos.
   `PH2D_CONSTRAINT_SMOKE=13`: o cartão diz «Opacity». Errado se: o Inspector não mexe, os filhos continuam visíveis, ou o cartão diz «Tint».
5. **Flip** — ponha uma chave no «Flip image» do Inspector: a linha na Timeline chama-se «Flip image H/V». Errado se aparecer um nome interno.

## Próximo
Onda 3 do plano 17 (item 8 quem ouve um sinal; item 9 Animating · Done), cada um aberto por uma investigação curta — só depois do smoke OK.
