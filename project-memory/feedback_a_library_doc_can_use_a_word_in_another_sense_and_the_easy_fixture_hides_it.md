---
name: a-library-doc-can-use-a-word-in-another-sense-and-the-easy-fixture-hides-it
description: «absolute coordinates» no usvg quer dizer comandos absolutos, não espaço absoluto — e a fixtura fácil concorda com as DUAS leituras
metadata:
  type: feedback
---

O `usvg::Path::data()` documenta-se como *"All segments are in absolute coordinates"*. Lido como
**espaço** absoluto, isso dispensaria aplicar o `abs_transform` do nó. Lido como **comandos**
absolutos (o `M` contra o `m` do atributo `d` do SVG), não dispensa nada.

A segunda leitura é a certa, e a prova está no construtor da própria biblioteca: o `Path::new`
guarda `data` intacto e calcula `abs_bounding_box = bounding_box.transform(abs_transform)` — se os
dados já estivessem em espaço absoluto, essa linha transformaria duas vezes.

⚠️ **E um ficheiro SEM transform nenhum lê IGUAL nas duas leituras.** A fixtura óbvia (um quadrado
num SVG simples) aprova as duas hipóteses; só um `<g transform>` **aninhado** as separa.

**Why:** uma palavra técnica pode ter dois sentidos no mesmo domínio, e o doc raramente diz qual. A
leitura errada produz código que passa em todos os casos fáceis e falha exactamente nos ficheiros
reais — que são os que têm transformações.

**How to apply:** quando um doc de biblioteca usar uma palavra ambígua numa afirmação de que o seu
código depende, (a) procure no **código dela** uma linha que só faça sentido numa das leituras, e
(b) escreva a fixtura que **separa** as hipóteses, não a que as satisfaz às duas. *A fixtura óbvia é
a que concorda com o erro.*

Relacionado: [[feedback_stale_comment_and_dead_code_lie]] ·
[[reference_topic_fixture_discipline]] · [[reference_topic_oracle_discipline]]
