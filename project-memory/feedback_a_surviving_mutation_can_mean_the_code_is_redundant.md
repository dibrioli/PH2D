---
name: feedback-a-surviving-mutation-can-mean-the-code-is-redundant
description: "Uma mutação que SOBREVIVE tem duas leituras — falta um gate, ou a linha era código a mais; pergunte primeiro se algum outro caminho já garante o que ela faz"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-03T01:27:11.115Z
---

Quando uma prova de mutação sobrevive, o reflexo é *«falta um gate»*. ⚠️ **Há uma segunda leitura, e
ela é a melhor notícia das duas: a linha mutada era REDUNDANTE** — outro caminho já garantia o que
ela fazia, e nenhum teste podia ver a diferença porque não havia diferença.

Caso medido (PH2D, `line/3DModeling`, 2026-09-02 — o menu do cabeçalho de cada vista): apagar o
`s.active = i` do ramo que **escolhe** uma linha do menu não matava gate nenhum. A causa não era um
buraco na cobertura: **quem acerta o quadrante comandado é o clique no CHIP**, que é a única porta
que abre aquele menu — quando uma linha é escolhida, o activo já é o certo. A mutação irmã (apagar a
linha **do chip**) mata o gate da costura à primeira.

**How to apply:**
- ⭐⭐ Antes de escrever o gate que falta, pergunte **«que outro caminho já garante isto?»**. Se
  houver um, a cura é **apagar a linha** e gatear o caminho verdadeiro — e não acrescentar um teste
  que passa a defender duas cópias da mesma lei.
- ⭐ Deixe uma `debug_assert` no sítio da linha apagada, a dizer de quem é a garantia. *Uma segunda
  cura que nenhuma mutação mata é código que ninguém pode remover com confiança*
  ([[feedback_i_write_the_right_guard_and_do_not_gate_it]] é o caso oposto: ali a guarda estava
  certa e sem gate).
- ⚠️ **A régua move-se para onde a lei acontece.** Aqui o gate deixou de medir o activo depois de
  *escolher* e passou a medi-lo depois de **abrir**, que é o instante em que a decisão é tomada.

---

⭐⭐ **TERCEIRA leitura, e ela não é «falta um gate» nem «é redundante»: O MUNDO AINDA NÃO TEM O
CASO QUE A DISTINGUE** (2026-09-02).

Uma cerca exigia que o nó-conversor sugerido **aceitasse o tipo que a fonte emite**. A mutação
que a apaga (`i.ty == saida` → `is_some()`) **sobreviveu**. Nenhuma fixtura a matava — e a razão
não era o gate: o catálogo inteiro daquele módulo tem **uma só** forma não-conduzível
(`Instances/Vec2`; o censo diz `Domain::Instances` em **100% das 138 portas**), então a cerca
não tinha um segundo caso contra o qual discriminar.

⇒ Ela **fica**, porque é correcta, e torna-se falsificável no dia em que um segundo domínio
entrar. O que muda é o doc-comment: a limitação passa a estar **declarada**, com o número que a
explica.

**Why:** as duas leituras habituais levam a agir — escrever um gate, ou apagar a linha. Nesta
terceira, **as duas acções estão erradas**: um gate impossível de escrever hoje, e apagar uma
guarda correcta porque o mundo é pequeno.

**How to apply:** quando uma mutação sobrevive, conte a POPULAÇÃO que a linha discrimina antes
de decidir. Se ela é 1, a linha é uma guarda para o futuro — declare-a no doc-comment com a
contagem, em vez de a apagar ou de fingir cobertura.
