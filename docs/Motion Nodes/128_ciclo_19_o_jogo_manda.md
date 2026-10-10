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
  Trigger o pulso; numa Color a luminosidade OKLab) e `r` `g` `b` `a` (a cor; num número, o cinza
  dele) — para dirigir os quatro canais de um param de cor.
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

`MotionCookPump` ganha uma FITA: o que estava publicado em cada tique cozido para a frente (só as
mudanças, por revisão). O replay do scrub põe, em cada tique, o que a fita diz; para lá do que foi
gravado usa o publicado agora. Tocar para a frente sobre chão já gravado com uma entrada DIFERENTE
é um ramo novo: a fita e o anel esquecem o futuro velho.

### §3.4 — `TimeMode::Driven` (apendado à escada)

`t' = offset + duration · curve(clamp(drive, 0, 1))` — o tempo da sub-árvore deixa de andar e passa
a ser uma POSIÇÃO (o Blend 1D do Rive, o `TimeSeek` do Godot). `drive` é um param apendado ao
`motion.time_remap`; dirigido por fio, o cozimento resolve o condutor no relógio de FORA antes de
reescrever o de dentro (o condutor não pode ler o tempo que ele próprio decide). Uma entrada parada
⇒ um instante parado ⇒ memo.

### §3.5 — Crossfade de estados sem máquina nova

`value.input("estado") → value.follow → mixer.weight_k`. `value.follow` é o suavizador TEMPORAL que
faltava: **Linear** (chega em `duration`, o `xfade` do Godot), **Smooth** (meia-vida, `exp2` da casa)
e **Spring** (a lei do `motion.spring`). Sequencial pelo laço `pre` (o estado viaja na saída).

### §3.6 — Os escritores

- **Componente `Motion Input`** (qualquer objecto): nome, fonte (*Valor* · *Vida 0–1* · *Dano* —
  dispara quando a vida desce), número, cor. Override por instância/variante pelo sistema de
  componentes, sem código novo.
- **Luau**: `motion.set(nome, número|booleano)`, `motion.color(nome, r, g, b[, a])`,
  `motion.fire(nome)`.
- **Sinais do runtime**: um sinal com o nome de uma entrada Trigger dispara-a.
- O recurso partilhado mora na `ph2d-ecs` (`MotionInputs`) — as três crates escritoras já dependem
  dela; a `ph2d-app-motion` publica-o como externals antes do cozimento.

## §4 — Waves

| wave | o quê | estado |
|---|---|---|
| W0 | a medição do scrub (vermelho), o oráculo Godot | |
| W1 | a fita das entradas (CPU e placa) | |
| W2 | `TimeMode::Driven` | |
| W3 | `value.input` · `value.follow` | |
| W4 | os escritores (recurso, componente, Luau, sinais) | |
| W5 | cena `=134`, fotos, tutorial 12, medições | |

## §5 — Medições

### §5.1 — O scrub antes da cura (W0, `scrub_entradas_tests.rs`)

Um acumulador sequencial que soma a entrada `$input:g` (1 até ao tique 19, 5 depois), tocado até
40 e com o scrub a voltar:

| pergunta | tocando viu-se | o replay deu |
|---|---|---|
| scrub para o tique 10 | `11` | **`15`** (cada tique replayado com o 5 de agora) |
| ramo novo (volta ao 25, entrada 2, toca até 35) e scrub para o 30 | `60` | **`72`** (o anel devolveu o checkpoint do ramo VELHO) |
