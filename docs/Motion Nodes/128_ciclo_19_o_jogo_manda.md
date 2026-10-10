# 128 — CICLO 19: O JOGO MANDA (entradas publicadas, tempo dirigido, crossfade de estados)

> Terceiro ciclo da obra 17 → 23 ([doc 126 §0](126_ciclo_17_o_ritmo.md)), aberto depois do smoke dos
> ciclos 17 e 18 aprovado (10/10). Base: `line/motion-value` sobre o `main` `2a925859d` (ciclos 13–19
> por integrar). Prompt: [handoff de continuação](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-10_CICLO_19.md).

## §1 — O grupo e a premissa do tutorial

**Nós:** `value.input` (novo) · `value.follow` (novo) · `motion.time_remap` (modo `Driven`) ·
`motion.mixer` (os pesos, dirigidos) — e os ESCRITORES: o componente `Motion Input`, o Luau
(`motion.set` · `motion.color` · `motion.fire`) e os sinais do runtime.

**Tutorial 12 «O jogo manda»:** uma cena com uma barra que enche, uma explosão que dispara e dois
estados que se cruzam — tudo mandado por três números que o jogo publica; o artista mexe nos
números no cartão e vê o grafo obedecer, e volta a fita para trás sem o replay mentir.

## §2 — Auditoria (o código e o estado da arte)

**O código** (exploradores, 10/10):
- a porta existe: `Cook::set_external(nome, Stream)` (`ph2d-nodegraph/src/cook.rs:171`), lida por
  `ctx.external(nome)`; o memo guarda a REVISÃO (hash do conteúdo) de cada external lido
  (`external.rs` `revs_of`). Hoje só a membrana do editor publica (objectos por nome, formas,
  `$cursor`, `$camera`). ⛔ **Nenhuma entrada publicada** existe, nem nó que a leia por nome;
- ⛔ **não há «componente Motion» por objecto**: o documento tem UM grafo (`MotionState.doc`),
  cozido por uma bomba. Os objectos entram nele como externals pelo nome do artista. ⇒ as entradas
  são do GRAFO (uma por nome), e o que tem override por instância/variante é o ESCRITOR — um
  componente no objecto (a granularidade de override da casa é o componente inteiro,
  `ph2d-ecs/src/instantiate.rs:54-78`);
- **o scrub esquece as entradas** — MEDIDO antes de desenhar (§5.1): o `CookCheckpoint` guarda o
  estado do grafo e não os externals, e o replay do anel coze cada tique com o valor publicado
  AGORA;
- `TimeMode` (`ph2d-nodegraph/src/time.rs`) tem seis modos, todos `f(t)`; nenhum lê um valor;
- o `motion.mixer` já tem `weight_0..3` (params) e qualquer param se dirige por fio (doc 58), mas
  ⛔ **não há suavizador TEMPORAL de valores**: o `value.smooth` é uma média ESPACIAL (sobre as
  instâncias) e o `motion.spring` só age em canais de transformação (`P`/`rot`/`size`);
- o Luau tem a tabela `ph2d` (`ph2d-script/src/host.rs:577`) e nenhuma ponte para o Motion; os
  sinais do runtime (`ph2d-runtime`) têm leitor com cursor próprio.

**O estado da arte:**

| quem | o que faz | o que levamos |
|---|---|---|
| Rive — *State Machine inputs* (Number · Boolean · Trigger) e *Data Binding* (rive.app/docs) | entradas tipadas por nome no artboard; o Trigger dispara e volta sozinho depois de UM avanço da máquina; o *Blend State 1D* escolhe o instante de uma animação por um número | os quatro tipos (+ cor, do Data Binding); o gatilho de UM tique; o tempo dirigido |
| Unity — *Animator parameters* (Float · Int · Bool · Trigger), `SetFloat(nome, v, dampTime, dt)` (docs.unity3d.com) | o amortecimento mora NA ESCRITA; o Trigger fica armado até ser consumido (a armadilha conhecida: dispara atrasado) | o amortecimento como NÓ (`value.follow`), legível no grafo; ⛔ o gatilho que fica armado |
| Unreal — *Niagara User Parameters* (docs.unrealengine.com) | parâmetros expostos do sistema, com override por COMPONENTE na instância | o override por instância/variante pelo componente escritor |
| Godot 4 — `AnimationNodeTransition` (`xfade_time`), `AnimationNodeBlendSpace1D`, `AnimationNodeTimeSeek` (MIT, instalado: `godot 4.7.2`) | o crossfade linear no tempo; o tempo como posição | ⭐ o ORÁCULO CORRIDO do crossfade (§5.2) |

## §3 — O desenho

### §3.1 — `value.input`: uma entrada publicada, com nome e tipo

- Lê `ctx.external("$input:<nome>")` (`external::input_of`, o espaço reservado `$` — um objecto
  chamado `vida` não colide com a entrada `vida`). Sem publicação ⇒ o valor do PRÓPRIO cartão (o
  número de teste do artista).
- `kind`: **Number · Toggle · Trigger · Color**. Saídas: `out` (o número; num Toggle `0/1`; num
  Trigger o pulso; numa Color a luminosidade OKLab), `r` `g` `b` `a` (a cor; num número, o cinza
  dele) — para dirigir os canais de um param de cor — e `pulse`: a coluna do `out` no tipo da
  família `pulse.*` (um Trigger liga direto ao `pulse.adsr`; a saída de VALOR não entra numa porta de
  pulso — `TypeMismatch`, achado pela cena).
- Vários escritores com o mesmo nome ⇒ uma linha por escritor (ordem estável), e um param dirigido
  lê a primeira (`driven_value`).
- `Effect::Pure` e CPU — o valor vem de um external, que um kernel não alcança; a placa recebe-o
  como o número de um param dirigido (doc 110 §14).

### §3.2 — O gatilho é um pulso de UM tique, decidido por quem ESCREVE

Quem publica conta os disparos; no tique em que a conta mudou publica `v = 1`, no seguinte `0` (o
Trigger do Rive). A borda vive no escritor — e não num laço `pre` do nó, que o tornaria sequencial
e o proibiria debaixo de um `time_remap`. Como o pulso é um external, a fita (§3.3) grava-o e o
scrub repete-o no tique certo.

### §3.3 — A fita das entradas (o scrub não pode mentir)

`MotionCookPump` ganha uma FITA (`ph2d-eval-motion/src/fita.rs`): o que estava publicado em cada
tique cozido para a frente — TODOS os externals, não só as entradas (o cursor, os objectos), só as
mudanças, por revisão.

- **O replay é uma CAMADA** (`Cook::set_externals_replay` / `visible_externals`): a fita mostra-se
  POR CIMA do publicado, que fica intacto por baixo — os nós e o memo leem a camada; voltar a tocar
  tira-a. (O primeiro desenho SUBSTITUÍA o publicado, e um tique novo depois de um replay gravava os
  valores da fita como se fossem os de agora.)
- **Uma regra por tique, nas duas rotas** (`MotionCookPump::entradas_do_tique`): um tique ANTES do
  último cozido, ou — com o relógio PARADO — um tique já gravado que não é o último (arrastar a fita
  para a FRENTE), repete a fita; tocar, e editar no mesmo tique, GRAVA o publicado. Gravar sobre
  chão já gravado com outra entrada é um RAMO: a fita, o anel da CPU e o anel da placa
  (`GpuCook::forget_after`) esquecem o futuro velho. O shell diz a cada quadro se toca
  (`define_a_tocar`). Achado pelo gate da PLACA: arrastar 35 → 75 marchava com o valor de agora
  (diverge `0,53`) — e o quadro da CPU (`ticks_owed`, tique a tique) tinha o mesmo caminho.
- **O pulso de cada gatilho é da bomba** (§3.2): quem escreve publica a CONTAGEM
  (`INPUT_COUNT = "n"`), e a bomba deriva `v = 1` na linha cuja contagem mudou desde o fim do tique
  anterior — idempotente no mesmo tique (as sub-passadas da placa), e depois de um scrub as
  contagens voltam às da época.
- **Orçamento** `FITA_BYTES = 32 MiB` (§5.4: 34 B/tique com três entradas a mexer ⇒ ~1 M tiques,
  ~4,6 h a 60 Hz); acima dele a marca mais velha DOBRA-SE na base (aberto nomeado, §7).

### §3.4 — `TimeMode::Driven` (apendado à escada)

`t' = offset + duration · curve(clamp(drive, 0, 1))` — o tempo da sub-árvore deixa de andar e passa
a ser uma POSIÇÃO (o Blend 1D do Rive, o `TimeSeek` do Godot). `drive` é um param apendado ao
`motion.time_remap`; dirigido por fio, o cozimento resolve o condutor no relógio de FORA antes de
reescrever o de dentro (o condutor não pode ler o tempo que ele próprio decide). Uma entrada parada
⇒ um instante parado ⇒ memo.

### §3.5 — Crossfade de estados sem máquina nova

`value.input("estado") → value.follow → mixer.blend` (ou `weight_k`). `value.follow` é o suavizador
TEMPORAL que faltava: **Linear** (chega em `duration`, o `xfade` do Godot), **Smooth** (meia-vida,
`exp2` da casa) e **Spring** (`ph2d_node_motion_spring::law::passo` — extraído do `motion.spring`
sem mudar a aritmética, os 24 testes dele verdes). Sequencial pelo laço `pre`. ⭐ **Uma mudança
conta a partir do tique em que CHEGA** — o passo anda na direcção do alvo do tique ANTERIOR (no
estado): foi o oráculo que o ensinou (§5.2).

### §3.6 — Os escritores

- **Componente `Motion Input`** (qualquer objecto, família Logic): `Input` (o nome — com um botão
  por cada nome que o grafo lê, `nomes_que_o_grafo_le`), `Source` (*Value* · *Color* · *Health*
  0–1 · *Damage* — um disparo cada vez que a vida desce), `Value`, `Color` — só a linha que a fonte
  lê aparece, e o cartão QUEIXA-SE quando o controlo não faria nada (nome vazio, grafo sem Input,
  nome que ninguém lê, fonte de vida sem `Health`). Override por instância/variante pelo sistema de
  componentes, sem código novo. ⚠️ UM por objecto (é um componente): a vida e o dano do mesmo herói
  são dois objectos — ou um `pulse.*` sobre a vida, no grafo.
- **Luau**: `motion.set(nome, número|booleano)`, `motion.color(nome, r, g, b[, a])`,
  `motion.fire(nome)`.
- **Sinais do runtime**: um sinal com o nome de uma entrada Trigger dispara-a.
- **A linha de comando** (`PH2D_MOTION_INPUTS=vida=0.25:estado=1:time=0.2,0.5,1`): o jogo lançado
  com valores, e a porta das FOTOS de outras entradas (a ferramenta não clica). Separador `:` (o do
  `PATH`): a ferramenta escreve a variável num script, e um `;` partia a linha.
- O recurso partilhado mora na `ph2d-ecs` (`MotionInputs`, uma linha por escritor em ordem estável:
  Script · Sinal · Linha · Objecto pelo NOME) — as três crates escritoras já dependem dela; a
  `ph2d-app-motion` copia-o (`le_do_jogo`, no `publish_objects`) e publica-o (`publica`, no
  `publish_all`) antes do cozimento, no MESMO quadro (a física, os scripts e as barras correm antes
  do Motion — `motores_do_quadro`, onde a ponte do componente entra numa linha). O publicador lê
  `kind`/`fire` pela PORTA da escada (`resolved_params`): um `fire` dirigido por fio só existe
  durante o cozimento.

## §4 — Waves (feitas)

| wave | o quê | commit |
|---|---|---|
| W0 | a medição do scrub (vermelho, §5.1) | `16542586e` |
| W1 · W2 | a fita (CPU e as duas rotas da placa) · `TimeMode::Driven` | `16542586e` |
| W3 | `value.input` · `value.follow` · o oráculo Godot | `391c610bd` |
| W4 | os escritores: recurso, componente, ponte, Luau, sinais, `PROJECT_SCHEMA` 187 → 188 | `79e462fc5` · `56cfd539e` |
| W4 | o cartão do `Motion Input` no Inspector (um agente) | `bb6ee2604` |
| W5 | a cena `=134` · a saída `pulse` · a fita por CAMADA e a regra do relógio parado · o gate da placa · a sonda · a linha de comando · o tutorial 12 e as fotos | `90a1a97d9` · `670215103` · `3c694ecdb` |

## §5 — Medições

### §5.1 — O scrub antes da cura (W0, `scrub_entradas_tests.rs`)

Um acumulador sequencial que soma a entrada `$input:g` (1 até ao tique 19, 5 depois), tocado até
40 e com o scrub a voltar:

| pergunta | tocando viu-se | o replay deu |
|---|---|---|
| scrub para o tique 10 | `11` | **`15`** (cada tique replayado com o 5 de agora) |
| ramo novo (volta ao 25, entrada 2, toca até 35) e scrub para o 30 | `60` | **`72`** (o anel devolveu o checkpoint do ramo VELHO) |

Depois da cura (`scrub_entradas_tests.rs`, seis gates): o replay dá o que se viu em seis alvos; o
ramo novo dá `60`; parado, arrastar para a frente tique a tique repete; editar no mesmo tique pesa
o publicado e a fita guarda o ramo; um disparo é um pulso de UM tique (dois tiques num quadro: só
no primeiro).

### §5.2 — O oráculo CORRIDO: o crossfade do Godot

`docs/Motion Nodes/ferramentas/godot_xfade_oracle/corre.sh` corre o Godot 4.7.2 instalado (MIT,
conferido no `pacman`) — um `AnimationNodeTransition` com `xfade_time` 0,25 · 0,5 · 1 s sobre duas
animações nossas, `tree.advance(1/60)` à mão — e grava
`crates/ph2d-node-value-follow/tests/fixtures/godot_xfade.txt`. O gate
(`the_linear_follow_is_the_godot_crossfade_frame_by_frame`) corre a cadeia do PRODUTO
(`value.input → value.follow` Linear) tique a tique: **pior `≤ 2e-5`** (o `CMP_EPSILON` do Godot no
quadro do pedido). O que a SAÍDA ensinou:
- **no quadro do pedido o peso ainda é o de antes** e sobe `dt/xfade` a partir do seguinte — o
  primeiro desenho andava já no quadro do pedido (um quadro à frente);
- três armadilhas do oráculo, medidas: no `_init` a cena não está montada (tudo `0`; é no
  `_initialize`); um valor numa SUB-propriedade (`position:x`) não se mistura (salta); e sem estado
  corrente (`current_index = -1`) o primeiro pedido não cruza nada.

### §5.3 — A cena `=134`, na SAÍDA pela bomba do produto (`motion_state_jogo_demo_tests.rs`)

| fenómeno | medido |
|---|---|
| a barra (cartão 0,6; vida 0 · 0,25 · 0,5 · 1) | **13** · 0 · 5 · 11 · 24 peças cheias de 24; aos 5 s, a mesma |
| o peso do cruzamento (pedido no tique 30) | `0` no 30 · `0,500` no 48 · `1` no 66; o anel `1,6005` de raio médio |
| a cor do time | os discos `== (0,2 · 0,4 · 0,9)` |
| o boom | o redemoinho `0,031 → 0,103` voltas/s (×3,3) e as estrelas `0,130 → 0,323` (×2,5); aos 2 s, `0,0315` e `0,130` |
| o scrub (vida, estado e um boom a mudarem) | a barra, as posições da galáxia e o peso iguais ao que se viu, em 5 alvos |

⛔ **Recusado por medição — o empurrão radial** (uma repulsão dentro da galáxia): com `K = 10 · 20 ·
40 · 80 · 160` o raio médio vai de `1,95` a `2,26 · 2,59 · 3,32 · 5,04 · 8,92` e **fica lá** (aos
5 s, o mesmo): o redemoinho em velocidade-alvo mata a velocidade radial e o íman (`0,158`) não traz
de volta — três disparos tiram a galáxia do ecrã. O redemoinho é que é o campo a sacudir: em
velocidade-alvo ele DEVOLVE a calma. (O preço, nomeado: `+0,08` m de raio por disparo, a força
centrífuga.)

**As fotos** (`target/prova/c19/fotos/`, cortadas em `tutoriais/fig/jogo_*.png`) acharam o que
nenhum gate via: a barra colada ao topo e meio cortada, as peças quadradas de `0,26` com vão `0,3`
eram uma FAIXA, os discos de `0,3` com vão `0,5` sobrepunham-se, e a legenda da barra fora do ecrã.
A foto de CONTROLO (as entradas pela linha de comando) mostra a barra a 25%, o anel e o azul — as
entradas chegam ao desenho.

### §5.4 — O custo (`measure_the_game_group`, `--release`, load `47,9` — ordem de grandeza)

```text
  caso                      │   mínimo / mediana
  follow Linear             │    0.083 /    0.086 ms     (32 761 linhas)
  follow Smooth             │    0.078 /    0.078 ms
  follow Spring             │    0.136 /    0.136 ms
  driven, entrada parada    │    0.002 /    0.002 ms     (memo: um instante parado)
  driven, entrada a mexer   │    0.008 /    0.526 ms     (32 761 linhas recozidas)
  cena =134, um quadro      │    0.065 /    0.120 ms
  a fita (3 entradas a mexer)│ 14383 B em 421 tiques = 34.2 B/tique
  scrub (5 alvos)           │    0.117 /    0.167 ms
```

**Onde corre (§2.1):** `value.input` e `value.follow` na CPU — o valor vem de um external, que um
kernel não alcança; dirigindo um param de um nó da placa, o número vai ao uniform por tique
(`valores_dirigidos`, doc 110 §14) — o gate da placa (`motion_bridge_gpu_entradas_tests`) corre uma
simulação no DISPOSITIVO com a força dirigida por uma entrada. Um escopo de tempo (o `Driven`) leva o
grafo à CPU (a escada do doc 98 §2) — a `=134` corre na CPU, `0,12 ms` por quadro.

### §5.5 — O tutorial

`tutoriais/12_o_jogo_manda.pdf` (5 páginas); gate `every_step_of_the_game_tutorial_is_reachable`.

## §6 — Lições (não as repague)

- **O replay de um canal de fora é uma CAMADA, nunca uma substituição**: substituir o publicado
  faz o tique seguinte gravar a fita como se fosse o presente.
- **Scrub ≠ tocar sobre chão gravado**: a mesma marcha tique a tique é replay com o relógio parado e
  gravação a tocar — só o shell sabe qual, e tem de o dizer à bomba.
- **Um pulso decidido por quem escreve conta, não pulsa**: a contagem é independente dos tiques que
  um quadro coze; o pulso de um tique deriva-se onde se sabe o tique.
- **Uma variável de ambiente que uma ferramenta escreve num script não leva `;`.**
- **Um campo em velocidade-alvo devolve a calma; uma força radial sobre ele não volta** — para um
  gatilho reversível numa simulação, mexa no campo que a simulação já devolve.

## §7 — Abertos nomeados

- **A dobra da fita acima do orçamento**: um scrub para antes da base repete o estado da base. Com
  três entradas são horas; com MUITOS objectos a mexer (todos os externals entram na fita) são
  minutos — por medir numa cena de objectos.
- **Um gesto para dirigir uma linha de COR inteira**: hoje são três fios (`r` `g` `b`) do `Input` ao
  `Tint`.
- **A vida e o dano do mesmo objecto** pedem dois objectos (um `Motion Input` por objecto).
