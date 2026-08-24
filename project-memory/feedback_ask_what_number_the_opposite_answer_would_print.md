---
name: feedback_ask_what_number_the_opposite_answer_would_print
description: Antes de acreditar no veredicto de uma sonda, calcule que valor a resposta CONTRÁRIA teria impresso — se ele estiver fora do alcance da grandeza, a sonda nunca testou a pergunta, e um valor encostado ao tecto dela é saturação, não defeito
metadata:
  type: feedback
---

⭐⭐⭐ **Antes de acreditar no veredicto de uma sonda, calcule que número a resposta
CONTRÁRIA teria impresso.** Se esse número está **fora do alcance** da grandeza, a
sonda nunca testou a pergunta — ela respondeu outra.

⚠️ E o corolário que se lê de graça: **um valor encostado ao máximo que a grandeza
consegue imprimir é um instrumento saturado, não um defeito grande.**

**Why:** medido no quad remesh (2026-08-23). Uma sonda chamada `holonomy` respondia
*«há singularidade dentro dos nossos patches, e a dívida é da fase anterior»*, e essa
frase estava escrita em **dois** doc-comments a prescrever a obra seguinte. Ela media
`29°` na orelha e `44°` no gancho.

⛔ **O que ela media era o resto depois de virar cada braço da cruz para o quarto de
volta mais próximo — limitado a `45°` por construção.** Uma singularidade dá `90°`,
que aquela linha **nunca teve como escrever**. E `29°`–`44°` não era «grande»: era o
*tecto*.

⛔⛔ **Pior do que o tecto era o ramo que faltava.** A holonomia só existe na aresta
que **fecha ciclo**; nas outras o valor do filho foi *definido* como o mais próximo do
pai, logo o desacordo é zero por definição. A sonda, no ramo que fecha, comparava o
braço **cru** do vizinho em vez do já penteado. *O teste de fecho não estava saturado
— ele não existia.*

⭐⭐⭐ **E a régua não estava só cega: estava ao contrário.** Provado por mutação, com
uma singularidade de índice `+¼` fabricada num leque plano:

| | régua antiga | ⭐ régua nova |
|---|---|---|
| singularidade **de verdade** | `11,25°` · `0` defeitos | `1` defeito, `1` quarto de volta |
| campo limpo mas irregular (patches reais) | `29°`–`44°` | — |

⇒ *Ela dava à singularidade a sério um número **menor** do que dava a campo sem
singularidade nenhuma.*

⚠️ **A conclusão que caiu não é a que parece.** Uma nota anterior dizia *«o raciocínio
é limpo e a medição desmente-o»* sobre a ideia de que uma barra entre `1°` e `10°`
separaria as duas classes. ⭐ **O raciocínio estava CERTO**; o que foi testado é que
*aquela grandeza* não separa. **Uma refutação vale exactamente o que a régua consegue
exprimir dos dois lados.**

**How to apply:**
1. ⭐ **Escreva o número da resposta contrária antes de ler o resultado.** «Se houvesse
   singularidade, esta coluna diria `90`.» Se `90` não cabe na grandeza, pare — a
   sonda está a responder outra pergunta.
2. ⚠️ **Confira o TECTO da grandeza.** Um `round` para o múltiplo mais próximo limita o
   resto a metade do passo; uma norma limita-se ao maior termo; uma fracção a `1`. Um
   valor a `95%` do tecto é o instrumento a falar, não a peça.
3. ⭐⭐ **Fabrique o fenómeno num controlo positivo** e meça-o com o **mesmo código**.
   Aqui foi um leque **plano** de 8 triângulos com o campo a dar exactamente um quarto
   de volta — plano de propósito, para o defeito angular de Gauss não se somar ao do
   campo. *A malha mais pequena com um vértice interior já tem um ciclo dual, e um
   ciclo é tudo o que a holonomia precisa.*
4. ⛔ **Uma asserção que não pode reprovar é a rede que faltou.** O gate que devia ter
   apanhado isto dizia `assert!(holonomia >= 0.0)` — sobre um **ângulo**, não-negativo
   por construção. Ele passava para qualquer valor, **incluindo o zero que significa
   «não mediu nada»**. Prove por mutação que o gate novo fica vermelho com a lei
   antiga reposta.
5. ⚠️ **Quando corrigir a grandeza, corrija o NOME.** A antiga sobrevive com valor —
   como `rough_*`, a rugosidade do campo — mas com o nome dela. *Duas grandezas com um
   nome só foi a doença; guardar o nome errado no sobrevivente seria mantê-la.*
6. ⭐⭐⭐ **Se a conclusão é sobre uma SEQUÊNCIA, uma amostra de um termo não a pode
   exibir.** *«O ponto fixo nem sequer contrai»* foi escrito a partir de `21,4 → 21,3`
   — **uma** varredura, num laço que corria uma vez. Medida a sequência a sério, ela
   contrai **por exactamente ½ por ronda** (`0,185 · 0,060 · 0,030 · 0,015 · 0,0075 ·
   0,0038`). ⚠️ E a armadilha por baixo era outra: cada varredura calculava a proposta
   a partir do estado que estava a substituir, ⇒ *a segunda varredura vê outra coisa, e
   nunca chegou a correr.* **Antes de dizer «não converge», corra duas.**

7. ⭐⭐⭐ **E antes de perguntar se um número é inteiro, zero ou grande, veja se ele é
   INVARIANTE.** Uma grandeza de **calibre** — uma que muda quando se muda uma escolha
   arbitrária do modelo, sem mudar nada no resultado — não responde a pergunta nenhuma
   sobre a peça. *Medido no mapa global: a translação de uma costura muda toda quando se
   soma uma constante ao `(u,v)` de um patch; a distância a inteiro dela deu `0,408`, e a
   grandeza certa — a volta a um ciclo, depois de a árvore ser levada a zero — deu
   `0,291`.* ⚠️ **A assinatura:** *se forçar a grandeza a mudar não muda o resultado, ela
   não é o resultado* — pregar as translações arredondadas deixou o ângulo em `2,9°`.

⇒ ⭐ **As três faces da mesma lei:** a régua tem de conseguir exprimir a resposta
(**alcance**), sobre amostras que a contenham (**extensão**), e não pode depender do que
não importa (**invariância**).

Irmãs: [[feedback_an_unlabelled_probe_column_gets_read_backwards]] ·
[[feedback_a_correct_mechanism_can_prescribe_the_wrong_cure]] ·
[[feedback_a_better_instrument_can_make_the_product_worse_and_that_is_the_finding]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[reference_topic_gate_discipline]]
