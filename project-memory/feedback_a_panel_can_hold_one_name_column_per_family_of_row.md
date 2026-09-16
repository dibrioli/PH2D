---
name: feedback_a_panel_can_hold_one_name_column_per_family_of_row
description: "Um painel pode ter uma coluna de nome por FAMÍLIA de linha (caixa · chip · número), as três dentro do mesmo cartão — e a varredura que converte o app tem de ser por família, nunca por painel «já convertido»"
metadata:
  type: feedback
---

Quando um app converte as linhas dele a um manual de UI, a pergunta *«este painel já foi
convertido?»* é a errada. A unidade que diverge é a **FAMÍLIA DE LINHA**, e um painel grande tem
várias — cada uma convertida (ou não) numa wave diferente, por uma pessoa diferente, com um número
diferente.

Caso medido (`line/UIUX`, 2026-09-16, o painel do Painter — **três** respostas à mesma pergunta
dentro do MESMO cartão):

| família de linha | o que ela usava | onde o nome saía |
|---|---|---|
| caixa de marcar | o default da porta | na **metade cega** da faixa |
| rótulo + chip · amostra de cor | um literal `LABEL_W = 60,0` | encostado à **esquerda** |
| número | um `fn seccao*()` por ficheiro, **seis** deles | na coluna da secção ✅ |

⇒ *duas colunas de nome alternando linha sim linha não* — exactamente o defeito que a spec daquele
app tinha escrito, um nível abaixo (entre linhas). Ninguém o via porque **cada família tinha sido
tratada isoladamente e declarada feita**.

⚠️ **E as SEIS declarações por ficheiro eram a segunda resposta, não a primeira.** Elas tinham sido
escritas para curar um report do dono sobre os NÚMEROS; as caixas e os chips da mesma secção não as
conheciam. *Duas derivações da mesma coluna são duas colunas, e elas divergem onde mais se nota:
dentro de um cartão.*

**Why:** o dono não vê famílias de linha — ele vê uma coluna torta. E a conversão parcial é **pior
que nenhuma**: com tudo na metade cega as linhas pelo menos alinham entre si.

**How to apply:**
1. Ao converter um painel, **enumere as FAMÍLIAS de linha dele** (o que pinta um nome à esquerda de
   um controlo) antes de tocar em qualquer uma. Uma família não convertida ao lado de uma convertida
   é raggedness nova.
2. A declaração da secção vive **num sítio só** por painel, e as declarações que já existirem
   **delegam** nela — nunca coexistem.
3. Deixe a porta receber a **chave** e derivar a secção, em vez de cada sítio escolher a sua: `~45`
   escolhas não podem estar todas certas, e nenhuma régua de registo as vê.
4. ⚠️ **A régua do censo enumera por NOME e envelhece com a grafia**: a mesma pergunta chamava-se
   `label_col_w`, `LABEL_W`, `ADJ_LABEL_W`, `BLEND_LABEL_W`, `CARD_LABEL_W`. Alargá-la exige
   **restringir o domínio** ao mesmo tempo — alargar a grafia a todo o repo devolveu `16` acusações
   falsas para `5` verdadeiras, e *uma régua assim é exemptada até deixar de medir*.

Ver [[feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site]] (a irmã:
lá o que muda com o consumidor é a LARGURA que ele dá) e
[[feedback_a_contrast_cure_carries_the_surface_it_was_calibrated_against]].
