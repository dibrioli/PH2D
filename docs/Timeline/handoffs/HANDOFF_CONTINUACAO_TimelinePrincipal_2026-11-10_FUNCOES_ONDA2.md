# Continuação — `line/TimelinePrincipal`, plano 17 (funções novas), onda 2 (itens 5–7 + cura do Opacity) + onda 2b — à espera do smoke da 2b

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commits da onda sobre `5d5c0616ab`: `8acfdb5040` (fix do smoke da onda 1),
  `f52404c449` (parte 1), `0b7c8d058c` (parte 2), `b2438d11b1` (parte 3), `bd99e53549` (acabamento do Curves + fotos), `ab823f94ae` (onda 2b).
- **Smoke do dono 10/11:** itens 2–5 (chave no canvas, chip → faixa, Opacity, Flip) OK. **Item 1 (Curves) RECUSADO:** «esse tipo de curva deve ser ajustado no canvas e não
  dentro da timeline. Na timeline só o botão pra editar as curvas no canvas». → onda 2b abaixo.

## Onda 2b (`ab823f94ae`) — a vista Curves SAI; entra a checkbox «Edit curves on canvas» e as alças no canvas

- **Saiu** (`ph2d-panel-timeline`): `curvas_vista*` e `curves_view_seam` apagados; ids `TIMELINE_VIEW_*` e olho por curva; i18n `panel.timeline.view.*`;
  `TimelineTinta::CurvaPrincipal`/`CurvaSecundaria`; `paint_track` volta a `label_w: f32` (o `Option` do acabamento acaba). Chips Store/Swap mantêm a largura medida.
- **Entrou:** checkbox `TIMELINE_CURVES_ON_CANVAS` (aba Keys, cartões; leitor público `state::curves_on_canvas()`; quebra para a sua fileira em janela estreita);
  `publish_curves_on_canvas` no idioma do `keys_mode`. Teste `curves_on_canvas_seam` (4).
- **Alças** (`ph2d-app-motion/src/motion_key_handles.rs`: `alcas`/`alca_hit`/`drag_alca`/`draw`; `MotionPathGrab::Alca` append-only; `drag_grab` porta única;
  edições em `motion_key_points*`, `motion_path_overlay`): alça = controlo da cúbica espacial. `BezierW` 1/3–2/3 por canal torna o tempo linear no parâmetro, logo o
  caminho é EXATAMENTE a cúbica (K0, C1, C2, K1). Posição pela derivada lateral (ajuste de 4 amostras); o arrasto escreve `bezier_w(1/3, C1−v0, 2/3, C2−v1)` com o
  outro lado parado. Chave que falta nasce no valor amostrado (Linear/Hold exato; curvo pela BezierW que reproduz a cúbica). 1 undo. Alça colada à chave
  (< `TANGENT_MIN_PX`) não se oferece. 7 testes (`motion_key_handles_tests`).
- Shell: `despacho_metodos_modos_e_alcas.rs` (+8), `fase_canvas_overlays.rs` (+1). Superfície de colisão: soma-se às crates da secção abaixo `ph2d-tokens` (variantes
  `CurvaPrincipal`/`CurvaSecundaria` REMOVIDAS: quem as usar parte), `ph2d-i18n/timeline.rs`.
- Mutações (todas sangram): `shows_keys→true`, `1/3→0,5`, sem `longe_da_chave`. clippy `-D warnings` e fmt limpos. Foto: `../fotos/29_alcas_no_canvas.png` (+ `.log`).
- Memória: `project-memory/project_timeline_curves_live_on_canvas_2026_11_10.md` (primário).
- As fotos `../fotos/26_*` ficam como registo da vista recusada; o texto abaixo sobre Curves (acabamento `bd99e53549`, item 5) é HISTÓRICO.
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

### Item 5 — Keys | Curves — SUBSTITUÍDO pela onda 2b (vista recusada pelo dono; ver acima)

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
- ph2d-panel-timeline: `the_channel_says_the_inspectors_word` (em `tracks_tests.rs`), `curves_on_canvas_seam` (4), `graph` (27), `buffer` (6),
  `tracks` (11), `chip_para_faixa` (4), `chip_drag` (2).
- ph2d-app-motion: `motion_key_handles` (7), `motion_key_points` (5), `motion_path` (19). ph2d-render: `a_mistura_do_device_chega_ao_pixel` (3, não é `--ignored`). ph2d-tokens: `timeline_tinta` (2).
- ph2d-editor-core: hr15, `architecture_panel_loc_cap`, `architecture_workspace_file_loc_cap`, `architecture_no_orphan_source_file`,
  `architecture_the_shell_only_shrinks`, `no_magic_numeric`, `no_literal_color`.
- Shell: `project_schema`, `timeline_preview` (8), `animacao` (1). ph2d-app-constraints: `kill_w5` (`--ignored`, perfil smoke, 2).

### Mutações
- `nome_do_cartao` → `None` sangra (`marcas_inspector_escritor_tests.rs:132`). Ambas restauradas.

### fmt e clippy
- clippy `-D warnings` das 16 crates da onda, e de novo ph2d-panel-timeline / ph2d-tokens / ph2d-host-desktop depois do acabamento: limpo. fmt limpo.

### Acabamento pela foto (`bd99e53549`)
- Mantido da parte Curves só: chips Store/Swap com a largura da palavra; resto (escala esquerda, `CurvaPrincipal`) saiu na 2b.
- Cena `PH2D_ANIMACAO_SMOKE=1`: o clipe «pular» ganhou Position X (-3 → -1) para o caminho no canvas ser um arco.

### Fotos
FOTO-HACK local revertido (`PH2D_FOTO_CURVES` em `paint.rs`, `PH2D_FOTO_CHIP` em `chip_drag::alveja`, `PH2D_FOTO_PULAR` em `animacao_smoke.rs`).
- `../fotos/26_curves.png` + `26_lado_a_lado_curves.png`: registo da vista RECUSADA. `../fotos/29_alcas_no_canvas.png`: alças (2b).
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

## Smoke da 2b (itens 2–5 da onda 2 aprovados pelo dono)
1. Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
   Clique o clipe «pular» e marque «Edit curves on canvas» (à direita, em cima na Timeline). No canvas: o arco tracejado, os losangos das chaves e pontinhos rosa entre eles.
   Arraste um pontinho para fora: o trecho entorta seguindo o ponteiro, o outro lado do trecho não mexe; dê play: o Herói percorre o arco torto. 1 Ctrl+Z desfaz.
   Desmarque: os pontinhos somem (os losangos ficam). Na aba Arrange nada aparece no canvas.
   Errado se: o arco não segue o ponteiro, o outro lado mexe, precisa de 2 Ctrl+Z, ou os pontinhos aparecem com a caixa desmarcada.
2. A setinha ao lado de «Position Y (px)» ainda abre a curva daquela linha dentro da Timeline. Errado se não abrir.

## Próximo
Onda 3 do plano 17 (item 8 quem ouve um sinal; item 9 Animating · Done), cada um aberto por uma investigação curta — só depois do smoke OK da 2b.
