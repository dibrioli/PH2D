---
name: feedback-an-attribute-separated-from-its-item-by-a-doc-comment-changes-owner
description: "Uma const nova entrou entre o doc de uma fn e a assinatura dela; os #[must_use]/#[allow] reataram-se em SILENCIO ao const, e o clippy disse-o com outras palavras"
metadata:
  node_type: memory
  type: feedback
---

Inseri uma constante nova logo abaixo do doc-comment de uma funcao. Os
`#[must_use]` e `#[allow(clippy::too_many_lines)]` que estavam entre o doc e a `fn`
passaram a pertencer ao **const**. A funcao perdeu os dois **sem erro nenhum**, e o
unico sinal foi um warning que diz outra coisa: *«`must_use` nao pode ser usado em
constantes»* — que nao se le^ como *«a funcao perdeu os atributos»*.

**Why:** em Rust os atributos e os doc-comments de um item sao um bloco contiguo que
termina no item; qualquer coisa inserida no meio **reata** o que vem antes dela.
`#[allow]` perdido nao levanta erro (o lint volta a ser um warning que talvez ja'
esteja silenciado noutro sitio) e `#[must_use]` perdido nao levanta nada. *A unica
mudanca visivel e' num lint cujo texto aponta para o item ERRADO.*

**How to apply:** ao inserir um item entre um doc-comment e o que ele documenta —
que e' o sitio natural para por uma constante nova de configuracao — verifique se
havia **atributos** naquele intervalo. E quando o clippy reclamar de um atributo
«no sitio errado», nao apague o atributo: procure o item de quem ele era. Prefira
colar os atributos a' assinatura, imediatamente acima dela, em vez de os deixar no
topo do bloco de doc. Irma^ de
[[feedback-a-doc-comment-naming-a-cfg-expires-grep-the-attribute]].
