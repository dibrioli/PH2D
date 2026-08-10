# Plano — o PLAYER DE PLATAFORMA (Dynamic)

> **FASE 2 de 3.** A pesquisa é [`05_pesquisa_player_plataforma.md`](05_pesquisa_player_plataforma.md);
> este documento **decide**. Ordem do Enio (2026-08-03): *"busque o estado da arte, o padrão
> ouro ou melhor. E tome as decisões e faça as escolhas"*, com duas balizas explícitas —
> **hoje é Dynamic** (o Kinematic virá um dia, e não é este trabalho) e *"não faremos
> milagres, mas faremos o melhor possível"*.
>
> Por isso este plano não traz opções: traz **decisões, com o motivo e o preço**. Onde o
> estado da arte não tem resposta, ele diz que está inventando e por quê.

---

## §1 — As dez decisões

| # | decisão | por quê (o motivo curto) |
|---|---|---|
| **D1** | **Cápsula flutuante** sobre corpo Dynamic | apaga o caso especial de degrau/rampa/plataforma; a perna é uma mola que nós escrevemos |
| **D2** | Motor **híbrido** `{ força, boost }` | força no regime contínuo, velocidade escrita onde a exatidão É o requisito (tnua issues #34/#39) |
| **D3** | **Reação da 3ª lei** no ponto do raio, com **DOIS** escalares | é o que o tnua não tem e o Enio pediu; os dois escalares vêm do wanderlust |
| **D4** | **`LockRotation`**, sem upright spring | 2D trava; o componente já existe e é o que Unity/Godot fazem |
| **D5** | **Fita de entrada** por tick, porta irmã do `SceneAtTick` | é o que faz o player conviver com scrub/bake/c9 em vez de mutilá-los |
| **D6** | **Query pipeline** como fatia própria no wrapper | não temos nenhum; a mola é um cast por tick |
| **D7** | Lei pura em **crate leaf nova** `ph2d-platformer` | gates headless sem rapier; o precedente é `ph2d-stroke-width`/`ph2d-guides` |
| **D8** | **Corner correction PREDITIVA** (shapecast antes do step) | o estado da arte não tem em Dynamic; reativa seria inventar energia |
| **D9** | §14 **sem seletor**, chamada `Platform Player` | seletor de um item é controle morto; o ponto de extensão é o componente |
| **D10** | Forgiveness em **segundos**, medidos aqui | os pixels de Celeste são de uma tela 320×180 e não transferem |

---

## §2 — A arquitetura, em uma figura

```
 shell  ──── grava/consulta ────►  FITA (por tick)
                                      │
                                      ▼   (porta irmã do SceneAtTick)
 bridge/player.rs ──► amostra o mundo (grounded, chão, velocidade do chão)
        │                             │
        │                             ▼
        │                    ph2d-platformer  (CRATE LEAF, sem rapier)
        │                    entrada + estado + amostra ──► PlayerMotor
        │                             │                     { força, boost, reação }
        ▼                             ▼
 PhysicsWorld ◄── aplica: força no corpo · boost na velocidade · REAÇÃO no chão
```

**A lei é pura e não conhece o rapier.** É isso que torna a fita testável, o coyote
gateável sem GPU, e a wave inteira debugável sem janela — o mesmo desenho que faz o
`ph2d-physics` ser um wrapper e a `ph2d-physics-ecs` uma ponte.

---

## §3 — As waves

Cada uma fecha com: gate batched verde · mutações · **cena de smoke com números MEDIDOS**
(a política do módulo) · entrada no tracker + linha no `00_plano_waves.md`, na mesma sessão.
Cenas a partir de **`PH2D_PHYSICS_SMOKE=80`** (a próxima livre).

### W1 — O QUERY PIPELINE (infra, sem player nenhum)

O buraco que a pesquisa achou: não temos `cast_ray` nem `cast_shape`. Sem isto não há mola.

- `PhysicsWorld::cast_ray_down` / `cast_shape_down` sobre o `QueryPipeline` do rapier 0.28.
- ⚠️ **O pipeline tem de ser atualizado quando a arena muda**, e esse é o lugar óbvio para
  nascer errado (um cast contra um pipeline velho responde sobre um mundo que não existe
  mais). **Decisão:** atualização no `prepare()` do dispatch, **uma vez por dispatch**, com
  gate que prova que um corpo movido no mesmo dispatch é visto pelo cast.
- ⚠️ **Determinismo:** o resultado entra no `physics_ecs_c9` só na W7; aqui basta ser
  `BTreeMap`-ordenado e livre de transcendental fora do `libm`.
- **Filtro:** o cast **ignora o próprio corpo** do personagem (senão ele se acha no chão), e
  respeita as **camadas de colisão** (`world/layers.rs`) — um player não pisa no que ele
  atravessa.

⛔ **Smoke `=80` CORTADO (2026-08-03, no fechamento da W3).** A cena prometida —
*"três raios contra chão plano / rampa / vão, imprimindo distância e normal"* — é um TESTE,
não um smoke: o `world::cast` já tem sete gates, com os dois lados do BVH medidos, e uma cena
que imprime números pede ao Enio para conferir exatamente o que uma máquina confere melhor.
**Uma cena de smoke existe para julgar o que só o olho julga.** As cenas seguintes sobem um
número cada (a mola vira `=80`, andar vira `=81`).

### W2 — A MOLA (o personagem PAIRA)

- `float_height`, `cling_distance`, `ride_spring_strength`, `ride_spring_damping`.
- Lei (a canônica, e é a mesma das duas fontes):
  `spring = offset·k − rel_vel·d`, força para cima **+ compensação da gravidade**, medida
  **relativa ao chão** (`rel_vel = v_corpo·n − v_chão·n`).
- ⚠️ **O amortecimento é aplicado como BOOST**, e é isso que o torna independente de `dt`.
  O tnua documenta que **2.0 explode** (a velocidade inverte). **Medimos o nosso teto** e
  escrevemos o número com a tabela — não citamos o dele.
- `LockRotation` (D4) é armado junto: sem ele a cápsula tomba e a mola vira um pêndulo.

**Kill-criterion (declarado ANTES do build, DIRETIVA §5):** *se o personagem em repouso sobre
chão ESTÁTICO oscilar com amplitude > 2% da `float_height` em regime, após a 2ª rodada de
ganhos, a cápsula flutuante é o desenho errado para o nosso solver e o plano volta à mesa.*

**Smoke `=80`:** o personagem paira; empurre-o com a MÃO e ele volta à altura.

⚠️ **E a W2 deixou UM número errado que só a W3 achou:** o `float_height` do ponto de partida
(`0,5`) deixa a cápsula canônica (`half_height 0,3`, `radius 0,2`) **TANGENTE** ao chão — ela
não paira. Flutuar de verdade é geometria, medida em
[`RideConfig::min_float_height`](../../crates/ph2d-platformer/src/ride.rs):
`float_height > half_height + radius / cos(max_slope)`. O plano ficou cego a isso porque a
W2 só tinha chão PLANO, onde tangente ainda funciona. A cura de produto — semear a altura a
partir do collider, como o collider já nasce da caixa do sprite — é item da W5.

### W3 — ANDAR

- `speed`, `acceleration`, `air_acceleration`, `max_slope`.
- **Aceleração como FORÇA** (D2), com o fator de mudança de direção do tnua
  (`1.5 − 0.5·cos`, até 2× num 180°) — é o que faz frear e virar responderem.
- **Parar é `boost`**, não força: é o issue #39 do tnua, e é a diferença entre parar no
  lugar e derrapar.
- **Rampa:** acima de `max_slope` o personagem **escorrega** (a projeção do tnua), em vez de
  subir parede.
- ✅ **Plataforma móvel cai de graça**: a velocidade é medida relativa ao chão, e a *mudança*
  de velocidade dele entra como boost. Andar sobre uma plataforma kinematic (que já temos) é
  andar.

**Smoke `=81`:** anda, freia, sobe rampa de 30°, escorrega na de 60°, e cavalga a plataforma
kinematic dirigida pela timeline.

⚠️ **A W3 acrescentou o que o plano não previu: a cena precisa de CONTROLE.** A caminhada é
uma resposta a um dedo, e nada num log a mostra, então o fio do teclado
(`shells/desktop/src/player_input.rs` → `physics_bridge::hand_input_to_players`) entrou
junto. Ele **observa sem consumir** — a seta já tem dono (o nudge de nó do Vector) — e a
política é pura, porque um `winit::KeyEvent` não pode ser construído num teste.

⚠️ **E o fator de mudança de direção NÃO é o `1.5 − 0.5·cos` do tnua**, por medição e não por
gosto: num eixo 1-D aquele cosseno só assume ±1, então a fórmula vira um DEGRAU de 1× para 2×
no cruzamento do zero. O nosso é `1 + |Δv| / (2·speed)` saturado em 2 — mesmos extremos
(2× num 180°, 1× no alvo), contínuo no meio.

### W4 — PULAR

Parametrizado por **ALTURA**, nunca por força (a lição do tnua: uma força dá alturas
diferentes conforme a velocidade inicial e a posição na mola).

- `jump_height` · `takeoff_extra_gravity` + `takeoff_above_velocity` (o *"painfully slow"*)
  · `fall_extra_gravity` · `shorten_extra_gravity` (**altura variável**: soltar corta) ·
  `peak_*` (o ápice).
- ⚠️ **O ápice tem DUAS escolas e elas são opostas:** Celeste **alonga** (gravidade reduzida
  no topo, é *forgiveness*), o tnua **encurta** (`peak_prevention`, é *snappiness*).
  **Decisão: alongar** — o pedido é um platformer preciso, e o *hang* é o que dá ao jogador
  a janela para mirar. O encurtamento fica como número negativo do mesmo knob, não como um
  segundo controle.

**Smoke `=83`:** pulo cheio × pulo curto (soltar cedo), com as duas alturas MEDIDAS impressas.

### W5 — A §14 DO INSPECTOR (autoria)

⚠️ **ANTECIPADA para depois da W3** (2026-08-03), e quem mandou foi um gate: o
`every_physics_component_is_authorable` reprovou no fechamento da W3 com
*"componentes de física que NENHUM caminho da UI escreve: `PlatformPlayer`"* — ele estava
vermelho desde a W2, e está **certo**: um componente registrado sem UI funciona em toda cena
de smoke (que constrói com código) e é inalcançável no produto. O plano já dizia que esta
wave entra *"aqui, e não no fim"*; o gate apenas disse quão cedo. A W4 (pular) vem depois.

Entra aqui, e não no fim: a partir dela tudo é autorável e o Enio smoka com a mão.

- Seção **`Platform Player`** (D9), sem seletor, com slot de nota próprio.
- Componente **`PlatformPlayer`** em `ph2d-physics-ecs` — ⚠️ registro **27 → 28**, e o
  número se **CONTA** contra o `main` do dia da integração.
- ⚠️ **O estado vivo (coyote, buffer, grounded) NÃO é campo do componente** — o
  `canonicalize` do undo ordena por bytes e cada frame viraria um passo de undo (ADR-0131).
  O componente é **CONFIG**; o estado vive na ponte, ao lado do `grab`.
- As **quatro condições de fechamento de UI** do módulo valem inteiras (existe · pintado e
  registrado · o clique chega · **a sequência leva a algum lugar**), e a quarta não é
  implicada pelas outras três.

**Smoke `=84`:** um sprite pelado vira um player só com gestos da UI.

### W6 — A REAÇÃO (a jangada afunda)

O que separa este controlador do tnua.

- `ground_force.linear = −(float + jump)·opposing_force_scale − movement·opposing_movement_force_scale`
- `ground_force.angular = (ponto_do_raio − centro_de_massa) × força` ⇒ **torque**, logo a
  jangada **inclina** sem código de jangada.
- ⚠️ **Dois escalares, defaults OPOSTOS, e o motivo é de produto** (o wanderlust já
  resolveu): `opposing_force_scale = 1.0` (o peso é transmitido — sem isso o personagem é um
  fantasma que a balança não pesa) e `opposing_movement_force_scale = 0.0` (senão a
  plataforma escorrega para trás como um tapete quando você anda).

**Kill-criterion:** *o personagem sobre plataforma DINÂMICA é um oscilador acoplado (mola
sobre mola). Se após a 2ª rodada de ganhos ele divergir ou oscilar visivelmente, a reação
vira opt-in com default OFF e a feature ship sem ela* — a wave entrega o resto de qualquer
jeito.

**Smoke `=85`:** a jangada afunda e **inclina** quando o player anda para a borda; a
gangorra desce do lado dele; e um controle ao lado com a reação desligada, para o olho
comparar.

### W7 — A FITA (o determinismo)

- `trait PlayerInputAtTick { fn input(&mut self, tick: u64) -> PlayerInput; }` — **porta
  irmã do `SceneAtTick`**, mesma forma, mesmo lugar no laço de ticks devidos.
- O estado do controlador passa a ser **função pura de `(tick, fita)`** ⇒ o scrub replaya, o
  ring de checkpoints **continua válido**, e o player **NÃO** entra no `is_poking()` (não
  mutila o cache, ao contrário da MÃO).
- ⚠️ **`physics_ecs_c9` ganha um player com fita sintética** — determinística por
  construção, como toda a fixture dele. O hash **muda**, e isso é correto e declarado.
- **1º corte: a fita é runtime-only** (não serializada), como o `TimelineFlags::performing`.
  Persistí-la é wave posterior, **nomeada aqui** para não virar promessa esquecida.

**Smoke `=86`:** grave 3 s de corrida, arraste a régua para trás, e a corrida se repete
**igual** — com o hash impresso nos dois passes.

### W8 — FORGIVENESS (o que faz ser *preciso*)

> **⚠️ ESTADO (2026-08-04): a wave FECHOU — a segunda metade é a W10.**
> **Coyote time e jump buffer LANDARAM** (commit `4277be065`, smoke `=87`, 6 gates,
> 6 mutações). **Corner correction e lift momentum landaram na W10** (cenas `=89`
> e `=90`), e o desenho abaixo sobreviveu à construção com **uma correção medida**:
> a correção de quina é um **DESLOCAMENTO de posição**, não o *"boost lateral"*
> que esta linha dizia — um impulso dá o mesmo deslocamento no tique e deixa o
> personagem derivando de lado a metros por segundo depois, porque ninguém o
> remove (medido: **5,05 m** de desvio contra os 0,11 do deslocamento).
>
> Duas decisões que a construção tomou e que este plano não previa:
> - **O POUSO passou a correr ANTES da DECOLAGEM** na `jump_step`. É o que faz um
>   aperto guardado disparar NO tique em que o pé toca; com a ordem antiga ele
>   saía 16 ms depois, exatamente no gesto que o buffer existe para adiantar.
> - **Os dois relógios são CONSUMIDOS pela decolagem** — o coyote porque perdoar
>   um erro de tempo não é dar uma segunda chance, o buffer porque um aperto que
>   sobrevive à própria decolagem re-dispara no tique seguinte.


- **Coyote time** e **jump buffer** — em segundos (D10), medidos.
- **Corner correction (D8)** — ⚠️ **inventada aqui, porque o estado da arte não a tem em
  Dynamic** (varrido: tnua, wanderlust e avian não têm; a literatura só a resolve em
  kinematic com `OverlapCapsule`+`ComputePenetration`):
  > **Preditiva, nunca reativa.** Subindo (`v_y > 0`), um shapecast curto para cima diz se a
  > cabeça vai bater. Se o overlap horizontal com a quina for **≤ `corner_reach`**, aplica-se
  > um **boost lateral** no tick **anterior** ao contato, deslocando o corpo para fora da
  > quina. Reativa seria detectar o contato depois que ele comeu a velocidade vertical e
  > devolvê-la — **inventar energia**, e brigar com o solver que acabou de resolver.
- **Lift momentum**: sair de uma plataforma preserva a velocidade dela por uma janela.

**Smoke `=87`:** os dois relógios do perdão (coyote e buffer).

### W10 — A QUINA E O VAGÃO (2026-08-04)

As duas metades que o W8 nomeou e não construiu. **Nada do desenho dele foi
refutado**; três números que ele não tinha vieram da medição, e uma frase dele
estava errada.

**(A) CORNER CORRECTION.** Preditiva, como o W8 exigia: subindo, o sensor mede o
teto que a cabeça alcançaria no PRÓXIMO tique, e se um deslocamento lateral
pequeno o livra, o personagem é movido **antes** do contato. Nada é devolvido
porque nada foi tirado.

⚠️ **O W8 dizia "boost lateral" e a medição derrubou:** um impulso de `escape/dt`
dá o deslocamento certo neste tique e **sobrevive**, porque ninguém o remove — o
personagem sai voando de lado (**5,05 m** contra os 0,11 do deslocamento, com o
controle aéreo desligado). Correção de quina é assistência de **POSIÇÃO**, e é
por isso que `PlayerStep::nudge` não é um `Motor`.

⚠️ **O sensor é um PERFIL de 65 raios, e a resolução foi MEDIDA.** O primeiro
corte usava 25 e o passo saía 2,7 cm num corpo de 40 cm: **um encosto de 10 cm
não era salvo com o alcance em 12 cm**, porque a meia célula que uma amostra não
pode afirmar mais o arredondamento comiam os 2 cm de folga. Com 65 o passo cai
para 1,0 cm. **O custo não foi o que decidiu, porque ele não existe:** o sensor
inteiro mede **+0,0004 ms por tique de subida** (~8 ns por raio), e só nos tiques
em que o personagem sobe.

| encosto | pico SEM | pico COM | desvio lateral |
|---|---|---|---|
| 0,04 m | 0,784 | **0,833** | −0,052 |
| 0,10 m | 0,727 | **0,833** | −0,112 |
| 0,12 m | 0,716 | 0,716 | 0,000 |
| 0,20 m (cabeça inteira) | 0,702 | 0,702 | 0,000 |

**A última linha é a que separa a assistência de um teletransporte:** com a
cabeça inteira tapada o pico é IDÊNTICO com e sem ela.

**(B) LIFT MOMENTUM.** ⚠️ **A doença não era do solver:** o corpo sempre manteve
a velocidade da plataforma. Quem a apagava era a **assistência** — a caminhada
mira `drive × speed` *relativo ao chão*, e no ar o chão valia zero, então o
controle aéreo passava a frear o que a física dera. Medido: um pulo de um vagão a
4 m/s avançava **11% do voo balístico**.

⚠️ **E o desvanecimento foi construído e REPROVADO pela medição.** A primeira
versão desvanecia a memória linearmente na janela — mais suave, e entregava
**metade**: o alvo caía continuamente e o controle aéreo freava o tempo todo
(1,03 m contra os 2,67 do balístico). A lei que ficou **SEGURA** o valor cheio e
solta no fim da janela; o degrau não é solavanco porque o que muda ali é o ALVO,
e o controle aéreo é uma aceleração limitada.

| janela | avanço no voo | fração do balístico |
|---|---|---|
| 0,00 s | 0,291 m | **11%** |
| 0,25 s | 1,358 m | 51% |
| 0,50 s | 2,291 m | 86% |
| **0,75 s** | **2,667 m** | **100%** |

**O default é 1,5 s**, e ele é função do PULO: um pulo default de altura cheia
fica **1,45 s no ar** (medido). Em chão estático a memória é `[0, 0]` — o default
ligado é inerte até existir uma plataforma que se mova.

**Smoke `=89` (A CHAMINÉ):** um vão de 0,60 m sobre um corpo de 0,40 — a janela
em que o pulo passa sai de **±0,10 m para ±0,22 m**. **Smoke `=90` (O VAGÃO):**
pular parado numa plataforma que anda e pousar EM CIMA dela; com a janela em 0 o
vagão sai debaixo do personagem.

### W9 — O NÚMERO QUE O ARTISTA ESCREVE (2026-08-04, do smoke do Enio)

> **⚠️ Wave FORA do mapa original — ela nasceu de um smoke, e cada item dele é uma
> linha aqui.** O report foi: *"o smoke do terminal erra: pular é com a seta e não
> com espaço"* · *"Max Slope na UI aparece 45, mas o player sobe até
> aproximadamente 60 graus"* · *"não entendo vários parâmetros então não sei
> julgar"* · *"esse tanto de parâmetros juntos não fica bem; organize-os em cards
> com um título"* · *"precisamos de dicas no mouse hover"*.

**(A) A TECLA.** Três roteiros mandavam apertar **Espaço** para pular, e o
`PlayerKeys` recusa o Espaço **de propósito** (ele é o Play/Pause do transporte).
⚠️ O custo não era ler errado: quem seguia a instrução **PAUSAVA a simulação** no
instante que a cena existe para medir. O gate ata o TEXTO ao KEYMAP, e não proíbe
uma palavra — se o Espaço um dia virar tecla de pulo, a primeira asserção cai
antes e diz que a segunda tem de ser revista.

**(B) `Max Slope` passa a ser o ângulo que o personagem DE FATO sobe.** Medido
antes de tocar em código: com o limite em **45**, ele subia **50° a +4,0 m em
3 s**; o teto efetivo era ~52°.

⚠️ **A perna estava certa** — o controle prova que o número move a fronteira dela
(rampa 55°: limite 54 ⇒ `+0,17 m`, limite 56 ⇒ `+13,25 m`). Quem escalava era o
**modo-ar**: recusada a superfície, a caminhada troca o eixo da rampa pela
HORIZONTAL, e um empurrão horizontal contra uma rampa é redirecionado morro acima
pelo contato. A ablação por ENTRADA fecha a atribuição:

| rampa | `air = 20` (default) | `air = 5` | `air = 0` |
|---|---|---|---|
| 46° | **+4,375 m** | +0,041 m | −20,826 m |
| 50° | **+4,010 m** | +0,004 m | −28,873 m |
| 52° | **+3,367 m** | −0,027 m | −33,369 m |

⚠️ **O teto era função de OUTRO knob**, que é a assinatura de bug de DESIGN e não
de afinação ([[feedback_ergonomics_verdict_is_a_design_bug]]): mexer na aceleração
aérea movia, em silêncio, a inclinação máxima.

**A cura é uma TERCEIRA resposta do sensor.** *"Não é chão"* colapsava dois
estados que pedem coisas opostas — **no ar** (não há em que se apoiar) e
**encostado numa ladeira recusada** (há, e é por isso que empurrar a escala).
`Footing::{Airborne, Steep, Ground}` (`ph2d-platformer::slope`), e o termo de
CAMINHADA passa por `no_uphill`: morro acima some, morro abaixo passa inteiro.
⚠️ **Só a caminhada** — a mola já está calada e o PULO é gesto deliberado do
artista; capá-lo faria o personagem perder o salto por encostar numa ladeira.

**Depois:** 44° sobe `+12,29 m`, 46° escorrega `−20,83 m`, e a tabela de ablação
fica **PLANA** (os três `air` dão o mesmo número) — a mesma tabela que
diagnosticou a doença é a que mostra a cura.

**(B') E as rampas da cena `=81` eram INALCANÇÁVEIS a pé** (medido em
`measure_walk_scene`): as duas subiam *para longe* do chão, o personagem passava
POR BAIXO delas, caía da beirada em `x = ±10` e despencava — **`y = −162 m`** seis
segundos depois, sem ter tocado rampa nenhuma. O roteiro mandava *"vá até a
rampa"* e não havia como chegar lá. O que decide é o **SINAL da rotação**.
Corrigida (chão entre duas paredes, rampa que sobe para o lado de onde ele chega,
patamar no alto), e a rampa íngreme mudou-se para a **`=88`**, a cena do par que
cerca o limite (**40° sobe / 50° escorrega**) — 60° já era recusado mesmo com o
defeito, então a `=81` nunca foi a fixture que continha o fenômeno.

**(C) e (D) A §14 vira CINCO CARDS titulados, e todo controle ganha DICA.** Os
títulos são os cinco módulos da lei (**LEG · WALK · JUMP · FORGIVENESS ·
REACTION**), não arrumação de gosto: a primeira metade da resposta a *"o que este
número faz?"* é *a que pergunta ele pertence*. **UMA tabela, TRÊS consumidores** —
o pintor, o `populate` (que registra as dicas) e a varredura de seam.

---

### W12 — DESCER DA PLATAFORMA (2026-08-05, cena `=91`)

A plataforma jump-through existe desde a W-OneWay e ali ela é julgada por
**baixo**: a caixa sobe através dela e pousa em cima. Faltava a outra metade do
idioma — **sair dela por baixo de propósito**.

**O gesto é `down + jump`**, e não `down` sozinho: um jogador que segura baixo
enquanto anda não pode cair da plataforma sem ter pedido, e o dia em que existir
um agachar o botão já estará lá com o significado certo. É o idioma de Celeste,
Hollow Knight, Ori e Dead Cells.

⚠️ **A lei diz COMEÇAR; a ponte diz quando ACABA** — a mesma divisão do sensor de
quina da W10. Decidir que o gesto aconteceu é uma pergunta sobre a ENTRADA e
sobre o tipo do chão (e por isso `GroundSample` ganhou `one_way`: o sensor reporta
*que tipo de chão* achou); decidir que o corpo já passou é uma pergunta sobre duas
caixas envolventes, e a lei pura não tem nenhuma.

⚠️ **O fim da descida NÃO é um relógio.** *"Eu já passei?"* tem resposta exata — a
caixa do personagem está inteiramente abaixo da caixa da plataforma. Um
temporizador seria um palpite sobre ela, e erraria exatamente onde dói
(plataforma grossa, queda lenta, gravidade baixa), re-solidificando com o
personagem ainda dentro dela e cuspindo-o para fora. O §0 pede medição antes de
todo teto; aqui não é preciso medir porque não é preciso teto.

⚠️ **E a descida mira UMA plataforma, não "todas as one-way"** — é isso que faz
duas plataformas empilhadas se comportarem como o artista espera (desce da de
cima, **pousa** na de baixo). O alvo é `ColliderHandle` e não corpo: a W-Compound
deu a um corpo várias formas, e um cenário pode ser UM corpo estático com dez
plataformas.

⚠️ **O sensor tem de excluir a plataforma, e não só o solver.** Quem segura o
personagem no ar não é o solver, é a MOLA — e ela age porque o raio achou chão.
Sem a exclusão (`cast_ray_skipping`, com o `cast_ray` a delegar) o personagem
pairaria sobre exactamente aquilo que pediu para atravessar.

⚠️ **O bit viaja no corpo que CAI** (`DROP_THROUGH_BIT`, o segundo consumidor que
o doc do `ONE_WAY_BIT` previa), escrito em **todos** os colliders do corpo — só no
primeiro daria a um personagem composto um pé que atravessa e um tronco que não.
E é **por tique**, nunca no `BodyDesc`: a descida deriva da fita, então um rewind
a re-deriva.

**Caso degenerado nomeado:** um vão entre duas plataformas MENOR que o personagem
deixa a descida armada para sempre. Essa cena já está quebrada sem descida nenhuma
— e é por isso que ela não ganhou um relógio de segurança.

**7 mutações, 7 sangram.** `c9` byte-idêntico (a fita do harness não segura o
baixo, de propósito). `PROJECT_SCHEMA` intocado, registro intocado, nenhum ADR.

### W13 — AS PAREDES (2026-08-05, cena `=92`)

Escorregar por uma parede e pular dela — as duas metades da mesma pergunta
(`cling`, feita **uma vez**).

⚠️ **Uma parede é o que a PERNA já recusou.** O `footing_verdict` já classifica
toda superfície em *chão* / *íngreme demais* / *nada*, e uma parede é a do meio.
Um `wall_min_angle` seria um segundo número a discordar do `Max Slope` autorado.

⚠️ **A LEI QUE EU ESCREVI ESTAVA ERRADA, e a medição a derrubou.** O primeiro
corte do escorregamento era um TETO (*"não caia mais rápido do que isto"*),
raciocinado. O knob que ele produz é **INERTE**: medido, um personagem que
empurra contra uma parede **não cai** — desce 9 cm em um segundo. As duas causas
são do produto que já shipa: o **atrito** (`DEFAULT_FRICTION = 0,5`) contra a
normal que o controle aéreo sustenta, e a **gravidade do ÁPICE**
(`peak_gravity = 0,5`), que corta metade do peso justamente na janela em que o
colado vive — auto-reforçante. A lei que ficou **DEFINE** a velocidade: quem cai
depressa é freado até ela, quem está colado é solto até ela.

⚠️ **E o pulo de parede entregava 76% do que promete** — o jogador ainda segura a
direção da parede, o controle aéreo obedece e puxa-o de volta. É a **mesma
doença que o `lift_momentum` da W10 nomeou**, e a cura é da mesma família: calar
o controle aéreo por `wall_jump_lockout` segundos. Tabela no `measure_wall`; o
`0,2 s` sai de onde a **ALTURA** para de ser perdida (97%) — ⚠️ e **não** de um
joelho no afastamento, que **não satura**: ele cresce linear, porque com o
controle calado nada freia a horizontal. Uma frase minha morreu nessa tabela.

⚠️ **O pulo de parede mora dentro do `jump_step`**, onde o aperto já tem dono; a
parede **oferece** (`WallLaunch`), a lei do pulo **aceita**. E `takeoff: false`,
porque a 3ª lei devolve ao CHÃO o que o pé nele empurrou — este empurrou uma
parede.

⚠️ **Nasce DESLIGADA**, ao contrário de todo o resto da §14: parede é uma
CAPACIDADE, não uma correção de física. É isso que mantém o `c9` byte-idêntico.

**5 mutações, 4 sangram**; a 5ª nomeia uma **defesa em camadas** (o `drive*side`
do `cling` é inalcançável pela ponte, e quem o mata é o gate de unidade).
`PROJECT_SCHEMA` **55→56** (5 campos apendados; ⚠️ **provisório**, o valor se
CONTA). Card **WALLS** próprio na §14.

**Aberto:** o sensor lateral olha só a altura do MEIO do corpo (uma beirada que
alcance só os pés não é vista — a mesma limitação honesta da folga lateral da
W10) · ~~não há *wall grab*~~ — ⚠️ **esta nota SOBREVIVEU AO FATO: a W23
construiu-o** (`PlatformPlayer::wall_grab_stamina`, com o botão próprio que ela
previa). A previsão estava certa no formato e a nota ficou a dizer que a coisa
não existia depois de ela existir.

## §4 — O que NÃO entra (nomeado, não esquecido)

- ~~**Wall slide / wall jump**~~ — **FEITOS: a W13** (2026-08-05, cena `=92`). ⚠️ E a
  previsão desta linha estava errada num ponto que importa: eles **não são duas waves**,
  são uma. As duas metades partilham a pergunta *estou agarrado?*, e separá-las daria
  duas respostas para *o que conta como parede*.
- ~~**Dash**~~ — **FEITO: a W14** (2026-08-05, cena `=93`). ⚠️ E a previsão desta
  linha estava certa no formato (uma *action* separada) e **calada sobre o que
  custa**: o arranque não é um termo a somar ao motor, é um regime em que a
  perna, a caminhada e a gravidade **calam-se** — três silêncios que são uma
  frase só (*durante o arranque o personagem é uma velocidade*).
- ~~**Agachar**~~ — **FEITO: a W15** (2026-08-05, cena `=94`). ⚠️ **E a previsão
  desta linha estava ERRADA na metade que a mantinha fora.** Ela dizia que o
  gesto *"exige **encolher o collider**, que é a primeira coisa desta linha a
  reescrever a forma de um corpo por tique"*, e que por isso tropeçava na
  premissa que a W-Compound derrubou (*"um corpo tem exactamente um collider"*).
  **Não exige nada disso.** O personagem é uma cápsula FLUTUANTE (D1): a silhueta
  dele acima do chão vale `float_height + meia-altura do collider`, e baixar a
  perna baixa a silhueta inteira **pelo mesmo delta**, com a forma intocada —
  medido, `topo 1,602 → 1,102` para uma perna que encurtou `0,500`. É a D1 a
  apagar mais um caso especial, o mesmo que ela já fizera a degrau, rampa e
  plataforma móvel; e é também o que o `bevy-tnua` faz, pela mesma razão. A
  metade da previsão que sobreviveu é a barata: o botão de BAIXO já existia desde
  a W12, com o significado certo, então a wave **não acrescenta entrada nenhuma**.
  ⚠️ O que a construção acrescentou à lista de preços foi outro: a altura
  agachada tem um **PISO GEOMÉTRICO** (`half_height + radius`, medido em `0,50`
  nesta cápsula) abaixo do qual o corpo enterra no chão — e a lei pura **não pode
  clampá-lo**, porque ela não conhece formas, de propósito.
- ~~**Descer de plataforma one-way**~~ — **FEITA: a W12** (2026-08-05, cena `=91`). Era a
  única linha desta lista com data marcada, e a previsão sobreviveu à construção: o
  mecanismo estava todo lá, a wave é o gesto.
- ~~**Bake de um player**~~ — **FEITO: a W16** (2026-08-05, cena `=95`). ⚠️ **E a
  previsão desta linha estava certa no que exigia e CALADA sobre o que faltava:**
  ela dizia *"com a fita, assar passa a fazer sentido"*, e a fita existia desde a
  W7 — mas o **bake não a lia**. Medido antes de uma linha ser escrita: uma
  corrida gravada que leva o personagem a `x = 8,765` era assada com o canal X
  **CONSTANTE** (nenhuma track horizontal para quem andou nove metros) e, com a
  ESQUERDA segurada, como `x = −8,765` — *o espelho exacto*. O caminho sem fita
  dirige os players pelo `player_input` **RETIDO**, ou seja pelo dedo do instante
  do clique. ⚠️ **É a frase do topo do próprio `bake.rs` dita ao outro eixo** —
  *"um bake que não avança a cena simula uma cena DIFERENTE"* —, e ela era
  verdade ali há tanto tempo quanto a fita. **A contradição está escrita** (era o
  que este item pedia): assar vira o corpo `Kinematic`, e a lei do player não
  dirige massa infinita, então **depois do bake o personagem para de responder ao
  teclado** — e o roteiro da cena manda o artista tentar, para ele encontrar isso
  de propósito em vez de reportar como bug.
- ~~**Persistir a fita** (W7)~~ — **FEITO: a W17** (2026-08-05, cena `=96`). ⚠️ **E a
  previsão desta linha era uma linha só porque ela supunha que persistir fosse um campo
  de arquivo.** Não era: a fita gravava **todo tique que o relógio andasse** — medido
  pela porta do produto antes de qualquer linha, **120 de 120 nas quatro células**, sem
  player na cena e com o Physics desarmado. Isso não é uma corrida, é o relógio andando,
  e as duas consequências **só existem depois de a fita viver num arquivo**: todo projeto
  do app carregaria uma corrida de ninguém, e — porque o toggle Physics nasce
  **DESMARCADO** — abrir um projeto e *assistir* à timeline apagaria a corrida gravada,
  em silêncio. **Um artefato destruído pelo ato de olhar para ele.** Persistir exigiu
  corrigir *o que era gravado* primeiro; a tabela hoje é **0 / 0 / 120 / 0**. ⚠️ O que
  torna a wave útil é o **bake da W16**: a fita é a entrada que ele replaya, então
  reabrir um projeto e apertar Bake devolve a corrida de ontem — *o bake é o caminho
  "torne durável", a fita persistida é o caminho "mantenha editável"*. **60 s pesam
  28,1 kB**, então não há teto: o que decide o tamanho é quanto o artista jogou.
- ~~**Player Kinematic**~~ — **CHEGOU: a `W-KinMove`** (2026-08-08, cena `=101`, plano 07),
  e depois os três modos (`Snap` · `Push` · `Pure`). ⚠️ **E a previsão desta linha estava
  CERTA no ponto que decidia o desenho** — *"a lei pura é agnóstica de como o motor é
  aplicado, e é exatamente onde um segundo consumidor entraria"*: foi literalmente isso,
  o `kinematic.rs` é o segundo consumidor da mesma lei, e é por isso que a paridade entre
  modos pôde virar oráculo. O que ela **não** previa era o preço: a paridade de arrasto é
  **aproximada** (`1,149%` no pico, medido em 2026-08-10 — um corpo cinemático não tem
  sub-passo para dividir), e um SENSOR teve de passar a ver corpos cinemáticos, o que
  alcança triggers e zonas inteiros e não só a água.

---

## §5 — Os números que serão MEDIDOS (nenhum teto sem tabela — CLAUDE.md §0)

| # | número | como se mede |
|---|---|---|
| 1 | ganhos da mola (`k`, `d`) | varredura de atraso e sobressinal, no molde das duas tabelas do `GRAB_STIFFNESS` |
| 2 | **teto do amortecimento** | onde o boost inverte a velocidade (o tnua diz 2.0; medimos o nosso) |
| 3 | `float_height` mínimo útil | contra a altura do collider, e o que acontece abaixo dele |
| 4 | estabilidade sobre plataforma dinâmica | amplitude em regime — **o kill-criterion da W6** |
| 5 | `max_slope` | onde escorrega, contra o `π/4` do KCC |
| 6 | custo por tick | cast + lei, com N players, contra o HR-4 |
| 7 | coyote / buffer / `corner_reach` | em segundos e metros, pelo smoke |

---

## §6 — O que a integração vai cobrar

- **Registro `ph2d-physics-ecs` 27 → 28** (gate `registers_every_physics_component`, que o
  doc dele diz existir *"to hurt"*).
- **`PROJECT_SCHEMA`**: o componente é novo ⇒ cunha blob-key própria ⇒ **não bumpa**
  (o precedente é o `PhysicsJoint` do W3). ⚠️ Se algum campo for **apendado** a um componente
  existente, aí bumpa — e o valor se **CONTA** contra o `main` do dia, nunca se escolhe.
- **Seção §14 + slot de nota 12** — contados contra o `main` do dia.
- **Crate nova `ph2d-platformer`** (leaf) e **ids novos** da §14, anotados no handoff.
- **Contrato congelado: nenhum** (nada aqui toca `Tool`/`NodeOp`/…).
- **Dep externa nova: nenhuma.**

---

## §7 — A regra que este plano se impõe

Esta linha tem 80 waves e um padrão que se repete: **quase todo defeito foi duas coisas que
deviam concordar sobre um fato, discordando.** Os dois pontos deste plano onde isso vai
tentar acontecer, nomeados agora:

1. **A amostra do chão e a reação** — quem decide *"estou no chão"* e quem decide *"em quem
   eu empurro"* têm de ser **a mesma resposta**; se forem duas consultas, um frame vai
   flutuar sobre um corpo e empurrar outro.
2. **A fita e o estado** — o estado do controlador tem de ser **derivado** da fita, nunca
   guardado em paralelo com ela; duas cópias divergem, e a divergência só aparece no scrub,
   que é onde ninguém olha.

**Próximo passo: FASE 3 — implementar, começando pela W1.**
