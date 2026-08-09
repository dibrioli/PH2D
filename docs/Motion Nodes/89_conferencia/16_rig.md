# 16 — RIG (6 nós) — conferência contra o mercado

> ⚠️ **Esta família está DEFERIDA por decisão do Enio (CLAUDE.md §5: rig+skinning "pro FIM de tudo"); a tabela existe para quando ela for retomada — nenhum item abaixo vira wave agora.**

**Data:** 2026-08-09 · **Agente:** família 16 do [plano 89](../89_plano_conferencia_dos_nos.md) · **Nós:** `rig.skeleton` · `rig.fk` · `rig.ik_2bone` · `rig.fabrik` · `rig.rubber_hose` · `rig.skin_deformer`
**Referências:** Rive (o runtime MIT que é a referência declarada do módulo vetorial) · Spine (Esoteric Software) · Blender · Battle Axe RubberHose / DUIK · [`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md)
**Params hoje:** **9 params em 6 nós** (4 · 0 · 2 · 1 · 1 · 1). Para comparação: **o IK do Spine sozinho tem 6 propriedades + 3 referências de osso**, e o Rive põe **`Strength` em cada um dos seus 7 tipos de constraint**.

---

## §0 — O achado que explica metade da tabela, e é uma linha

O catálogo inteiro sabe **LER** qualquer coluna por nome — `value.attribute` em modo *Custom* (`crates/ph2d-node-value-attribute/src/lib.rs`, o text param `ATTR_KEY`) e `motion.expression` (*"any **scalar column** of the input stream by name"*, doc-header). E sabe **ESCREVER** exatamente **cinco**: `motion.drive` (`labels: &["X", "Y", "Rotation", "Size", "Opacity"]`, `lib.rs:335`), que é o mesmo eixo de `oscillator`/`wiggle`/`noise`/`step`/`stagger`/`spring`.

⇒ **`parent` e `len` — as duas colunas que FAZEM de uma stream um esqueleto (doc 40 §1) — não têm ESCRITOR nenhum.**

Isso não é uma observação de estilo: é a causa mecânica de **seis** dos "inexprimíveis" desta tabela de uma vez só (comprimento por-osso · ramificação · peso por-osso · limite de junta · rigidez por-junta · stretch/compress). E é por isso que a §SUPERAR abaixo aponta para **um** nó (o escritor genérico de coluna) em vez de para seis features: a família não é magra por natureza, ela é **magra por não ter uma caneta**.

⚠️ E o contrário também é verdade e vale registrar como ⛔: **posar ângulo já é expressível hoje** — `rot` é coluna ordinária, `drive(Rotation)` a escreve, `rig.fk` a resolve. Foi a decisão **M4.N3** (doc 40) que comprou isso, e ela é o motivo de a família ter 9 params em vez de 40: metade do que as referências chamam de "propriedade de rig" aqui é um FIO.

---

## §1 — A tabela

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `rig.skeleton` | 4 (`joints`·`length`·`angle`·`root_angle`); `length` tem `ParamUnit::Length` | **comprimento POR OSSO** (Spine: cada bone tem `length` próprio — [spine-bones](http://esotericsoftware.com/spine-bones); Rive idem) — hoje **um** `length` para a corrente toda | **NÃO** — `len` não tem escritor (§0). `drive` cobre X/Y/Rotation/Size/Opacity e nada mais; `motion.expression` só **lê** colunas | **omissão** (mecanismo: falta o escritor, não falta o modelo — `len` já é coluna) | **P1** ⏸ | um `len` uniforme = a corrente de hoje, ao bit |
| `rig.skeleton` | idem | **RAMIFICAÇÃO** — todo pacote de rig tem uma ÁRVORE de ossos (Spine "bones are hierarchical"; Rive bone chains); aqui o source só emite `parent[i] = i−1`, e `is_chain()` do `fabrik`/`rubber_hose` **recusa** qualquer outra coisa | **NÃO** — a árvore é *representável* (a coluna `parent` é livre e o `fk::resolve` já a honra: ele resolve por pai, não por `i−1`), mas nada a **escreve**. `motion.combine` funde streams e não re-baseia índices de `parent` | **omissão** — e a mais barata de todas, porque o *avaliador* já suporta | **P0** ⏸ (é a diferença entre um membro e um personagem) | um `branches` vazio = a corrente única de hoje |
| `rig.skeleton` | idem | **NOMES de osso** (Spine/Rive endereçam por nome; `motion.look_at` já tem `mode`=objeto nomeado) — aqui a junta é um ÍNDICE | **NÃO** — mas o **padrão canônico existe**: text param (doc 32, `Graph::set_text_param`), o mesmo que deu a `motion.expression` sem tocar contrato | omissão | P2 ⏸ | sem nomes = índices de hoje |
| `rig.skeleton` | idem | **limites de junta** (min/max por junta — Blender *IK limits* + *IK stiffness* por osso) | **NÃO** (§0) | omissão | P2 ⏸ | limites `±180°` = livre = hoje |
| `rig.fk` | **0** (deliberado, doc 40 §3: *"um dial aqui seria uma 2ª fonte de verdade para o mesmo ângulo"*) | **HERANÇA por osso** — Spine `Transform mode`: *Normal · Only Translation · No Rotation Or Reflection · No Scale · No Scale Or Reflection* ([spine-bones](http://esotericsoftware.com/spine-bones)); Rive tem *Transform Spaces* World/Local nos constraints. É o que mantém a CABEÇA na vertical enquanto o corpo gira | **NÃO** — `fk::resolve` compõe `mundo = mundo_do_pai + rot` **incondicionalmente** | **omissão** ⚠️ **e NÃO fere a cerca do doc 40 §3**: aquela recusa é de um *dial global*; isto é uma **coluna por-junta**, que é exatamente a forma que a família já usa | **P1** ⏸ | herdar-tudo (a coluna ausente) = FK de hoje, ao bit |
| `rig.ik_2bone` | 2 (`root`·`flip`) | **STRENGTH / MIX** — Rive põe `Strength` em **todos** os 7 constraints (*"A Strength of 0% means the target does not affect the bones"*, [rive/ik](https://rive.app/docs/editor/constraints/ik-constraint)); Spine chama `Mix` (*"blends between forward kinematics (0%) and inverse kinematics (100%)"*, [spine-ik](http://esotericsoftware.com/spine-ik-constraints)); Blender `Influence`; Unity `Target Position Weight` | **SIM, a 6 nós** — `ik_out → value.attribute("rot")` · `fk_out → value.attribute("rot")` · `value.mix` · `motion.drive(Rotation, Set)` · `rig.fk`. Funciona porque a verdade é ANGULAR (doc 41 §3) ⇒ misturar ângulos é legítimo. ⚠️ **`motion.mixer` NÃO serve**: ele mistura streams (posições), e `P` misturado discorda de `rot` = a cadeia **rasgada** que o doc 41 §3 proíbe | **omissão** | **P1** ⏸ (§7 do plano: *exprimível a um custo que ninguém paga* — 6 nós por um knob) | `strength = 1` = o solve de hoje |
| `rig.ik_2bone` | idem | **POLE TARGET** como PORTA (Blender *Pole Target* + *Pole Angle*; Unity *Hint* + *Hint Weight*; Unreal *Joint Target*) — hoje o cotovelo é **um bit** (`flip`) | **NÃO** — nada computa o plano do cotovelo. ⚠️ **Não é cerca:** o doc 41 §8 lista literalmente *"e, se der vontade, `pole` como porta em vez de bit (`flip`)"* — está **aberto**, não recusado | omissão | **P1** ⏸ | porta desligada ⇒ cai no `flip` de hoje |
| `rig.ik_2bone` | idem | **SOFTNESS** (Spine, 2 ossos: *"Slows down the bones as the constrained bones straighten"*) — é o que mata o **POP do joelho** na extensão máxima | **NÃO** — o `d.clamp(…, l1+l2)` de hoje é um degrau duro exatamente ali (`lib.rs:104`) | omissão — e é a que o artista SENTE sem saber nomear | **P1** ⏸ | `softness = 0` = o clamp de hoje |
| `rig.ik_2bone` | idem | **STRETCH · COMPRESS · UNIFORM** (Spine: *"Causes the constrained bones to be scaled larger when the distance to the target is greater than the bones' lengths"*) | **NÃO** (§0: `len` sem escritor) | omissão ⚠️ **encosta numa cerca**: o doc-header diz *"It never stretches"* como decisão. Um param cujo default reproduz isso **honra** a cerca; um default diferente a revoga | P2 ⏸ | `stretch = 0`, `compress = 0` = hoje |
| `rig.ik_2bone` | idem | ⛔ **Bone Count** (Rive: *"set how far up the bone chain the IK constraint should reach"*) — ou seja, o IK de Rive é UM nó para N ossos | ⛔ **RECUSADO COM MOTIVO**: nós fatoramos em dois nós de propósito (doc 41 §1) — fechado e **EXATO** para 3 juntas, iterativo (FABRIK) para N. O `rig.fabrik` **é** o caso N | natureza | ⛔ | — |
| `rig.fabrik` | 1 (`iterations`) | **STRENGTH / MIX** (idem acima) | idem — 6 nós | omissão | **P1** ⏸ | `strength = 1` |
| `rig.fabrik` | idem | **coerência TEMPORAL** — o paper semeia da pose do frame anterior; sem isso o lado da dobra pode **inverter** ao cruzar a reta | **NÃO** (`Effect::Pure` por design; a aresta `pre` é o mecanismo, e os `sim.*`/`boids` já a usam) | ⚠️ **CERCA COM GATILHO**: doc 41 §6-bis — *"um `rig.fabrik` sequencial resolveria isso; fica pra quando alguém precisar"*. O gatilho é o smoke: um membro que pisca | **P1** ⏸ | um `mode` cujo default é `Pure` = hoje, ao bit |
| `rig.fabrik` | idem | **limites / rigidez por junta** (o próprio paper tem *FABRIK with constraints*; Blender *IK Stiffness* por osso) — é a diferença entre uma CAUDA e um macarrão | **NÃO** (§0) | omissão | **P1** ⏸ | limites `±180°`, stiffness 0 = hoje |
| `rig.fabrik` | idem | a **ORIENTAÇÃO** do alvo é ignorada (Blender *Weight Rotation*; Unity *Target Rotation Weight*) — `pose::goal` lê só o `P` do 1º elemento | **NÃO** hoje, mas **BARATO**: a stream do alvo já carrega `rot`; é ler a coluna que já está lá | omissão | P2 ⏸ | peso `0` = ignorar, como hoje |
| `rig.rubber_hose` | 1 (`flip`) | **BEND / bias da barriga** — a ferramenta que o próprio doc-header nomeia (Battle Axe **RubberHose**, AE) tem dobra e viés; DUIK idem. Aqui `α` é **resolvido** da distância e não há como pedir mais curva, nem puxar a barriga para o punho | **NÃO** — e o caminho óbvio **falha por construção**: `drive(Rotation, Add)` depois da mangueira quebra a curvatura constante, que **É** o nó (o gate `the_tip_reaches_the_goal_and_the_curvature_is_constant` reprovaria) | omissão | **P1** ⏸ (é o único dial estético do nó, e ele não existe) | `bend = 0`, `bias = 0.5` = o arco de hoje |
| `rig.rubber_hose` | idem | ⛔ **TAPER / largura ao longo da mangueira** (o controle mais usado do RubberHose) | ⛔ **EXPRESSÍVEL, cadeia curta**: `hose → value.instance_field(Ramp) → value.curve → motion.drive(Size, Set)`. A largura é a coluna `size`, e `drive` a escreve | natureza (o taper não é do solver, é do estilo do elemento) | ⛔/P2 ⏸ | — |
| `rig.rubber_hose` | idem | **squash & stretch / auto-volume** (RubberHose preserva volume esticando) | **NÃO** (§0: `len`) | omissão | P2 ⏸ | `volume = 0` = comprimento fixo de hoje |
| `rig.rubber_hose` | idem | **STRENGTH / MIX** (idem) | idem | omissão | **P1** ⏸ | `strength = 1` |
| `rig.skin_deformer` | 1 (`falloff`, `IntSlider` 1..8) + 3 portas | **PESO AUTORÁVEL** — Spine e Rive **pintam** peso por-vértice/por-osso (Rive: *Tendons*); Blender tem *envelope distance* + *envelope weight* **por osso** e Maya *dropoff rate* por influência. Aqui existe **um** expoente global | **NÃO** — `motion.falloff` mascara por ELEMENTO da pele, nunca por OSSO; não há canal de peso por-osso (§0) | **omissão** — é o item que todo rigger encontra no 1º dia | **P0** ⏸ | um envelope ausente = `1/dᶠ` de hoje |
| `rig.skin_deformer` | idem | **`falloff` CONTÍNUO** — todos os pacotes têm rigidez contínua; aqui é **inteiro 1..8** | **NÃO** — e o mecanismo está no código: `for _ in 0..falloff…{ w /= d }`, porque *"a `powf` is a transcendental (HR-5)"* (`lib.rs:173`) | **omissão com mecanismo NOMEADO** — e a cura é uma ROTA, não um param: LUT/interpolação entre os dois inteiros vizinhos, o precedente exato do [doc 24](../../Painter/24_transferencia_srgb_tabelada.md) | P2 ⏸ | um `falloff` inteiro = hoje, ao bit |
| `rig.skin_deformer` | idem | **max influences / normalize** (Maya `maxInfluences`; Blender limita a 4 no export de engine) — hoje **todo** ponto soma sobre **todos** os ossos: `O(pontos × ossos)` sem teto | **NÃO** | omissão — e é também a história de CUSTO da família (§4) | P2 ⏸ | `max = ∞` = a soma de hoje |
| `rig.skin_deformer` | idem | ⛔ **dual quaternion** (o *candy-wrapper* do LBS) | ⛔ **RECUSADO COM MOTIVO, medido no domínio**: doc 42 §2 — *"Em 2D, nos ângulos que uma mangueira ou um membro realmente atingem, não aparece"* | natureza | ⛔ | — |
| `rig.skin_deformer` | idem | ⛔ **bone heat** (a outra opção do auto-bind do Blender) | ⛔ **RECUSADO COM MECANISMO**: doc 42 §2 — bone heat resolve um Laplaciano **sobre a superfície da malha**, e aqui não há malha, só pontos | natureza | ⛔ | — |

---

## `CONSTRAINTS QUE FALTAM:`

A pergunta central da família. Rive e Spine convergiram num conjunto **pequeno e nomeado**, e é contra ele que um rig se mede — não contra a contagem de params.

| Constraint | Rive | Spine | Nós hoje | Veredito |
|---|---|---|---|---|
| **IK** | `IK Constraint` (`Bone Count`, `Invert Direction`, `Strength`) | `IK` (`Mix`, `Positive`, `Compress`, `Stretch`, `Uniform`, `Softness`) | `rig.ik_2bone` + `rig.fabrik` | ✅ **temos o SOLVER — falta a INTERFACE** (strength · pole · softness · limites). Nosso solve é *exato* onde o deles não é (lei dos cossenos sem `acos`, doc 41 §2) |
| **Distance** | `Distance` (`Distance`, `Mode` = *Closer · Further · Exactly*, `Strength`) | — | ⚠️ **`motion.pin_constraint` NÃO é isto** — ele escreve `inv_mass` (PBD), que é *massa infinita*, não *distância a um alvo* | **FALTA** · **P1** ⏸ |
| **Transform** (copia tudo) | `Transform` (`Strength`) | `Transform` (mix de *rotate/translate/scale/shear*, offsets, `Local`, `Relative`) | — | **exprimível, caro:** `attribute → (map_range/gain) → drive(canal, Set/Add)`, **um nó por canal**. ⚠️ **e só ÍNDICE-a-ÍNDICE** — "copie o osso 3 no osso 7" não é exprimível (nada re-endereça elementos) · **P1** ⏸ |
| **Translation / Rotation / Scale** (copiar + limitar) | três constraints, cada um com `Strength`, min/max, offset, *Source/Destination Space* | dentro do `Transform` | `motion.drive(X/Y/Rotation/Size)` + `value.map_range` (o clamp) | ✅ **EXPRESSÍVEL hoje** — inclusive o *Limit Rotation* do Blender. Falta o **espaço** (World/Local): `drive` escreve a coluna crua |
| **Follow Path** | `Follow Path` (`Distance`, `Strength`) | `Path` (`Position Mode` *Fixed/Percent* · `Spacing Mode` *Length/Fixed/Percent/Proportional* · `Rotate Mode` *Tangent/Chain/Chain Scale* · `Position` · `Spacing` · `Rotate Offset` · mix) | `motion.spline_wrap` deforma **pontos**; `motion.path` é uma **fonte** | ⚠️ **FALTA para OSSOS** — o Path constraint do Spine dirige uma **CADEIA** ao longo de uma curva (é o *Spline IK* do Blender), e é o rig de cauda/cabelo/espinha do mercado · **P1** ⏸ |
| **Aim / Look At** | (via Rotation constraint) | (via Transform) | **`motion.look_at` ESCREVE `rot`** (`lib.rs:358`) e tem `offset` | ⚠️ **CORREÇÃO, não feature:** ele escreve o heading de **MUNDO** numa coluna que o `rig.fk` lê como **LOCAL** (doc 40 §2) ⇒ acerta a **raiz** e **rasga** tudo abaixo dela. Derivado do código, **pendente de gate red-first** · **P1** ⏸ |
| **Physics / motion secundário** | — | **`Physics`** (`Inertia`, `Strength`, `Damping`, `Mass`, `Wind`, `Gravity`, `Mix`) — cabelo, cauda, pano | `motion.spring` (canais X/Y/**Rotation**/Size, `tension`/`friction`, `Effect::Temporal`) + `motion.pin_constraint` (`inv_mass`) | ⚠️ **PARECE expressível e NÃO É** — o `spring` **persegue um alvo no MESMO canal que escreve**, e num esqueleto o movimento do PAI não muda o `rot` **local** do filho ⇒ *a inércia — a metade que faz a coisa toda — é estruturalmente invisível para ele*. E as `force.*` escrevem `accel` para o `motion.integrate`, que move `P`, que o `rig.fk` seguinte **sobrescreve** · **P1** ⏸ |
| **Manipulator / Rig Control** | — | — | — | ⚠️ **FALTA e não é nó:** [`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) linhas 81 e 105 — *"Manipulator \| cria alças de controle no viewport \| **FALTA**"* e *"Rig Control … \| **PARCIAL (rig.\* sem UI de controle)**"*. Hoje o alvo do IK é o **primeiro elemento** de uma stream: uma convenção invisível onde as três referências têm uma **alça no canvas** · **P0** ⏸ *(é UI, não param — mas é o que torna a família usável)* |

**Duas coisas que o quadro acima diz e a contagem de params não dizia:**

1. **`Strength`/`Mix` é o buraco de MAIOR alcance da família** — Rive o põe em 7 de 7 constraints e Spine em 4 de 4; nós temos **zero**. Um item, sete lugares.
2. **Nenhuma referência tem `rubber_hose` como constraint** (a Cavalry o tem como *layer*, linha 80: *"Rubber Hose Limb … TEMOS (rig.rubber_hose!)"*) — este nó é onde **estamos à frente**, e é justamente o que está mais magro em dials.

---

## `SUPERAR:`

Derivado do que **só nós temos** (lei 8 do plano; §1 do doc 63):

1. **O `Mix` de um constraint pode ser um CAMPO ESPACIAL — e custa zero.** Em Rive/Spine o `Strength`/`Mix` é **um número keyado**. Aqui um esqueleto é uma stream ordinária (M4.N3), então o strength é uma **coluna** — e a família `field.*` inteira (`field.box`, `field.radial_sweep`, `field.remap`, com **gizmo de canvas**) já a produz. *"Um IK cuja influência desvanece com a distância de uma caixa que o artista arrasta na tela"* não existe em nenhuma das três referências, e aqui é um fio. **É o mesmo item do gap #1 acima, entregue melhor do que a referência.**
2. **Um constraint de FÍSICA que SCRUBBA.** O Physics constraint do Spine é um acumulador — nas referências ele não rebobina (reset, não scrub). Aqui o `Cook::checkpoint`/`restore` + `CheckpointRing` (doc 11) já dão scrub **bit-exato** aos `sim.*`; um `rig.physics` herda isso ⇒ **o único rig 2D com inércia que anda para trás na régua sem drift**.
3. **A ORDEM dos constraints é um FIO, não uma lista.** O próprio doc do Rive avisa que *"constraint order matters — the lower constraint overrides the one above it if both have 100% Strength"*, e essa ordem mora numa lista de inspector. Aqui solvers **compõem** porque escrevem POSE (doc 41 §3), e a ordem é o grafo: visível, re-ligável, e com o `motion.duplicator`/subgrafos aplicável a muitos membros de uma vez.
4. **A bind pose é um FIO** (doc 42 §2). Blender/Maya/Spine guardam-na como snapshot escondido, e "re-bind" é um comando destrutivo e temido. Aqui `rest` e `posed` são **duas portas** ⇒ dá para fazer bind contra um esqueleto que **também está animado**, e trocar a bind pose é arrastar um fio.
5. **Determinismo cross-OS, gateado.** Unity/Unreal resolvem IK com `acos`; o nosso é exato e transcendental-free (HR-5, doc 41 §2), e o repo já pina hash cross-OS na matriz de 3 SOs. Um rig que dá **o mesmo bit** nos três é o que torna replay-hash de gameplay possível — nenhuma referência oferece.
6. **⚠️ E o multiplicador de tudo isto é UM nó, não seis (§0):** um **escritor genérico de coluna** (`motion.set_attribute`, o gêmeo escritor do `value.attribute`, com o nome pelo text param do doc 32) destrava de uma vez: comprimento por-osso · ramificação · peso por-osso · limites de junta · stiffness · stretch. **Zero contrato tocado** (é um nó como qualquer outro), e ele serve o catálogo INTEIRO, não só o rig. *Se esta família for retomada, este é o primeiro item — e provavelmente ele não é do rig.*

---

## `CERCAS:`

Grepadas antes de propor (lei 5). Nenhuma proposta acima as revoga sem dizer.

| # | Cerca | Onde | Como a tabela a honra |
|---|---|---|---|
| 1 | **M4.N3: um esqueleto é uma stream ORDINÁRIA** — sem `Domain::Rig`, sem ADR, contrato **8/2/1** | doc 40 §1 | tudo aqui é coluna ou param; **nada** encosta em `NodeOp`/`OpResolver`/`NodeManifest` |
| 2 | **`rot` é LOCAL, `P` é DERIVADO** (a escolha KineFX de ângulo de MUNDO foi recusada com mecanismo: um modificador genérico giraria uma junta e **rasgaria** o membro) | doc 40 §2 | é o que torna a inexpressibilidade do `look_at` um **BUG**, não um pedido de feature |
| 3 | **`rig.fk` não tem params** — *"um dial aqui seria uma 2ª fonte de verdade para o mesmo ângulo"* | doc 40 §3 / `lib.rs:56` | a herança por-osso proposta é **coluna**, não dial ⇒ não fere |
| 4 | **Um solver escreve uma POSE, nunca posições** — mutante provado: escrever `P` direto deixa **16 de 17 testes VERDES** | doc 41 §3 | mata `motion.mixer` como rota de `Strength`, e é por isso que a cadeia proposta mistura **ângulos** |
| 5 | **Alvo desligado = NO-OP, jamais "a origem"** | os 3 solvers | qualquer porta nova (pole, orientação) herda a mesma regra |
| 6 | **FABRIK stateless — sem coerência temporal** ⚠️ *cerca com GATILHO*: *"fica pra quando alguém precisar"* | doc 41 §6-bis | listado como P1 **com o gatilho nomeado** (o smoke que mostrar o membro piscando) |
| 7 | **`break_collinearity`** — a degenerescência não é exótica, é o **default** aqui (o nó é `Pure` e parte da pose reta) | `fabrik/lib.rs:104-129` | ⛔ não mexer: qualquer `strength`/`pre` novo tem de rodar **depois** dela |
| 8 | **A mangueira NÃO ESTICA / o IK NÃO ESTICA** | doc-headers | `stretch`/`volume` propostos com default que reproduz hoje **ao bit** |
| 9 | **bone heat recusado** (não há malha) · **dual quaternion recusado** (2D não alcança o candy-wrapper) | doc 42 §2 | mantidos como ⛔ na tabela |
| 10 | **Os leaves `fk.rs`/`pose.rs`/`trig.rs` são BYTE-IDÊNTICOS nas 6 crates** — *"a cópia **não pode divergir**, é o contrato dela"* (com `#[allow(dead_code)]` onde uma não usa tudo) | doc 42 §4 | ⚠️ **toda mudança de leaf é × 6**; uma wave desta família que edite `fk.rs` numa crate só nasce quebrada |

---

## `O DOC 63 ERROU EM:`

1. **A §3 do doc 63 — a tabela de lacunas nó a nó, 22 nós, já priorizada — NÃO TEM UMA LINHA DE `rig.*`.** Não é um erro de conteúdo: é **silêncio**, e foi ele que a recusa em atacado do doc 88 §9 herdou. A família passou de "não pesquisada" a "recusada" sem nunca ter sido medida.
2. **§5 item 8 está ERRADO nos dois sentidos:** *"**Rig = params promovidos** (Cavalry Control Centre): o expose de subgrafo + painel 'controles do grafo' com os params marcados — **o artista rigga sem bones**."**
   - **(a)** Nós **temos** bones — seis nós, FK + IK fechado + FABRIK + hose + LBS —, e eles já existiam quando aquela linha foi escrita (a baseline do próprio doc 63 diz "87 nós", e as 6 crates `rig.*` fecharam em 2026-07-12, docs 40-42).
   - **(b)** A frase mede-nos contra a **Cavalry**, e a referência do repo diz por que essa régua é curta: *"**Rigging Duik-like:** sem esqueleto IK completo … rig = atributos promovidos, **não bones**"* ([`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) linha 232). ⇒ **medir esta família pela Cavalry mandaria REMOVER aquilo em que já a superamos.** As referências certas — Rive e Spine — o doc 63 nunca nomeia.
3. **O que do doc 63 SOBREVIVE e é o veredito certo:** o `status vs PH2D` da linha 105 da referência Cavalry — *"Rig Control / Manipulator … **PARCIAL (rig.\* sem UI de controle)**"*. **A metade que falta é a superfície de CONTROLE, não o solver** — e é exatamente o que a linha **Manipulator** do quadro de constraints diz. O item de rig do doc 63 estava certo sobre o *sintoma* (falta o Control Centre) e errado sobre a *causa* (não é porque não temos ossos).

---

## §4 — Notas menores, verificadas, que não cabiam na tabela

- ⚠️ **Um slider com um degrau inalcançável:** `rig.ik_2bone` oferece `root` até **62** (`PARAM_HINTS`, `max: 62.0`), mas `is_chain` exige `root + 2 < n` e `MAX_JOINTS = 64` ⇒ o maior `root` que pode formar uma cadeia é **61**. `root = 62` é um no-op silencioso em qualquer esqueleto possível. (Doc 88: *um teto digitável não pode passar do que o kernel HONRA*.)
- ⚠️ **Os dois tetos da família são ESCOLHIDOS, não MEDIDOS** (CLAUDE.md §0): `MAX_JOINTS = 64` (*"a chain is walked once per joint"*) e `MAX_ITERATIONS = 32`. Nenhum diz **de que recurso** é, nenhum traz tabela. E há um custo real ao lado deles que ninguém mediu: `rig.skin_deformer` é **`O(pontos × ossos)` sem teto de influências** — 64 juntas × uma nuvem de pele é a conta que decide se 64 é generoso ou tímido.
- **Zero `ParamHardMin`/`ParamHardMax`, zero seções, zero `param_gates`** na família inteira (doc 88). Com 0-4 params por nó, seções são **magras por natureza** aqui; os hard-max, não — `joints`/`iterations` são exatamente a forma de param que o slider dual do doc 88 §B2 existe para servir.
- **Unidades:** só `rig.skeleton::length` declara `ParamUnit::Length`, e está **certo** — é o único param da família que é uma distância de mundo. Os `flip`/`root`/`iterations`/`falloff`/`angle`/`root_angle` são contagens, bits e ângulos (o widget `Angle` já os veste).
