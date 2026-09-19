---
name: feedback_a_declared_blind_spot_is_still_a_blind_spot
description: "Uma cegueira que um gate DECLARA no próprio cabeçalho continua a ser uma cegueira — e levantá-la costuma ser uma fixtura derivada, não um instrumento novo"
metadata:
  node_type: memory
  type: feedback
---

⛔⛔⛔ **Um gate que descreve por escrito o que ele NÃO vê lê-se como auditado e não é.** A frase no
cabeçalho conforta quem passa por lá — *«isto está sob controlo, já foi pensado»* — e a população
por trás dela nunca é medida.

**Medido 2026-09-19** (`line/UIUX`): a varredura de elisões pintava todo painel do registo **no
estado de FÁBRICA**, e o cabeçalho dela dizia:

> *«um painel de fábrica não tem objecto seleccionado»*

O Inspector tem **28 secções condicionais** — o painel com mais fileiras do app, e o único cujo
conteúdo depende do documento. Levantada a cegueira, a **primeira** corrida devolveu **8 rótulos a
pintar NADA** (as unidades `px` e `1/s` de campos de número) e **24 cortados**.

**Why:**
- ⭐ **O preço de levantar era uma FIXTURA, não um instrumento** — a varredura já existia, os tipos
  já existiam, e o que faltava era um objecto impossível que tem todas as secções ao mesmo tempo.
  *Uma cegueira declarada tende a custar menos do que a nota que a declara sugere*, e é por isso
  que ela sobrevive: ninguém a re-precifica.
- ⛔ **A nota envelhece do lado seguro e por isso não incomoda ninguém.** Ela não é falsa: é
  verdadeira e inerte.
- ⭐⭐ **A fixtura tem de ser DERIVADA ou nasce velha:** o censo lê as `pub fn set_current_*` do
  fonte da crate e exige que cada uma seja armada, com a metade que exige que cada uma seja
  **desarmada** (elas são `thread_local` e o binário de teste corre módulos na mesma thread).
- ⚠️ **E as linhas de lista precisam de texto A SÉRIO**: uma fixtura construída por `Default` tem a
  `String` vazia, e mede-se exactamente igual a uma secção ausente.

**How to apply:**
- ao ler *«este gate não vê X»* num cabeçalho, **meça o preço de ele passar a ver** antes de
  aceitar a nota — a resposta costuma ser uma fixtura;
- a fixtura de uma população condicional deriva-se do FONTE que declara as portas, e o gate que a
  guarda exige as duas metades (armar · desarmar);
- ⚠️ e separe o que a passagem nova pode alimentar: aqui ela **não** entra no censo *«esta palavra
  vem da tabela?»*, porque o que ela põe no painel é texto do **documento**, que a tabela não sabe
  produzir e nem devia.

Irmãs: [[feedback_an_order_with_two_halves_can_be_obeyed_only_in_the_half_that_removes]] ·
[[reference_topic_measurement_discipline]] · [[reference_topic_gate_discipline]]
