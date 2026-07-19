---
name: feedback-a-parameter-that-changes-nothing-is-discarded-downstream
description: "Quando um controle 'não faz diferença nenhuma', suspeite de que o resultado dele é DESCARTADO a jusante — não de que ele está fraco"
metadata:
  type: feedback
---

Enio, smoke do Flip (2026-07-18): *"**independente do valor de gap ou trap** o fill se ajusta
perfeitamente à linha até o momento em que se sobreponham duas linhas"*. Essa frase continha o
diagnóstico inteiro.

Dois parâmetros independentes ficarem **simultaneamente** sem efeito não é fraqueza dos dois: é
sinal de que existe, a jusante, um caminho que **joga fora o que eles produzem**. Era o
`filled_shape_target` — ele roda depois do solver e, quando dispara, descarta o contorno
traçado (que é o que Gap e Trap movem) e pinta o polígono do próprio traço.

**Why:** a reação natural a "o slider não resolve" é calibrar o slider — subir faixa, mudar
default, procurar a constante certa. Isso é procurar no lugar onde o número *entra*, quando a
informação está no lugar onde ele **deixa de importar**. Um parâmetro sem efeito é uma
afirmação sobre o GRAFO de dados, não sobre o valor.

**How to apply:** ao ouvir "mexer nisso não muda nada", **grepe o consumidor do resultado
antes de mexer no produtor** — procure um ramo a jusante que substitua, descarte ou
curto-circuite a saída. Se **dois** controles independentes morrem juntos, o ramo comum é quase
certo. Irmão de [[feedback_ergonomics_verdict_is_a_design_bug]] (parar de calibrar e questionar
o modelo) e de [[feedback_tool_unit_green_integration_dead]].

Corolário do mesmo bug: **área é um proxy fraco de "é a mesma região"** — o critério
descartador comparava áreas com 15% de tolerância, e a forma quebrada passava com 0,7% (o
shoelace de um polígono que se CRUZA é soma algébrica com sinais que se cancelam, não a área
pintada). Duas formas bem diferentes têm a mesma área; medir a **distância entre as curvas**
separou os casos por 150×. Ver [[feedback_test_with_product_numbers_not_convenient_ones]].
