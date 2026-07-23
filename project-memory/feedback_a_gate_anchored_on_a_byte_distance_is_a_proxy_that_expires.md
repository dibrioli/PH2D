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
