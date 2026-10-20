# W13 — o tutorial «fazer» ensina o Fill (2026-10-20)

> Pedido: [PROMPT_W13](handoffs/PROMPT_W13_o_tutorial_ensina_o_fill_2026-10-20.md) (smoke do dono da W12 OK, 20/10).
> Handoff: [W13](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W13.md). Antes: [W6](03_plano_W6.md) (o tutorial),
> [W11](09_W11_o_cef_a_gravar.md) (o ColorDrop gravado, a etapa «Fill»).

## §1 — O defeito, medido

Gravação `Pincelada · Fill · Pincelada` (`tests/it/macro_colordrop.rs::grava_o_colordrop`, modal e gesto parado) pelo
tutorial de antes (`EnsinoDoPainter::medido`): **zero critérios**, não dois. O tutorial dava-se por acabado sem pedir
nada. Duas causas:

1. `pinceladas_em` saltava a etapa Fill; a medida da 1.ª pincelada corria até à pincelada SEGUINTE, já com o Fill —
   que inunda o papel à volta e também a região dela na trilha SEM ela. Gravada = base = 1 ⇒ Δ vazio ⇒ sem critério.
2. A 2.ª pincelada (preta sobre o vermelho do Fill) contava «pintado» contra o papel inicial: a base (só o Fill) já
   diferia do branco em toda a área ⇒ Δ vazio.

E um defeito vizinho, da mesma família (W11): **apagar a etapa Fill no painel** deixava o `ColorDropComeca` (a etapa
começava no `Down`); a pincelada seguinte tocava em modo Fill e inundava a tela inteira na cor dela.

Gates vermelhos antes da cura (`ca5ead7dd4`): `tests/it/macro_tutorial_fill.rs` — `o_tutorial_ensina_o_fill_entre_as_pinceladas`
(`left: []`) e `apagar_o_fill_nao_deixa_a_pincelada_seguinte_em_fill` (o papel `[20,20,20]` em vez de branco).

## §2 — A lei

- **A etapa Fill começa no `ColorDropComeca`** (`etapas.rs`): apagá-la leva a entrada em Fill; o tipo lê-se em
  `eventos[..=inicio]`.
- **A região de um Fill é o que ele pintou na reprodução** (`tutorial_fill.rs::regioes_dos_fills`): os texels que o
  gesto mudou, do 1.º ao último evento da etapa, e deixou na cor dele, opacos (a água que anda ao lado nunca dá a cor
  ao byte). A reprodução corre em `EnsinoDoPainter::de` (no fio `macro-referencias`) até ao fecho do último Fill.
- **Quem julga cada texel:** um Fill julga só onde nada depois o repinta; uma pincelada, onde nenhuma anterior pintou
  e nenhum Fill seguinte repinta. Um Fill apaga a cobertura das pinceladas anteriores (o texel volta a ser «do
  fundo»).
- **Cumprir:** a mesma régua da W7 (`(fracção − base) / (gravada − base) ≥ 0,6655`), com a fracção por tipo:
  - Fill: alguma camada mudou ali **para a cor dele** ([`parecida`]: `≤ TOLERANCIA_DA_COR` por canal, alfa
    `≥ 255 − 26`);
  - pincelada: alguma camada mudou ali e, onde um Fill anterior pôs cor (`fundos`), **para outra que não a dele**.
- **O balão:** na etapa Fill, `PedidoDoTutorial::Cor { cor }` («Pick this colour on the ringed C&F button: #RRGGBB»)
  enquanto a cor do pincel não é parecida, depois `Solta { cor }` («Drag from the ringed C&F button onto the marked
  spot…»); o anel no `PAINTER_RAIL_FILL`; a amostra da cor na linha do título; na imagem, um alvo (anel do acento do
  tamanho de uma linha do chrome, centro marcado) onde a cor caiu (`AreaNaImagem::solta`). «Mostra-me» vale nos dois.
- **«Mostra-me» de um Fill** (`tutorial_fill.rs::mostra`): o gesto gravado (ponteiro, portas do C&F, o modal) com a cor
  e o limiar do toque (`etapas::NoToque::limiar`); o C&F volta ao modo do ALUNO; a cor e o limiar dele voltam no fim.

### A tolerância da cor, medida

| Limite | De onde | Valor |
|---|---|---|
| o rato alcança | a faixa do matiz do seletor: 256 px lógicos (`280 − 2 × 12`) ⇒ 1,41°/px ⇒ até 5,98 níveis por canal num píxel ⇒ metade | ≥ 3 |
| a paleta distingue | as duas amostras mais próximas da paleta por omissão, `(240,96,0)` × `(220,50,50)` (`blender_color_picker/state.rs`) | < 50 |

Meio do intervalo: 26,5 ⇒ **26**. O hex, a amostra da paleta e o conta-gotas dão a cor exacta.

## §3 — Medições (sonda `tutorial::fill::tests::sonda_a_caixa`, a caixa a 256², load ~170)

A caixa (`roteiro::grava_a_caixa_com_um_fill`): quatro traços pretos que fecham um quadrado, o azul `(40,110,210)`
solto dentro pelo C&F com o Done do modal (limiar por omissão 0,1), um traço amarelo por cima do azul.

| | Digital | Wet Paint |
|---|---|---|
| `de` (com a reprodução até ao Fill) | 19 ms | 370 ms |
| `mede` (as curvas, 4 fios) | 46 ms | 33,0 s |
| `novos` por etapa | 5396 · 4392 · 4392 · 3388 · **3849** · 5367 | 5288 · 4392 · 4392 · 3356 · **3507** · 5407 |

Razão do Fill (os dois meios iguais): exacta 1,0 · outro ponto do quadrado 1,0 · cor a 26 níveis 1,0 · **cor a 27
níveis 0,0** · a cor certa fora da caixa 0,0. O Fill é tudo-ou-nada; a régua de 0,6655 fica longe dos dois lados.

## §4 — ⛔ Recusas e limites

| O quê | Porquê |
|---|---|
| Julgar o Fill por «pintado» (qualquer cor), como a pincelada | o pedido é a cor certa; e a 2.ª pincelada sobre o Fill nunca teria critério |
| A região do Fill pela geometria (um flood na tela inicial) | a região depende das pinceladas antes dela e do limiar afinado no modal; só a reprodução a sabe |
| Diferença de cor perceptual (ΔE) | a métrica do balde é o máximo por canal (`fill.rs::color_match`); duas métricas para a mesma pergunta divergem |
| Penalizar o Fill do aluno que vaza (traços que não fecham) | o vazamento é consequência de etapas já cumpridas; o tutorial bloquearia sem dizer porquê. A região certa na cor certa cumpre |
| A cor como «preparo» (antes da etapa, sem «Mostra-me») | escolher a cor É metade do gesto do Fill; quem não a acerta precisa do «Mostra-me» |
| Uma cena de smoke nova (`=7`) | crescia a shell (`the_shell_only_shrinks`); a `=4` lê `PH2D_MACRO_TUTORIAL=caixa` na família |
