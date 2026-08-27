---
name: feedback-a-denominator-above-the-curvature-is-slow-below-is-inf
description: Num relaxador de Gauss-Seidel, um denominador ACIMA da curvatura sub-relaxa (lento, convergente) e um ABAIXO diverge — errar para cima e' lento, errar para baixo e' inf.
metadata:
  type: feedback
---

Todo passo `Δ = gradiente / denominador` de um relaxador coordenada-a-coordenada tem uma
barra assimétrica: **acima da curvatura verdadeira sub-relaxa** (converge, devagar);
**abaixo sobre-relaxa**, e a `ω > 2` diverge. *Errar para cima é lento; errar para baixo
é `inf`.*

**Why:** medido no `ph2d-gridmap` (2026-08-27). A `relax_tie` dividia por
`Σ den[classe]` — a curvatura de cada membro **em isolamento** — enquanto os membros eram
incógnitas **livres** que movem dezenas de dependentes. `H/H_fingida` deu **p50 39×, max
81×** ⇒ a esfera ia a `NaN`. ⚠️ E a Hessiana "correcta" também **não é exacta**: quando o
numerador de uma célula depende das **vizinhas**, mover muitas de uma vez tem termos
cruzados negativos que a soma ignora (aqui `1,46×` acima da efectiva) — *e é isso que a
torna segura*.

**How to apply:** ao escrever um passo novo neste feitio, não persiga a igualdade — prove
a **desigualdade** (`denominador ≥ curvatura efectiva`), que é o que separa lento de `inf`.
A curvatura efectiva mede-se: `(gradiente_antes − gradiente_depois) / andado`. Um doc que
promete «minimização exacta ao longo daquela coordenada» está optimista sempre que o passo
move mais do que uma variável — [[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]].
Ver também [[feedback-a-gate-on-the-reported-quantity-is-green-when-the-product-divides-by-the-other]].
