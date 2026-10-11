# CONTINUAÇÃO — `line/MiroClone` W14: depois do botão «Forma», soltar ficheiros no Wayland (2026-10-11)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. As regras R1–R9 da [continuação RÁPIDA](HANDOFF_CONTINUACAO_RAPIDA_FECHO_2026-10-11.md) valem iguais.
> Antecessor: [W13 acabamentos](HANDOFF_CONTINUACAO_W13_ACABAMENTOS_2026-10-11.md) (o ciclo que o dono escolheu: «Forma» + arrastar imagens).

## Estado

- Por integrar (não integre, não pushe): W12+W13 ([handoff](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md), adenda §12)
  **+ `935aed878`** (o botão «Forma», abaixo). Base = `main` `8f20c60de`. O handoff de INTEGRAÇÃO desta onda escreve-se no
  fecho do ciclo (as três partes) e inclui `935aed878` + este doc.
- Binário `--profile smoke` compilado com `935aed878` (`Finished … in 50.97s`). Smoke do «Forma» (cena 2): **OK do dono (11/10)**.

## Parte 1 — FECHADA: o botão «Forma» (`935aed878`, não reconstruir)

| o quê | onde | gate |
|---|---|---|
| `set_shape_type` nunca toca/faz nota; só escreve o que muda | `ph2d-board-edit/src/lib.rs:579` | `notes_tests::reshaping_turns_the_shapes_in_one_step_and_never_touches_or_makes_a_note` |
| `Item::Reshape` no 1.º grupo de `Selected::Shapes`; escondido com nota; cara = 1.ª forma; id no FIM de `items()` | `board_bar.rs`, `board_bar_shapes.rs` | `board_bar_shapes_tests.rs` (4, clique real) |
| a grelha (6 col., por baixo do botão) fecha: ao escolher · outro clique · selecção/ferramenta diferente · barra some (arrasto) · «mais formas» | `board_bar_shapes.rs` `toggle`/`sync`; estado `BoardLive::reshape` (`documents.rs`) | idem |
| corte por assunto: `paint_item` → `board_bar_look.rs`; `board_bar.rs` 686 → 550 | | `architecture_workspace_file_loc_cap` ✓ |

Mutação 10 → 7 + 3: M3 equivalente (`History::record` já não grava lote vazio — a guarda saiu), M4 (não reescrever o que já
é do tipo) e M10 (a barra some ⇒ fecha) ganharam gate e sangram. Portão: nextest de board-edit/i18n/app-board/editor-core
2 443/2 443, `nextest-impacted` 17 965/17 965 (302 s, load 106), clippy `-D warnings`, fmt, typos ✓. Foto da cena 2 com o
botão: `docs/MiroClone/ferramentas/cena2_forma/cena2_botao_forma.png`. Nenhuma cena nova: a cena 2 abre com o losango
seleccionado e o botão à vista.

⛔ Lembrete medido: o arnês `board_bar_tests::T` carimba todo clique no instante 0 — o MESMO botão duas vezes seguidas
vira `DoubleClick` e o `click_bar` falha. Intercale outro botão.

## Parte 2 — A FAZER: soltar ficheiros na janela, no Wayland do dono

Pesquisa feita nesta janela (agente, fontes abaixo). Decisão (técnica, delegada — padrão-ouro): **(b) um receptor de DnD
ao lado do winit**, numa drop-crate nova (ex.: `ph2d-wayland-dnd`, `cfg(target_os = "linux")`).

- **(a) subir o winit — NÃO agora:** só a `0.31.0-beta.3` (04/09/2026) tem DnD no Wayland (`winit-wayland/src/dnd.rs`,
  `DragEntered/DragPosition/DragDropped/DragLeft` + `fetch_data_transfer` assíncrono); é beta e quebra 128 sítios
  `winit::` em 40 ficheiros (shell, `ph2d-app-flip`, `ph2d-probe-cursor-grab`). Nada além do nosso `"0.30"` segura o winit.
- **(c) forçar XWayland — NÃO:** funciona (`EventLoopBuilderExtX11::with_x11`) mas leva a app inteira para X11 (escala
  fraccionada borrada no KDE, tablet e IME piores).
- **(b) o caminho:** o padrão do `smithay-clipboard` — `Backend::from_foreign_display(WaylandDisplayHandle.display)` →
  `Connection::from_backend` → fila PRÓPRIA (`registry_queue_init`), `DataDeviceManagerState` (v1..=3) + `SeatState`,
  `get_data_device(seat)`. O winit 0.30.13 liga `wayland-backend/client_system` (`from_foreign_display` existe) e o
  `smithay-client-toolkit 0.19.2` já está no lock (`data_device_manager/data_device.rs:42-77`, `data_offer.rs:89-163`). O
  KWin entrega enter/motion/drop a TODOS os data devices do cliente nesse seat (`kwin/src/wayland/seat.cpp`
  `dataDevicesForSurface`); o winit 0.30 não tem nenhum ⇒ o nosso é o único.
  - No `enter`: comparar a superfície da oferta com a da janela (`ObjectId::from_ptr(WlSurface::interface(),
    WaylandWindowHandle.surface)`); `accept_mime_type(serial, Some("text/uri-list"))` + `set_actions(Copy, Copy)` (v3) —
    sem isto o KWin nunca manda o drop. A posição vem do `motion` (melhor que o `last_cursor` de hoje).
  - No `drop`: `receive("text/uri-list")` → `ReadPipe` NÃO bloqueante até EOF → `finish()` (v3) → destruir a oferta.
    ⛔ guardar a superfície do drop: o `leave` limpa o estado antes de o drop ser processado (Blender rB00aa57594c74).
  - Lista: linhas com `\r\n` OU `\n` (Qt/Dolphin mandam `\n`; Blender rB727cc426bc72), saltar `#`, percent-decode
    `file://`, largar o host.
  - Bombear a fila a cada quadro: `prepare_read()` → `read()` → `dispatch_pending()` + `flush()` (nunca
    `blocking_dispatch` no fio da UI) — ou um fio próprio como o smithay-clipboard.
  - Entrega: empurrar os caminhos para a MESMA fila de hoje (`main.rs:981` `on_dropped_file` → `pending_drops` →
    `input_drop.rs:22 handle_dropped_files`), com a posição do drop.
- **E o destino pela aba activa** (o 3. da W13): com um quadro activo, `handle_dropped_files` hoje manda para a CENA
  escondida — tem de perguntar `documents.active()`; no quadro, até haver a parte 3, a recusa vê-se na tela (nunca só
  `eprintln!`).
- **Gates:** sem compositor — o parser da uri-list (as duas quebras, `#`, percent, host) e a máquina enter → motion →
  drop → finish com a superfície guardada; o despacho por aba pela porta real (`pending_drops` → `handle_dropped_files`).
  Com compositor: `kwin_wayland --virtual` com `KWIN_WAYLAND_NO_PERMISSION_CHECKS=1` libera `org_kde_kwin_fake_input`;
  um pequeno cliente sctk que mapeia uma superfície, carrega pelo fake_input, `start_drag` com `text/uri-list`, move-se
  para a app e larga. ⛔ Nunca `ydotool` na sessão do dono (move o rato REAL); nunca `spectacle` (`import -window`).
  Instalados: kwin 6.7.5, wtype, wl-clipboard, xdotool, ydotool, libei; NÃO: ripdrag, dragon-drop.

## Parte 3 — depois: a IMAGEM no quadro

O plano do [antecessor](HANDOFF_CONTINUACAO_W13_ACABAMENTOS_2026-10-11.md) §«E no MESMO ciclo» 2. vale inteiro. Achado
desta janela para o «colar»: a shell usa `arboard 3.6.1` (com `image-data`) SEM `wayland-data-control` — fala com o
clipboard X11 via XWayland, que o KWin faz ponte; `get_image()` lê `image/png` (`arboard-3.6.1/src/platform/linux/x11.rs:1024`),
mas a shell só chama `get_text()` (`shells/desktop/src/forwarding.rs:230`). Nativo: a feature `wayland-data-control`
(`wl-clipboard-rs 0.9.3` já no registo) ou a seleção pela mesma ligação do receptor.

Fontes: [changelog winit 0.31](https://raw.githubusercontent.com/rust-windowing/winit/master/winit/src/changelog/v0.31.md) ·
[KWin seat.cpp](https://invent.kde.org/plasma/kwin/-/raw/master/src/wayland/seat.cpp) ·
[smithay-clipboard](https://raw.githubusercontent.com/Smithay/smithay-clipboard/master/src/lib.rs) ·
[Blender: quebras](https://developer.blender.org/rB727cc426bc723c8757941ce9bf23745ee5f6ab9d) ·
[Blender: superfície do drop](https://developer.blender.org/rB00aa57594c74c69d53ba326e48d2d1339ab99b75)
