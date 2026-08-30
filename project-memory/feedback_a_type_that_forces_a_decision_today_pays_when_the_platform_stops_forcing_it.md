---
name: feedback_a_type_that_forces_a_decision_today_pays_when_the_platform_stops_forcing_it
description: Quando duas coisas distintas se fundem num tipo só, toda decisão que o tipo antigo OBRIGAVA a tomar passa a ser herdada em silêncio — e é a disciplina antiga que decide se isso é grátis ou catastrófico
metadata:
  type: feedback
---

⭐⭐ **Quando uma plataforma funde dois tipos num só, toda escolha que o tipo antigo
OBRIGAVA a fazer passa a ser herdada em silêncio.** Se a escolha estava certa, é grátis;
se estava errada em algum sítio, esse sítio **compila** e muda de sentido.

**Medido (2026-08-29, `rapier2d` 0.31 → 0.35).** No `nalgebra`, `Point2` e `Vector2` são
tipos **distintos de propósito** — um ponto é um lugar, um vetor é um deslocamento — e
`Isometry2 * Vector2` **só roda**, enquanto `Isometry2 * Point2` roda **e** translada.
No `glam` os dois são o **mesmo tipo**, logo `Pose2 * Vec2` é **sempre**
`transform_point`. ⇒ todo sítio que multiplicava uma pose por uma direcção passaria a
ganhar uma translação — **e compila**. Era a única classe de defeito silencioso da
migração inteira.

⭐ **Resultado: exposição ZERO em 119 ficheiros — e não por sorte.** Enquanto o
`nalgebra` distinguia os tipos, o código **era obrigado** a escolher em cada sítio, e
escolheu certo em todos. A fusão herdou a escolha. *Um tipo que force uma decisão hoje
paga-se quando a plataforma deixar de a forçar amanhã.*

⚠️ **A varredura tem de ser POR OPERAÇÃO, não por ficheiro, e com CONTROLO.** Classifique
cada multiplicação como *era-ponto* ou *era-direcção*; se a varredura não achar **nem um
caso de cada lado**, ela está partida, não limpa.

**Why:** o instinto é varrer os ficheiros que deixaram de compilar. Os 116 erros de
compilação eram os **seguros**. O risco vivia inteiro no que continuou a compilar.

**How to apply:** em toda migração que **funde** ou **apaga** um tipo, pergunte que
decisão o tipo antigo obrigava a tomar, e varra essa decisão — não os erros.
