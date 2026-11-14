# Integração — `line/TimelinePrincipal`: plano 17 onda 5 (itens 11–13) — o plano 17 completo

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · HEAD `80a3b1e6f5` (+ o commit deste handoff) ·
  merge-base `4f46a3573c` (= main de 12/11, sem rebase) · 15 commits · 46 ficheiros, +1 721 / −468.
- Ondas 1–4 do plano 17 (e onda 3 do plano 15) **já estão no main**: [handoff de 11/11](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-11_PLANO17_ONDAS_1_A_4.md).
  A linha foi reaberta do main em 12/11 (HEAD antigo em `backup/TimelinePrincipal-pre-reabrir-2026-11-12`); este diff é só a onda 5.
- Mecanismo e provas por item (não repetidos aqui): [itens 11 e 12](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-12_ONDA5_ITENS_11_12.md) ·
  [item 13 + os 3 abertos + alças](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-12_ONDA5_ITEM13.md) · plano [17](../17_plano_funcoes_novas.md) § «13 — desenho medido e feito».
- **Smoke do dono OK em 12/11** (Done rebobina, itens 11–13, pivô, os 3 abertos, alças maiores rosa). Sem push, sem integração por esta linha.

## O que a linha entrega

1. **Done rebobina a agulha** (`take_done(&mut Playhead)` pausa e rebobina).
2. **Item 11:** com a Timeline a animar (Play da própria Timeline), o rato fica no editor — curvas e chaves do canvas arrastam-se a tocar (`Tapa.animar` em `posse_do_jogo`).
3. **Item 12:** pintura e clique das curvas leem a mesma condição (provas; sem mudança de comportamento).
4. **Abas presas em Keys:** o restauro (Done, Ctrl+Z) carimba `timeline_last_selected` — deixa de ser «seleção nova».
5. **Item 13 (ADR-0141 ACEITO, modelo After Effects):** uma linha «Position» = ritmo ao longo do caminho; a FORMA mora no `MotionPath`; o easing só muda o ritmo.
   Entortar a alça converte X/Y → Position no mesmo passo de undo, guardando a forma (cúbica por 4 amostras), em **todo** clip do objeto.
6. **Ponto da curva sobre o pivô agarra-se** (`ph2d_editor_core::gizmo::is_gizmo_id` no guarda do canvas).
7. **Os 3 abertos do item 13:** caminho sob pai movido/rodado/escalado (porta única `world_for_local`/`local_for_world`, arrasto em `motion_key_points::drag_path`);
   laço da cena ANIMACAO no documento (`loop_whole_active_clip`); Convert to Motion Path com a pose real (`apply_intent_in`).
8. **Alças de tangente do caminho** com o token/raio de alça (5,0 / 6,5 px), iguais nos dois modos.

## Foundational / compartilhado tocado (aditivo salvo onde dito)

- `ph2d-timeline`: **novos** `separate_to_path_with_rest`, `loop_whole_active_clip`, `apply_intent_in` (re-export em `lib.rs`); `separate_to_path` e `apply_intent` mantidos.
  **Assinatura mudada:** `set_position_mode(entity, to_path, repouso: [f32; 2])` — chamadores todos na crate. `rewrite_path_key_values` privada → `pub(crate)`.
  `path_convert.rs` reescrito (a conversão passa por todo clip, nos dois sentidos).
- `ph2d-panel-timeline`: **mudada** `take_done(para: impl FnOnce())` → `take_done(agulha: &mut Playhead)` (único chamador externo: `fase_timeline_drain.rs:94`, atualizado);
  novo `request_curves_on_canvas()`; `ph2d-core` passou de dev-dep a dependência normal (sem duplicar).
- `ph2d-app-motion`: as portas do overlay (`marks`, `anchor_screen`, `tangent_screen`, `motion_path_hit`, `motion_path_curve_hit`) ganharam `world: &World` — sem chamadores fora da crate e da shell;
  novas `world_for_local`, `drag_path`, `caminho_visivel`, `curve_hit`, `marks_dragging`, `smoke_7`. `ALCA_R` 3,5→5,0, `ALCA_VIVA_R` 5,0→6,5.
- `ph2d-app-components`: `posse_do_jogo` ganha `animar: bool` no `Tapa`.
- `ph2d-editor-core`: só USADO (`gizmo::is_gizmo_id`, já existente); nada mudado.

## Contratos e formato — intocados (prova)

- §6: grep do diff por `NodeOp|OpResolver|NodeManifest|trait Tool|RasterEditTool|CanvasPaintTool|PanelEvent|VectorOp` = vazio; nenhum ficheiro de `ph2d-nodegraph`, `ph2d-vec*`, `ph2d-imageio` no diff.
- `PROJECT_SCHEMA` = **237** (`shells/desktop/src/project_schema.rs:459`) e `DOC_VERSION` = **25** (`crates/ph2d-timeline/src/doc.rs:132`), iguais ao main. Nada a recontar.
- Sem tool/node/painel novo; sem ADR novo (o 0141 mudou de estado para ACEITO e ganhou «Emenda — item 13»).

## Números medidos

- Shell (`shells/desktop/src`): **+27 / −44 = −17 linhas**; `the_shell_only_shrinks` verde.
- `ph2d-app-motion`: a bateria de fecho achou a crate **69 linhas acima do tecto** (197 585 × 197 516) e `motion_path_overlay_tests.rs` com 725 > 700.
  Cura por corte (`80a3b1e6f5`): os testes do overlay chamam as portas num mundo vazio por cinco embrulhos (`marks`, `anchor_pts`, `tangent_pts`, `hit`, `curve_hit` em `motion_path_overlay_tests.rs`) — −174 linhas; o ficheiro a 636. Mesmos testes (20/20).
- Item 13, reprodução: Hausdorff do arco depois do easing 0,269 antes da cura → < 1e-2 depois.
- Fotos (PNG, ~165–171 KB cada): `docs/Timeline/fotos/17_item13_{smoke_1,smoke_7,ponto_final_arrastado,alcas_maiores}.png`.

## Bateria de fecho (1×, sobre o diff acumulado, load 20–56)

Suíte completa e `nextest-impacted` NÃO corridos (regra do dono 23/10: são do integrador).
- `cargo fmt --all -- --check` ✓ · clippy `--all-targets -D warnings` em `ph2d-timeline`, `ph2d-app-motion`, `ph2d-panel-timeline`, `ph2d-app-components`, `ph2d-host-desktop` ✓ (o de `ph2d-app-motion` re-corrido após a cura ✓).
- Por nome: `ph2d-app-motion` motion_key_handles 11 · motion_key_points 6 · motion_path 20 (pós-cura 20) · motion_key 17 · `ph2d-timeline` path_convert 11 · view_clock 6 · intents 67 ·
  `ph2d-panel-timeline` animando 4 · seam 131 · `ph2d-app-components` posse_do_jogo 22 · `ph2d-host-desktop` the_motion_path 13 · gizmo 71 · position_mode 1 · double_click 1 ·
  the_undo_preserves_the_vector_selection 3 · the_drag_writes_through_the_documents_single_door 1 · timeline_bridge 41 · `--test it file_loc_caps` 4 ·
  `ph2d-editor-core` arch_safe_clamp_only 2 · o_claude_md_cabe_no_orcamento 1 · the_shell_only_shrinks 2 (vermelho → verde pela cura) · file_loc_cap 2 (vermelho → verde). Tudo ✓.
- `cargo build --profile smoke -p ph2d-host-desktop` ✓.
- Mutações: feitas por item na onda (todas sangraram; lista nos handoffs de continuação). M1 do item 13 sobrevive no teste de reprodução — a prova dele é ter estado vermelho no código antigo.
- **Auditoria, 2 lentes** (correção/seam · gates/contratos): nenhum defeito bloqueante. Os achados menores estão em «Aberto».

## Superfície de colisão

- `ph2d-timeline`: `path_convert.rs`, `path_convert_tests.rs`, `intent_apply.rs`, `intent_apply_path.rs`, `intent_loop_sync.rs`, `doc_path.rs`, `lib.rs`, `view_clock_tests.rs`.
- `ph2d-app-motion`: `motion_key_handles.rs`, `motion_key_points.rs`, `motion_path_overlay.rs` e os quatro `*_tests.rs`. ⚠️ O tecto de LOC da crate soma entre linhas — outra linha que cresça `ph2d-app-motion` recontas no integrador.
- `ph2d-panel-timeline`: `Cargo.toml`, `animando.rs`, `paint.rs`, `state.rs`, `state_requests.rs`.
- `ph2d-app-components/src/posse_do_jogo.rs`.
- Shell: `input_dispatch/despacho_metodos_modos_e_alcas.rs`, `input_dispatch/posse_do_jogo.rs`, `render_loop/{animacao_smoke,fase_canvas_overlays,fase_timeline_drain,timeline_bridge,timeline_bridge_k_tests}.rs`,
  `undo_app.rs`, `vec_text_reopen.rs`; gates `tests/it/the_motion_path_{anchor_is_drawn_and_dragged,is_offered_only_on_the_keys_tab}.rs`, `tests/it/the_undo_preserves_the_vector_selection.rs`.
- Cena **`PH2D_ANIMACAO_SMOKE=7`** nova (no main só existem 1–6): outra linha que acrescente uma cena ANIMACAO colide no número — conte, não escolha.
- Docs: ADR-0141 (append), `17_plano_funcoes_novas.md`; memória: `project-memory/MEMORY.md` + `project_timeline_easing_is_rhythm_along_the_path_2026_11_12.md` (nova) e `project_timeline_done_turns_off_autokey_2026_11_11.md`.

## Smoke (dono — já feito 12/11; para re-conferir após integrar)

1. `cd /home/enio/Documentos/Projetos/PH2D && PH2D_ANIMACAO_SMOKE=7 ./target/smoke/ph2d-host-desktop` (atalho: abre «pular» já entortado e com easing; `=1` abre sem nada).
2. Timeline, aba Keys, chip «pular», caixa «Edit curves on canvas» marcada; arrastar uma alça rosa do arco do herói; botão direito na 1.ª chave de Position → um easing.
3. O arco mantém a forma, só a velocidade muda; Ctrl+Z desfaz o entortar num passo; o ponto do fim (sobre o pivô) arrasta-se; Done leva a agulha ao início; com Play da Timeline, as alças arrastam-se a tocar.
4. Errado se: o arco muda de forma depois do easing, as linhas X/Y continuam separadas, o objeto se move em vez do ponto, ou a aba volta sozinha a Keys.

## Acréscimo — ajustes do dono 12/11 (depois do fecho; commits `aabe9342da`, `e524303f92` e o deste acréscimo)

Mecanismo e provas dos seis ajustes da barra: [continuação dos ajustes](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-12_AJUSTES_DA_BARRA.md).
1. **Barra:** caixa vazia com contorno 6,2:1 (depois, 2.ª queixa do dono, `tons_da_barra`: texto do Auto Key/Marker `TextoValor`→`TextoPropriedade` (3,7→6,6:1 na barra) e contorno da caixa `TextoPropriedade`→`TextoCabecalho`; teste `a_caixa_vazia_da_barra_tem_contraste_com_o_fundo` mede ≥ 4,5:1 contra `CardField` e `KanbanList` em todo tema, mutação sangrou; [foto antes/depois](../fotos/17_ajustes_12_11_contraste_auto_key_marker.png)); 3.ª queixa («nada»): as peças discretas (Auto Key, Marker, Edit Path) ganham corpo em repouso, `TonsDaBarra.corpo_da_peca` = `CardField` (campo novo em `ph2d-editor-core`), e `sob_o_rato` passa a `CardFieldHover` (também o hover das abas); [foto](../fotos/17_ajustes_12_11_auto_key_marker_com_caixa.png); «Record while playing» na engrenagem; «Show clips as a list»; «Edit Path»; Time/Frame compactos com ícones (`FilmFrame` novo); transporte em traço Thick.
2. **Ícones da lista de clipes = os do chip** (filme / cubo): `transport_clips::{CLIP_GLYPH, CONTAINER_GLYPH}` passam a `Film`/`Cube` (eram `Layer`/`Prefab`) e são a fonte única que `barra_clips::glifo_do_tipo` lê.
   Teste `o_glifo_do_chip_e_o_da_lista_aberta`. Foto: [antes/depois](../fotos/17_ajustes_12_11_icones_da_lista_antes_depois.png).
   E na lista FECHADA («as a list»): `CardCtx::lista_com_icone_em` → `widget::paint_card_chip_com_icone` (novo em `ph2d-editor-core`, `dropdown/card_chip.rs`; `chip` delega com `None`, sem mudança visual) e `card_chip_icone_w`. [Foto](../fotos/17_ajustes_12_11_icone_na_lista_fechada.png); prova = foto (a mesa de teste não lê glifos pintados). Por nome: clipes_em_lista 2, os_cartoes 35, seam 131, editor-core dropdown 36, censo 15, hr15 1, file_loc_cap 2, the_shell_only_shrinks 2; clippy editor-core + panel-timeline ✓.
3. **Defeito do Done (reproduzido em `ANIMACAO_SMOKE=1` e `=7`):** depois do Done as faixas liam «#7290» e não o nome do objeto, o caminho do canvas sumia, e na cena 7 o clip em edição trocava de «pular» para «andar».
   - Mecanismo: o restauro da cena dá bits novos a todo objeto (o mesmo `StableId`). Com o Done a Timeline fica parada (`timeline_bridge.rs`, `if sessao.1 {}`), e o apply era o único que marcava `binding.missing` (`apply.rs:284`). Sem a marca, a cura do `upkeep` nunca corria.
     E o `follow_selection` corre ANTES do `upkeep`: no quadro do restauro lia «o selecionado não tem Animação» e ia ao 1.º clip.
   - Cura (`ph2d-timeline-persist`): o `upkeep` marca também as bindings mortas e extrai a cura para `fn cura`. Novo `pub fn recola` (cura sem purga), chamado pelo `follow_selection`, que passa a `&mut World`. Único chamador: `fase_timeline_drain.rs:88`, onde `sim.world()` passa a `sim.world_mut()` (shell ±0).
   - Testes `o_upkeep_recola_a_faixa_sem_apply_depois_do_restauro` e `seguir_a_selecao_depois_do_restauro_mantem_o_clip_em_edicao`; as duas mutações sangraram.
     Fotos: [o defeito](../fotos/17_done_solta_as_faixas_antes_depois.png) · [depois da cura, cena 1](../fotos/17_done_mantem_as_faixas_depois_da_cura.png) · [depois da cura, cena 7](../fotos/17_done_mantem_clip_e_curvas_depois_da_cura.png).
- **Foundational tocado:** `ph2d-editor-core` (cards_banda, cards_banda_tons, checkbox `card_box.rs`, icons + `film-frame.svg`; ⚠️ o discriminante do `IconId` desloca-se uma casa depois de `film`, e ninguém grava o número, só o slug).
  `ph2d-timeline-persist`: **assinatura mudada** de `follow_selection(…, world: &mut World, …)`; nova `recola`.
- **Superfície a mais:** `ph2d-panel-timeline/src/{barra_clips,transport_clips,barra_menus,transport_sources,event,state}.rs` + `tests/it/clipes_em_lista.rs`; `ph2d-i18n/src/timeline.rs`; `ph2d-timeline-persist/src/{lib,animation,timeline_owner_tests}.rs`; shell `render_loop/fase_timeline_drain.rs`.
- **Bateria por nome (re-corrida 12/11, load 9–15):** fmt ✓ · clippy `-D warnings` (panel-timeline, timeline-persist, host-desktop) ✓ · censos 13/13 ✓.
  `ph2d-timeline-persist`: owner_tests 11, clip_names 3, purga_itens 3, item_apagado 4, reset_guard 3, publish_tests 4, textos 1, vetor_ids 1, tests:: 43.
  `ph2d-timeline`: `apply_from_doc_is_zero_alloc_steady_state` 1. `ph2d-panel-timeline`: os_cartoes 35, clipes_em_lista 2, transport 25, glyph 3, o_glifo_do_chip 1, seam 131, card 5.
  `ph2d-host-desktop`: the_motion_path 13, autokey 67, timeline 86, undo 94. Arquitetura: the_shell_only_shrinks 2, file_loc_cap 2, hr15 1. Tudo ✓.
- **Smoke ao dono (Done):** `PH2D_ANIMACAO_SMOKE=7`, Done ⇒ continua em «pular», o arco curvo continua no canvas, as faixas dizem «Herói»/«Espada».

## Acréscimo 2 — dono 13/11: controlos flutuantes e a duração na barra

- **Os controlos de tocar FLUTUAM acima do painel**, canto superior esquerdo, colados por fora da moldura (`transport_flutuante.rs`, submódulo de `transport_cartoes`): ⏮ ‹ ■ ▶ › ⏭ ●. O ▶ verde (`Success`), o ● vermelho cheio (`Danger`) = **Auto Key** (mesmo id `TIMELINE_AUTOKEY`, o mesmo `Toggle` à shell; a caixa da 2.ª fileira saiu e a chave i18n `panel.timeline.card.autokey` com ela), o ■ = **Stop** novo (`TIMELINE_STOP`, `clique_dos_cartoes`: `Pause` + `Scrub(0)`, sem tocar a shell).
  O rect é `painel.y − FIELD_GAP − FIELD_H` (preso à janela), a partir do `rect` real do painel (`paint_bar` ganha `(corpo, painel)`; o `ctx.slot` é a posição de NASCIMENTO de um flutuante).
  O HUD de diagnóstico cede-lhe: `bottom_hud::publica_fila_flutuante(r)` (novo, `ph2d-editor-core`, consumido a cada quadro como a coluna do módulo).
- `CardCtx::icones_em_fila_tingidos` (novo, `ph2d-editor-core`): a fila de ícones com tinta por célula; `icones_em_fila_com_traco` delega.
- **A Length (duração do clipe) sai da engrenagem para a 1.ª fileira**, entre as abas e o Time, com um separador de cada lado (`grupo_do_tempo`, glifo `CardLen`; o ∞ sem duração autorada continua por `barra_menus::valor_lido_da_dur`, agora `pub(crate)`).
- Testes: `tests/it/controles_flutuantes.rs` (3: flutua acima e por ordem, o ● liga o Auto Key, o ■ pára e volta ao início; mutações do Stop e do `rect` sangraram).
  Ajustados ao desenho novo: `os_cartoes` (duas fileiras, separadores da Length, alvo do Auto Key = célula 28 × 28), `os_chips_da_barra` (a 2.ª fileira mede-se pelo Marker), `o_cartao_estreito` (a fila flutuante é a única coisa pintada fora do cartão, de propósito), `no_lugar_seam` (a FX mede-se da lane Base: num painel de 860 px a migalha do container aberto faz a barra ganhar uma fileira).
- Por nome ✓: panel-timeline os_cartoes 35, seam 131, controles_flutuantes 3, os_chips_da_barra 7, o_cartao_estreito 3, tab 50, transport 25, barra 21, clip 38, duration_chip 15, animando 4 · editor-core bottom_hud 6, the_hud 3, dropdown 36, censo 15, hr15 1, file_loc_cap 2, the_shell_only_shrinks 2 · host-desktop timeline 86, autokey 67, the_motion_path 13 · fmt ✓ · clippy `-D warnings` (editor-core, panel-timeline, i18n) ✓. Shell intocada.
- [Foto](../fotos/17_ajustes_13_11_controles_flutuantes.png).
- **Depois (2.ª volta do dono, 13/11):** ordem ● ⏮ ‹ ▶ ■ › ⏭ (o gravar primeiro, o Stop depois do Play) e a fila numa **ABA** colada ao topo da lista (`aba()`: tinta `KanbanList`, só os cantos de cima redondos, desce `LIST_RADIUS` para dentro da moldura e cobre a junta) — painel e aba leem-se como um objecto. A caixa dos botões tem as quinas de baixo pontudas (`icones_em_fila_tingidos` recebe os raios ↖↗↘↙). E sem folga em baixo: o fundo da caixa assenta na linha do topo da lista. `o_cartao_estreito` aceita a aba (do ● ao ⏭ + a folga + o raio). [Foto](../fotos/17_ajustes_13_11_controles_na_aba.png).

## Acréscimo 3 — dono 13/11: abas laterais de pasta e o Done verde no clipe

- **Keys · Containers · (container aberto) · Arrange viram ABAS DE PASTA na borda esquerda** (`src/abas_laterais.rs`, novo; testes em `abas_laterais_tests.rs`, 5).
  Mesma lista, escolhida e ids de sempre (`transport_tabs::items/selected`), o mesmo clique. Texto rodado 90° (`paint_text_rotated_ccw`, de baixo para cima), `VALUE_PX`, âncora arredondada ao pixel.
  - **2.ª volta do dono (13/11, verbatim):** «mal posicionada, mal renderizada, abas pequenas e feias. Sobrepostas de forma errada. Deve ocupar toda lateral da timeline e se expandir quando a timeline crescer.»
    Defeitos da 1.ª versão: as inativas eram translúcidas (alfa 0,55), pintadas ANTES da moldura, e a sombra da lista caía-lhes em cima (faixa escura). A sobreposição de duas translúcidas fazia uma risca. As abas tinham tamanho fixo (42 px).
  - **Agora:** as abas dividem a altura INTEIRA da lista, entre as quinas (`lista.y + LIST_RADIUS` … `fim − LIST_RADIUS`). A ativa pesa `PESO_ATIVA = 2`, as outras 1, e a ativa nunca fica menor que o nome. Crescem com o painel.
    Contorno de aba de navegador (`contorno`): base reta do lado da moldura, passando `PONTA = FIELD_RADIUS` da fatia de cada lado; lado de fora mais curto; pontas inclinadas; quinas de fora redondas (quadrática amostrada; `paint_shapes::fill_polygon`, sem trig nem Affine).
    Todas se pintam DEPOIS da moldura (`pinta`, chamada em `paint_bar`): a mais longe da ativa primeiro, a ativa por último. Cores SÓLIDAS: ativa `KanbanList` + `TextoCabecalho` (entra `SEG_INSET` na moldura); inativa `CardField` + `TextoValor`; sob o rato `CardFieldHover` + `TextoPropriedade`.
    O nome fica ao centro da fatia; se não cabe, mostra o começo (`comeco`, sem reticência), lido a partir de baixo. O alvo é a fatia; a dica é o nome inteiro (`set_tooltip`).
  - **3.ª volta do dono (13/11, verbatim):** «está quase boa. Deixe ir mais alto até a altura dos comandos. Faça acabamentos pois tem defeitos. A aba ativa deve mostrar o nome completo.»
    Defeitos medidos na foto ampliada: um vão em «V» do lado de fora entre duas abas inclinadas; uma cunha no pé, ao lado da quina de baixo da lista; «Arrang» porque o `max_width` do texto rodado (`w + 1`) QUEBRAVA a linha (o pintor engrossa a letra).
    Agora a pilha começa no topo da aba dos comandos (`pilha((lista, topo))`, `topo = flutuante::aba(...).y`, que passou a `pub(crate)`) e vai até `lista.y + lista.h − LIST_RADIUS`. A aba dos comandos tem a quina de cima-esquerda reta.
    Cada lado de uma aba é uma `Ponta`: `Reta` nas pontas da pilha (quina de fora redonda), `Inclinada` à frente da vizinha, `PorBaixo` atrás dela (reta, `2·PONTA` por baixo, sem «V»). A ativa mostra sempre o nome inteiro; o texto rodado tem largura de folga (`desenho.h.max(2w)`).
    Testes 6 (+ `a_de_tras_passa_por_baixo_e_a_ativa_mostra_o_nome_inteiro`, mutação «sempre Inclinada» sangrou). `controles_flutuantes` mede «acima do painel» pela fileira do título; `o_cartao_estreito` aceita a coluna da calha, do topo da aba dos comandos ao fundo do cartão.
    Fotos: [Keys · Containers · Arrange](../fotos/19_abas_laterais_v3_keys_containers_arrange.png) · [zoom](../fotos/19_abas_laterais_v3_zoom.png).
  - **4.ª volta do dono (13/11, verbatim):** «o K tem que aparecer com a timeline no mínimo. Pelo menos a primeira letra de cada aba. As abas inativas devem ter cor diferente entre si.»
    Na altura mínima (`MIN_H` 120) a ativa ficava com a coluna toda e a Keys inativa sem letra nenhuma.
    Agora a reserva das inativas sai PRIMEIRO: 1.ª letra + margens, o máximo entre elas, vezes o número de inativas. A ativa fica com `max(partilha, min(nome, altura − reserva))`. Se nem tudo cabe (container aberto no mínimo), as 1.as letras ganham e a ativa corta.
    As margens do nome passaram a ser por lado (`margem(Ponta)`: `SEG_INSET`, mais `PONTA/2` por trás da vizinha); antes era `PONTA + SEG_INSET` dos dois lados, e isso comia a coluna.
    Cor: cada degrau de distância à ativa lava a inativa com `TextoValor` a `TIMELINE_ABA_RECUO_ALPHA = 0,22` por degrau (token novo; 0,12 lia-se igual na foto).
    Testes 8 (+ `no_minimo_cada_inativa_mostra_a_primeira_letra`, `sem_lugar_para_tudo_a_primeira_letra_ganha`). A mutação «sem reserva» sangrou só no 2.º: no mínimo real sobra lugar.
    Fotos: [no mínimo, as três](../fotos/19_abas_laterais_v4_no_minimo.png) · [cores](../fotos/19_abas_laterais_v4_cores.png).
  - O token `TIMELINE_ABA_INATIVA_ALPHA` da 1.ª versão saiu. Mutação «altura fixa da ativa» sangrou em `as_abas_ocupam_a_lateral_inteira_e_crescem_com_ela`.
- **A calha:** o cartão reserva `ESPESSURA = FIELD_H` à esquerda (`abas_laterais::sem_a_calha`). Moldura, barra, folha (`paint.rs::area_da_folha`), faixa de arrastar (`cartao_solto::faixa_do_titulo`) e a aba dos controlos medem-se do rect sem a calha. As alças de redimensionar continuam no rect inteiro (as abas registam-se depois e ganham-lhes).
- A 1.ª fileira perdeu as abas: `abas_e_migalha` → `migalha`, `segs_das_abas` apagada. O separador antes da Length só se pinta se algo vem antes (`f.no_inicio()`): a Length encosta ao «TIMELINE».
- **O Done** (dono 13/11, verbatim: «agregado ao próprio clip ou, se os clips estão no modo dropdown, à direita do dropdown depois do ⋯; não como Done (palavra) mas como um botão verde com ícone de checado»):
  - Sai da pílula «● Animating …», que fica só com o aviso. `animando::botao_done` = `Success` + `IconId::Check` em `AccentFg`, anel sob o rato, e `dicas::icone`.
  - No modo chips: redondo dentro da ponta direita do chip escolhido, depois do «⋯» (`Chip.done`, +`FIELD_H` na largura; `botao_done_no_chip`).
  - No modo lista: um campo `FIELD_H` à direita do «⋯», antes do «+» (`como_lista` recebe `(opcao, done)`).
  - Id, clique e lei da sessão inalterados (`TIMELINE_ANIMATING_DONE`). `pilula_w` ganhou um vão (`tw.ceil() + gap`): a reticência comia «andar».
- **Testes ajustados:** `os_cartoes` — a 1.ª fileira mede-se pela Length; o alvo da aba em pé: a ATIVA tem de comprido ≥ 36,8/69,9/55,2 e a inativa ≥ 24×24; a folha começa depois da barra sem contar as abas. `controles_flutuantes` — a fila começa à direita das abas. `o_cartao_estreito` — a sombra envolve a moldura sem a calha.
  Teste novo `animating_done_seam::o_done_e_um_botao_colado_ao_clipe_depois_do_menu` (chips e lista); a mutação «⋯ sem descontar o Done» sangrou.
  `numa_janela_estreita_a_barra_quebra_e_nada_sai`: as abas descem AO LADO da folha (à esquerda de `+ Track`), não por cima.
- **Por nome ✓ (13/11):** panel-timeline it 73 (os_cartoes, o_cartao_estreito, controles_flutuantes, view_tabs, animating_done, curves_on_canvas, clipes_em_lista, os_chips_da_barra, no_lugar); lib abas_laterais 8, animando 4, transport_tabs 3, container_list 9.
  editor-core it 20 (file_loc_cap, hr15, the_shell_only_shrinks, panel_loc_cap, censo). fmt ✓ · clippy `-D warnings` (panel-timeline, tokens) ✓. `barra_clips.rs` = 600 linhas (no tecto).
- `PH2D_CARDS=0` intocado. Shell intocada. Fotos da 2.ª volta: [Keys · Containers · Arrange](../fotos/19_abas_laterais_v2_keys_containers_arrange.png) · [zoom](../fotos/19_abas_laterais_v2_zoom.png). 1.ª volta (recusada): [Keys + Done verde](../fotos/19_abas_laterais_keys_e_done_verde.png) · [Containers](../fotos/19_abas_laterais_containers.png) · [Arrange](../fotos/19_abas_laterais_arrange.png). Done em lista: [foto](../fotos/19_done_verde_clips_em_lista.png).

## Acréscimo 4 — dono 13/11: smoke das abas OK; pulso de «ligado», sem «Animating», Edit Path botão

- **Smoke do dono das abas laterais: OK** (13/11).
- **Pulso de «ligado»** (verbatim: «para auto rec e play, animação cíclica de "ligado" — piscando com brilho, quando ativados»).
  Com o Auto Key ligado, o ●, e a tocar, o ▶ levam um halo na cor do botão que cresce e acende em ciclo (`transport_flutuante::brilho`, `TIMELINE_LIGADO_BRILHO_ALPHA = 0,45`, token novo).
  O relógio: `WidgetStore::on_pulse()` (novo, `ph2d-editor-core`, `tool_bars_ops.rs`), período 2 × `Duration::Slow` = 800 ms, abaixo do tecto da WCAG 2.3.1, a partir do `clock_ns` que o `dispatch_tick` já anota.
  Com movimento reduzido fica aceso fixo: `WidgetStore::note_reduced_motion` (novo), espelhado por `hero/live.rs` depois do `motion.advance`; campo `ToolBars::reduced_motion`.
  Teste `the_on_pulse_cycles_and_holds_still_with_reduced_motion`. [Foto: três instantes do ▶](../fotos/20_pulso_do_play_tres_instantes.png).
- **A pílula «● Animating …» saiu** (dono: «não é mais necessário, já temos o check ao lado do nome do clip»). Saíram `animando::pilula`, `pilula_w` e `medidas`; o `rotulo` continua a decidir se o ✓ aparece.
- **Edit Path é um botão que acende, só com o nome** (dono: «mais compacto tirando o checkbox e colocando como toggle button só com o nome»). `src/alternador.rs` (novo; saiu de `transport_cartoes`, que estava no tecto de 600). Ligado leva `ativa_fundo` + `ativa_texto` (o chip escolhido); o mesmo id e o mesmo clique (o store espelha a caixa em `prepara`).
  ⚠️ Desvio consciente da regra da casa «liga/desliga = checkbox» (dono 10/11): pedido EXPLÍCITO do dono para este botão. [Foto](../fotos/20_edit_path_botao_e_sem_animating.png).
- Por nome ✓: panel-timeline it 73, lib abas_laterais 8, animando 4, a_caixa_vazia_da_barra 1 · editor-core it 20 (file_loc_cap, hr15, the_shell_only_shrinks, panel_loc_cap, censo), lib flash/on_pulse/bottom_hud 9 · fmt ✓ · clippy `-D warnings` (panel-timeline, editor-core, tokens) ✓.
- **Smoke do dono do acréscimo 4: OK** (13/11). E mais dois pedidos (verbatim: «Stop deve desativar Auto Rec · Edit Path deve piscar continuamente se ativado»):
  - O ■ chama `animando::para()` (novo: liga o `FEITO` sem estacionar a sessão). A shell já desliga o Auto Key por `take_done` em `fase_timeline_drain.rs:94`, e esse `take_done` também pára e volta ao início. Shell intocada.
    `controles_flutuantes::o_stop_para_e_volta_ao_inicio` agora exige o `take_done`; a mutação «sem `para()`» sangrou.
  - O Edit Path ligado pulsa: o `Accent` a `TIMELINE_LIGADO_BRILHO_ALPHA × on_pulse()` por cima do corpo (`alternador.rs`). [Foto: três instantes](../fotos/20_edit_path_pulsa_tres_instantes.png).
  - Por nome ✓: it 42 (controles_flutuantes, os_cartoes, o_cartao_estreito, animating_done, curves_on_canvas), animando 4, editor-core it 18 · clippy panel-timeline ✓ · fmt ✓.
- ~~PEDIDO ABERTO para a próxima janela: o widget do fim do clip~~ — feito no acréscimo 5.

## Acréscimo 5 — dono 13/11: o fim do clipe na barra (Stop · Loop · Ping-Pong)

- **Pedido** ([continuação, verbatim](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-13_FIM_DO_CLIP.md)): um widget original na 2.ª fileira — lista suspensa com `‹ ›` nas laterais e, perto delas, os ícones das outras opções; clicar no ícone escolhe direto; a troca é animada.
- **`src/fim_do_clip.rs`** (novo; `transport_cartoes` e `barra_clips` no tecto de 600): `‹ [vizinha] [ícone NOME ▾] [vizinha] ›`, encostado à direita da fileira dos clipes. Roda `Stop → Loop → Ping-Pong`; `‹`/`›` andam um passo (com volta); o ícone da vizinha usa o id da opção (`TIMELINE_END_STOP|LOOP|PINGPONG`, já registados); o centro (`TIMELINE_END_DD`, novo) abre a lista (`barra_menus::flutua`, fecha pelo clique de fora, como a engrenagem). Ids novos `TIMELINE_END_DD/PREV/NEXT` (`ids/timeline_barra.rs`, registados em `populate`). Estado `TimelinePanelState::fim_aberto` e `BarView::fim_aberto`; o abrir de uma janelinha fecha as outras; esconder o painel fecha.
- **O evento é o de antes:** `toggle_que_pede` (mudou-se de `barra_menus` com o `Fim`) pede o `Toggle` de `TIMELINE_LOOP`/`TIMELINE_PINGPONG` que a caixa pedia; a escolhida não pede nada.
- **Saiu da engrenagem** «When the clip ends» (um só lugar); a chave `panel.timeline.card.when_ends` saiu da tabela. E a órfã `panel.timeline.animating.done` (a pílula que saiu no acréscimo 4) também — o portão `every_key_of_this_crate_exists_on_both_sides` estava vermelho por ela.
- **Animação:** `fenda(op, (de, para), t)` — cada ícone desliza da fenda velha para a nova (−1 · 0 · +1); o que dá a volta à roda sai por um lado transparente e entra pelo outro; o nome só se vê dentro do centro e cruza a meia fenda. `Duration::Default` (240 ms) com smoothstep, relógio `clock_ns`; com movimento reduzido é seco — `WidgetStore::reduced_motion()` (getter novo, `tool_bars_ops.rs`). A troca vive num `thread_local` do módulo (a 1.ª vista não anima).
- **Fileira:** `Fluxo::guarda_a_direita` reserva a largura do widget nas fileiras do interior antes do gravar/clipes; `solta` põe-no lá. Se nem um chip cabe ao lado (painel estreito, ex.: 266 px nos testes), `barra_clips` chama `larga_reserva` e o widget desce para a fileira seguinte em vez de esmagar os clipes.
- **Ícones novos** `IconId::Loop` (Lucide «repeat») e `IconId::PingPong` (Lucide «arrow-left-right»), ISC, `docs/design/icons/{loop,ping-pong}.svg`; o Stop é o `IconId::Stop`. Dicas em inglês (`panel.timeline.tip.end_*`).
- **Fotos** (tiradas com um gancho TEMPORÁRIO que trocava sozinho e deixava a troca 8× mais lenta — removido, não commitado): [troca a meio + lista aberta](../fotos/21_fim_do_clip_troca_e_lista.png) · [na barra](../fotos/22_fim_do_clip_na_barra.png).
- **Testes:** novos `fim_do_clip::tests` (4: toggles, roda, `fenda` a meio e na volta, movimento reduzido) e `os_cartoes::o_fim_do_clipe_mora_na_barra_e_as_setinhas_pedem_o_do_lado` (ordem, mesma fileira do Marker, sem pisar os clipes, `‹` = ícone da esquerda, lista abre e fecha por fora, a engrenagem já não o tem). `onde_mora` e `stop_pede_o_que_desmarcar_a_caixa_pedia` atualizados (Loop/Ping-Pong agora na barra; a escolhida clica-se na lista).
- Por nome ✓: panel-timeline os_cartoes 36, o_cartao_estreito 3, os_chips_da_barra 7, every_word 3, controles_flutuantes 3, clipes_em_lista 2, barra_menus 1, fim_do_clip 4 · editor-core icons 9, on_pulse 1 · registry-init toda_dica 2 · i18n tests 10 · fmt ✓ · clippy `-D warnings` (panel-timeline, editor-core, i18n) ✓.
- **Smoke do dono: aprovado (13/11, «ótimo»).**

## Acréscimo 6 — dono 13/11: sem setinhas, escolhida com cor, placa de pare, régua legível

- **Setinhas `‹ ›` saíram** (dono: «parecem supérfluas»): ids `TIMELINE_END_PREV/NEXT` e chaves `tip.end_prev/next` apagados. O widget é `[vizinha] [escolhida ▾] [vizinha]`, `largura = 2·FIELD_H + centro`.
- **A escolhida com cor** (dono: «mais destacada, com cor»): o centro veste o chip escolhido da barra (`tons_da_barra`: `ativa_fundo` + `ativa_texto`); sob o rato/aberta, `sob_o_rato` com o contorno `ativa_fundo`. [Foto](../fotos/21_fim_do_clip_sem_setinhas_com_cor.png). Teste renomeado `o_fim_do_clipe_mora_na_barra_e_cada_lado_pede_a_sua_opcao`.
- **Placa de pare** (dono: «o ícone de stop está muito simples… talvez de placa»): `IconId::StopSign` (novo, `stop-sign.svg`, desenho próprio): octógono CHEIO (`cheio()`, `fill="currentColor"`) com a barra vazada por enrolamento contrário. O octógono de contorno (Lucide `octagon-minus`) e o de quinas vivas lidos a 16 px pareciam um círculo — fotografados e recusados. O `IconId::Stop` do transporte fica. [Foto](../fotos/22_placa_de_pare.png).
- **A régua** (dono: «poucas marcações e as que tem são quase invisíveis»): medido — o traço `ReguaTraco` contra `FitaRegua` dava **1,3:1**, e os riscos eram fixos a 1 s / 0,5 s em qualquer zoom. Agora `src/ruler_degraus.rs` (novo; `ruler.rs` no tecto): três degraus pelo espaço em píxeis — número (≥ 90 px, passos 0,1 · 0,25 · 0,5 · 1 · 2 · 5 · … · 600 s), meio (metade, ≥ 24 px) e miúdo (um QUADRO quando ≥ 6 px e divide o número; senão ⅒ · ⅕ · ¼ · ½) — alturas 10/7/4 px. Tintas `tons_dos_riscos`: número e risco grande `TextoPropriedade`, meio e miúdo `TextoValor` (≥ 3:1 em todo tema, WCAG 1.4.11); no clássico Text2/Text3. Números «0.25», «1.5». No zoom de fábrica (~480 px/s, 24 fps): número a cada ¼ s, meio a cada ⅛ s, um risco por quadro. [Foto](../fotos/22_regua_com_riscos_por_quadro.png).
- Testes novos `ruler::degraus::tests` (4: zoom de fábrica, todo zoom × fps cabe e divide, contraste em todo tema, texto do número).
- Por nome ✓: panel-timeline ruler 30, os_cartoes 36, fim_do_clip 5, every_word 3 · editor-core icons 9 (inclui o portão do ícone cheio) · registry-init toda_dica 2 · fmt ✓ · clippy `-D warnings` (panel-timeline, editor-core) ✓.
- **Smoke do dono: aprovado (13/11, «ótimo»).**

## Acréscimo 7 — dono 13/11: a cara dos menus do botão direito da Timeline

- **Pedido** ([continuação](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-13_MENUS_DO_BOTAO_DIREITO.md), verbatim: «Agora é hora de melhorar a aparência os menus de botão direito»).
- **A porta é ÚNICA e partilhada:** `editor-core/src/screens/hero/context_menu_overlay.rs` pinta todo menu do app. A cara nova só se liga a ids da timeline: o novo `screens/hero/menu_row_look.rs` responde `icone(id)`, `abre_submenu(kind, id)`, `destrutiva(id)`, `risco_antes(id)`; um id fora dele pinta como antes (o `SettingsMenu` mantém a seta). Menu com ícones ALARGA ao rótulo mais longo medido (`prefix_width`), senão fica nos `MENU_W` 200.
- **Riscos** entre grupos saem da cara da linha, não da tabela (a tabela também alimenta a paleta de comandos): chave `Hold·Nearest·Linear | Ease In·Out·In-Out | Custom·Rove`; faixa `conversões | Extrapolation Pre·Post | Delete`; pista `modos | Rename | Delete`; tira `Enter·Duplicate | Once·Loop·Ping-Pong·Reset | Delete`; marcador `Rename·Signal | Delete`.
- **Delete por último** em todas as tabelas (`TIMELINE_{TRACK,AXIS_TRACK,PATH_TRACK,STRIP}_MENU` reordenadas; ninguém indexava por posição), rótulo e ícone em `ColorToken::Danger`.
- **Seta de submenu**: o «▶» saiu das cinco chaves i18n (`ease_in/out/in_out`, `extrapolation_pre/post`); `IconId::ChevronRight` em `Text3` à direita, e o rótulo reserva a largura dela.
- **Marca do estado atual**: canal novo `WidgetStore::set_menu_marks(kind, ids)` / `menu_row_is_marked(kind, id)` (campo `menu_marks`, preso ao `kind`, que traz chave/faixa). Quem publica é o painel: `panel-timeline/src/menu_marcas.rs` (chamado no `paint` logo após `publish_view`) acende a interpolação da chave (Column só se todas as chaves concordam; Rove junto se rova), a família no submenu de easing (só no modo dela, via `preset_for`) e a extrapolação Pre/Post. O pintor usa a bolinha `Accent` de sempre + ícone em `Accent` + texto `Text1`.
- **Peso do texto:** mantido o do app (os cartões pintam todos os rótulos no mesmo peso); trocar só aqui deixaria o menu diferente do resto.
- **Fotos** (Xvfb + clique direito real, `PH2D_ANIMACAO_SMOKE=1`): [faixa antes × depois](../fotos/30_menu_da_faixa_lado_a_lado.png) · [chave antes × depois](../fotos/30_menu_da_chave_lado_a_lado.png) · [submenu de extrapolação com Hold marcado](../fotos/30_submenu_extrapolacao_depois.png).
- **Testes novos:** `editor-core screens::hero::menu_row_look::tests` (2: toda linha da timeline tem ícone exceto as famílias; o destrutivo é a última linha e nenhum rótulo leva «▶») · `panel-timeline menu_marcas::tests` (1: chave Hold/Ease Out, família só no modo dela, extrapolação Pre/Post, chave inexistente não acende).
- Por nome ✓: editor-core context_menu 31, menu_row 13, every_menu_row 5, every_timeline_menu 3, timeline 45 · panel-timeline menu_marcas 1, timeline_presets_menu 10, extrapolation 5, grupo_de_canais 5, every_word 3 · i18n tests 10 · host-desktop timeline_presets 14 · fmt ✓ · clippy `-D warnings` (editor-core, panel-timeline, `--all-targets`) ✓ · `cargo build --profile smoke` ✓.
- Smoke do dono: pendente.

## Acréscimo 8 — dono 13/11: o menu da chave vira o CARTÃO da maquete

- **Pedido** (verbatim: «veja a maquete. Melhore a nossa»; [maquete](../fotos/31_cartao_da_chave_maquete.png)): título «Key at frame 12 · Position Y», três blocos Curve · Straight · Hold, fileira Ease in · Ease in-out · Ease out, escolhido em azul.
- **`editor-core/src/screens/hero/key_card.rs`** (novo): o overlay desvia `TimelineSegment{Key|Column}` para o cartão (`e_cartao`); o fade continua em lista. Cartão = título · 3 blocos (`TILE_W/H` 76×58) · fileira do ease · «Curve shape [Cubic ▾]» (abre a lista das dez formas, `TimelineSegmentEase`) · risco · Nearest / Custom (Bézier) / Rove Across Time (o que a maquete não tem; nenhuma função saiu). `geo()` é a conta única do desenho e do teste.
- **Cliques** pela mesma `TimelineInterpPick` (a shell não mudou): `key_card::acao` → Curve = forma acesa (ou Cubic) × modo aceso (ou In-Out); Ease x = forma acesa × modo x; Shape abre a lista no modo aceso; o fundo (`CTX_MENU_TL_KEY_CARD`, em `click_belongs_to_the_open_menu`) não fecha. Straight/Hold/linhas de baixo = os ids de sempre. Ids novos `CTX_MENU_TL_{KEY_CARD,CURVE,CURVE_IN,CURVE_INOUT,CURVE_OUT,SHAPE}` na tabela `TIMELINE_KEY_CARD` (em `ALL_TIMELINE_MENUS` ⇒ registados).
- **Estado e título** pelo canal do acréscimo 7: `set_menu_marks(kind, ids, titulo)` + `menu_title(kind)`; `menu_marcas` publica também a FORMA no menu da chave e o título («Key at frame {n} · {canal}», «Keys at frame {n}» no Summary; quadro = `round(t·fps)`); coluna mista ⇒ título sem marca.
- **Cores medidas na maquete:** fundo do bloco = o botão de cartão (`CardField`); escolhido `TimelineTinta::BlocoEscolhido` (novo, `#354C73` medido) com texto `TextoCabecalho`; não escolhido com texto `TextoPropriedade` (cinzento, como na maquete). Rótulos dos blocos/ease em peso NORMAL (o app pinta MEDIUM; a maquete é normal), título `paint_text_title` em `Base`.
- **Fotos:** [maquete × nosso](../fotos/31_cartao_da_chave_lado_a_lado.png) (depois de clicar Ease in: Curve + Ease in acesos, Cubic) · [chave Linear](../fotos/31_cartao_da_chave_straight.png).
- Testes novos `editor-core key_card::tests` (2: cada clique pede a mesma escolha do menu em lista; todo alvo cabe e nenhum se sobrepõe); `menu_marcas` atualizado (forma acesa, título, chave inexistente = título sem marca).
- Por nome ✓: panel-timeline menu_marcas 1, timeline_presets_menu 10, extrapolation 5 · editor-core key_card 2, menu_row 13, context_menu 31, every_menu_row 5, every_timeline_menu 3, timeline 45 · host-desktop timeline_presets 14 · i18n tests 10 · tokens timeline_tinta 2, contrast 10 · fmt ✓ · clippy `-D warnings` (editor-core, panel-timeline, tokens) ✓.
- Smoke do dono: pendente.

## Acréscimo 9 — dono 13/11: o cartão da chave no idioma dos cartões do Inspector; arrastar, fica aberto, Done

- **Pedidos** (verbatim): «Capriche no modal. Está fora do padrão do app. Veja nossos próprios cartões como estão belos» · «Permita arrastar o modal, mudar os parâmetros sem ele se fecha. Coloque um botão de concluir».
- **`key_card.rs` reescrito com as peças do Inspector** (dentro de `card_ink::com(ink, …)`): moldura de cartão (`CARD_RADIUS`, `KanbanCard`, sombra de lista), título `paint_text_title` a `CARD_TITLE_PX`, rótulos `card_field::paint_label` («Interpolation», «Ease», «Curve shape»), `panel_chrome::paint_segmented_group` (Interpolation = Curve·Straight·Hold·Nearest·Custom; Ease = In·In-out·Out), `paint_card_chip` para a forma, `paint_card_check` para «Rove across time», **Done** no acento `KanbanBlueBold`. Métricas de `ph2d_card_stack::metrics`. `CARD_W` 352 (fotografado: a 300 «Straight/Nearest/Custom» saíam com «…»). A `TimelineTinta::BlocoEscolhido` do acréscimo 8 saiu (sem uso: o segmentado escolhe em `KanbanCardRaised`, como o Inspector).
- **Fica aberto:** `Acao::Escolhe` já não fecha; as linhas de sempre (Straight, Hold, Nearest, Custom, Rove) também não, quando o cartão está ABERTO (`context_menu()`; com o pedido já fechado consome-se como antes — gate `timeline_segment_menu_parks_each_leaf_pick_for_the_shell`). `click_belongs_to_the_open_menu` passa a aceitar todo alvo do cartão (`key_card::e_alvo_do_cartao`): o aperto fora do menu fechava-o ANTES do clique (visto em foto). **Done** (`CTX_MENU_TL_KEY_DONE`, `Acao::Fecha`) fecha.
- **Forma:** «Cubic ▾» abre a lista das dez formas DEBAIXO do campo; escolher nela VOLTA ao cartão no mesmo sítio (`WidgetStore::set/take_key_card_volta`).
- **Arrastar pelo título:** `interaction/dispatch/key_card_drag.rs` (novo): Down no `CTX_MENU_TL_KEY_CARD_HEAD` arma a pega (cursor − canto pintado) e consome; Move reabre o pedido no sítio novo; Up solta (`pointer_up`). Campo `WidgetStore::key_card` (pega, volta). Fundo e título registados no `pre_populate`.
- **Fotos** (Xvfb, cliques e arrasto reais): [aberto](../fotos/32_cartao_da_chave_aberto.png) · [clicou In — continua aberto, Curve+In](../fotos/32_cartao_da_chave_clicou_in_continua_aberto.png) · [Bounce escolhido na lista, voltou ao cartão](../fotos/32_cartao_da_chave_bounce_voltou.png) · [arrastado para o meio da tela](../fotos/32_cartao_da_chave_arrastado.png) · [ao lado do cartão Interaction do Inspector](../fotos/32_cartao_ao_lado_do_inspector.png). Done fecha: fotografado (cartão some).
- Testes: `key_card::tests` 3 (cliques; geometria cabe/sem sobreposição/título no topo; Done fecha, título nada, todo alvo é do cartão). Por nome ✓: editor-core key_card 3, timeline 45, context_menu 31, menu_row 13, every_menu_row 5, every_timeline_menu 3 · panel-timeline menu_marcas 1, timeline_presets_menu 10 · host-desktop timeline_presets 14 · i18n tests 10 · fmt ✓ · clippy `-D warnings` (editor-core, panel-timeline) ✓.
- Smoke do dono: pendente.

## Acréscimo 10 — dono 13/11: a chave escolhida com o destaque da maquete

- **Pedido** (verbatim): «A seleção de keys é quase imperceptível. Na maquete temos belos destaques para keys selecionados». Antes: a escolhida no hover desbotava para creme (`cor(Hover)` mistura com `Chave`), e a coluna escolhida do Summary ficava âmbar com fio azul de 1,5 px.
- **Maquete medida** (`magick … %[pixel]`, linha de 22 px): chave azul `(88,156,255)` de meia-diagonal ~5 (= a branca), folga escura ~1,5, **anel** de ~1,5 px com meia-diagonal ~8,7 em `(56,93,151)` sobre `(20,23,38)` = o azul a 0,52. O laranja `(255,158,90)` (anel `(132,87,63)`, mesma geometria) é a tinta `Mudado`/`PecaContainer`; o sentido dele na maquete não está escrito em doc nenhum — **escolhida = azul** (`ChaveEscolhida`); laranja não usado.
- **`alca_estado::anel_da_escolhida`** (novo): losango vazado (traço, não preenchimento — a folga é o fundo de QUALQUER linha) em `acento × TIMELINE_KEY_SEL_ANEL_ALPHA`, cheio no hover (`cor_do_anel`). `pinta_chave` pinta-o para a escolhida (faixa e linha de grupo, via `chave_da_folha`) e o corpo da escolhida no hover fica azul. `summary_paint::paint_column`: coluna toda escolhida, nos cartões = anel + corpo azul + contorno `FitaPropriedade` (como a chave da faixa). `PH2D_CARDS=0` intocado.
- **Tokens** (`ph2d-tokens/chrome.rs`, escala da faixa de 30 px = ×1,36): `TIMELINE_KEY_SEL_ANEL_HALF_PX` 11 · `TIMELINE_KEY_SEL_ANEL_PX` 1,5 · `TIMELINE_KEY_SEL_ANEL_ALPHA` 0,55. Nenhuma tinta nova.
- **Fotos** (Xvfb, clique real): [antes](../fotos/33_chave_escolhida_antes.png) · [maquete](../fotos/33_chave_escolhida_maquete.png) · [depois — coluna do Summary escolhida](../fotos/33_chave_escolhida_depois.png) · [lado a lado](../fotos/33_chave_escolhida_lado_a_lado.png).
- Testes: `a_chave_escolhida_tem_o_azul_e_o_anel_da_maquete_em_todo_tema` (novo; `Theme::ALL` × fita propriedade/cabeçalho/Summary: corpo ≥ 3:1 WCAG, medido 3,78 no pior; anel composto ΔOK ≥ 0,15, medido 0,224). Por nome ✓ panel-timeline: alca_estado 12, summary 13, tracks 11, linhas 22, ruler 30, os_cartoes 36, chave 7, key 58, losango 6 · tokens: timeline_tinta 2, contrast 10, chrome 1 · fmt ✓ · clippy `-D warnings` (panel-timeline, tokens) ✓.
- Culling da chave continua por `chave_h()`: o anel (11 px) pode cortar 4 px antes na borda esquerda da folha — não visto em foto.
- Smoke do dono: OK 13/11 (acréscimos 7–10, «parece OK»).

## Acréscimo 11 — dono 13/11: o cartão da chave com as curvas desenhadas; a barra é a curva

- **Pedido** (verbatim): «O modal de easing é pobre e simples. Quero belíssimo, todas as escolhas em botões com as curvas de animação desenhadas neles. As linhas colorinhas entre frames devem também representar a curva de easing escolhida».
- **Cartão** (`key_card.rs`): os segmentados e o chip «Cubic ▾» saíram; toda escolha é um BOTÃO desenhado (`botao`, `TILE_H` 58 = o bloco da maquete 31): Interpolation 5 (Curve = forma×modo acesos · Straight diagonal · Hold salto no fim · Nearest salto a meio · Custom Bézier em S com as alças), Ease 3 (a forma acesa em In/In-out/Out), **as dez formas numa grade 5×2** no modo aceso (clique = `Escolhe{fam, modo}` direto, sem lista). Guias 0/1; o que passa (Back, Elastic) encolhe a escala para caber. Escolhido = anel `KanbanBlueBold` por preenchimento (`SEL_W` 1,5) sobre `KanbanCardRaised`, curva no acento; apagado = `CardField`/`CardFieldHover`, curva `KanbanCardFg`×0,72, rótulo `CardLabel`. Rove e Done numa fileira.
- **A régua é a do documento:** editor-core não conhece `ph2d-anim`; o painel publica `EaseSampler = fn(NodeId, u8, f32) -> Option<f32>` (`WidgetStore::set_ease_sampler`, em `menu_marcas::publica` → `amostra_ease` = `preset_for` → `Easing::eval`). Sem régua o botão de forma fica só com o rótulo (nunca uma curva inventada).
- **Saiu** (sem uso): `Acao::AbreFormas`, `CTX_MENU_TL_SHAPE`, `set/take_key_card_volta` e o ramo «voltar ao cartão» do `timeline_segment` (`TIMELINE_KEY_CARD` 6 → 5). O submenu de formas em lista continua para o menu em lista (`TimelineSegmentEase`).
- **Barra = curva** (`linhas_maquete::{paint_barras, pinta_curva, pontos_da_curva}`): da ponta de baixo da chave de saída à de cima da seguinte (`Faixa{cy, amp = chave_h(), lim}`), fração = `Interp::value(0,1,u)` (BezierW: `(value−v0)/dv`, parada = centro); desce quando o valor desce; passo 2 px (tecto 256); presa à linha. Hold continua tracejada e reta. A legenda pinta a mesma curva.
- **Fotos** (Xvfb, cliques reais): [cartão](../fotos/34_cartao_desenhado.png) · [na tela](../fotos/34_cartao_na_tela.png) · [Out + Bounce escolhidos, e a barra Bounce](../fotos/34_cartao_out_bounce_e_a_barra.png) · [a barra](../fotos/34_barra_bounce.png).
- Testes novos: `key_card::tests::each_button_draws_what_its_choice_does` · `menu_marcas::tests::the_card_buttons_draw_the_documents_own_easing` (30 forma×modo = `Easing::eval`) · `linhas::maquete::curva_tests::the_bar_between_two_keys_draws_the_chosen_easing`; `key_card` cliques/geometria atualizados. Por nome ✓: editor-core key_card 4, menu_row 13, every_timeline_menu 3, context_menu 31, timeline 45 · panel-timeline menu_marcas 2, linhas 23, timeline_presets_menu 10, extrapolation 5, os_cartoes 36 · host-desktop timeline_presets 14 · fmt ✓ · clippy `-D warnings --all-targets` (editor-core, panel-timeline) ✓.
- Smoke do dono: pendente.

## Aberto (visto, NÃO feito)

1. **Limite conhecido:** com o pai ANIMADO durante o clip, o duplo-clique na curva projeta pelo pai na 1.ª chave (desenho e âncoras são exatos por instante). Foto de cena com pai rodado não tirada.
2. `apply_intent` sem mundo converte com repouso `[0, 0]`; o produto usa `apply_intent_in` (a ponte da shell). Porta dupla latente — só testes/smokes sem mundo a usam.
3. `apply_intent_in` lê o repouso de `Transform.translation` (a pose amostrada); só vale para eixo sem track E sem `binding.rest`. Com dois clips em que o outro anima esse eixo, poderia herdar a pose do clip ativo — não reproduzido.
4. `path_to_separate` remove a track Position de todo clip, inclusive de um clip sem caminho resolvível (`path_for = None`) — essa track não desenha nada sem caminho, logo nada visível se perde; não há teste.
5. O pivô agora conta como gizmo no guarda do duplo-clique: duplo-clique no pivô passa a abrir o rótulo do objeto, como já fazia no corpo (consistente; não medido em smoke).
6. A cena `=1..6` passa a laçar o clip ativo inteiro (antes, `set_loop(0, 2)` só no relógio, que se perdia ao entrar na aba Keys).
7. O teste `bending_one_segment_keeps_the_shape_of_the_rest` (ex-`a_missing_x_key_is_born…`) perdeu o assert da chave X nascida a meio — no modelo A não nasce chave X; a régua é a forma (Hausdorff < 1e-2).
8. Integração NÃO feita (só por ordem do Enio).
