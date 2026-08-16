---
name: feedback-a-gate-anchored-on-a-byte-distance-is-a-proxy-that-expires
description: "Arch-gate que afirma DISTÂNCIA ou JANELA em bytes no fonte fica vermelho sobre código correto assim que alguém insere código legítimo no meio — afirme a propriedade posicional, não a métrica"
metadata:
  node_type: memory
  type: feedback
---

Um arch-gate lê o **fonte** quando o único consumidor é o desenho do shell (a `render_loop` precisa de
janela e GPU, nenhum unit test a alcança). O padrão é certo. O que apodrece é **como** ele ancora.

Na integração de 2026-07-23 a `line/Vector` chegou com dois gates seus **vermelhos no próprio tip**, e
os dois falhavam pela mesma razão — a âncora era **métrica**, não posicional:

* `the_dispatch_is_handed_the_live_geometry` procurava o literal `self.offset_live.live()` numa
  **janela de 400 bytes** depois de `dispatch(`. Depois, a MESMA linha deu **duas** fontes à
  `LiveGeometry` (offset + pattern), o mapa passou a nascer num binding acima da chamada, e o literal
  saiu da janela. Produto correto, gate vermelho.
* `the_render_loop_wires_the_handle_gesture` afirmava **distância < 1200 bytes** entre o press da alça
  do texto e o do conector (*"mesmo cluster de Select"*). Um terceiro bloco do mesmo cluster — o
  Picker, que **tem** de preceder as duas alças — entrou no meio: 1810. Produto correto, gate vermelho.

A cura não é aumentar o número. **A distância nunca foi a propriedade** — era proxy de *"os dois presses
correm antes do picking genérico"*, que é posicional e se afirma direto, para cada press. Idem a janela:
a pergunta real é *qual argumento o `dispatch` recebe* e *de onde aquele nome nasce*.

⚠️ **E a re-escrita quase shipou uma asserção VÁCUA:** afirmei que *"o span entre os dois presses é
gateado em `DrawMode::Select`"* — e a mutação mostrou que ela **não podia falhar**, porque o span
termina DENTRO da cadeia de guards do próprio bloco do textpath, então aquele `DrawMode::Select` está
sempre lá. Verde por construção. Removida, não shipada.

**Why:** uma métrica de fonte (distância, janela, contagem de linhas) é sempre *proxy* de uma relação
estrutural, e todo código legítimo inserido no meio a invalida — sem que a relação tenha mudado. O modo
de falha é o pior possível: **vermelho sobre produto correto**, o que convida a afrouxar o número até
passar, enterrando o que o gate protegia. Um gate assim envelhece contra o próprio dono.

**How to apply:** ao escrever arch-gate sobre fonte, pergunte *"que relação eu quero?"* e afirme-a —
**A vem antes de B**, **este argumento é aquele nome**, **este nome nasce daquela fonte`** — nunca
*"A está a menos de N bytes de B"* nem *"a agulha aparece nos próximos N bytes"*. Se só souber
exprimi-la por métrica, o gate está a medir a coisa errada. E toda asserção nova **passa por mutação
antes de shipar**: reinstale o bug que ela guarda e exija RED ([[feedback_a_mutation_that_survives_may_mean_a_missing_gate]]) —
foi só isso que apanhou a minha vácua. Vermelho no seu código correto? meça nos dois lados antes de
tocar em nada ([[feedback_a_gate_red_on_your_correct_code_may_predate_you]]).

---

## ⚠️ A metade OPOSTA, e ela é pior: a janela larga demais fica **VERDE POR VÁCUO** (2026-08-15)

As duas falhas acima são *vermelho sobre produto correto* — barulhentas, e por isso curáveis. A mesma
âncora métrica tem um segundo modo, silencioso, que só uma wave alheia expõe.

O `the_paint_hands_what_it_knows_to_the_skin` (painel autorado) delimitava a janela dele assim:

```rust
let call = src.find("paint_widget_skin_with(\n")…;
let end  = src[call..].find("\n        );")…;   // um fecho a OITO espaços
```

— e a chamada que ele inspecciona fecha a **DOZE**. O `find` não parava nela: seguia dezenas de linhas
abaixo até um `);` que pertencia a um **`matches!` de outra função**. A janela era enorme, continha
tudo o que o gate procura, e ele passava. O doc-comment dele até se gabava de ir *"até o `);` que
fecha"* — e não ia.

Ele só caiu quando a wave dos scrollbars apagou aquele `matches!` **por um motivo inteiramente
diferente**, e o `.expect()` do terminador rebentou numa crate que a wave nem devia estar a mexer.

**Why:** uma âncora métrica errada para MENOS dá vermelho e é vista. Errada para MAIS dá **verde**, e
uma asserção `window.contains(X)` sobre uma janela que engloba meio ficheiro é satisfeita por qualquer
`X` em qualquer lado — o gate deixa de falar sobre a chamada que nomeia. É [[reference_topic_gate_discipline]]
na sua forma mais barata de cometer e mais cara de encontrar: só um terceiro que remova o terminador
acidental o denuncia, e o sintoma aparece **na crate errada**.

**How to apply:**
- delimitar por estrutura **derivada do próprio sítio**, nunca por um literal: meça a indentação da
  linha em que a chamada abre e feche **nela** (`format!("\n{indent});")`);
- quando um gate seu falhar numa crate que a sua wave não tocou, **suspeite do gate antes do merge** —
  a pergunta é *"o que ele estava a ler, e porque parou de ler?"*;
- e a mutação que o prova é repor o literal: ela tem de sangrar com a mensagem do terminador, não com
  a asserção de conteúdo.
