---
name: feedback_a_smoke_step_that_needs_three_decisions_at_once_is_not_a_step
description: "«O smoke não foi claro»: o defeito mede-se contando DECISÕES por passo — e a cena que ensina bem só ensina UMA coisa"
metadata:
  type: feedback
---

Quando o dono diz *«não foi claro o suficiente para eu entender»* sobre um smoke, a tentação é
reescrever o texto. A régua que serve é **contar quantas decisões cada passo pede**, e a resposta
costuma ser que o problema é a CENA, não a redacção.

Medido em 2026-09-06 (`line/components`). O passo reprovado dizia:

> *«Na lista da esquerda abra a linha `Car` — é a receita, as cópias chamam-se `Car (1)` e
> `Car (2)` — e apague o `Body` dela.»*

Três decisões num passo: **qual** das três linhas parecidas é a receita · **que** é ali que se apaga
(e como) · e depois **ler um cartão** noutro painel para ver o efeito. Cada uma tem uma forma de
correr mal, e o dono não tem como saber qual delas correu.

⇒ a cura foi uma **cena nova** com estas propriedades, e não um texto novo:

1. **Um gesto por passo.** Clicar · arrastar · escolher um item de menu · clicar num botão.
2. **O passo diz ONDE ele acontece** — *«(na TELA)»* / *«(na LISTA da esquerda)»*. Numa app com
   canvas e painéis, metade da confusão é o artista procurar no sítio errado.
3. **O sujeito está sempre visível** enquanto se opera nele. Um passo cujo sujeito é uma linha de
   lista que ele tem de encontrar já falhou.
4. **A cena ensina UMA coisa.** A anterior já ensinava a escada do *Aplicar*; empilhar-lhe um
   segundo assunto fez o texto crescer sem o tornar mais claro.
5. **Elementos distinguíveis por FORMA e COR**, e afastados o suficiente para um clique de canvas
   os separar — com gate a medir a folga, senão ela desaparece na primeira afinação de layout.

**Why:** o smoke é onde o dono **aprende a ferramenta** (CLAUDE.md §0.8). Um passo com três
decisões testa se ele adivinha a mesma coisa que quem escreveu — e quando falha, o report que volta
é *«não funcionou»*, que é indistinguível de um defeito real e custa uma jornada a bissecar.

**How to apply:** antes de mandar um smoke, releia cada passo e conte os verbos e os lugares. Se um
passo tem dois de qualquer um deles, ele são dois passos — ou a cena está errada. ⚠️ E quando o
report é sobre o smoke e não sobre o produto, **construa a cena**: reescrever o texto sobre uma cena
que exige a decisão difícil só torna a instrução mais longa. Ver
[[feedback_ready_to_smoke_example]] e [[feedback_communication_simplicity]].
