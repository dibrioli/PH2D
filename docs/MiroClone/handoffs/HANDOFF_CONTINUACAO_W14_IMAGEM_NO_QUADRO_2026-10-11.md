# CONTINUAÇÃO — `line/MiroClone` W14: soltar no Wayland FECHADO; falta a IMAGEM no quadro (2026-10-11)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. R1–R9 da [continuação RÁPIDA](HANDOFF_CONTINUACAO_RAPIDA_FECHO_2026-10-11.md) valem iguais.
> Antecessor: [W14 soltar no Wayland](HANDOFF_CONTINUACAO_W14_SOLTAR_NO_WAYLAND_2026-10-11.md) (Partes 1 e 2; a 3 é esta).

## Estado

- Por integrar (não integre, não pushe): W12+W13 ([handoff](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md)),
  `935aed878` (botão «Forma», smoke OK do dono) e **`f8ee5309b`** (soltar no Wayland, abaixo). Base = `main` `8f20c60de`.
  O handoff de INTEGRAÇÃO desta onda escreve-se no fecho da Parte 3 e inclui os três.
- Binário `--profile smoke` compilado com `f8ee5309b` (2.ª corrida: `Finished … in 0.29s`, zero `Compiling`). Smoke do
  dono da Parte 2: **por fazer** (passos no fim).
- O gate de confinamento do `unsafe` passou a varrer todo alvo (`src`, `tests`, `benches`, `examples`): vermelho com
  o `tests/kwin_drop.rs` antes da cura, verde depois — sem prova de mutação formal.

## Parte 2 — FECHADA: soltar ficheiros no Wayland (não reconstruir)

| o quê | onde | gate |
|---|---|---|
| `text/uri-list`: `\r\n` e `\n`, `#`, percent (bytes, não-UTF-8 ok), host largado, NUL final, contagem do que não é ficheiro | `crates/ph2d-wayland-dnd/src/uri_list.rs` | `uri_list_tests` (8) |
| a sessão pura enter → motion → drop → finish; UMA leitura por oferta (na entrada, para o aviso ter nomes); `leave` depois do `drop` não cancela (Blender rB00aa57594c74); `enter` novo fecha o anterior | `src/session.rs` | `session_tests` (8) |
| o receptor: a `wl_display` do winit por `from_foreign_display`, fila e fio próprios, `poll` sobre ligação + fio de parar + leituras (nunca `blocking_dispatch`); `accept` + `set_actions(Copy)` (sem isso o KWin não manda `drop`); ponta de leitura não bloqueante (o `O_NONBLOCK` da de escrita partilhar-se-ia com a fonte) | `src/receiver.rs` | `tests/kwin_drop.rs` (KWin real) |
| a shell: `attach` no `resumed`, `drain` no início do quadro (`fase_input_and_drops`) para as MESMAS portas do winit (`on_hovered_file`/`on_dropped_file`), pixels lógicos × `scale_factor`; largado no `exiting` (antes da ligação do winit) | `shells/desktop/src/input_drop.rs` | `no_app.sh cena` (foto) |
| destino pela aba: com um quadro activo `handle_dropped_files` recusa NA TELA («Boards don't take files yet») — antes ia para a cena escondida | `input_drop.rs` | `no_app.sh quadro PH2D_BOARD_SMOKE=2` (foto, 0 sprites) |
| lista sem ficheiro (um `https://`): aviso «Skipped {count}: not a file» | `input_drop.rs` | `session_tests::a_list_with_no_local_file…` (o aviso em si sem foto) |
| `unsafe` confinado: 3.ª excepção nomeada (`src/receiver.rs`, `tests/kwin_drop.rs`), `unsafe_code = "deny"` | `architecture_every_member_inherits_the_workspace_lints.rs` | o próprio gate |

**Com compositor:** `bash docs/MiroClone/ferramentas/soltar_wayland/corre.sh` (exit 0): `kwin_wayland --virtual` num
D-Bus próprio, duas superfícies `layer-shell` (alvo a ecrã inteiro, origem 64×64 no canto), rato =
`org_kde_kwin_fake_input`. Prova: o `Hover` nomeia os dois ficheiros (acento e espaço), sair dá `Leave`, largar dá os
dois na posição EXACTA, a origem recebe `dnd_finished` e UM `send`. **App inteiro:** `no_app.sh <nome> [CHAVE=VALOR]`
(o app nativo no Wayland virtual, maximizado por regra; a origem é o teste `drag_the_env_files_onto_the_screen_centre`;
fotos pelo ScreenShot2 da sessão virtual, `foto_kwin.py`). Fotos: `docs/MiroClone/ferramentas/soltar_wayland/*_depois.png`.

### ⛔ Recusas e armadilhas MEDIDAS nesta janela

| o quê | medido |
|---|---|
| KWin virtual SEM `--no-lockscreen` | engole TODO o rato falso, calado: dispositivo criado e «authorized», zero `wl_pointer` em 5 s |
| `commit` com buffer feito DENTRO do `configure` | não sai sem outra volta da fila: zero `wl_surface.enter` até um `roundtrip` a mais |
| foto do aviso 3 s depois de largar | o aviso já sumiu; a 0,25–1,5 s ele está lá (rajada de 6 fotos) — `no_app.sh` fotografa a 0,5 s |
| `sctk::DragSource::set_actions` | chama `set_actions` DUAS vezes na v3 e uma na v<3 (erro de protocolo) — use `inner().set_actions` |
| aviso longo no toast | «A board doesn't take dropped files yet — o…» cortado; ≤ ~36 caracteres cabem |
| aviso «queue destroyed while proxies still attached» | é das superfícies do TESTE (marcado depois de o receptor largar); o receptor destrói as ofertas antes da fila |

## Parte 3 — A FAZER: a IMAGEM no quadro

O plano do [antecessor W13](HANDOFF_CONTINUACAO_W13_ACABAMENTOS_2026-10-11.md) §«E no MESMO ciclo» 2. vale inteiro,
e o achado do «colar» do [W14](HANDOFF_CONTINUACAO_W14_SOLTAR_NO_WAYLAND_2026-10-11.md) §Parte 3 também (`arboard`
sem `wayland-data-control`; `get_image()` existe, a shell só chama `get_text()` em `forwarding.rs:230`). Ao fechar:
- o desvio de `handle_dropped_files` (o `if hero.documents.active().is_some()` no topo) passa a mandar as IMAGENS
  ao quadro na posição do largar e a recusar só o resto — a chave `board_takes_no_files_yet` muda de frase ou sai
  (censo do i18n);
- `no_app.sh quadro PH2D_BOARD_SMOKE=2` passa a fotografar a imagem no quadro (é o gate visual dela).

## Smoke do dono da Parte 2 (dar com a Parte 3 ou já)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && ./target/smoke/ph2d-host-desktop`
2. No Dolphin, arraste uma imagem `.png` para o meio da tela do PH2D (aba «Scene») e solte.
3. Ao passar por cima aparece o aviso com o nome; ao soltar, a imagem fica onde soltou, selecionada, e aparece «Imported …».
4. Errado: nada acontece ao soltar, ou a imagem aparece longe do ponto. Com um quadro aberto (aba «Board»), soltar
   mostra «Boards don't take files yet» e não cria nada.
