# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 13/10 — F14b: três relatos do smoke da F14

> A F14 está FECHADA (gate, mutações, binário): [handoff de integração](HANDOFF_INTEGRACAO_line_UIUX_2026-10-13_F14_QUINAS.md).
> Regras R1–R9: topo do [handoff da F10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md). Esta
> janela parou por R1 (~150 chamadas). Não integrada, não enviada.

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, HEAD `d3af166cf0`, árvore limpa, merge-base
  `757afebd58` (o `main`). FASE 0 → 2 do `MODELO_TROCA_DE_AGENTE_NA_LINHA.md`, depois isto.
- O dono gostou (*«Ficou muito bom!»*) e mandou três relatos, com fotos, sobre o balão *Speech*.
  §0.10: resolva os três num bloco, com UMA rodada de medição, UM portão.

## 1 — O ⇕ e um ponto da forma no mesmo sítio (*«A seta e o ponto não devem ficar na mesma posição seja vertical ou horizontal»*)

- **Mecanismo (lido no código):** `shape_handles::handle_grip`, ramo `Spot::Rail`, põe o 1.º ⇕ na
  borda direita da caixa visível a `h/4` do topo (e, na caixa baixa, F13 D, um `RAIL_PITCH` por fora
  à meia altura) olhando SÓ a caixa. No *Speech*, o ⊙ do *Radius* do corpo
  (`Spot::Rounding{corner:0, next:true}`) desce pela borda DIREITA quando o raio cresce, e o «Corner
  round» F14 deu ao *Speech* um ⇕ (`arrow(&[5])`) — os dois cruzam-se. Qualquer forma com ⇕ e um
  ponto cujo caminho passe pela borda direita tem o mesmo defeito.
- **Desenho proposto (o dono: as setas ficam como estão):** no ramo `Rail`, antes de assentar, ler os
  grips das outras alças (não-`Rail`) da forma com os valores ACTUAIS; se algum fica a menos de
  `park_at` de qualquer ⇕ da fila, a fila inteira começa `+1` passo por fora (repetir até 3). O F13 D
  já tem a morada «por fora»; reaproveitar. ⚠️ Medir o custo (cada `handle_grip` de ⇕ recalcularia
  ~5 grips = ~5 `cook` por ⇕ por quadro) antes de escolher entre isto e uma leitura estável pelos
  TRILHOS (`law::track`), que não salta durante o arrasto do ⊙ mas custa ~24× mais.
- **Gate:** estender `no_arrow_touches_a_box_handle_or_the_action_bar_from_20_to_400_px`
  (`shape_handles_f12_tests.rs`) para «nenhum ⇕ toca o grip de NENHUMA outra alça», varrendo cada
  campo de cada forma (não só o default). Vermelho primeiro, no *Speech* com *Radius* alto.

## 2 — «Radius 0» no *Speech* desenha «orelhas» no traço dos quatro cantos do corpo

- Foto do dono: lentes escuras de borda branca atravessando a diagonal de cada canto do corpo, maiores
  que o traço, a sair para FORA da quina.
- **Medido nesta janela:**
  - `cook(SpeechRect)` com `v[0] = 0` exacto: 7 vértices limpos, quinas vivas, alças nos pontos — NÃO
    há arco nenhum a raio 0 exacto.
  - O contorno do traço pela casa (`kurbo` `expand_stroke`, `Join::Miter`, limite 4, traço 30 px numa
    caixa de 400 px) — régua: pontos a menos de `0,4·w` do contorno sem enrolamento: **0 furos** no
    *Speech* e no rectângulo com «Corner round», raios `0 · 0,01 · 0,1 · 0,5 · 1 · 3 · 10 · 14 · 16 ·
    30` px (o 1 do rect a 3 px é ruído de borda).
- **Hipótese a medir PRIMEIRO (não confirmada):** o valor guardado não é 0 mas uma fracção de pixel
  (o ⊙ arrastado até ao canto pelo `Law::Nearest`; a UI mostra «0»); o canto é então um arco de raio
  `r < w/2`, cujo offset INTERIOR vira um arco ao contrário de raio `w/2 − r` do lado de FORA — a
  lente. O `kurbo` resolve-o; o traço NATIVO do Vello na placa (`stroke_uniform` → `inner.stroke`,
  quando `cut_outline` devolve `None`) talvez não. Se for isso, atinge TODO canto arredondado com
  `r < w/2` (o rectângulo arredondado incluído), não só o *Speech*.
- **Sonda pronta a escrever:** no molde do `pelo_vello` de
  `crates/ph2d-shape-gpu/tests/it/paridade_com_o_vello.rs` (o `VelloPass` do produto +
  `ph2d_vec_render::stroke_uniform`), só o TRAÇO (sem fill) do `cook(SpeechRect)` com o raio do
  corpo em `0 · 0,01 · 0,1 · 1 · 3 · 10 · 14 · 30` px e `w = 30` px; contar pixels pintados a mais
  de `w/2 + 1,5` px do contorno (a orelha para fora) e sem tinta a menos de `w/2 − 1,5` px (o furo).
  `PH2D_GPU=1`. Ao lado: o rectângulo arredondado e o «Corner round» do rectângulo nos mesmos raios.
- **Curas candidatas (decidir pela medida):** (a) quando o menor raio de curvatura do caminho é
  `< w/2`, o `stroke_uniform` desenha pelo contorno do `kurbo` preenchido NonZero (a porta que já
  existe para o bico cortado, `cut_outline`); (b) e/ou o arrasto do ⊙ encaixa no zero abaixo de meio
  pixel. Gate: a régua da sonda, vermelha antes.

## 3 — O bico do *Speech* nunca sai pontudo (*«nenhuma opção do painel lateral produz uma ponta pontuda, mas apenas quadrada e redonda»*)

- **Medido:** no `cook(SpeechRect)` default, o vértice do bico (v3, `[1,2; 0,0]` na caixa 4×4) tem
  `in_handle = out_handle = [1,2; 0,501]` — as duas curvas chegam pela MESMA tangente (vertical): ângulo
  interno **0**. Com *Miter*, o bico de um ângulo 0 passa sempre o limite (4 ⇒ só bica abaixo de
  `2·asin(1/4) ≈ 29°`) e a lei do bico cortado (F12, SVG 2 `miter-clip`) corta-o — é o «quadrado»
  comprido da foto; *Round* arredonda; *Bevel* chanfra. Nenhuma junta faz bico num ângulo 0.
- **Cura (geometria, não estilo):** o bico chega à ponta com um ângulo de verdade, acima do limiar do
  corte (p.ex. 35–45°), mantendo os lados côncavos — oráculo: o rabo de balão do produto do dono
  (Affinity), fotografado/medido antes de escolher o número.
- **A família (§0.10), a medir num censo:** toda quina do catálogo, no default e na varredura dos
  campos, com ângulo interno `< 29°` — no levantamento da F14 aparecem cúspides de ângulo 0 nos chifres
  da *Moon*, no bico da *Brace*, e o bico do *Oval say* (`SpeechOval`) e do *Thought* devem ser
  conferidos. Para cada uma: é uma PONTA que o artista quer pontuda (cura pela geometria) ou uma
  cúspide intrínseca (fica, com a razão escrita). Gate: o censo, com a lista escrita.

## Fecho da F14b

Mutação de cada cláusula nova · portão batched (`nextest-impacted` + clippy do `ship.sh` + `fmt` +
`check --workspace --all-targets`) · o handoff de integração da F14 ganha uma §9 «F14b» (ou um novo
que o cite) · `CLAUDE.md` §5.1 «Último:» · binário smoke 2× · fotos (o *Speech* com *Radius* alto e
o ⇕; a raio 0 / fracção com traço grosso; o bico com *Miter*) · o smoke ao dono em passos numerados.
