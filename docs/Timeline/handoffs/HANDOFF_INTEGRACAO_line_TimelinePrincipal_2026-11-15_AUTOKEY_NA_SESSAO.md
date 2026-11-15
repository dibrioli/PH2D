# Integração — `line/TimelinePrincipal`: o Auto Key grava durante a sessão «Animating» (dono 15/11)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · reaberta do main em 15/11
  (merge-base `7b3dd983f7`, sem rebase; a ponta anterior, já integrada em 13/11, está em
  `backup/TimelinePrincipal-pre-reabrir-2026-11-15`). Anterior: [plano 17 onda 5 + acréscimos 1–14](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md).
- Sem push, sem integração por esta linha.

## Pedido (verbatim)

1. «Autokey ligado, se eu criar novo clip, não grava key no novo clip até que eu aperte done no primeiro clip. Corrija: permita criar keys no novo clip mesmo sem ter checado done no anterior.»
2. «regravar o key com autokey ligado não está funcionando. faça funcionar»

## Mecanismo (medido no app, Xvfb, cliques reais, `PH2D_AUTOKEY_LOG=1` + sonda provisória na população)

- **Uma causa para os dois.** A sessão «Animating» (doc 17 item 9) declara cada objecto que a curva escreve com
  `PreviewDrive::driven_held` (`ph2d-timeline-preview::declare_timeline_writes_held`, chamado por
  `timeline_bridge.rs:240`) — condutor PERSISTENTE: a entrada não morre na `settle` enquanto a sessão durar.
- `autokey_amostra::populacao` tira da população quem `preview.drives(e)` (a guarda de 08/09 contra chaves cunhadas
  da saída de um motor do esqueleto). A sessão contava como motor ⇒ **todo objecto já com curva saía do autokey até ao
  Done**: regravar uma chave (B), chave nova noutro tempo do mesmo clip, e qualquer chave num clip NOVO (A — o objecto
  continua seguro pela sessão do clip anterior).
- Medido: com o Herói seleccionado e o Auto Key armado, a sonda deu `sel=[(fffffffa, drives=true)]` em 1 128 de 1 130
  quadros e `populacao=[fffffff7]` (só a Cena), `fila=0` em todos; depois do Done a população voltava.

## Cura

- `ph2d-preview-drive`: `Entry.held: bool` — `true` por `driven_held`, `false` por `driven` (um motor que escreve por
  cima volta a ser motor). Pergunta nova `drives_outside_the_session(entity)` (`consultas.rs`): algum condutor da
  entidade que NÃO é a sessão. `drives` fica como está (o outro consumidor, `ph2d-app-vec ui_state_bridge`, mantém a
  resposta de sempre).
- `shells/desktop/src/render_loop/autokey_amostra.rs::populacao` pergunta `drives_outside_the_session`.
- Porque é seguro: o que a sessão escreve É a curva (`mundo == curva` ⇒ o diff do autokey não cunha nada); o que a mão
  muda por cima é a chave que o artista quer. A guarda de 08/09 (esqueleto, IK, física — `driven`) continua inteira.
- Nenhum formato gravado muda (o ledger é estado de quadro). `PROJECT_SCHEMA` intocado (242, = main).

## Provas

- **App (Xvfb, cliques reais, Stop → Auto Key → gesto):** A — «+» cria «Clip 2», arrastar o Herói grava `Position Y`
  no Clip 2 sem Done · B — regravar a chave do instante 0: `Position X` −300 → −204; dois arrastos seguidos regravam a
  mesma chave duas vezes (−300 → −200 → −300, `Position Y` 48) · chave nova no quadro 3 do mesmo clip. Antes da cura os
  mesmos roteiros davam `fila=0` e nada no documento.
- **Teste novo:** `ph2d-timeline-preview held_tests::the_animating_session_is_not_a_motor_to_the_autokey` (a sessão
  segura o objecto — controlo — mas não é motor; a mesma curva sem sessão é). **Mutação** `drives_outside_the_session`
  ≡ `drives` ⇒ vermelho na linha da sessão (`held_tests.rs:123`); restaurado.
- Bateria por nome ✓: **ph2d-preview-drive inteira 6** (ledger mudou) · timeline-preview held 5 · host-desktop autokey 67, preview_drive 12, timeline_bridge 41, the_timeline_drivers_list 1, done 4 · app-vec ui_state_bridge 11 · panel-timeline animando 4, animating_done 4 · fmt ✓ · clippy `-D warnings --all-targets` (preview-drive, timeline-preview) ✓ · `cargo check -p ph2d-host-desktop --all-targets` ✓. Censos e suíte: do integrador.

## Acréscimo — a marca do estado nos menus da lane, da faixa, do fade e do Auto-Orient (dono 15/11)

- **Pedido** (verbatim, com foto do menu da lane): «o modo da lane não fica checado (não posso saber que modo está
  ativo). corrija» · depois: «procure problemas similares».
- **Mecanismo:** o pintor acende uma linha por `store.menu_row_is_marked(kind, id)`, e quem publica as marcas da
  timeline é `menu_marcas::linhas_acesas` — que só tinha braços para chave/coluna, submenu de ease e extrapolação.
  Varredura (explorador) dos menus da timeline com linhas de ESCOLHA: lane (Override/Additive), faixa
  (Once/Loop/Ping-Pong), fade (`StripFade` devolvia `None` de propósito — Smooth/Linear/Ease e o submenu de formas) e
  Auto-Orient (interruptor da track de trajetória; o estado não estava no snapshot). Os outros menus da timeline são
  ações/cascatas.
- **Cura:** braços novos em `linhas_acesas` (`TimelineLane`, `TimelineStrip`, `TimelineTrackPath`, `StripFade` no
  `TimelineSegment` e no `TimelineSegmentEase`; ajudantes `faixa`, `fade_de`); `TimelineViewSnapshot.auto_orient_on:
  Vec<u64>` (entidades cuja `Position` PEDE auto-orient — o pedido, como o interruptor; a recusa por Rotation fica na
  `TimelineDoc::auto_orient`), cheio no `rebuild`. Campo no snapshot e não no `TrackView` (29 literais de teste).
- **Fora da timeline, NÃO feito (visto pelo explorador, não confirmado):** o menu do tipo de alça das curvas
  (`CurvePointHandle`: Free/Aligned/Symmetric/Vector/Auto) e o do falloff não publicam marca; o estado vive na shell.
  Menus da Hierarquia/Board/Asset/Catalog não verificados linha a linha. Barra de menus e submenus de Settings já marcam.
- **Provas:** testes `menu_marcas::tests::{the_lane_menu_lights_the_lanes_mode,
  the_strip_fade_and_auto_orient_menus_light_their_state}` · app (Xvfb, Arrange → «+ Lane» → botão direito):
  [Override marcado](../fotos/38_lane_mostra_o_modo.png). Faixa, fade e Auto-Orient só por teste (a cena do smoke não
  tem faixa nem trajetória). `cargo check --workspace --all-targets` ✓ · timeline snapshot 20 · panel-timeline
  menu_marcas 4 · clippy `-D warnings --all-targets` (timeline, panel-timeline) ✓ · fmt ✓.

## Superfície de colisão

- `crates/ph2d-preview-drive/src/{lib.rs,consultas.rs}` (aditivo: um campo privado e um método público).
- `crates/ph2d-timeline-preview/src/held_tests.rs` (um teste).
- `shells/desktop/src/render_loop/autokey_amostra.rs` (uma linha + uma de doc).
- `crates/ph2d-panel-timeline/src/menu_marcas{,_tests}.rs` · `crates/ph2d-timeline/src/snapshot.rs` (campo novo `auto_orient_on`, aditivo).

## Aberto

- Smoke do dono deste pedido e dos acréscimos 13–14 (pontos e tipos de alça da curva Custom), já no main.
- Inspector: arrastar o ícone «X» de Position regrava a chave na sessão (sonda: −3,00 → −2,52, `fila=1` por passo).
  **Não medido:** DIGITAR o valor no campo — no Xvfb o texto não entrou (o campo ficou em edição com −300 e o mundo não
  mudou em nenhum quadro); não se sabe se é o roteiro (foco do teclado) ou o produto. Conferir no smoke do dono.
