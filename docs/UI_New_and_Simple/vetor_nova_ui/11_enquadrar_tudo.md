# 11 — Enquadrar tudo (F8, 10/10)

> Pedido do dono (10/10, depois do smoke OK do Edit Shape): *«enquadrar tudo»* — o desenho inteiro
> cabe na tela, num gesto; no iPad, o beliscão rápido do Procreate. Prompt:
> [`10_prompt_F8_enquadrar_tudo.md`](10_prompt_F8_enquadrar_tudo.md). Código: `ph2d_render::camera_frame`
> (a lei), `ph2d_app_vec::{frame_boxes, view_request}` (as caixas e o pedido), a cena 14.

## §1 — A premissa que a medição derrubou

O prompt dizia que enquadrar *«não existe em lado nenhum»*. **Existia, escondido**:
`ViewFocusKind::All` (o 3.º estado do botão VIEW da régua), com três defeitos medidos:

| defeito | consequência |
|---|---|
| contava só IMAGENS (`scene_sprites_bbox_world`) | numa cena de formas caía na «cena vazia» (a nota da `PH2D_VEC_BONE_SMOKE=2` dizia que ele lá funcionava *«por acaso»*) |
| encaixava na JANELA, `×1,1` | os painéis docados tapavam as bordas — a `PH2D_VEC_BONE_MEDIA_SMOKE` já tinha uma nota *«decisão do dono»* a pedir a cura |
| saltava | o único movimento da câmera que não andava pela mola |

A grep do prompt (`fit`, `frame_all`, …) não o apanhava porque o nome era `ViewFocusKind`.

## §2 — Os oráculos (corridos, nunca lidos)

`oraculo_enquadrar/`: o **tldraw 5.5.2** («tldraw license»: corre-se, não se porta) COM a interface
(variante `~/Apps/tldraw-oracle/www_ui` — com `hideUi` as teclas não existem) e o **Excalidraw
`next`** (MIT), pelo Playwright, sobre 11 cenas NOSSAS (`cenas.mjs`) × 3 janelas (1280×800,
1920×1080, 700×900) × 3 câmaras de partida; tecla `Shift+1` (tudo) e `Shift+2` (seleção). 66
fixtures com cabeçalho; resumo: `python3 -I resume.py <dir>`.

| | tldraw | Excalidraw |
|---|---|---|
| área | a janela inteira (a UI dele flutua) | a janela MENOS a interface (`getViewportOffsets`: 84/24/24/24 px; 235 à esquerda com seleção) |
| margem no eixo apertado | **64 px fixos** nas três janelas | **0** além da interface |
| zoom ao enquadrar | `[5 %, 800 %]` (o máximo da câmera dele) | `[10 %, 100 %]` (nunca aproxima) |
| ponto sozinho (2×2) | 800 % — não enche a tela | 100 % |
| partida da câmera | não importa (o destino é o mesmo) | idem |
| anima? | tecla: **~330 ms** (sempre, qualquer distância); API pura: salta | salta |
| cena vazia | nada se move | volta a 100 % |
| trancado | conta | conta |
| seleção (`Shift+2`) | a 100 %: encaixa; noutro zoom: vai a 100 % centrado nela (um ciclo) | encaixa (com o tecto de 100 %) |

⭐ **Os dois são a MESMA lei com parâmetros diferentes** — o eixo apertado manda, o outro centra, o
ponto vai ao zoom máximo, o que transborda centra. O gate
`the_law_with_each_oracles_parameters_reproduces_its_recorded_frames` reproduz as ≥ 200 corridas de
tecla dos dois, ao centésimo de px, com a nossa conta e os parâmetros de cada um.

Manuais (sem corrida): **Procreate** — *«Quick-Pinch is the same gesture as Pinch to Zoom, but
performed much faster»*, a tela *«pops»* para caber, e *«to return to the exact view you had before
your Quick-Pinch, do the Quick-Pinch gesture in reverse»*
([handbook](https://help.procreate.com/procreate/handbook/interface-gestures/gestures)).
**Illustrator** — `Ctrl+0` *Fit Artboard in Window*, `Ctrl+Alt+0` *Fit All in Window* (cheat sheets;
a página da Adobe recusa o robô); o perfil do Illustrator que o Inkscape traz liga `Ctrl+0` ao
*zoom page*.

## §3 — A lei (`ph2d_render::camera_frame`)

- **«Tudo» = o que a cena DESENHA:** as imagens do mundo de apresentação (a malha posada de uma
  imagem presa a ossos incluída), as formas que entraram na ORDEM do quadro (`FrameOrder`), os
  objectos Flip não escondidos (pela árvore). O escondido não conta; o trancado conta (os dois
  oráculos). **A seleção** é a primária mais as extras; um grupo é a união dos filhos desenhados.
- **A área:** a visível (`canvas_area::visible`, sem painéis nem réguas) **menos a faixa da pílula
  ⑤** (`canvas_area::frameable`) — medido na foto da cena 14: as formas de baixo iam para debaixo dela.
- **A margem é DERIVADA, não escolhida:** `float_bar::frame_margin()` = o afastamento `Lg` da barra
  de ações ④ + uma linha de botões + o ar `Xs` dos dois lados = **44 px lógicos** — o meio da faixa
  dos oráculos (24 … 64), e cobre a coluna ① (36 px). Gate `the_frame_margin_clears_both_floating_bars`.
- **O zoom é o da câmera** (`Camera2d::clamp_height`, perguntado; a roda passou a perguntar o mesmo):
  o tldraw pela razão dele (o tecto é o da câmera, não um número novo). Um ponto vai ao zoom máximo —
  ⇒ `0,5 m` de altura visível, não enche a tela.
- **Cena vazia:** a vista de omissão (o Excalidraw; e era o comportamento de sempre do `All`).
- **O voo:** `Role::Viewpoint` — a mesma mola criticamente amortecida da roda (`DISCRETE`, assenta em
  ~0,38 s; o tldraw anda ~0,33 s), e a única que sobrevive ao *reduced motion* (o corte desorienta
  mais). Zoom em `ln(h)` linear, centro pelo **ponto fixo** (o ponto do mundo que está no mesmo sítio
  do ecrã nas duas vistas não se mexe — gate). Parte da câmera do 1.º tique; uma escrita alheia
  (pan, dedo) larga-o; assentado, publica o destino verbatim. Não é passo de undo: a câmera
  não está no `ProjectState`.
- **A roda × o voo: o gesto mais NOVO ganha** (`ViewFlight::arbitrate`, chamada no
  `canvas_zoom::publish`): um voo pedido e ainda sem tique manda a roda largar; uma roda girada
  durante um voo larga o voo. ⛔ Sem isto o 1.º entalhe da roda a meio de um voo PERDIA-SE (a mola
  da roda nasce parada, o voo escrevia por cima e a roda largava) — apanhado por um gate da shell.
- **O pedido resolve-se UM quadro depois** de feito: uma cena que monta e enquadra no mesmo quadro
  media o mundo desenhado do quadro anterior (fotos `media1` → `media1b`, as três mídias cortadas →
  inteiras).
- **O beliscão rápido** (`TouchOut::QuickPinch`): a navegação inteira acabou antes de um
  toque-e-segurar (`LONG_PRESS_NS`, 0,5 s, o UIKit — a mesma fronteira do toque de 2 dedos), e o
  afastamento mudou mais do que a folga (10 pt) e mais do que o ponto médio andou. Juntar = *enquadrar
  tudo*; afastar = a vista de antes, **só** se o gesto começou onde o enquadramento deixou a câmera, e
  uma vez. A pinça viva acontece igual (é a mesma pinça). Fronteira varrida com a mão sintética de 100
  a 1200 ms (`the_quick_slow_border_is_the_long_press`); a pinça da cena 11 (1 s) continua só zoom.

## §4 — As portas

| porta | pedido |
|---|---|
| *Ver ▸ Frame All · Ctrl+0*, `Ctrl+0` (e o `0` numérico) | `All` (voa, lembra a vista de partida) |
| *Ver ▸ Frame Selection · F*, `F`/`Home` | `FrameSelected` (voa) |
| beliscão rápido a juntar / a afastar | `All` / `Previous` |
| botão VIEW da régua (ciclo) | `Selected` → `Camera` → `All` |
| duplo-clique na Hierarquia, ~10 cenas de smoke | `Selected` — **só centra, salta** (inalterado: as cenas contam com isso) |

**Teclas — decisão do dono (10/10):** `Ctrl+0` = *Enquadrar tudo* (o perfil Illustrator; sem
pranchetas, o *Fit Artboard* e o *Fit All* são a mesma coisa) e `F` = *Enquadrar seleção* com zoom
(Unity/Maya/Blender; antes só centrava). O `Home` anda com o `F`, como sempre andou; o `0` sem
`Ctrl` não faz nada novo.

## §5 — ⛔ Recusas MEDIDAS

| recusa | porquê |
|---|---|
| o ciclo do `Shift+2` do tldraw (fora dos 100 % vai a 100 %) | dois gestos iguais fazem coisas diferentes conforme o zoom; o Excalidraw e o *Frame Selected* do Blender/Unity encaixam |
| o tecto de 100 % do Excalidraw | um app vetorial aproxima um desenho pequeno (tldraw, Illustrator); o tecto da câmera já impede o ponto de encher a tela |
| margem proporcional à janela (o `×1,1` antigo) | os dois oráculos usam px fixos nas três janelas medidas |
| mudar o `Selected` (as cenas e o duplo-clique) para encaixar com zoom — o `F` passou a pedir o `FrameSelected` | ~10 cenas de smoke de outras linhas centram e pintam a coordenadas da câmera de omissão |
| resolver o pedido no próprio quadro | as mídias presas a ossos saíam cortadas (o mundo desenhado era o do quadro anterior) |
