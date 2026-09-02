---
name: feedback_a_bucket_nobody_fills_reads_as_perfect
description: Uma régua que agrega (mediana/média) devolve ZERO quando o balde está vazio, e zero lê-se como «perfeito» — ponha a CONTAGEM ao lado, e nunca deixe um `continue` saltar a escrituração do laço
metadata:
  type: feedback
---

Uma régua que **agrega** — mediana, média, percentil — devolve `0` sobre um balde
**vazio**, e `0` é indistinguível de *«medi e está perfeito»*. **Imprima a contagem ao
lado do valor, sempre.**

⛔ **E a forma nº 1 de esvaziar um balde sem dar por isso é um `continue` a meio de um
laço cuja escrituração vive no fim.** Quem escreveu a contabilidade tinha um caminho em
mente; a ramificação seguinte salta-a, e nada avisa. *Use `else` — ele não tem como
armar a armadilha.*

**Why:** medido no `ph2d-quadfill` (2026-08-23). Dois campos separavam o enviesamento
por valência do patch; o caminho do patch de 4 lados saía do laço por `continue` antes
da escrituração ⇒ (a) o balde do domínio dos rectângulos ficava **sempre vazio** e
imprimia `0,0°`, (b) o vector de etiquetas não era estendido e as faces do rectângulo
eram rotuladas *leque* pelo primeiro leque a seguir. Sobre o `0,0°` escreveu-se uma
conclusão inteira — *«a grade do rectângulo nasce perfeita no domínio e chega torta à
superfície ⇒ são dois defeitos em duas fases»* — e um dia de hipóteses saiu dela. Os
números reais eram `1,0°` e `16°`, não `0,0°` e `12°`. ⚠️ A régua irmã até **trazia o
aviso no doc** («0 não é perfeito, é não medido») — para a coluna dela. *Um aviso escrito
ao lado de uma régua não protege a régua do lado.*

**How to apply:**
1. Toda mediana/média num relatório leva a **contagem da amostra** no mesmo struct e na
   mesma linha de log. Um inteiro a zero não se disfarça de bom resultado.
2. Uma função que recebe uma etiquetagem por índice **recusa alto** (`NaN`, `None`,
   `Err`) quando ela não cobre a colecção — nunca `unwrap_or(default)`, que atira os
   não-etiquetados para uma das colunas em silêncio.
3. Contabilidade no fim de um laço ⇒ **`else`, nunca `continue`**. E uma cerca
   (`debug_assert` de cobertura) por cima.
4. Antes de construir sobre um número surpreendente, **prove que ele foi medido** — o
   par numerador/denominador é o teste (`slid: 5/5`, `flattened: 19/19`).

Irmãs: [[feedback_an_unlabelled_probe_column_gets_read_backwards]] ·
[[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]] ·
[[feedback_a_new_features_gate_can_expose_a_pre_existing_bug_check_the_control_first]] ·
[[reference_topic_gate_discipline]]

⚠️ **Refinamento (2026-08-23) — a TERCEIRA variante da mesma lei, na mesma semana: um
numerador sem MOTIVO.** Depois de «mediana sem contagem» e «numerador sem denominador»
veio `deslizou 1/2` — e eu escrevi por cima dele *«não é o mapa»*, tendo o mapa corrido
em **um patch de seis** (ele só se aplica a patches de quatro lados, e a peça tinha
`{3:4, 4:2}`). ⭐ **A cura foi a mesma três vezes: acrescentar a coluna que falta.** Com
`slid_refused` a linha passou a dizer `1/2 · recusas [0,0,0,1,0]`, e a coluna `3` —
*fronteira livre não-monótona* — mudou a conclusão de *«a cura é insuficiente»* para
*«a cura RECUSOU-SE a correr»*. **São afirmações diferentes e mandam construir coisas
diferentes.**

⇒ ⭐ **A regra completa, nas três formas:** ao lado de todo número derivado ponha
*quantos* (a amostra), *de quantos* (o denominador) e, quando ele é o resultado de um
predicado, *porquê não* (o motivo da recusa). **Um número que passou por um filtro e não
diz qual filtro é uma conclusão à espera de dono.**

⚠️ **Refinamento (2026-08-23) — a QUARTA variante, no mesmo dia: um RECUO sem voz.** Um
caminho de cura devolvia `None` num patch e o chamador caía no comportamento de sempre —
**byte-idêntico ao controlo, sem uma palavra**. ⛔ E aconteceu **três vezes seguidas no
mesmo ficheiro, com três causas diferentes** (uma circularidade · um `?` a abortar a
função inteira · um `return` que não preenchia um campo novo). ⭐⭐ **O que as separou não
foi raciocínio — foi acrescentar colunas, uma de cada vez:** o **numerador** (quantos
arcos mudaram) apanhou a segunda; o **motivo** da desistência, por passo (`gave_up:
[usize; 5]`), apanhou a terceira **numa corrida** — a coluna dizia `sem alfa 57`.

⇒ ⭐ **A lei, completa em quatro formas.** Ao lado de todo número derivado ponha:
*quantos* (a amostra) · *de quantos* (o denominador) · *porquê não* (o motivo da recusa)
· e, se há um caminho de recuo, **que ele CONTE**. ⛔ *«Byte-idêntico ao controlo» nunca é
um resultado — é uma pergunta*, e sem essas colunas ela não tem resposta.

⚠️ **E há um corolário sobre campos novos:** quem acrescenta um campo tem de o preencher
em **todos** os `return` do construtor (eram quatro), e o compilador não ajuda quando o
tipo tem `Default`. *O gate de presença é o que apanha o que faltou.*

⚠️ **2026-08-31 — e a variante que eu própria construí, no ficheiro escrito para curar isto.**
A régua nova (`tip_deviation`) media a distância da escultura à saída junto de cada ponta. Uma
ponta comida **por inteiro** não tem superfície junto do ápice ⇒ não há amostra ⇒ ela era
**saltada**, e o relatório dizia `0 de 3 pontas acima da barra` sobre um espinho amputado em
**`−46,6 %`**. *O balde vazio não era ausência de informação: era o defeito máximo.*
⇒ **Lei:** uma régua escrita a partir dos casos PARCIAIS (os que a foto mostrava) tem de ser
exercitada pelo caso TOTAL antes de alimentar uma decisão — e a cura é registar o **piso do
que se sabe** (aqui, o raio da busca: *«mais longe do que eu olhei»*), nunca `continue`.
⚠️ E o caso **vizinho** fica a saltar de propósito: quando é a ENTRADA que não tem amostra,
acusar mediria a fixtura. *Os dois lêem-se igual no código e são opostos.*
