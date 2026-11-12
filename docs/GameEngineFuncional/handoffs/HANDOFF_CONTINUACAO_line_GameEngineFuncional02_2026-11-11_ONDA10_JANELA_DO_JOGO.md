# Onda 10 — a janela do jogo vem à frente (GEF02, 11/11)

Mecanismo da onda; o resumo e o ABERTO estão no [handoff de integração](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-11_ITEM5_E_POR_TIQUE.md).

## «app travou» não era travamento

Relato do dono 11/11: «app travou» ao simular (3.ª corrida, depois de criar uma forma de colisão para uma imagem). Diagnóstico (perf no pid vivo, símbolos casados com um gémeo sem strip): **não** é travamento e **não** é a forma de colisão (nenhum traçado de contorno em 10 s de amostras; física < 1 %). O Run/Play com o palco omissão «Window» abre o jogo numa janela própria de 1280×720 que, em KWin/Wayland, abriu ATRÁS do editor maximizado (`focus_window()` sem ficha xdg-activation é ignorado); `da_outra_janela` (`shells/desktop/src/janela_do_jogo.rs`) descarta todo evento do editor menos `CloseRequested` ⇒ editor com cara de congelado; o X parava o jogo. Pré-existente desde `bc3bf2ea58` (25/10), não desta linha.

Decisão do dono 11/11: **as duas** — (1) a janela do jogo sempre à frente, (2) aviso sobre o editor parado.

- `crates/ph2d-editor-core/src/game_stage.rs`: decisão pura `passo_da_janela` (pede ficha → espera → abre com ficha, fecha, cancela; o editor pinta UM quadro com o aviso antes de a superfície mover-se; sem ficha após `QUADROS_SEM_FICHA = 30` quadros abre sem ela — número RACIOCINADO, **não medido**), `aviso_da_janela`, `reactiva_a_janela`. Testes `game_stage_tests.rs` (4): código primeiro, não nasceram vermelhos; 7/7 mutantes vermelhos, ficheiro restaurado byte-idêntico.
- `screens/hero/aviso_do_jogo.rs` (novo): BgScrim + cartão centrado, chamado no fim de `paint_hero_screen` quando `ViewState.aviso_da_janela_do_jogo`. i18n `shell.janela_do_jogo.running` («Game running in its own window — close it to stop»).
- Shell `janela_do_jogo.rs`: `palco_janela` roda a decisão; `pede_a_ficha` (`WindowExtStartupNotify::request_activation_token` na janela do editor); `da_outra_janela` recebe `ActivationTokenDone` e, em clique/tecla no editor, chama `request_user_attention(Critical)` na janela do jogo (winit 0.30.13 não reativa janela existente com ficha). X11: `focus_window` + urgência. Fora do Linux: sem ficha. `undo.rs`: campo `pedido_da_janela`. Líquido da shell −1 linha (cortes de comentário em `main.rs` e `fase_game_camera.rs`).
- Gates: `the_shell_only_shrinks` verde, `no_magic_numeric` verde, clippy `-D warnings` editor-core/i18n/host-desktop limpo, rustfmt limpo. A fiação da shell **não tem teste headless**.
- Foto: [01_editor_com_o_aviso.png](../smoke/onda10/01_editor_com_o_aviso.png) (`PH2D_GAME_SMOKE=1 PH2D_GAME_SMOKE_DRIVE=window`).

