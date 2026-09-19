# `WeaponFire` — a ARMA do jogador: o RITMO, o PENTE e o ESPALHAMENTO

> **Estado:** plano (2026-09-19). O item é o que o `#14 ProjectileMotion` deixou **ABERTO por
> escrito** no `CLAUDE.md` §5 (*«⏳ ABERTO: o `WeaponFire` (P1)»*) e o que o
> [levantamento §6](00_levantamento_componentes.md) descreve como *«a ARMA do player:
> cooldown/ammo/reload/overheat/spread — completa o trio WeaponFire + ProjectileMotion + Spawner»*.
>
> ⚠️ **Este doc começa pela MEDIÇÃO**, porque nesta linha a §5.0 já REESCREVEU seis entregas — e
> reescreveu esta também: **uma das quatro peças do item já está construída**, e outra **não é deste
> componente**.

## §1 — O que a composição JÁ dá (medido em 2026-09-19)

A sonda é [`mede_o_que_a_composicao_ja_da_a_arma`](../../crates/ph2d-app-components/tests/it/mede_o_que_a_composicao_ja_da_a_arma.rs),
e o lado medido **não é um espantalho**: é a melhor composição que esta casa tem hoje, feita de
**cinco** componentes que esta mesma linha shipou — `SignalOnAction` (#18) · `Factory` +
`aim_from_spawner` (#11 + #18) · `Timers` (#2) · `Counter` (#20) · `SignalActions` (#5).

| a pergunta | a resposta MEDIDA | o que ela decide |
|---|---|---|
| **A MIRA** — a bala sai apontada? | ⭐ **SIM, ao bit**: com o herói a `1,5707964` rad, o nascimento traz `aim = Some(1,5707964)` e a pose `[2,0 ; 0,0]` | ✅ **FECHADA** pelo gatilho (#18) — esta wave **não lhe toca** |
| **A CADÊNCIA** — segurar o gatilho um segundo | ⛔ **`60` balas**: a linha `Hold` fala em TODO tique e a fábrica nasce a cada sinal | *uma arma é, antes de tudo, um RITMO* |
| **…e o desvio pelo RELÓGIO** (o que um artista escreveria) | a 1.ª bala sai no tique **`15`** (`0,250 s` depois de carregar) e saem **`3`** onde se pediam `4` | ⛔ *um relógio dispara no FIM do período e uma arma dispara no INÍCIO* — carregar e não acontecer nada lê-se como um botão partido |
| **AS MUNIÇÕES** — a arma pára quando o pente acaba? | ⛔ **NÃO**: `10` gatilhos num pente de `6` dão **`10` balas** e o contador vai a **`−4`** | nada liga o contador à fábrica; o `total_max` dela conta a CORRIDA inteira |
| **A RECARGA** — há verbo que REPONHA um contador? | ⛔ **um só, e ele SOMA** (`Add to Counter`, de `10` verbos) | repor o pente cheio só acerta com ele a ZERO — recarregar com três dentro dá **nove** ⇒ **INEXPRIMÍVEL** |
| **A DISPERSÃO** — três balas da mesma rajada | ⛔ os três ângulos são **o MESMO** (`0,0`) | uma rajada sai como uma bala só, e uma caçadeira é exactamente a diferença |

⇒ **O buraco tem três nomes — ritmo, pente e espalhamento — e nenhum deles se compõe do que
existe.** A mira, que era a quarta peça do item, **já está paga**.

## §2 — O desenho, e a PORTA ÚNICA de cada pergunta

| pergunta | a porta | ⛔ o que NÃO é |
|---|---|---|
| **quando posso disparar outra vez** | `WeaponFire::cooldown_ms` | ⛔ **não** é um `Timer` — ver §2.1 |
| **quantas balas tenho** | um [`Counter`] NOMEADO (`WeaponFire::ammo_counter`) | ⛔ **não** é um campo do runtime — ver §2.2 |
| **como se recarrega** | `reload_ms` (automática ao esvaziar) + `reload_on` (ao sinal) | ⛔ **não** é um verbo novo na tabela |
| **o que sai do cano** | o sinal `on_fire`, que a `Factory` ouve | ⛔ **não** é um `master` aqui — dois instanciadores seriam dois motores |
| **quantas por tiro e com que abertura** | `Factory::burst` (já existe) + `Factory::spread_deg` (novo) | ⛔ **não** é da arma — ver §2.3 |

### §2.1 — ⭐ Porque o `cooldown` NÃO é um `Timer`, com a medição ao lado

A `Factory` **recusou** ter um `rate` próprio, e a razão dela está escrita no `factory.rs`: *«um
`Timer{repeat, signal}` mais `Factory{on_signal}` já dão «nasce um por segundo», e um `rate` aqui não
compraria capacidade nenhuma»*. **Aquela recusa continua de pé e não responde a esta pergunta.**

Um relógio é uma **emissão periódica**; uma cadência é um **piso no intervalo entre pedidos que são
honrados** — ela é conduzida pela MÃO, não por um relógio. A diferença está medida no §1: pelo
relógio a primeira bala chega **um período inteiro atrasada** e a taxa lê `3` onde se pediam `4`.
*A recusa da fábrica responde «quem dá o ritmo a quem nasce sozinho»; esta pergunta é «quanto tempo
depois de eu carregar»*, e nenhum relógio a responde.

### §2.2 — ⭐⭐ O PENTE é um `Counter`, e é isso que o põe no HUD de graça

O `LabelSource::Counter(String)` do #20 lê **a soma dos contadores com um nome**. Se a munição
vivesse no `WeaponRuntime`, o placar não a veria e a wave teria de inventar uma sétima fonte de
rótulo. Com o pente a ser um contador:

- o HUD mostra a munição **sem uma linha nova**;
- o Inspector já o edita;
- *«apanhei um pacote de balas»* é o `Add to Counter` que já existe;
- e **o `Ctrl+Z` e o rebobinar já sabem o que fazer com ele** (o `CounterRuntime` entrou no
  `rewind_runtime` na wave do HUD).

⚠️ **O contador vive na MESMA entidade da arma** (`requires`), senão escrever nele seria ambíguo —
a leitura soma todos os do nome, e a escrita precisa de um dono. ⛔ E a munição **não** é
duplicada no runtime: *dois sítios com o número de balas divergem no dia em que um deles ganhar
uma cerca*.

### §2.3 — ⭐⭐ O ESPALHAMENTO é da FÁBRICA, e a prova é o GERADOR

Ele é uma propriedade do **NASCIMENTO** — irmão do `burst` (*quantas de cada vez*) e do
`aim_from_spawner` (*para onde*), que já vivem lá. E há uma prova mais dura que a arrumação: um
espalhamento precisa de um **gerador com semente**, determinista e que renasça no rebobinar. A
`Factory` tem um (`FactoryRuntime::rng`, semeado com a identidade dela). Pô-lo na arma criaria
**um segundo gerador** — e o `CLAUDE.md` §5.0 já paga essa lição noutro sítio.

⇒ `Factory::spread_deg: f32` — a abertura TOTAL do cone, em graus. `0` = **byte-idêntico ao de
hoje** (a lei degenera, não é um ramo).

## §3 — Onde isto encosta em contrato e schema

- ⛔ **Zero contrato congelado** (§6): nada em `NodeOp`/`Tool`/`Vector` é tocado.
- `PROJECT_SCHEMA` **+1** — o `WeaponFire` é tipo novo no snapshot **e** o `Factory` ganha um campo,
  e o postcard é POSICIONAL.
- Registo do `ph2d-ecs` **+1** e os **dois espelhos +1** cada (`ph2d-render`, `ph2d-script`).
  ⛔ O `WeaponRuntime` **não se regista** — é estado vivo, como o `FactoryRuntime`.
- `LIVE_SECTIONS` **+1**; catálogo do `ph2d-component-desc` **+1** (⚠️ a lista é procurada por busca
  **binária** — a entrada entra na posição alfabética do id).
- `rewind_runtime` **+1 membro** — *rebobinar é RENASCER*: o cooldown zera, a recarga cancela, e o
  pente volta ao `start` (que o `CounterRuntime` já faz).

## §4 — A UI, nas quatro condições independentes

1. **o componente EXISTE** — `WeaponFire` no catálogo, categoria `Logic`, `O::ANY`, com
   `requires = ["ph2d::ecs::Counter"]`.
2. **é pintado e registado** — secção própria do Inspector com os oito campos, e os ids no array
   do painel.
3. **o clique chega ao barramento** — `ComponentEdit::Weapon*` pelo `action_bus`, como as irmãs.
4. **a SEQUÊNCIA leva a algum lugar** — a cena de smoke: carregar numa tecla dispara, a cadência
   limita, o pente esvazia, a recarga enche, e o HUD mostra a munição.

## §5 — Os gates, red-first

| # | o que afirma | porquê ele existe |
|---|---|---|
| G-1 | o 1.º tiro sai no tique **`0`** | é o defeito medido no §1 (a 1.ª bala atrasada) |
| G-2 | segurar o gatilho um segundo dá `1/cooldown` tiros, não `60` | o defeito principal |
| G-3 | com `cooldown_ms = 0` todo pedido dispara | o neutro é observável |
| G-4 | o pente esvazia e a arma **pára**, com `on_empty` a falar | (D) do §1 |
| G-5 | a recarga repõe o **`start`** do contador, nunca soma | (E) do §1 |
| G-6 | recarregar com balas dentro dá o pente CHEIO e não `start + resto` | a armadilha do `Add to Counter` |
| G-7 | durante a recarga o gatilho **não** dispara | senão a recarga é decorativa |
| G-8 | `ammo_counter` vazio ⇒ munição infinita e nenhuma escrita | o neutro da 2.ª metade |
| G-9 | `spread_deg = 0` dá os ângulos **byte-idênticos** aos de hoje | o neutro da fábrica |
| G-10 | `spread_deg = 20` com `burst = 3` dá três ângulos DISTINTOS dentro de `±10°` | (F) do §1 |
| G-11 | duas fábricas com a mesma semente e espalhamentos diferentes **não** dão a mesma sequência | a lei do gerador |
| G-12 | rebobinar zera cooldown, cancela a recarga e enche o pente | *rebobinar é RENASCER* |
| G-13 | a secção do painel está na `LIVE_SECTIONS` e chega a PIXEL | a lei da wave do HUD |
| G-14 | o gesto REAL do dono na cena arma e dispara | costura |

**A fixtura contém o fenómeno:** a cadência mede-se ao longo de **um segundo inteiro** de tiques
(uma fixtura de dois tiques não distingue `cooldown` nenhum), e a do pente pede **mais pedidos que
balas**, senão a cerca nunca é observada.

## §6 — A cena de smoke

`PH2D_WEAPON_SMOKE=1` — um herói que aponta e dispara, com o **CONTROLO na própria cena**: duas
armas com a MESMA tecla e **UM knob de diferença** (uma com cadência e pente, a outra sem nenhum
dos dois), e o HUD a mostrar a munição de cada uma. Os números saem da sonda headless ANTES da
mensagem ao dono.

## §7 — O que fica FORA, com o motivo

- ⛔ **`overheat`** (o levantamento nomeia-o): é a mesma grandeza do pente com outra palavra — um
  número que sobe com o tiro e desce com o tempo. Construí-lo agora seria um **segundo motor de
  munição** antes de alguém o pedir. Fica nomeado.
- ⛔ **munição de RESERVA** (o pente cheio sai de um depósito maior): pede um segundo contador e uma
  lei de transferência; a wave entrega o pente, e o depósito compõe-se com `Add to Counter`.
- ⛔ **um verbo `Fire` na tabela**: um verbo age sobre um ALVO, e uma arma dispara de quem a tem —
  o mesmo argumento que manteve o `Shake` fora da tabela no #25.
