---
name: feedback-a-default-that-fits-the-majority-is-still-a-law
description: Um default acertado pela maioria dos casos vira "fato do motor" hardcoded, e o 3º caso não tem onde morar — dê porta à lei antes de precisar dela
metadata:
  type: feedback
---

Quando quase todo consumidor concorda com uma resposta, ela é escrita como **fato
do motor** em vez de **lei com porta**. O 3º caso então não tem onde morar: o nó
diverge do próprio `eval`, em silêncio, e nenhum gate fica vermelho porque o caso
ainda é **inalcançável**.

**Caso real (GPU/M5, `line/gpu-nodes`, 2026-07-18).** O sequenciador dimensionava
todo stage por `base.count` (a porta 0) — certo para ~25 transformadores — com o
gerador como ÚNICO caso especial, hardcoded no único call site. `value.lfo`
desconectado tem uma 3ª lei (`input(0).count().max(1)` = **um** valor global) e
não havia onde declará-la: a GPU dava `count = 0`, o stage era **pulado**, e
**nada conseguia produzir um campo VALUE de comprimento 1 no dispositivo** — a
família `value.*` inteira ficou inalcançável por construção. Descoberto só ao
construir o primeiro consumidor (broadcast + `look_at`), que teve de ser
**revertido** por causa disso.

**Why:** um default não é neutro — ele é uma resposta, e a ausência de porta faz
dele a ÚNICA. Enquanto todos os consumidores concordam, o hardcode e a lei são
indistinguíveis; a diferença só aparece no consumidor que discorda, e aí o custo
não é um número errado, é uma **capacidade que não existe**. Pior: a divergência
nasce verde, porque nada a alcança ainda.

**How to apply:**
- Antes de escrever uma resposta no call site, pergunte *"isto é fato da máquina
  ou é uma LEI que algum nó vai querer diferente?"*. Se for lei: **uma função,
  `None` = a resposta da maioria** (zero boilerplate para quem concorda), e o
  contexto que ela recebe traz só os fatos de que alguém já dependeu — cada campo
  com o consumidor que o exigiu escrito ao lado.
- **A lei e seu 1º consumidor no MESMO commit.** Motor sem consumidor foi o que
  se reverteu aqui; e consumidor sem lei foi o que falhou. Ver
  [[feedback_two_doors_to_the_same_question_diverge]].
- Se a lei tem metadado companheiro (aqui: a contagem **e** a janela de
  identidade), devolva **UMA resposta** carregando os dois. Perguntar duas vezes
  deixa o stage despachar `n` e contar ao kernel outro número — e **nenhum gate
  nota, porque os dois são plausíveis separadamente**.
- O gate tem de visitar o caso **degenerado/desligado**. O gate que existia
  conectava a entrada e por isso nunca viu o bug
  ([[reference_topic_fixture_discipline]]); e escolha o fixture cuja resposta
  certa é **longe de zero**, senão um stage que nunca rodou passa com buffer
  zerado.
