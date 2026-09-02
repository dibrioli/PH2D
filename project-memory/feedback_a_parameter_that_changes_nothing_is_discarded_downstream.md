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


## O caso mais barato de todos: a coluna PUBLICADA com o nome que ninguém lê

**L-System, 2026-08-30.** Enio: *"as folhas … sem rotação [relativa] ao galho"*. A membrana
calculava o ângulo certo, tirava-o da coluna certa do esqueleto, e publicava-o numa coluna
chamada **`rotation`**. A convenção de instâncias do Motion chama-lhe **`rot`**.

⛔ **Um nome de coluna errado não é um erro** — num stream de colunas nomeadas, o consumidor
pede `get("rot")`, recebe `None`, e usa o DEFAULT. Compila, corre, desenha; a rotação é a
identidade. É a mesma família do param descartado a jusante, na sua forma mais barata: o valor
não é *descartado por uma decisão*, é **nunca procurado**.

⚠️ **E é por isso que o gate tem de atravessar até ao CONSUMIDOR.** Todos os gates que eu tinha
liam a coluna publicada — todos passavam. O que mata a mutação é baixar a corrente pela função
real do lowering e medir a INSTÂNCIA (aqui, a `basis`).

⚠️ **Corolário, e este quase me escapou:** a mutação que troca a FONTE (ler `wrot` em vez de
`rot`) **sobrevive** no caso comum, porque naquele default as duas colunas trazem o mesmo
número. Só o modo em que elas divergem torna a escolha observável — foi preciso um gate no
`Orient = Local` para a lei ter cerca.

**How to apply:** ao publicar num canal com chaves por NOME (colunas de stream, externals,
mapas de props), grepe o nome no consumidor no mesmo minuto em que o escreve — o compilador não
o faz. E escreva pelo menos um gate que **corra o consumidor real**, não que releia o que você
acabou de escrever. Relacionado:
[[feedback_a_promise_that_justifies_a_decision_must_have_a_reader]] ·
[[feedback_paint_and_dispatch_must_read_the_same_source]] ·
[[feedback_a_dead_knob_has_two_species_no_probe_catches]].
