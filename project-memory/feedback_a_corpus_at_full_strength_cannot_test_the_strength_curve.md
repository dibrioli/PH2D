---
name: a-corpus-at-full-strength-cannot-test-the-strength-curve
description: A lei da força do modo B estava errada havia um mês e toda fixture passava — porque s² = s⁴ = 1 na força cheia
metadata:
  type: feedback
---

Medido em 2026-09-13 (`line/sculpt3d`, os gestos de puxar): o modo de referência
do Blender aplicava `StrengthCurve::Squared` a **todos** os verbos; a referência
aplica-a **por verbo** (o agarrar e o gancho são lineares). Com o slider a `0,4`
o oráculo move `0,200000` e nós movíamos `0,071414` — `0,4²` contra `0,4`.

**Why:** o corpus inteiro corria a **força `1,0`**, onde `s = s² = s⁴`. As
fixtures não podiam distinguir uma lei da outra, e a suíte do repo ficou **verde
com a troca da lei** — nenhum teste guardava aquele número. O mesmo mecanismo
mordeu duas vezes no mesmo dia: o meu primeiro polegar multiplicava a força uma
vez a mais (a quarta potência) e **as fixtures de força cheia passavam todas**;
quem o apanhou foi a **única** fixture de meia força.

**How to apply:** ao portar uma lei que tem um KNOB, exija no corpus pelo menos
**duas** posições dele — e prefira uma que não seja o neutro nem o máximo, porque
é aí que `x`, `x²` e `x⁴` deixam de coincidir. Quando não houver fixture fora do
neutro, o gate honesto é a **RAZÃO entre duas corridas nossas** comparada com a
razão do lado aprovado: ela é imune às opções que ainda não implementámos. É a
irmã da lição *«um corpus no neutro de um knob não testa esse knob»*
([[reference-topic-measurement-discipline]]) — aqui o ponto cego não é o neutro,
é o **máximo**.
