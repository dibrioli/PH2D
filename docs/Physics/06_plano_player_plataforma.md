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

**Smoke `=80`:** três raios contra chão plano / rampa / vão, imprimindo distância e normal.

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

**Smoke `=81`:** o personagem paira; empurre-o com a MÃO e ele volta à altura.

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

**Smoke `=82`:** anda, freia, sobe rampa de 30°, escorrega na de 60°, e cavalga a plataforma
kinematic dirigida pela timeline.

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

**Smoke `=87`:** pula raspando a quina e **passa**; e o controle com `corner_reach = 0`, onde
bate e cai — para o olho ver que a assistência está agindo.

---

## §4 — O que NÃO entra (nomeado, não esquecido)

- **Wall slide / wall jump / dash / agachar** — cada um é uma wave própria; o tnua os tem
  como *actions* separadas e a nossa lei suporta o mesmo formato. Fora do 1º corte.
- **Descer de plataforma one-way** — o mecanismo existe (`world/oneway.rs`); a *feature* é o
  gesto, e é uma wave curta depois da W8.
- **Bake de um player** — ⚠️ a W-BakeJoint já mediu a contradição (um `Kinematic` do bake
  não é movido por joint). Com a fita, assar passa a fazer sentido; sem ela, não. Fica para
  depois da W7, **com a contradição escrita**.
- **Persistir a fita** (W7).
- **Player Kinematic** — o Enio disse que virá um dia. Este plano não o proíbe: a lei pura
  da `ph2d-platformer` é agnóstica de como o motor é aplicado, e é exatamente onde um
  segundo consumidor entraria.

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
