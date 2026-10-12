# 89 · CONFERÊNCIA — Família 3: **SIMULAÇÃO** (6 nós)

> História verbatim: [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/03_simulacao.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| `motion.boids` — 1 regra = 1 NÓ empilhável com peso próprio (família POP Steer, dump houdini §A.… | RECUSADO POR NATUREZA (2026-08-24), e o bloqueio é uma PRIMITIVA que não existe — escrito no doc-comment da crate. As três regras leem uma VIZINHANÇA, e o catálogo não a… | vivo, tabela |
| `motion.boids` — arrive (desaceleração na chegada, Reynolds) |  | vivo, tabela |
| `motion.soft_body` — particle radius / auto-colisão (Cavalry Forge Soft Body verbatim) | O TESTE DECIDIU (2026-08-24) — RODADO, e FUNCIONA (gate `soft_body_radius.rs`, o gêmeo exacto do `rope_thickness.rs`). Um corpo `5×5` de passo `0,30` com um colisor de r… | vivo, tabela |
| `motion.collide` — repulsão CONTÍNUA com sinal (POP Interact: "Magnitude (+atrai/−repele) · fallof… |  | vivo, tabela |
| `motion.spring` — massa (CHOP Spring: "k, massa, damping", dump houdini §A.3; MOPs Spring Modifie… | global (natureza) · ~~omissão~~ ergonomia por-elemento | vivo, tabela |
| `motion.spring` — entrada Posição OU Força (CHOP Spring, dump houdini §A.3) | (P2 se o Enio achar caro) | vivo, tabela |
| `motion.spring` — rest offset |  | vivo, tabela |
| `motion.pin_constraint` — `invert` (pinar TUDO menos a faixa) |  | vivo, tabela |
| `motion.pin_constraint` — pin aleatório / por probabilidade |  | vivo, tabela |
| `motion.pin_constraint` — pin-to-animation (Houdini Vellum `pintoanimation`) |  | vivo, tabela |
| `motion.pin_constraint` — seleção por bbox / expressão / idade (Houdini POP Group, dump houdini §A.1) |  | vivo, tabela |
| `motion.pin_constraint` — `Activation` por-nó ("padrão de TODO POP", dump houdini §B, `popforce.html`) |  | vivo, tabela |
| _Linhas ✅ (fechadas) da tabela_ — arquivadas quando o doc passou dos 40 KB | O placar e o gate `conferencia()` leem as linhas VIVAS (≥ 1 por nó); as fechadas estão verbatim no arquivo | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/03_simulacao.md) |

**Data:** 2026-08-09 · **Linha:** `line/motion-value` · **Escopo:** `motion.boids` · `motion.verlet_rope` ·
`motion.soft_body` · `motion.collide` · `motion.spring` · `motion.pin_constraint`
**Método:** §3 do [plano 89](../89_plano_conferencia_dos_nos.md). Params lidos do `MANIFEST` e do
`register_*` de cada crate (não do doc). Referência: os dumps do repo
([`referencia_pesquisa_houdini_mops.md`](../referencia_pesquisa_houdini_mops.md) ·
[`referencia_pesquisa_niagara_stardust.md`](../referencia_pesquisa_niagara_stardust.md) ·
[`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md)) + páginas oficiais citadas por URL
onde o repo está vazio.
**Status:** claims. A §5 do plano 89 é a verificação, e ela é do Enio.

> ⚠️⚠️ **O `motion.collide` SAIU DA LISTA DO ARTISTA em 2026-09-17, e esta folha continua a
> descrevê-lo como um nó que se escolhe.** Ordem do dono: *«tirar o collide e deixar tudo pela
> Shape»* — o nó deixou de aparecer na paleta e no catálogo, e a separação passou a ser um
> **interruptor do sink** (`Collide`), com a forma a declarar o próprio colisor.
>
> ⭐ **Tudo o que as células dizem sobre a LEI dele continua verdade** — a lei é a mesma, e o
> **kernel de dispositivo FICOU** por decisão do dono (medido: `4,19 M` objectos são `88×` mais
> rápidos ali do que no passe de CPU). O que envelheceu é a **PORTA**: onde se lê *«o artista liga
> um `motion.collide`»*, hoje lê-se *«o artista liga o `Collide` do sink»*.
>
> ⛔ **A folha NÃO foi reescrita célula a célula de propósito** — ela é o registo do que foi
> conferido naquela data, e reescrevê-la apagaria a proveniência de 43 linhas para corrigir uma
> palavra. *Uma folha de conferência é um instantâneo com data; o roteador é que tem de dizer o
> que mudou depois.* Mecanismo e medições: [doc 115 §14](../115_o_colisor_sai_do_grafo.md).

---

## §0 — O que os seis nós têm hoje (lido do código)

| nó | params (`MANIFEST`) | lê colunas | escreve colunas | hard max | unidades | seções |
|---|---|---|---|---|---|---|
| `motion.boids` | 16 — `count·seed·radius·separation·separation_radius·alignment·cohesion·seek·max_speed·spread·fov·speed_floor·avoid·avoid_radius·lookahead·max_force` | — (só o próprio `state`) | `P·vel·sim_t` | `count 2 000` | **nenhuma** | 3 (Flocking/Steering/Spawn) |
| `motion.verlet_rope` | 8 — `count·length·gravity·iterations·damping·pin_tail·bend·solver_substeps` | — | `P·rope_prev·sim_t` | `count 50 000` · `damping 0,5` | `length` = Length | — |
| `motion.soft_body` | 10 — `rows·cols·spacing·gravity·stiffness·stretch·damping·pin·pressure·clusters` | — | `P·sb_vel·sim_t` | `rows/cols 512` | `spacing` = Length | — |
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
| `motion.boids` | idem | **avoid obstacle / lookahead** (POP Steer Avoid Obstacle, dump houdini §B) | **NÃO** — mesmo mecanismo do wander, mais a ausência de qualquer geometria de obstáculo alcançável de dentro | **omissão** | ✅ **FEITO** (era P1) — a porta `obstacle` + `avoid`/`avoid_radius`/`lookahead`. ⚠️ **A objecção da célula (*"a ausência de qualquer geometria de obstáculo alcançável de dentro"*) deixou de valer NESTA janela:** a porta-template é o padrão que o `field.shape` estreou e o `force.attractor` repetiu. A sonda é `p + v·lookahead`, então o raio efectivo cresce com a velocidade DO AGENTE sem um segundo knob. Rampa linear e não inverso do quadrado (infinito no contacto). ⚠️ Ligado, RECUSA o device | `lookahead = 0` |
| `motion.boids` | idem | **`min_speed`** exposto (POP Speed Limit: *"Min Speed (0) · Max Speed"*, dump houdini §B) | **NÃO** — hoje é a const `MIN_SPEED_FRAC = 0.2` **escondida no arquivo**; o artista não a alcança | **omissão** — o número existe e não tem porta | ~~**P1**~~ ✅ **FECHADO em 2026-08-15** — param `speed_floor`, rótulo **"Speed Floor"**, default `0,2` (a const que sempre shipou). ⚠️ **FRAÇÃO e não velocidade absoluta, e a razão é byte-identidade:** o piso vivo sempre foi `max_speed * 0,2`, então expô-lo em unidades de mundo pediria um default que **depende de outro param** e mudaria o desenho de todo documento cujo `max_speed` não fosse o de fábrica, em silêncio. O rótulo evita *Min Speed* justamente para o `0,2` não ser lido como m/s | `min_speed_frac = 0.2` ⇒ bit-idêntico |
| `motion.boids` | idem | 1 regra = 1 NÓ empilhável com peso próprio (família POP Steer, dump houdini §A.1/§B) | **NÃO** (não há primitiva de vizinhança exposta: `value.reduce`/`median`/`percentile` reduzem o stream INTEIRO, não uma vizinhança) — **mas os 3 pesos já são params dirigíveis** (doc 58), que é o ganho de mixar animadamente | ⛔ **RECUSADO POR NATUREZA (2026-08-24), e o bloqueio é uma PRIMITIVA que não existe** — escrito no doc-comment da crate. As três regras leem uma **VIZINHANÇA**, e o catálogo não a expõe: as reduções que há (`value.reduce`/`median`/`percentile`) reduzem o stream INTEIRO. Decompor exigiria construir antes *«o que está perto de mim»* como canal — uma wave, não uma refactoração. ⚠️ **E o que a decomposição compraria já está comprado:** os três pesos são params **dirigíveis**, então misturar as regras animadamente faz-se hoje com fios; o que fica por comprar é a EXTENSIBILIDADE, que é promessa de um substrato. natureza — o monolito entrega o gesto; a decomposição entrega extensibilidade | **P2** | — |
| `motion.boids` | idem | *arrive* (desaceleração na chegada, Reynolds) | ⛔ **REFUTADO** — o `seek` é mola **linear** (`accel += (target − p)·seek`), então a força já cai com a distância: isso **é** o *arrive* | natureza | ⛔ | — |
| `motion.verlet_rope` | 6 | **gravidade VETOR** (Niagara Gravity Force `vec3` default `(0,0,−980)`, dump niagara §C.1; Cavalry Forge world `Gravity`, dump cavalry §Forge) | **SIM — pela CADEIA DE FORÇA, e isto foi MEDIDO.** O mecanismo que esta linha nomeava (*"o nó não lê `accel`, e a cadeia `force.wind → rope.state` é autorável mas INERTE"*) foi removido pelo **W1-A**; pelo §0 (*quem move o número que tornava algo inalcançável reconfere a nota*) o item foi re-perguntado ao produto: **`gravity = 0` + `force.wind(270°)` reproduz a gravidade embutida** dentro de 1e-4 da queda — a diferença que resta é a ASSOCIAÇÃO IEEE (`(g·dt)·dt` contra `a·(dt·dt)`), não o modelo. Um `dir` por nó seria a SEGUNDA porta para a mesma pergunta, contra a semântica Houdini que o módulo inteiro segue | **DISSOLVIDO no W1-A** | ~~P0~~ | gate `vector_gravity_is_the_force_chain_and_needs_no_second_door` + o controle irmão; 3 gates sangram se o mecanismo regredir |
| `motion.verlet_rope` | 7 | **bend stiffness** (Houdini Vellum Constraints: *Stretch Stiffness* **+ *Bend Stiffness*** + *Thickness* + *Plasticity*, [vellumconstraints](https://www.sidefx.com/docs/houdini/nodes/dop/vellumconstraints.html); Cavalry Forge Collision Shape **Chain**, dump cavalry §Forge) | **NÃO** — é uma restrição i↔i+2 dentro da relaxação; nada de fora acrescenta constraint | **FEITO no W1** — a irmã `i↔i+2` no MESMO passe de relaxação, repouso `2·seg_rest` | ✅ | `bend = 0` ⇒ a corrente de hoje, ao bit (early-out: `x + (−0,0)` devolve `x`, mas `(−0,0) + 0,0` devolve `+0,0`) |
| `motion.verlet_rope` | 6 | **perfil de rest length** ao longo (Vellum *Rest Length Scale*; Path Deform *Scale Ramp*, dump houdini §B) | **NÃO** (o rest é `length/(count−1)` uniforme) | **omissão** | ✅ **FECHADA em 2026-08-24** — params `rest_start` / `rest_end` / `rest_profile` (a mesma família de quatro dos `field.*`). ⚠️ **O COMPRIMENTO TOTAL é preservado**, e não é cosmética: os multiplicadores são **normalizados pela média**, então o perfil REDISTRIBUI o repouso em vez de encompridar a corda — sem isso, baixar o `Rest End` fá-la-ia encolher e o artista teria **dois knobs a discordar sobre o comprimento**. ⚠️ **Plano ⇒ `None`**, e os TRÊS leitores (a semente, a restrição de distância, a de flexão) usam o mesmo `f32` de sempre — byte-idêntico por ESTRUTURA. ⚠️ **A semente passa a ser a soma CUMULATIVA** dos segmentos: uma corda afunilada tem de nascer com os nós onde o solver os quer. ⚠️ **E um perfil degenerado (média ≤ 0) DESISTE** em vez de colapsar a corda num ponto. **8 gates.** Cena `=92` | ✅ | `1, 1` ⇒ uniforme |
| `motion.verlet_rope` | 6 | pinar índice arbitrário (não só head/tail) | **NÃO** — fence declarada: [doc 34 §7](../34_pin_constraint_e_slit_scan_nota_adr.md) (*"não há como um stream entrar neles"*) | **omissão CONHECIDA** | ~~**P1**~~ ✅ **FECHADO em 2026-08-16** (grupo J) — e a cerca caiu **sem porta `in` nenhuma**: o fio é a **CADEIA DE ESTADO**, que já era o fio pelo qual o `accel` entra. `rope.out --pre--> motion.pin_constraint --> rope.state` prega um índice arbitrário, e o pino é `Effect::Pure` ⇒ não carimba `sim_t`. ⚠️ **A fórmula PBD estava ESCONDIDA na tabela de quatro braços da relaxação** (`(true,false) => (0.0, 1.0)` etc.), que é a forma booleana de `w_i/(w_i+w_j)` com pesos em `{0,1}` — a substituição é **byte-idêntica por ARITMÉTICA** (`0/1`, `1/1` e `1/2` são exactos em IEEE-754) e é isso que fez os 11 gates daquele arquivo passarem **sem uma edição**. Medido: o ponto 12 de uma corda de 24 sai de `[-0,70, -3,07]` para `[3,13, 0,00]`, e o pior deslocamento é **6,0012** | — |
| `motion.soft_body` | 9 | **`pressure`** (Cavalry Forge Soft Body, **verbatim**: *"stiffness, damping, **pressure**, shape matching, particle radius"*, dump cavalry §Forge; Houdini Vellum *Balloon*) | **NÃO** — é um termo dentro da projeção ao goal | **FEITO no W1** — `pressure`, a defesa do VOLUME | ✅ | `0` = off, byte-idêntico; a lei é o passo *deadbeat* `1 + (1−k)/k·(1−√(A/A₀))` |
| `motion.soft_body` | 10 | **shape matching por CLUSTERS sobrepostos** (Müller et al., *Meshless Deformations Based on Shape Matching*, SIGGRAPH 2005, **§4.3 — o PRÓPRIO paper que o nó cita**; Cavalry expõe *"shape matching"* como opção, dump cavalry §Forge) | **NÃO.** Compor N `motion.soft_body` dá N corpos INDEPENDENTES — clusters existem justamente porque as regiões **compartilham partículas** e os frames se misturam nas emendas; sem partícula compartilhada não há emenda | **FEITO no W1** — regiões sobrepostas, goal = média sobre quem contém a partícula | ✅ | `clusters = 1` ⇒ o frame único de hoje (early-out; a rota agrupada nem é entrada) |
| `motion.soft_body` | 8 | **particle radius / auto-colisão** (Cavalry Forge Soft Body verbatim) | **A MESMA cadeia do rope** (`soft_body.out --pre--> motion.collide → soft_body.state`) — `motion.collide` é Pure e repassa `sb_vel`/`sim_t`. **Não rodei** | omissão/ergonomia (o teste decide) | ⛔ **O TESTE DECIDIU (2026-08-24) — RODADO, e FUNCIONA** (gate `soft_body_radius.rs`, o gêmeo exacto do `rope_thickness.rs`). Um corpo `5×5` de passo `0,30` com um colisor de raio `0,28` no laço de estado: o par mais próximo vai de **0,3000 para 0,3448** e a extensão de **1,9221 para 3,4814** — as partículas param de se aproximar mais que o colisor pede, e o corpo INCHA, que é o que um *particle radius* faz. ⇒ **não falta capacidade, falta o GESTO** (dois nós e duas arestas à mão), a MESMA conclusão do rope e pelo mesmo mecanismo. ⚠️ **A régua NÃO é a distância entre vizinhos de malha** — o corpo segura-os a `spacing` de propósito, e `spacing` é menor que o diâmetro; medir todos os pares acusaria o corpo de se atravessar exactamente onde as molas dele trabalham. ⚠️ **E o CONTROLE vem primeiro**: as duas poses têm de diferir, senão o gate ficaria verde sobre uma cadeia que nunca integrou — a 2ª fixture reprovada do gate da corda. **2 gates** | ⛔ | — |
| `motion.soft_body` | 8 | **gravidade VETOR** | **SIM** — idêntico ao rope (mesma família, mesmo mecanismo), e a medição cobre os DOIS na mesma varredura | **DISSOLVIDO no W1-A** | ~~P0~~ | o mesmo gate |
| `motion.soft_body` | 10 | 8 sliders numa lista plana (doc 88 B3: *seções*) — Malha / Física / Pin são três perguntas | **N/A** | ergonomia | ~~**P2**~~ ✅ **FECHADO em 2026-08-23** (junto da porta) — **Mesh** / **Physics** / **Pin**, as três perguntas da célula. ⚠️ **Nada fica solto**, e é uma escolha: um param solto aparece ANTES de toda seção, e aqui os dez pertencem, cada um, a exactamente uma das três. ⚠️ **A seção `Mesh` só responde com a porta `shape` VAZIA, e o painel não a esconde de propósito** — desligar o fio devolve o corpo que ela descreve, e um knob que desaparece leva a forma com ele | seções não movem valor |
| `motion.collide` | 3 | **peso do EFEITO por elemento** (`falloff`, a espinha MOPs) | **PARCIAL.** `field.* → motion.pin_constraint → collide` dá um peso — mas com semântica de **MASSA**: um disco de `w=0` continua **obstáculo** (empurra os outros). `falloff=0` na referência significa *o nó não age ali*. São grandezas diferentes | **omissão** — e a distinção importa: *pinar* ≠ *desligar* | ~~**P1**~~ ✅ **FECHADO em 2026-08-15** (grupo H) — e a distinção que esta célula nomeia é o desenho inteiro: `falloff = 0` torna o disco **TRANSPARENTE** (nem empurra nem é empurrado), que é precisamente o que `inv_mass = 0` **não** significa (pinado é imóvel **e obstáculo**). O peso do PAR é o produto `f_i · f_j`, então mutar qualquer lado faz o par desaparecer. ⚠️ **MEDIDO E REJEITADO, não refaça: a tally da média PONDERADA.** Pôr o peso do par no numerador E no divisor o faz **CANCELAR** num par isolado — `falloff = 0,5` desenhava exatamente o mesmo que `1,0` (0,6 e 0,6), e o knob seria decorativo. A tally é uma CONTAGEM, e o resíduo (um par a um fio de zero ainda ocupa um slot, `n/(n−1)`) está NOMEADO no `push_apart`, a mover-se na direção inofensiva | `falloff` ausente ⇒ 1,0 |
| `motion.collide` | 3 | repulsão CONTÍNUA com sinal (POP Interact: *"Magnitude (+atrai/−repele) · falloff exponencial"*, dump houdini §B) | ⛔ **REFUTADO** — `force.attractor(repel) → motion.integrate` é exatamente isso, e já shipa. `motion.collide` é a **não-penetração** (raio duro); são dois nós porque são dois modelos | natureza | ⛔ | — |
| `motion.spring` | 3 | **massa** (CHOP Spring: *"k, **massa**, damping"*, dump houdini §A.3; MOPs Spring Modifier: *"Hooke (**mass**/k/damping)"*, dump mops §A.4) | ⛔ **GLOBAL: REFUTADO por álgebra** — o kernel é `a = −friction·v − tension·x`, e a referência é `a = −(c/m)v − (k/m)x`: o par `(tension, friction)` **já é** `(k/m, c/m)`; um `mass` global só re-parametriza. ✅ **POR ELEMENTO: gap real** — `inv_mass` existe mas é lido como **BLEND** (`fs = falloff·inv_mass`, `out = lerp(target, spring, fs)`), não como massa: `inv_mass = 0,5` dá meia-mistura, não uma mola 2× mais lenta | ⛔ global (natureza) · ~~**omissão**~~ **ergonomia** por-elemento | ⛔ / ~~**P1**~~ → **P2** ✅ **REFUTADO POR MEDIÇÃO em 2026-08-16** (grupo L), sem uma linha de código. A metade *"POR ELEMENTO: gap real"* supunha que só reinterpretar o `inv_mass` como massa daria molas diferentes no mesmo conjunto — e **a composição já as dá**: cada `motion.spring` tem os SEUS `tension`/`friction`, e o `falloff` torna uma mola **transparente** onde vale zero, então duas molas em série com falloff COMPLEMENTAR entregam duas massas efetivas. Medido (sonda `measure_whether_two_springs_give_two_effective_masses`, alvo que salta 0 → 5): o elemento da mola RÁPIDA sai em **7,0213** — **idêntico ao controle de uma mola só, ao dígito**, que é a prova de que a segunda é transparente ali — e o da LENTA em **2,0119**, ainda a subir. ⚠️ **E a cerca do `inv_mass` não devia ser derrubada de qualquer modo:** ali `0` significa *segue o alvo RIGIDAMENTE* (documentado e gateado), que é a lei do grupo J lida corretamente — *massa infinita segue a pose que o nó sabe PRESCREVER*, e o que este nó prescreve é o alvo cru; sob a lei de massa o mesmo `0` CONGELARIA o valor, a resposta oposta. ⇒ o custo real é **N nós para N massas** (ergonomia), exactamente como a linha 68 do mesmo nó já registra, e a massa CONTÍNUA por elemento fica **nomeada e não construída** | massa 1 ⇒ hoje |
| `motion.spring` | 3 | entrada **Posição OU Força** (CHOP Spring, dump houdini §A.3) | ⛔ **REFUTADO com cadeia** — `force.attractor(alvo) + force.drag + motion.integrate` **é** uma massa-mola dirigida por força, e os três já shipam. Custo: 3 nós | natureza | ⛔ (P2 se o Enio achar caro) | — |
| `motion.spring` | 3 | 1 canal por nó ⇒ **4 nós** para molejar um transform (MOPs Spring faz T/R/S de uma vez, dump mops §A.4) | ✅ **EXPRIMÍVEL** — encadear 4 `motion.spring`; cada um tem `pre` próprio e o clobber de `spring_value` não os cruza (o loop de cada um lê a PRÓPRIA saída) | ergonomia (4 nós para um knob) | ✅ **FECHADA em 2026-08-24 — no caso COMUM, e a fronteira está nomeada.** Canal **`Position XY`** (o precedente e a palavra do `motion.wiggle`): uma mola de posição precisa dos dois eixos, e dois nós para *«esta coisa persegue aquela»* é o dobro do grafo para um gesto só. ⚠️ **O estado é APPEND-ONLY** (`spring_value_y`/`spring_vel_y`), colunas que os quatro canais escalares **nunca escrevem** — e há gate a prová-lo, mais um que mede o eixo X do par **ao bit** contra o canal X. ⛔ **E NÃO é o T/R/S inteiro da referência:** rotação e tamanho têm unidades próprias e quereriam a sua própria tensão (uma mola que persegue graus com a rigidez de metros não é a mesma mola), então juntá-los pediria um jogo de knobs por canal — o nó de quatro cabeças que a cerca §4-C3 do `motion.scale` já ensinou a não construir. **3 gates.** Cena `=92` | ✅ | `channel = X` ⇒ o de sempre |
| `motion.spring` | 3 | *rest offset* | ⛔ **REFUTADO** — `motion.spring → motion.move` assenta em `target + offset`, que é a definição | natureza | ⛔ | — |
| `motion.pin_constraint` | 3 | `invert` (pinar TUDO menos a faixa) | ⛔ **REFUTADO** — `field.index_range{invert:1}` (ou `field.remap{invert}`) → `falloff` → `pin{count = n}`; o nó multiplica pelo `falloff` (`scalar_or(input, FALLOFF, n, 1.0)`) | natureza (fatorado, e o doc do nó já diz) | ⛔ | — |
| `motion.pin_constraint` | 3 | pin **aleatório / por probabilidade** | ⛔ **REFUTADO** — `field.remap` tem `probability` **e** `seed` (conferido no MANIFEST dele) → `falloff` → pin | natureza | ⛔ | — |
| `motion.pin_constraint` | 3 | **pin-to-animation** (Houdini Vellum `pintoanimation`) | ⛔ **REFUTADO** — `inv_mass = 0` em `motion.integrate` zera `sim_d` e o elemento **monta a pose viva do `rest`** (a tabela do [doc 34 §5](../34_pin_constraint_e_slit_scan_nota_adr.md), com mutação vermelha ao lado) | natureza — já é o comportamento | ⛔ | — |
| `motion.pin_constraint` | 3 | seleção por bbox / expressão / idade (Houdini POP Group, dump houdini §A.1) | ⛔ **REFUTADO** — a família `field.*` (5 nós) + `motion.expression` escrevem `falloff`, que é a porta de seleção deste nó | natureza (D1: composição por NÓS) | ⛔ | — |
| `motion.pin_constraint` | 3 | `Activation` por-nó (*"padrão de TODO POP"*, dump houdini §B, `popforce.html`) | ⛔ **REFUTADO** — é o **BYPASS (H)** do editor, semântico e no fingerprint do cook (CLAUDE.md §5, record `y`) | natureza | ⛔ | — |

⚠️ **DUAS células desta tabela ENVELHECERAM e a varredura de 2026-08-15 as reconferiu antes de construir** (§0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*): o **`max_force`** já existe como param desde `2ec7dc552` (*o SEGUNDO clamp de Reynolds*), e o **wander** deixou de estar bloqueado quando o **W1-A** fez os três geradores consumirem `accel` — a célula ainda diz *"boids não lê `accel`"*, e o `gpu.rs` traz `ColumnAccess::Consume` naquela coluna. *Construir a partir de uma tabela sem a reconferir teria produzido dois params que já existiam.*

⚠️ **E as DUAS últimas linhas desta tabela não vieram da referência — vieram de um REPORT do smoke da cena `=50`** (2026-08-16), e por isso não movem o placar: elas nascem `✅ FECHADO` sem P1 anterior. *Uma varredura que só lesse a coluna de referência as teria perdido: o Enio olhou para a tela e nomeou duas coisas que o catálogo não pede e o produto precisa.*

**Contagem (DERIVADA, reconciliada em 2026-08-24):** 43 linhas — **P0 = 0** · **P1 = 0** · **P2 = 0** · ✅ fechadas **31** · ⛔ recusadas/refutadas **12**. ⭐⭐ **ESTA FOLHA ESTÁ A ZERO.** ⚠️ **E metade das quatro que restavam não era código:** uma dizia *«não rodei»* — e uma célula que diz isso **não é uma omissão nem uma recusa, é uma pergunta sem resposta**, com o veredito já escrito à espera de um número (`omissão/ergonomia — o teste decide`). Rodada, a cadeia `soft_body → motion.collide → state` **funciona** (par mais próximo `0,3000 → 0,3448`), e a célula fechou sem uma linha de produto, exactamente como a gêmea do rope tinha fechado em 15/08. A outra era **natureza com bloqueio nomeado** (decompor o `motion.boids` pede uma primitiva de VIZINHANÇA que o catálogo não expõe). ⚠️ *Duas das quatro custaram um gate cada, e nenhuma um param.* Antes: P2 = 4 · ✅ 29 · ⛔ 10.

Re-medir: `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` — ⚠️ **esta linha é DERIVADA da coluna `P` da tabela acima; não a edite à mão** (a contagem desta conferência envelheceu SEIS vezes, e a folha 13 chegou a contradizer a própria prosa três parágrafos abaixo).

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
