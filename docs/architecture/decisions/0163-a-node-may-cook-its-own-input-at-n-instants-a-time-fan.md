# ADR-0163 — Um nó pode cozinhar a PRÓPRIA entrada em N instantes (o *leque de tempo*)

- **Status:** Accepted
- **Data:** 2026-08-23
- **Linha:** `line/motion-value` (conferência dos nós, doc 89 folha 07 — o P1 / `SUPERAR:` S1)
- **Toca:** `ph2d-nodegraph` (foundational) · `ph2d-eval-motion` · `ph2d-node-motion-trail` · `shells/desktop`
- **Não move:** `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` (CLAUDE.md §6, gate `architecture_contract_surface`)

## Contexto

O `motion.trail` desenha um eco guardando um **ring**: a saída do tique anterior
volta por uma aresta `pre`, as linhas envelhecem, as velhas caem. É uma máquina
correcta e barata, e ela tem um limite que não é de afinação — **um ring contém o
passado porque passado é o que um ring é**.

Isso deixa quatro coisas fora de alcance, e a folha 07 da conferência nomeia-as:

1. **eco para a FRENTE** — o *Echo* da After Effects com *Echo Time* positivo, e o
   *CC Wide Time · Forward Steps*, existem porque a AE **re-renderiza**. Um ring
   não pode, por construção.
2. **`length` sem tecto de memória** — o que sobra é orçamento de instância.
3. **espaçamento NÃO-UNIFORME** dirigido por curva (ecos que se adensam perto da
   cabeça).
4. **scrub exacto sem depender do `CheckpointRing`** — um eco puro é função do
   playhead.

O mecanismo que as destrava já existia meio construído: o `motion.time_remap`
reescreve o relógio da sub-árvore **de cima** (`Cook::cook_scoped`), e a tabela do
próprio `motion.delay` chama esse caminho de *"exact, stateless, scrub-perfect,
free"*. O que faltava era a capacidade de um nó pedir a **própria entrada em N
instantes**, e não num só — e a folha registou isso como *"uma capacidade que o
substrato não dá a um nó. Wave própria, com ADR."*

⚠️ **A composição à mão já era possível e é o argumento de que o item é P1 e não
P2:** `4 × time_remap + 4 × tint + 4 × scale + combine` = **13 nós para um eco de
4**, e o `motion.combine` tem 4 entradas, então `length 8` pede combines
aninhados.

## Decisão

**O substrato ganha um LEQUE DE TEMPO: `TimeFans = BTreeMap<NodeId, Vec<TimeMap>>`.**
A **porta 0** de um nó listado é cozida uma vez por mapa, e as N saídas chegam ao
`eval` por `EvalCtx::fan(k)` / `fan_len()`.

O `motion.trail` ganha um param `Source`:

| `Source` | de onde a cauda vem |
|---|---|
| **`Remembered`** (default) | o ring que sempre existiu — **byte-idêntico** |
| **`Resampled`** | a entrada re-cozida em `t ± k·spacing` |

e um param `Forward Steps`: quantos dos ecos vêm da FRENTE (`0` = todos atrás, o
rastro de sempre).

### As quatro escolhas que a decisão contém

1. **É um ponto de extensão APPEND-ONLY, e nenhuma assinatura existente se mexeu.**
   `cook_scoped_fanned` e `advance_tick_fanned` nascem ao lado dos irmãos, que
   delegam com um leque vazio. Razão medida: pendurar um argumento em
   `advance_or_scrub_scoped` mexeria em **29 sítios de chamada** e faria desta
   linha um ímã de conflito para todas as outras (CLAUDE.md §0.2: *ao criar
   foundational novo, projete-o para isolamento*).

2. **Os leques são ESTADO da marcha (`MotionCookPump::set_time_fans`), e os
   escopos continuam ARGUMENTO.** A assimetria é deliberada: um leque precisa da
   **duração de um tique**, que não sai do grafo — ela é do shell, que é quem faz
   o playhead andar —, enquanto um escopo é função pura do documento.

3. **Só a porta 0.** Um leque sobre uma porta de estado não teria significado (um
   `pre` é o tique anterior, não um instante pedido), e um leque sobre TODAS as
   portas multiplicaria o custo por uma coisa que nenhum nó pediu.

4. **`Resampled` é um MODO, nunca uma substituição.** O limite honesto já estava
   escrito no repo (`motion.delay`): *"`time_remap` cannot delay a **simulation**:
   a sim is not a function of `t`"*. Um leque sobre uma sub-árvore com `pre` é
   **RECUSADO** pelo cook (`CookError::SequentialInTimeScope`) em vez de desenhar
   uma trajectória plausível e falsa — a mesma política que o escopo já tinha.

### A LEI das gerações vive numa função só

`ph2d_node_motion_trail::echo_offsets(length, spacing, forward)` devolve o
deslocamento de cada geração em tiques, com sinal, **em ordem de desenho**. Ela
tem **dois leitores** — o `time_fans`, que a converte nos mapas que o cook aplica,
e o `eval`, que dela tira a IDADE de cada geração para desbotar. Escrever a mesma
escada nos dois sítios seria a receita conhecida: *o desenho pousaria num instante
e a cor viria de outro*.

`forward = 0` devolve exactamente `[-(L−1)s, …, −s, 0]`, que é a cauda que o ring
produz. **É essa redução que faz o modo novo nascer no ponto neutro.**

## Consequências

### O que fica melhor, medido

- ⭐ **O eco para a frente existe** (gate `the_forward_echo_leads_the_element_and_the_others_trail_it`,
  cena `=88`).
- ⭐ **O rastro re-cozido é EXACTO sob scrub**: chegar ao quadro 90 saltando é
  byte-a-byte o mesmo que chegar lá andando (`the_resampled_tail_is_the_same_whether_you_walk_or_jump_to_the_frame`).
  O ring depende do `CheckpointRing` para isso.
- ⭐ **A cauda re-cozida é a mais CERTA das duas.** O ring promove a cabeça a
  fantasma **periodicamente** (a cada `spacing` tiques), então as idades dos ecos
  dele passeiam por `1..=spacing` conforme a fase do quadro: ele carrega até
  `spacing − 1` tiques de erro de fase, o tempo todo. A re-cozida lê `t − k·s`
  exacto. Medido na cena: **1,9×** o passo de um tique, dentro de um ciclo de 4.

### O preço, nomeado

- **A entrada é cozida `length` vezes.** Para uma sub-árvore cara isso é `length`
  vezes o custo; o ring paga uma. É a troca que o modo É, e é por isso que ele não
  é o default.
- **A porta 0 continua a ser cozida no AGORA**, além do leque: o leque
  ACRESCENTA fatias, não substitui a entrada. Num rastro re-cozido isso é grátis
  (a geração 0 é a identidade e partilha a faixa não-escopada); num leque que só
  olha para a frente é uma cozedura a mais, na faixa que o resto do grafo já pediu.
- ⚠️ **Uma sub-árvore com `pre` é recusada**, e a recusa é um `CookError` que
  derruba a cozedura daquele sink. É a política que o escopo já tinha; o
  diagnóstico do editor é quem a explica.

### O que a medição CORRIGIU nesta decisão

⚠️ **O doc-comment do `TimeFans` afirmava mais do que a máquina faz.** Ele dizia
que fatias no mesmo instante *«partilham a faixa e o custo»* graças ao
`push_scope`. Uma mutação — trocar `push_scope(in_key, node, map)` por `in_key` —
deixou **seis gates verdes**: dentro do laço cada leitura segue a própria
cozedura, então os valores saem certos de qualquer maneira, e duas fatias
ADJACENTES no mesmo instante batem no memo mesmo partilhando a faixa.

O que a faixa própria compra é o instante repetido **fora de ordem** — o caso do
espaçamento não-uniforme, que é justamente para onde esta máquina foi construída.
A afirmação **encolheu até ao que a máquina faz**, e ganhou o gate que a mata
(`repeating_an_instant_out_of_order_still_hits_the_memo`). *Uma afirmação que
nenhuma mutação mata é uma afirmação sobre nada.*

## Alternativas medidas e recusadas

| alternativa | por que não |
|---|---|
| **Compor à mão** (`N × time_remap + combine`) | 13 nós para um eco de 4, e `motion.combine` tem 4 entradas ⇒ `length 8` pede combines aninhados. É a razão de o item ser P1. |
| **Um argumento novo em `advance_or_scrub_scoped`** | 29 sítios de chamada, e um conflito de merge garantido para toda linha viva. |
| **Trocar o valor de `TimeScopes` por um enum `Um \| Leque`** | mexe no tipo que a camada de domínio e os gates constroem — mesma churn, sem ganho. |
| **Concatenar as fatias no substrato**, entregando um stream só | o substrato teria de escrever uma coluna de geração, ou seja **conhecer um tipo de nó** — a linha que o `TimeScopes` traça de propósito (*"o substrato fica type-agnostic"*). |
| **Substituir o ring** | uma simulação não é função de `t`. `Resampled` é um MODO, e o default reduz. |

## Referências

- Folha 07 da conferência, `SUPERAR:` **S1** — [`docs/Motion Nodes/89_conferencia/07_tempo_estilisticos.md`](../../Motion%20Nodes/89_conferencia/07_tempo_estilisticos.md)
- [ADR-0031](0031-nodegraph-contract.md) — a caixa preta FBP que este leque **não** abre (o nó continua sem ver o grafo: ele vê o próprio leque, como já via os próprios params).
- Plan §1.5 / M2.N1 — os escopos de tempo, de que este leque é o irmão.
- Cena de smoke **`PH2D_GPU_COOK_DEMO=88`**.
