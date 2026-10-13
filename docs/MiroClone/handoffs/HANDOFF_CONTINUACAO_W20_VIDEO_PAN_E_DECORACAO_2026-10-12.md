# CONTINUAÇÃO — `line/MiroClone` W20: o pan não pausa o vídeo, o vídeo decora-se como um retângulo e arrasta-se (2026-10-12)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: a W19 FECHOU e integrou-se ao `main` por ordem do dono (12/10)
> ([handoff W19](HANDOFF_INTEGRACAO_line_MiroClone_W19_2026-10-12.md); mecanismo em
> [`10_video_do_youtube.md`](../10_video_do_youtube.md) §W19; [ADR-0185 §W19](../../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md)).

## Os pedidos do dono (12/10, depois do smoke da W19), verbatim

1. **«Pan pausa o vídeo. vamos evitar isso. Vídeo só deve pausar se o quadro estiver fora da área visível.»**
   Reproduza VERMELHO antes de curar (o pan pelo despacho real: espaço+arrastar, botão do meio, a mão, dois dedos).
   Hipóteses a MEDIR, não a escolher: (a) o carregar do pan cai no `board_video::pointer_down` — «clicar fora
   desactiva» — e desactivar pausa; (b) a regra `≥ 200×200 px VISÍVEIS` (W19) pausa quando o pan deixa menos de
   200 px à vista — o dono pede «só se estiver FORA da área visível»: confronte com a *Required Minimum
   Functionality* do YouTube (≥ 200×200 visíveis) e, se a regra do YouTube impede o pedido, diga-o ao dono com a
   medida em vez de a violar; (c) outra coisa que a medição mostrar.
2. **«A janela de vídeo deve ter as opções de decoração como um retângulo: borda (stroke), cor da borda, etc.»**
   O `Embed` já tem um `Style` (hoje só `round` e `opacity` se lêem). Meça no oráculo (Excalidraw: o `embeddable`
   aceita `strokeColor`/`strokeWidth`/`strokeStyle`/`roundness`/`opacity`/`backgroundColor`?) o que se aplica e
   como se desenha; a barra de estilo do quadro para um vídeo seleccionado passa a ter os grupos do retângulo que
   fizerem sentido; o `.excalidraw` lê/escreve-os. ⚠️ Regra do YouTube: a decoração fica À VOLTA do tocador (a
   borda por fora ou no rebordo da caixa), nunca por cima do quadro do vídeo; e um preenchimento não pode tapar o
   vídeo vivo.

3. **«Não é possível arrastar a janela de vídeo pois clicar no vídeo dá play. Logo, precisamos resolver isso:
   segurar e arrastar move, apenas clicar dá play/pause.»** Hoje (`board_video::pointer_down`) o carregar no play
   — ou num vídeo já seleccionado — ACTIVA-o no próprio Down, e activo o rato vai todo para o tocador: não sobra
   gesto para mover. Cura a desenhar e a gatear pelo despacho real: a decisão passa para o LARGAR — Down guarda o
   ponto; um Move além do limiar de arrasto da casa vira o arrasto do editor (move o elemento); Up sem arrasto é o
   clique (activa / toca-pausa). Atenção ao vídeo ACTIVO e vivo: o clique dentro dele vai ao tocador (o play ao
   meio, o canal) — o arrasto num activo precisa de uma pega que não seja o tocador (a moldura/borda da caixa, que
   o item 2 desenha, ou a nossa barra), porque o Down dentro do tocador vivo é do YouTube. Meça o oráculo (o
   Excalidraw: arrastar um `embeddable` activo vs inactivo) antes de escolher.

## Lembretes medidos da W19

- **Reaberta em 13/10** sobre o `main` `757afebd58` (`git reset --keep main`: os commits da linha já estavam todos
  lá). Depois da integração entrou a UIUX F9 (`e5cf9785f8`, as alças das formas): mexeu em 2 linhas do
  `hero/board_view.rs` e na cadeia do `Esc` da shell (`input_dispatch/keyboard_escapes.rs`) — leia-as antes de
  mexer no arrastar do vídeo (item 3) e no `Esc` do nosso ecrã inteiro.

- O enquadramento das cenas não desconta os painéis laterais (aberto (f) do handoff W19).
- Os testes do navegador real: `bash scripts/ph2d-run.sh cargo test -p ph2d-board-video --release --lib -- --ignored --nocapture`.
- A foto da cena: `PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/MiroClone/ferramentas/soltar_wayland/no_app.sh <nome> PH2D_BOARD_SMOKE=<n>`.
