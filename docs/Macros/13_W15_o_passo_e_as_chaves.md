# W15 — o passo da macro e as chaves na Timeline (smoke do dono 22/10)

> Pedido: [PROMPT_W15](handoffs/PROMPT_W15_o_passo_e_as_chaves_na_timeline_2026-10-22.md). Handoff:
> [W15](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-22_W15.md). Fotos: [smoke_2026-10-22_W15/](handoffs/smoke_2026-10-22_W15/).

## 1. «a playhead não se desloca exatamente para o local do keyframe»

**Mecanismo — dois instantes diferentes para a mesma etapa, e um arredondamento:**

| Quem | Antes | Onde |
|---|---|---|
| Painel MACROS (o número da linha e o clique) | o FIM da etapa (`t1_s`) | `ph2d-panel-macros/src/corpo.rs:173`, `seam.rs` (`VaiPara { t_s: t1_s }`) |
| Losango da linha «Macro steps» | o INÍCIO (`t0`) | `ph2d-panel-timeline/src/macro_steps.rs` (paint e Click → `Scrub(t0)`) |
| `VaiPara` | `TimelineIntent::Scrub(t1)` = arredondado ao quadro (1,63 → 1,625) | `ph2d-app-macros/src/lib.rs`, `ph2d-timeline/src/intent_apply.rs` (`snap`) |

A captura do dono (uma etapa «Stroke 1» de ~1,0 s a 1,63 s; losango em ~1,0; cabeça em 1,625) é
exactamente isto: o losango marcava o início, o painel o fim. O plano W5 (§5) escolheu o FIM para o
clique (a tela mostra a etapa feita); a W5d pintou o losango no início. Nenhum gate comparava os dois.

**Cura:** um instante só — o FIM. `EtapaDaMacroView::fim` (era `t0`; renomeado para que o compilador
achasse cada leitor), o losango pinta em `TimeView::x(fim)` (a mesma conta da régua e da linha da
cabeça), e o painel e o losango pedem `TimelineIntent::ScrubExact(fim)` — nunca arredondado: o fim de
uma etapa é um instante GRAVADO, não um quadro, e arredondar para baixo mostraria a etapa por acabar.
`TimelineIntent::is_seek` passa a ser a porta única dos dois leitores da shell que tratam um salto
(sinais `jumped` em `timeline_bridge`, auto-key `scrub_now` em `fase_timeline_drain`).

**Dur(s) 6,208 com um clipe de 4,21 s** (também na captura): na reprodução (`=6 etapa`) o Dur é igual
ao clipe (4,207 e 4,311 nas duas corridas). `Dur` = `length_override` ou a última chave de QUALQUER
faixa do clipe (`ph2d-timeline/src/doc_extent.rs`); arrastar um losango escreve uma espera e estica o
clipe. Sem a sequência de gestos do dono não se reproduz — **pergunta aberta ao dono**.

## 2. «Mover keyframes … às vezes alguns são apagados»

**Mecanismo:** o arrasto de chaves emite um `MoveSelectedKeys` POR QUADRO dentro de um bracket
`BeginEdit`/`EndEdit` (`ph2d-panel-timeline/src/key_drag.rs`), e `Track::move_keys`
(`ph2d-anim/src/track.rs`) fazia a absorção («a movida que cai no tempo exacto de uma parada
absorve-a») a CADA quadro. Com o Snap, o tempo anda em quadros inteiros: uma chave que só passa por
cima de outra cai no tempo exacto dela durante um quadro e apaga-a — e ela não volta quando o grupo
segue. Onde o defeito aterra (`merge_moved_over_stationary`) é a regra certa; o que o provoca é
aplicá-la a um estado INTERMÉDIO do gesto.

**Cura:** dentro do bracket o movimento é `Track::move_keys_unmerged`; `TimelineHistory::note_moved`
regista as chaves movidas e `absorb_where_released` (no `EndEdit`, e num `BeginEdit` que encontre um
bracket nunca fechado) corre a absorção UMA vez, onde o gesto largou. Largar em cima de uma chave
continua a absorvê-la (o dono: «seria normal apagar»). O movimento atómico (fora de bracket: teclado,
outros chamadores) não muda.

Código: Timeline (`ph2d-anim`, `ph2d-timeline`, `ph2d-panel-timeline`), não da Macros — curado aqui
em Modo L, aditivo (handoff §3).

## 3. Medição e prova

- Vermelho (cura retirada): `[1.5, 2.0]` onde devia `[1.0, 1.5, 2.0]` (a chave de 1 s apagada ao ser
  atravessada) e `[0.5, 0.75]` no arrasto para trás.
- Cena `PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=etapa`: `playhead=3.27`, `steps=["1.65", "3.27"]`, o
  relógio da macro `t=3.2711`; foto com a linha da cabeça sobre o 2.º losango.
- Mutação: 11 mutações, 9 sangraram à 1.ª; M11 (losango pintado −0,5 s) sobrevivia a tudo → gate novo
  (o losango a < 1 px da linha da cabeça), sangra; M5 (`begin` sem limpar `moved`) equivalente →
  a linha saiu.

## ⛔ Recusas

| Recusada | Porquê |
|---|---|
| Losango e clique no INÍCIO da etapa | a cabeça mostraria a tela ANTES da pincelada; o painel lista o fim |
| Arredondar o fim ao quadro (Scrub) | 1,63 → 1,625 mostra a etapa por acabar e fica fora do losango |
| Refazer o movimento a partir do estado do `begin` a cada quadro | perderia qualquer outra edição feita no mesmo bracket (o bracket é partilhado por todos os gestos da Timeline) |
| Absorção com tolerância / só no fim sem registar as movidas | a seleção pode mudar; a lei «a movida ganha» precisa de saber QUEM se moveu |
