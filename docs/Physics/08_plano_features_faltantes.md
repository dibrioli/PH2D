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
| **o sensor lateral só olha o meio do corpo** | ❌ **JÁ FEITO** — `WALL_SAMPLES = 3` cobre o flanco. ⚠️ E a fresta que restava foi **MEDIDA na §4.3 e é benigna**: para os três raios falharem juntos ela teria de ser mais alta que o corpo. O shape cast existe (`sweep_body`) e a parede **não** o usa, com motivo |

---

## §4 — A FILA, com o racional de ordem

> **A ordem não é por tamanho nem por gosto: é por DEPENDÊNCIA e por RISCO.** O
> que destrava outros vem primeiro; o que toca o wrapper vem cedo, para o
> problema aparecer com tempo; o que é ajuste de aparência vem por último,
> porque quem o julga é o smoke e não um gate.

### 4.1 · `W-Swim` — **NADAR** ⟨o item do Enio⟩ — ✅ **CONSTRUÍDO (2026-08-10)**

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

#### O que de facto foi construído, e onde o plano errou

**A `waterborne` NÃO foi tocada** — e o aviso acima, embora correto sobre o
risco, apontava para o lugar errado: a trava do arco e a trava do nado são
**duas perguntas distintas** (*"saí do arco balístico?"* × *"estou a nadar?"*),
com limiares diferentes e desarmes diferentes. Colapsá-las teria sido a segunda
resposta que o aviso temia, por outra via.

**O limiar saiu de medição** (`ph2d-physics/tests/measure_the_swim_threshold`),
e a tabela **mudou a forma do número**: a `Buoyed` mistura densidade e imersão,
então converter um limiar em ALTURA depende das duas densidades da cena.

```text
  buoyed >= 0.25  <=>   9,6% submerso
  buoyed >= 0.50  <=>  15,8%
  buoyed >= 1.00  <=>  27,2%   <- o default: a LINHA DE FLUTUAÇÃO
  buoyed >= 1.50  <=>  38,7%
  submerso 100%    =   3,99    (= densidade do fluido / a do corpo)
```

⇒ o default é **`1.0`**, e ele é uma frase de física: *a água sozinha me
sustenta*. É por construção a linha em que o corpo boiaria parado, **em qualquer
densidade** — enquanto uma altura diria coisas diferentes em cada poça.

**A entrada vertical não custou schema.** O plano dizia *"não tem entrada livre
— as cinco estão tomadas"*, e a saída não foi uma sexta: **`jump` é subir,
`down` é descer** (o mapeamento do Mario/Rayman). O preço do alternativo está
medido: a fita do jogador guarda um quadro como `(f32, u8)`, então um eixo novo
mudaria a forma do arquivo e **recusaria toda corrida já salva**; um botão é um
bit que já lá está.

**A saída é uma TRAVA**, não o limiar ao contrário — armar pede `buoyed >=
enter`, continuar pede só `buoyed > 0`, e o **chão desarma**. Sem a histerese o
nadador oscilaria em torno do limiar exatamente onde o jogador tenta emergir.

⚠️ **E a medição achou um fato de produto que o plano não previa:** numa poça
`d` vezes mais densa que o corpo, o empuxo líquido sobre um corpo submerso vale
`|g|·(d − 1)` — na fixture destas waves, **29,4 m/s²**. Com a autoridade de
partida do servo (`12`) o personagem **não consegue mergulhar**: ele apenas sobe
mais devagar. *Não é defeito, é a física da cena* (uma rolha não mergulha por
querer), e o gate `diving_needs_more_authority_than_the_water_has` a torna
executável nos dois sentidos.

**Superfície:** `PROJECT_SCHEMA` **70→71** (três campos apendados ao
`PlatformPlayer`, ⚠️ **provisório** — conta-se contra o `main` do dia) · card
**SWIM** na §14 (3 rows) · **4 ids** novos, todos `hash_node_id` · registro do
`ph2d-ecs` e o do `ph2d-physics-ecs` **intocados** (o estado é da lei, não um
componente) · **zero `Cargo.toml`** · **nenhum ADR** · cena **`=105`**.

#### As duas perguntas do Enio, e o defeito que a segunda achou

> *"Kinematic não vai nadar?"* · *"não temos parâmetros para o quanto fica
> submerso quando boia na superfície?"*

**(1) Vai** — e já ia: os gates de produto varrem `[false, true]` em **todos**,
porque o `player_motor` é um só e o `kinematic_advance` honra o `motor.accel` e
o `motor.boost` que a braçada escreve. O que faltava era o artista poder **ver**,
e a cena `=105` passou a montar um terceiro corpo, **verde**, cinemático. *Uma
capacidade que só os testes conhecem é uma capacidade que o próximo report
pergunta de novo.*

**(2) Tem, e são as duas DENSIDADES** — a submersão de repouso é `1/razão`,
exata. Medido (`measure_the_float_line`), com a capacidade **desligada**:

```text
  fluido 4,00x  ->  24,5% submerso   (previsto 25%)
  fluido 2,00x  ->  50,1%            (previsto 50%)
  fluido 1,25x  ->  80,1%            (previsto 80%)
```

⚠️ **Mas a pergunta destapou um defeito, e ele era grande:** o repouso do nado
mirava **velocidade vertical zero**, com um doc-comment meu a dizer *"o que
sobra é o que a água faz com ele"*. O servo **cancela** o empuxo a cada tique,
então o nadador parado não boiava — congelava onde estava:

```text
  fluido 1,25x, nado 0  ->  80,1% submerso   (a linha da física)
  fluido 1,25x, nado 4  -> 100,0% submerso   (afundado, e lá ficava)
```

⇒ **ligar o nado AFUNDAVA quem boiava.** O repouso passou a procurar a LINHA.

⚠️ **E não há knob novo, de propósito** — a linha **é** o `swim_enter`. Um
`ride` próprio teria de ser co-afinado com ele (o valor certo de um seria função
do outro, [[feedback_ergonomics_verdict_is_a_design_bug]]) e o caso degenerado é
feio: mirar uma linha que o fluido não alcança faz o nadador remar para baixo
**para sempre**. Com `ride ≡ enter` esse estado é **inexprimível**, porque o
regime só arma quando `buoyed >= enter` de facto acontece.

Resultado, nos dois modos: **25,0% · 49,9% · 79,8%** contra `25 · 50 · 80`
previstos — e a amplitude do bobeio a `4×` cai de **0,43 m para 0,00**.

⚠️ **Fronteira NOMEADA:** um corpo de densidade **neutra** (razão `1,00`) nunca
arma o nado com o default, porque a tesselação do collider lê `buoyed ≈ 0,996`
submerso por completo (o viés de `0,64%` que o `AreaBuoyancy` documenta). Ele
fica onde está sem fazer nada — a flutuação neutra sai de graça da razão —, e
quem quiser que ele NADE baixa o `swim_enter` um pouco.

### 4.2 · ~~`W-ZoneForce`~~ — **FECHADA (2026-08-10)**, e a cura foi maior que o item

Uma correnteza movia o modo Dynamic e **não** movia o Snap/Push/Pure: o
`effector::apply` recusa corpo não-dinâmico antes de tocar nele, e a lei
cinemática integrava `Fluid { buoyed, drag }` — sem força. **Medido antes:
`0,0000 m` em QUALQUER força**, contra 21,83 m de um caixote solto.

⚠️ **E a medição achou um defeito que o item não previa, no caminho para a
porta: um corpo COMPOSTO recebia a força, o torque e o arrasto uma vez por
FORMA.** A `W-CompoundZone` (02/08) curou exatamente isto no EMPUXO e deixou os
outros três dentro do laço de pares — com a massa mantida fixa e a área partida
em 1/2/4 peças, o mesmo vento levava o corpo `1,00× · 2,00× · 4,00×`, e o
arrasto compunha ao contrário (`v·kᴺ`: 4,95 → 2,53 → 1,28 m). **Isto é alcançável
hoje por um player com pé-sensor** (a `W-PartSensor` deu-lhe a segunda forma), e
uma porta não pode reproduzir um bug ⇒ curado primeiro. A lista deduplicada
passou a servir os **cinco** efeitos da zona, e por isso deixou de se chamar
`to_float`.

**A cura do item foi a que o plano prescreveu:** porta única
`effector::zone_push_at` (frame ∘ espelho ∘ falloff, os dois EMPURRÕES) chamada
pelo solver **e** pela `fluid_at`, que ganhou `push: [f32; 2]` — a **ACELERAÇÃO**,
não a força, porque quem pergunta é uma lei que não tem massa na mão; a divisão
usa a massa REAL do solver e é ela que preserva *a folha voa, o caixote não*.

⚠️ **Duas mudanças de comportamento, nomeadas:** a consulta deixou de sair pelo
early-out de **gravidade** (uma correnteza não precisa de peso para empurrar), o
que faz o `drag` passar a ser reportado num mundo sem gravidade; e o filtro
`displaces` passou a guardar só o **empuxo**, para que a consulta veja as mesmas
formas que o solver vê (um pé-sensor não desloca fluido, mas o corpo dele é
empurrado na mesma).

⚠️ **O TORQUE fica fora da consulta, com motivo:** a porta o devolve, e o
consumidor é uma lei que não integra velocidade angular — um personagem fica em
pé por construção (`LockRotation`). Entregá-lo seria um knob morto.

**Medido depois** (freio de fábrica, 16 N, 2 s): caixote 87,3 · dinâmico 21,0 ·
cinemático 21,4 · puro 21,4. Cena **`=106`**. `physics_ecs_c9` byte-idêntico.

<!-- o texto original do item, para a próxima LLM ver o que foi prescrito: -->

⚠️ **O motivo escrito é bom e a cura tem de o honrar:** a força de uma zona
precisa do **frame** (W-AreaFrame), do **espelho** (W-AreaMirror) e do
**falloff** (W-AreaFalloff), e re-derivá-los numa consulta seria a **segunda
resposta** a *"que empurrão esta zona dá neste ponto?"*. ⇒ **A cura é estender a
consulta que já existe** (`fluid_at`, que hoje devolve empuxo e arrasto numa
varredura só) para devolver também a força **já resolvida pelas portas do
solver** — nunca uma re-derivação no lado do player.

Vem logo depois do nadar porque **os dois se encontram na mesma cena**: um
nadador numa correnteza é o teste de que as duas metades falam a mesma língua.

### 4.3 · ~~`W-ShapeCast`~~ ✅ **FEITO** — o wrapper varre o CORPO

**A porta é `PhysicsWorld::sweep_body`** (`world/sweep.rs`): varre **todos** os
colliders do corpo ao longo de uma direção e devolve o impacto mais próximo, com
o mesmo filtro do `cast_ray` (camadas · `EXCLUDE_SENSORS` · a exclusão do próprio
corpo, que aqui deixa de ser higiene e passa a ser aritmética — a forma nasce em
cima do próprio collider).

**Medido ANTES de uma linha ser escrita** (`measure_the_gap_between_rays`), e o
defeito estava na direção que um doc declarava impossível: um pilar de **8 cm**
posto no vão entre duas amostras do sensor do agachar punha a cabeça a **1,267**
contra uma face de pedra em **1,25**, com o corpo ainda debaixo dela; um de 4 cm
fazia o solver expulsá-lo **0,155 m** de lado. O doc do `probe_headroom` dizia
*"o erro possível é ficar agachado onde caberia, nunca levantar-se para dentro da
pedra"* — verdade sobre a **caixa contra a cápsula**, falsa sobre o vão entre dois
raios, e a segunda metade é a perigosa.

**O consumidor é o sensor do AGACHAR**, e ele devolveu duas coisas: o vão cego
morreu, e a **caixa envolvente** morreu com ele (a varredura usa a cápsula, o que
vale **18 cm** de altura de teto que a caixa recusava sobre espaço vazio).

⚠️ **Os outros dois sensores NÃO foram convertidos, e o motivo está em cada um:**

| sensor | por que fica de raios |
|---|---|
| **quina** (65 amostras) | é um **PERFIL** — a lei precisa de saber *onde* há teto para escolher para que lado escapar, e uma varredura devolve **um** contacto |
| **parede** (3 alturas) | a lei **reduz** sobre as amostras com a régua da perna (`max_slope`) para decidir *qual* superfície é parede; uma varredura entregaria a rampa aos pés. Trocar seria perder a escolha, não ganhar precisão |

E o vão cego que sobra na parede é o **benigno** (um buraco reportado como
parede): para os três raios falharem juntos a fresta teria de ser mais alta que o
corpo, e aí não há parede em frente ao corpo.

⚠️ **A promessa do ledge grab NÃO foi paga aqui, e a §4.5 muda de forma por
causa disso:** *"onde o chão acaba"* é uma pergunta de **PERFIL** (como a quina),
não de varredura. O `sweep_body` diz *"o corpo cabe se eu andar para ali?"*, que
é o que o **mantle** vai querer; achar a beirada continua a ser um segundo par de
raios ou um perfil. O item fica com a metade certa nomeada em vez de com uma
dependência que não se realiza.

**Cena `=107`** · `physics_ecs_c9` **byte-idêntico** (o hash não tem lane que
agache) · 10 gates na porta + 4 no produto + 6 na cena, com a mutação que devolve
os três raios a sangrar dois deles.

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

**Onde entra na fila:** ⚠️ **antes** do `W-Ledge`, porque as waves 4.4/4.5 vão
**acrescentar sensores**: um ledge grab sem os raios visíveis é afinado às cegas,
que é o que este item existe para acabar.

⚠️ **E a razão que o prendia ao `W-ShapeCast` MORREU com a medição.** Este
parágrafo dizia *"quando o chão deixar de ser um raio (4.3) o desenho tem de
mostrar a forma nova"* — o chão **continua a ser um raio**: a 4.3 converteu só o
sensor do agachar, e os outros dois ficaram de raios com o motivo escrito em cada
um. Sobra **um** sensor a desenhar como forma (o do agachar, uma cápsula varrida
para cima) contra cinco a desenhar como linha, e isso é uma nota no item, não uma
dependência de ordem.

⚠️ **Corolário para quem construir isto:** o overlay tem de saber desenhar as
DUAS coisas. Um overlay que só saiba linhas desenha o sensor do agachar como um
raio que ele não é — e o artista afinaria o `crouch_height` contra um desenho que
mente sobre o que o produto mede.

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
~~W-Swim~~ ✅ → ~~W-ZoneForce~~ ✅ → ~~W-ShapeCast~~ ✅ → W-Probes → W-MultiJump → W-Ledge → (W-Glide?) → o ajuste
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
