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
   desta família. ⚠️ **CORRIGIDO em 2026-08-11 — esta linha ENVELHECEU dentro da própria linha
   que a escreveu:** ela dizia *"`motion.drive` tem exatamente cinco canais"* e citava
   `labels: &["X", "Y", "Rotation", "Size", "Opacity"]`, o que era verdade no dia em que foi
   medida e deixou de ser na wave W3 COR, que lhe acrescentou **Falloff · Hue · Sat · Val**
   (`channel.rs`: `CH_FALLOFF = 5`). *Quem move o número que tornava algo inalcançável tem de
   reconferir a nota* — e o que ele destrava é grande: `motion.drive(Falloff)` → `motion.cull`
   **dentro da zona** é uma MORTE dirigida por valor, então o *die on collide* passou a ser uma
   composição de nós que já existem. O que segue verdadeiro é a frase seguinte: **nenhum
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
| `sim.zone` | 0 | **SUBSTEPS / time step** — Houdini DOP Network `Substeps` ([houdini_mops §A.1](../referencia_pesquisa_houdini_mops.md), *"cada POP é um micro-solver … que o solver integra"*) · Cavalry Forge Dynamics **`Time Step`** ([cavalry:163](../referencia_pesquisa_cavalry.md)) · Niagara **Simulation Stages** (múltiplos passes nomeados por tick — [niagara §A1](../referencia_pesquisa_niagara_stardust.md)) | **PARCIAL — o mecanismo EXISTE e a medição o achou; falta o ESCOPO.** Encadear `sim.step` duas vezes no interior é no-op exato (o 1º termina em `out.set("sim_t", vec![playhead; n])`, `sim-step:265`, então o 2º lê `sim_t == playhead` ⇒ `dt = 0`) — mas o corolário que esta folha tirou daí (*"não há outro lugar para pôr um 2º passe"*) está **REFUTADO por medição**: o `dt` é `playhead − read_sim_t(i)`, logo **um substep é propriedade do RELÓGIO, não um param do passo** — subdividir o playhead entre `cook`/`advance_tick` substepa a zona com **zero mudança de motor**. Medido (queda com aceleração constante, alvo analítico 20,0): erro **−0,9889 / −0,4972 / −0,2493 / −0,1248 / −0,0625** em sub 1/2/4/8/16 — **cai pela metade a cada dobra**, Euler de 1ª ordem exato. ⚠️ **O que falta é o escopo, e o preço dele está medido:** quem torna isto GLOBAL é o `advance_tick`, que cozinha **TODA** fonte `pre` do grafo — numa cena com a zona mais um vizinho de 20k linhas que nada tem com ela, N passadas custam **1,83× / 3,95× / 8,13× / 16,36×** (0,309 → 5,062 ms/quadro), **linear e pago pelo grafo inteiro**. ⇒ o substep tem de ser POR-ZONA: um bracket no `Cook` que cozinha só o alvo em `t_k`, fotografa só as fontes `pre` DELE, e **restaura o `prev_playhead`** no fim — ele avança durante o laço porque a lei de contagem do `sim.spawn` lê `dt` dali (`born_in` **telescopa** ⇒ o total de nascimentos por quadro não muda), e restaurar é o que impede o resto do grafo de receber **uma fatia como se fosse o quadro**. O param entra como **metadado lateral no registry** (a §5 já o diz), nunca no `NodeManifest`; a última sub-passada cai em `playhead` ⇒ o `cook` do quadro **bate no memo** e não dobra o passo <br><br>**⬛ O MOTOR LANDOU** (`Cook::substep`, filho `cook_substep.rs`; 6 gates, 6 mutações, 5 sangram). A convergência é exata — erro **−0,9889 / −0,4972 / −0,2493 / −0,1248 / −0,0625** em n = 1/2/4/8/16 — e o escopo tem gate próprio (substepar uma zona deixa a vizinha **byte-idêntica**). ⚠️ **Três fatos de relógio que a medição achou, não o desenho:** um substep é um **sub-TIQUE** (o fingerprint de um pre-consumidor inclui `self.tick`; sem bumpar, as passadas 2..n batem no memo) · o **`frame_start` é do CHAMADOR** (ler o `prev_playhead` interno é `None` no 1º tique e colapsa o span, o que faz o 1º quadro rodar GROSSO e **satura o erro em −0,64** — converge e para, a pior forma de um defeito) · e o relógio é **restaurado**, embora **avance** dentro do laço (a lei de contagem lê `dt` dali; `born_in` telescopa ⇒ os nascimentos não mudam). ⚠️ **E a fixture da convergência não continha o 1º:** na zona que cai quem consome o `pre` é o `force.wind`, **`Effect::Temporal`** ⇒ o playhead o re-cozinha sozinho e MASCARA o bump ausente; quem consome o `pre` na fiação canônica é o `motion.combine`, **`Effect::Pure`**, e é a zona que ACUMULA por ele que expõe. **⬛ E O PARAM E O PUMP FECHARAM.** O `sim.zone` ganhou **`substeps`** (default **1**, faixa confortável **1..16**, teto digitável **64**) — e ⚠️ **a declaração é uma CONVENÇÃO DE MANIFESTO, não um canal novo:** um nó cujo manifesto declara um param `substeps` está a dizer *"o meu interior sub-tica"*, e é **o mesmo param que o artista edita** (um fato, um lugar), a forma das convenções de stream que o módulo já usa (`texture_id`, `geometry_id`). ⚠️ **O teto é do RELÓGIO DE PAREDE e a tabela está no `MAX_SUBSTEPS`** (§0): a 16.384 partículas `sub = 64` custa **15,91 ms — 95% de um quadro de 60 fps** —, e a faixa do arrasto para em 16 porque o erro já está em 0,6% ali. O pump chama o achador nas **DUAS** marchas (a de frente e o replay do scrub), com `frame_start` vindo do relógio do próprio cook e **`None` pulando** (o primeiro tique de todos, onde a zona ainda emite o `init`). ⚠️ **E o substep é da ILHA, não da zona — isso é CORREÇÃO, e a medição é que o disse:** substepar cada declarante por conta **OVER-STEPA** qualquer um que viva no cone de outro (o laço de baixo re-cozinha o cone inteiro) — num par acoplado a de cima ia de **4,876 para 15,094**, quase o triplo. `substep_islands` parte o grafo em ilhas e devolve **uma raiz por ilha**; a fronteira é a aresta **`pre`** (*o valor do tique anterior não é deste alvo para avançar*), então ler a outra zona por um `pre` **não** acopla as duas. ⚠️ **E o RITMO é do GRAFO, não da ilha** — a decisão que o Enio desempatou ao dizer que **cada objeto Motion terá o seu próprio grafo**: com isso o grafo É o contêiner, e é exactamente onde as referências põem o substep (Houdini na **DOP Network**, Niagara o Fixed Tick no **System**; o device já exigia **um sink** por documento, fechando a mesma fronteira do outro lado). Todas as ilhas correm no maior ritmo que qualquer declarante pede. ⚠️ **E é isso que APAGA a recusa do device, em vez de a estreitar:** marchar o plano inteiro `n` vezes dá a cada ilha exactamente os `n` sub-tiques que o bracket da CPU lhe daria — **idêntico, não aproximado** —, então os dois produtores concordam **por construção** e nenhum precisa escolher entre acelerar e estar certo. A recusa anterior custava a aceleração a **100% dos documentos reais** (medido: toda cena de demo deste repo tem **uma** zona). ⚠️ **O que se abre mão é nomeado:** duas simulações independentes **no mesmo objeto** em ritmos diferentes correm as duas no mais fino — e com um grafo por objeto isso autora-se como dois objetos. Um ritmo por ilha exigiria relógios por-ilha DENTRO do sequenciador de device (uma chamada de `cook` carrega UM playhead), e compraria só esse caso. ⚠️ **No device o TIQUE não se subdivide, só o PLAYHEAD**, e as duas metades são load-bearing: ele avança o ping-pong do `pre` a cada CHAMADA de `cook`, e o ring de scrub chaveia pelo tique com `should_record` a deduplicar — então a 1ª sub-passada grava o estado de ENTRADA do quadro e as seguintes não o sobrescrevem com um do meio. Gates: **10 no motor + 3 no pump + 3 no device + o arch-gate de colocação**; **16 mutações, 14 sangram** (as duas sobreviventes são cost-only e estão documentadas). ⚠️ **Uma delas sobreviveu por buraco de FIXTURE e ganhou gate:** fazer o cone atravessar arestas `delayed` passava, porque nenhuma cena tinha um `pre` cruzando de uma zona para OUTRA — que é precisamente o que aquela fronteira existe para separar (*ler o tique anterior da outra não a acopla*). **Sem item aberto: o device não recusa por substep.**<br><br>⚠️ **2026-08-19 — a convenção de MANIFESTO era guardada por um spot-check, e um nó entrou por baixo dele.** O gate `the_declaration_is_the_manifest_param_not_a_side_table` nomeava três nós e afirmava que eles não declaram — o que prova que *aqueles três* estão bem e **não diz nada** sobre os outros ~118. Quatro dias depois desta linha, a `motion.verlet_rope` ganhou um param `substeps` que é um laço dentro do `eval` dela, virou declarante sem que ninguém quisesse, e o app passou a compor as duas leis (folha 03, a linha da corda: a cauda caía **4,8× menos** do que os gates daquele crate medem, e uma zona ao lado mudava de resposta). A cura é um **CENSO** — `only_the_declared_clock_owners_offer_the_substeps_param` varre o registry inteiro e compara com uma lista fechada (`CLOCK_DECLARERS`), nomeando o infractor na mensagem. *É a diferença entre «estes três estão bem» e «ninguém mais o faz».* O 2º declarante legítimo entrou em seguida: o `motion.integrate` (folha 17), com o **mesmo teto de 64** — o ritmo é do grafo, então tetos diferentes seriam contornáveis, e há gate para isso | **omissão** | ✅ | `substeps = 1` ⇒ o laço de hoje |
| `sim.zone` | 0 | **CICLO DE VIDA** — start/delay/duration/loop: Houdini DOP `Start Frame` · Cavalry Forge **`Start Frame`** (cavalry:163) · Niagara **Emitter State** `Life Cycle Mode` · `Loop Behavior: Once\|Multiple\|Infinite` · `Loop Duration` · `Loop Delay` · `Inactive Response` ([niagara §C.13](../referencia_pesquisa_niagara_stardust.md)) | **NÃO** para a zona: `ctx.started()` é *"eu emiti algo no tick passado?"* (`prev_outputs.contains`), **não um relógio** — nada upstream adia o 1º cook dela. ⚠️ E envolver a zona num `motion.time_remap` **está cercado** (ver CERCAS). **PARCIAL por outra via:** o caso comum (*"pare de nascer depois de N s"*) é exprimível dirigindo `sim.spawn.rate` a 0 — ver a linha `sim.spawn`/duração | **omissão** | **P2** | `start = 0`, `duration = ∞` |
| `sim.zone` | 0 | Blender Simulation Input expõe **Delta Time** e **Elapsed Time** como saídas | **SIM.** `value.time` dá o playhead; `age`/`sim_t` são colunas por-elemento que `value.attribute` lê. Sem `start`, elapsed-da-zona ≡ playhead | natureza | ⛔ | — |
| `sim.zone` | 0 | **O que a zona GUARDA é uma const** (`TRANSIENTS = ["accel","falloff"]`) — o Blender faz o artista **fiar** os itens guardados pelos sockets da zona | **N/A** — não é gap de param, é a DOBRADIÇA de que todo item novo depende (ver CERCAS: uma coluna `hit`/`died` **tem** de entrar nessa const no mesmo commit) | **natureza** (a lei *"guarda estado, não rascunho"* foi paga com um bug real) | ⛔ | — |
| `sim.step` | `damping` | **SPEED LIMIT** (clamp min/max de \|v\|) — Houdini **POP Speed Limit** ([houdini:30](../referencia_pesquisa_houdini_mops.md), *"**FALTA** (barato, alto valor)"*) · Niagara **Limit Force** `Force Limit` ([niagara §C.8](../referencia_pesquisa_niagara_stardust.md), roda DEPOIS das forças e ANTES do solve) | **NÃO era — três cadeias tentadas e as três seguem refutadas.** (a) `value.attribute`(Speed) → `value.math` → `motion.drive`: **nenhum canal do drive é `vel`** (e a §0.1 corrigida acima só lhe acrescentou Falloff/HSV, nada disso é velocidade). (b) `force.drag` escala `vel`, mas o `coefficient` é **param** ⇒ um número por tick, nunca função da velocidade de CADA elemento. (c) `motion.expression` esbarra no mesmo conjunto fechado de sete escritores de `vel` | **omissão** | ✅ **CONSTRUÍDO (2026-08-11): `max_speed` + `min_speed` no `sim.step`**, pela porta única `limit_speed`. ⚠️ **NÃO é um nó novo, e a COLOCAÇÃO é a feature inteira:** o clamp roda ENTRE a atualização da velocidade e a da posição, então capa a **distância que o elemento anda no tique**; um `sim.speed_limit` a jusante caparia o número que ele *reporta* e deixaria `P` já ter avançado — um tique atrasado **por construção**, e é o tique em que o arremessado atravessa a parede. Só quem tem `vel` e `dt` na mão no meio do passo consegue capar as duas, e custa o oitavo escritor de `vel` a menos. ⚠️ **O teto vence o piso** (um piso capaz de empurrar através do teto tornaria o teto uma sugestão) e **um elemento parado não tem direção**, então o piso não o acorda — a consequência, nomeada: um piso **não** levanta o que assentou. ⚠️ **A faixa do slider é MEDIDA** (queda livre nas gravidades do corpus: 20 / 24 / 32 u/s ⇒ 0..40) e **não há `ParamHardMax`**, porque um teto maior é *menos* teto — a varredura da cena `=31` mostra teto 20 e teto 0 idênticos até o dígito. ⚠️ **E a unidade é `Length` num par que é metro por SEGUNDO**: a tabela não tem velocidade e só `Length` atravessa o `pixels_per_meter`, então o número converte exato e o RÓTULO fica uma casa grosseira — o número certo com rótulo grosso vence o rótulo certo com número errado, e o vão fica nomeado (um `ParamUnit::Speed` pede sufixo COMPOSTO, que nenhuma unidade desta tabela tem) | `0` nos dois ⇒ byte-idêntico, e a porta sai na PRIMEIRA linha sem pagar sequer a raiz |
| `sim.step` | `damping` | **ESTADO ANGULAR** (spin integrado) — Houdini **POP Spin** / **POP Torque** / **POP Drag Spin** ([houdini:19,24,25](../referencia_pesquisa_houdini_mops.md), os três **FALTA**) · Niagara `Rotational Drag` ([niagara §C.2](../referencia_pesquisa_niagara_stardust.md)) | **PARCIAL, e a metade exprimível é a que os artistas usam.** Tentei: `value.attribute("age")` → `value.math`(× taxa) → `motion.drive(channel = Rotation)` **FUNCIONA** (o drive escreve `rot`, `motion-drive:169-180`) ⇒ *"cada partícula gira à sua própria taxa constante"* já é exprimível. O que **não** é: arrasto angular, torque, e spin **alterado por uma colisão** (precisa de `vel` angular no estado) | omissão | **P2** | `spin = 0` ⇒ `rot` intocado |
| `sim.step` | `damping` | **`MAX_DT = 1/20` é literal HARDCODED** sem medição e sem controle — Cavalry expõe `Time Step` (cavalry:163) | **NÃO** (é uma const privada). É o mesmo assunto dos substeps: um limite que só diz *"scrub/frame perdido"* e não traz a tabela (**§0 do CLAUDE.md**) | omissão | **P2** | expor com default `0.05` ⇒ bit-idêntico |
| `sim.spawn` | `rate` `scatter` `seed` | **`Spawn Probability` 0..1** — Niagara Spawn Rate ([niagara §C.10](../referencia_pesquisa_niagara_stardust.md)) · Cavalry Particle Emitter **`Probability`** ([cavalry:166](../referencia_pesquisa_cavalry.md)) | **NÃO era, e a MEDIÇÃO achou um mecanismo mais fundo que o desta célula.** Ela nomeava uma tentativa — dirigir o `rate` por um valor aleatório, que não *filtra* nascimentos e sim **re-deriva a história** (`born_in` usa o `rate` de AGORA nos dois termos do `floor`) —, mas a cadeia que a metodologia desta wave exige tentar é outra: `sim.spawn → value.instance_field(Random) → motion.drive(Falloff) → motion.cull`. Ela é **TUDO-OU-NADA**: medida em quatro seeds onde o alvo era 0,5, dá **0,000 · 0,000 · 0,000 · 1,000** (sonda `measure_spawn_probability`). ⚠️ **A causa vale para o catálogo inteiro:** todo sorteio por-elemento do domínio de VALOR é chaveado no **ÍNDICE DA LINHA** (`value.instance_field` é `rand01(seed, i)`; o `value.noise` amostra `i·frequency` e é *coerente* por declaração), e **nenhum tique emite mais de um nascimento** enquanto `rate ≤ 60` — que é o próprio teto do slider (medido: 481 tiques com 0 e 119 com 1, a rate 12; 201 e 399, a rate 40) ⇒ o índice é sempre **0** e o sorteio é uma CONSTANTE por seed. O mesmo corte decidido pelo **id** dá 0,437-0,555 | **omissão** | ✅ **CONSTRUÍDO (2026-08-11): o param `probability`** — cada nascimento devido acontece se `rand01(seed, id, 11) < probability`, pela porta única `survives`. ⚠️ **O sorteio é do ID, e do id JÁ ENVOLVIDO** (pós-`% span`): é o número que o `slot` e o carimbo veem, então os três concordam sobre quem o elemento é — e é isso que faz um **scrub reproduzir a mesma população**, porque um filtro pelo índice-na-linha a renumeraria a cada rebobinada. ⚠️ **Pista de hash PRÓPRIA** (11, ao lado das do slot e do estouro): partilhada com a do `slot`, os dois sorteios devolvem o MESMO número e **todo sobrevivente cairia nas primeiras `p·n` linhas** do template. ⚠️ **Alcança as irmãs de um ESTOURO também** — a referência põe a `Probability` na tríade do burst (§C.11), e é o que faz *"de cada dez faíscas, três pegam"*. ⚠️ **E o dispositivo NÃO recua:** com `probability < 1` a janela vira ESPARSA e a identidade `saída[i] == window_first + i` — de que o kernel vivia — cai, então ele acha o `i`-ésimo SOBREVIVENTE por **varredura de posto** limitada pelo `MAX_PER_TICK`, que é o mesmo teto que o `born_in` já impõe ao span ⇒ a varredura é provadamente suficiente. Paridade CPU×GPU verde na RTX | `probability = 1` ⇒ byte-idêntico, **sem sequer avaliar o hash** |
| `sim.spawn` | idem | **BURST** (N num tempo T; periódico) — Niagara **Spawn Burst Instantaneous** (`Spawn Count`·`Spawn Time`·`Probability`, §C.11) · VFXG `Single Burst`/`Periodic Burst` (§A2) · Cavalry `Duration`·`Interval` (cavalry:166) | ⚠️ **A CÉLULA ENVELHECEU — re-conferida em 2026-08-11.** O mecanismo que ela descreve continua verdadeiro (um pulso no `rate` **desloca a régua de ids**, não injeta nascimentos) e a CONCLUSÃO deixou de ser: o `sim.spawn` ganhou a porta **`pulse`** e o param **`burst`** na wave da morte-que-semeia, então *N nascimentos num instante* é exprimível desde então — `Spawn Count` ✅ (o `burst`) e `Spawn Time` ✅ (quem pulsa decide quando). O que segue FALTANDO da tríade do Niagara é só a `Probability`, que é a linha acima | omissão (só a probabilidade) | ✅ **CONSTRUÍDO: `burst` + a porta `pulse`** | porta desconectada ⇒ byte-idêntico |
| `sim.spawn` | idem | **DURAÇÃO / envelope** (pare de nascer depois de N s) — Cavalry `Duration`·`Interval` | **SIM.** `value.time` → `value.step`/`value.curve` → dirige `rate`; `born_in` devolve `0..0` para `rate <= 0` (`sim-spawn:136`). O **envelope** é exprimível mesmo com o **burst** não sendo | natureza (o param dirigido é a resposta) | ⛔→P2 | — |
| `sim.spawn` | idem | **TETO DE POPULAÇÃO** — Cavalry **`Maximum Particles`** (cavalry:166) · Niagara emitter max. ⚠️ **O irmão já tem:** `motion.emitter` carrega o param `max` (`motion-emitter:140`); `sim.spawn` tem só `MAX_PER_TICK = 256`, que é por TICK | **NÃO.** Tentei `motion.cull` no modo Fraction: ele mantém `amount·n` — uma **fração**, não uma contagem —, então ele **rala a população inteira** em vez de capar, e só mataria "os mais velhos primeiro" se alguém ordenasse antes. Não é equivalente. Uma zona com spawn e sem lifetime cresce sem limite | **omissão** | ✅ **CONSTRUÍDO (2026-08-11): `motion.cull` no modo `Max Count`** — ⚠️ **e o teto NÃO é do `sim.spawn`, o que não é escolha de gosto:** aquele nó tem duas portas (`template` e `pulse`) e **não pode ver a população**, que é o estado da ZONA; o próprio doc dele declara que não funde nada (*"`motion.combine` does that, and that is the whole design"*). Um `max` no nascimento precisaria de uma terceira porta trazendo o laço que ele alimenta ⇒ **o teto é propriedade da POPULAÇÃO, então mora no nó que a vê**. ⚠️ **Ele guarda os mais NOVOS, pela lei que o irmão que esta linha cita já traz ESCRITA** (`motion.emitter`: *"the cap keeps the NEWEST particles … not a frozen ancient cloud"*) — e a premissa foi MEDIDA em vez de assumida: numa zona o `motion.combine` apende os recém-nascidos ao estado, logo **o prefixo é o mais VELHO**, e guardar o prefixo seria exatamente a nuvem congelada que o emitter recusa. ⚠️ **Uma FRAÇÃO não vira um teto**, e o gate o pina: o MESMO grafo em `Fraction 0.5` **plateia em 2**, porque uma fração persegue um alvo móvel. Medido na cadeia real: sem teto a zona passa de 100 em 600 tiques · com `max = 20` ela **ENCHE e para em 20** (o pico E o fim), e o menor id **ANDA** a cada nascimento. O `amount` e o `max` são params SEPARADOS com `ParamGate` (um número por modo — o valor da porta de VALOR segue vencendo os dois), e o teto é `ParamUnit::Count` | `max` só existe no modo novo ⇒ byte-idêntico |
| `sim.spawn` | idem | **Velocidade/direção inicial** — Niagara `Add Velocity in Cone` (§C.16) · Cavalry `Initial Direction Type (Angle/Inwards/Outwards)` + `Initial Speed` (cavalry:166) | ⛔ **ESTE VEREDITO FOI REFUTADO em 2026-08-10, pelo smoke da cena `=27`** (*"não se dividem, as filhas ficam juntas como uma só"*). Ele vale para o nascimento por **TAXA** — cada filho pega uma LINHA distinta do template, logo já nasce separado — e é **falso** para o nascimento por **PULSO**, onde `burst` filhos saem da **MESMA** linha e herdam `P` e `vel` idênticos. Esta linha foi escrita antes de a porta `pulse` existir. ⚠️ **E não era afinação: era impossibilidade** — toda força do catálogo é função da POSIÇÃO, então `curl(P)` dá a duas partículas no mesmo ponto a MESMA aceleração, e duas irmãs assim ficam bit-idênticas para sempre (medido: 150 tiques). A simetria só quebra no NASCIMENTO, pela única coisa que difere entre irmãs — o id | omissão (só no pulso) | ✅ **CONSTRUÍDO: o param `burst_speed`** do `sim.spawn` — impulso aditivo à velocidade herdada, direção sorteada da identidade da própria filha, **só nos pulse-born** (é o que o mantém fora do caminho de GPU, que o pulso já recusa por declaração) | `burst_speed = 0` ⇒ byte-idêntico |
| `sim.spawn` | idem | `rate` tem `ParamUiHint.max = 60` e **nenhum `ParamHardMax`**, enquanto o kernel honra até 256/tick (≈15 360/s a 60 fps) | **N/A** — é a lei do slider dual do [doc 88](../88_plano_parametros_nos_unidades_e_slider.md): *a faixa confortável e o teto disfuncional são dois números*, e aqui só existe um | omissão (lei de param) | **P2** | `ParamHardMax` medido; `ParamUiHint` intocado ⇒ arrasto idêntico |
| `sim.lifetime` | `life` `variance` `seed` | **EVENTO DE MORTE → spawn filho** — VFXG **`Trigger Event On Die`** → Initialize do sistema filho (§A2) · Niagara **Death Event / GPU events** (§A1) · Stardust **Aux** (§A3) · Houdini **POP Replicate** ([houdini:37](../referencia_pesquisa_houdini_mops.md), **FALTA**) · **doc 63 linha 97 = `sim.replicate`, P0** | **NÃO.** `reap` constrói a saída **só a partir de `keep`** (`sim-lifetime:123-131`): as linhas mortas são descartadas e nada a jusante as enxerga. `pulse.*` é canal **escalar por tick**, não um conjunto por elemento; `motion.combine` funde streams, mas **não existe stream de mortos** para fundir | **omissão** | ✅ **CONSTRUÍDO (2026-08-10): as saídas `died` + `pulse`** — e o achado é que **`sim.replicate` NÃO É UM NÓ**: ele é a FIAÇÃO das duas nas duas portas que o `sim.spawn` ganhou no mesmo dia (`died → template`, `pulse → pulse`). ⚠️ **São DUAS saídas para um evento só porque o SISTEMA DE TIPOS as separa** — `connects_directly` exige domínio+dim+relógio iguais, e a carga (`Instances/Vec2/Frame`) não cabe no mesmo fio que o gatilho (`Instances/Scalar/Event`); é a divisão *payload × trigger* que a referência faz, aqui verificada pelo compilador. Alinhadas por índice **por construção** (as duas saem da mesma lista `gone`). ⚠️ **E a wave arrastou DUAS correções que ela tornou load-bearing:** (a) *um recém-nascido tem idade ZERO por definição* — o `newborns` herdava `age` do template, e um filho de cadáver nasce **passado da própria vida** e morre no tique seguinte (a lei já estava escrita no `sim.step`: *a row with no `age` is newborn*); (b) *um estágio de GPU produz UM buffer* — o `eligible` não perguntava por porta de saída, e um consumidor da porta 1 receberia o buffer da porta **0** em silêncio (nunca apareceu porque o único nó de duas saídas, o `carry`, não tem kernel; a recusa mora no PLANEJADOR para o próximo nascer coberto) | porta desconectada ⇒ byte-idêntico |
| `sim.lifetime` | idem | Niagara expõe **`Particles.Lifetime`** ao lado de `NormalizedAge` (§C.14) — o span por-elemento | **SIM.** `span = age / life` via `value.attribute(age)` ÷ `value.attribute(life)` no `value.math` (indefinido em `life = 0`, i.e. no instante do nascimento) | natureza | ⛔ | — |
| `sim.lifetime` | idem | `Lifetime Mode: Direct \| Random(min,max)` — Niagara §C.14 | **SIM** — nominal ± fração cobre exatamente `[life(1−v), life(1+v)]`: mesmo conjunto, outra face | natureza | ⛔ | — |
| `sim.lifetime` | idem | **MATAR POR LUGAR** (kill volume, com invert) — Niagara/VFXG **`Kill (AABox/Sphere/Plane)`** (§A2/B) · **doc 63 linha 96 = `sim.kill_zone`, P1** | **SIM — e isto CORRIGE o doc 63.** `motion.falloff` (Circle/Rect/Linear, tem `invert`) **ou** `field.box` (caixa **rotacionada**, com gizmo de canvas) escrevem `falloff` → `motion.cull` no modo Falloff com `invert` mata por ele, e **dentro da zona um cull é uma morte definitiva**. Dois nós que já existem | natureza (composição) | ⛔→**P2** (ergonomia: 1 knob × 2 nós) | — |
| `sim.collide` | 7 | **RAIO DA PARTÍCULA** — Niagara Collision **`Radius Calculation`** (auto do sprite size + scale, [§C.17](../referencia_pesquisa_niagara_stardust.md)) · VFXG `Collide (…)` colide um raio (§A2) | **NÃO era.** Colidíamos um **PONTO** (`p[1] < height`), então um sprite de qualquer tamanho **afundava até a metade** no chão — medido, um quad 1×1 num chão em `y = −2` descansava com a borda de baixo em **−2,5**. Compensar por `height` só funciona com tamanho **uniforme**: `size` é coluna por elemento e `height` é param | **omissão** | ✅ **CONSTRUÍDO (2026-08-10): a INFLAÇÃO DE MINKOWSKI** — o ponto colide contra a mesma forma *crescida por `r`*: um termo por forma (`p.y − r` · `radius + r` · `radius − r` clampado em 0) e o `respond` **intocado**, então a resposta segue com uma implementação só. ⚠️ **De onde `r` vem é `particle_radius`, a PORTA ÚNICA** que a `eval` e o WGSL perguntam — duas respostas a *"quão grande é esta partícula?"* divergiriam no primeiro arrasto de slider, e a divergência seria um sprite repousando a alturas diferentes nos dois caminhos. Três modos (`radius_from`): **Point** (o default) · **Fixed** · **Sprite Size**. ⚠️ **O inscrito, `min(\|w\|,\|h\|)·0.5`, e a metade não é convenção** — o `sprite.wgsl` expande um quad unitário em `[-0.5, 0.5]` por `size`; inscrito e não circunscrito porque um círculo que **sai** do sprite o faz pairar com um vão que o artista vê e não corrige editando a arte, enquanto um dentro dele no pior caso deixa uma quina passar — que é o que um colisor redondo sob um sprite quadrado sempre faz (`size_scale` alcança o circunscrito). ⚠️ **`size` AUSENTE lê `[1,1]` nos dois caminhos** — `SIZE_IDENTITY`, que é *também* o `default_size` do shell, logo literalmente o quad que o renderizador desenha ali: **um número, não dois** | `radius_from = Point` ⇒ o ponto de hoje, byte-idêntico |
| `sim.collide` | 7 | **PLANO NÃO-HORIZONTAL** (parede, rampa) — Niagara `Collision Mode: **Analytical Planes**` (§C.17) · VFXG `Plane` (§A2) | **NÃO era — duas cadeias tentadas, as duas falham com mecanismo, e as duas seguem REFUTADAS.** (a) *girar o mundo → colidir → desgirar*: `motion.rotate` **não gira `P`**, ele escreve só a coluna `rot` (`motion-rotate:100-117`) — e ainda que girasse `P`, não giraria `vel`, então a reflexão seria contra o frame errado. (b) *encadear colisores*: encadear **funciona** (cada um é Pure e lê/escreve `P`/`vel`), mas todo Floor era horizontal ⇒ a cadeia constrói uma **escada**, nunca uma rampa | **omissão** | ✅ **CONSTRUÍDO (2026-08-10): a forma carrega a PRÓPRIA orientação** — nenhuma das duas cadeias podia produzir uma rampa, então o `shape` deixou de ser horizontal por definição e virou a meia-reta analítica canônica, na **forma de HESSE**: `mundo = { p : dot(p, n) >= offset }`, com `n` = o vetor "para cima" girado por `angle`. **Uma parede e um teto vêm de graça** (90° e 180°, a mesma meia-reta virada), e o rótulo do enum passou de "Floor" a **"Plane"** porque uma forma que pode ser um teto não pode chamar-se chão. ⚠️ **`offset` é distância à ORIGEM ao longo da normal, não uma coordenada `y`, e é ESSA escolha que torna a parede exprimível:** a alternativa óbvia — *"o plano pivota em torno de `(0, height)`"* — lê melhor numa rampa rasa e **prende toda parede em `x = 0` para sempre**, porque a 90° o pivô está SOBRE o plano e o botão desliza a parede ao longo de si mesma; o rótulo virou **"Offset"** (o NOME do param fica `height`: nome viaja em todo grafo salvo). ⚠️ **A normal sai da MESMA senoide parabólica corrigida do `field.box`/`force.wind`** — copiada por-crate (a convenção de drop-crate) e portada verbatim para o `wgsl_lib`, porque um `sin()` de WGSL é uma segunda resposta correta num último bit diferente por driver; **exata nos quatro quartos de volta**, e é isso que faz de `angle = 0` uma BYTE-identidade (`dot(p, (0,1))` **é** `p.y`) em vez de uma tolerância. ⚠️ **O `sqrt` de normalização não é decoração:** o polinômio fica ~0,09% fora do círculo unitário e `depth` **e** a reflexão são medidos ao longo da normal — um `n` 1% curto é um plano em que a partícula afunda 1%; **sem zero-guard de propósito**, licenciado pelo `stays_near_unit_circle` do próprio `trig.rs`, porque um ramo que nunca dispara é uma defesa que gate nenhum consegue provar. ⚠️ **O tilt não alcança Disc nem Bowl, e isso é GEOMETRIA:** os dois são rotacionalmente simétricos ⇒ um ângulo neles seria provadamente um knob que não muda nada, e ele carrega `ParamGate` em vez de comentário | `angle = 0°` ⇒ a normal `(0,1)` de hoje, **bit-idêntica** |
| `sim.collide` | 7 | **EVENTO DE CONTATO / atributo `hit`** — Cavalry Forge **`Collision Events`** (Color/Impulse/**Sticky**/Visibility, [cavalry:163](../referencia_pesquisa_cavalry.md)) · Houdini **POP Collision Detect** (marca grupo, [houdini:39](../referencia_pesquisa_houdini_mops.md)) · Niagara collision events · **doc 63 linha 98 = `sim.collision_pulse`, P1** | **NÃO era.** Nada observável mudava num toque que um nó a jusante pudesse ler: `P` e `vel` mudam, mas mudam **todo tick** por causa do step ⇒ *"tocou?"* era inexprimível | **omissão** | ✅ **CONSTRUÍDO (2026-08-11): a coluna `hit`** — *quão fundo a colisão deste tique empurrou o elemento de volta*, em unidades de mundo, e `0` onde nada tocou. ⚠️ **UM número carrega os DOIS fatos sem que possam discordar:** contato é `depth > 0` **por construção** (o teste de cada forma devolve `Some` exatamente quando a profundidade que ele calcula é estritamente positiva), então não existem um flag e uma magnitude para reconciliar. ⚠️ **Ela ACUMULA por `max` ao longo do tique e a zona a ESTRIPA** — a forma exata do `accel`, pelas duas mesmas razões: colisores ENCADEIAM (a cena `=29` empilha rampa e parede), e uma escrita simples significaria *"o último colisor me tocou?"*, um fato sobre a ordem em que o artista ligou os fios; e guardada no estado ela diria "tocou" no tique seguinte ao que parou de tocar. ⚠️ **A binding é `ReadWrite`, e isso NÃO é convenção:** com `Write` o codegen não emite `read_hit` e **o kernel deixa de compilar** (`no definition in scope for identifier: read_hit`) — a lei de acumulação é recusada pelo compilador, não por um gate. O 9º canal do `READ_CHANNELS` (`value.attribute(Hit)`) é o que a leva ao domínio de VALOR, e com ele *"morrer ao tocar"* (a linha 98 do doc 63) é a cadeia `hit → drive(Falloff) → cull`, **sem um nó novo** | coluna ausente ⇒ byte-idêntico |
| `sim.collide` | 7 | **COMPORTAMENTOS nomeados: die / stick / slide** — Houdini **POP Collision Behavior** ([houdini:39](../referencia_pesquisa_houdini_mops.md), *"sem os 4 comportamentos nomeados"*) · VFXG collide com **lifeloss** (§A2) | **EXPRIMÍVEL, e o item DISSOLVEU com o `hit`.** *stick* é `restitution 0 + friction 1` e *slide* é `restitution 0 + friction 0` — os dois knobs já estão na tela, e um enum `behaviour` ao lado deles seria uma **segunda porta** para o par que já responde; *die* e *lifeloss* dependiam de haver o que observar, e agora há: `value.attribute(Hit) → motion.drive(Falloff) → motion.cull` **dentro da zona** é a morte no contato. ⚠️ Um `behaviour = Bounce\|Stick\|Slide` shipa três nomes para dois números que o artista já tem — a definição de knob morto desta casa | natureza (composição) | ⛔ | — |
| `sim.collide` | 7 | **Mais formas** (box, segmento, SDF, forma vetorial) — VFXG `Sphere/AABox/Cylinder/Plane/SDF` (§A2) | **PARCIAL:** encadear colisores dá a **união** das formas que existem; uma **caixa rotacionada** cai de graça assim que o Floor puder inclinar (item acima) | omissão | **P2** (depois do ângulo) | variante nova no enum ⇒ `shape` intocado |
| `sim.collide` | 7 | **`Restitution Randomness`** (por elemento) — Niagara §C.17 | **NÃO** (o kernel não faz leitura por-elemento de um jitter; `id` está lá, mas o hash não) | omissão | **P2** | `randomness = 0` ⇒ bit-idêntico |
| `sim.collide` | 7→10 | **Nenhuma `ParamSection`** num nó de 7 params — o [doc 88](../88_plano_parametros_nos_unidades_e_slider.md) deu seções a 10 nós (*a parede de sliders vira três perguntas*) | **N/A** — lei de param. O corte natural é **Forma** (`shape`/`height`/`center_*`/`radius`) × **Resposta** (`restitution`/`friction`) | omissão (lei de param) | **P2**, e a metade que MORDIA caiu de carona no P0 do raio (2026-08-10): os **quatro knobs mortos** ganharam `ParamGate` — `height` só aparece no Floor, `center_*`/`radius` só no Disc/Bowl, e os dois números de raio só no modo que os lê. Um `Height` que o kernel de um Disc nunca olha é o knob-morto que esta casa recusa; o que resta em aberto é a AGRUPAÇÃO visual, que é cosmética | seção é metadado ⇒ zero efeito no cozido |

**Contagem (DERIVADA, reconciliada no grupo M em 2026-08-16):** 24 linhas — **P0 = 0** · **P1 = 0** · **P2 = 7** · ✅ fechadas **10** · ⛔ recusadas/refutadas **7**. ⚠️ **esta linha dizia

Re-medir: `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` — ⚠️ **esta linha é DERIVADA da coluna `P` da tabela acima; não a edite à mão** (a contagem desta conferência envelheceu SEIS vezes, e a folha 13 chegou a contradizer a própria prosa três parágrafos abaixo).
`3 P0 · 7 P1` até 2026-08-16, e a própria prosa abaixo dela já dizia o contrário**
(*"a folha 13 não tem item aberto"*): os três P0 fecharam com o W7 e os sete P1 nas
varreduras de 08-11 e 08-12, e **ninguém reconferiu o CABEÇALHO**. É a quinta vez que uma
linha de contagem desta conferência envelhece — e a mais barata de detectar, porque ela
contradiz a tabela que está três parágrafos acima. ⚠️ **e a varredura de
2026-08-11 fechou mais TRÊS P1 e derrubou DUAS células por envelhecimento**: o **evento de
contato** virou a coluna `hit` (construída), os **comportamentos nomeados** dissolveram nela
(dois knobs que já existem, e o *die* virou composição), e o **BURST** já estava construído desde
a porta `pulse` sem ninguém reconferir a nota. E o ÚLTIMO P1 — **substeps** — fechou em 2026-08-12:
ele nunca foi *"inexprimível"*, era **escopo** (subdividir o playhead JÁ substepa; o que faltava era
fazê-lo por-ILHA em vez de cobrar do grafo inteiro), e a linha 59 traz o mecanismo, os três fatos de
relógio e a razão de o device ter deixado de recusar. **A folha 13 não tem item aberto.** O **teto de
população** fechou como um modo do `motion.cull` (o teto é da POPULAÇÃO, e o `sim.spawn`
estruturalmente não a vê), a **probabilidade de nascimento** fechou na mesma varredura (a linha acima), e o que
ela deixa para o catálogo é maior que o param: **nenhum sorteio por-elemento do domínio de VALOR
é chaveado na IDENTIDADE**, todos no índice da linha, então *"decida por elemento e reproduza no
scrub"* só é exprimível dentro do nó que conhece o id — o **speed limit** fechou na mesma varredura, e com ele a §0.1
ganhou a metade que ainda vale: nenhuma cadeia do catálogo escreve `vel` por elemento, então a
operação teve de morar no único nó que já tem a velocidade e o `dt` na mão. ⚠️ **e os TRÊS P0 estão
CONSTRUÍDOS** (o evento de morte · o raio da partícula · o plano não-horizontal, os três em
2026-08-10). Sobram 2 linhas que são constatação de desenho e não gap. ⚠️ **A folha permanece o
registro do que foi TENTADO** — as cadeias refutadas continuam escritas nas células, porque uma
composição que não funciona é a razão de a capacidade ter virado kernel, e apagá-la faria a
próxima LLM re-tentar.

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
coluna `hit` (**CONSTRUÍDA em 2026-08-11**) é lida pelo domínio de valor, e por ser função do
tick, *um flash de cor no impacto re-toca idêntico quando o artista volta a régua por cima dele*.
É a diferença entre um **preview** e um efeito **editável**.

⚠️ **E a construção acrescentou um preço que esta seção não previa, medido:** a cadeia GPU-residente
é `value.attribute(Hit) → motion.drive(<canal>)` — a cena `=30` sai **FULLY GPU, 8 estágios**, com o
canal de simulação indo ao domínio de VALOR e voltando *dentro do laço*. Consumi-lo com a família
**`pulse.*`** funciona e custa a residência: ela é CPU-only (a folha 12 o mede), e um nó sem kernel
no interior faz o ADR-0135 D3 devolver a zona inteira ao pump. *A colisão pulsa; escolher o
consumidor é escolher onde o laço roda.*

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
   proxies FALTA)".** Subestima: o que faltava primeiro **não era variedade de proxy** — era o
   **RAIO** (colidíamos um ponto, e o sprite afundava até a metade) e a **ORIENTAÇÃO do plano**
   (o nosso chão não inclinava). Os dois aparecem na primeira cena, antes de qualquer proxy
   exótico, e **os dois fecharam em 2026-08-10**. ⚠️ E a variedade de proxy que o doc 63 pedia
   ficou **menor do que ele supunha**: uma parede, um teto e uma rampa não são formas novas, são
   a MESMA meia-reta com um ângulo — o que resta de verdade são caixa, segmento, SDF e forma
   vetorial.
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
| **Simulation Stages** (N passes nomeados por tick) | — | ❌ **FALTA** = a generalização dos **substeps**. ⚠️ **A palavra "inexprimível" foi RECONFERIDA e caiu pela metade** (§0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*): encadear `sim.step` segue sendo no-op exato (`sim_t` já vale `playhead`) — mas o **substep** é exprimível hoje, pelo RELÓGIO, e está medido na linha 59. O que continua faltando é a outra metade: N passes **NOMEADOS** e distintos por tique, que é uma pergunta de escopo do cook, não de aritmética de `dt` |
| **Render / Output** | `motion.output` + o lowering para instâncias | ✅ **TEMOS** |

**Placar: 4 estágios cobertos, 2 ausentes (EVENTS · CICLO DE VIDA) e 1 sub-estágio ausente
(multi-pass/substep).** Os dois ausentes são o mesmo item da tabela visto de cima — e o EVENTS é
onde o §8 do plano manda procurar o SUPERAR, porque é o único lugar em que o nosso scrub bit-exato
resolve um problema que as referências têm e não conseguem resolver.
