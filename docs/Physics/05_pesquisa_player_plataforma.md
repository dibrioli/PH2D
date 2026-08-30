# Pesquisa — o PLAYER DE PLATAFORMA sobre um corpo Dynamic

> **FASE 1 de 3** (pesquisa → plano → implementação), missão do Enio de 2026-08-03.
> Entregável desta fase: este documento + a síntese reportada. Nada de código antes do plano.
>
> **Fontes lidas na FONTE, não em documentação de terceiro** (o handoff §5 exige):
>
> | fonte | versão / commit | onde |
> |---|---|---|
> | `rapier2d` `KinematicCharacterController` | **0.28.0** (a nossa dep) | `~/.cargo/registry/.../rapier2d-0.28.0/src/control/character_controller.rs` |
> | `bevy-tnua` | **0.32.0**, `61e5dbd` (2026-07-23) | clone raso no scratchpad |
> | cápsula flutuante (re-criação com código do *Very Very Valet*) | `joebinns/stylised-character-controller` `33e3a60` | idem |
> | `avian` `dynamic_character_2d` | `main` | fetch do exemplo |
> | Celeste / TowerFall | os dois artigos de Maddy Thorson | fetch |
>
> ⚠️ O `bevy_mod_wanderlust` foi **conferido e descartado como referência primária**: ele
> **não suporta 2D**, e é literalmente por isso que o `bevy-tnua` nasceu. O que ele tem de
> aproveitável já está no tnua, que é 2D+3D.

---

## §1 — A pergunta que decide a wave

O Enio pediu duas coisas que a literatura trata como **opostas**:

> *"Todas as features que fazem dos games de plataforma tão precisos, mas sem perder a
> interação com os objetos físicos."*

A precisão de um platformer vem de controladores que **não são física** (Celeste é aritmética
inteira sobre AABBs); a interação vem de ser um corpo do solver. A tese desta pesquisa é que
**a oposição é falsa porque as duas coisas vivem em camadas diferentes**, e que quase toda a
literatura de *feel* é sobre a camada de cima:

```
    INTENÇÃO          ← coyote · buffer · altura variável · corner correction · apex
       ↓                (isto é Celeste, e é PORTÁVEL: é sobre a ENTRADA)
    LEI DE MOVIMENTO  ← como a intenção vira número no corpo
       ↓                (isto é a escolha (a)/(b)/(c) do handoff §4.1)
    RESOLUÇÃO         ← quem decide onde o corpo pára
                        (isto é o solver, e é onde a interação mora)
```

O que **não** é portável de Celeste é a camada de baixo — e é só ela.

---

## §2 — O espaço de soluções, medido na fonte

### 2.1 `KinematicCharacterController` do rapier — o catálogo, não a solução

Não vamos usá-lo (o Enio pediu Dynamic explicitamente), mas ele **enumera o problema**, e
é por isso que o handoff manda lê-lo. Os campos do `struct`
(`character_controller.rs:120-152`) são a lista do que um personagem tem de resolver:

| campo | default | o que resolve |
|---|---|---|
| `up` | `+Y` | onde é o chão |
| `offset` | `Relative(0.01)` | folga contra o mundo (**não pode ser zero** — estabilidade numérica) |
| `slide` | `true` | escorregar em vez de parar na parede |
| `autostep` | **`None`** | subir degrau sem pular — ⚠️ *"currently a very computationally expensive feature"* |
| `max_slope_climb_angle` | `π/4` | rampa que dá para subir |
| `min_slope_slide_angle` | `π/4` | rampa em que escorrega sozinho |
| `snap_to_ground` | `Relative(0.2)` | não decolar na quebra de rampa descendo |
| `normal_nudge_factor` | `1e-4` | anti-emperramento no deslize |

E a saída (`EffectiveCharacterMovement`) é `{ translation, grounded, is_sliding_down_slope }`.

⚠️ **Duas coisas dele que valem mesmo indo para Dynamic:** o `offset` que nunca é zero, e o
fato de o laço de deslize ser **iterativo com teto** (`max_iters = 20`,
`character_controller.rs:~240`). São propriedades da geometria, não do método.

⚠️ **E o que o torna inaplicável não é gosto:** um KCC move um corpo **kinematic**, que tem
massa infinita. Esta linha já mediu a consequência exata na **W-BakeJoint** — *um joint do
rapier não move um corpo kinematic* — e é o mesmo fato que faz a **MÃO** recusar corpos
não-dinâmicos (`world/grab.rs:254`). Um player kinematic **não pode pendurar-se numa corda**,
que é metade do pedido.

### 2.2 `avian::dynamic_character_2d` — a família (a) pura

Escreve `LinearVelocity` direto:

```rust
linear_velocity.x += direction * movement_acceleration * delta_time;
linear_velocity.y = jump_impulse;                    // no pulo
linear_velocity.x *= 1.0 / (1.0 + damping * delta_time);
```

Grounded por `ShapeCaster` para baixo com teste de ângulo da normal.

⚠️ **É o desenho que o pedido do Enio recusa, e o mecanismo é preciso:** escrever a
velocidade **substitui** o que o solver acabou de calcular. Um joint que puxa o personagem
produz uma velocidade que a linha seguinte sobrescreve — *o cabo vira decoração*, exatamente
como o handoff §4.1 previu. Fica aqui como o **controle** da comparação, não como candidato.

### 2.3 `bevy-tnua` — o híbrido flutuante (a referência mais próxima)

**A ideia central:** o personagem **não encosta no chão**; ele paira a `float_height` sobre
um sensor, e a **mola é a perna**. Isso apaga o caso especial de degrau, rampa e plataforma
móvel — não há contato pé-chão para negociar.

**A lei da mola** (`src/builtins/walk.rs:624-645`), na íntegra:

```rust
spring_force        = spring_offset * spring_strength;      // aceleração
gravity_compensation = -gravity;                            // a mola CANCELA a gravidade
dampening_boost      = relative_velocity * spring_dampening;
TnuaVelChange {
    acceleration: up * spring_force + gravity_compensation, // vira FORÇA
    boost:        up * -dampening_boost,                    // vira VELOCIDADE
}
```

⚠️ **A fronteira com o rapier é o achado mais importante desta pesquisa**
(`rapier2d/src/lib.rs:416-427`):

```rust
velocity.linear  += motor.lin.boost;                              // escrita DIRETA
external_force.force = motor.lin.acceleration * mass;             // FORÇA real
velocity.angular += motor.ang.boost.z;
external_force.torque = motor.ang.acceleration.z * principal_inertia;
```

Ou seja: **o tnua não escolheu entre (a) e (b) — ele usa os dois, por motivo declarado**, e
os motivos estão nos comentários do código, cada um amarrado a um issue:

- *"When stopping, prefer a **boost** to be able to reach a precise stop (issue #39)"* —
  a **precisão** exige escrita de velocidade;
- *"When accelerating, prefer an **acceleration** because the physics backends treat it
  better (issue #34)"* — o **regime contínuo** exige força.

⚠️ **E o `boost` é o que torna o amortecimento independente de `dt`:** ele remove
`relative_velocity * spring_dampening` da velocidade **de uma vez**, então `dampening = 1.0`
amortece perfeitamente num passo. É por isso que a doc do campo diz *"as this approaches
**2.0**, the character starts to shake violently and eventually get launched upward"* — em
2.0 a velocidade **inverte** (`v → −v`) e acima disso amplifica. **Não é um número de gosto:
é o limite de estabilidade de um amortecimento aplicado como impulso.**

**Defaults medidos** (`walk.rs:152-168`): `spring_strength 400` · `spring_dampening 1.2` ·
`cling_distance 1.0` · `acceleration 60` · `air_acceleration 20` · `coyote_time 0.15` ·
`free_fall_extra_gravity 60` · `max_slope π/2`.

**Plataforma móvel sai de graça** e é elegante (`walk.rs:240-309`): a velocidade é medida
**relativa à entidade pisada** (`effective_velocity = velocity − sensor.entity_linvel`), e a
*mudança* de velocidade da plataforma entra como `impulse_to_offset` — um boost. Andar sobre
um vagão é andar, e o vagão acelerando não derruba ninguém.

⚠️ **O DEFEITO do tnua para o nosso pedido, verificado por busca no fonte inteiro: ele
NUNCA aplica a reação ao chão.** Não há `apply_impulse` no corpo pisado em lugar nenhum —
`standing_on` guarda a entidade só para **ler** a velocidade dela. Consequência exata: **um
personagem do tnua sobre uma jangada dinâmica não a afunda**, e sobre uma gangorra não a
inclina. O Enio pediu literalmente essa cena.

**O `jump` do tnua é o catálogo de feel já em forma de parâmetro**
(`src/builtins/jump.rs:48-135`): `height` (uma ALTURA, não uma força) · `takeoff_extra_gravity`
+ `takeoff_above_velocity` (*"without this, jumps feel painfully slow"*) · `fall_extra_gravity`
· `shorten_extra_gravity` (**altura variável** — soltar o botão corta) · `peak_prevention_*`
· `input_buffer_time` (**jump buffer**) · `upslope_extra_gravity`. O **coyote** mora no
*basis*, não no jump (`walk.rs:100`, `airborne_timer`).

### 2.4 A cápsula flutuante canônica — a que fecha a 3ª lei

A técnica do *Very Very Valet*; o código legível é a re-criação do joebinns
(`PhysicsBasedCharacterController.cs:245-271`):

```csharp
relVel     = dot(rayDir, vel) - dot(rayDir, otherVel);   // RELATIVA ao chão
currHeight = rayHit.distance - _rideHeight;
springForce = (currHeight * _rideSpringStrength) - (relVel * _rideSpringDamper);
maintainHeightForce = -_gravitationalForce + springForce * Vector3.down;

_rb.AddForce(maintainHeightForce);
hitBody.AddForceAtPosition(-maintainHeightForce, rayHit.point);   // ← A REAÇÃO
```

⚠️ **A última linha é a diferença inteira**, e ela tem duas propriedades que importam:

1. é aplicada **no PONTO do raio** (`AddForceAtPosition`), logo **produz torque** — a
   jangada **inclina** quando o personagem anda para a borda, sem nenhum código de jangada;
2. a força devolvida inclui o `−gravitationalForce`, então **o peso do personagem é
   transmitido ao chão**. Sem isso o personagem é um fantasma que a balança não pesa.

É a mesma lei do tnua **mais** a terceira lei de Newton. E é gratuita: o número já foi
computado; devolvê-lo é uma chamada.

### 2.5 Celeste / TowerFall — o feel, e a fronteira do que é portável

O modelo (artigo *Celeste and TowerFall Physics*): **Actor** e **Solid**, com três regras —
*todos os colliders são AABB* · *todas as posições/larguras/alturas são INTEIRAS* · *Actors e
Solids nunca se sobrepõem*. O movimento é `MoveX`/`MoveY` **um pixel por vez**, com o resto
fracionário acumulado num `remainder`. Solids que se movem **carregam** (`IsRiding`) ou
**empurram** (`Squish`), e *"pushing takes priority over carrying"*.

⛔ **NADA disso é portável para nós, e o motivo é estrutural:** é um **segundo solver**, e
um que proíbe rotação, sobreposição e coordenadas contínuas — os três presentes em toda cena
desta linha (a W-Compound tem corpos de várias formas, os joints giram, as zonas usam
recorte de polígono). Adotá-lo seria [[feedback_two_engines_one_state_is_worse_than_a_slow_engine]].

✅ **O que É portável é a lista de *forgiveness*** (artigo *Celeste & Forgiveness*), porque
ela é toda sobre a **intenção**: coyote time · jump buffering (*"jump on the exact frame that
you land"*) · **corner correction** de pulo e dash · **gravidade reduzida no ápice** ·
*semi-solid popping* · **lift momentum storage** (janela de alguns frames guardando a
velocidade da plataforma) · *wall jump* tolerado a **2 px** da parede (5 px no super).

⚠️ **Os números de Celeste são em PIXELS numa tela de 320×180** e **não transferem** — o
nosso mundo é métrico e contínuo. O que transfere é *que a janela existe*, e a largura dela
é medição nossa.

---

## §3 — A recomendação

**Cápsula flutuante sobre corpo Dynamic, com a reação da 3ª lei, e o feel numa camada de
intenção separada.** Isto é a família **(b)** do handoff §4.1 — mas com uma correção ao
enunciado dela, que a leitura do tnua produziu:

> ⚠️ **(b) não é "força pura".** O controlador que funciona é **híbrido por dentro**, e a
> escolha entre escrever velocidade e aplicar força é **por-termo, com critério declarado**:
> força para o regime contínuo (mola, caminhada), velocidade para o que precisa ser
> **exato** (parar, amortecer, herdar a velocidade de uma plataforma). O `TnuaVelChange
> { acceleration, boost }` é essa distinção reificada, e vale a pena copiar a *forma*, não
> só a ideia.

**Por que ela, e não as outras** — o critério é o pedido do Enio, não elegância:

| | precisão | pendura em joint | afunda a jangada | empurra caixote | atravessa parede |
|---|---|---|---|---|---|
| KCC kinematic | alta | **não** (massa ∞) | não | só com código extra | não |
| (a) escrever velocidade | alta | **não** (sobrescreve) | não | fraco | pode |
| tnua (flutuante, sem reação) | alta | **sim** | **não** | sim | não |
| **flutuante + reação** | alta | **sim** | **sim** | sim | não |
| Celeste | máxima | não | não | não | não |

A última linha é a única que fecha as cinco colunas, e a diferença dela para a penúltima é
**uma chamada de função**.

---

## §4 — As três perguntas do handoff, respondidas

### 4.1 — O que torna um Dynamic preciso?

**Três coisas, e nenhuma é "mais força":**

1. **Não encostar no chão.** A imprecisão de um Dynamic vem da negociação de contato, que o
   artista não controla. Um personagem que **paira** não tem contato pé-chão para negociar:
   a perna é uma mola que *nós* escrevemos, com ganhos que *nós* escolhemos. Degrau, rampa e
   plataforma móvel deixam de ser casos especiais.
2. **Escrever velocidade onde a exatidão é o requisito.** Parar exatamente, amortecer
   exatamente, herdar a velocidade da plataforma exatamente. O tnua paga isso com `boost` e
   documenta os dois issues que o obrigaram.
3. **Separar a INTENÇÃO da execução.** Coyote e buffer não são física: são a resposta à
   pergunta *"o jogador quis pular?"*, e ela é respondida **antes** de qualquer força existir.

### 4.2 — O estado por-tick, e o scrub ⚠️ (a que podia custar a wave)

O controlador carrega estado entre ticks (coyote, buffer, `is_grounded`, jump-held,
dash-cooldown). Ele **não está no rapier** e **não está no ring de checkpoints**
(`bridge.rs:575`), então um scrub o perderia e a re-simulação divergiria.

**As três saídas que o handoff nomeia, com o preço de cada uma:**

| saída | preço | veredito |
|---|---|---|
| entra no **checkpoint** | o ring clona 8 tipos do rapier; teria de clonar também o nosso estado. `STRIDE=10` já é **medido** (um checkpoint ≈ um step) e o estado do player é ~dezenas de bytes contra os KB de uma arena | **viável e barato** |
| é **derivável** de `(tick, entrada)` | exige gravar a ENTRADA — e isso já é obrigatório pela §4.3 | **a espinha** |
| **não-rebobinável**, re-baseliza em silêncio (o `discard_contact_history`) | o artista perde a corrida ao scrubbar; contradiz o bake e o c9 | **recusado** |

✅ **A resposta é a segunda, com a primeira como aceleração:** grave a **fita de entrada**
(um valor por tick), e o estado do controlador passa a ser **função pura de `(tick, fita)`**.
Aí o scrub replaya a fita e é bit-exato; o ring continua válido; e o checkpoint pode carregar
o estado do player só para *pular* o replay, nunca para defini-lo.

⚠️ **Corolário que precisa estar no plano:** o estado do controlador **não pode virar campo
de componente autorado**. O `canonicalize` do undo ordena por BYTES do componente, então
estado vivo ali faz **cada frame virar um passo de undo** — a lei que o ADR-0131 já escreve e
que a §7 do handoff repete.

### 4.3 — A entrada é do JOGADOR, e teclado não é reproduzível

**O precedente exato já existe nesta linha, e resolve o problema inteiro.**

Quando o `Kinematic` quebrou o invariante *"o mundo é função de `(tick, repouso)`"*, a
auditoria do W4b não o abandonou — ela **acrescentou um termo** e criou uma porta para
perguntá-lo (`bridge/kinematic.rs:39`):

```rust
pub trait SceneAtTick {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool;
}
```

...com a doc dizendo *"a física não aprende nada sobre timelines: a pergunta é só «ponha a
cena no estado que ela tem no tick T»"*.

✅ **Um player faz a MESMA pergunta com outro sujeito:** *"que entrada o jogador deu no tick
T?"*. Se a resposta vier de uma **fita gravada**, o invariante volta a valer como
`f(tick, repouso, curvas, fita)`, e as três consequências caem juntas:

- **scrub** = replay da fita (é o que todo netcode determinístico faz, e o ring GGPO desta
  linha já é meio caminho — o handoff §4.3 já apontava isso);
- **`physics_ecs_c9`** *pode* incluir o comportamento, com uma **fita sintética** escrita no
  harness (determinística por construção, como toda a fixture dele);
- **bake** passa a fazer sentido: assar um player é assar *uma corrida gravada*, que é
  reprodutível — ⚠️ com a contradição já medida pela W-BakeJoint (o `Kinematic` do bake não é
  movido por joint) valendo aqui igual, e por isso o bake de um player tem de ser decidido no
  plano, não assumido.

⚠️ **E a fita é o que diferencia isto da MÃO.** A mão é a única entrada não-reproduzível do
módulo hoje, e por isso ela **descarta o ring** e **não grava enquanto existe**
(`bridge/grab.rs:89-92`, `bridge.rs:575`, via `is_poking()`). Um player que gravasse fita
**não** precisa dessa mutilação — e um player que *não* gravasse teria de herdá-la, o que
mataria o scrub numa cena de jogo. **É a fita que decide se a feature convive com a timeline.**

---

## §5 — A matéria-prima, e o buraco que a pesquisa achou

O handoff §3 lista o que já existe (cápsula, lock rotation, offset, one-way, sensores,
eventos de contato, sinal, camadas, gravity scale, damping, massa, dominance, material, CCD,
plataforma kinematic, zonas, joints, corpo composto, a mão, IK/FK, bake, params keyframáveis).
Tudo confirmado. Dois detalhes que valem para o plano:

- ✅ **O `isGrounded` clássico JÁ está construído e curado.** A **W-PartSensor** (cena `=71`)
  existe *precisamente* para o sensor de pé — o doc dela cita *"o `isGrounded` de Box2D e
  Unity"* e mede o defeito que o tornava morto (`bridge/triggers.rs:14-35`).
- ✅ **O one-way já deriva a direção do frame do collider** (`world/oneway.rs:17-34`), então
  plataforma rotacionada funciona; *descer* de uma delas é uma feature a desenhar sobre ele.

⛔ **O QUE FALTA, e não estava no inventário do handoff:**

> **O nosso `PhysicsWorld` não tem query pipeline.** Não há `cast_ray`, não há `cast_shape`,
> não há `QueryPipeline` em lugar nenhum de `crates/ph2d-physics/src/` (varrido). A
> `explode` encontra corpos **iterando o body set** e medindo distância
> (`world/blast.rs:138-147`), e o `queries.rs` tem uma função só (`waterlines`).

Isso é material porque **a cápsula flutuante é um raycast** (ou shapecast) por tick, por
personagem. O `rapier2d 0.35` traz `QueryPipeline` com `cast_ray` e `cast_shape`
(`src/pipeline/query_pipeline.rs:167,332`), então é integração e não invenção — mas é
**infraestrutura nova no wrapper**, com um custo de manutenção próprio (o pipeline precisa
ser atualizado quando a arena muda) e um lugar óbvio para nascer errado. **O plano tem de
orçá-la como uma fatia própria, não como um detalhe do controlador.**

> ⚠️⚠️ **RE-VERIFICAR — a FORMA da API do `QueryPipeline` mudou entre a `0.28` e a `0.35`.** Este
> levantamento foi feito lendo o source da **0.28**; a casa subiu para a **0.35** em 2026-08-29
> ([ADR-0168](../architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md)),
> e no caminho a `rapier` trocou `nalgebra` por `glam` (via `glamx 0.3`) — o `parry2d 0.30` **apagou**
> `Point`, `Isometry` e `Translation`, e o vocabulário da casa é hoje `Pose`/`Rotation`/`Vector`
> ([`rmath.rs`](../../crates/ph2d-physics/src/rmath.rs)). **A capacidade continua lá; as assinaturas e
> os números de linha citados acima, não necessariamente.** Confira o source da 0.35 antes de orçar
> esta fatia.

---

## §6 — A §14 (a seção de Comportamentos)

O ponto de fiação existe e é único: `paint_physics_sections` em
`crates/ph2d-panel-inspector/src/paint_frame.rs`, hoje com §11 Physics Body · §12 Physics
Joint · §13 Pulley Wheel, cada uma com slot de nota (9, 10, 11).

⚠️ **A tensão que o handoff §6 nomeia é real e tem uma saída que não é framework vazio:**
o Enio pediu uma seção de *Comportamentos* cujo primeiro comportamento é Plataforma, e um
seletor com uma opção só é um controle morto — a lei que esta linha aplica há 80 waves.

**Recomendação:** a §14 nasce **sem seletor**, chamando-se o que ela é (*Platform Player*),
com o ponto de extensão sendo o **componente**, não a UI. Um segundo comportamento
acrescenta a sua própria seção ou promove o seletor **quando existir** — e a promoção é
barata porque as quatro condições de fechamento de UI do módulo já obrigam cada row a ser
pintada, registrada, clicada e a levar a algum lugar. *Um seletor de um item é uma promessa
de framework; uma seção nomeada é a feature.*

⚠️ Registrar no plano: número da seção e slot de nota se **CONTAM** contra o `main` do dia
da integração ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]), e o registro do
`ph2d-physics-ecs` idem — hoje **27**, pinado no gate `registers_every_physics_component`
(`crates/ph2d-physics-ecs/src/lib.rs:145`), cujo doc diz *"this count exists to hurt"*.

---

## §7 — O que a FASE 2 tem de MEDIR antes de escrever qualquer número

O §0 do CLAUDE.md proíbe escrever teto sem medição. Estes são os números que o plano precisa
produzir, cada um com a tabela ao lado:

1. **Os ganhos da mola** (`ride_spring_strength`, `damper`): a varredura de atraso e
   sobressinal, no molde exato das duas tabelas do `GRAB_STIFFNESS` — que já provaram que
   este tipo de tabela é o que separa afinação de folclore.
2. **O teto do amortecimento**, se for aplicado como boost: o tnua diz que 2.0 explode.
   **Medir o nosso**, não citar o dele.
3. **`float_height` vs. a altura do collider**, e o que acontece quando ela é pequena demais
   (o tnua avisa que o controlador simplesmente não funciona).
4. **A reação no chão**: quanto uma jangada afunda / inclina, e se o par
   personagem-sobre-plataforma-dinâmica é estável (uma mola de cada lado é um oscilador
   acoplado — **este é o candidato mais provável a kill-criterion**).
5. **`max_slope`**, medindo onde o personagem escorrega, contra o `π/4` do KCC.
6. **O custo por tick** do raycast + do controlador, contra o HR-4, com N personagens.
7. **As janelas de forgiveness** (coyote, buffer) **em segundos**, porque os pixels de
   Celeste não transferem.

⚠️ **E um kill-criterion tem de ser declarado ANTES do build** (DIRETIVA §5): a
recomendação é sobre o item 4 — *se o personagem sobre plataforma dinâmica oscilar ou
afundar de forma instável após a 2ª tentativa de ganhos, a reação vira opt-in e a feature
ship sem ela.*

---

## §8 — Duas notas de higiene

1. **O registro do `ph2d-physics-ecs` é 27** — conferido no código, não em doc: o gate
   `registers_every_physics_component` (`crates/ph2d-physics-ecs/src/lib.rs:145`) o pina.
   ⚠️ Havia uma divergência **aparente** (o handoff de integração de 02/08 diz `24→26`); ela
   é histórica — o 26 é o estado *daquela* jornada, e as duas waves de 03/08 o levaram a 27.
   Registrado aqui porque a próxima LLM faria a mesma dupla-leitura.
2. **O `physics_ecs_c9` e o player**: o harness hoje é uma fixture de corpos caindo com um
   representante por feature. Se o player entrar, entra com **fita sintética**; se não
   entrar, o plano tem de **dizer por quê** (o handoff §4.3 pede a justificativa explícita).

⚠️ **E uma lição de método desta sessão, porque ela quase envenenou o §5:** três buscas por
`QueryPipeline` rodaram com a cwd escorregada para o clone de referência no scratchpad e
voltaram **vazias** — o falso negativo perfeito, sobre a afirmação mais cara do documento. A
conclusão só entrou aqui depois de repetida **com `cd` explícito e com controle positivo**
(`git grep -c PhysicsWorld -- world/grab.rs` → 8, e só então o alvo → 0). É
[[feedback_a_negative_search_needs_a_positive_control]] e a regra da cwd do handoff §0, as
duas na mesma armadilha.

---

## §9 — Veredito da FASE 1

**Recomendo:** cápsula flutuante sobre Dynamic, mola com reação da 3ª lei no ponto do raio,
motor híbrido `{ força, boost }` com critério por-termo, camada de intenção separada
carregando o catálogo de forgiveness, **entrada gravada em fita** consultada por uma porta
irmã do `SceneAtTick`, e um query pipeline novo no wrapper como fatia própria.

**As três perguntas do handoff estão respondidas**, e a §4.3 é a que muda o desenho: a fita
não é um detalhe de determinismo — **é o que decide se um player convive com a timeline deste
editor ou se ele mutila o scrub como a mão mutila.**

**Aguardando o veredito do Enio para entrar na FASE 2 (o plano detalhado).**
