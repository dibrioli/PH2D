# 89 · CONFERÊNCIA — Família 3: **SIMULAÇÃO** (6 nós)

**Data:** 2026-08-09 · **Linha:** `line/motion-value` · **Escopo:** `motion.boids` · `motion.verlet_rope` ·
`motion.soft_body` · `motion.collide` · `motion.spring` · `motion.pin_constraint`
**Método:** §3 do [plano 89](../89_plano_conferencia_dos_nos.md). Params lidos do `MANIFEST` e do
`register_*` de cada crate (não do doc). Referência: os dumps do repo
([`referencia_pesquisa_houdini_mops.md`](../referencia_pesquisa_houdini_mops.md) ·
[`referencia_pesquisa_niagara_stardust.md`](../referencia_pesquisa_niagara_stardust.md) ·
[`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md)) + páginas oficiais citadas por URL
onde o repo está vazio.
**Status:** claims. A §5 do plano 89 é a verificação, e ela é do Enio.

---

## §0 — O que os seis nós têm hoje (lido do código)

| nó | params (`MANIFEST`) | lê colunas | escreve colunas | hard max | unidades | seções |
|---|---|---|---|---|---|---|
| `motion.boids` | 9 — `count·seed·radius·separation·alignment·cohesion·seek·max_speed·spread` | — (só o próprio `state`) | `P·vel·sim_t` | `count 2 000` | **nenhuma** | 3 (Flocking/Steering/Spawn) |
| `motion.verlet_rope` | 6 — `count·length·gravity·iterations·damping·pin_tail` | — | `P·rope_prev·sim_t` | `count 50 000` · `damping 0,5` | `length` = Length | — |
| `motion.soft_body` | 8 — `rows·cols·spacing·gravity·stiffness·stretch·damping·pin` | — | `P·sb_vel·sim_t` | `rows/cols 512` | `spacing` = Length | — |
| `motion.collide` | 3 — `radius·iterations·strength` (+ porta `spread`) | `inv_mass` | `P` | **nenhum** | `radius` = Length | — |
| `motion.spring` | 3 — `channel·tension·friction` | `falloff·inv_mass·id` | canal + `spring_value·spring_vel·sim_t` | **nenhum** | **nenhuma** | — |
| `motion.pin_constraint` | 3 — `first·count·strength` | `falloff` | `inv_mass` | `first/count 1 000 000` | **nenhuma** | — |

⚠️ **Os três GERADORES (`boids`/`verlet_rope`/`soft_body`) não declaram `Coupling` nenhum** — não consomem
`accel`, nem `falloff`, nem `inv_mass`. Os dois SOLVERS de stream (`collide`/`spring`) consomem `inv_mass`;
só o `spring` consome `falloff`. Conferido por grep em `register_couplings`.

---

## §1 — A TABELA

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.boids` | 9, `count` hard max **2 000** | contagens de bando de 10⁵–10⁶ (Niagara *Simulation Stages* GPU, [Key Concepts](https://dev.epicgames.com/documentation/unreal-engine/key-concepts-in-niagara-effects-for-unreal-engine); Houdini POP Interact: *"O(n²)/**grade**"*, dump houdini §B) | **N/A — não é gap de param, é o TETO ERRADO.** O teto de 2 000 saiu do `measure_the_count_ceiling`, que cozinha pelo `Cook` do registry = **caminho CPU O(N²)**; o nó tem `register_grid` + `gpu::GPU_KERNEL` e o kernel WGSL faz varredura 3×3 de células (**O(N)**, ADR-0140). ⚠️ **O próprio nó se contradiz:** o doc-comment de `spread` diz *"is how the count reaches the **millions** the GPU can do"* na mesma crate cujo hard max diz 2 000 | **omissão** — *o caminho mais lento definiu o teto do mais rápido* (CLAUDE.md §0.0, o caso GPU/M5 literal) | ✅ **FECHADO (`f9b17c80e`, já no `main`): o teto é 2²⁰ = 1.048.576**, com a tabela MEDIDA no doc-comment do `PARAM_HARD_MAX` (65.536 → 0,624 ms/4% no device contra 56,633 ms/340% na CPU; 2²⁰ → 14,283 ms/86%). ⚠️ **O número não foi escolhido por folga: é o que o `PH2D_GPU_COOK_DEMO=7` de fato SHIPA**, e o teto antigo tornava o próprio demo **indigitável** — o documento continha um valor que a caixa de texto recusava. ⚠️ E o preço da outra coluna ficou NOMEADO em vez de escondido: um milhão com `spread` DESLIGADO pede ~256× a célula de 65 k (a semeadura é uma caixa fixa, o enxame cai em meia dúzia de células, a varredura 3×3 visita ~todo mundo ⇒ **o device também é O(N²)**) — o kernel **honra** o pedido, o que o mantém do lado certo da lei *um teto digitável não pode passar do que o kernel honra*. ⚠️ **O slider fica em 500 por RESOLUÇÃO, não por custo** (o device faz 8.000 em 13% de um quadro): é o par slider/chip do doc 88 B2 | N/A (o teto sobe; nenhuma arte muda) |
| `motion.boids` | idem | **`max_force`** — clamp da ACELERAÇÃO de steering (Reynolds, *Steering Behaviors For Autonomous Characters*, GDC 1999, [red3d.com/cwr/steer](https://www.red3d.com/cwr/steer/gdc99/); Houdini POP Steer: *"Solver **clampa aceleração máx**"*, dump houdini §B) | **NÃO.** O clamp vive entre `accel` e a integração, dentro do `step`; nada fora do nó vê `accel` (boids emite só `P`/`vel`/`sim_t`). Inserir um nó na cadeia de `state` vê a velocidade **já** integrada — que é o `max_speed` que já existe | **omissão** — o modelo canônico tem DOIS clamps e nós temos um | ✅ **FECHADO (`2ec7dc552`, já no `main`): o param `max_force`**, o orçamento de steering, clampando a MAGNITUDE de `accel` entre a soma dos três pesos e a integração (`lib.rs:330`) — dentro do `step`, que é o único lugar de onde `accel` é visível. ⚠️ **`0` é o neutro e significa *sem orçamento***, não *força zero*: o guard é `if p.max_force > 0.0`, então o default deixa a soma de hoje **bit a bit**. ⚠️ E a aceleração EXTERNA fica deliberadamente FORA do clamp — `max_force` é quanto o BANDO pode querer, não um teto sobre o que o mundo lhe faz | `max_force = 0` (sem orçamento) ⇒ a soma de hoje, bit a bit |
| `motion.boids` | idem | **ângulo de visão** do vizinho (Houdini POP Steer Separate/Cohesion/Align: *"raio **+ ângulo de visão**"*, dump houdini §B; Reynolds: vizinhança = distância **e** ângulo) | **NÃO.** A seleção de vizinho é o laço interno (CPU) / a varredura de células (GPU); nenhum nó a alcança | **omissão** — um boid que enxerga atrás de si não vira bando, vira nuvem | **P1** | `fov = 360°` ⇒ o disco de hoje |
| `motion.boids` | idem | **wander** (POP Steer Wander: freq/força, dump houdini §B; Reynolds §wander) | **PARCIAL — a cadeia É autorável e sai INERTE.** `force.curl` → `boids.state` é legal: `motion_bridge_plumbing.rs` faz de **qualquer** porta `state`/`forces` do tipo do output 0 um *feedback host* e plumba `out --pre--> head`. `force.curl` é `Pure`, passa `P`/`vel`/`sim_t` adiante ⇒ o relógio sobrevive. **Mas boids não lê `accel`** (zero `Coupling`), então a força é escrita e descartada — exatamente o caso que o ADR-0155 diagnostica | **omissão** — falta UMA leitura de coluna, não um kernel | **P1** (vira P2 com o SUPERAR §2.1) | `wander = 0` |
| `motion.boids` | idem | **avoid obstacle / lookahead** (POP Steer Avoid Obstacle, dump houdini §B) | **NÃO** — mesmo mecanismo do wander, mais a ausência de qualquer geometria de obstáculo alcançável de dentro | **omissão** | **P1** | `lookahead = 0` |
| `motion.boids` | idem | **`min_speed`** exposto (POP Speed Limit: *"Min Speed (0) · Max Speed"*, dump houdini §B) | **NÃO** — hoje é a const `MIN_SPEED_FRAC = 0.2` **escondida no arquivo**; o artista não a alcança | **omissão** — o número existe e não tem porta | **P1** | `min_speed_frac = 0.2` ⇒ bit-idêntico |
| `motion.boids` | idem | **`neighbours` como coluna** (MOPs *Neighbors*, dump mops §A.4; Houdini POP Proximity, dump houdini §A.1) | **NÃO** por composição — **mas o número JÁ É COMPUTADO** e jogado fora nos dois caminhos (`var neighbours = 0u` no `gpu.rs:130`; `inv_n` no laço da CPU) | **omissão** — emitir custa uma coluna | **P1** (é SUPERAR §2.3) | ausente ⇒ ninguém lê ⇒ byte-idêntico |
| `motion.boids` | idem | 1 regra = 1 NÓ empilhável com peso próprio (família POP Steer, dump houdini §A.1/§B) | **NÃO** (não há primitiva de vizinhança exposta: `value.reduce`/`median`/`percentile` reduzem o stream INTEIRO, não uma vizinhança) — **mas os 3 pesos já são params dirigíveis** (doc 58), que é o ganho de mixar animadamente | **natureza** — o monolito entrega o gesto; a decomposição entrega extensibilidade | **P2** | — |
| `motion.boids` | idem | `radius` sem `ParamUnit::Length`; `radius`/`max_speed` sem `ParamHardMax` (lei do doc 88 A/B2) | **N/A** | **omissão** — `radius` é distância de MUNDO e é o único nó da família sem NENHUMA unidade declarada | ✅ **FECHADO (2026-08-12), e os mecanismos são TRÊS — incluindo um param que não existia quando esta célula foi escrita.** **`radius` = `ParamUnit::Length`**, seguindo a convenção que o irmão `motion.collide` já traz escrita (*só coordenada ou distância de MUNDO se declara; peso, fração, taxa e contagem ficam nus, porque uma unidade errada é pior que uma ausente*) — e por essa mesma regra **`max_speed` e `max_force` ficam NUS de propósito**: são velocidade e aceleração, o `ParamUnit` **não tem variante para nenhuma das duas**, e rotulá-los `Length` ensinaria ao artista algo falso. **`max_speed` = 1e20**, a parede de **REPRESENTAÇÃO** — a MESMA que o `motion.verlet_rope` encontrou em `gravity`/`length`, porque o que estoura não é o param e sim a **POSIÇÃO em `f32`** (vivo em 1e20, `inf` em 1e21). ⚠️ **E a fronteira ÚTIL fica muito antes e é uma RAZÃO:** o bando sobrevive enquanto o passo por tique não passa do próprio raio de percepção (`passo/raio` 1,0 → bando; 50 → espalhado por 307× o raio) — o teto **não** é esse ponto, de propósito, porque ele é `radius / dt`, **função de outro param**, e o pior caso daria perto de zero; um bando espalhado é **visível e reversível**, e o que o teto torna inalcançável é o `inf`. **`max_force` = 90**, e aqui o teto é a **INÉRCIA** (a lei do `iterations` do collide): é um CLAMP, então acima da maior magnitude que o steering pode ter ele **não faz mais nada** — medido com os quatro pesos no MÁXIMO do slider (são eles que decidem essa magnitude), `90` ainda morde e **`95` / `100` / `1e4` / `1e21` saem byte a byte iguais entre si E iguais a `max_force = 0`, que é o DESLIGADO**. ⚠️ **E o `radius` NÃO ganhou teto, por medição:** acima da extensão do bando ele **SATURA** (todo agente vira vizinho de todo agente — resposta cara e **correta**), o espalhamento absoluto trava em 4,9407 e nada morre nem em 1e21; gate afirma a **AUSÊNCIA**, senão a próxima varredura completa a tabela | declaração é metadata ⇒ bit-idêntico |
| `motion.boids` | idem | *arrive* (desaceleração na chegada, Reynolds) | ⛔ **REFUTADO** — o `seek` é mola **linear** (`accel += (target − p)·seek`), então a força já cai com a distância: isso **é** o *arrive* | natureza | ⛔ | — |
| `motion.verlet_rope` | 6 | **gravidade VETOR** (Niagara Gravity Force `vec3` default `(0,0,−980)`, dump niagara §C.1; Cavalry Forge world `Gravity`, dump cavalry §Forge) | **SIM — pela CADEIA DE FORÇA, e isto foi MEDIDO.** O mecanismo que esta linha nomeava (*"o nó não lê `accel`, e a cadeia `force.wind → rope.state` é autorável mas INERTE"*) foi removido pelo **W1-A**; pelo §0 (*quem move o número que tornava algo inalcançável reconfere a nota*) o item foi re-perguntado ao produto: **`gravity = 0` + `force.wind(270°)` reproduz a gravidade embutida** dentro de 1e-4 da queda — a diferença que resta é a ASSOCIAÇÃO IEEE (`(g·dt)·dt` contra `a·(dt·dt)`), não o modelo. Um `dir` por nó seria a SEGUNDA porta para a mesma pergunta, contra a semântica Houdini que o módulo inteiro segue | **DISSOLVIDO no W1-A** | ~~P0~~ | gate `vector_gravity_is_the_force_chain_and_needs_no_second_door` + o controle irmão; 3 gates sangram se o mecanismo regredir |
| `motion.verlet_rope` | 7 | **bend stiffness** (Houdini Vellum Constraints: *Stretch Stiffness* **+ *Bend Stiffness*** + *Thickness* + *Plasticity*, [vellumconstraints](https://www.sidefx.com/docs/houdini/nodes/dop/vellumconstraints.html); Cavalry Forge Collision Shape **Chain**, dump cavalry §Forge) | **NÃO** — é uma restrição i↔i+2 dentro da relaxação; nada de fora acrescenta constraint | **FEITO no W1** — a irmã `i↔i+2` no MESMO passe de relaxação, repouso `2·seg_rest` | ✅ | `bend = 0` ⇒ a corrente de hoje, ao bit (early-out: `x + (−0,0)` devolve `x`, mas `(−0,0) + 0,0` devolve `+0,0`) |
| `motion.verlet_rope` | 6 | **espessura / colisão** (Vellum `Thickness`; Cavalry Forge `Chain`; Stardust *Path-follow com colisão*, dump niagara §A3) | **PROVAVELMENTE SIM, e ninguém sabe.** `rope.out --pre--> motion.collide.in` + `motion.collide.out → rope.state` é legal (feedback host genérico), e o bloqueio conhecido **não se aplica**: `motion.collide` é `Effect::Pure` ⇒ **não carimba `sim_t`** e repassa `rope_prev`. O Verlet lê o deslocamento como velocidade — que é como toda colisão PBD entra num Verlet. ⚠️ **Não rodei; é o teste decisivo mais barato desta conferência** | **omissão** se falhar; **ergonomia** se funcionar | **P1**→**P2** | — |
| `motion.verlet_rope` | 6 | **sub-steps** (Cavalry Forge `Time Step`, dump cavalry §Forge; Vellum *substeps*) | **NÃO** — 1 passo por tick; `iterations` só relaxa (não re-integra) | **omissão** | **P1** | `substeps = 1` ⇒ bit-idêntico |
| `motion.verlet_rope` | 6 | **perfil de rest length** ao longo (Vellum *Rest Length Scale*; Path Deform *Scale Ramp*, dump houdini §B) | **NÃO** (o rest é `length/(count−1)` uniforme) | **omissão** | **P2** | rampa plana ⇒ uniforme |
| `motion.verlet_rope` | 6 | pinar índice arbitrário (não só head/tail) | **NÃO** — fence declarada: [doc 34 §7](../34_pin_constraint_e_slit_scan_nota_adr.md) (*"não há como um stream entrar neles"*) | **omissão CONHECIDA** | **P1** | — |
| `motion.verlet_rope` | 6 | `length`/`gravity`/`iterations` sem `ParamHardMax` (o slider — 40/40/128 — É o teto) | **N/A** | **omissão** — a lei do doc 88 B2 só alcançou `count` e `damping` | ✅ **FECHADO (2026-08-12): `iterations = 128` · `gravity = 1e20` · `length = 1e20`**, medidos (sonda `measure_rope_ceiling`, tabelas no doc-comment do `PARAM_HARD_MAX`). O **`iterations`** já era **clampado** (`.clamp(1, 128)` no `eval`) ⇒ o teto **É** o clamp, e medido a queda da ponta em `129`, `512` e `100.000` sai **byte a byte** igual à de 128. ⚠️ **E `gravity` e `length` não são dois achados — são a MESMA parede por dois caminhos:** os dois morrem exactamente em **1e21**, porque o que estoura não é o parâmetro e sim a **POSIÇÃO** da corda em `f32` (*precisão de representação*, o recurso que o §0 admite nomear), com gate afirmando que as duas são a mesma para ninguém afinar uma sem a outra. ⚠️ **O modo de falha é o SILENCIOSO, e é o que paga um teto 19 ordens acima do slider:** em 1e21 a queda é **exactamente zero** — a corda não explode à vista, ela **DESAPARECE**, sem erro e sem aviso (o mesmo desenho do `SALTA` do `motion.spring`). O `length` é **escala-invariante até morrer** (`queda/repouso` = 1,0000 de 1e8 a 1e20), como o `radius` do `motion.collide` — e a diferença entre os dois casos é que lá a invariância ia até onde a medição alcançava e **não havia teto a escrever**. ⚠️ **DUAS armadilhas de fixture, nenhuma acusou sozinha:** a porta de estado é a **2** (as 0 e 1 são `anchor_x`/`anchor_y`) e o `connect` **aceitou** o `pre` na 1 sem reclamar — a corda nunca integrou e devolvia a pose de repouso, `6,0000` exacto até gravidade 1e8; e medir a queda com `powi(2)` em `f32` estoura em ~1e19, acusando a corda de morrer em 1e24 **por culpa da própria régua** | metadata ⇒ bit-idêntico |
| `motion.soft_body` | 9 | **`pressure`** (Cavalry Forge Soft Body, **verbatim**: *"stiffness, damping, **pressure**, shape matching, particle radius"*, dump cavalry §Forge; Houdini Vellum *Balloon*) | **NÃO** — é um termo dentro da projeção ao goal | **FEITO no W1** — `pressure`, a defesa do VOLUME | ✅ | `0` = off, byte-idêntico; a lei é o passo *deadbeat* `1 + (1−k)/k·(1−√(A/A₀))` |
| `motion.soft_body` | 10 | **shape matching por CLUSTERS sobrepostos** (Müller et al., *Meshless Deformations Based on Shape Matching*, SIGGRAPH 2005, **§4.3 — o PRÓPRIO paper que o nó cita**; Cavalry expõe *"shape matching"* como opção, dump cavalry §Forge) | **NÃO.** Compor N `motion.soft_body` dá N corpos INDEPENDENTES — clusters existem justamente porque as regiões **compartilham partículas** e os frames se misturam nas emendas; sem partícula compartilhada não há emenda | **FEITO no W1** — regiões sobrepostas, goal = média sobre quem contém a partícula | ✅ | `clusters = 1` ⇒ o frame único de hoje (early-out; a rota agrupada nem é entrada) |
| `motion.soft_body` | 8 | **particle radius / auto-colisão** (Cavalry Forge Soft Body verbatim) | **A MESMA cadeia do rope** (`soft_body.out --pre--> motion.collide → soft_body.state`) — `motion.collide` é Pure e repassa `sb_vel`/`sim_t`. **Não rodei** | omissão/ergonomia (o teste decide) | **P1**→**P2** | — |
| `motion.soft_body` | 8 | **gravidade VETOR** | **SIM** — idêntico ao rope (mesma família, mesmo mecanismo), e a medição cobre os DOIS na mesma varredura | **DISSOLVIDO no W1-A** | ~~P0~~ | o mesmo gate |
| `motion.soft_body` | 8 | forma inicial ≠ retângulo `rows×cols` (Cavalry põe soft body em QUALQUER shape; Vellum em qualquer geo) | **NÃO** — sem porta `in`, o corpo é o `rest_shape(rows, cols, spacing)` | **omissão** | **P1** | `rows×cols` segue o default |
| `motion.soft_body` | 8 | **goal/peso por partícula** (Blender Softbody *Goal* por vertex group, [docs.blender.org](https://docs.blender.org/manual/en/latest/physics/soft_body/settings/goal.html); espinha MOPs: *TODO modifier é modulado por `mops_falloff`*, dump mops §A.4) | **NÃO** — o nó não lê `falloff` nem `inv_mass` (zero `Coupling`) | **omissão** | **P1** | ausente ⇒ 1,0 em toda partícula |
| `motion.soft_body` | 8 | 8 sliders numa lista plana (doc 88 B3: *seções*) — Malha / Física / Pin são três perguntas | **N/A** | ergonomia | **P2** | seções não movem valor |
| `motion.collide` | 3 (+ `spread`) | **raio POR ELEMENTO** (Houdini POP Interact: *"raio por **`pscale`** ou explícito"*, dump houdini §B; C4D Push Apart usa o tamanho do clone) | **NÃO — e o dado JÁ CHEGA.** A porta `spread` é `Domain::Instances, Dim::Scalar` (**por-instância**) e o `eval` faz `spread_amount(…) = vals.first()`: uma coluna inteira colapsada num escalar. Não é param faltando, é **entrada descartada** | **omissão** | ⚠️ **MEIO-REFUTADO, e a metade errada era a minha** (`caaef0c45`, já no `main`). O `vals.first()` **não era descarte, era metade de uma DIVERGÊNCIA**: o WGSL lia `read_spread_v(i)` **por elemento** e a CPU lia o primeiro, então alimentar a porta com um campo variável dava **DOIS desenhos** — medido, a CPU devolvia a grade intocada (`vals[0] = 0` ⇒ raio 0 ⇒ identidade) e o device empurrava `0,165` de divergência de posição, **com todos os gates verdes porque as fixtures alimentavam comprimento 1**. Os dois lados passaram a ler a MESMA linha, e o global é agora **de propósito**: a porta é de VALOR e o que este nó quer dela é a *respiração* animável, um multiplicador só para todos os discos. ⚠️ **O raio POR ELEMENTO continua capacidade real e desejada, e é OUTRA LEI** — `r_i + r_j`, simétrica (e byte-idêntica à de hoje sob spread uniforme) —, que na grade do device exige limitar o alcance pelo raio **MÁXIMO**, ou seja um `Max` reduce sobre a coluna: **nenhum kernel do repo combina `register_grid` com `reduces()`**, então é wave própria com bloqueador NOMEADO (plano 89 §10.2, W1-B), não uma linha | **P1** (a lei `r_i + r_j`, com o bloqueador acima) | ler a coluna com fallback ao 1º elemento ⇒ toda cena de hoje (uniforme) bit-idêntica |
| `motion.collide` | 3 | **modos Push / Scale / Hide** (C4D Push Apart Effector — os três nomeados no [doc 63 §2.2](../63_pesquisa_industria_2026_e_plano_estado_da_arte.md)) | **NÃO** — Scale e Hide precisam de *quanto* cada disco se sobrepõe, e o nó não publica sobreposição nenhuma; nenhum campo a conhece | **omissão** ⚠️ **e o doc 63 propôs isso como NÓ NOVO (`motion.push_apart`, P1)** — os três modos partilham a MESMA varredura de pares, então são um param `mode` **aqui** | **P1** | `mode = Push` ⇒ hoje |
| `motion.collide` | 3 | **peso do EFEITO por elemento** (`falloff`, a espinha MOPs) | **PARCIAL.** `field.* → motion.pin_constraint → collide` dá um peso — mas com semântica de **MASSA**: um disco de `w=0` continua **obstáculo** (empurra os outros). `falloff=0` na referência significa *o nó não age ali*. São grandezas diferentes | **omissão** — e a distinção importa: *pinar* ≠ *desligar* | **P1** | `falloff` ausente ⇒ 1,0 |
| `motion.collide` | 3 | `radius` sem `ParamHardMax` (o slider, **5,0**, é o teto digitável) | **N/A** | **omissão** — `radius` é distância de mundo, não recurso; o §0 proíbe teto sem recurso nomeado | ✅ **FECHADO (2026-08-12), e a célula estava CERTA sobre o `radius`:** a medição confirmou a intuição dela — a fração adimensional `folga / 2·raio` vale **0,3317 de `r = 1` a `r = 1e15`**, quinze ordens de grandeza sem uma casa decimal se mover ⇒ o nó é **escala-invariante**, não existe ponto em que o kernel deixe de honrar o raio, e **ele NÃO ganhou teto**, com gate afirmando a AUSÊNCIA (senão a próxima varredura "completa" a tabela e inventa o palpite que o §0 proíbe). ⚠️ **Mas os OUTROS DOIS params do nó ganharam, e um deles mentia:** o **`iterations`** já era **clampado** em `MAX_ITERATIONS = 64` no `eval`, então a caixa aceitava 100.000 e o kernel entregava 64 — medido, `65 / 200 / 100.000` saem **byte a byte** iguais a 64 (a cicatriz do `lattice` 400 e do `kaleidoscope` 256, *aceita e mente*) ⇒ teto = **64**, o clamp. E o **`strength` = 3,0**, o único desta varredura em que a faixa confortável tem **mesmo folga**: o slider para em `1,0` e a sobre-relaxação acima disso **compra** empacotamento (0,5183 → 0,5995 a 64 varreduras, **+16% pelo mesmo custo**), com o teto onde o ganho ESTAGNA e antes de a nuvem ser ATIRADA (extensão salta de 2,4-2,5 para 3,54 em `3,8`, depois 6,17 e 12,28). ⚠️ **E a derivação óbvia do `strength` estava ERRADA — o PAR ISOLADO a refutou:** o limite clássico do SOR (*acima de 2 o par atravessa e oscila*) **não se aplica**, porque a restrição é de **um lado só** — o laço só empurra onde há penetração, então depois do overshoot não sobra sobreposição e a varredura seguinte não faz nada (medido: folga monotônica 0,6000 / 0,9500 / 3,0500 em `strength` 1 / 2 / 8). O que degrada é o caso **DENSO** | metadata |
| `motion.collide` | 3 | repulsão CONTÍNUA com sinal (POP Interact: *"Magnitude (+atrai/−repele) · falloff exponencial"*, dump houdini §B) | ⛔ **REFUTADO** — `force.attractor(repel) → motion.integrate` é exatamente isso, e já shipa. `motion.collide` é a **não-penetração** (raio duro); são dois nós porque são dois modelos | natureza | ⛔ | — |
| `motion.spring` | 3 | **massa** (CHOP Spring: *"k, **massa**, damping"*, dump houdini §A.3; MOPs Spring Modifier: *"Hooke (**mass**/k/damping)"*, dump mops §A.4) | ⛔ **GLOBAL: REFUTADO por álgebra** — o kernel é `a = −friction·v − tension·x`, e a referência é `a = −(c/m)v − (k/m)x`: o par `(tension, friction)` **já é** `(k/m, c/m)`; um `mass` global só re-parametriza. ✅ **POR ELEMENTO: gap real** — `inv_mass` existe mas é lido como **BLEND** (`fs = falloff·inv_mass`, `out = lerp(target, spring, fs)`), não como massa: `inv_mass = 0,5` dá meia-mistura, não uma mola 2× mais lenta | ⛔ global (natureza) · **omissão** por-elemento | ⛔ / **P1** | massa 1 ⇒ hoje |
| `motion.spring` | 3 | entrada **Posição OU Força** (CHOP Spring, dump houdini §A.3) | ⛔ **REFUTADO com cadeia** — `force.attractor(alvo) + force.drag + motion.integrate` **é** uma massa-mola dirigida por força, e os três já shipam. Custo: 3 nós | natureza | ⛔ (P2 se o Enio achar caro) | — |
| `motion.spring` | 3 | modos **Average / Blend / Spring** + clamp de slope/aceleração ([doc 63 §3.2](../63_pesquisa_industria_2026_e_plano_estado_da_arte.md) já lista; CHOP Lag: *"clamp de slope e de aceleração"*, dump houdini §A.3) | **NÃO** aqui — e o doc 63 §2.2 já dá o endereço certo: **`motion.lag` como nó P0 próprio**, não um modo do spring | **omissão da FAMÍLIA**, não deste nó | **P1** | — |
| `motion.spring` | 3 | 1 canal por nó ⇒ **4 nós** para molejar um transform (MOPs Spring faz T/R/S de uma vez, dump mops §A.4) | ✅ **EXPRIMÍVEL** — encadear 4 `motion.spring`; cada um tem `pre` próprio e o clobber de `spring_value` não os cruza (o loop de cada um lê a PRÓPRIA saída) | ergonomia (4 nós para um knob) | **P2** | — |
| `motion.spring` | 3 | `tension`/`friction` sem `ParamHardMax` — e o **próprio código** fala em *"absurd hand-authored overrides"* acima de 60, que a UI não deixa digitar | **N/A** | **omissão** | ✅ **FECHADO (2026-08-12): `tension = 1.600.000` · `friction = 20`, os dois MEDIDOS** (sonda `measure_spring_ceiling`, tabela no doc-comment do `PARAM_HARD_MAX`). ⚠️ **O relógio do PIOR caso é que decide:** o `eval` clampa `dt` em `MAX_DT = 0,1`, então um quadro perdido entrega legitimamente `0,1` ao integrador — *um teto que só vale a 60 fps depende da MÁQUINA*. ⚠️ **E o achado é sobre o SLIDER:** o `20,0` do `friction` **nunca foi número de gosto** — é `2 / MAX_DT`, o limite de estabilidade do amortecimento explícito no pior passo, e o slider já estava SENTADO no teto sem ninguém saber; a faixa confortável e a digitável **coincidem**, com gate para ninguém "consertar" a igualdade abrindo folga que o kernel não tem. O preço fica nomeado: entre 20 e ~120 a mola anda a 60 fps e explode no 1º `dt` grande, e o valor certo seria `20 · sub-passos`, ou seja **função da TENSÃO** — o pior caso é o único número que não mente. ⚠️ **A derivação óbvia da tensão estava ERRADA:** ela dá o teto na saturação do passo adaptativo (20.480) e **nada quebra ali**, porque `STABLE = 0,05` é **80× mais conservador** que a fronteira real do passo semi-implícito (`sub_dt²·T < 4`); o teto verdadeiro é `4·(64/0,1)² = 1.638.400`, o número que o relógio achou ao dígito. ⚠️ **E o oráculo precisou de TRÊS braços:** a 4 M o pico é exactamente `100,000` — *sadia*, para quem só olha magnitude — porque o guard de NaN **prega a mola no alvo** a cada tique; um controle que **parece não fazer nada**, que é o que o artista vê hoje sem teto nenhum | metadata |
| `motion.spring` | 3 | *rest offset* | ⛔ **REFUTADO** — `motion.spring → motion.move` assenta em `target + offset`, que é a definição | natureza | ⛔ | — |
| `motion.pin_constraint` | 3 | **break threshold** — o pin que RASGA sob carga (Houdini Vellum, *Breaking Threshold* em constraints, [vellumconstraints](https://www.sidefx.com/docs/houdini/nodes/dop/vellumconstraints.html)) | **NÃO** — nenhum solver publica a força/violação sentida no pin, então nada a jusante pode compará-la a um limiar | **omissão** — é o ÚNICO gap real deste nó | **P1** | `threshold = ∞` |
| `motion.pin_constraint` | 3 | `invert` (pinar TUDO menos a faixa) | ⛔ **REFUTADO** — `field.index_range{invert:1}` (ou `field.remap{invert}`) → `falloff` → `pin{count = n}`; o nó multiplica pelo `falloff` (`scalar_or(input, FALLOFF, n, 1.0)`) | natureza (fatorado, e o doc do nó já diz) | ⛔ | — |
| `motion.pin_constraint` | 3 | pin **aleatório / por probabilidade** | ⛔ **REFUTADO** — `field.remap` tem `probability` **e** `seed` (conferido no MANIFEST dele) → `falloff` → pin | natureza | ⛔ | — |
| `motion.pin_constraint` | 3 | **pin-to-animation** (Houdini Vellum `pintoanimation`) | ⛔ **REFUTADO** — `inv_mass = 0` em `motion.integrate` zera `sim_d` e o elemento **monta a pose viva do `rest`** (a tabela do [doc 34 §5](../34_pin_constraint_e_slit_scan_nota_adr.md), com mutação vermelha ao lado) | natureza — já é o comportamento | ⛔ | — |
| `motion.pin_constraint` | 3 | seleção por bbox / expressão / idade (Houdini POP Group, dump houdini §A.1) | ⛔ **REFUTADO** — a família `field.*` (5 nós) + `motion.expression` escrevem `falloff`, que é a porta de seleção deste nó | natureza (D1: composição por NÓS) | ⛔ | — |
| `motion.pin_constraint` | 3 | `Activation` por-nó (*"padrão de TODO POP"*, dump houdini §B, `popforce.html`) | ⛔ **REFUTADO** — é o **BYPASS (H)** do editor, semântico e no fingerprint do cook (CLAUDE.md §5, record `y`) | natureza | ⛔ | — |
| `motion.pin_constraint` | 3 | alcançar `boids`/`verlet_rope`/`soft_body` | **NÃO** — fence do [doc 34 §7](../34_pin_constraint_e_slit_scan_nota_adr.md) | **omissão CONHECIDA** (é o SUPERAR §2.1) | **P1** | — |

**Placar (atualizado na execução do W1):** **P0 = 8** → **ZERO abertos** · **P1 = 17** · **P2 = 6** · **⛔ refutados = 8**.

> **O que o W1 fechou, e como:** o **`bend stiffness`** da corda e o **`pressure`** do soft body (as duas
> últimas omissões de MODELO da família) · o **teto de `count`** do boids (medido no device: 1.048.576 em 14,283 ms contra os 10,392 ms que a CPU cobra por 2.000) · o **`max_force`** (o segundo clamp de Reynolds) · e ⚠️ **DOIS itens que DISSOLVERAM sem uma linha de feature** — as duas gravidades vetor, cujo mecanismo declarado era exatamente *"o nó não lê `accel`"*, removido pelo W1-A. O raio por-elemento do `motion.collide` virou a **W1-B** (a divergência CPU×GPU que ele escondia está FECHADA; o que falta é o `Max` reduce que o alcance da grade exige, máquina que nenhum kernel do repo tem).
>
> **Abertos na família:** nenhum. Os oito P0 fecharam — cinco construídos, dois DISSOLVIDOS
> por medição e um (o raio por-elemento do `motion.collide`) transferido para a W1-B com a
> máquina que lhe falta nomeada.
>
> ⚠️ **Os CLUSTERS fecharam com o oposto do que a fila supunha:** o item estava marcado
> *"omissão"* e a medição mostrou que era pior — o desvio da espinha de uma cobra 32×4 em
> relação à reta é **0,0000** do próprio comprimento, em toda stiffness e com o modo linear
> no máximo. Um frame só translada, gira e cisalha uniformemente; o corpo **não conseguia
> dobrar**, e nenhum ajuste alcançava isso. Com regiões sobrepostas o erro contra um arco
> conhecido cai **1,075 → 0,017** (63×, monotônico, na razão `1/n²` que um ajuste rígido
> contra um arco tem), e a espinha de uma cobra chicoteada desvia **5,6×** mais.
>
> ⚠️ **E o custo interage com o cap de 512 do nó, que foi medido contra UM shape match.**
> Clusters cobrem a malha ~4 vezes: 512² custa **1,77 ms com um frame e 5,3-5,9 agrupado**
> (3,4×, ou 275-297% do sub-orçamento). O cap **não** foi baixado quando há clusters — um
> teto de resolução que se mexesse com outro knob tiraria do artista uma malha que ele já
> autorou, e o valor certo seria função do vizinho. Os dois números multiplicam, os dois
> estão no painel, e a medição está escrita no `MAX_SIDE`.
>
> ⚠️ **O `pressure` fechou com a medição a decidir a LEI, não só o valor.** O plano dizia
> *"um termo dentro da projeção ao goal"*, e a primeira pergunta era se havia sequer o que
> fazer: o goal é a forma de repouso RÍGIDA ou o mapa linear ÁREA-PRESERVADO do artigo, e os
> dois já carregam a área de repouso. A sonda respondeu — a nuvem viaja só `stiffness` do
> caminho até o goal, e o volume se perde nessa folga: um corpo espremido assenta em **90,8 %**
> da área de repouso e fica lá para sempre, um sacudido perde **15 %**, e subir o `stretch` ao
> máximo muda isso para **90,7 %**, ou seja nada.
>
> ⚠️ **E a lei teve de ser derivada duas vezes.** A primeira — o ganho dividido pelo `travel` —
> ainda divergia para **12,3×** a área de repouso em `stiffness = 1`, porque ali o corpo *vira*
> o próprio goal e não sobra o que corrigir: todo empurrão é overshoot puro. O fator `(1−k)`
> é esse fato, e faz o termo desaparecer lá por ARITMÉTICA em vez de por caso especial. Com ele,
> a mesma pressão entrega **0,985 / 0,987 / 0,988** numa faixa de `stiffness` de quase uma ordem
> de grandeza — o knob passou a querer dizer uma coisa só.

---

## §2 — `SUPERAR:`

> Derivado do que só nós temos: **scrub bit-exato** (`Cook::checkpoint`/`restore` + `CheckpointRing`, GGPO
> save/load/advance, [doc 11](../11_checkpoint_restore_scrub_nota_adr.md)) · **grade espacial GPU-resident**
> (ADR-0140) · **params dirigidos = arestas** (doc 58) · **determinismo cross-OS gateado**.

### 2.1 — O `accel` é a porta que já existe e ninguém abriu *(o maior ganho por linha de código da família)*

Os três geradores não leem `accel`. Fazê-los ler dá, de uma vez, a **`force.*` inteira** a uma corda, a uma
gelatina e a um bando: gravidade com direção, vento, curl (= o *wander* do Reynolds), atrator, vórtice,
arrasto, empuxo — **todos já GPU-resident e bit-exatos**. E a fiação **já está construída**:
`shells/desktop/src/render_loop/motion_bridge_plumbing.rs` faz de **qualquer** porta `state`/`forces` do tipo
do output 0 um *feedback host*, e plumba `out --pre--> head` quando uma cadeia é ligada — a regra é genérica,
não uma exceção do `motion.integrate`.

**Ninguém tem isto.** No Houdini uma força chega a um Vellum só pela rede DOP; no Cavalry Forge os *Fields*
são uma lista fixa no mundo; no Stardust as forças são tipos de nó com regras de conexão próprias. Aqui é
**uma leitura de coluna** — e o `Coupling::Consumes("accel")` já é o vocabulário.

⚠️ **A armadilha, medida no código antes de propor:** a cadeia só sobrevive com nós **`Pure`**. Todo nó
`Temporal` carimba `sim_t = playhead` na saída (`motion.integrate` linha 364; `motion.spring` idem), e os
geradores derivam `dt = playhead − sim_t(state)`. Um `motion.integrate` dentro da cadeia de `state` de um
boids faz o boids ler `dt = 0` e **CONGELAR**. Corolário desagradável: a auto-cura do **ADR-0155** — que
propõe inserir `motion.integrate` quando vê `accel` sem consumidor — produziria exatamente esse grafo
congelado. Ou os geradores consomem `accel` (e o diagnose passa a vê-los como consumidores), ou o diagnose
precisa saber que uma cadeia de `state` de sim é outro contexto. **É o mesmo commit.**

### 2.2 — **O LOOP PERFEITO de uma simulação** *(a coisa que a categoria inteira quer e ninguém pode)*

Toda referência re-simula do zero: Cavalry Forge tem `Cache Solver → .sdcache`, o Stardust tem cache com
`Off/On/**Freeze**`, o Houdini precisa de DOP cache, e uma Niagara System tocada para trás simplesmente
recomeça. Nós temos **restore bit-exato e barato**.

O que isso destrava: `restore(tick 0)` + `advance(N)` é uma função pura de estado, então **um método de
tiro** sobre um punhado de restores resolve o estado inicial `s*` tal que `estado(N) == estado(0)`. Uma corda
que balança, um bando que gira e uma gelatina que treme **em loop costurado** — a coisa mais pedida do motion
design, e hoje impossível em qualquer app 2D. A Cavalry ship `Looping + Loop Length (frames)` no **noise**
justamente por isso (dump cavalry §Noise) e **não consegue fazê-lo na física**. Param: `loop_period`,
reusando o vocabulário que o cluster D3 já fixou. `loop_period = 0` ⇒ hoje, bit a bit.

### 2.3 — Pre-roll (a sim que nasce assentada), pelo mesmo mecanismo

MOPs lista *"Pre/Post-Roll (Plus) — gera automaticamente entrada/saída da animação"* como **FALTA** nosso
(dump mops §A.4); a Cavalry resolve com `Start Frame` (o artista desloca a cena). Com checkpoint/restore o
nó pode rodar `settle` ticks **no cook** e guardar o checkpoint assentado: a corda **pende certa no quadro 0**
e a gelatina começa em equilíbrio, sem janela de aquecimento e sem o artista mover nada. `settle = 0` ⇒ hoje.

### 2.4 — A vizinhança como COLUNA, de graça

`neighbours` já é computado nos dois caminhos do boids (`gpu.rs:130`, e o `inv_n` da CPU) e **descartado**; o
`motion.collide` percorre os mesmos pares. Emitir `neighbours` + `nearest_d` entrega ao domínio `value.*`
inteiro (cor por aglomeração, tamanho por densidade, `motion.cull` do solitário, `field.remap` sobre densidade)
o que Houdini vende como POP Proximity e MOPs como *Neighbors* — **dois nós que a referência precisa ter, e
que aqui não precisam existir**, porque a grade já está construída e é bit-exata cross-OS. Coluna ausente ⇒
ninguém lê ⇒ byte-idêntico.

### 2.5 — O cache que não existe é uma feature, não uma ausência

Cavalry (`.sdcache`), Stardust (`Freeze`) e Houdini (DOP cache) shipam um cache **e o problema de invalidação
dele**. A nossa sim é função de `(tick, params, ring)`, então o que eles expõem como cache nós expomos como
**nada** — é a mesma lei que o [ADR-0157](../../architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md)
do Painter registrou (*a lista É o estado; o cache é provadamente descartável*). Vale **nomear** isso na doc
de cada sim, para ninguém "completar a paridade" construindo um cache.

---

## §3 — `CERCAS:` (decisões já registradas — grepadas ANTES de propor)

1. **Os três geradores não têm porta `in`, e é declarado** — [doc 34 §7](../34_pin_constraint_e_slit_scan_nota_adr.md)
   (*"`motion.verlet_rope` / `soft_body` / `boids` ganharem porta `in` … Não é urgente e não foi feito: hoje
   não há como um stream entrar neles"*) e o doc-comment do `pin_constraint` (*"são **generatores** … um pin
   upstream não tem fio por onde chegar até eles"*). **Toda** proposta de pin/falloff/goal por-partícula nos
   três esbarra nesta cerca — a §2.1 a contorna por outro lado (`accel` no `state`), sem a porta `in`.
2. **`motion.pin_constraint` foi DEFERIDO até existir um desenho de cadeia de constraints** —
   [doc 22 §5](../22_soft_body_wave_nota_adr.md) (*"precisa de um port de cadeia de constraints no sim; DEFERIDO
   até esse design"*). Ele **landou depois** (doc 34) para os solvers de stream; a cerca sobrevive para os
   geradores.
3. **O spatial hash do boids é follow-up PURO-DE-PERF** — [doc 21 §2](../21_verlet_rope_boids_nota_adr.md)
   (*"não mudaria uma posição emitida"*). Ele **existe hoje na GPU** (ADR-0140). ⚠️ A cerca continua verdadeira
   *sobre a aparência* e é exatamente por isso que o teto de `count` da CPU não deveria governar o nó.
4. **`motion.collide` é Jacobi MEDIADO, não Gauss-Seidel, por CORREÇÃO** — doc-comment do nó (ordem do stream
   mudava o resultado em **6,11 unidades de mundo**; averaged Jacobi mede **0,0**). *Não "otimizar" de volta.*
5. **A divisão da penetração por `w` é a regra de projeção do PBD** — [doc 34 §5](../34_pin_constraint_e_slit_scan_nota_adr.md),
   com mutação vermelha registrada. Livre×livre = metade cada, **bit-idêntico ao mundo pré-pin**.
6. **`motion.soft_body` é shape matching, NÃO XPBD, e o cap 40→512 veio da MEDIÇÃO** — CLAUDE.md §5 + o
   doc-comment de `MAX_SIDE` (*"era 40 … custava 0,005 ms, 0,25% do orçamento — 164× abaixo"*). No cap quem
   aperta é o **RENDER**, não a sim. Não re-derivar.
7. **`motion.verlet_rope` é Gauss-Seidel por aresta, sequencial por semântica, e NÃO tem cap** — CLAUDE.md §5.
8. **A integração do soft body é a reformulação PBD (Müller 2007), não o velocity-blend da eq. 12 de 2005** —
   [doc 22 §2](../22_soft_body_wave_nota_adr.md). O `stretch` é o `β` do §4.2 do paper de 2005.
9. **`pin_tail` fixa a 75% do comprimento de propósito** (`PINNED_SPAN`) — *"deixando 25% de folga para o vão
   ceder numa catenária em vez de esticar numa reta"*.
10. **O `seek` do boids É a coleira** — [doc 21 §2](../21_verlet_rope_boids_nota_adr.md) (*"sem fly-away, sem
    hack de fronteira"*). Uma proposta de *bounds/kill volume* tem de dizer por que a mola linear não basta.
11. **`spread` (√N) é byte-idêntico em TODA contagem quando off** — doc-comment do param. Um default novo ali
    move arte já autorada.
12. **O `pre` não é fio que o artista desenha** — [doc 03/O1](../03_reentrada_integrate_estudo_padrao_ouro.md) via
    `motion_bridge_plumbing.rs` (*"gold-standard authoring never makes the artist DRAW that loop"*). Toda cadeia
    proposta aqui passa pela plumbing, nunca por uma aresta autorada.

---

## §4 — `O DOC 63 ERROU EM:`

1. **A §3.2 (a tabela de gap nó-a-nó) cobre `motion.spring` e MAIS NENHUM nó desta família.** `boids`,
   `verlet_rope`, `soft_body`, `collide` e `pin_constraint` **não têm linha** — cinco dos seis. A tabela
   anunciava *"o gap nó-a-nó dos 87 EXISTENTES"*; é o mesmo modo de falha que a §0 do plano 89 nomeia no
   doc 88 (**64 nós sem veredito**), um doc antes.
2. **`motion.push_apart` está proposto como NÓ NOVO (§2.2, P1)** com o motivo *"collide cobre o modo físico"*.
   Os três modos do C4D (Push / Scale / Hide) **partilham a varredura de pares** — Scale e Hide precisam do
   *quanto* cada disco se sobrepõe, que só o `motion.collide` computa. É um param `mode` **nele**, não uma
   crate nova; construir a crate duplicaria o `O(n²)` e daria duas respostas a *"quais discos colidem?"*.
3. **O dump do MOPs marca *Relax Modifier* (`separa instâncias sobrepostas`) como FALTA** (§A.4) — já era
   **falso quando foi escrito** (2026-07-24): `motion.collide` landou em 12/07 (doc 26). *Item marcado FALTA
   que já existe é tão caro quanto o inverso* — a §1 do plano 89 avisa exatamente isso.
4. **O dump do Niagara marca *Physics Connect (chains/springs)* como TEMOS** e o do Cavalry marca *Forge
   Dynamics* como PARCIAL — as duas linhas são certas quanto ao NOME e cegas quanto ao CONTEÚDO: o que a
   referência chama de *chain* traz **bend stiffness + espessura de colisão**, e o que ela chama de *soft
   body* traz **pressure + particle radius**. Uma coluna `status vs PH2D` que compara nomes não vê gap de
   PARÂMETRO — que é o aspecto 2 do próprio doc 63.
5. **A D13 (*"produto final, não MVP — o superset dos apps pro, conferido no catálogo ANTES de fechar o nó"*)
   não foi aplicada a esta família.** O `motion.soft_body` cita Müller 2005 no doc-comment e não implementa o
   §4.3 do paper que cita; o `motion.verlet_rope` cita Jakobsen 2001 e não tem bend. Os dois são o caso que a
   D13 descreve palavra por palavra.
6. **O §1 (*"Onde estamos"*) lista *"Zero pontes"* e *"Falloff raso"* mas não lista a ponte que falta DENTRO
   da própria casa:** os três geradores não consomem `accel`, então a família `force.*` — inteira, e já
   GPU-resident — não alcança nenhuma das três simulações. É o vão estrutural desta família e não aparece em
   nenhuma das sete linhas do §1.

---

## §5 — Os três fatos que a verificação (§5 do plano 89) deve conferir primeiro

1. **O teto de 2 000 do boids é da CPU?** — rodar `measure_the_count_ceiling` e confirmar que ele cozinha pelo
   `Cook` do registry (caminho CPU) enquanto `register_grid`/`gpu::GPU_KERNEL` estão registrados. Se sim, é o
   caso GPU/M5 da CLAUDE.md §0.0 repetido, e é o P0 mais barato da família.
2. **`motion.collide` na cadeia de `state` de um `verlet_rope` funciona?** — é a única cadeia desta conferência
   que pode já estar viva hoje. Se funcionar, a corda ganha auto-colisão **sem uma linha de código** e o item
   cai de P1 para nota de documentação.
3. **A auto-cura do ADR-0155 congela um boids?** — ligar `force.curl` no `state` de um `motion.boids` e ver se
   o diagnose oferece inserir `motion.integrate`. Se oferecer, o *quick-fix* produz um grafo em que
   `dt = playhead − playhead = 0`, e a sim para de andar sem erro nenhum.
