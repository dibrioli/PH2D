---
name: feedback_an_order_with_two_halves_can_be_obeyed_only_in_the_half_that_removes
description: "Uma ordem do dono que MOVE uma coisa tem duas metades — tirar e pôr; cumprir só a que tira apaga a capacidade em silêncio, e a lei órfã que fica passa em todos os gates"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e990a2f5-7d16-405a-8ddf-54393edf203d
  modified: 2026-09-19T18:34:58.905Z
---

⛔⛔⛔ **Uma ordem que MOVE uma capacidade tem duas metades — RETIRAR e CONSTRUIR — e elas caem em
waves diferentes.** Cumprir só a que retira apaga a capacidade, e o app fica sem ela **em silêncio**:
a lei continua viva, continua gateada, e ninguém a pode accionar.

**Medido 2026-09-19** (`line/UIUX`):

> *«Vamos retirar a opção de colapsar arrastando. Deixa o colapsar apenas no menu da barra
> superior.»* — Enio, 2026-09-09

O gesto da borda saiu na wave seguinte. **O item de menu nunca foi escrito.** Durante dez dias o
artista não tinha maneira nenhuma de fechar uma coluna lateral — e a `dock_columns::close`, com três
gates a provar a involução dela, tinha **ZERO chamadores de produto**.

**Why:**
- ⛔ **Nenhuma sonda deste repo pergunta se uma PORTA tem chamador.** Um `pub fn` sem consumidor
  compila, passa o clippy, e os testes dele ficam verdes — eles chamam-na directamente.
- ⛔⛔ **E a metade que faltava estava ESCRITA no roteador** (*«quem construir o item de menu herda a
  involução pronta»*) desde o dia seguinte. *Uma nota não é um gate*: ela descreve a ausência para
  quem já sabe que ela existe.
- ⛔⛔ **Uma lei escrita num doc-comment e não gateada é uma nota — e uma função sem chamador NUNCA
  é medida**, logo o doc dela é a única defesa e ele não defende nada. Aqui: o doc do `close` dizia
  que fechar tem de passar a *ESCOLHA* de largura (`None` quando ninguém arrastou) e nunca o número
  (que devolve o default). A mutação que trocava os dois **sobreviveu à suíte inteira** — porque
  ninguém chamava a função.

**How to apply:**
- ao cumprir uma ordem que MOVE, escreva as duas metades no mesmo commit, ou deixe a metade que
  falta como uma **acusação executável** (um gate `#[ignore]` com o nome do que falta, uma entrada
  numa catraca) — nunca só uma frase num roteador;
- ⭐ o sinal barato: depois de retirar um gesto, **conte os chamadores de produto da lei dele**. Zero
  é um defeito, não uma limpeza;
- ⭐ e quando ligar uma lei órfã, **corra a prova de mutação sobre o doc-comment dela**: as leis que
  nunca foram exercitadas por um caminho de produto são exactamente as que ninguém gateou.

Irmãs: [[feedback_a_rename_probe_is_void_if_the_check_stops_at_the_first_error]] ·
[[reference_topic_control_design_hazards]] · [[reference_topic_gate_discipline]]
