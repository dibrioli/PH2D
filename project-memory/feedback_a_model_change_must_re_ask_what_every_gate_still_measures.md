---
name: a-model-change-must-re-ask-what-every-gate-still-measures
description: Mudar o modelo pode deixar um gate verde sobre um dano que já não é exprimível — a régua tem de seguir o dano
metadata:
  type: feedback
---

Vector / máquina de estados do Morph, 2026-08-25 (W10). O modelo passou de `n(n-1)`
arestas com condição por passagem para `n` estados com a tecla no destino.

Um gate media *"uma tecla segurada não re-dispara"* com uma **cadeia**
`A --jump--> B --jump--> C`: com `pressed` em vez de `just_pressed`, a máquina saltava a
cadeia inteira num quadro.

**Why:** essa cadeia deixou de ser **exprimível** — uma tecla passou a nomear UMA forma. A
mutação `just_pressed → pressed` passaria a **SOBREVIVER**: o segundo disparo é recusado
por já se estar em `B`, e nada observável muda. O gate ficava verde, com o nome certo, a
medir coisa nenhuma. *O dano mudou de forma, e a régua não seguiu.*

O dano real sob o modelo novo é outro: com `pressed`, uma tecla segurada **PINA** a máquina
naquela forma — toda outra transição é desfeita no quadro seguinte.

**How to apply:**
- Depois de mudar um modelo, percorra os gates dele e pergunte de cada um: *que mutação o
  mata hoje?* Não *"ele ainda compila"*, nem *"ele ainda passa"* — os dois são compatíveis
  com ele já não medir nada.
- O sinal barato: **re-corra as mutações antigas**. Uma que sobrevive é um gate cujo alvo o
  modelo dissolveu.
- Quando o dano mudar de forma, **reescreva a fixtura**, não só o nome. Aqui: segurar `jump`,
  carregar `dash`, exigir que a máquina FIQUE em C.
- Corolário barato de aplicar: uma mudança de modelo é também a janela em que **quebrar o
  formato custa zero** — mas só enquanto nada gravado o contém. Meça isso (`git log main --
  <ficheiro>`, procurar ficheiros de projecto) antes de decidir adiar.

Ver [[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]] ·
[[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]] ·
[[feedback-the-ceiling-is-the-hardwares-never-the-fallbacks]]
