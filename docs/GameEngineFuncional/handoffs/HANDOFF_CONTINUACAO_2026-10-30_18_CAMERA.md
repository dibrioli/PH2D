# Handoff de continuação — GEF 18: a câmera no canvas, onda 18 (30/10)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · reaberta sobre o main `422fec2027`
(a linha inteira já estava no main desde 29/10). Antecessor: [17b](HANDOFF_CONTINUACAO_2026-10-28_17b_CAMERA.md) ·
plano: [17](../17_camera_no_canvas_plano.md) A5–A7 + [17b](../17b_auditoria_camera_e_plano.md) passo 10.

## Estado

**Smoke do dono 30/10: APROVADO** (passos 1–7; no 6 a pergunta «o que se move é a zona morta?» — sim, ver §Decidido).
Commits sobre `422fec2027`: `39cb3c81fd` · `ccea39bea3` · `3d5bb4dd23` · `71f36d745c`. Árvore limpa. Binário
`smoke` compilado 2× (2.ª `Finished smoke profile in 0.19s`, zero `Compiling`). Linha **não** fechada (sem o
gate batched §1.5.9 nem handoff de integração): continua com a miniatura.

## O que entrou

- **Seta da Animation numa imagem Animated** (`image_use_rows.rs`): a §11 pintada dentro da Visibility ficava fora
  do `begin_section` ⇒ o cabeçalho nunca entrava no hit index (controlo morto). Regista-se ali
  (`anim::anim_header_h`). Gate `seam_image_use::a_seta_da_animation_de_uma_animada_dobra` (com
  `with_panel_and_shared_chrome`: a dobra das vivas declara-se no `pre_populate`); sem o register, vermelho.
- **Alças com o relógio parado em QUALQUER instante** (`fase_game_camera`: `em_edicao = !is_playing()`, era
  `&& t <= 0`). O defeito «o arrasto não grava chave com Auto Key» era ESTE: medido pela foto `camera_chave`
  (Auto Key + painel Timeline à vista, que é condição do `armed`): a 0 s já gravava 2 chaves (follow_offset x/y);
  a 1 s as alças estavam desligadas e o `Down` ia para a seleção (Sky). Depois: 1 s grava as 2 chaves e a câmera
  fica selecionada.
- **Formato do jogo no Desfazer**: `ProjectState.formato_do_jogo: Option<FormatoDoJogo>` com `#[serde(skip)]`
  (formato, moldura, ícone; a escala e as unidades ficam fora, como o cabeçalho de `project_settings.rs` já
  decidia). `capture_project` enche-o; `apply_project_with` só o repõe quando vem `Some` ⇒ o load continua a
  ler o blob `jogo` (sem degrau de `PROJECT_SCHEMA`, bytes do ficheiro iguais — gate
  `o_formato_do_jogo_e_um_passo_e_nao_muda_o_ficheiro`). Foto `camera_desfaz`: passo registado (`world=false`),
  Ctrl+Z 16:9, Ctrl+Y Game Boy; sem a captura, `global=false` (vermelho).
- **Câmera filha de pai rodado/escalado**: `Arrasto::com_pai(parent_world_transform)` (só com `ChildOf`); o miolo
  leva a câmera a `pose_mundo0 + delta` e a local sai de `Transform::inverse_compose`. Gate
  `com_pai_rodado_e_escalado_o_miolo_segue_o_ponteiro_no_mundo` (vermelho sem `com_pai`).
- **A5 — zona morta à vista e arrastável**: `Segue.zona`; `caixa_da_zona`/`pegas_da_zona` centradas na **MIRA**
  (a âncora de `dead_zone_goal` com o relógio parado), nunca no centro da moldura (a cerca prende-a); a moldura
  fora da zona tingida (`ZONA_TINTA`), contorno tracejado, 4 pegas nos meios das bordas, abertas a
  `ZONA_PEGA_MIN = 3` agarres do centro quando a zona é estreita/zero (fora do ícone). `Alca::ZonaMorta(borda)` →
  `Escrita::ZonaMorta` → `CameraFieldEdit::DeadZone` (fracção simétrica `|p − mira| / meia`, `0..=1`).
- **A7 — «Outside: Normal | Dimmed»** e **«Editor View: Free | Follows»** no cartão Camera (depois de Icon Size):
  `CameraCanvasView { dim_outside, follows }` em `ViewState` + `aplica()`; gestos do EDITOR como o Look through
  (não gravam, não desfazem); `CameraFieldEdit::{DimOutside, ViewFollows}`; `InspectorCameraInfo.canvas`;
  `POR_FUNCAO camera_format.rs::format_rows = NaoAnimavel`. Dimmed: `moldura_tinta::moldura_vazada` pinta o canvas
  fora da moldura da selecionada (ou da activa). Follows: `encaixe::centro_que_segue` — o centro do jogo no meio
  do canvas com a altura (zoom) do artista; o Look through continua a mandar.
- **Catraca `ph2d-app-game`** (3 465 > 3 202): `moldura_tests.rs` (só API pública) → `shells/game-scenes/tests/it/jogo_moldura_lei.rs` (7).
- **Fotos-gesto novas** (`PH2D_GAME_SMOKE_FOTO`): `camera_chave` (+`PH2D_GAME_SMOKE_TEMPO`, imprime
  `[camera-chave]` com as chaves da Camera), `camera_desfaz` (`[camera-desfaz]`), `camera_fora`, `camera_segue`.
  Fotos em [`fotos_18/`](fotos_18/).

## Decidido e recusado

- **A6 — a miniatura NÃO entrou.** Pede um 2.º desenho do mundo com outra câmera num alvo fora do ecrã; o present
  (`render_loop/present*.rs`, 1 673 linhas: faixas, Flip, Motion na placa, vetor Vello, luz 2D, tonemap,
  compositor) assume UMA câmera e alvos do tamanho da janela. Meia-miniatura só com `render_with_streams` (sprites)
  mentiria (sem vetor/Flip/luz). A fileira «Preview» foi retirada (botão morto). Mapa do explorador: porta
  reaproveitável `renderer.render_with_streams(GameRt::view(), …)` + `GameRt::new(gpu, size)` + `Compositor`;
  risco: estado da luz 2D/`vista` derivado da câmera do editor. A mesma capacidade falta ao retrato do asset
  (`asset_card_portrait.rs:11-14`).
- **Pausado a meio (t > 0) a moldura não anda com o arrasto** — é a vista daquele instante da corrida (o passe
  com `ticks == 0` não renasce fora do zero); o arrasto muda a mira (cruz) e a zona vai com ela; as chaves gravam
  a mudança. No zero a moldura renasce e anda. O dono perguntou no smoke (passo 6) e aceitou.
- Toggles da vista no cartão Camera (onde o D-V pôs as opções da câmera), não um botão novo no canvas.

## Gates e provas

Por nome (verificador): panel-inspector 88 · registry-init 3 · app-game 15 · components 45 · i18n 7 ·
game-scenes 8 (+ `jogo_moldura_lei` 7) · desktop 21 · editor-core (`every_editable_field_is_a_channel` 6,
`the_shell_only_shrinks` 2, `cada_edicao_de_vista_escreve_o_seu_campo`, e os pedidos). **Mutação 8/8**:
zona na mira · distância vs delta · pegas abertas · fileira view · leitura do snapshot · registo do OUTSIDE ·
`centro_que_segue` · (M8 `aplica` DimOutside→follows, sobrevivia ⇒ teste novo, agora sangra). Contraprovas à mão:
seta (sem register), `com_pai`, captura do formato (app real). Clippy, fmt da workspace e suíte inteira: NÃO
corridos (clippy só no fecho; suíte = integrador).

## ABERTO

1. **A6 — a miniatura do que a câmera vê** (próximo item): o mundo desenhado 2× por quadro com a câmera do jogo
   num `GameRt` pequeno, composto num canto do canvas; medir o custo da 2.ª passagem e conferir a luz 2D antes.
2. Fecho da linha (§1.5.9) depois da A6: gate batched, clippy `PH2D_SUITE=fecho`, handoff de integração.
