# CONTINUAÇÃO — `line/MiroClone` W17: os defeitos conhecidos das imagens (2026-10-12)

> **FECHADO — ver [HANDOFF_INTEGRACAO_line_MiroClone_W17_2026-10-12.md](HANDOFF_INTEGRACAO_line_MiroClone_W17_2026-10-12.md).**

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: a W16 FECHOU com **smoke OK do dono (12/10)** —
> [handoff de integração W16](HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md) (integra-se JUNTO com W15, W14 e
> W12+W13; **não integre, não pushe**: integrar é só por ordem do dono). Ordem do dono (12/10): *«vamos primeiro corrigir
> os defeitos conhecidos»* — antes de integrar.

## O que já existe e NÃO se reconstrói

Tudo o que o handoff W16 §1 lista: recorte pelo gesto (`ph2d-board-edit/src/crop.rs`), espelho, `ExportTarget` +
`DrawOp::Embed`, GIF (`ph2d-board-render/src/anim.rs`), colar com prazo (`ph2d-app-host/src/clipboard.rs`), «Send to
board…» da selecção com a pose (`ph2d-app-board/src/send_to_board.rs` + `shells/desktop/src/render_loop/fase_send_to_board.rs`),
a leitura do Flip por ladrilhos, a cena 17. O oráculo Excalidraw 0.18.1 está instalado (`ferramentas/excalidraw_oracle/`,
README §W16 tem a sonda das imagens — reutilize-a).

## Os cinco defeitos, num bloco só (§0.10) — confirme cada um no CÓDIGO e reproduza-o com um teste VERMELHO antes de curar

1. **«Left out: , X»** — um objecto SEM nome entra no aviso como vazio. O nome vem de `ph2d_ecs::Name` na fase da shell;
   a lista junta-se em `deliver` (`send_to_board.rs`). Cura: um nome de recurso (chave i18n nova, ex. «an unnamed
   picture/shape/drawing» pelo tipo) — e o caso de UM só objecto sem nome também (as frases `board.send.empty` etc. têm `{name}`).
2. **Objecto apagado entre o pedido e a escolha sai em silêncio** — o `continue` em `fase_send_to_board.rs` (o laço dos
   alvos). Cura: entra como `Sent::missing(…, "board.send.gone")` e conta no aviso; se TODOS sumiram, o aviso é o `gone`
   (hoje cairia no `wrong_kind`, que engana). A lógica vai para a família (a shell só compõe; tecto da shell).
3. **Imagem com o cabeçalho bom e os dados estragados fica cinzenta PARA SEMPRE, sem aviso** — a porta de entrada
   (`ph2d-app-board/src/images.rs` `ingest`) lê só o TAMANHO do cabeçalho (de propósito: o fio da janela não descodifica);
   o desenho falha-a fora do fio e marca `failed` (`ph2d-board-render/src/image.rs`), e o cinzento do «a chegar» fica.
   ⛔ Recusa da W14 que segue de pé: descodificar no fio da janela. ⇒ Corra o ORÁCULO primeiro: o que o Excalidraw mostra
   para um PNG truncado / JPEG com dados lixo (gere as entradas nossas; grave a saída em `saidas/w17/`)? Depois a lei: (a)
   o desenho distingue «a chegar» de «ilegível» (um ícone/estado próprio, `failed` → outro placeholder) e (b) quem pôs a
   imagem é avisado na tela (uma recusa que só o terminal vê é botão mudo) — p.ex. o `Images` expõe os ficheiros que
   falharam e a vista do quadro mostra um toast uma vez por ficheiro. O export já a omite (confirme). Ao vivo: o par que
   recebe também deve ver o estado «ilegível».
4. **O PDF de um quadro com vários frames descodifica a mesma foto uma vez por PÁGINA** — `ph2d-board-export/src/lib.rs`
   `paint_page` cria um `RenderCache::default()` por página (as imagens com ele). Cura: a cache de imagens partilhada
   pelas páginas de UM export (o `io.rs` do `ph2d-app-board` chama `paint_page` por página: PDF e SVG por frame). MEÇA
   antes e depois (régua no molde de `tests/it/measure_svg_export_images.rs`: as duas variantes no MESMO processo,
   intercaladas, o mínimo; um quadro com N frames e uma fotografia 4000×3000 em cada).
5. **Shift no recorte com a imagem ESTICADA** — `cropped` (`crop.rs`) toma a proporção natural em px naturais; com a caixa
   esticada (`kx ≠ ky`) a proporção no ecrã não bate. O oráculo só mediu `kx = ky`. ⇒ Corra a sonda W16 com uma imagem
   esticada (redimensionar com Shift solta a proporção no Excalidraw — medido na W16, sonda A9b) e depois Shift num canto
   do recorte: a lei é a proporção da imagem NATURAL ou a do que se VÊ? Gate com os números dele.

## Fecho

Portão batched 1× (`nextest-impacted` com `BASE=merge-base`, `CARGO_BUILD_WARNINGS=deny cargo check --workspace
--all-targets`, clippy do `ship.sh`, `fmt`, `typos`, `machete`, `deny`, `file_loc_caps`, censos da árvore combinada),
auditoria de 2 lentes, mutação nas leis novas. Smoke: estenda a cena 17 OU crie a 18 (conte no roteador `smoke.rs`) com
lições para o que o dono VÊ mudar (a imagem estragada com o aviso; o «Send to board…» com um objecto sem nome), fotografada
com `PH2D_GPU=1 bash scripts/ph2d-run.sh env PH2D_GPU=0 bash docs/MiroClone/ferramentas/soltar_wayland/no_app.sh <nome>
PH2D_BOARD_SMOKE=<n>` (⚠️ o `env PH2D_GPU=0` é preciso: o script chama a porta outra vez por dentro e esperaria pela placa
que ele próprio segura). Handoff de integração W17 (≤ 15 KB), binário `smoke` compilado 2×, `rm -rf target/*/incremental`.
Relatório ao dono curto e sem jargão, smoke em passos numerados.
