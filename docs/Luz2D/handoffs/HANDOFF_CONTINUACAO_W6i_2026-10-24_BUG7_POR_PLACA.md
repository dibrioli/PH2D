# HANDOFF de continuação — Luz 2D W6i: o bug 7 por placa (24/10/2026)

Base: HEAD da linha `c46ae5b4ff` (`line/Luz2D`); main `972f411a6f`. Anterior:
[W6h](HANDOFF_CONTINUACAO_W6h_2026-10-23_O_CUSTO_DOS_CANTOS.md).

## 1. O que a W6i fez

Fechou o bug 7 (7 testes da Luz 2D vermelhos numa placa integrada): os 7 passam na Radeon (RADV) e na RTX 5060 Ti.
Quatro famílias: o «mesmo bit» passou a lei por placa (decisão do dono 24/10), o G5 ganhou lei de variação em vez
de teto fixo, a holográfica ganhou f16 ao mais próximo ligado só onde a placa trunca (medido uma vez por
adaptador), e a lei de custo do miolo só vale onde os despachos cortados pesam ≥ 20 %. Commits `2e7bfd84b9` ·
`a047753a1b` · `c46ae5b4ff`. Mecanismo, números, prova de mutação e reprodução:
[BUGS_luz2d.md §7](../BUGS_luz2d.md). A Luz 2D pode correr na fila da placa onboard.

### ⛔ Recusas MEDIDAS (W6i)

| Recusa | Porquê (medido) |
|---|---|
| Correção do f16 da holográfica sempre ligada | +6–13 % do c2 na RTX, acima do critério de 2 %; só onde a placa trunca |
| Exigir 0 texels entre formas em toda placa | Decisão do dono: seria lutar contra o compilador de cada placa (RADV reescreve a conta; morre com `ACO_DEBUG=noopt`) |
| Teto fixo do G5 calibrado na NVIDIA aplicado a toda placa | O arranjo em ladrilhos da RADV passa-o em quase toda textura |
| Corrigir o formato 8192×8 da `hrc_r0` para a AMD | São 512 KiB só na AMD; não compensa |

## 2. ABERTO (por ordem — substitui o W6h §14)

1. A conta do `de_fora` em n 11–12 a 2160p Alta (~2,8 ms; W6h §14.2).
2. O regime da soma da vista P (W6h §13 b).
3. A anomalia da r5b (W6g §12) e a família (b) (W5g §37–38).
4. O miolo na integrada ~11 ms (15× a RTX): trabalho e não despachos — por medir se vale.
5. A suíte no fecho da linha (não corre desde o W6g §56).
