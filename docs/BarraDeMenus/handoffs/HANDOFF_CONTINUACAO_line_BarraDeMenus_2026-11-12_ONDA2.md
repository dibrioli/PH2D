# HANDOFF de continuação — line/BarraDeMenus — onda 2 feita (12/11/2026)

Worktree `Worktrees/line-BarraDeMenus` · branch `line/BarraDeMenus` · base (merge-base com `main`) `4f46a3573c`.
Commits da onda: `f7f6164939` (produto + testes) · `f68b8985f7` (fotos + script). Onda 1: ver
[HANDOFF_..._ONDA1.md](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA1.md) (smoke do dono: OK).
Plano: [01_plano.md](../01_plano.md). Memória: `project-memory/project_menu_bar_version_c_2026_11_12.md`.

## Onda 2 — FEITO
- **Faixa** (`screens/hero/menu_faixa.rs`, porta `faixa(bar, ts)`): «Menu» à esquerda · abas de documento · campo
  «Find anything … Ctrl+K» no centro (máx. 2×`MENU_W`; vira lupa < `MENU_W`, ~600 px de janela) · jogo à direita.
  `menu_bar::menu_rects`/`pinta_titulo` saíram; `menu_bar.rs` guarda as tabelas e o pintor.
- **Cascata de verdade**: `interaction/state/menu_chain_ops.rs` (`MenuChain`: os pais ficam; `context_menu` continua a
  ser a PONTA, pais órfãos se outra porta escreve o slot) + `screens/hero/menu_cascade.rs` (`opens_submenu` = porta
  única do «›»; pairar re-toca o `Click` da linha; mira em triângulo; ↑↓→←Enter; `Alt` sozinho abre/fecha, desarma
  com outra tecla ou ponteiro a mexer). A shell entrega as teclas por `menu_cascade::shell_key`
  (`shells/desktop/src/input_dispatch/keyboard.rs`), porque o mapa do editor não tem `Alt` e só passa o Down.
  `ContextMenuKind::MenuBarRoot` (5 linhas, todas em `CASCADES`) e `MenuBarPlay`. As 10 cascatas das Preferências
  abrem por `menu_cascade::open_cascade` (empilham).
- **Busca em LISTA** (só a global; as outras paletas em nuvem): `PaletteModel.layout = List{..}`,
  `widget/command_palette/lista.rs`; caminho por linha derivado das tabelas (`global_palette_menus.rs`), atalho em
  coluna (`shortcut_column`, dica do chip para as ferramentas), Recent (5) em `global_palette_session.rs` (memória da
  sessão), presa por baixo da faixa centrada no campo, rodapé «Enter to run · Up and Down to move · Esc to close».
  Um id da barra leva o caminho da barra e aparece uma vez; «This tab» só com os comandos da aba ativa.
  O campo e *Help ▸ Commands…* abrem a busca e pedem à shell os grupos das famílias (`request_global_palette`).
- **Play com nome** + seta ▾ (`MENUBAR_PLAY_MENU` → Editor · Window · Fullscreen, marca no palco lembrado,
  `ModuleTruth::Stage`); Pause, Stop, Controls. `transport_bar::rects(bar, de, ate, play_w)` encosta à direita.
- Linha de menu acesa não acende o botão gémeo fora do menu (porta no store, para não subir a catraca do DAG);
  a dica cala-se com a busca aberta.

## Fotos (Xvfb, `bash docs/BarraDeMenus/ferramentas/fotos_onda2.sh`) — `docs/BarraDeMenus/fotos/onda2_*.png`
Todas conferidas: cascata a 3 e 4 níveis só a pairar, Alt + setas, busca vazia e «exp» (9 of 166), Recent, seta do Play,
janela estreita. Binário de smoke compilado no HEAD `f68b8985f7`.

## Testes (verdes por nome)
`menu_cascade` 10 · `menu_faixa` 3 · `transport_bar` 17 · `menu_bar` 18 · `command_palette` 46 · `global_palette` 18 ·
`context_menu` 31 · `hover` 38 · `tooltip` 8 · `barras_cartoes` 10 · `the_menu_bar_relocates_the_verbs_it_shows` 15 ·
`architecture_the_foundation_modules_form_a_dag` · `architecture_the_shell_only_shrinks` · registry-init (`the_app_frame_…`,
`the_area_hands_…`, `global_palette_catalog`, `a_paleta_fecha_ao_escolher`, os do Pixel) · `onda3_hover` · game-scenes
`a_cena_p5_…` · app-code `paleta`/`onda1` · i18n censos · shell `keyboard`/`alt`.
`global_palette_tests.rs` partido em `global_palette_list_tests.rs` (tecto 700).

## Smoke do dono (12/11)
Tudo OK salvo o passo 3 (a diagonal até ao submenu trocava de menu). Curado em `778bb14824`: o vértice da mira é a
posição de 4 amostras atrás (o rato real anda aos degraus de 1 px; um degrau (0, 1) caía fora do triângulo), folga de
duas linhas, e o ponteiro parado na irmã troca após `Duration::Default` (240 ms) pelo `tick_motion`. Teste
`a_diagonal_para_o_filho_nao_troca_de_submenu` com degraus de 1 px; mutação `RASTO=1` reprova. **Re-smoke do passo 3 pendente.**

## Aberto / decisões
- Busca vazia: Tools e ~30 painéis vêm antes dos comandos dos menus (ordem herdada). Candidato: menus primeiro.
- Nomes de painel que diferem do título da aba: «Pixel Prefs» (Grid & Background), «Pixel Keys» (Keyboard Shortcuts),
  «Wet Tuning» (Wet Paint) — pergunta feita ao dono.
- O campo da faixa abre a busca num cartão por baixo; digitar no PRÓPRIO campo da faixa ficou por fazer.
- Sem prova de mutação dos testes novos. fmt feito; clippy/fecho por correr (`PH2D_SUITE=fecho`).
- Módulo `layout_tabs` morto (desde a onda 1).

## PRÓXIMO — onda 3 (plano)
Busca que responde: ações do Input Map com teclas, objetos da cena, campos dos cartões; «Alt+Enter: show me where it lives»
(abrir a cascata — `menu_chain_ops`/`open_cascade` já servem — e acender a linha); preferências por utilizador → contador
«3.ª vez pelo menu» e o chip que ensina o atalho; Open Recent; Recent persistido. Onda 4: faixa única (sem barra do SO), testada no KWin.
