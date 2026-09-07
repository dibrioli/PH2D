---
name: a-field-can-be-wrong-exactly-where-no-surface-ruler-looks
description: Um ramo de união que é um semiespaço INFINITO ganha o `min` fundo dentro da peça, onde as coordenadas dele são singulares — e toda régua de superfície lê zero de erro
metadata:
  type: feedback
---

Ao escrever a rosca (W135, `ops_thread.rs`) o filete era `min(cilindro, max(flanco₊, flanco₋))`. O
`max` de dois semiplanos é uma **cunha infinita**: ela continua para dentro até ao eixo. Lá dentro
ela **ganha** o `min` contra o cilindro (o termo constante do flanco é negativo e empurra-a abaixo),
e ali a coordenada dela é singular — `|∇w| = b/ρ` explode.

Medido: `‖∇f‖` até **`2,4562`** a `ρ = 0,013`, em **54 de 72** células da região permitida.

⛔⛔ **E NENHUMA RÉGUA DE FORMA O VIA:** a secção meridiana lia `0,000 %` de erro antes e depois, o
volume não muda, a silhueta não muda. O ponto é **fundo dentro da peça**.

**Why:** as réguas de forma medem a FRONTEIRA; um campo implícito também é lido no interior (a marcha
atravessa-o, e o censo mede a caixa inteira). Um ramo de união que se estende para além da região que
ele descreve leva as coordenadas dele para onde elas não valem.

**How to apply:** ao unir um ramo local a um corpo, **feche-o com a geometria em que ele assenta** —
aqui `max(flanco₊, flanco₋, −dr)`, e o termo do fecho **não** leva o factor de segurança dos outros,
porque a face dele já é exacta. E ao medir uma forma nova, corra uma varredura de gradiente sobre a
**CAIXA INTEIRA**, nunca só sobre a casca: [[a-shape-ruler-and-a-field-ruler-answer-different-questions]].

Relacionado: [[feedback_a_minorant_composed_with_the_inverse_of_what_it_minorises_is_not_the_identity]]
(o lado oposto: uma folga aplicada antes de uma SUBTRACÇÃO desloca a superfície; aplicada a uma
distância que vai ser JUNTADA, ela só a torna honesta e o zero não se mexe).
