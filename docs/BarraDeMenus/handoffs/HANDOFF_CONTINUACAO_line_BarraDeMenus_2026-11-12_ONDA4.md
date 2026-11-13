# HANDOFF de continuação — line/BarraDeMenus — onda 4 feita (12/11/2026)

Worktree `Worktrees/line-BarraDeMenus` · branch `line/BarraDeMenus` · base (merge-base com `main`) `4f46a3573c`.
Onda 3 (aprovada no smoke do dono, 12/11): [HANDOFF_..._ONDA3.md](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA3.md).
Plano: [01_plano.md](../01_plano.md) (onda 4 = «faixa única»).

## Onda 4 — FEITO: a faixa é o topo da janela
- **Omissão: sem a barra do sistema** (`shells/desktop/src/init.rs`: `with_decorations(!faixa_unica::ligada())`).
  `PH2D_FAIXA_UNICA=0` volta à barra do sistema (a faixa fica exactamente como na onda 3: sem botões).
- **A lei** em `crates/ph2d-editor-core/src/screens/hero/window_buttons.rs` (testes `window_buttons_tests.rs`, 5):
  - `rects(bar, frame)` — − □ × encostados à direita (`ids::MENUBAR_WIN_{MINIMIZE,MAXIMIZE,CLOSE}`), largura
    `altura + Spacing::Md`; maximizada troca □ por `IconId::WindowRestore`. `frame == None` (barra do SO) = nenhum.
    `menu_faixa::faixa(bar, ts, frame)` ganhou o 3.º argumento e o jogo/campo acabam antes dos botões (`Faixa::janela`).
  - `paint` — chip da faixa no hover; o × acende com `ColorToken::Danger` (no tema de cartões o perigo é o azul do
    «Don't Save»: padrão da casa, decidido de propósito, não é um defeito).
  - `on_click` → `store.request_window(WindowRequest)`; chamado em `chrome/menu_bar.rs::apply`.
  - `edge_at(x, y, w, h, maximized)` — borda `Spacing::Sm` (6 px), cantos `Spacing::Xl` (16 px) ao longo da borda;
    maximizada não tem bordas.
  - `WindowPress::press` / `press(hero, …)` — o Down primário: borda → `Resize(Edge)`; `MENUBAR_BACKDROP` → `Move`,
    o 2.º dentro de `DOUBLE_CLICK_WINDOW_NS` → `ToggleMaximize`; o resto é do editor. Estado em `FaixaState`.
  - `free_backdrop_x(hero, largura)` — o maior trecho livre do fundo (diagnóstico e alvo da prova no KWin).
- **Store** (`interaction/state/faixa_ops.rs`): `set_window_frame/window_frame` (`Some(maximizada)` = janela do app),
  `request_window/take_window_request`, `window_press_mut`.
- **Shell** `shells/desktop/src/faixa_unica.rs` (só o `winit`): `faixa_unica_down` no topo de
  `despacho_rato.rs::on_mouse_input` (Down esquerdo; `drag_resize_window`/`drag_window`/`set_maximized`; tomado = o
  editor não o vê); `faixa_unica_cursor` em 1.º lugar no `update_eyedropper_cursor` (setas de redimensionar);
  `faixa_unica_tick` no `RedrawRequested` depois do `render_frame` (publica `window_frame`, cumpre o pedido; o Close é o
  MESMO caminho do `CloseRequested`: jogo no palco pára primeiro, senão `on_close_request` → pergunta de gravar).
  `PH2D_FAIXA_LOG=1` imprime cada Down tomado e o fundo livre.
- Ícones novos `docs/design/icons/window-{maximize,minimize,restore}.svg`; i18n `chrome.menu_bar.win_*`; dicas e nomes
  (`topbar/tooltips.rs`, `topbar/chip_name.rs`).

## Prova no KWin 6.8.2 (Wayland nativo) — `bash docs/BarraDeMenus/ferramentas/kwin_faixa_unica.sh`
`kwin_wayland --virtual` dentro de `dbus-run-session` (D-Bus próprio; recusa o do dono e o socket dele), app como
cliente Wayland (`DISPLAY` removido), geometria por script do KWin (`QT_FORCE_STDERR_LOGGING=1` — sem isto o `print`
vai ao journal), rato falso `ferramentas/rato_falso/` (org_kde_kwin_fake_input, compilado na hora), fotos pelo
ScreenShot2 do D-Bus da sessão virtual (`ferramentas/foto_kwin.py`). Resultado gravado em
`docs/BarraDeMenus/fotos/onda4_kwin.txt` — **8/8**: abre sem barra (1024×768 = só o cliente) · arrasto +150+100 ·
canto SE +120 (+16 em altura: bateu no fundo da tela) · duplo clique maximiza 1600×1000 e restaura · □ maximiza e
restaura · − minimiza · × = «Save changes before closing?», igual ao × do sistema (`closeWindow` do KWin).
Fotos `fotos/onda4_{1_abre_sem_a_barra_do_sistema,4_maximizada,5_rato_sobre_o_fechar,8_fechar_pergunta}.png`.
⛔ **Armadilhas medidas:** (1) a tela virtual NÃO tem rato; cada ligação do rato falso cria e apaga o ponteiro do
assento, e o `drag_window` do winit só age sobre ponteiros que ENTRARAM na janela (`apply_on_pointer`) — com um rato
por passo o arrasto saía sem `xdg_toplevel.move` nenhum. ⇒ UM rato a sessão inteira, por fifo. (2) Com Down e Up no
mesmo lote o `latest_button_serial` é o do Up e o KWin recusa o move — um clique «humano» segura ≥ 60 ms.

## Testes (verdes por nome) e curas de dívida das ondas 2–3
editor-core `architecture_` 122 · `every_` 173 · `menu_` 132 · `window_buttons` 5 · `enum_order_matches_svgs` ·
i18n `known_keys`/`as_chaves` · registry-init `the_app_frame_is_reachable` · host-desktop `keyboard`. fmt e clippy
(`-D warnings`, editor-core + host-desktop, all-targets) limpos.
- A lógica pura (`WindowRequest`, `Edge`, `edge_at`, `WindowPress`) mora em `interaction/state/window_frame.rs` e o
  `window_buttons` reexporta-a: no hero ela levava `interaction → screens` a 23 contra o tecto 18
  (`the_foundation_modules_form_a_dag`).
- Dívidas das ondas 2–3 que o 1.º fecho apanhou: `chrome.palette.menus` órfã saiu da tabela; as 5 chaves do
  `menu-memory.txt` em `NOT_LANGUAGE_TEXT` (formato do ficheiro); `command_palette/lista.rs` em `NOT_THIS_QUESTION` do
  ritmo (linhas de menu encostadas, como `tracks_lista.rs`); `transport_bar.rs` em `NOT_THIS_QUESTION` do vão
  ícone→rótulo (a seta do palco não tem rótulo); clippy em `dock_width_ops.rs:428`, `global_palette_menus.rs:18`,
  `global_palette_finds_tests.rs` (hex, lifetime) e `global_palette_tests.rs` (import).

## Aberto
- **Caneta (zwp_tablet_v2):** os botões − □ × funcionam (Click do editor) e o duplo clique no fundo maximiza
  (`input_drop/caneta.rs` → `faixa_unica_down(x, y, caneta = true)`). Arrastar e redimensionar NÃO: a caneta é drenada
  no topo do quadro, fora do evento do winit, e o `drag_window`/`drag_resize_window` do winit 0.30 só manda o serial do
  `wl_pointer` (`apply_on_pointer`) — o KWin exige o serial do Down que segura. Esses Downs seguem para o editor (a
  borda de 6 px não come cliques da caneta). Cura possível: `xdg_toplevel.move` com o serial do `zwp_tablet_tool_v2.down`
  pela própria porta da caneta — o winit não expõe o `xdg_toplevel`. Não testável na tela virtual (sem tablete).
  O Meta+arrastar do KWin move qualquer janela com o rato (com a caneta: não conferido).
- Sem fotos X11 desta onda (as da tela virtual Wayland bastam; o `fotografa_cena.sh` fotografa só o cliente).
- SEM provas de mutação das ondas 2–4.

## PRÓXIMO
Smoke do dono da onda 4.
