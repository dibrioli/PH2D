# 89 · CONFERÊNCIA — Família 13: `sim.*` (o STACK) — 5 nós

> História verbatim: [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/13_sim_stack.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| `sim.zone` — Blender Simulation Input expõe Delta Time e Elapsed Time como saídas |  | vivo, tabela |
| `sim.zone` — O que a zona GUARDA é uma const (`TRANSIENTS = ["accel","falloff"]`) — o Blende… |  | vivo, tabela |
| `sim.spawn` — DURAÇÃO / envelope (pare de nascer depois de N s) — Cavalry Duration·Interva… | →~~P2~~ ⛔ REFUTADO 2026-08-26 — corrido, com controlo | vivo, tabela |
| `sim.lifetime` — Niagara expõe `Particles.Lifetime` ao lado de `NormalizedAge` (§C.14) — o span… |  | vivo, tabela |
| `sim.lifetime` — `Lifetime Mode: Direct \/ Random(min,max)` — Niagara §C.14 |  | vivo, tabela |
| `sim.lifetime` — MATAR POR LUGAR (kill volume, com invert) — Niagara/VFXG Kill (AABox/Sphere/Pl… | →~~P2~~ ⛔ REFUTADO 2026-08-26 — corrido | vivo, tabela |
| `sim.collide` — COMPORTAMENTOS nomeados: die / stick / slide — Houdini POP Collision Behavior (… |  | vivo, tabela |
| _Linhas ✅ (fechadas) da tabela_ — arquivadas quando o doc passou dos 40 KB | O placar e o gate `conferencia()` leem as linhas VIVAS (≥ 1 por nó); as fechadas estão verbatim no arquivo | [arquivo](../../archive/motion-nodes-2026-10-12/89_conferencia/13_sim_stack.md) |

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
| `sim.zone` | 0 | Blender Simulation Input expõe **Delta Time** e **Elapsed Time** como saídas | **SIM.** `value.time` dá o playhead; `age`/`sim_t` são colunas por-elemento que `value.attribute` lê. Sem `start`, elapsed-da-zona ≡ playhead | natureza | ⛔ | — |
| `sim.zone` | 0 | **O que a zona GUARDA é uma const** (`TRANSIENTS = ["accel","falloff"]`) — o Blender faz o artista **fiar** os itens guardados pelos sockets da zona | **N/A** — não é gap de param, é a DOBRADIÇA de que todo item novo depende (ver CERCAS: uma coluna `hit`/`died` **tem** de entrar nessa const no mesmo commit) | **natureza** (a lei *"guarda estado, não rascunho"* foi paga com um bug real) | ⛔ | — |
| `sim.step` | `damping` | **`MAX_DT = 1/20` é literal HARDCODED** sem medição e sem controle — Cavalry expõe `Time Step` (cavalry:163) | **NÃO** (é uma const privada). É o mesmo assunto dos substeps: um limite que só diz *"scrub/frame perdido"* e não traz a tabela (**§0 do CLAUDE.md**) | omissão | ~~P2~~ ✅ **FECHADO 2026-08-23** | expor com default `0.05` ⇒ bit-idêntico ✅ **MEDIDO (bloco Z, [doc 91](../91_os_tetos_que_ninguem_mediu.md) §5).** ⚠️ **Este era o número MAIS CERTO do catálogo** — as duas malhas medem igual até 1/30, e a `0,1` do `motion.integrate` deixava a mesma cena chegar a **127** onde esta a segurava em **2,49**. Passa ao joelho medido, `0,03`. ⚠️ **Não foi exposto como param**, e a razão é a medição: o número não é uma preferência do artista, é onde o solver deixa de dar a mesma resposta. |
| `sim.spawn` | idem | **BURST** (N num tempo T; periódico) — Niagara **Spawn Burst Instantaneous** (`Spawn Count`·`Spawn Time`·`Probability`, §C.11) · VFXG `Single Burst`/`Periodic Burst` (§A2) · Cavalry `Duration`·`Interval` (cavalry:166) | ⚠️ **A CÉLULA ENVELHECEU — re-conferida em 2026-08-11.** O mecanismo que ela descreve continua verdadeiro (um pulso no `rate` **desloca a régua de ids**, não injeta nascimentos) e a CONCLUSÃO deixou de ser: o `sim.spawn` ganhou a porta **`pulse`** e o param **`burst`** na wave da morte-que-semeia, então *N nascimentos num instante* é exprimível desde então — `Spawn Count` ✅ (o `burst`) e `Spawn Time` ✅ (quem pulsa decide quando). O que segue FALTANDO da tríade do Niagara é só a `Probability`, que é a linha acima | omissão (só a probabilidade) | ✅ **CONSTRUÍDO: `burst` + a porta `pulse`** | porta desconectada ⇒ byte-idêntico |
| `sim.spawn` | idem | **DURAÇÃO / envelope** (pare de nascer depois de N s) — Cavalry `Duration`·`Interval` | **SIM.** `value.time` → `value.step`/`value.curve` → dirige `rate`; `born_in` devolve `0..0` para `rate <= 0` (`sim-spawn:136`). O **envelope** é exprimível mesmo com o **burst** não sendo | natureza (o param dirigido é a resposta) | ⛔→~~P2~~ ⛔ **REFUTADO 2026-08-26 — corrido, com controlo** | MEDIDO (`measure_envelope_and_kill_volume`): `value.time → value.step(invert) → value.map_range(0..1 → 0..rate)` a dirigir o `rate`. Com corte a `2 s`: **59 nascidos antes, `0` depois**. O CONTROLO (sem envelope): **59 antes, 90 depois** — a emissão continua. ⭐ **E o «antes» é o MESMO número nos dois**, que era precisamente o medo desta família: o `born_in` usa o `rate` de AGORA nos dois termos do `floor`, e foi isso que refutou a cadeia da *probabilidade* na célula vizinha — aqui ele **não** reescreve o passado. ⚠️ **A 1.ª versão da sonda mediu a MINHA cadeia, duas vezes:** lia a contagem de UM tique como se fosse acumulado (o `sim.spawn` emite só os recém-nascidos), e o `value.math(Mul)` tinha a 2.ª entrada **solta**, que vale `0` — a taxa dirigida era zero desde o instante zero. *Foi o CONTROLO que denunciou, porque ele tem de emitir.* |
| `sim.spawn` | idem | **Velocidade/direção inicial** — Niagara `Add Velocity in Cone` (§C.16) · Cavalry `Initial Direction Type (Angle/Inwards/Outwards)` + `Initial Speed` (cavalry:166) | ⛔ **ESTE VEREDITO FOI REFUTADO em 2026-08-10, pelo smoke da cena `=27`** (*"não se dividem, as filhas ficam juntas como uma só"*). Ele vale para o nascimento por **TAXA** — cada filho pega uma LINHA distinta do template, logo já nasce separado — e é **falso** para o nascimento por **PULSO**, onde `burst` filhos saem da **MESMA** linha e herdam `P` e `vel` idênticos. Esta linha foi escrita antes de a porta `pulse` existir. ⚠️ **E não era afinação: era impossibilidade** — toda força do catálogo é função da POSIÇÃO, então `curl(P)` dá a duas partículas no mesmo ponto a MESMA aceleração, e duas irmãs assim ficam bit-idênticas para sempre (medido: 150 tiques). A simetria só quebra no NASCIMENTO, pela única coisa que difere entre irmãs — o id | omissão (só no pulso) | ✅ **CONSTRUÍDO: o param `burst_speed`** do `sim.spawn` — impulso aditivo à velocidade herdada, direção sorteada da identidade da própria filha, **só nos pulse-born** (é o que o mantém fora do caminho de GPU, que o pulso já recusa por declaração) | `burst_speed = 0` ⇒ byte-idêntico |
| `sim.spawn` | idem | `rate` tem `ParamUiHint.max = 60` e **nenhum `ParamHardMax`**, enquanto o kernel honra até 256/tick (≈15 360/s a 60 fps) | **N/A** — é a lei do slider dual do [doc 88](../88_plano_parametros_nos_unidades_e_slider.md): *a faixa confortável e o teto disfuncional são dois números*, e aqui só existe um | omissão (lei de param) | ~~P2~~ ✅ **FECHADO 2026-08-23** | `ParamHardMax` medido; `ParamUiHint` intocado ⇒ arrasto idêntico ✅ **`ParamHardMax = 15 360/s`, MEDIDO pela porta do produto** (bloco Z, [doc 91](../91_os_tetos_que_ninguem_mediu.md) §3): `MAX_PER_TICK × DEFAULT_HZ`. A `0,25 s` no teto o nó entrega os **3 840** devidos; **ao dobro entrega o mesmo número** — o excedente é perdido, não adiado, e é isso que prova que o teto é o certo. O `ParamUiHint` fica intocado. |
| `sim.lifetime` | idem | Niagara expõe **`Particles.Lifetime`** ao lado de `NormalizedAge` (§C.14) — o span por-elemento | **SIM.** `span = age / life` via `value.attribute(age)` ÷ `value.attribute(life)` no `value.math` (indefinido em `life = 0`, i.e. no instante do nascimento) | natureza | ⛔ | — |
| `sim.lifetime` | idem | `Lifetime Mode: Direct \| Random(min,max)` — Niagara §C.14 | **SIM** — nominal ± fração cobre exatamente `[life(1−v), life(1+v)]`: mesmo conjunto, outra face | natureza | ⛔ | — |
| `sim.lifetime` | idem | **MATAR POR LUGAR** (kill volume, com invert) — Niagara/VFXG **`Kill (AABox/Sphere/Plane)`** (§A2/B) · **doc 63 linha 96 = `sim.kill_zone`, P1** | **SIM — e isto CORRIGE o doc 63.** `motion.falloff` (Circle/Rect/Linear, tem `invert`) **ou** `field.box` (caixa **rotacionada**, com gizmo de canvas) escrevem `falloff` → `motion.cull` no modo Falloff com `invert` mata por ele, e **dentro da zona um cull é uma morte definitiva**. Dois nós que já existem | natureza (composição) | ⛔→~~**P2**~~ ⛔ **REFUTADO 2026-08-26 — corrido** | MEDIDO (`measure_envelope_and_kill_volume`): grelha 11×11 (121 elementos), `motion.falloff(Circle, invert)` → `motion.cull(Falloff)`. Sobreviventes por raio: `0 → 121` · `1,5 → 112` · `3 → 96` · `9 → 0`. Matar por lugar existe em **dois nós que já existem**, e isto fecha também a linha 96 do doc 63 (`sim.kill_zone`), que a marcava `P1`. |
| `sim.collide` | 7 | **RAIO DA PARTÍCULA** — Niagara Collision **`Radius Calculation`** (auto do sprite size + scale, [§C.17](../referencia_pesquisa_niagara_stardust.md)) · VFXG `Collide (…)` colide um raio (§A2) | **NÃO era.** Colidíamos um **PONTO** (`p[1] < height`), então um sprite de qualquer tamanho **afundava até a metade** no chão — medido, um quad 1×1 num chão em `y = −2` descansava com a borda de baixo em **−2,5**. Compensar por `height` só funciona com tamanho **uniforme**: `size` é coluna por elemento e `height` é param | **omissão** | ✅ **CONSTRUÍDO (2026-08-10): a INFLAÇÃO DE MINKOWSKI** — o ponto colide contra a mesma forma *crescida por `r`*: um termo por forma (`p.y − r` · `radius + r` · `radius − r` clampado em 0) e o `respond` **intocado**, então a resposta segue com uma implementação só. ⚠️ **De onde `r` vem é `particle_radius`, a PORTA ÚNICA** que a `eval` e o WGSL perguntam — duas respostas a *"quão grande é esta partícula?"* divergiriam no primeiro arrasto de slider, e a divergência seria um sprite repousando a alturas diferentes nos dois caminhos. Três modos (`radius_from`): **Point** (o default) · **Fixed** · **Sprite Size**. ⚠️ **O inscrito, `min(\|w\|,\|h\|)·0.5`, e a metade não é convenção** — o `sprite.wgsl` expande um quad unitário em `[-0.5, 0.5]` por `size`; inscrito e não circunscrito porque um círculo que **sai** do sprite o faz pairar com um vão que o artista vê e não corrige editando a arte, enquanto um dentro dele no pior caso deixa uma quina passar — que é o que um colisor redondo sob um sprite quadrado sempre faz (`size_scale` alcança o circunscrito). ⚠️ **`size` AUSENTE lê `[1,1]` nos dois caminhos** — `SIZE_IDENTITY`, que é *também* o `default_size` do shell, logo literalmente o quad que o renderizador desenha ali: **um número, não dois** | `radius_from = Point` ⇒ o ponto de hoje, byte-idêntico |
| `sim.collide` | 7 | **EVENTO DE CONTATO / atributo `hit`** — Cavalry Forge **`Collision Events`** (Color/Impulse/**Sticky**/Visibility, [cavalry:163](../referencia_pesquisa_cavalry.md)) · Houdini **POP Collision Detect** (marca grupo, [houdini:39](../referencia_pesquisa_houdini_mops.md)) · Niagara collision events · **doc 63 linha 98 = `sim.collision_pulse`, P1** | **NÃO era.** Nada observável mudava num toque que um nó a jusante pudesse ler: `P` e `vel` mudam, mas mudam **todo tick** por causa do step ⇒ *"tocou?"* era inexprimível | **omissão** | ✅ **CONSTRUÍDO (2026-08-11): a coluna `hit`** — *quão fundo a colisão deste tique empurrou o elemento de volta*, em unidades de mundo, e `0` onde nada tocou. ⚠️ **UM número carrega os DOIS fatos sem que possam discordar:** contato é `depth > 0` **por construção** (o teste de cada forma devolve `Some` exatamente quando a profundidade que ele calcula é estritamente positiva), então não existem um flag e uma magnitude para reconciliar. ⚠️ **Ela ACUMULA por `max` ao longo do tique e a zona a ESTRIPA** — a forma exata do `accel`, pelas duas mesmas razões: colisores ENCADEIAM (a cena `=29` empilha rampa e parede), e uma escrita simples significaria *"o último colisor me tocou?"*, um fato sobre a ordem em que o artista ligou os fios; e guardada no estado ela diria "tocou" no tique seguinte ao que parou de tocar. ⚠️ **A binding é `ReadWrite`, e isso NÃO é convenção:** com `Write` o codegen não emite `read_hit` e **o kernel deixa de compilar** (`no definition in scope for identifier: read_hit`) — a lei de acumulação é recusada pelo compilador, não por um gate. O 9º canal do `READ_CHANNELS` (`value.attribute(Hit)`) é o que a leva ao domínio de VALOR, e com ele *"morrer ao tocar"* (a linha 98 do doc 63) é a cadeia `hit → drive(Falloff) → cull`, **sem um nó novo** | coluna ausente ⇒ byte-idêntico |
| `sim.collide` | 7 | **COMPORTAMENTOS nomeados: die / stick / slide** — Houdini **POP Collision Behavior** ([houdini:39](../referencia_pesquisa_houdini_mops.md), *"sem os 4 comportamentos nomeados"*) · VFXG collide com **lifeloss** (§A2) | **EXPRIMÍVEL, e o item DISSOLVEU com o `hit`.** *stick* é `restitution 0 + friction 1` e *slide* é `restitution 0 + friction 0` — os dois knobs já estão na tela, e um enum `behaviour` ao lado deles seria uma **segunda porta** para o par que já responde; *die* e *lifeloss* dependiam de haver o que observar, e agora há: `value.attribute(Hit) → motion.drive(Falloff) → motion.cull` **dentro da zona** é a morte no contato. ⚠️ Um `behaviour = Bounce\|Stick\|Slide` shipa três nomes para dois números que o artista já tem — a definição de knob morto desta casa | natureza (composição) | ⛔ | — |
| `sim.collide` | 7 | **`Restitution Randomness`** (por elemento) — Niagara §C.17 | **NÃO** (o kernel não faz leitura por-elemento de um jitter; `id` está lá, mas o hash não) | omissão | ~~**P2**~~ ✅ **FECHADO 2026-08-26** | `restitution_randomness` + `seed` (gateado por `ParamGateAbove`: com `0` o cook nunca lê a semente). A lei é `rest · (1 − randomness · h)`, `h ∈ [0,1)` do hash estável da identidade — **uma porta**, portada termo a termo para o WGSL. ⚠️ **Ela só TIRA, nunca acrescenta:** a restituição autorada é o TETO e o acaso diz quanto daquela batida se perdeu. Um `±` centrado faria `restitution = 1` devolver **mais** energia do que recebeu em metade dos elementos — a máquina de fazer energia que o clamp do `eval` existe para impedir. ⚠️ **A chave é o `id`, não a posição** (a lei do `pick` do `motion.duplicator`): pôr um `motion.sort` no meio não pode redistribuir quão saltitante cada partícula é. Sem `id` a posição é a única resposta — e no device é o `HAS_id` que impede a coluna ausente de dar a TODOS a mesma sorte. ⭐ **O hash é uma CÓPIA por drop-crate** (a convenção declarada da biblioteca) e o que a torna segura é um **golden de três pares** — ⚠️ *derivado, não escrito*: a 1.ª versão trazia-os de cabeça e reprovou no primeiro, porque `(0, 0)` dá **zero** (a avalanche parte de `0·k` e nada tira um `0` de lá). **1 mutação: 4 gates vermelhos, o controlo verde** — ⚠️ e a 1.ª corrida dela mostrou que o gate da identidade passava com a lei **morta** (se ninguém salta diferente, a invariância à ordem é trivial); ele ganhou a metade justa. |
| `sim.collide` | 7→10 | **Nenhuma `ParamSection`** num nó de 7 params — o [doc 88](../88_plano_parametros_nos_unidades_e_slider.md) deu seções a 10 nós (*a parede de sliders vira três perguntas*) | **N/A** — lei de param. O corte natural é **Forma** (`shape`/`height`/`center_*`/`radius`) × **Resposta** (`restitution`/`friction`) | omissão (lei de param) | ~~**P2**~~ ✅ **FECHADO 2026-08-26** | Três secções, não duas: **Shape** (`shape`/`center_*`/`radius`/`height`/`angle`) · **Particle Size** (`radius_from`/`particle_radius`/`size_scale`) · **Response** (`restitution`/`friction` + os dois novos). ⚠️ **A célula propunha Forma × Resposta e a leitura dos params pediu a terceira:** o trio do raio da partícula não responde nem a *onde está o obstáculo* nem a *o que acontece na batida* — ele responde *quem bate*, e enfiá-lo numa das outras faria a secção mentir sobre o que contém. ⚠️ **Nenhuma nasce fechada:** o `folded()` é para o que o artista raramente toca, e este nó não ganhou params que justifiquem esconder os velhos. ⚠️ A metade que MORDIA já tinha caído de carona no P0 do raio (2026-08-10): os **quatro knobs mortos** ganharam `ParamGate` — `height` só no Floor, `center_*`/`radius` só no Disc/Bowl, e os dois números de raio só no modo que os lê. | seção é metadado ⇒ zero efeito no cozido |

**Contagem (DERIVADA por `ferramentas/placar_conferencia.py`, reconciliada em 2026-08-26):** **P0 = 0** · **P1 = 0** · **P2 = 0** · ✅ fechadas **17** · ⛔ recusadas/refutadas **7**. ⭐⭐⭐ **A FOLHA 13 FECHOU** — zero P0, zero P1 e zero P2. A última foi o **CICLO DE VIDA da zona** (linha 60): o `Life Cycle` (`Forever`/`Once`/`Loop`) com `Start`, `Duration` e `Loop Delay`, medido antes de custar nada e construído sobre o **relógio** e não sobre «o meu estado está vazio» (a cerca 3). ⚠️ **O que ela ensinou não foi a feature, foram as DUAS aritméticas de fronteira:** perguntar *«em que fase eu estava em `t − dt`?»* re-semeava a sim no 2.º tique de cada janela para sempre (`1,0 + dt − dt = 0,999999999999999`), e trocar a pergunta pelo mecanismo certo **não curou** — `since` é ele próprio uma subtração e vem `6e-17` abaixo de `dt`. *Curar o mecanismo não cura a aritmética.* ⭐ E uma afirmação do `eval` foi **apagada por mutação**: *«com os defaults a maquinaria não corre»* era inmatável, porque a lei geral concorda com o ramo de sempre — o que o curto-circuito compra é a zona não **perguntar as horas**, visível só num relógio negativo. Cena **`=103`**. ⭐⭐ **A CAIXA SÓLIDA fechou em 2026-08-26, e a célula dizia a palavra errada:** encadear colisores é **conjunção** (um contentor), não união (um obstáculo) — medido, 121 de 121 peças ficam DENTRO. Sobra **uma**: o CICLO DE VIDA do `sim.zone`. ⭐⭐ **O ESTADO ANGULAR fechou em 2026-08-26 e não precisou de canal novo:** o `motion.drive` já escrevia qualquer coluna pelo nome (`Custom…`) — faltava alguém INTEGRAR. Sobram **duas**, e são obra: o CICLO DE VIDA do `sim.zone` e MAIS FORMAS de colisor. ⚠️ **A wave de 2026-08-26 tirou QUATRO, e DUAS delas nunca foram trabalho de código** — as duas células diziam **SIM** na coluna *exprimível?* com a cadeia escrita, e **ninguém a tinha corrido**: o envelope de nascimento (`59` nascidos antes do corte, `0` depois, contra `59` e `90` no controlo) e o matar-por-lugar (`121 → 112 → 96 → 0` sobreviventes com o raio). *Uma cadeia escrita numa célula é uma hipótese; só a corrida a torna uma refutação.* ⭐⭐ **E a célula que parecia a mais fraca — «nenhuma `ParamSection`, é cosmética» — fechou junto com a que dava ao nó o primeiro controlo de MATERIAL por elemento** (`restitution_randomness`), que é o que torna as três secções necessárias em vez de bonitas. ⏳ Sobram **três**, e as três são obra: o **estado angular** do `sim.step` (spin integrado + arrasto — o substrato de que o torque e o «spin alterado por uma colisão» dependem), **mais formas** de colisor no `sim.collide`, e o **ciclo de vida** do `sim.zone`. ⭐ **Re-medida em 2026-08-23 (bloco Z, [doc 91](../91_os_tetos_que_ninguem_mediu.md)): DUAS fecharam, e as duas eram tetos sem medição** — o `MAX_DT` do `sim.step` (que se revelou **o número mais certo dos dois do catálogo**) e o `rate` do `sim.spawn` (15 360/s, medido pela porta do produto). ⚠️ **esta linha dizia

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
