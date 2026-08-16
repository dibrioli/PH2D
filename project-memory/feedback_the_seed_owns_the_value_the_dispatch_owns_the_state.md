---
name: feedback_the_seed_owns_the_value_the_dispatch_owns_the_state
description: "Painel que re-semeia widgets do documento a cada quadro tem de REMENDAR o valor no lugar; re-registar o widget inteiro apaga o estado que o passe de ponteiro acabou de escrever"
metadata:
  node_type: memory
  type: feedback
---

Um painel deste app espelha o modelo nos widgets **a cada quadro**, de dentro do `paint`. Há duas
formas de o fazer e elas parecem equivalentes:

```rust
// (A) re-registar o widget inteiro   — o estado vai junto, cravado
store.register(id, InteractiveState::Slider { state: SliderState::Normal, value: v, .. });

// (B) registar-se-ausente, depois REMENDAR o campo
let _ = store.register_if_absent(id, InteractiveState::Slider { state: Normal, value: v, .. });
if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(id) { *value = v; }
```

**(A) apaga o estado que o passe de ponteiro acabou de escrever.** Medido pela porta do produto: UM
paint leva `Hovered -> Normal`. A linha acende sob o rato e apaga-se antes de ser desenhada.

**Why:** o `state` de um widget é **transiente e do dispatch**; o `value` é **do documento e do
seed**. Um `register` escreve os dois, então quem só queria espelhar o valor destrói o outro sem o
mencionar. Remendar **não pode** perder o estado — não há campo para esquecer de copiar —, enquanto
"leia o estado e passe-o de volta" é uma regra que o campo N+1 nasce sem.

⚠️ **O defeito costuma ser INERTE até alguém dar cor ao estado.** A barra do timeline carregava este
clobber há muito e ninguém notava, porque o polegar não lia `Hovered`: foi a wave que lhe deu hover
que o tornou load-bearing. Corolário operacional: **quem ensina um widget a reagir tem de varrer os
sítios que o re-semeiam no mesmo commit** — curar só uma metade shipa algo que acende e apaga dentro
do mesmo quadro.

⚠️ **Procure o outlier no PRÓPRIO ficheiro antes de procurar longe.** Nos dois casos de 2026-08-15
(painel de params do Motion; scrollbar do timeline) o ficheiro já continha a resposta certa a poucas
linhas — o ramo do toggle copiava o estado e dizia porquê em prosa, e `mirror_number`/`mirror_text`
já eram o molde. Um ramo discordava dos vizinhos.

**How to apply:**
- todo espelho por-quadro usa `register_if_absent` + `get_mut`, nunca `register`;
- o gate é **comportamental e pela porta real**: arme o store em `Hovered`, chame o `paint` de
  produção, e afirme que sobreviveu. A mutação é repor o `register`;
- e ele precisa do **CONTROLO**: afirme também que o valor CONTINUA a ser espelhado — senão «não
  escrever nada» satisfaz o primeiro e o painel deixa de seguir o documento
  ([[reference_topic_gate_discipline]]);
- num builder, o gémeo desta doença é `.visual(par).state(x)`: `visual(v)` **é**
  `state(v.0).hover_t(v.1)`, então o `.state(x)` a seguir sobrescreve metade do par.

Related: [[reference_topic_authored_state_and_clocks]] · [[reference_topic_ui_seam_discipline]] ·
[[feedback_derived_coordinate_seed_must_match_sample]] (o irmão no eixo das COORDENADAS: ali o seed e
a amostragem discordam sobre um número; aqui o seed e o dispatch discordam sobre quem é dono do campo).
