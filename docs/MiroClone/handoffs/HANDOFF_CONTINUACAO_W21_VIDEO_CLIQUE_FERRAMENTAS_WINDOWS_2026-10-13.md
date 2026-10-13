# CONTINUAÇÃO — `line/MiroClone` W21: o clique longo, o vídeo com outra ferramenta na mão, e o tocador no Windows (2026-10-13)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: a W20 FECHOU (portão verde, smoke do dono **OK** em 13/10) e **NÃO está integrada** — a W21
> continua por cima dela, na mesma linha (o dono não deu ordem de integrar; a próxima integração leva W20 + W21).
> Leia: [handoff W20](HANDOFF_INTEGRACAO_line_MiroClone_W20_2026-10-13.md) (o que mudou, os abertos), o mecanismo
> [`10_video_do_youtube.md`](../10_video_do_youtube.md) §W20, [ADR-0185 §W20](../../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md).

## Decisões do dono sobre os abertos da W20 (13/10, verbatim)

| aberto | o dono | consequência |
|---|---|---|
| as pegas da selecção pintam-se nos cantos do vídeo | «não prejudica a exibição em nada. Vamos deixar.» | NÃO mexer. Fecha o aberto (a) da W18/W20. |
| o trilho das setas ‹ › perde-se ao reabrir o projecto | «Não precisa» | NÃO persistir. Fecha o aberto (e) da W20. |
| um clique LONGO (parado) num vídeo conta como clique | «corrija» | **item 1** abaixo |
| com outra ferramenta (caneta, forma), clicar no vídeo desenha por cima | «corrija» | **item 2** abaixo |
| no Windows o vídeo abre no navegador (sem tocador no quadro) | «implemente.» | **item 3** abaixo |

## Os três itens

1. **Clique longo não é clique.** Hoje (`board_video::pointer_up`, `documents::VideoClick`) o largar sem passar o
   limiar de distância activa/toca-pausa, por mais tempo que se segure. O oráculo (`sonda_w20.mjs` A2, medido):
   carregar 600 ms parado e largar **não** activa; o clique rápido activa no largar. Meça o LIMIAR DE TEMPO do oráculo
   antes de escrever um número (varie o tempo parado: 150/300/450/600 ms… — a sonda já monta o caso) e procure se a
   casa já tem a janela de um clique (`crate::interaction` — `DOUBLE_CLICK_WINDOW_NS` é a do duplo-clique; pode haver
   uma de «clique longo»/press-and-hold; uma lei da casa vence uma constante nova). O `VideoClick` guarda o instante do
   carregar (o `board_view::Input::now_ns` do evento, nunca `Instant::now()` — os testes andam o relógio à mão).
   Vermelho primeiro pelo despacho real (`board_video_tests.rs`, o `T` com o relógio do `inp`).

2. **Com outra ferramenta na mão, o vídeo.** Hoje (`board_video::pointer_down`, `select = matches!(tool, None |
   Some(Tool::Select))`) com a caneta, uma forma, a seta, o texto… o carregar num vídeo segue para a ferramenta: ela
   desenha POR CIMA dele e, tapado, ele pausa (as regras). O dono disse «corrija» — **o que ele espera não está escrito**:
   meça o oráculo PRIMEIRO (Excalidraw com o rectângulo / a caneta / o texto na mão: carregar num `embeddable` — cria
   por cima? activa? e um clique sem arrastar?) e, se a medida não decidir entre (a) «um clique num vídeo é sempre
   play/pause, com qualquer ferramenta; o arrasto continua da ferramenta» e (b) «sobre um vídeo, a ferramenta não
   começa: o vídeo toma o gesto», PERGUNTE ao dono com as duas opções e um desenho de cada (AskUserQuestion com
   `preview`). Atenção à mão e ao laser (a mão já salta o vídeo — é pan; o laser não desenha no documento).

3. **O tocador no Windows.** ADR-0185 §W19.8: a porta TCP (`--remote-debugging-port`) está **RECUSADA** (qualquer
   processo local mandaria no navegador). A rota é o tubo com handles herdados: `--remote-debugging-pipe` +
   `--remote-debugging-io-pipes=<leitura>,<escrita>` (os valores dos dois HANDLEs, marcados herdáveis; o Chrome no
   Windows lê/escreve por eles o mesmo JSON+NUL do Linux). Hoje o arranque é `sh -c 'exec … 3<&0 4>&1'`
   (`ph2d-board-video/src/browser.rs:81`) — só Unix; `browser::find` e o resto já pensam no Edge («no Windows o Edge
   está sempre lá», ADR §Decisão). Implemente o arranque Windows (`#[cfg(windows)]`: `CreatePipe` ×2, os handles do
   lado do filho herdáveis, `std::process::Command` não passa handles arbitrários ⇒ `windows-sys`/`std::os::windows`
   com `STARTUPINFOEX` + `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`, ou o que a medição mostrar mais simples — veja se a
   stack já tem `windows-sys`: `bash scripts/stack-audit.sh --tetos` antes de pôr dependência). ⚠️ **Não há Windows
   nesta bancada**: o mínimo provável é (i) `cargo check --target x86_64-pc-windows-gnu -p ph2d-board-video` (o alvo
   não está instalado: `rustup target list --installed` = linux, apple, wasm — instalar o alvo é da bancada, diga-o)
   e (ii) correr o tocador Windows sob o Wine se o Wine + um Chrome/Edge Windows funcionarem (há Wine na bancada: o
   MagicaCSG corre nele — memória `reference_magicacsg_installed_under_wine`). Se nenhum dos dois der a prova de
   que TOCA, o item fecha como «compila para Windows, por medir num Windows real» e diga-o ao dono com a medida —
   nunca «feito» sem ter tocado. O caminho de hoje (abrir no navegador e dizer porquê) fica como recuo.

## Lembretes medidos da W20

- As sondas do oráculo: `excalidraw_oracle/sonda_w20.mjs` (a página em `https://ph2d.local/` — de `http://127.0.0.1`
  o YouTube recusa o vídeo), `youtube_motor/sonda_w20_pausa.mjs`, `youtube_motor/sonda_w20_mais_videos.mjs`.
- Os testes do navegador real: `bash scripts/ph2d-run.sh cargo test -p ph2d-board-video --release --lib -- --include-ignored --nocapture --test-threads=1` (13, verdes em 13/10).
- A foto da cena: `PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/MiroClone/ferramentas/soltar_wayland/no_app.sh <nome> PH2D_BOARD_SMOKE=<n>` (o fotógrafo larga uma imagem no meio — não é da cena).
- `documents.rs` NÃO importa de `screens` (o portão `architecture_the_foundation_modules_form_a_dag` apanhou-o na
  W20); mensagens de teste sem `→` (`no_tofu_glyphs`); o shell é zsh (`"${P[@]}"` para listas de caminhos).
- A máquina é partilhada e esteve a `load` 20–76 o dia todo: relógio não vale; gates determinísticos sim.
