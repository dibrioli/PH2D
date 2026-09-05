---
name: feedback_the_note_beside_a_count_describes_the_population_it_had_when_it_was_written
description: "Acrescentar uma fileira a uma conta existente herda a JUSTIFICAÇÃO escrita ao lado dela — e ela é uma afirmação sobre a população antiga"
metadata:
  type: feedback
---

Uma conta que soma linhas — `altura = line * n_linhas`, `custo = k * n_itens`, um teto, um
orçamento — costuma trazer **a razão por que ela é suficiente** escrita ao lado. Essa razão é uma
afirmação sobre a população que a conta tinha **no dia em que a nota foi escrita**. Acrescentar um
item novo à mesma conta **herda a nota sem a reconferir**, e nada no código pisca.

Medido em 2026-09-05 (`line/components`, cartão de instância). A altura do cartão contava uma linha
por entrada de lista, com esta nota ao lado:

> *«as linhas dos componentes overridados ficam na conta: elas são NOMES do catálogo, curtos por
> construção — e medir cada uma custaria um layout por quadro por linha»*

Verdadeira: o nome mais longo do catálogo tem **20** caracteres. Quatro dias depois entrou na
**mesma** conta uma segunda família de linhas — as excepções sem alvo —, cujo rótulo embrulha um
`Name` que **o artista escreveu**. Medido: com um nome de 33 caracteres o botão de baixo ficava em
`y = 198`, **o mesmo `y` do nome curto** — pintado por cima da 2.ª linha do texto. É o defeito que
o dono já tinha fotografado no mesmo cartão semanas antes, por outra porta.

⚠️ **E o argumento nunca foi só sobre a string:** embrulhar é função da **LARGURA**, e a largura de
um painel não é uma constante. *Uma justificação que depende de duas grandezas e só nomeia uma
falha assim que a outra se mexe.*

**Why:** a nota parece uma prova e é uma **medição datada**. O item novo não a lê — ele lê a conta,
que continua a compilar e a correr. Nenhum gate acorda, porque o gate que existia media a população
antiga.

**How to apply:** ao acrescentar um item a uma conta que **já tem uma justificação escrita**, a
primeira coisa a fazer é ler a justificação e perguntar *«ela vale para o que eu estou a pôr aqui?»*
— e, se não valer, **apagar a nota** em vez de acrescentar uma excepção a ela (aqui a cura foi medir
todas as linhas, não medir só as novas: duas leis na mesma conta é a próxima divergência). ⚠️ E
quando a conta e a execução são **duas passagens** (medir antes de pintar, orçar antes de gastar),
derive as grandezas partilhadas **uma vez** e passe-as: *uma largura calculada duas vezes diverge no
dia em que só uma delas passar a descontar um botão*. Ver
[[feedback_stale_comment_and_dead_code_lie]] e
[[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]].
