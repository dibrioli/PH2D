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

### 4.55 · `W-Probes` — **OS SENSORES FICAM VISÍVEIS** ✅ ⟨pedido do Enio⟩

> *"os Rays (ou outros modos de cast) dos Setups dos Players não estão visíveis
> e nem são plenamente editáveis através da UI"*

**FECHADO em 2026-08-10** (cena `=108`). ⚠️ **As duas metades do report tiveram
vereditos OPOSTOS**, e a segunda é o achado: *visíveis* era real e grande;
*editáveis* estava **refutado pela própria medição** — e a §4.55 anterior
construía um "defeito" em cima de um censo que contava um sensor duas vezes.

#### (a) VISÍVEIS: não havia nada. Zero. ✅

O overlay desenhava collider, joint, glifo de zona, seta de força, anel de
falloff, linha d'água, cruz de contato e as ferramentas de ponto — **e dos
sensores do player não desenhava um pixel**. É a mesma lei que fez o contorno do
collider existir (W2a: *um collider é invisível e um sprite é um quad*), levada a
uma coisa **ainda mais invisível**: um raio nem forma tem.

⚠️ **O CENSO decidiu o desenho, e ele veio antes do código.** Se os sensores
condicionais fossem lançados na maioria dos tiques, desenhar *só o que foi
castado* bastaria. Medido (`measure_player_probes`, 600 tiques de um personagem
a andar / pular / agachar):

| sensor | perguntado |
|---|---|
| chão | 100% |
| agachar | 50,3% |
| **parede** | **13,5%** |
| **quina / lado** | **6,0%** |

⇒ desenhar só o castado mostraria a perna e **mais nada em ~90% dos quadros**,
justamente nos dois sensores cujo alcance o artista afina às cegas. Então o
**ALCANCE é desenhado sempre** e a **cor diz o que aconteceu**, em três estados:

| estado | significa |
|---|---|
| `Idle` | armado, e a lei **não perguntou** neste tique |
| `Clear` | perguntou e **não achou** |
| `Hit` | perguntou e **achou** (mais um **TIQUE** na distância) |

⚠️ **O `Idle` é a metade nova**: ele separa *"a capacidade está lá, não é a
hora"* de *"o alcance é curto demais"* — dois vereditos opostos que produziam o
mesmo nada na tela. E uma capacidade **DESARMADA não publica marca nenhuma**:
seis raios apagados à volta de todo personagem de toda cena onde o pulo de
parede nunca foi ligado seria ruído permanente.

⚠️ **O estado não gastou um hue novo.** O overlay já tem nove famílias saturadas
(verde · ciano · violeta · magenta · âmbar · branco · laranja · amarelo ·
vermelho), e uma décima leria como *mais um sistema*. Ele mora na
**intensidade** mais o **tique**, que ainda responde *onde*.

⚠️ **E o overlay sabe desenhar as DUAS coisas**: o sensor do agachar VARRE o
corpo (`W-ShapeCast`), então ele é o **contorno real desenhado onde ele quer
ficar de pé** — pela mesma porta do contorno vivo, todas as formas de um corpo
composto.

**Custo:** +0,78 µs/tique (**0,005%** de um quadro de 60 fps), medido por ablação
das capacidades ⇒ always-on, o precedente dos contatos.

#### (b) EDITÁVEIS: ⚠️ **a premissa desta metade estava ERRADA**

A §4.55 anterior listava **cinco** sensores — chão, parede, quina, **teto** e
headroom — e derivava daí um achado: *"o alcance do TETO / headroom não tem row
nenhuma … o mais próximo de um defeito nesta lista"*. Medido:

1. ⛔ **Não existem cinco sensores, existem QUATRO.** `probe_ceiling` produz um
   `CeilingProbe` e ele tem **UM consumidor** (`corner_nudge`, a correção de
   quina) — *"quina"* e *"teto"* eram **o mesmo sensor contado duas vezes**, e o
   *"alcance do teto sem row"* era essa duplicata a pedir uma row para si mesma.
2. ⛔ **O alcance do headroom é DERIVADO de dois números autorados**:
   `rise = float_height − crouch_height`, e os dois têm row
   (`INSP_PLAYER_FLOAT`, `INSP_PLAYER_CROUCH_HEIGHT`). Ele é o caso (b.3), não um
   defeito — um knob próprio seria a **segunda porta** para *"quanto ele sobe ao
   levantar-se"*.
3. ✅ **Censo completo:** os **36** campos do `PlatformPlayer` têm leitor no
   Inspector. Nenhum número que a lei lê é inalcançável.
4. ⛔ **As CONTAGENS de amostra ficam `const`, com motivo medido.** O doc do
   `CORNER_SAMPLES` já traz o número (*o sensor inteiro custa +0,0002 ms por
   tique de subida, ~8 ns por raio*) ⇒ **não há recurso a trocar**: mais amostras
   é só mais precisão (o erro é `passo/2`, hoje 0,5 cm), e um slider sobre um
   número sem downside é um controle que só pode ser posto no lugar errado. O
   `CORNER_LOOKAHEAD` é **margem medida** (a mutação `2.0 → 1.0` sobrevive aos
   cinco gates de comportamento) — expor margem é pior que a esconder.
5. ✅ **O alcance do CHÃO** (`float_height + cling_distance`) segue **derivado de
   propósito** e agora é **VISÍVEL**, que era o que lhe faltava.

⇒ **A metade (b) fecha sem construir knob nenhum**, e o que ela deixa é a
correção desta nota. *Quem conta os sensores conta o consumidor, não a tabela.*

#### O que ficou aberto

- ⚠️ **Um quadro PAUSADO mostra a leitura do ÚLTIMO TIQUE** — arrastar um corpo
  com o relógio parado move o desenho e deixa os sensores onde o último tique os
  leu. É a **mesma propriedade** das cruzes de contato, dos triggers e da linha
  d'água (todos escritos pelo `step`); limpar seria APAGAR os sensores no gesto
  em que o artista os quer olhar. Nomeado, não escondido.
- O sensor de **quina** desenha o vão e as células tapadas, e **não** a busca de
  escape (`CORNER_SEARCH_STEPS`) — *para onde ele decidiu empurrar* é outra
  pergunta, e ela tem resposta visível (o personagem desvia).


### 4.56 · `W-Probes2` — **O SMOKE REPROVOU AS TRÊS METADES** ✅ ⟨report do Enio⟩

> *"Não percebi claramente as hastes (Beiral) nem os Traços (parede). Ao arrastar
> o player fora do runtime, os traços não acompanham. não temos inputs para
> ajustes dos tamanhos e posições dos sensores nem a quantidade de sensores.
> Para um usuário pro não seria bom ter todos os ajustes disponíveis na UI?"*

**FECHADO em 2026-08-11.** Os três reproduzidos e medidos pela porta do produto
(cena 108 → ponte → `probe_marks`, câmera 100 px/m) **antes** de qualquer
hipótese.

#### (1) As hastes não estavam fracas — mediam ZERO ✅

| marca | tamanho na tela | alpha |
|---|---|---|
| perna | 10 × **115 px** | 1,00 |
| **quina (o vão)** | 64 × **0,0 px** | 0,22 |
| **parede** (cada) | **35 × 0,0 px** | 0,22 |
| agachar | 40 × 100 px | 0,22 |

⚠️ **`rise = 0.0000 m` nos TRÊS momentos**, inclusive subindo: ele vale
`rel_up · dt · CORNER_LOOKAHEAD`, e o `record_marks` usa `corner.map_or(0.0, …)`
— quando a lei não pergunta (94% dos quadros, pelo censo da própria wave
anterior) a altura do leque é **zero**. O que o passo 5 do smoke mandava procurar
**não existia**.

⚠️ **E o traço da parede media 35 px dos quais 20 ficavam DENTRO do corpo** — o
raio nasce no CENTRO (o `exclude_body` precisa disso) e a meia-largura é 0,2 m.

**A cura:** `ProbeRay.skin` (um FATO, dois consumidores com necessidades
opostas — o cast parte do centro, o desenho começa na BORDA; quem sabe é a porta
que lançou o raio) · o **TIQUE DE PONTA** (3 px, menor que os 5 do acerto de
propósito: um diz *até onde ele olha*, o outro *onde achou*) · alpha do `Idle`
0,22 → 0,45 e espessura 1,0 → 1,25 px.

⚠️ **Alongar o leque teria sido MENTIR** — um sensor parado olha mesmo zero para
cima. O que a ponta desenha é o **vão lateral**, o número autorado, que nunca é
zero.

#### (2) A leitura não seguia o corpo arrastado ✅

Medido: corpo movido para `x = 5.000`, perna publicada em **`x = 2.000`**.
`drive_players` só corre no ramo que DÁ PASSO.

⚠️ **E era decisão minha, escrita num gate** — a §4.55 declarou que *"um quadro
pausado mostra a leitura do último tique, a mesma propriedade das cruzes de
contato"*. **Errado:** um contato descreve um EVENTO e some com a corrida que o
produziu; o alcance de um sensor é uma propriedade do CORPO e existe com o solver
desligado — e o gesto de o afinar é **encostar o corpo na parede sem relógio**.

`preview_player_probes` re-deriva a geometria no ramo pausado **e no `hold`**, e
**não casta**: todo estado sai `Idle` porque a lei não correu. Castar responderia
uma pergunta que ninguém fez.

#### (3) Os ajustes — a §4.55 fechou (b) com a pergunta ERRADA ✅

Ela mediu que **cada NÚMERO tem row** e concluiu *"não há knob a construir"*. A
pergunta que faltava é a do Enio, e é sobre a **GEOMETRIA das amostras**:
`WALL_SAMPLES = 3`, `CORNER_SAMPLES = 65` e `CORNER_LOOKAHEAD = 2.0` eram
`const`, e as alturas do flanco saíam de `wall_offsets(half_height)` sem
parâmetro.

Quatro números novos, `PROJECT_SCHEMA` **71 → 72** (provisório):

| row | card | default | faixa |
|---|---|---|---|
| **Corner Rays** | Forgiveness | 65 | 1..257, passo 2 |
| **Corner Look-ahead** | Forgiveness | 2 | 0..8 tiques |
| **Wall Rays** | Walls | 3 | 1..257, passo 2 |
| **Wall Ray Spread** | Walls | 1,0 | 0..1 |

⚠️ **O TETO é MEDIDO e o recurso NÃO é tempo** (§0): **18 ns por raio, PLANO em
N** ⇒ 257 custam **4,55 µs = 0,027% de um quadro**. O que se esgota é a
**precisão de representação** — o passo cai a **2,5 mm**, e o solver assenta com
`normalized_allowed_linear_error` de ~**1,3 mm**. Um número, um argumento, os
dois sensores.

⚠️ **ÍMPAR, e não é cerimónia** (`odd_samples`): a amostra do meio é a âncora do
desempate do `cling` e os dois sensores são simétricos — uma contagem par ou
deixaria o meio de fora ou enviesaria um lado. Arredonda para cima.

⚠️ **Os defaults SÃO as consts de sempre** ⇒ todo player já salvo fica
byte-idêntico, e o que muda é só quem pode mexer neles.

⚠️ **`CORNER_SEARCH_STEPS` fica `const`, com motivo:** ele é a resolução da
**BUSCA de escape**, não a geometria de um sensor — o pedido é sobre *tamanhos,
posições e quantidade dos sensores*, e ele não é nenhum dos três.

**Gates:** 5 no desenho + 5 na leitura + 4 na varredura de seam; **7 mutações, 7
sangram**. ⚠️ **Duas "sobreviveram" e o defeito era do meu script de mutação** —
ele não afirmava que a substituição aplicava: *uma mutação que não entra é um
verde que não significa nada*.

**Smoke:** `PH2D_PHYSICS_SMOKE=108`, roteiro de **9** passos (o 7 é o arrasto
parado, o 8 são os quatro números novos, o 9 é o controle da tecla `B`).


### 4.57 · `W-FootFan` — **A PERNA É UM LEQUE** ✅ ⟨pergunta do Enio⟩

> *"teremos ajustes para os demais sensores? Por que a perna não poderia ter mais
> de um?"*

**FECHADO em 2026-08-11.** Duas perguntas, e a segunda **virou um bug com
número** — a resposta não saiu de opinião, saiu de uma sonda.

⚠️ **Os commits carregam o prefixo `W-Probes2`** porque este trabalho nasceu do
mesmo fio de report; a wave é distinta e mora aqui.

#### (a) O inventário — os outros sensores ✅

| sensor | tamanho | posição | quantidade |
|---|---|---|---|
| perna | `float_height` + `cling_distance` | — | ⛔ era **1**, fixo |
| flanco | `wall_reach` | `Wall Ray Spread` | `Wall Rays` |
| quina | `corner_reach` | `Corner Look-ahead` | `Corner Rays` |
| teto (agachar) | `crouch_height` | — | é uma **VARREDURA**, não um leque |

⇒ dos cinco, **um** estava fora: a perna. O teto não tem contagem por
**natureza** (a `W-ShapeCast` trocou o leque dele por um sweep do corpo, que é
uma forma e não N linhas) — e dizer *"complete-o"* seria pedir de volta a
aproximação que aquela wave removeu.

#### (b) A perna — o defeito, medido antes de qualquer linha ✅

Parado sobre uma fenda que o **corpo atravessa**:

| fenda | corpo | queda | fração do `float_height` |
|---|---|---|---|
| 0,10 m | 0,40 m | **0,411 m** | **46%** |
| 0,20 m | 0,40 m | 0,436 m | 48% |
| 0,40 m | 0,40 m | **113 m** | sai do mundo |

É a mesma doença que o **flanco** teve na W13 (um raio no meio não vê o que as
bordas do corpo veem) e a mesma cura. `RideConfig` ganha `samples` (default
**3**) + `spread`, e o `PlatformPlayer` os dois campos — `PROJECT_SCHEMA`
**72 → 73** (provisório).

⚠️ **O default MUDA a física, de propósito** — e é a única wave desta família em
que isso acontece: os quatro números da §4.56 nasceram nas consts de sempre. Aqui
o valor antigo *é* o defeito, então shipá-lo seria shipar o bug com um knob ao
lado. O `physics_ecs_c9` move-se por isso, e a atribuição é por **ablação**: com
`samples = 1` o hash volta **exatamente** ao do `main`.

⚠️ **A redução é o MAIS PRÓXIMO** (`<` estrito ⇒ o raio do meio, índice 0 do
`wall_offsets`, ganha todo empate) ⇒ **chão plano é byte-idêntico**. Não é regra
nova: é a que o `cling` já ship a no flanco — *o chão é o degrau mais alto que
qualquer parte do pé alcança*.

⚠️ **O clamp para ÍMPAR mora na LEI** (`odd_samples`), nunca no caminho de
autoria — um segundo arredondamento faria o número guardado discordar do número
castado.

⚠️ **O overlay mostra o que a lei CONSUMIU:** as respostas **por raio** viajam
dos casts da própria lei (o padrão do `WallProbe.hits`). Carimbar o veredito
reduzido no raio do meio desenharia, depois da redução, uma resposta que ninguém
usou — o vencedor pode ser um pé de fora.

**Gates:** 5. ⚠️ **O quinto nasceu de uma mutação SOBREVIVENTE:** trocar *"fica o
mais próximo"* por *"fica o último"* passava nos quatro primeiros, porque sobre
uma **FENDA** os dois pés de fora acham chão à MESMA distância — a propriedade só
é observável sobre chão **DESIGUAL**, e a fixture que a contém é um **degrau**.
6 mutações, 6 sangram.

**Smoke:** `PH2D_PHYSICS_SMOKE=109` — dois personagens iguais sobre fendas
iguais, e a única diferença é a contagem. ⚠️ **O controle está DENTRO do
quadro:** a cura é *nada acontecer*, então o vizinho de um raio só — que afunda —
é a fotografia do mundo de antes; e a terceira fenda, mais larga que o corpo,
engole os dois (a perna não é levitação).


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
~~W-Swim~~ ✅ → ~~W-ZoneForce~~ ✅ → ~~W-ShapeCast~~ ✅ → ~~W-Probes~~ ✅ → ~~W-Probes2~~ ✅ → ~~W-FootFan~~ ✅ → W-MultiJump → W-Ledge → (W-Glide?) → o ajuste
   1         2             3            4           5            6              7           8          9
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
SEQUÊNCIA leva a algum lugar) **e com uma cena de smoke de números MEDIDOS**.

⚠️ **O número da cena se CONTA no `physics_smoke.rs`, nunca numa nota.** Esta
linha dizia *"a próxima livre é a `=105`"* e a `=105` já era o mergulho — quem
pegou foi o `unreachable_patterns` do compilador, que é o gate estrutural deste
roteador (ele é um `match`, ao contrário da cadeia de `if` que a `line/Vector`
teve de gatear à mão). Hoje o máximo é **`=108`**, e o `=84` **não existe de
propósito**.

---

## Fontes da pesquisa

* [bevy_tnua — docs.rs](https://docs.rs/bevy-tnua/latest/bevy_tnua/) · [control_helpers](https://docs.rs/bevy-tnua/latest/bevy_tnua/control_helpers/index.html) — o referencial declarado deste módulo; de onde vem o *air actions counter* e a frase sobre o raio único na beirada
* [Unity-2D-Platformer-Controller (cjddmut)](https://github.com/cjddmut/Unity-2D-Platformer-Controller) — double jump, wall jump, corner grab
* [Advanced platformer movements — GDevelop](https://wiki.gdevelop.io/gdevelop5/extensions/advanced-jump/) · [Platformer behaviors](https://wiki.gdevelop.io/gdevelop5/behaviors/platformer/)
* [Double jump and Coyote Time — GDQuest](https://school.gdquest.com/courses/learn_2d_gamedev_godot_4/side_scroller_character/double_jump_and_coyote_time)
* [2D Platformer Controller — Godot Asset Library](https://godotengine.org/asset-library/asset/4696)
* [Movement feature list (gist)](https://gist.github.com/GivaldoF/3cde9c920a9a9c837734ec21a2b2eb31) — de onde vem o *glide*
