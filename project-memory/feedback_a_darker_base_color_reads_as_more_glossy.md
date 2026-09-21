---
name: feedback_a_darker_base_color_reads_as_more_glossy
description: "Na composição do OpenPBR só o lobo DIFUSO escala com o base_color — uma base mais ESCURA deixa o mesmo realce relativamente mais forte, e o report chega como «ficou mais brilhante»"
metadata:
  node_type: memory
  type: feedback
---

Report do dono (2026-09-21): *«se seleciono o objeto 3d, ele muda a aparência (fica mais
brilhante)»*. Medido pela porta do produto, o que mudou foi o **ALBEDO**, e para **mais ESCURO**:
média do ecrã `195,99` com a matéria certa contra **`169,66`** com o barro de fábrica.

**Why:** na composição do OpenPBR o especular **não** escala com o `base_color` e o difuso escala.
Baixar a base baixa uma das duas parcelas e deixa a outra onde estava ⇒ a razão realce/corpo SOBE,
e é essa razão que o olho lê como *brilho*. *Duas grandezas mexem-se em sentidos opostos e o
vocabulário do artista só tem uma palavra para o resultado.*

**How to apply:** um report de *«mais brilhante»* mede-se **primeiro no albedo**, nunca num knob de
brilho — varra o `base_color` pelo caminho do produto antes de olhar para `specular_*` ou
`roughness`. ⛔ Procurar no knob que tem o nome da queixa gasta a wave e não acha nada: ali **nada
mudou**. Vale ao contrário também — uma base mais clara lê-se como *«perdeu o brilho»*.

Relacionado: [[feedback_a_per_texel_albedo_is_the_base_color_never_a_factor_at_the_end]] ·
[[reference_topic_measurement_discipline]]
