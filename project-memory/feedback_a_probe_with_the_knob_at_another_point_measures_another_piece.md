---
name: feedback-a-probe-with-the-knob-at-another-point-measures-another-piece
description: A minha varredura do escudo corria com `round = 0,02` onde o gate usa `0,04` e leu `0,88` onde o gate lia `1,05` — e duas réguas da mesma grandeza a discordarem É o achado, não ruído.
metadata:
  type: feedback
---

Medido em 2026-09-05 (doc 06 §122.6). Ao escolher a cerca do escudo varri a razão `s/w` com uma
sonda escrita à pressa; ela punha o filete em `0,02` porque era o valor que eu tinha à mão, e o
representante do catálogo usa `0,04`. A sonda deu **`0,88`** onde o gate dava **`1,05`** para a
mesma célula — e eu quase escolhi a cerca pelo número mais simpático.

**Why:** um filete, um chanfro, uma resolução, uma semente: qualquer knob que a régua fixa é uma
**escolha de peça**. Duas réguas da mesma grandeza a discordarem não é ruído de medição — *é o
achado*, e ignorá-la é escolher a que dá a resposta que se queria. É a terceira ocorrência da mesma
família nesta linha, depois da sonda que armava o módulo por env var em vez do pill
([[feedback_a_probe_that_arms_a_module_by_env_var_measures_another_program_than_the_pill]]) e da
sonda que corria com o param no default
([[feedback_a_missing_knob_cell_can_hide_a_defect_measure_before_pricing]]).

**How to apply:** a sonda **importa os knobs do caminho do produto** (aqui, o construtor do
catálogo) em vez de os escrever; se tiver de os escrever, imprime-os ao lado do resultado, para que
a discordância com o gate seja visível na página. E quando duas medidas da mesma coisa não batem,
**pare e concilie antes de decidir** — nenhuma decisão tomada sobre a que se preferiu sobrevive à
próxima corrida. Ver [[feedback_an_aggregate_that_already_measures_item_by_item_must_return_the_table]].
