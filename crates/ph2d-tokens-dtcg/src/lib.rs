#![forbid(unsafe_code)]
//! **A TABELA DE TOKENS SAI E ENTRA EM DTCG** — o formato W3C que o Tokens Studio, o Style
//! Dictionary e o Penpot falam (plano UI/UX W9, a última da fila W4c).
//!
//! # O mapeamento é a IDENTIDADE, e isso não foi escolhido aqui
//!
//! Um caminho DTCG é `grupo.token` — pontos separam os níveis de aninhamento —, e um alias é a
//! mesma coisa entre chavetas: `{spacing.md}`. **As nossas chaves já são exactamente isso**, e de
//! propósito: o `num.rs` escreveu, ao pôr o ponto na chave numérica, que *"o ponto é também a
//! forma que o DTCG fala (W4c.5)"*.
//!
//! Então:
//!
//! | O que temos | O que o arquivo tem |
//! |---|---|
//! | `"accent"` (cor) | um token `accent` na raiz |
//! | `"spacing.md"` (px) | o token `md` dentro do grupo `spacing` |
//! | `NumValue::Alias(spacing.md)` | `"$value": "{spacing.md}"` |
//! | `NumValue::Expr("{spacing.md} * 2")` | o texto, **verbatim**, no `$extensions` |
//!
//! ⚠️ **Nenhum prefixo é inventado.** Pôr as cores debaixo de um grupo `color` daria caminhos mais
//! bonitos (`color.accent`) e **quebraria a coincidência**: a fórmula que o artista escreve no
//! painel (`{spacing.md} * 2`) deixaria de ser o mesmo texto que o arquivo carrega, e o
//! round-trip passaria a precisar de uma tabela de tradução — exactamente o *mapeamento inventado*
//! contra o qual a fila desta linha avisou.
//!
//! # UM ARQUIVO É UM MODO, e a razão é que o DTCG não tem modos
//!
//! O nosso override é do par `(modo, token)`; o formato W3C não tem esse eixo — modos são um
//! conceito de *resolver*, uma spec separada que quase nenhuma ferramenta lê. As duas alternativas
//! eram enfiar os quatro modos em `$extensions` (que nenhuma outra ferramenta saberia ler, o que
//! anula o motivo de exportar) ou **um arquivo por modo**, que é o que o ecossistema faz.
//!
//! ⚠️ E o import escreve no **modo VIGENTE**, nunca no que o arquivo diz: o artista está a olhar
//! para um modo, o painel nomeia-o na primeira linha, e re-vestir um modo que ele não vê é a mesma
//! falha que o *Reset This Mode* evita ao só resetar o vigente. O modo do arquivo viaja no
//! `$description` — para uma pessoa ler, não para uma máquina obedecer.
//!
//! # O export traz a TABELA INTEIRA; o import só autora o que DIFERE
//!
//! Exportar só o que o artista tocou daria, num projeto de fábrica, um arquivo **vazio** — inútil
//! como interop, e sem os tokens que os `{...}` referenciam. Então o export traz os ~80 tokens.
//!
//! ⚠️ Isso obriga a outra metade, e ela é a lei desta crate: **um valor que já é o de fábrica NÃO
//! é autorado**. Sem ela, reimportar um export de um projeto intocado autoraria a tabela toda — e
//! a partir daí re-editar o `docs/design/tokens.json` deixaria de alcançar o app, em silêncio. É a
//! mesma frase que a porta de escrita já diz: *"escrever a cor de fábrica como override não é o
//! mesmo que soltar"*.
//!
//! ⚠️ **Só vale para LITERAIS.** Um alias e uma fórmula são estruturais — o artista autorou o
//! *vínculo*, e o número que ele por acaso dá hoje não o desfaz.
//!
//! ⚠️ **O preço honesto:** um literal autorado que por acaso *é* a cor de fábrica volta como
//! não-autorado. A aparência do app é idêntica (o `resolve` dá o mesmo valor pelos dois caminhos);
//! o que muda é a contagem do readout. A alternativa — autorar os ~80 — é muito pior.
//!
//! # O que sai é a spec de 2025.10, com a ponte para quem ainda lê a antiga
//!
//! Medido na spec, não assumido: em `2025.10` o `$value` de uma cor é um **objeto**
//! (`{colorSpace, components, alpha, hex}`) e o de uma dimensão também (`{value, unit}`) — a string
//! `"#rrggbb"` e a string `"12px"` são a forma dos rascunhos anteriores, que metade do ecossistema
//! ainda emite.
//!
//! - **Escrevemos** a forma da spec. O campo `hex` é opcional *nela* e nós escrevemo-lo sempre: é
//!   a ponte declarada pela própria spec para as ferramentas de gamut limitado.
//! - **Lemos as duas.** Um arquivo que existe hoje é mais provavelmente da forma antiga, e recusá-lo
//!   por isso seria fazer o interop falhar exactamente nos arquivos que há para importar.
//!
//! ⚠️ `rem` é **recusado e contado**, não convertido: converter exige um tamanho de fonte-raiz que
//! este app não tem, e inventar `16` seria escrever um número que ninguém autorou.
//!
//! # A math não existe no formato, então ela viaja onde as coisas que não existem viajam
//!
//! O DTCG não tem expressões. Uma fórmula sai com o `$value` **resolvido** (todo leitor vê o número
//! certo) e o texto no `$extensions`, sob a chave desta aplicação. Um round-trip por aqui recupera
//! a fórmula; um round-trip por outra ferramenta recupera o número — que é a degradação honesta.

mod export;
mod import;

pub use export::{VENDOR_KEY, export};
pub use import::{DtcgError, Imported, import};
