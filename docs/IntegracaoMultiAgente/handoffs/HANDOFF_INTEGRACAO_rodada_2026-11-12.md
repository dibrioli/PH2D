# HANDOFF — rodada de integração de 12/11 (6 linhas, SEM o Painter)

Leitor: a próxima LLM e o próximo integrador. Ramo `integ/2026-11-12b` (worktree `Worktrees/integ-2026-11-10`), main de partida `bc4a6ce930`, tip antes deste commit `dbe0fc749f`; tip final = HEAD do `main` depois do `--ff-only` (este commit é o último), CI = run do `spike.yml` desse push. Curas: `git log --oneline --reverse bc4a6ce930..integ/2026-11-12b --grep='integ 12/11'` (28 commits, `12/11` e `12/11b`). Estado da cadeia: `/home/enio/Documentos/Projetos/PH2D/.git/worktrees/integ-2026-11-10/integ-rodada/estado.tsv` (6 linhas, todas `verde`).

## Ordem, medida por sobreposição de ficheiros (sem o Painter)

| # | Linha | Commits | Handoff da linha |
|---|---|---|---|
| 1 | PixelArt | 4 | sem handoff novo: 4 curas do smoke P13 sobre o [P13+P14](../../PixelArt/handoffs/HANDOFF_INTEGRACAO_line_PixelArt_2026-11-09_P13_P14.md) já no main |
| 2 | ScriptEditor | 170 | [acabamento](../../CodeEditor/handoffs/HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-10_ACABAMENTO.md) |
| 3 | TimelinePrincipal | 49 | [plano 17 ondas 1–4](../../Timeline/handoffs/HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-11_PLANO17_ONDAS_1_A_4.md) |
| 4 | GameEngineFuncional | 31 | [linha inteira](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-11_LINHA.md) |
| 5 | InputMap | 244 | [onda 4](../../InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA4.md) |
| 6 | GameEngineFuncional02 | 63 | [item 5 e por tique](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-11_ITEM5_E_POR_TIQUE.md) |

SfxLab: sem commits, fora. Total no ramo: 589 commits (561 das linhas + 28 curas), 1 505 ficheiros (+84 539 / −11 635).

## PainterWatercolor EXCLUÍDA (decisão do coordenador)

Foi integrada primeiro (ramo de referência `integ/2026-11-12`, mantido local, NÃO publicado). A suíte deu ~45 vermelhos que JÁ vinham do tip da linha: leis de pintura mudadas pela missão 4K (alarmes de pincel, testes «ao byte», mistura espectral B32.1, papel Rough B33.1, grão do Impasto B29.3) — só o dono aprova, nunca regravar. A linha estava a meio (tip = prompt de continuação, trabalho «envio sem cópia» por comitar; voltou a compilar durante a integração). A linha rebaseia sobre o main novo e fecha ela.

Curas da referência que SÃO do Painter e esperam por ele (cherry-pick do `integ/2026-11-12`):
- `f8068064c7`: testes velhos da linha (base fixa nas camadas, documento da macro v3, portas internas, `mask_spacing_saved`, Oil Wash 57/46, chaves `card.painter.shape.*`).
- `bfa2f7a50d`: `wet_tuning` 14→12.
- `9bae88fdbb`: entrada `edita_textura` em `arch_mode_has_reconcile` (neste ramo a Texture reconcilia por `rerender_texture_layer`; `RECONCILES_VIA` aceita os dois verbos, `3fb9701b51`).
- `1b03870a56`: `perde_a_mao` (Focused/CursorLeft por uma só porta, chamando `on_focus_lost` do ScriptEditor). Sem o Painter, os dois eventos ficam nos braços atuais; quem trouxer a porta junta os dois.

Aberto do Painter para o fecho da linha: `h2_os_cartoes::the_top_and_the_basics…` (11 ids do WetPaint somem) · `cada_nome_deste_painel_cabe_na_coluna_da_seccao` («Concentration», «Edge Darkening (%)») · `a_memoria_do_dabber_cabe` (576 > 515 MiB) · `plane_fork…` · `the_shell_folds_the_window_the_tool_reports_dirty`.

## Contadores (DELTA)

- `PROJECT_SCHEMA` 231 → 237, tripla `(237, 13, 29)` (`shells/desktop/src/project_schema_tests.rs:378`). Degraus: 232 Timeline · 233–234 GEF · 235 InputMap · 236–237 GEF02. Reconta-se por `python3 scripts/schema-recount.py`.
- `CLAUDE.md` §5: `PH2D_GAME_SMOKE=1..23`, `PH2D_PHYSICS_SMOKE=120..122`, `PH2D_CODE_SMOKE=1..15`, `PH2D_NEST_SMOKE=1..5`; entrada nova «Input Map» (≤ 600 B). Tecto 40 KB.
- Cena 49 `physics_smoke_joint_break`: `ph2d-app-physics` → `ph2d-physics-smoke-scenes`.
- `SHAPE_LOOK` (`ph2d-component-desc`) ganha os campos 4..7 (largura, tracejado, traço/vão, traço próprio) que o catálogo já declarava.
- Tectos de LOC por corte: shell −179, `ph2d-app-components` −86, `fase_physics_overlay` 205→189, `fase_snapshots_publish` 201→194. Catraca widget→interaction 41→32. Catraca do Inspector 80→74 e 78→72 (medida).

## O que um merge pode partir

- **`ph2d-app-host` × InputMap:** o prefs importava `ph2d_app_input_map::View` (ciclo); `View` vem da folha `ph2d-input-map-editor`. `ph2d-app-host` liga também `ph2d-recovery` e `ph2d-tokens`.
- **Console do ScriptEditor:** `Store` desceu de `ph2d-app-code` para `ph2d-panel-code::console_store`; a entrega dos dispositivos ao script é `ScriptHost::feed_devices` (`crates/ph2d-script/src/devices_frame.rs`). Chamadores novos do `Store` apontam à crate nova.
- **Escuta do «Bind…» (GEF02 `43593d81` × InputMap):** a JANELA flutuante do Input Map caiu; a cura foi PORTADA para a aba `InputMapDoc::end_frame`, campo `Option<ActionId>`, teste `bind_on_the_triggers_listen_cancels_instead_of_rearming` (`ph2d-input-map-editor/src/doc_tests.rs`). `PedidoDoMapa::CriarEEscutar` (o `Criar` saiu).
- **`CursorLeft` do Controls:** um só braço (`on_cursor_left` da aba + `movimento_dos_controles`).
- **Arnês** da aba Input Map e do ScriptEditor preenchem `PointerInput.primary_held`.
- **Ids do Input Map** únicos: `LIST_SEARCH`/`LIST_ROW`/`KB_BASE`/`TRY_BASE` (o `ids::SEARCH` antigo reprova o `node_id_collisions`).
- **Lei do contorno** desceu para `ph2d-shape-handles/src/contorno.rs`; `row_badge.rs` novo; `pointer_look` recebe `em_cartoes`.
- **Player em cartões:** a coluna do rótulo mede-se UMA vez pela SECÇÃO inteira (`d2b986894b`); a régua da largura da linha lê a dobra, sem a moldura de cartão. Caixa de habilidade = marca nua por `rows::caixa_de_cabecalho`.
- `collision-surface.sh` tem sonda CEGA para os espelhos `ph2d-render`/`ph2d-script` do registo: `recontar.py` cobre.

## Suítes e curas

- #1–#2 com o Painter: pré-voo vermelho (fmt/clippy/typos/índice); depois 102 FAIL + 1 TIMEOUT → Painter excluído.
- 12b #3: pré-voo vermelho (fmt; 21× `ids::SEARCH` na game-scenes depois do rename).
- 12b #4: 35 572 testes, 9 FAIL + 1 TIMEOUT. 6 curados (`3fb9701b51`, `d2b986894b`, `b821655807`, `dbe0fc749f`…); 4 de CARGA (load ~200, verdes isolados 3/3): `playback_sampling_is_zero_alloc`, `the_cost_of_sampling_a_path_is_flat_in_its_anchors`, `the_cost_of_depth_is_linear_not_explosive`, `undo_vivo_grade_1_fluxo_3` (70–80 s sozinho, o TIMEOUT).

Curas por família (`integ 12/11`):
1. **Tectos de LOC por corte:** shell, components, funções da shell, `main.rs` (banner → `init::banner`). Arquivo verbatim: [COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-12.md](../../archive/integracao-jornadas/COMENTARIOS_CORTADOS_PARA_O_TECTO_2026-11-12.md).
2. **Composição:** `View` na folha (ciclo do host), `primary_held` nos arnês, Store do console, cena 49, `SHAPE_LOOK` 4..7, escuta do Bind portada.
3. **Gates:** molduras `FRAME-RAW-OK` na Timeline/Controls; lei do contorno; ids únicos do Input Map; catraca widget→interaction 41→32; `fase_bus_chrome` sem `?` no braço; `a_panel_still_wins_the_wheel` acha a chamada, não o comentário.
4. **Inspector:** ShapeLook no censo de escritores; px/m no pivô do Look; coluna do Player por secção; régua da dobra.
5. **Rótulos:** «Pure kinematic» no chip do Player; cortes nomeados no degrau estreito (rótulo que não cabe elide em vez de quebrar, `e6bbb754be`): `wet_tuning` 0→4 e `widget_gallery` 0→10 nomeados, oito rótulos do Wet Tuning em CORTADOS_HOJE.
6. **Tema:** hover neutro nos temas Plumber/Sunset (`ph2d-tokens/src/row_hover.rs`, croma da base > 0,06 → cinzento médio a alfa ×1,9).
7. **Pré-voo/texto:** fmt, typos, índices, §5 (`GAME_SMOKE` 1..23, `PHYSICS_SMOKE` 120..122).

## Para o OLHO DO DONO no smoke (nada aprovado ainda)

- **PixelArt:** repetir o smoke P13 (tween, som, CRT) — as 4 curas vieram dele.
- **ScriptEditor:** 4.ª + 5.ª rondas PENDENTES (handoff §6, 8–12): faixa Console/Problems a 900×560, linha longa, perder o foco larga a mão.
- **Timeline:** re-smoke do «Done» (`PH2D_ANIMACAO_SMOKE=1`), `PH2D_NEST_SMOKE=4` e `=2`, chip arrastado para a faixa.
- **GEF:** `PH2D_GAME_SMOKE=5` (Pivot/Align/Ctrl+Z) e `=4` (Character, Jump Arc, Dash). NOVO: o chip do Player diz «Pure kinematic» (a ficha pedia «Kinematic, pushes nothing», não cabe); a caixa de habilidade do Player não tem losango (exceção nomeada em `DIALOGO_SEM_COLUNA`).
- **Temas:** hover das linhas nos temas Plumber e Sunset ficou mais neutro — confirmar que ainda se lê.
- **InputMap:** smoke do dono OK em 11/11; repetir só se a escuta do Bind na aba parecer estranha.
- **GEF02:** onda 10 PENDENTE — `PH2D_GAME_SMOKE=1`, Play: a janela «PH2D · Game» abre na frente e o editor escurece com aviso; Wayland/KWin real não verificado.

## ABERTO

- **Painter** (acima): reintegrar a linha dele; as 4 curas e os 5 vermelhos esperam por ela.
- **Prova de mutação da cura portada** (`bind_on_the_triggers_listen_cancels_instead_of_rearming` / `InputMapDoc::end_frame`): FALTA.
- **Catraca do Inspector desceu (80→74, 78→72) sem atribuição** a uma linha.
- ScriptEditor: sem teste do caminho hero→`CodeDocs`; `despacho_clique_flip.rs` por verificar; oráculos 10/12 só com `bash -n`.
- Timeline: onda 5 (itens 11–13) NÃO começada, só por ordem do dono. GEF: `janela_do_jogo_smoke.rs` a mover da shell; 12 dobras sem dica. GEF02: `QUADROS_SEM_FICHA = 30` por medir.
- InputMap: Num Lock (f), Sprite «nada muda» (g), gamepad físico (h), token `selection` (i).
- Integrar não é aprovar.

## Smokes (do repo principal, depois do ship)

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run --profile smoke -p ph2d-host-desktop
cd /home/enio/Documentos/Projetos/PH2D && PH2D_GAME_SMOKE=1 cargo run --profile smoke -p ph2d-host-desktop
cd /home/enio/Documentos/Projetos/PH2D && PH2D_ANIMACAO_SMOKE=1 cargo run --profile smoke -p ph2d-host-desktop
```
Os smokes por linha estão nos handoffs da tabela; sem `PH2D_CARDS=0` no ambiente.

## Prova de fecho

B2 e `ship.sh` sobre a árvore com este handoff; CI = run do `spike.yml` desse push.
