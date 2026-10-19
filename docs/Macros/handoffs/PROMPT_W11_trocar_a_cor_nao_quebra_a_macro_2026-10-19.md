# PROMPT W11 — trocar a cor do pincel a gravar não quebra a macro (smoke do dono, 19/10, depois da W10)

> Rota «linha reaberta» de `docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`
> antes de ler; sem reset nem rebase. A W10 está fechada ([handoff](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md)).

## O que o dono viu

A gravar, trocou a cor do pincel. O painel MACROS: «The recording went through something a macro does not
replay: begin_colordrop_fill», com as etapas `Refused: begin_colordrop_fill` · `Refused: fill_commit` ·
`Refused: restore_after_…` · `Refused: fill_commit`, depois `Stroke 3` e `Stroke 4` (vermelhas). A reprodução
pára na 1.ª recusa (`macro_track/tocar.rs:370`, `ErroDaMacro::PortaRecusada`) — as pinceladas seguintes não
tocam.

## Mecanismo (leitura do explorador, 19/10 — confirmar com um gate VERMELHO antes de curar)

- A cor troca-se pelo botão **C&F** do rail (`PAINTER_RAIL_FILL`). Um press nele arma o gesto do ColorDrop
  (`shells/desktop/src/input_dispatch/fill_drag.rs:103-131`, `arm_fill_drag_if_on_button`; consome o evento).
- Cada `CursorMoved` com o gesto armado chama `painter.begin_colordrop_fill(prev_mode)` **enquanto `!on_canvas`**
  (`fill_drag.rs:161-171`), antes de saber se o ponteiro entrou no sprite — o comentário diz «no primeiro
  sample dentro do sprite», o código não testa a pegada.
- Soltar sem ter entrado na tela abre o seletor de cor e chama `restore_after_colordrop` (`fill_drag_up`).
- As quatro portas são `Recusa` (`macro_track/portas.rs:145-147`; `tool/paint/fill.rs:364-399`). A troca de
  cor pelo seletor em si grava bem: `set_brush_color_srgb8` → `Passo::Cor` (`portas.rs:100`,
  `painter_bridge_phases.rs:60-76`).

## O que a W11 faz

1. **Gate vermelho primeiro**: a gravar, o gesto do C&F que só abre o seletor (press, um pouco de arrasto que
   não entra no sprite, soltar) e a cor escolhida ⇒ a trilha tem `Passo::Cor` e **nenhuma** `Recusada`; a macro
   toca até ao fim com as pinceladas na cor nova.
2. **Cura**: o ColorDrop só começa quando o ponteiro entra no sprite; o ramo do seletor não chama portas do
   preenchimento (ou chama-as sem gravar, se nada começou).
3. **Decidir e, se o padrão-ouro o pedir, fazer**: o ColorDrop de verdade (arrastar a cor para a tela e
   soltar = preenchimento) gravável como passo. É determinístico dado o estado (`fill.rs:141-190`: semente
   `floor(fill_seed)`, tolerância `round(threshold·255)` por canal, a cor do pincel em u8, a componente da
   seleção): um `Passo::ColorDrop { semente relativa ao quadro de referência, cor, limiar, commit/cancel }`,
   com o dwell e o modo ao vivo do limiar (`set_fill_threshold`, `fill_commit`).
4. Smoke conduzido e fotografado; o dono confere à mão na `=5` (gravar, trocar a cor no C&F, pintar, Stop, ▷).
