---
name: feedback_a_source_reader_that_ignores_char_literals_reads_the_file_inside_out
description: Um leitor de fonte que não conhece o literal de CARÁCTER lê `find('"')` como uma aspa a abrir e passa a ler o resto do ficheiro ao contrário — comentário como código, código como prosa
metadata:
  type: feedback
---

Todo censo textual deste repo começa por **tirar os comentários**, e todos eles reconhecem a aspa
dupla. ⛔⛔ **Nenhum reconhecia o literal de CARÁCTER**, e `find('"')` é Rust perfeitamente legítimo:
ao ver aquela aspa dentro das plicas, o leitor abre uma string que nunca fecha ali e **passa a ler o
resto do ficheiro ao contrário** — comentário como texto, texto como código.

⚠️ **O `'` também abre um TEMPO DE VIDA** (`&'a str`), que não fecha. O discriminador é haver um `'`
a fechar dentro do alcance de um escape:

```
'x'      -> carácter (o 3.º char é a plica)
'\n'     -> carácter (plica a fechar dentro de ~6 chars, depois de `\`)
&'a str  -> TEMPO DE VIDA: não fecha ⇒ deixa passar
```

**Why:** medido em 2026-09-10 na `line/UIUX`. Um censo de literais pintados publicou **438** e o
número era **418**; e o gate irmão, escrito em Rust, acusou na primeira corrida **um comentário do
próprio ficheiro do gate** — porque uma linha acima havia um `find('"')`.

**How to apply:** ao escrever (ou copiar) um `strip_comments` para varrer fonte Rust, trate as três
formas — string, comentário, **carácter** — antes de acreditar em qualquer contagem. ⭐ E a lição de
segunda ordem: **duas implementações da mesma régua, em linguagens diferentes, não são redundância**
— foi a de Rust que acusou a de Python, e o acordo delas ao número é o que torna o resultado
acreditável. Ver [[feedback_a_textual_census_that_cannot_tell_prose_from_code_lies_both_ways]] e
[[reference_topic_gate_discipline]].
