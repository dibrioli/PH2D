# 89 · CONFERÊNCIA — Família 13: `sim.*` (o STACK) — 5 nós

**Data:** 2026-08-09 · **Plano-mãe:** [89_plano_conferencia_dos_nos.md](../89_plano_conferencia_dos_nos.md) §3/§4
**Nós:** `sim.collide` · `sim.lifetime` · `sim.spawn` · `sim.step` · `sim.zone`
**Status:** conferência (claims). Nada implementado, nada priorizado em definitivo (§5/§7 do plano são do Enio).

---

## §0 — O que a família é hoje (lido do `MANIFEST`, não do doc)

| nó | params | efeito | portas | lê | escreve |
|---|---|---|---|---|---|
| `sim.zone` | **0** | Temporal | `init` · `state` (feedback host) → `out` | — | seleciona `init`/`state`, tira `accel`+`falloff` |
| `sim.spawn` | 3 — `rate`(12) `scatter`(1, toggle) `seed`(1) | Temporal | `template` → `out` | playhead, `dt` | os recém-nascidos + `id` |
| `sim.step` | **1** — `damping`(1.0) | Temporal | `in` → `out` | `sim_t` `accel` `inv_mass` | `P` `vel` `age` `sim_t`; **consome** `accel` |
| `sim.lifetime` | 3 — `life`(2.0 s) `variance`(0.35) `seed`(1) | Pure | `in` → `out` | `age` `id` | `life` (0→1); compacta os mortos |
| `sim.collide` | 7 — `shape`(enum 3) `height` `center_x` `center_y` `radius` `restitution`(0.3) `friction`(0.2) | Pure | `in` → `out` | `P` `vel` | `P` `vel` |

**14 params em 5 nós.** Side-metadata registrada: `param_units` só em `sim.collide` (4 `Length`) e
`sim.lifetime` (`life` = `Seconds`). **Zero `ParamHardMax`, zero `ParamSection`, zero `param_gates`
na família inteira.**

### §0.1 — As TRÊS fronteiras do substrato que decidem quase todo item abaixo

Antes de julgar um gap eu medi **o que o grafo consegue entregar a um passo de simulação**:

1. **Modulação por-TICK de QUALQUER param** — ✅ existe ([doc 58](../58_params_dirigidos_nota_adr.md):
   `Graph::drive_param`, e os 118 nós ficaram dirigíveis sem uma linha de mudança porque todos leem
   por `EvalCtx::param`). ⚠️ **Um param dirigido é UM número por TICK, nunca por elemento.**
2. **Gate multiplicativo por-INSTÂNCIA** — ✅ existe: a coluna `falloff` (`motion.falloff`
   Circle/Rect/Linear + `field.box` rotacionado + `field.combine`/`field.remap`), consumida por
   `motion.cull` no modo Falloff (com `invert`). **Dentro da zona, `motion.cull` é uma MORTE**
   (doc-comment do `sim.zone`). ⚠️ Mas o `falloff` é **transiente**: a zona o tira do estado.
3. **Escrever `vel` a partir de um grafo de valor** — ❌ **NÃO EXISTE**, e é a fronteira mais cara
   desta família. Medido: `motion.drive` tem exatamente cinco canais —
   `labels: &["X", "Y", "Rotation", "Size", "Opacity"]` (`motion-drive/src/lib.rs:335`) — **nenhum
   deles é `vel`**. Quem escreve `vel` no repo inteiro é uma lista fechada de sete crates:
   `motion.boids`, `motion.emitter`, `motion.integrate`, `force.drag`, `force.buoyancy`,
   `sim.collide`, `sim.step` (`grep -rln '"vel"'`). E `force.drag` — o único que *escala* `vel` —
   tem só as portas `in`/`out`: o `coefficient` é **param**, logo uniforme por tick.
   ⇒ **toda operação por-elemento sobre velocidade é inexprimível hoje.**

`value.attribute` **lê** `vel` como *Speed* (a LENGTH do Vec2 — `value-attribute/src/lib.rs:113`),
então o grafo consegue **perguntar** a velocidade de cada elemento e **não consegue responder** com
uma. É essa assimetria que mata os itens de speed-limit, de spin e de variância de nascimento.

---

## §1 — A TABELA (colunas fixas do plano §3)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `sim.zone` | 0 | **SUBSTEPS / time step** — Houdini DOP Network `Substeps` ([houdini_mops §A.1](../referencia_pesquisa_houdini_mops.md), *"cada POP é um micro-solver … que o solver integra"*) · Cavalry Forge Dynamics **`Time Step`** ([cavalry:163](../referencia_pesquisa_cavalry.md)) · Niagara **Simulation Stages** (múltiplos passes nomeados por tick — [niagara §A1](../referencia_pesquisa_niagara_stardust.md)) | **NÃO.** Tentei encadear `sim.step` duas vezes no interior: o 1º termina em `out.set("sim_t", vec![playhead; n])` (`sim-step:265`), então o 2º lê `sim_t == playhead` ⇒ `dt = clamp(playhead−playhead, 0, MAX_DT) = 0` ⇒ **todo termo é multiplicado por 0: no-op exato.** Não há outro lugar para pôr um 2º passe: o cook roda o interior UMA vez por tick | **omissão** | **P1** | `substeps = 1` ⇒ o laço de hoje |
| `sim.zone` | 0 | **CICLO DE VIDA** — start/delay/duration/loop: Houdini DOP `Start Frame` · Cavalry Forge **`Start Frame`** (cavalry:163) · Niagara **Emitter State** `Life Cycle Mode` · `Loop Behavior: Once\|Multiple\|Infinite` · `Loop Duration` · `Loop Delay` · `Inactive Response` ([niagara §C.13](../referencia_pesquisa_niagara_stardust.md)) | **NÃO** para a zona: `ctx.started()` é *"eu emiti algo no tick passado?"* (`prev_outputs.contains`), **não um relógio** — nada upstream adia o 1º cook dela. ⚠️ E envolver a zona num `motion.time_remap` **está cercado** (ver CERCAS). **PARCIAL por outra via:** o caso comum (*"pare de nascer depois de N s"*) é exprimível dirigindo `sim.spawn.rate` a 0 — ver a linha `sim.spawn`/duração | **omissão** | **P2** | `start = 0`, `duration = ∞` |
| `sim.zone` | 0 | Blender Simulation Input expõe **Delta Time** e **Elapsed Time** como saídas | **SIM.** `value.time` dá o playhead; `age`/`sim_t` são colunas por-elemento que `value.attribute` lê. Sem `start`, elapsed-da-zona ≡ playhead | natureza | ⛔ | — |
| `sim.zone` | 0 | **O que a zona GUARDA é uma const** (`TRANSIENTS = ["accel","falloff"]`) — o Blender faz o artista **fiar** os itens guardados pelos sockets da zona | **N/A** — não é gap de param, é a DOBRADIÇA de que todo item novo depende (ver CERCAS: uma coluna `hit`/`died` **tem** de entrar nessa const no mesmo commit) | **natureza** (a lei *"guarda estado, não rascunho"* foi paga com um bug real) | ⛔ | — |
| `sim.step` | `damping` | **SPEED LIMIT** (clamp min/max de \|v\|) — Houdini **POP Speed Limit** ([houdini:30](../referencia_pesquisa_houdini_mops.md), *"**FALTA** (barato, alto valor)"*) · Niagara **Limit Force** `Force Limit` ([niagara §C.8](../referencia_pesquisa_niagara_stardust.md), roda DEPOIS das forças e ANTES do solve) | **NÃO — três cadeias tentadas.** (a) `value.attribute`(Speed) → `value.math` → `motion.drive`: os canais do drive são `X/Y/Rotation/Size/Opacity`, **não há `vel`** (§0.1). (b) `force.drag` escala `vel`, mas o `coefficient` é **param** (só portas `in`/`out`) ⇒ um número por tick, não pode ser função da velocidade de CADA elemento. (c) `motion.expression` esbarra no mesmo conjunto fechado de escritores de `vel` | **omissão** | **P1** | `max_speed = ∞` (ou `0` = desligado) ⇒ nenhum clamp |
| `sim.step` | `damping` | **ESTADO ANGULAR** (spin integrado) — Houdini **POP Spin** / **POP Torque** / **POP Drag Spin** ([houdini:19,24,25](../referencia_pesquisa_houdini_mops.md), os três **FALTA**) · Niagara `Rotational Drag` ([niagara §C.2](../referencia_pesquisa_niagara_stardust.md)) | **PARCIAL, e a metade exprimível é a que os artistas usam.** Tentei: `value.attribute("age")` → `value.math`(× taxa) → `motion.drive(channel = Rotation)` **FUNCIONA** (o drive escreve `rot`, `motion-drive:169-180`) ⇒ *"cada partícula gira à sua própria taxa constante"* já é exprimível. O que **não** é: arrasto angular, torque, e spin **alterado por uma colisão** (precisa de `vel` angular no estado) | omissão | **P2** | `spin = 0` ⇒ `rot` intocado |
| `sim.step` | `damping` | **`MAX_DT = 1/20` é literal HARDCODED** sem medição e sem controle — Cavalry expõe `Time Step` (cavalry:163) | **NÃO** (é uma const privada). É o mesmo assunto dos substeps: um limite que só diz *"scrub/frame perdido"* e não traz a tabela (**§0 do CLAUDE.md**) | omissão | **P2** | expor com default `0.05` ⇒ bit-idêntico |
| `sim.spawn` | `rate` `scatter` `seed` | **`Spawn Probability` 0..1** — Niagara Spawn Rate ([niagara §C.10](../referencia_pesquisa_niagara_stardust.md)) · Cavalry Particle Emitter **`Probability`** ([cavalry:166](../referencia_pesquisa_cavalry.md)) | **NÃO, e o mecanismo é o achado da linha.** Tentei dirigir `rate` por um valor aleatório: `born_in` calcula `floor(rate·t) − floor(rate·(t−dt))` **com o MESMO `rate` (o de agora) nos DOIS termos** (`sim-spawn:135-142`), então mexer em `rate` não *filtra* nascimentos — **re-deriva a história**: subir pula ids, descer faz `last < first` e o `.max(first)` emite **zero em silêncio** até o relógio alcançar | **omissão** | **P1** | `probability = 1` ⇒ todo nascimento devido nasce |
| `sim.spawn` | idem | **BURST** (N num tempo T; periódico) — Niagara **Spawn Burst Instantaneous** (`Spawn Count`·`Spawn Time`·`Probability`, §C.11) · VFXG `Single Burst`/`Periodic Burst` (§A2) · Cavalry `Duration`·`Interval` (cavalry:166) | **NÃO** — mesmo mecanismo da linha acima: um pulso no `rate` não injeta N nascimentos, ele desloca a régua de ids | omissão | **P1** | `burst_count = 0` ⇒ só o fluxo contínuo |
| `sim.spawn` | idem | **DURAÇÃO / envelope** (pare de nascer depois de N s) — Cavalry `Duration`·`Interval` | **SIM.** `value.time` → `value.step`/`value.curve` → dirige `rate`; `born_in` devolve `0..0` para `rate <= 0` (`sim-spawn:136`). O **envelope** é exprimível mesmo com o **burst** não sendo | natureza (o param dirigido é a resposta) | ⛔→P2 | — |
| `sim.spawn` | idem | **TETO DE POPULAÇÃO** — Cavalry **`Maximum Particles`** (cavalry:166) · Niagara emitter max. ⚠️ **O irmão já tem:** `motion.emitter` carrega o param `max` (`motion-emitter:140`); `sim.spawn` tem só `MAX_PER_TICK = 256`, que é por TICK | **NÃO.** Tentei `motion.cull` no modo Fraction: ele mantém `amount·n` — uma **fração**, não uma contagem —, então ele **rala a população inteira** em vez de capar, e só mataria "os mais velhos primeiro" se alguém ordenasse antes. Não é equivalente. Uma zona com spawn e sem lifetime cresce sem limite | **omissão** | **P1** | `max = 0` (= sem teto) ⇒ hoje |
| `sim.spawn` | idem | **Velocidade/direção inicial** — Niagara `Add Velocity in Cone` (§C.16) · Cavalry `Initial Direction Type (Angle/Inwards/Outwards)` + `Initial Speed` (cavalry:166) | ⛔ **ESTE VEREDITO FOI REFUTADO em 2026-08-10, pelo smoke da cena `=27`** (*"não se dividem, as filhas ficam juntas como uma só"*). Ele vale para o nascimento por **TAXA** — cada filho pega uma LINHA distinta do template, logo já nasce separado — e é **falso** para o nascimento por **PULSO**, onde `burst` filhos saem da **MESMA** linha e herdam `P` e `vel` idênticos. Esta linha foi escrita antes de a porta `pulse` existir. ⚠️ **E não era afinação: era impossibilidade** — toda força do catálogo é função da POSIÇÃO, então `curl(P)` dá a duas partículas no mesmo ponto a MESMA aceleração, e duas irmãs assim ficam bit-idênticas para sempre (medido: 150 tiques). A simetria só quebra no NASCIMENTO, pela única coisa que difere entre irmãs — o id | omissão (só no pulso) | ✅ **CONSTRUÍDO: o param `burst_speed`** do `sim.spawn` — impulso aditivo à velocidade herdada, direção sorteada da identidade da própria filha, **só nos pulse-born** (é o que o mantém fora do caminho de GPU, que o pulso já recusa por declaração) | `burst_speed = 0` ⇒ byte-idêntico |
| `sim.spawn` | idem | `rate` tem `ParamUiHint.max = 60` e **nenhum `ParamHardMax`**, enquanto o kernel honra até 256/tick (≈15 360/s a 60 fps) | **N/A** — é a lei do slider dual do [doc 88](../88_plano_parametros_nos_unidades_e_slider.md): *a faixa confortável e o teto disfuncional são dois números*, e aqui só existe um | omissão (lei de param) | **P2** | `ParamHardMax` medido; `ParamUiHint` intocado ⇒ arrasto idêntico |
| `sim.lifetime` | `life` `variance` `seed` | **EVENTO DE MORTE → spawn filho** — VFXG **`Trigger Event On Die`** → Initialize do sistema filho (§A2) · Niagara **Death Event / GPU events** (§A1) · Stardust **Aux** (§A3) · Houdini **POP Replicate** ([houdini:37](../referencia_pesquisa_houdini_mops.md), **FALTA**) · **doc 63 linha 97 = `sim.replicate`, P0** | **NÃO.** `reap` constrói a saída **só a partir de `keep`** (`sim-lifetime:123-131`): as linhas mortas são descartadas e nada a jusante as enxerga. `pulse.*` é canal **escalar por tick**, não um conjunto por elemento; `motion.combine` funde streams, mas **não existe stream de mortos** para fundir | **omissão** | ✅ **CONSTRUÍDO (2026-08-10): as saídas `died` + `pulse`** — e o achado é que **`sim.replicate` NÃO É UM NÓ**: ele é a FIAÇÃO das duas nas duas portas que o `sim.spawn` ganhou no mesmo dia (`died → template`, `pulse → pulse`). ⚠️ **São DUAS saídas para um evento só porque o SISTEMA DE TIPOS as separa** — `connects_directly` exige domínio+dim+relógio iguais, e a carga (`Instances/Vec2/Frame`) não cabe no mesmo fio que o gatilho (`Instances/Scalar/Event`); é a divisão *payload × trigger* que a referência faz, aqui verificada pelo compilador. Alinhadas por índice **por construção** (as duas saem da mesma lista `gone`). ⚠️ **E a wave arrastou DUAS correções que ela tornou load-bearing:** (a) *um recém-nascido tem idade ZERO por definição* — o `newborns` herdava `age` do template, e um filho de cadáver nasce **passado da própria vida** e morre no tique seguinte (a lei já estava escrita no `sim.step`: *a row with no `age` is newborn*); (b) *um estágio de GPU produz UM buffer* — o `eligible` não perguntava por porta de saída, e um consumidor da porta 1 receberia o buffer da porta **0** em silêncio (nunca apareceu porque o único nó de duas saídas, o `carry`, não tem kernel; a recusa mora no PLANEJADOR para o próximo nascer coberto) | porta desconectada ⇒ byte-idêntico |
| `sim.lifetime` | idem | Niagara expõe **`Particles.Lifetime`** ao lado de `NormalizedAge` (§C.14) — o span por-elemento | **SIM.** `span = age / life` via `value.attribute(age)` ÷ `value.attribute(life)` no `value.math` (indefinido em `life = 0`, i.e. no instante do nascimento) | natureza | ⛔ | — |
| `sim.lifetime` | idem | `Lifetime Mode: Direct \| Random(min,max)` — Niagara §C.14 | **SIM** — nominal ± fração cobre exatamente `[life(1−v), life(1+v)]`: mesmo conjunto, outra face | natureza | ⛔ | — |
| `sim.lifetime` | idem | **MATAR POR LUGAR** (kill volume, com invert) — Niagara/VFXG **`Kill (AABox/Sphere/Plane)`** (§A2/B) · **doc 63 linha 96 = `sim.kill_zone`, P1** | **SIM — e isto CORRIGE o doc 63.** `motion.falloff` (Circle/Rect/Linear, tem `invert`) **ou** `field.box` (caixa **rotacionada**, com gizmo de canvas) escrevem `falloff` → `motion.cull` no modo Falloff com `invert` mata por ele, e **dentro da zona um cull é uma morte definitiva**. Dois nós que já existem | natureza (composição) | ⛔→**P2** (ergonomia: 1 knob × 2 nós) | — |
| `sim.collide` | 7 | **RAIO DA PARTÍCULA** — Niagara Collision **`Radius Calculation`** (auto do sprite size + scale, [§C.17](../referencia_pesquisa_niagara_stardust.md)) · VFXG `Collide (…)` colide um raio (§A2) | **NÃO era.** Colidíamos um **PONTO** (`p[1] < height`), então um sprite de qualquer tamanho **afundava até a metade** no chão — medido, um quad 1×1 num chão em `y = −2` descansava com a borda de baixo em **−2,5**. Compensar por `height` só funciona com tamanho **uniforme**: `size` é coluna por elemento e `height` é param | **omissão** | ✅ **CONSTRUÍDO (2026-08-10): a INFLAÇÃO DE MINKOWSKI** — o ponto colide contra a mesma forma *crescida por `r`*: um termo por forma (`p.y − r` · `radius + r` · `radius − r` clampado em 0) e o `respond` **intocado**, então a resposta segue com uma implementação só. ⚠️ **De onde `r` vem é `particle_radius`, a PORTA ÚNICA** que a `eval` e o WGSL perguntam — duas respostas a *"quão grande é esta partícula?"* divergiriam no primeiro arrasto de slider, e a divergência seria um sprite repousando a alturas diferentes nos dois caminhos. Três modos (`radius_from`): **Point** (o default) · **Fixed** · **Sprite Size**. ⚠️ **O inscrito, `min(\|w\|,\|h\|)·0.5`, e a metade não é convenção** — o `sprite.wgsl` expande um quad unitário em `[-0.5, 0.5]` por `size`; inscrito e não circunscrito porque um círculo que **sai** do sprite o faz pairar com um vão que o artista vê e não corrige editando a arte, enquanto um dentro dele no pior caso deixa uma quina passar — que é o que um colisor redondo sob um sprite quadrado sempre faz (`size_scale` alcança o circunscrito). ⚠️ **`size` AUSENTE lê `[1,1]` nos dois caminhos** — `SIZE_IDENTITY`, que é *também* o `default_size` do shell, logo literalmente o quad que o renderizador desenha ali: **um número, não dois** | `radius_from = Point` ⇒ o ponto de hoje, byte-idêntico |
| `sim.collide` | 7 | **PLANO NÃO-HORIZONTAL** (parede, rampa) — Niagara `Collision Mode: **Analytical Planes**` (§C.17) · VFXG `Plane` (§A2) | **NÃO — duas cadeias tentadas, as duas falham com mecanismo.** (a) *girar o mundo → colidir → desgirar*: `motion.rotate` **não gira `P`**, ele escreve só a coluna `rot` (`motion-rotate:100-117`) — e ainda que girasse `P`, não giraria `vel`, então a reflexão seria contra o frame errado. (b) *encadear colisores*: encadear **funciona** (cada um é Pure e lê/escreve `P`/`vel`), mas todo Floor é horizontal ⇒ a cadeia constrói uma **escada**, nunca uma rampa. A normal é o literal `vec2(0,1)` | **omissão** | **P0/P1** | `angle = 0°` ⇒ a normal `(0,1)` de hoje, bit-idêntica |
| `sim.collide` | 7 | **EVENTO DE CONTATO / atributo `hit`** — Cavalry Forge **`Collision Events`** (Color/Impulse/**Sticky**/Visibility, [cavalry:163](../referencia_pesquisa_cavalry.md)) · Houdini **POP Collision Detect** (marca grupo, [houdini:39](../referencia_pesquisa_houdini_mops.md)) · Niagara collision events · **doc 63 linha 98 = `sim.collision_pulse`, P1** | **NÃO.** Nada observável muda num toque que um nó a jusante possa ler: `P` e `vel` mudam, mas mudam **todo tick** por causa do step; distinguir exigiria comparar com o tick anterior, e só a zona possui uma aresta `pre` | **omissão** | **P1** | coluna `hit` ausente ⇒ byte-idêntico (e ela **tem** de entrar em `TRANSIENTS`) |
| `sim.collide` | 7 | **COMPORTAMENTOS nomeados: die / stick / slide** — Houdini **POP Collision Behavior** ([houdini:39](../referencia_pesquisa_houdini_mops.md), *"sem os 4 comportamentos nomeados"*) · VFXG collide com **lifeloss** (§A2) | **PARCIAL.** *stick* ≈ `restitution 0 + friction 1` (a resposta zera a parte tangencial) — **meio exprimível**; *die* e *lifeloss* dependem do item anterior (não há o que observar) | omissão | **P1** (junto com o evento) | `behaviour = Bounce` ⇒ hoje |
| `sim.collide` | 7 | **Mais formas** (box, segmento, SDF, forma vetorial) — VFXG `Sphere/AABox/Cylinder/Plane/SDF` (§A2) | **PARCIAL:** encadear colisores dá a **união** das formas que existem; uma **caixa rotacionada** cai de graça assim que o Floor puder inclinar (item acima) | omissão | **P2** (depois do ângulo) | variante nova no enum ⇒ `shape` intocado |
| `sim.collide` | 7 | **`Restitution Randomness`** (por elemento) — Niagara §C.17 | **NÃO** (o kernel não faz leitura por-elemento de um jitter; `id` está lá, mas o hash não) | omissão | **P2** | `randomness = 0` ⇒ bit-idêntico |
| `sim.collide` | 7→10 | **Nenhuma `ParamSection`** num nó de 7 params — o [doc 88](../88_plano_parametros_nos_unidades_e_slider.md) deu seções a 10 nós (*a parede de sliders vira três perguntas*) | **N/A** — lei de param. O corte natural é **Forma** (`shape`/`height`/`center_*`/`radius`) × **Resposta** (`restitution`/`friction`) | omissão (lei de param) | **P2**, e a metade que MORDIA caiu de carona no P0 do raio (2026-08-10): os **quatro knobs mortos** ganharam `ParamGate` — `height` só aparece no Floor, `center_*`/`radius` só no Disc/Bowl, e os dois números de raio só no modo que os lê. Um `Height` que o kernel de um Disc nunca olha é o knob-morto que esta casa recusa; o que resta em aberto é a AGRUPAÇÃO visual, que é cosmética | seção é metadado ⇒ zero efeito no cozido |

**Contagem: 3 P0 · 7 P1 · 8 P2 · 6 REFUTADOS por composição** (⛔) — ⚠️ **e dois dos três P0 estão CONSTRUÍDOS** (o evento de morte e o raio da partícula, os dois em 2026-08-10); resta o **plano não-horizontal**, mais 2 linhas que são
constatação de desenho e não gap.

---

## `SUPERAR:`

O que só nós temos, medido: o **scrub bit-exato** (`Cook::checkpoint`/`restore` + `CheckpointRing`,
[doc 11](../11_checkpoint_restore_scrub_nota_adr.md) — save/load/advance do GGPO, **denso: um
checkpoint por tick**, sobre um cook determinista por construção: `BTreeMap` ordenado, RNG por hash,
zero transcendental) e as **colunas `Arc`** ([ADR-0138](../../architecture/decisions/0138-motion-stream-columns-are-arc-shared-clone-is-a-refcount.md):
`clone()` de um `Stream` é **refcount, não cópia**, e não existe mutação in-place de `Column`).

**1. A MORTE VIRA UMA FUNÇÃO DO TICK, não uma fila de eventos — e é isso que faz o `sim.replicate`
(o P0 acima) ser melhor aqui do que em qualquer referência.** Em toda referência o evento é
*enfileirado*: as GPU events do VFX Graph alimentam o Initialize do filho **no frame seguinte**
(latência estrutural de 1 frame), o Event Handler do Niagara lê o buffer do frame anterior, e
**nenhum deles sobrevive a um scrub do Sequencer** — você não consegue arrastar a régua de volta
por cima de uma explosão de faíscas e ver as mesmas faíscas. Aqui o conjunto dos mortos é
`entrada − saída` do `reap` **no próprio tick T**, antes do `motion.combine` a jusante: sem
latência, e **re-computável em qualquer tick** porque o cook inteiro é função pura de
`(tick, estado autorado)` com o ring denso por trás. *Ninguém consegue scrubbar um rastro de
faíscas; nós conseguiríamos — e com os mesmos ids.*

**2. O checkpoint de um passo de partículas é um REFCOUNT, não uma cópia — e é isso que torna (1)
e (3) pagáveis.** Todo cache de sim das referências é **bytes**: o `.sdcache` do Cavalry
(cavalry:163), o cache do DOP do Houdini, o bake do Blender. Uma zona é precisamente o caso em que
a maioria das colunas (`id`, `size`, `tint`, `inv_mass`) **não é tocada por um tick** — o `sim.step`
copia por referência tudo que não é `P`/`vel`/`sim_t`/`age`/`accel` (`sim-step:213-217`). Logo o
histórico DENSO de um sistema de partículas custa, nessas colunas, **zero**. Corolário que vale
como nó: um **`sim.history(n)`** — *"onde esta partícula estava n ticks atrás"* — é uma **CONSULTA
ao ring**, não um acumulador; as referências compram a mesma imagem com um ring buffer por
partícula dentro do ribbon renderer, `O(partículas × comprimento)`.

**3. A colisão que PULSA e sobrevive ao scrub.** O Cavalry tem Collision Events (Color/Impulse/
Sticky/Visibility) mas eles vivem dentro de um solver com cache e um estado **Freeze**. Aqui uma
coluna `hit` (o P1 acima) é consumida pela família **`pulse.*` que já existe**, e por ser função do
tick, *um flash de cor no impacto re-toca idêntico quando o artista volta a régua por cima dele*.
É a diferença entre um **preview** e um efeito **editável**.

**4. Os substeps são exatos, e nas referências não são.** O `dt` do `sim.step` vem de uma **coluna
por-elemento** (`sim_t`), não de um contador de frames — então um sub-passo não é *"rode o solver
de novo"*, é *"avance o relógio de cada elemento por uma fração"*, e o resultado continua sendo
função pura do playhead. Um substep no Houdini/Cavalry muda o cache e obriga a re-bakear; aqui ele
não muda o que um scrub devolve.

---

## `CERCAS:`

Grepadas antes de propor. **Cada uma delas é o preço de um item acima.**

1. **[ADR-0135](../../architecture/decisions/0135-gpu-sim-zone-is-a-conditional-passthrough-and-a-partial-claim-retreats.md) D3
   — um claim PARCIAL do laço RECUA.** Um nó no interior da zona **sem kernel de GPU** faz o plano
   proibir os `pre`-sources e re-planejar: o laço inteiro volta para o pump. ⚠️ **Isto põe uma
   etiqueta de preço em TODO item P0/P1 desta tabela:** *um nó novo no interior sem WGSL custa a
   residência de GPU da zona inteira.*
2. **ADR-0135 D1 — a zona NÃO é um kernel por-elemento** e **não** deve virar um `NodeOp` de
   "select" genérico: ela escolhe entre streams de **contagens diferentes**, uma operação de host.
   ⇒ um substep/start-frame na zona é **metadado lateral** (o padrão do `StateSelect`), nunca um
   campo do `GpuKernel` nem do manifesto.
3. **`sim.zone`: `ctx.started()` é *"eu emiti algo no tick passado?"*** — e o doc registra as **duas
   respostas erradas já pagas**: *"meu `state` está vazio?"* (matar tudo **RESSUSCITA** a cena) e
   *"uma aresta entregou valor em `state`?"* (sempre entregou). ⇒ um `start frame` **não pode** ser
   construído sobre vazio.
4. **`sim.zone`: `TRANSIENTS = ["accel","falloff"]` e a lei *"guarda ESTADO, não RASCUNHO"***, paga
   com um bug real (a máscara do `motion.falloff` cavalgou o estado de volta e **mascarou a própria
   gravidade que a fez**, e depois vazou para fora esticando a cena). ⇒ **uma coluna `hit`/`died`
   TEM de entrar nessa const no MESMO commit**, ou a flag de colisão do tick T sobrevive ao T+1 e
   todo leitor a jusante mente.
5. **`sim.zone` é `Effect::Temporal` e o comentário do manifesto diz: *"it may not run inside a
   rewritten time scope"***. ⇒ a ideia óbvia de *"simulação em câmera lenta / ao contrário
   embrulhando a zona num `motion.time_remap`"* **já está cercada** — não a proponha.
6. **`sim.spawn`: o id é `floor(rate·t)`, NUNCA um contador em coluna de estado** — deliberado, para
   que um scrub reproduza o mesmo mundo (*"um contador faria os ids dependerem da HISTÓRIA do cook;
   um scrub renumeraria o mundo"*). ⇒ nenhuma proposta pode introduzir contador de nascimento.
7. **`sim.spawn`: `MAX_PER_TICK = 256` é DECLARADO lossy, em voz alta, no único lugar onde morde.**
   Não é um bug escondido; é uma decisão com o preço escrito ao lado.
8. **`sim.spawn`: o `init` da zona fica DESLIGADO de propósito** na demo da chuva
   ([doc 49:74](../49_nascimento_na_zona_nota_adr.md)) — a população começa em nada e é
   *inteiramente nascida*.
9. **`sim.step`: `motion.integrate` dentro de uma zona daria à sim DUAS memórias** (a da zona e o
   `pre` do integrador) e elas discordariam no instante em que um kill removesse uma linha.
   Exclusão deliberada.
10. **`sim.collide`: o early-out `vn >= 0` é o guarda ANTI-JITTER** (*o clássico zumbido do corpo
    parado que ganha energia do próprio teste de contato*) e **restituição ≤ 1 é guarda de
    ENERGIA**. ⇒ não "conserte" nenhum dos dois.
11. **`sim.collide` × `motion.collide` NÃO são segunda porta** — e a checagem era obrigatória
    (§ do briefing). São perguntas diferentes e o próprio doc-comment do `sim.collide` declara a
    divisão: `motion.collide` é **push-apart instância-contra-instância** (PBD, `Effect::Pure`,
    *"knows nothing about a floor, and nothing about velocity"*), `sim.collide` é
    **instância-contra-forma-do-mundo** e **reflete `vel`**. ⚠️ **O ACHADO é o NOME:** os dois
    aparecem no Add menu como **"Collide"** (`motion.collide`) e **"Collider"** (`sim.collide`) —
    uma letra de distância, e o artista não tem como saber qual é qual antes de ligar. *Isto é um
    item de produto (rótulo), não de motor* — sugestão: "Push Apart" × "World Collider".

---

## `O DOC 63 ERROU EM:`

1. **Linha 96 — `sim.kill_zone` marcado `P1` (FALTA).** **Existe hoje por composição:**
   `motion.falloff` (ou `field.box`, que ainda tem rotação e gizmo de canvas) → `motion.cull` no
   modo Falloff com `invert`, **dentro da zona** (onde um cull é uma morte). ⇒ **P2 de ergonomia**,
   não capacidade ausente. *Item marcado FALTA que já existe manda construir o que está
   construído.*
2. **Linha 207 — `sim.spawn (rate·scatter·seed) | burst · probability (espelha o emitter)`.** A
   LISTA está certa e a DIFICULDADE está errada: o emitter ganhou P2 porque *"o `pulse.*` já dirige
   o `rate`"*, e no `sim.spawn` esse truque **não funciona** — `born_in` usa o rate de AGORA nos
   dois floors, então mexer em `rate` re-deriva a história de nascimentos em vez de filtrá-la
   (§1). ⇒ **P1**, e o doc 63 deve **re-conferir a nota do emitter pelo mesmo mecanismo**.
3. **Tabela B, linha "Collision (planos/depth/SDF/scene) … TEMOS (`sim.collide`; variedade de
   proxies FALTA)".** Subestima: o que falta primeiro **não é variedade de proxy** — é o **RAIO**
   (colidimos um ponto, e o sprite afunda até a metade) e a **ORIENTAÇÃO do plano** (o nosso chão
   não inclina). Os dois aparecem na primeira cena, antes de qualquer proxy exótico.
4. **Tabela B, linha "Kill (volume box/sphere/plane, invert) … PARCIAL (lifetime/collide existem;
   kill-volume dedicado?)".** Medido: é **TEMOS por composição** (mesmo item 1).
5. **Linha 335 (D3) agrupa `sim.collision_pulse` + `sim.kill_zone` + `motion.spawn_per_unit` como
   UM item.** Hoje são **três vereditos diferentes**: P1 (inexprimível) · P2 (exprimível) · P1
   (inexprimível). Agrupá-los faz a wave errada nascer.
6. **Confirmado, não errado:** a linha 30 do `houdini_mops` (**POP Speed Limit — FALTA**) segue
   verdadeira, e esta conferência acrescenta o **mecanismo**: nenhum nó do repo escreve `vel` a
   partir de um grafo de valor (§0.1), o que é a razão pela qual ela não é exprimível.

---

## `ESTÁGIOS QUE FALTAM:`

O stack do Niagara é `System Spawn → System Update → Emitter Spawn → Emitter Update →
Particle Spawn → Particle Update → **Events/Simulation Stages** → Render`
([niagara §A1](../referencia_pesquisa_niagara_stardust.md)); o do VFX Graph é
`Spawn → Initialize → Update → Output` (§A2). Mapeando contra o nosso, com o mesmo teste de
expressibilidade:

| estágio da referência | nós | veredito |
|---|---|---|
| **Spawn** (quantos nascem) | `sim.spawn` (+ `motion.emitter` stateless) | ✅ **TEMOS** |
| **Initialize / Particle Spawn** (roda UMA vez, no nascimento) | o trecho **entre `sim.spawn` e `motion.combine`** | ✅ **TEMOS — e como LUGAR, não como estágio.** Tudo fiado ali roda exatamente uma vez por partícula, porque a saída do spawn **é** o conjunto dos recém-nascidos. É uma resposta melhor que um módulo `Initialize Particle` com N checkboxes (niagara §C.14): compõe com a biblioteca inteira. ⛔ refutado |
| **Update / Particle Update** (todo tick) | o interior da zona | ✅ **TEMOS** |
| **EVENTS / Event Handler** (`Spawned \| Every Particle`) | — | ❌ **FALTA — e é o estágio inteiro.** São exatamente os dois P0/P1 da tabela: `sim.lifetime` não publica os mortos, `sim.collide` não publica os contatos. Sem ele não há trilha-de-faísca, não há respingo, não há filho. **Inexprimível** (o `pulse.*` é escalar por tick; não existe stream de mortos) |
| **System/Emitter Update — o CICLO DE VIDA** (`Loop Behavior/Duration/Delay`, `Inactive Response`) | — | ❌ **FALTA** (a linha `sim.zone`/ciclo de vida). **PARCIALMENTE exprimível**: o envelope de nascimento sai dirigindo `rate` a 0; o *start/delay da zona* e o *reset* não |
| **Simulation Stages** (N passes nomeados por tick) | — | ❌ **FALTA** = a generalização dos **substeps**. **Inexprimível**, provado: encadear `sim.step` é no-op exato (`sim_t` já vale `playhead`) |
| **Render / Output** | `motion.output` + o lowering para instâncias | ✅ **TEMOS** |

**Placar: 4 estágios cobertos, 2 ausentes (EVENTS · CICLO DE VIDA) e 1 sub-estágio ausente
(multi-pass/substep).** Os dois ausentes são o mesmo item da tabela visto de cima — e o EVENTS é
onde o §8 do plano manda procurar o SUPERAR, porque é o único lugar em que o nosso scrub bit-exato
resolve um problema que as referências têm e não conseguem resolver.
