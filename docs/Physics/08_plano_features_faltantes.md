# Plano 08 — o que falta ao PLAYER, medido contra o catálogo (2026-08-10)

> **Pedido do Enio:** *"Vamos implementar todos e vc deve fazer uma pesquisa
> sobre features faltantes em nossos Players Platform. Exemplo: Pulo múltiplo e
> Ledge grab. Faça uma pesquisa e junto aos 4 planeje a fila de implementação."*
>
> Este doc é **plano, não implementação**. Ele traz o censo do que existe (lido
> do código, não de memória), a pesquisa confrontada com ele, e a fila com o
> racional de ordem. ⚠️ **A pesquisa foi LIMITADA a três frentes de propósito**
> ([[feedback_a_research_fanout_recurses_bound_it]]), e o **fato decisivo de cada
> achado foi verificado aqui**, não aceite da web.

---

## §1 — O censo (lido do código)

**Entradas: cinco.** `drive` · `jump` · `down` · `dash` · `grab`.

**Leis: onze módulos.** `ride` (a perna flutuante) · `walk` (caminhada + rampa)
· `jump` (arco, coyote, buffer, altura variável, trava do fluido) · `react` (a
3.ª lei) · `wall` (slide · jump · grab) · `dash` · `crouch` · `corner`
(correção de quina, 65 raios) · `slope` · `kinematic` (os três modos) ·
`contract`.

**Fora da lei, no módulo:** plataforma móvel · descida de one-way · a fita ·
o bake · empuxo/arrasto/trava de fluido.

---

## §2 — A pesquisa, confrontada

O catálogo que a indústria repete (Godot asset library · o motor 2D do cjddmut ·
GDevelop · GDQuest · o `bevy_tnua`, que é o **referencial declarado deste
módulo**) tem **dezoito** itens recorrentes. Confrontados com o censo:

| item do catálogo | nós | onde |
|---|---|---|
| aceleração/desaceleração | ✅ | `walk` |
| altura de pulo variável | ✅ | `cut_gravity` |
| coyote time | ✅ | W8 |
| jump buffer | ✅ | W8 |
| modelagem do arco (leve no ápice, pesado na queda) | ✅ | W4 |
| wall slide | ✅ | W13 |
| wall jump | ✅ | W13 |
| wall grab | ✅ | W23 |
| dash | ✅ | W14 |
| agachar | ✅ | W15 |
| corner correction | ✅ | W10 |
| descer de one-way | ✅ | W12 |
| preservação de momento | ✅ | `lift_momentum`, W10 |
| rampas / `max_slope` | ✅ | `slope` |
| plataformas móveis | ✅ | `ground_velocity` |
| **pulo múltiplo (air actions)** | ❌ | — |
| **ledge grab / hang / mantle** | ❌ | — |
| **planar (glide)** | ❌ | — |
| **escalar (escada / corda)** | ❌ | — |
| **nadar** | ❌ | — |

⚠️ **O catálogo está 15/20 coberto**, e os cinco buracos são exactamente os que o
Enio nomeou por instinto (*"pulo múltiplo e ledge grab"*) mais três.

### ⚠️ O achado que a pesquisa trouxe e que muda a FILA

A doc do `bevy_tnua` nomeia um problema estrutural do desenho que ele e nós
partilhamos: *"Tnua, by default, casts a single ray to the ground. This can be a
problem when the character stands on a ledge, because the ray may be past the
ledge while the character's collider isn't."*

**Verificado aqui, não aceite:** o nosso sensor de chão é
`cast_ray_skipping(origin, [0,-1], …)` — **um raio**
(`bridge/player.rs:250`). O sensor de parede tem **três** alturas
(`WALL_SAMPLES = 3`) e o `wall.rs` já nomeia o que resta: *"uma fresta mais
estreita que meia altura, entre duas amostras, segue invisível. **A cura dos três
sensores é a mesma** (um shape cast, que este wrapper ainda não tem)"*.

⇒ **O ledge grab não é uma feature de lei: ele é uma feature de SENSOR.** Saber
que há uma beirada é saber que o chão acaba *dentro* da largura do corpo — e um
raio no eixo não pode responder isso. É por isso que ele entra na fila **depois**
do item de infra, e não por gosto de ordem.

---

## §3 — Os quatro itens do Enio, re-medidos

⚠️ **Um deles já está feito**, e a nota que o listava sobreviveu ao fato
(corrigida no plano 06 na mesma sessão):

| item | veredito |
|---|---|
| **nadar** | ✅ real — é *feature*, nunca foi correção |
| **a FORÇA de uma zona não leva um cinemático** | ✅ real — e é **desenho**, não esquecimento (ver §4.2) |
| **`fall_gravity` na entrada da água** | ⚠️ **é ajuste de APARÊNCIA, não wave** (ver §4.7) |
| **o sensor lateral só olha o meio do corpo** | ❌ **JÁ FEITO** — `WALL_SAMPLES = 3` cobre o flanco; o que resta é a fresta sub-meia-altura, e a cura dela é o **shape cast** (§4.3) |

---

## §4 — A FILA, com o racional de ordem

> **A ordem não é por tamanho nem por gosto: é por DEPENDÊNCIA e por RISCO.** O
> que destrava outros vem primeiro; o que toca o wrapper vem cedo, para o
> problema aparecer com tempo; o que é ajuste de aparência vem por último,
> porque quem o julga é o smoke e não um gate.

### 4.1 · `W-Swim` — **NADAR** ⟨o item do Enio⟩

O primeiro porque o contexto está **quente** (a água acabou de ser medida hoje) e
porque ele não depende de nada novo: o `Buoyed` e o `Fluid` já chegam à lei, e a
trava `waterborne` já nomeia *"o corpo saiu do arco balístico"*.

**O desenho, numa frase:** nadar é um **REGIME**, como o dash — dentro dele a
perna, a modelagem do arco e a caminhada calam-se, e o que resta é uma velocidade
que o `drive` dirige nos DOIS eixos contra o arrasto do meio.

⚠️ **A pergunta que decide, e ela é de produto:** *entra-se em natação por
SUBMERSÃO ou por BOTÃO?* Submersão é o Ori/Hollow Knight (automático); botão é o
Rain World. O módulo já tem a grandeza para o automático (`Buoyed`), e não tem
entrada livre — as cinco estão tomadas. **Recomendo submersão**, com o limiar a
sair de medição e não de palpite.

⚠️ **O que já sabemos que vai morder:** a `waterborne` é uma **trava que só o
CHÃO desarma**. Um nadador que sai da água por cima precisa que ela desarme sem
chão, ou o arco fica calado no ar depois de sair — e isso é exactamente o
mecanismo que a medição de hoje provou estar a funcionar. **Mexer nela sem um
gate de regressão é como o `857 m` volta.**

### 4.2 · `W-ZoneForce` — **a força de uma zona leva um personagem cinemático** ⟨o item do Enio⟩

Hoje uma correnteza move o modo Dynamic e **não** move o Snap/Push/Pure: o
`effector::apply` recusa corpo não-dinâmico antes de tocar nele, e a lei
cinemática integra `Fluid { buoyed, drag }` — sem força.

⚠️ **O motivo escrito é bom e a cura tem de o honrar:** a força de uma zona
precisa do **frame** (W-AreaFrame), do **espelho** (W-AreaMirror) e do
**falloff** (W-AreaFalloff), e re-derivá-los numa consulta seria a **segunda
resposta** a *"que empurrão esta zona dá neste ponto?"*. ⇒ **A cura é estender a
consulta que já existe** (`fluid_at`, que hoje devolve empuxo e arrasto numa
varredura só) para devolver também a força **já resolvida pelas portas do
solver** — nunca uma re-derivação no lado do player.

Vem logo depois do nadar porque **os dois se encontram na mesma cena**: um
nadador numa correnteza é o teste de que as duas metades falam a mesma língua.

### 4.3 · `W-ShapeCast` — **o wrapper ganha varredura de FORMA** ⟨infra⟩

Não é feature: é o que os **três** sensores pedem por escrito (`wall.rs`), e o
que o ledge grab exige para existir. Um `cast_shape` responde *"o corpo cabe
aqui?"* em vez de *"esta linha toca algo?"*, e com ele:

* a fresta sub-meia-altura deixa de ser invisível (o resto do item 4 do Enio);
* o chão deixa de ser **um raio** — o problema que o tnua documenta e que nós
  temos igual;
* o ledge grab passa a ter como perguntar *onde o chão acaba*.

⚠️ **Toca `ph2d-physics` (o wrapper), que é a crate mais compartilhada desta
linha** — por isso vem CEDO, com tempo para o problema aparecer, e não véspera de
integração. O rapier já traz `cast_shape`; o trabalho é a porta, o filtro
(`EXCLUDE_SENSORS`, a mesma frase do `cast_ray`) e o determinismo.

### 4.4 · `W-MultiJump` — **PULO MÚLTIPLO** ⟨o exemplo do Enio⟩

O `air actions counter` do tnua, e o item mais pedido do catálogo. É **estado no
`JumpState` + um número no `JumpConfig`**, e encaixa onde coyote e buffer já
vivem.

⚠️ **A armadilha, e ela é a razão de isto NÃO ser trivial:** o contador tem de
zerar no CHÃO, e *"estou no chão"* já é uma porta única com **dois** consumidores
(`JumpState::on_ground`, que o `jump` e o `dash` partilham). Um terceiro
consumidor com a sua própria cópia do predicado daria *"às vezes o duplo pulo não
recarrega"* — o sintoma exacto que aquele doc-comment já descreve para o dash.

⚠️ **E a segunda pergunta é de produto:** o pulo do ar tem a **mesma altura** do
primeiro (Celeste) ou menor (Hollow Knight)? Um número, e ele muda o jogo.

### 4.5 · `W-Ledge` — **LEDGE GRAB + MANTLE** ⟨o exemplo do Enio⟩

**Depende do 4.3.** São duas metades que a literatura trata como uma:

* **hang** — o personagem agarra a beirada e fica pendurado (regime, como o wall
  grab, e provavelmente partilha o `grab` que já existe);
* **mantle** — subir por cima, que é um deslocamento AUTORADO e não uma força.

⚠️ **A decisão de desenho que tem de vir antes do código:** o mantle move o corpo
para um lugar que a física não escolheu. No modo cinemático isso é natural (o
controlador já escreve a pose); no **Dynamic** é um teleporte, e este módulo tem
uma regra sobre isso desde o W2a (*`set_body_pose` zera a velocidade*). **Os dois
modos podem precisar de respostas diferentes, e isso tem de estar escrito antes.**

### 4.55 · `W-Probes` — **OS SENSORES FICAM VISÍVEIS E PLENAMENTE EDITÁVEIS** ⟨pedido do Enio⟩

> *"os Rays (ou outros modos de cast) dos Setups dos Players não estão visíveis e
> nem são plenamente editáveis através da UI"*

**As duas metades do report foram verificadas, e as duas são reais** — mas por
motivos DIFERENTES, e a distinção decide o trabalho.

#### (a) VISÍVEIS: não há nada. Zero.

O overlay de física (tecla `B`) desenha collider, joint, glifo de zona, seta de
força, anel de falloff, linha d'água, cruz de contato e as ferramentas de ponto.
**Dos cinco sensores do player ele não desenha um pixel** — a única linha que
menciona player ali é o bit da descida de prancha (W20).

⚠️ **Isto é exactamente a lei que fez o contorno do collider existir**, escrita
no W2a: *"um collider é invisível e um sprite é um quad, então a resposta é a que
Unity/Godot/Box2D dão: wireframe sobre a arte"*. Um **raio** é ainda mais
invisível que um collider — ele nem tem forma —, e hoje o artista afina
`float_height`, `cling_distance`, `corner_reach` e `wall_reach` **às cegas**,
inferindo o alcance pelo comportamento.

**Os cinco a desenhar** (todos já existem na ponte):

| sensor | raios | o que governa o alcance |
|---|---|---|
| chão | **1** | `float_height + cling_distance` (derivado — ver ⚠️ abaixo) |
| parede | **3** (`[0, −½h, +½h]`) | `wall_reach` |
| quina (corner correction) | **65** | `corner_reach` |
| teto | **65** | `corner_reach` **e** `rel_up · dt · CORNER_LOOKAHEAD` |
| headroom (levantar do agachado) | **3** | a altura de pé menos a agachada |

#### (b) EDITÁVEIS: os ALCANCES sim, o resto não — e "o resto" tem três espécies

Isto é o que a palavra *"plenamente"* aponta, e o censo separa-o em três coisas
com **vereditos diferentes**:

1. **A CONTAGEM de amostras é `const` de compilação** — `WALL_SAMPLES = 3` ·
   `CORNER_SAMPLES = 65` · `HEADROOM_SAMPLES = 3`. Não estão no componente nem no
   Inspector. ⚠️ **E o §0 manda medir antes de expor:** os 65 do corner são um
   número de custo, não de gosto, e um slider que os mova mexe no orçamento de
   raios por tique. **Medir primeiro, expor depois** — e talvez a resposta certa
   seja um teto medido em vez de um slider.
2. **O alcance do TETO / headroom não tem row nenhuma** (não existe
   `INSP_PLAYER_CEILING`). O teto empresta o `corner_reach` do pulo e mistura-o
   com um termo de velocidade — ou seja, hoje o artista mexe no teto **sem saber
   que está a mexer**, por um knob que diz outra coisa. ⚠️ **Isto é um achado do
   censo, não do report**, e é o mais próximo de um defeito nesta lista.
3. **O alcance do CHÃO é DERIVADO de propósito** (`float_height +
   cling_distance`) e ⛔ **não deve ganhar knob próprio**: seria a segunda porta
   para *"até onde a perna alcança"*, e as duas divergiriam. O que ele precisa é
   de ser **VISÍVEL** — que é a metade (a).

#### O desenho, e o que ele NÃO pode fazer

⚠️ **O overlay LÊ, nunca DERIVA.** Os offsets têm de vir das portas que a ponte
já usa (`wall_offsets` · `corner_offsets` · `headroom_offsets`) — um segundo
cálculo no lado do desenho seria uma segunda resposta a *"onde este raio nasce?"*,
e ela divergiria no primeiro dia em que alguém mexesse numa das duas. É a mesma
regra que o `scaled_shape` do W6 impôs ao contorno do collider, e que a seta da
zona do W-AreaFrame impôs à força.

⚠️ **E o raio tem de dizer se ACHOU:** um sensor desenhado sempre da mesma cor
responde *"onde ele olha"* e não *"o que ele viu"* — e a segunda é a pergunta que
o artista faz quando o personagem não se agarra à parede. Cor por resultado, como
o sensor magenta do W7 (apagado/aceso).

**Onde entra na fila:** ⚠️ **antes** do `W-Ledge` e de preferência **junto do
`W-ShapeCast`** — porque quando o chão deixar de ser um raio (4.3) o desenho tem
de mostrar a forma nova, e construir o overlay depois seria desenhá-lo duas
vezes. E porque as waves 4.4/4.5 vão **acrescentar sensores**: um ledge grab sem
os raios visíveis é afinado às cegas, que é o que este item existe para acabar.

### 4.6 · `W-Glide` — **PLANAR**

O mais barato do catálogo depois do multi-jump, e provavelmente **um caso do
mesmo mecanismo**: planar é um multiplicador de gravidade sob botão, na queda —
`fall_gravity` reduzido enquanto o dedo segura. ⚠️ **Se for isso, ele não é uma
wave: é um campo**, e o honesto é descobrir isso ao escrever a W-MultiJump em vez
de reservar-lhe uma linha na fila. Fica aqui **nomeado, com a suspeita escrita**.

### 4.7 · A entrada na água ⟨o item do Enio⟩ — **ajuste, não wave**

Medido hoje: o personagem cruza a superfície a `1,299×` a velocidade de uma
cápsula porque `fall_gravity = 2.0`, e mergulha mais fundo por isso. **Não há
defeito** (a trava contém, o bobeio decai). Se a sensação não agradar, o que se
mexe é um knob que governa o platformer inteiro — ⇒ **é decisão de aparência, e
o instrumento é o smoke**, não um gate. Fica no fim da fila de propósito.

### 4.8 · `W-Climb` — **ESCADA / CORDA** ⟨não pedido⟩

O quinto buraco do catálogo, e o mais caro: é um regime com **geometria própria**
(o que É uma escada?) e provavelmente um componente novo. **Não entra na fila sem
pedido** — está aqui para a lista ser honesta, não para ser construído.

---

## §5 — A ordem, numa linha

```
W-Swim → W-ZoneForce → W-ShapeCast → W-Probes → W-MultiJump → W-Ledge → (W-Glide?) → o ajuste
   1         2             3            4           5            6           7           8
```

**Porquê esta e não outra:** 1 e 2 são a água, e o contexto está quente hoje · 3
é infra e **destrava** 6, além de fechar o resto do item 4 do Enio · **4 vem
COLADO ao 3** — quando o chão deixar de ser um raio o desenho tem de mostrar a
forma nova, e construí-lo antes seria desenhá-lo duas vezes · 5 é independente e
pode trocar de lugar com 3 se a infra atrasar · 6 é o único com dependência dura,
e **precisa do 4 para ser afinado com os olhos** em vez de às cegas · 7 pode
dissolver-se num campo · 8 é aparência e o smoke julga.

⚠️ **Se a fila tiver de ser cortada, o corte honesto é depois do 4.** Os quatro
primeiros fecham tudo o que o Enio pediu (a água, a zona, o resto do sensor
lateral, e os sensores visíveis+editáveis); 5 e 6 são catálogo, e o catálogo
espera.

⚠️ **Cada wave desta fila fecha com as QUATRO condições de UI do plano 00** (o
componente existe · é pintado e registado · o clique chega ao barramento · e a
SEQUÊNCIA leva a algum lugar) **e com uma cena de smoke de números MEDIDOS** — a
próxima livre é a **`=105`**.

---

## Fontes da pesquisa

* [bevy_tnua — docs.rs](https://docs.rs/bevy-tnua/latest/bevy_tnua/) · [control_helpers](https://docs.rs/bevy-tnua/latest/bevy_tnua/control_helpers/index.html) — o referencial declarado deste módulo; de onde vem o *air actions counter* e a frase sobre o raio único na beirada
* [Unity-2D-Platformer-Controller (cjddmut)](https://github.com/cjddmut/Unity-2D-Platformer-Controller) — double jump, wall jump, corner grab
* [Advanced platformer movements — GDevelop](https://wiki.gdevelop.io/gdevelop5/extensions/advanced-jump/) · [Platformer behaviors](https://wiki.gdevelop.io/gdevelop5/behaviors/platformer/)
* [Double jump and Coyote Time — GDQuest](https://school.gdquest.com/courses/learn_2d_gamedev_godot_4/side_scroller_character/double_jump_and_coyote_time)
* [2D Platformer Controller — Godot Asset Library](https://godotengine.org/asset-library/asset/4696)
* [Movement feature list (gist)](https://gist.github.com/GivaldoF/3cde9c920a9a9c837734ec21a2b2eb31) — de onde vem o *glide*
