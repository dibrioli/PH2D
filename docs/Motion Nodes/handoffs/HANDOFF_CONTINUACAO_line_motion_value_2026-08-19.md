# HANDOFF DE CONTINUAÇÃO — `line/motion-value` · **o PLANO e as tarefas em aberto**

**Data:** 2026-08-19 · **Para:** o próximo agente desta linha · **Worktree:**
`/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value`

> ⚠️ **Isto não é um handoff de integração** (aquele é o
> [`…_FECHO_2026-08-18.md`](HANDOFF_INTEGRACAO_line_motion_value_FECHO_2026-08-18.md), já
> integrado). Isto é o que a próxima janela precisa para **continuar a implementar**: onde
> parar de reconstruir, o que fazer a seguir, em que ordem, e as leis que esta linha pagou
> para aprender.

---

## §0 — Os primeiros três comandos, antes de abrir arquivo nenhum

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && pwd && \
  git branch --show-current && git log --oneline -1
```

⚠️ **A janela abre na raiz (= `main`) e o MESMO caminho relativo existe nas duas árvores** —
editar a errada compila e commita **sem erro**. E a `cwd` do Bash **volta ao primário entre
turnos**: prefixe **todo** comando com o `cd`
([`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).

Depois:

```bash
bash scripts/hw-profile.sh          # tier → MODO (aqui: workstation ⇒ Modo L)
cargo check --workspace | tail -3   # a base compila?
python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"   # o placar VIVO
```

---

## §1 — Onde a linha está

| | |
|---|---|
| base | `main` `ee1432203` — a linha foi **reaberta por fast-forward** depois de a integração ter entrado |
| commits desta janela | `git log --oneline main..HEAD` (⚠️ não se pina aqui — o commit que escreve o número muda o número) |
| estado | **verde** — `fmt` 0, `clippy` 0, LOC 0, suítes das crates tocadas 0 falhas |
| smoke pendente | **`=71`** (a família `force.*`, e ela SÓ se julga com PLAY) — a `=70` foi aprovada depois de duas correcções |

⚠️ **Dois smokes de ontem nunca foram vistos pelo Enio** e já estão no `main`:
`=58` (re-smoke depois da correção do relógio que expirava) e `=59` (a porta de tempo).
Se ele reportar algo sobre eles, o mecanismo está no handoff do FECHO.

### §1.1 — O que as JANELAS de 2026-08-19/21 fecharam (a conferência foi de **P1 59 → 7**)

⚠️ **O saldo de P1 não é o número de células fechadas**, e é bom que não seja: as janelas fecharam
**trinta e duas** e ABRIRAM **duas**, ambas por medição no meio de uma wave. Uma conferência que só
descesse seria uma que parou de olhar para os lados.

| grupo | entrega | smoke |
|---|---|---|
| **S** | o 2º `fx.glow` mudo passa a **avisar**; o aviso cego ao tipo de coluna **dissolveu-se** por medição e virou teste de classe | — |
| **T** | ✅ **folha 17 fechou** — o `motion.integrate` **declara `substeps`**, e o motor já existia (as rotas (c)/(d) da célula caíram em 12/08 sem ninguém reconferir). Achado no caminho: a palavra `substeps` tinha **dois donos** e o app corria as duas leis — a corda caía **4,8× menos** que os gates dela medem | `=61` ✔ · `=52` ✔ |
| **T** | ✅ **folha 15 fechou** — o `value.switch` de N entradas é **exprimível** (`2·⌈(N−4)/3⌉+1` nós, per-elemento preservado); e a ramificação MORTA de um switch ganhou badge | `AUTOFIX=8` ✔ |
| **U** | `source.shape`: **sweep/start/inner** + **raio por canto/smoothing**; e o `corner` deixou de ser da caixa para ser do **catálogo** (4 → **38** espécies) | `SHAPE_SMOKE=3` ✔ |
| **U** | ✅ **folha 14 FECHOU** — `source.shape` ganha **Trim** (`fx_trim`, arco exato, na pilha de efeitos) + **tracejado** (o `StrokeSpec` já o falava) + o **`size` como COLUNA** (geometria em raio 1). Achados no caminho: o traço **apagava o preenchimento** de toda forma; o `Speech Rect` era a única espécie cujo raio de quina **não escalava**; e um param **conduzido por fio** fazia a forma DESAPARECER | `=76` ⏳ |
| **U** | ⛔ **três refutações medidas**: o *trim/dash* (a cura, não o item — 42 de 47 formas são fechadas), o *`fill_rule`* (a estrela citada **nunca** auto-intersecta) e o *Pick Instances* (já existe: `combine` + `duplicator(pick)`) | — |
| **V** | folha 08: `motion.sort` ganha **direção arbitrária** (`axis_angle`) e a **chave como CAMPO** (porta `weight`, modo 5) | `=63` ✔ |
| **V** | folha 08: ✅ **o `reindex` do `motion.sort`** — a ordenação não chegava ao efector indexado. **Aberta por um smoke do Enio**, não por uma tabela | `=63` ✔ |
| **V** | 🆕 **duas células ABERTAS por medição** (folha 08 `motion.cull` · folha 05 `motion.mirror`/`kaleidoscope`): a mesma lei da identidade nos vizinhos que encolhem e que crescem | — |
| **V** | folha 08: o **taper por cópia** do `motion.clone` e o **peso por entrada** do `motion.mixer` — a folha desce a **3 P1**, e os três que sobram têm espécie declarada | `=64` |
| **V** | folha 08: o **`followMouse`** — fechado por um nó novo (`value.cursor`, o **primeiro do repo com duas saídas**) mais a rota do param dirigido, e **não** pelo toggle que a célula pedia. A folha desce a **2 P1** | `=65` |
| **V** | **folha 10 (`field.*`): QUATRO células**, e vieram juntas porque duas partilhavam uma causa — o `clamp` inline do `Add` era o defeito, e era ele que tornava o `Average` inexprimível. Mais o **anel** (`inner_radius`) e a **força com sinal** (`strength`). A folha desce de **6 para 2 P1** | `=66` |
| **V** | 🔬 **UM INSTRUMENTO NOVO, e é o achado do dia:** `conferencia_vs_manifesto.py` cruza a coluna «params hoje» de cada célula com o MANIFESTO do nó (sonda `measure_node_params`, derivada do registry) e sai vermelho quando discordam. Ele acusou **31 células ABERTAS em 16 nós** a descrever um nó que já mudou | — |
| **V** | folha 01: **cinco células fecharam e QUATRO não custaram código** — `mode`/`spacing` do `distribute_curve`, `align` do `distribute_radial`, `size_random` e `dir_mode` do emitter já tinham shipado. A quinta é o **`probability`** do emitter, construído hoje. A folha desce de **6 para 1 P1** | `=67` |
| **V** | 🔬 **e a SEGUNDA passagem do instrumento**, com o sinal FORTE (o param que a coluna «default que reduz» nomeia já está no manifesto): 7 acusadas, **2 verdadeiras** (`probability` do emitter — uma SEGUNDA célula pedia o mesmo — e `lacunarity` do `motion.noise`), 5 falsos positivos **todos da mesma forma**, agora tabelados no próprio instrumento | — |
| **V** | **folha 04 (deformers): CINCO células numa wave** — a `direction` do `bend`, o `radius` e o `profile` do `twist`, o `radius_y` do `spherize` e o `mode` (Fit/Keep Length) do `spline_wrap`. As cinco são adição de param com default literal. A folha desce de **6 para 1 P1**, e o que sobra é de outro tamanho (as arestas de Bézier + patch de Coons do `four_point_warp`) | `=68` ✔ |
| **V** | **folha 02 (`force.*`): TRÊS células** — `scale_x`/`scale_y` do `drag` · `curve` do `vortex` · a coluna `density` do `buoyancy`. ⚠️ **As três tinham veredito «NÃO»/«PARCIAL», e as três estavam certas *sobre a COMPOSIÇÃO*:** o arrasto anisotrópico é impossível de compor e trivial DENTRO do nó que escreve o `accel`; a densidade por-instância tinha escritor no catálogo o tempo todo (o canal **Custom** do `motion.drive`). A folha desce de **5 para 2 P1**, e as duas que sobram são de outra espécie (o alvo = outro STREAM) | `=71` |
| **V** | ✅ **folha 11 (`fx.*`) FECHOU: cinco células, três nós, DUAS arquiteturas** — `softness` do `drop_shadow` (disco de Vogel, 16 taps, densidade preservada por raízes) · `stretch`+`angle`+`clamp` do `glow` (uniformes de PASSE: base 2×2 na tenda + teto do bright-pass) · `ParamUnit`/`ParamHardMax` nos três (a família tinha **ZERO** dos quatro canais). ⚠️ **A cerca C1 foi REESCRITA e não removida**: o borrão raster continua fora (o passe é aditivo, *um halo escuro não pode ser somado*); o que caiu foi a frase sobre a *pilha de fantasmas*, que era sobre ENCADEAR. ⚠️ **Uma objecção da célula estava 4× desatualizada** (`MAX_INSTANCES` é `262_144`, não `65_536`) | `=70` |
| **V** | ✅ **folha 05 (transform) FECHOU: cinco células, cinco nós** — `space` (World/Local) do `move` · `use_falloff_y` do `scale` **+** `mask_channel` do `falloff` (uma célula, dois nós) · `flip_rot` do `mirror` · `reindex` do `mirror` **e** do `kaleidoscope` · `carry_rotation` do `orbit`. ⚠️ **As cinco eram a MESMA pergunta em cinco lugares**: o nó sabe o que cada ELEMENTO é (orientação, posição na lista, máscara) e respondia como se a lista fosse um bloco. Os 3 P2 que sobram não são dessa família | `=69` |
| **V** | **folha 14 (source): a POSE do objeto** — `Transform = Position Only / Object Pose` no `source.object`. ⚠️ **O dado estava em mãos e era deitado fora:** o `Transform` vinha na query do shell e só a translação era publicada. A pose ganhou canal próprio (`external::pose_of`), pelo desenho do `position_of`. ⚠️ **Achado por uma MUTAÇÃO SOBREVIVENTE:** trocar `t.rotation` por `0.0` no publicador não punha nada vermelho — a expressão estava enterrada num laço que precisa de mundo ECS e atlas para correr, e por isso ninguém a gateava. Extraída (`pose_stream`), tem um gate de três linhas | `PH2D_MOTION_OBJ_SMOKE=8` |
| **V** | **folha 03 (simulação): DUAS das três** — o `break_above` do `motion.pin_constraint` (o pin que RASGA) e o desvio de obstáculo do `motion.boids` (porta `obstacle` + `avoid`/`avoid_radius`/`lookahead`). ⚠️ **As duas objecções da folha caíram por razões diferentes:** a do pin era verdadeira do SOLVER e falsa do STREAM (a carga já viaja como `accel`); a do boids era verdadeira até esta janela, e o que a derrubou foi a porta-template que o `field.shape` estreou. Sobra **1**: a forma arbitrária do `motion.soft_body`, que é de outro tamanho — o `shape_goals_weighted` é livre de topologia, mas o `cluster_goals_weighted` e a pressão precisam da grelha `rows×cols` | `=75` |
| **V** | ✅ **folha 02 (`force.*`) FECHOU: as duas células que sobravam, e as duas eram do `force.attractor`** — `Target = Point/Stream` (a porta `target`: cada peça puxada pelo ponto MAIS PRÓXIMO daquele stream) e `Predict` (o tecto em segundos: `t = min(d/velocidade própria, lead)`, o intercepto **POR partícula**). ⚠️ **A segunda só existia por causa da primeira:** antecipar precisa da VELOCIDADE do alvo, e um par de params não tem velocidade. ⚠️ **`Stream` sem fio não faz força nenhuma**, de propósito — cair nos params ali seria o knob morto que o Enio acabara de reportar noutro nó | `=74` |
| **V** | ✅ **folha 08 (stream & utilidade) FECHOU: as duas células que sobravam, de espécies diferentes** — o `reindex` do `motion.cull` (**defeito**: as colunas de identidade descreviam a lista de ANTES, e o degradê parava a meio) e o **`field.shape`**, nó NOVO (**ausência**: nem o `motion.falloff` nem nenhum `field.*` aceitava geometria como fonte de máscara — o gap era dos dois lados da porta). ⚠️ O `reindex` escreve **as DUAS** colunas, incluindo em stream que não as trazia: meia cura faz a rampa alcançar *metade, duas vezes* | `=73` |
| **V** | ✅ **folha 10 (`field.*`) FECHOU: os 2 P1 estruturais** — `key = Attribute` + porta `attr` no `field.index_range` (o **posto** sem reordenar o stream) e `curve_offset` no `field.remap` (deslocar a curva, com wrap). ⚠️ **Uma medição ENCOLHEU um item:** o *Auto Range* citado ao lado do rank já era exprimível (`value.attribute → value.normalize → motion.drive(Falloff)`, device-resident), então só o posto era gap. ⚠️ E a guarda `offset == 0 ⇒ devolve t` do deslocamento é **load-bearing**: `x − floor(x)` leva `1.0` a `0.0`, e `t = 1` é o que toda peça a máscara cheia entrega | `=73` |
| **V** | ✅ **folha 09 (cor) FECHOU: três células, dois nós** — `blend` (Mix·Add·Subtract·Multiply·Divide) do `motion.tint` · o `Offset` do `motion.color_array` como **CAMPO** (a escada `0/1/n`; a lei antiga lia `.first()` e DESCARTAVA o campo em silêncio) · e o **kernel de GPU** do `color_array`, que era o único dos quatro nós de cor sem um. ⚠️ **A rota do kernel NÃO é a que a célula sugeria:** ela apontava o canal de LUT do `color_ramp`, mas aquele acessor (`_sample(t)`) LERPA entre vizinhos e duas cores de uma paleta não têm nada entre si — a rota certa é a do `value.pattern` (contagem no slot 0, o corpo **indexa** o buffer). Teto **1024 cores**, do BUFFER, com a tabela ao lado da const | `=72` |
| **V** | ✅ **folha 07: a célula do pareamento do `motion.step`** — e ⚠️ **eram DOIS defeitos com uma frase só.** O `state` vem do tique ANTERIOR (o conjunto girou) ⇒ casa por `id`, arm por arm com o `motion.integrate`; as portas `pulse`/`reset` vêm do MESMO tique e não giram ⇒ o que nelas desalinhava era o COMPRIMENTO, e um batimento **global** (uma linha) chegava **só ao elemento 0**. A folha desce a **2 P1** | `=72` |

⚠️ **E uma correção de GEOMETRIA em `ph2d-vec-scene`, que o smoke do Enio devolveu:** a
borda que fecha uma fatia **abaulava** 19–25% do raio, porque o handle do arco sobrava na
ponta — e era o mesmo defeito que fazia o motor de quinas não ver quina nenhuma ali. Os 387
testes daquela crate passaram sem uma edição de asserção.

---

## §2 — O que NÃO se reconstrói (feito e integrado)

- **A porta de tempo** em `oscillator`/`noise`/`wiggle` — porta VALUE opcional, índice **1**,
  desligada ⇒ `ctx.playhead()` bit-a-bit, ligada ⇒ **um relógio por elemento**. CPU + GPU.
- **`TimeMode::Curve`** (índice 5) no `ph2d-nodegraph`, com a janela a **REPETIR** — ele é o
  superset cíclico do `Loop`/`PingPong`.
- **`motion.drive`**: canais `Size X` (10) e `Size Y` (11).
- **`value.attribute`**: os chips `Position X` · `Position Y` · `Radius` · `Angle`.
- **`motion.noise`**: o **espaço do campo** — `rotation` + `uniform`/`scale_y`. ⚠️ O *offset*
  e o *scale uniforme* **não são params de propósito**: saem da composição e do próprio
  `scale` (medido em `measure_noise_space`).
- **A folha 06 FECHOU** — 0 P0, 0 P1, 12 ✅, 18 P2.

---

## §3 — O PLANO, em quatro grupos, nesta ordem

> A regra de cadência é do Enio: **implementar em GRUPOS de nós, e a cada grupo UMA cena de
> smoke**. A próxima cena livre é a **`=61`** — ⚠️ e esse número se **CONTA lendo o `match`**
> do [`motion_state_demo_router.rs`](../../../shells/desktop/src/motion_state_demo_router.rs),
> nunca esta linha (ela envelhece no primeiro grupo).

### Grupo S — os DEFEITOS, antes de qualquer knob

Um defeito silencioso vale mais que uma feature, e há **dois** nomeados e medidos:

1. ⛔ **Um SEGUNDO `fx.glow` é silenciosamente INERTE** (folha 11) — `from_graph` faz
   `.find(…)` e o segundo nó nunca corre. O artista empilha dois glows, vê um, e conclui que
   o parâmetro não funciona.
2. ✅ **FEITO em 2026-08-19, e a medição DISSOLVEU o item como estava escrito.** O
   diagnóstico de nome de facto não olha o modo — mas o buraco é menor do que a nota dizia:
   as colunas não-escalares do repo são **seis** (`P` · `size` · `vel` · `accel` · `tint` ·
   `sim_d`), **quatro têm chip** e as **duas** restantes estão na denylist `INTERNAL` do
   picker. Cair nele exige digitar à mão um transiente que o picker esconde. ⇒ em vez do
   badge, um **gate de classe**
   (`every_non_scalar_column_is_reachable_or_deliberately_hidden`) que torna a situação
   impossível de nascer. *Um aviso de runtime cura o caso; um gate cura a classe.*

✅ **O Grupo S está FECHADO** (2026-08-19). Nenhum dos dois precisou de cena nova: o smoke do
primeiro é *"empilhe dois glows e veja o badge no segundo"*, e o segundo virou gate.

⚠️ **E a lição do segundo vale para o resto do plano:** o item foi escrito como *"há um
defeito, cure-o"* e a medição mostrou que o defeito era **alcançável só por um gesto que o
produto esconde**. *Meça o TAMANHO do buraco antes de escolher o tamanho da cura* — vale para
todas as células dos grupos abaixo.

### ~~Grupo T~~ — ✅ **FECHADO (2026-08-19): as duas folhas não têm mais P1**

⚠️ **Nenhuma das três células era o que dizia ser, e as três lições valem para o resto:**

1. **`motion.integrate` sub-steps** — o motor já existia (`Cook::substep`, folha 13, 12/08) e
   a célula listava rotas recusadas que **caíram quatro dias depois** sem ninguém reconferir.
   Faltava o nó **declarar**. ⚠️ *Sempre que uma célula diz «inalcançável», datar a afirmação
   e ver o que aterrou desde então.*
2. **`value.unary` Ceil/Round/Truncate** — o `P1` era um **PONTEIRO** para o item do
   `value.quantize`, que fechou em 15/08. Uma contagem que soma ponteiros conta duas vezes.
3. **`value.switch` N entradas** — a nota dizia *"contrato congelado"* e conflaciava duas
   coisas: `&'static [PortSpec]` barra a arity **dinâmica**, não uma lista estática maior (o
   §6 congela a contagem de CAMPOS do `NodeManifest`). E a composição já o exprime, medido.

⚠️ **A frase «mexe no MANIFESTO, leia a lei das portas apendadas» estava neste handoff e
estava ERRADA** — não era preciso mexer em porta nenhuma.

### ~~Grupo U~~ — ✅ **FECHADO (2026-08-21): a folha 14 não tem mais P1**

✅ **Feito:** sweep/start/inner · raio por canto + smoothing · o `corner` geral (4 → 38
espécies) · e a correção de geometria que o smoke do Enio devolveu (a borda da fatia
abaulava 19–25% do raio). ⛔ **Refutados por medição:** `fill_rule` (a estrela citada nunca
auto-intersecta; só 2 de 43 espécies distinguem as regras, e nelas a actual é a certa) e
*Pick Instances* (já existe: `combine` + `duplicator(pick)`).

**As TRÊS que fecharam em 2026-08-21, e o que cada uma ensinou:**

| item | como fechou |
|---|---|
| **TRIM / dash** | A recusa medida apontava a **função errada**. O *Trim Paths* é o [`fx_trim`](../../../crates/ph2d-vec-scene/src/fx_trim.rs) — arco exato, **abre** o contorno, testado desde os Live Path Effects. Entra na **pilha de efeitos** do `VecPath` (o `cooked()` corre-a), então alcança contornos compostos sem uma linha por espécie, e o neutro `{0,1,0}` volta a `Cow::Borrowed`. O tracejado já morava no `StrokeSpec`. **Lei 59.** |
| **`size` é GEOMETRIA, não coluna** | Geometria em **raio 1** + coluna `size`. A linearidade da receita foi **medida nas 43 espécies**, e a medição achou a exceção: o **`Speech Rect`** tirava o raio de quina dos `defaults()` em unidades de MUNDO (erro de 23%) — a quina do balão não crescia com o balão. ⚠️ **A mudança de semântica que esta tabela previu ACONTECEU e é a certa** (medido, com o tamanho autorado 2 e um valor 3): `Add = 5`, `Set = 3`, `Multiply = 6`. Sem a coluna, o `drive` partia da identidade unitária e `Set` e `Multiply` davam **o mesmo número** — o modo era um controle sem diferença. |
| **a POSE do objeto não viaja** | ✅ fechada na janela anterior (`Transform = Position Only / Object Pose`, canal `external::pose_of`). |

⚠️ **O que a wave ABRIU, com o número medido:** um param de forma animado interna **uma
geometria por quadro** (`measure_shape_store_growth`: 600 em 600 quadros, ~500 B cada ⇒
**~30 KB/s por forma animada**). O `size` saiu da chave e por isso não cresce; o Trim, o
`sweep` e os outros crescem, porque mudam a geometria de facto. ⛔ **A cura NÃO é uma
allowlist nem um teto adivinhado:** é podar o `VecPathStore` para as chaves publicadas no
quadro — e isso está **bloqueado** por o `push()` sem chave do `source.object` partilhar o
mesmo `by_handle` indexado por posição, então podar invalidaria handles vivos. O passo é
handle com geração, e é uma wave própria.

⚠️ Este grupo **encostou no módulo Vector e foi certo encostar**: a correção de
`cap_arc_ends` é em `ph2d-vec-scene` (foundational, Modo L permite) e os 387 testes daquela
crate passaram sem uma edição de asserção. Contrato congelado (§6) continua a ser parar e
reportar.

### Grupo V — as folhas grandes, por ORDEM DE DEFEITO

`08_stream_utilidade` (8) · `01_distribuicao` (6) · `04_deformers` (6) · `10_field` (6) ·
`02_force` (2) · ~~`11_fx_raster`~~ ✅ · ~~`05_transform`~~ ✅ · `03_simulacao` (3) ·
`07_tempo` (2) · ~~`09_cor`~~ ✅ · ~~`14_source`~~ ✅.

⚠️ **Derivado, não escrito:** rode `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"`. Em 2026-08-21 o TOTAL é **P1 = 5**: folha 01 (1) · 03 (1) · 04 (1) · 07 (2).

⚠️ **A contagem por folha se DERIVA** (`python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"`),
nunca se lê daqui: esta lista envelhece a cada célula fechada.

⚠️ **Não ataque por tamanho.** Dentro de cada folha, o que vem primeiro é o que a célula
descreve como **comportamento errado** (o `fx.glow` inerte, o `motion.duplicator` que perde a
escala do ponto, o `motion.step` com limitação auto-declarada), e só depois o que é knob
ausente.

### Grupo W — **a CAÇA AOS KNOBS MORTOS** (multi-agente) · ⚠️ NO FIM DA FILA

> **Pedido do Enio, 2026-08-21**, imediatamente depois de o smoke do `=73` ter achado um:
> *"Coloque no fim da fila de implementação auditoria multiagêntica a busca de parâmetros
> mortos como esse."* ⚠️ **Ele disse NO FIM** — as células abertas da conferência vêm
> primeiro. Isto abre quando a fila acima esvaziar, ou quando ele mandar.

**O que é um knob morto:** um controle que o painel pinta e que **não muda a imagem**. A
linha já encontrou **quatro espécies distintas**, e elas não se acham com a mesma sonda —
é por isso que isto é uma varredura e não um grep:

| espécie | caso medido | como se acha |
|---|---|---|
| **morto no braço DEFAULT** | `field.remap::curve_offset` — `curve.map_or(t, …)` saía antes de o deslocamento correr, e *sem curva autorada* é o estado em que o nó nasce | mutar o param e cozinhar **a partir do documento em branco**, não de uma fixture já configurada |
| **inerte no MODO que o painel mostra** | `curvature`/`steps` do mesmo nó, vivos com o contorno em `Curve` | varrer o espaço de MODOS: inerte em *algum* modo e sem `ParamGate` ⇒ acusa |
| **descartado a JUSANTE** | `motion.color_array::offset` lido por `.first()` — um campo por-instância evaporava | mutar o param **por elemento** e exigir que a saída varie por elemento |
| **declarado e nunca LIDO** | — (nenhum medido ainda) | cruzar `MANIFEST.params` com os `ctx.param("…")` da crate |

**O método, e é ele que torna a varredura paralelizável:** para cada nó × cada param ×
cada modo, cozinhar duas vezes com valores diferentes do param e comparar as colunas de
saída **ao bit**. Idênticas em todo o espaço alcançável ⇒ morto. Cada nó é independente
de todos os outros ⇒ um agente por família de nós.

⚠️ **A armadilha, e ela já mordeu:** *inerte* não é *morto*. Um param legitimamente
inerte noutro modo é o que o `ParamGate` existe para esconder — o veredicto tem de
separar **«inerte em TODO o espaço»** (defeito no nó) de **«inerte em ALGUM modo e não
gateado»** (defeito no painel). Um audit que colapse os dois devolve uma lista de falsos
positivos do tamanho do catálogo.

**O que já existe e não se reconstrói:** `measure_node_params` (a sonda que deriva os
params do registry) · `conferencia_vs_manifesto.py` (cruza a conferência com o manifesto)
· `measure_scene_layout` (cozinha uma cena e mede o que ela desenha) ·
`each_pair_actually_differs_in_the_cooked_result` (o molde do gate: comparar colunas
cozidas, sobre TODAS as colunas).

⚠️ **Isto é uma AUDITORIA, não uma wave de features** — o produto dela é uma tabela de
acusações com o mecanismo ao lado de cada uma, e as curas entram na fila depois, uma a
uma, cada qual com o seu gate. ⛔ Não «corrija» ao passar: uma acusação sem cena de smoke
é uma mudança de comportamento que ninguém olhou.

---
---

## §4 — As SESSENTA E CINCO LEIS que esta linha pagou para aprender

⚠️ **Cada uma destas custou um gate vermelho, um smoke reprovado ou uma medição** — elas não
são estilo.

1. **TRAP 1 SEMPRE, e ele vale para a FOUNDATION também.** Dez células da folha 06
   envelheceram — a última **em metade**: o *scale uniforme* do campo do ruído já era o param
   `scale`, bit-a-bit. E na porta de tempo o orçamento listava **três saídas caras** porque o
   seletor de variante só vê params — e o canal certo (`ColumnAccess::ReadBroadcast` + o
   `const HAS_<porta>_<col>` do codegen) **já existia**. *Meça se o substrato já exprime,
   antes de orçar um mecanismo novo.*
2. **Um ✅ de MECANISMO não é um ✅ de ARTISTA.** A folha 15 marcava as lanes de uma `Vec2`
   como fechadas porque o degrau existia — e não havia gesto que chegasse lá. *Um degrau sem
   chip é inalcançável.*
3. **Uma fixture só prova o que ela CONTÉM.** A fileira de teste do `motion.noise` tem
   `y = 0` em toda peça, e um gate de `scale_y` reprovou sobre código correcto. ⚠️ E a
   rotação **mostra-se** numa fileira, que é o que esconde o buraco de quem só olha para um
   dos dois eixos.
4. **A régua tem de ser a coisa REAL.** O oráculo da cena `=60` subtraiu a *média* para tirar
   a grade; a grade varre 4,48 de mundo em Y e a razão do controle deu **0,21** em vez de ~1.
5. **A DIREÇÃO de um knob pode ser contra-intuitiva — meça-a.** Escala maior num eixo =
   feição **menor** nele (`dx/dy` cai de 0,976 para 0,341). O rótulo tem de dizer o que o
   artista vê, não o que o número sugere.
6. **Nenhum controle pode EXPIRAR.** O `TimeMode::Curve` clampava a janela e a sub-árvore
   congelava para sempre; os gates mediam **dentro** da janela e ficavam verdes sobre produto
   morto. Mesma classe do `fade` do oscilador. *Um gate que só olha para dentro da janela não
   pode ver uma janela que não repete.*
7. **Uma exceção por NÚMERO DE LINHA quebra em silêncio.** A tabela `HAND` do
   `placar_conferencia.py` era chaveada por `(arquivo, nº)`; acrescentar uma linha desalinhou
   tudo e o placar imprimiu **um ✅ a menos**. Hoje a chave é um TRECHO e cada uma tem de
   casar **exactamente uma** linha, senão a ferramenta sai vermelha.
8. **UM GATE MEDE O QUE A CENA PRODUZ; SÓ O OLHO MEDE O QUE ELA MOSTRA.** A `=60` foi
   reprovada **duas vezes** com todos os gates verdes, e a segunda é a lição maior.
   **v1 (posição):** ⚠️ um sprite **sem coluna `size` desenha a `1,0`** (o `SIZE_IDENTITY` do
   shell) contra um vão de `0,32` — o bloco era uma placa sólida · o deslocamento valia
   **1,31×** o vão · e havia **2,5** manchas no bloco inteiro.
   **v2 (tamanho):** o padrão estava lá e o olho não o via — com o bloco a 220 px e 21 pontos
   de lado, os pontos iam de **3,4 px a 9,5 px**. ⚠️ **E a aritmética mostrou que não havia
   saída por números:** para as manchas terem pontos que cheguem **e** os pontos serem
   grandes seria preciso mais janela do que existe. *Duas exigências que puxam a mesma folga
   em direções opostas não se resolvem afinando; resolvem-se trocando o CANAL.*
   **v3 (cor):** `motion.color_ramp` pinta o `tint` a partir do campo lido de volta
   (`value.attribute(Size)` → `value.map_range`). Luminância medida: **0,073 a 0,923**.
   ⚠️ **Os números viraram três gates** — `the_dots_never_touch_so_the_field_is_readable`,
   `the_block_holds_enough_blobs_for_a_rotation_to_read` e
   `the_colour_carries_the_field_all_the_way_to_the_instance` (que lê o `tint` **no
   instance**, depois do lowering). *Ao desenhar uma cena, meça o elemento em PÍXEIS e
   pergunte que canal perceptual carrega o sinal — antes de a mandar.*
9. **UMA SUBAMOSTRA UNIFORME PODE DESENHAR UMA FIGURA QUE NÃO ESTÁ LÁ.** O carimbo dos cards
   (`preview_points`) subamostrava por **passo fixo**, e sobre uma GRADE isso **alia numa
   reta**: com 441 pontos e 21 colunas o passo é 10, `10·k mod 21` anda −1 por linha, e
   **21 das 45** amostras caíam na MESMA diagonal (5 das 21 diagonais tocadas). Os cards
   mostravam um traço enquanto o canvas mostrava manchas — e **o gate que existia ficava
   verde**, porque ele media se o carimbo *abrangia* a grade, e uma diagonal abrange todas as
   linhas e todas as colunas. ⚠️ **A cobertura também não acusa**: dividido em nove ladrilhos,
   o passo fixo enchia os nove. *O que se mede é ESTRUTURA, não cobertura.* Cura: um jitter
   determinístico dentro do balde (4 de 45 na pior diagonal), com o passo fixo como **controle
   negativo dentro do próprio gate**.
10. **PARIDADE PROVA QUE OS DOIS LADOS FAZEM O MESMO, NUNCA QUE O MESMO É CERTO.** A ordem
    do espaço do `motion.noise` (`escala→rotação`) foi escrita, **defendida num comentário** e
    coberta pelo gate de paridade CPU×GPU — e estava **errada**: com `M = R·S` as feições do
    mundo são `S⁻¹R⁻¹(círculo)`, cujos eixos são os do MUNDO, então **a rotação não gira as
    faixas** de um campo anisotrópico. O olho do Enio apanhou (*"não há faixas diagonais"*);
    a paridade nunca poderia. ⚠️ **E o gate «próprio» que existia dava falso conforto**: ele
    construía as duas ordens à mão e provava que elas **diferem** — verdade, e inútil, porque
    nunca perguntou **qual** delas o nó embarcava. *Um gate que prova que duas escolhas são
    distintas não defende a escolha.* Cura: um gate que mede a **direção das faixas**, que é
    a afirmação que o produto faz.
11. ⛔ **OLHE o arquivo antes de escrever nele.** Nesta janela eu sobrescrevi a cena `=51`
   inteira ao criar um módulo com um nome que já existia (`…_demos_space.rs`). Recuperou-se
   com `git checkout --`, mas só porque a árvore estava limpa. *Um `ls` antes do `Write`
   custa nada.*
12. ⚠️ **A suíte inteira é um relógio.** Duas corridas marcaram falhas que eram **carga**
   (`the_cost_of_depth_is_linear_not_explosive` e
   `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`), com `load average` em 14,8.
   Sozinhas passam. *Nada desta workstation vale acima de `load ~5`.*
13. **A porta de tempo é uma COLUNA, não um escopo** — ela não herda a recusa
    `CookError::SequentialInTimeScope`. Se acrescentar uma porta a outro nó, o gate
    `the_time_port_is_a_column_not_a_cook_scope` é o molde.
14. ⚠️ **O oráculo da cena é o que o OLHO lê, nunca o que o nó emite** — e esta é irmã da 10,
    apanhada pelo mesmo Enio no mesmo dia. A cena `=63` tinha quatro gates verdes que mediam a
    **permutação das posições** à saída do `motion.sort`; o que o smoke mostra é a **cor**, e
    a cor vinha do `motion.tint`, que lê a coluna `Index` — que o `sort` levava consigo. As
    três bandas saíam com **a mesma pintura** e a suíte estava verde, porque cada gate media o
    lado certo da costura **do lado errado da fronteira**. *Se a cena existe para ser olhada,
    o gate mede o pixel — ou pelo menos a última coluna antes dele.* O sintoma tem forma
    reconhecível: **a cena mostra a ordem de nascimento** (aqui, a grelha por linhas de baixo
    para cima), porque é isso que sobra quando a operação não alcança o consumidor.
15. ⚠️ **A PEÇA TEM DE CABER NO PASSO, e o passo é o `gap_*` do `motion.grid`.** Uma instância
    sem coluna `size` é desenhada com `SIZE_IDENTITY` = **1,0 unidade de mundo**, e as cenas
    desta conferência autoram passos de **0,12 a 0,6** — ou seja, quase todas desenham peças
    sobrepostas, e a cena `=63` tinha **5,7 peças empilhadas em cada ponto**. ⛔ **Isso não é
    um defeito universal e não se varre:** numa cena de campo denso a sobreposição é o look, e
    o Enio já aprovou vários. Ela é fatal **quando o assunto da cena é a ORDEM DE SAÍDA** —
    porque aí a ordem de desenho é a mesma variável, e a metade desenhada por último **tapa**
    a primeira. Sintoma: as bandas de ordem espacial (X, diagonal) leem-se bem, e a de ordem
    embaralhada sai *quase toda da cor final, com manchas*. Cura: um `motion.scale` com
    `amount < gap` antes da ordenação. Gate `no_piece_is_wide_enough_to_hide_its_neighbour` —
    ⚠️ ele mede o **cozido** (lado da peça contra a menor distância entre vizinhos), não os
    dois literais lado a lado, e **exige** a coluna `size` em vez de a tolerar: a ausência
    dela é exactamente o estado que reprovou.
16. ⚠️ **TODO modificador honra o `falloff`, e um `motion.move` de LAYOUT posto depois de um
    campo é mascarado por ele.** A cena `=66` punha as seis bandas nos seus quadrantes com um
    `move` no fim da cadeia; as peças no cheio do campo andaram o vão inteiro e as de fora
    ficaram onde estavam, então a banda **espalhou-se** em vez de se mudar (medido: um vão de
    `5,6` deu um deslocamento efectivo de `4,6`). Os quatro gates da cena reprovaram e o
    diagnóstico veio de uma SONDA que imprimiu o alcance de `x`/`y` por banda — a asserção
    sozinha só dizia *"os dois lados são iguais"*. *Layout antes do campo; e aí o centro do
    campo tem de seguir a banda.* Irmã da 14 e da lei do quadro da cena `=65`.
18. ⚠️ **A CONFERÊNCIA MEDE O PRODUTO CONTRA UMA FOTOGRAFIA DELE, e a fotografia envelhece.**
    Cada célula tem uma coluna «params hoje»; uma wave que acrescenta um param fecha a célula
    DELA e deixa as vizinhas a descrever um nó que já não existe — e o placar passa a contar
    como aberto o que já shipou. Medido em 2026-08-19: a célula do `motion.emitter` dizia
    **10** params, o manifesto tinha **20**, e **cinco das seis** linhas P1 daquela folha já
    estavam feitas. ⛔ **Isto não se resolve com disciplina, resolve-se com instrumento:**
    `python3 "docs/Motion Nodes/ferramentas/conferencia_vs_manifesto.py"` cruza as duas
    contagens e sai vermelho. ⚠️ **Ele imprime DOIS sinais e eles têm forças diferentes.** O
    **forte** é o param que a coluna «default que reduz» nomeia já estar no manifesto — ele
    aponta o ITEM. O **fraco** é a contagem de «params hoje» discordar — diz só que o nó mudou,
    e o que mudou pode ser de outra célula (31 linhas em 16 nós, quase todas benignas).
    ⚠️ **Calibração medida sobre as 7 que o sinal forte acusou: 2 verdadeiras, 5 falsos
    positivos, e os cinco da MESMA forma — o nome existe com outro significado ou com menos
    valores** (um `emit_mode` a que falta um terceiro valor; um `start` que é o do *sweep* e
    não o do *trim*). A tabela está no doc-comment do instrumento. *Um homónimo e um enum com
    um valor a menos leem igual num nome — o que decide é ler a célula.*
20. ⚠️ **O `applicable` virou a saída padrão para o device, e a lista já tem TRÊS** — cada uma
    por um motivo estrutural diferente, e vale atacá-las juntas um dia: o `reindex` do
    `motion.combine` (a concatenação no device é um `copy_buffer_to_buffer` sem shader), o
    `probability` do `motion.emitter` (a `count_law` é aritmética e um portão por hash torna a
    contagem dependente de DADOS — pede o prefix-sum que o `motion.cull` já tem) e a
    `direction` do `motion.bend` (a expressão de um `ReduceSpec` só alcança `params`, então um
    extent rodado exigiria o polinómio do `trig.rs` escrito uma segunda vez dentro da string).
    ⚠️ **O contra-exemplo importa tanto quanto a lista:** o `radius` do `motion.twist` mexe no
    MESMO tipo de redução e **não** recua — porque a redução não muda de expressão, só o
    consumidor dela escolhe entre o valor medido e um param. *O que força a recusa é a redução
    (ou a contagem) mudar de FORMA, nunca o param existir.*
21. ⚠️ **Um gate que empurra params por uma FRAÇÃO do curso acusa todo enum de estar morto.**
    O `every_control_the_write_on_scene_offers_does_something_in_it` cutucava cada param por
    37% do curso; o `mode` do `spline_wrap` é lido com `.round()`, então `0,37` voltava a `0` e
    o gate reprovou um controle vivo. A cura é do gate e não do param: ele arredonda o empurrão
    ao `step` do hint quando o passo é inteiro, o que vale para **todo** param discreto.
22. ⚠️ **Ao escolher a amostra de um gate, evite os pontos FIXOS da lei que mede.** Dois gates
    do `twist` nasceram vermelhos sobre produto correcto porque eu amostrei em `t = 0,5`: o
    smoothstep e o smootherstep **fixam o meio** (`0,5 → 0,5`), então três dos quatro perfis
    dão ali o mesmo número e o gate acusa de decorativo um enum que funciona. A amostra tem de
    ser onde as leis DISCORDAM. ⚠️ E irmã disto: **a senoide parabólica do HR-5 não é
    norma-preservante** (`c² + s² ≠ 1` ao bit) — uma rotação por ângulos diferentes perturba o
    raio em ~0,1%, então um gate de *"a rotação não muda o raio"* precisa de uma barra relativa
    e não de um épsilon de `f32`.
23. ⚠️ **Ao consertar um nó de ESTRUTURA, MEÇA os irmãos que mexem na mesma lista** — a sonda
    `measure_identity_after_structure` foi escrita depois do conserto do `sort` e achou o mesmo
    defeito em três vizinhos (`cull` encolhe e deixa `Count` velho; `mirror` e `kaleidoscope`
    crescem e deixam `Index` **e** `Count` velhos; `clone` faz o certo). ⛔ **E não corrija
    meio:** no `mirror`, arrumar só o `Count` faz a rampa alcançar **metade, duas vezes** — as
    duas colunas são uma pergunta só, e a resposta é de família.
24. ⚠️ **Uma FIXTURE sobre um eixo não distingue uma rotação da sua TRANSPOSTA.** A mutação que
    troca `dx·c − dy·s` por `dx·c + dy·s` **sobreviveu** a cinco gates do `space` do
    `motion.move`, porque todos usavam `dy = 0` — e com `dy = 0` as duas expressões são a mesma
    linha. O comprimento também não separa (as duas são isometrias). *Um oráculo de rotação
    precisa de um vetor OBLÍQUO e de um ângulo OBLÍQUO*, e o gate que nasceu disso fixa o
    SENTIDO (`(0,1)` a 90° tem de ir para `(−1,0)`).
25. ⚠️ **Um canal que ninguém consegue ESCREVER não existe** — e a célula que pede o leitor
    paga o escritor. O `use_falloff_y` do `motion.scale` sem o `mask_channel` do `motion.falloff`
    seria um toggle que o smoke não distingue de um bug: nada no catálogo escrevia `falloff_y`.
    As duas metades são **uma** célula da conferência, e é assim que ela foi fechada.
    ⚠️ E a metade irmã: **ligue por PARAM, nunca pela presença de uma coluna.** Presença faz um
    nó a montante mudar o resultado de um nó que ninguém tocou (invisível no painel), e dá ao
    device uma pergunta que ele não pode ver — o kernel não sabe se a coluna existia. Com o
    toggle, a ausência tem um significado previsível (`1.0`, a identidade da binding) e as duas
    portas resolvem a MESMA expressão.
26. ⚠️ **Uma PALAVRA que já tem dono não se reusa, mesmo com sentido diferente — há gate.** Pus
    um param `channel` no `motion.falloff` (que máscara escrever) e o
    `no_param_of_a_channel_driven_node_is_declared_a_fixed_length` reprovou: neste app `channel`
    significa *a GRANDEZA dirigida* (`drive`/`oscillator`/`noise`/`wiggle`), onde um comprimento
    vale metros em Position e **graus** em Rotation. O gate varre a palavra, e estava certo sobre
    a palavra. ⛔ **A cura é RENOMEAR (`mask_channel`), não abrir excepção no gate**: uma palavra
    com dois sentidos é a falha de duas-portas no vocabulário, e cada excepção num censo é um
    buraco que o próximo nó atravessa.
27. ⚠️ **Ao comparar duas bandas de uma cena, compare a forma RELATIVA ao centro de cada uma.**
    Três gates da `=69` nasceram vermelhos porque eu comparei `P` absoluto de bandas que vivem em
    `x = ±5,6` — estavam a medir o layout, não o nó. E depois de subtrair o centroide, a
    igualdade tem de ser APROXIMADA: `q − centroide` cancela ~5,6 de ~5,9 em `f32`, e os dois
    lados perdem bits diferentes (medido: `0,3000002` contra `0,2999997`).
32. ⚠️ **«INEXPRIMÍVEL PELO CATÁLOGO» não é «inexprimível».** Três células da folha 02
    tinham veredito **NÃO**/**PARCIAL** e as três estavam CERTAS — sobre a composição.
    O arrasto anisotrópico é impossível de compor (nada escreve `accel` de um vetor
    arbitrário) e é uma multiplicação **dentro do nó que escreve o `accel`**. ⛔ Um
    veredito de composição **não decide** se o nó pode; ao ler «NÃO», pergunte *«NÃO
    por quê — pela parede do substrato, ou por o nó não ter o knob?»*
33. ⚠️ **Antes de declarar um canal inalcançável, procure o ESCRITOR no catálogo.** A
    densidade por-instância do `force.buoyancy` mediu uma cadeia de **quatro nós por
    material** e concluiu «não a custo pagável» — enquanto o canal **Custom** do
    `motion.drive` escreve QUALQUER coluna por nome, e sempre escreveu. É a irmã da
    lei 25 (*um canal que ninguém escreve não existe*) pelo outro lado: **um canal cujo
    escritor você não procurou parece não existir.**
34. ⚠️ **Um gate que reimplementa a lei mede a CÓPIA dele.** Escrevi seis gates para
    esta folha com um `run()` local que repetia a aritmética do `eval`, e **quatro
    mutações sobreviveram** — apagar a anisotropia, trocar os dois eixos, ignorar a
    coluna de densidade, e nunca aplicar o perfil. Todos ficaram verdes porque o teste
    tinha a lei certa e o produto não era chamado. *Um gate de nó COZINHA o nó.*
28. ⚠️ **Um gate VERDE sobre `rot` não prova que a cena MOSTRA a orientação — a peça é
    simétrica.** O par 3 da `=69` nasceu como um leque de raios; o Enio reprovou-o com
    *"sem flip não pude entender"*, e a medição deu-lhe razão de duas maneiras: (a) uma
    barra tem simetria de **180°**, então quatro das catorze peças saíam a `0` ou `180`
    graus do radial e a orientação errada desenhava-se **igual** à certa; (b) depois de um
    espelho o «radial» passa a ser medido do CENTROIDE DA FIGURA, que num LEQUE não é o
    centro do anel (medido: `1,03` de distância) — nem a metade correcta era radial.
    ⛔ **Não conserte isto afrouxando o gate nem trocando a cor:** escolha uma diferença
    angular que **não seja múltipla de 180°** (aqui `180 − 2·35 = 110°`, o `\` contra o `/`),
    e prefira um layout cujo centroide o olho já conhece. *Um leque é um layout em que
    «para fora» deixa de ter um sítio.*
29. ⚠️ **Uma CERCA pode ter duas razões, e elas envelhecem em ritmos diferentes.** A C1 da
    folha 11 recusava a maciez da sombra por (a) *"o borrão é raster"* e (b) *"maciez falsa
    a partir de uma pilha de fantasmas"*. A (a) continua **verdadeira e com mecanismo** — o
    passe do Motion compõe aditivamente e um halo escuro não pode ser somado. A (b) era
    sobre **ENCADEAR** o nó, e a própria célula tinha medido os três defeitos do
    encadeamento — nenhum dos quais um disco de UM passe tem. ⛔ *Ler uma cerca não é
    obedecer-lhe nem ignorá-la: é separar as razões dela e medir cada uma.* E o que se
    escreve depois é a cerca **reescrita**, com a metade que sobrevive em destaque.
30. ⚠️ **Ao pôr um param novo num nó, procure a CONTAGEM escrita à mão que o conta.** O
    `fx.glow` tinha um gate `assert_eq!(MANIFEST.params.len(), 9)` — ele ficou vermelho
    sobre código correcto, e a cura não é subir o número: é trocá-lo por uma DERIVADA. O que
    a célula queria dizer era *"nenhum knob nasce mudo"*, e isso mede-se mexendo em cada
    param e exigindo que o leitor devolva outra coisa (com controle positivo, senão passa
    por vácuo quando a lista esvazia).
31. ⚠️ **Uma raiz `N`-ésima é `sqrt` encadeado se, e só se, `N` for potência de dois** — e
    é isso que torna o número exacto em toda plataforma (o `sqrt` do IEEE-754 é
    correctamente arredondado; um `powf` é libm). Foi essa a razão de o disco da maciez ter
    **16** taps e não 12: a densidade do miolo pede `a = 1 − (1−A)^(1/N)`, e a alternativa
    ingénua `A/N` clareia a sombra em 15% ao ligar o knob.
35. ⚠️ **A ROTA que uma célula sugere pode ser a errada pelo motivo certo.** A folha 09
    apontava, para o kernel do `motion.color_array`, *"o canal de LUT que o `color_ramp` já
    usa"*. O **canal** estava certo; o **acessor** não: o `<name>_sample(t)` que o gerador
    emite LERPA entre vizinhos, e duas cores de uma paleta não têm nada entre si — a cor `k`
    sairia misturada com a `k±1` por ~1e-7, invisível a olho e fatal num gate de paridade. A
    resposta já estava paga por escrito no `value.pattern` (contagem no slot 0, o corpo
    **indexa** o buffer). *Leia a célula, e depois procure quem já resolveu a mesma forma.*
36. ⚠️ **Um fixture com a IDENTIDADE do operador apaga o teste que o usa.** O gate da ORDEM
    do `blend` nasceu com `existing = 1.0` e `Multiply`: `1 · lerp(1, t, f)` **é**
    `lerp(1, 1·t, f)`, então as duas ordens dão o mesmo número e o controlo não controlava
    nada. A cura foi uma fonte cujos quatro canais não são 0 nem 1. *Escolha o fixture
    contra a álgebra da lei, não contra o que é fácil de escrever.*
37. ⚠️ **Uma mutação que sobrevive tem DUAS leituras, e a segunda é mais provável do que
    parece: a mutação era EQUIVALENTE.** A prova do guarda de divisão por zero passou verde
    à primeira porque eu escrevera `if t > 0 { e/t } else if t == 0 { e } else { e/t }` —
    literalmente a mesma função. *Antes de escrever o gate que falta, leia a sua própria
    mutação.* A real (`t.to_bits() == 0`, que só apanha o `+0.0`) sangrou.
38. ⚠️ **Uma limitação declarada num cabeçalho pode ser DUAS.** *"Pareamento posicional"* no
    `motion.step` juntava um **estado que gira no TEMPO** (o `pre`: entre dois tiques
    nasceram e morreram peças ⇒ casa por `id`) com **portas que só variam em COMPRIMENTO**
    (`pulse`/`reset`, cozidas no mesmo tique ⇒ a escada `0/1/n`). Curar as duas com a mesma
    ferramenta teria posto id-keying numa porta que não precisa dele — e teria deixado o
    defeito visível (o batimento global a alcançar só o elemento 0) de pé.
39. ⚠️ **Uma nota que diz «não há máximo» tem prazo de validade.** O `DEFAULT_PALETTE` do
    `color_array` dizia *"This is a DEFAULT, not a cap. There is no maximum"* — verdade
    enquanto o nó era CPU-only, falsa no minuto em que o kernel entrou e o `storage` do
    device passou a ser o teto. §0 outra vez, do outro lado: *quem move o número que tornava
    a nota verdadeira tem de reconferir a nota* — e o teto novo diz **de que recurso é**
    (16.388 B por nó, constante) com a tabela ao lado.

40. ⚠️ **Um item de lista pode ENCOLHER quando se mede — e metade dele não existir.** A
    célula do `field.index_range` citava *"rank por atributo (min/max + **Auto Range**)"*
    e parecia um item; medido, o Auto Range já era exprimível hoje e **sem cair para a
    CPU** (`value.attribute → value.normalize(Range) → motion.drive(Falloff, Set)` — o
    `normalize` DESCOBRE o extento, é a razão de ele existir). Só o posto era gap.
    *Meça a composição antes de construir, mesmo quando a célula já traz uma citação —
    a citação descreve a referência, não este repo.*
41. ⚠️ **Um gate de CENA apanha um defeito de CENA, e vale a pena escrevê-lo assim.** O
    `the_pentagon_fits_inside_the_grid_it_masks` reprovou o MEU layout: com o passo de
    grelha que eu escolhera, o pentágono (raio + penumbra) transbordava a grade e as
    duas bandas do par 4 sairiam mascaradas por inteiro — **iguais**, com o par verde e
    mudo. O que o viu foi a conta ser **derivada da grelha** em vez de escrita à mão.
    *Um par cujos dois lados podem colapsar por um número de layout precisa de um gate
    sobre esse número, não sobre os params que ele compara.*
42. ⚠️ **O ponto neutro de um knob de WRAP não é grátis.** `x − floor(x)` é o wrap certo
    em todo o intervalo **menos no topo**: ele leva `1.0` a `0.0`. Onde o domínio é uma
    máscara, `t = 1` não é um caso de canto — é o que toda peça a máscara cheia entrega,
    ou seja metade da cena. A guarda explícita (`offset == 0 ⇒ devolve t`) é uma linha, e
    sem ela ligar o nó SEM tocar no knob repinta a cena. *Teste a identidade de um knob
    no topo do domínio dele, não no meio.*

43. ⚠️ **POSICIONAR É UPSTREAM DA MÁSCARA — e isto invalidou uma cena inteira.** Todo
    comportamento desta biblioteca multiplica o seu efeito pelo `falloff` (§1.2 — é a
    razão de os `field.*` existirem), e isso inclui o `motion.move`/`motion.transform`
    que se usa para **pôr uma banda no quadrante dela**. Posto DEPOIS do campo, o
    deslocamento de colocação vira `dx · falloff_i`: cada peça anda uma distância
    diferente e a banda estica-se por cima das vizinhas. Medido na cena `=73` v1: uma
    fileira de **7,5** de largura por construção saiu com **1,50** (o passo colapsou de
    0,50 para 0,10), e uma grelha de 2,94 saiu com **8,94**. *A colocação corre logo a
    seguir à fonte, onde ainda não há coluna de máscara nenhuma.*
44. ⚠️ **Eu autorava cenas ÀS CEGAS, e esse era o buraco real.** O grafo é o que eu
    escrevo; a IMAGEM é o que o cook devolve — e até 2026-08-21 o único instrumento que
    media a diferença entre os dois era **o olho do Enio, depois de compilar em
    release**. Nenhum dos nove gates daquela cena viu o defeito, porque nenhum media
    ONDE as peças ficam: todos mediam params. A cura é a sonda
    `measure_scene_layout` (`PH2D_LAYOUT_LEVEL=<n>`), que cozinha cada banda e imprime
    a caixa dela, mais o gate `no_band_leaves_its_slot`, cuja caixa prevista é
    **derivada** da tabela de grelhas. *Um smoke que só o Enio pode correr é um teste
    com uma pessoa no meio do laço.*
45. ⚠️ **Um exemplo que só se entende lendo o terminal não é um exemplo** (Enio,
    2026-08-21: *"tudo misturado e bagunçado sem explicação simples"*). As três curas,
    e nenhuma delas é código de feature: **legenda no CANVAS** (`source.text` — a linha
    diz o nome dela ao lado dela), **uma pergunta de sim/não por linha** em vez de uma
    lista de oito rótulos no `stderr`, e **quadrantes com margem medida** em vez de
    posições escolhidas a olho.
46. ⚠️ **O branco não é o «apagado».** O default do `motion.tint` é branco opaco, então
    numa cena de MÁSCARA as peças **não** acesas eram a coisa mais clara da tela e o
    padrão desaparecia dentro delas. A leitura de uma máscara é *escuro → aceso*: pinte
    o repouso de cinza-chumbo **antes** do campo, e a cor viva **depois** dele.

47. ⚠️ **O ramo que o estado DEFAULT toma é o que menos se testa — e é o único que o
    artista vê primeiro.** O `curve_offset` do `field.remap` estava escrito
    `curve.map_or(t, |c| c.eval(shifted(t, off)))`: com **nada autorado** (o estado em
    que o nó nasce) o `map_or` devolve `t` e o `shifted` **nunca corre**. Três coisas
    numa: (a) o knob estava morto exactamente onde se mexe nele primeiro; (b) o gate
    não viu porque o FIXTURE dele passava `Some(&curva)` — o outro ramo do `map_or`;
    (c) o **device fazia o contrário** (a LUT assa a identidade e o WGSL amostra-a já
    deslocada), então os dois caminhos divergiam em silêncio. *Ao pôr um knob atrás de
    um `Option`, teste o braço `None` — ele é o default.*
48. ⚠️ **Um par de demonstração tem de diferir na IMAGEM, e isso só se mede
    COZINHANDO.** Nove gates daquela cena afirmavam que cada par difere no param que
    anuncia; as duas metades da linha da RAMPA saíam **byte-idênticas** e os nove
    passavam. O gate certo lê as colunas cozidas das duas metades e exige que ALGUMA
    difira — sobre **todas** as colunas, não sobre uma escolhida (escolher a coluna é
    escrever a resposta ao lado da pergunta). *É a terceira vez que esta linha paga a
    mesma lei: um gate de param não é um gate de produto.*

49. ⚠️ **Uma célula pode DEPENDER de outra da mesma folha, e implementá-las juntas é o
    que a torna barata.** As duas últimas da folha 02 eram ambas do `force.attractor`:
    *alvo = outro STREAM* e *Predict Intercept*. A segunda **não existe sem a
    primeira** — antecipar exige a VELOCIDADE do alvo, e um par de params escalares não
    tem velocidade. Separadas, a segunda pareceria «arquitetura»; juntas, foi um `min`
    de duas linhas. *Antes de abrir uma wave, olhe se a célula ao lado é a metade que
    falta.*

50. ⚠️ **Uma objecção de célula pode ser verdadeira de um SÍTIO e falsa de outro.** A do
    `motion.pin_constraint` dizia *"nenhum solver publica a força sentida no pin"* — e é
    verdade **do solver**. Mas a carga já viaja no stream antes dele: as `force.*`
    acumulam em `accel` e o integrador consome-a, então um pin posto **depois** das
    forças e dentro do laço lê exactamente quanta força tenta movê-lo. *Antes de aceitar
    um «não há de onde tirar», pergunte de onde o VIZINHO a jusante tira.*
51. ⚠️ **Uma objecção pode CADUCAR por causa de uma wave anterior, e ninguém a
    reconfere.** A do `motion.boids` dizia *"a ausência de qualquer geometria de
    obstáculo alcançável de dentro"*, e era verdade — até a porta-template (um segundo
    stream lido como geometria) ser estreada pelo `field.shape` e repetida pelo
    `force.attractor`, duas waves antes. §0 outra vez: *quem move o número que tornava
    algo inalcançável tem de reconferir a nota* — e aqui quem o moveu fui eu, três dias
    antes, sem ligar as duas coisas.

52. ⚠️ **Escrevi no doc uma afirmação sobre o VIZINHO que nunca medi — e o gate seguinte
    pinou a afirmação errada.** *"O `motion.integrate` lê o `accel` e o `inv_mass` da
    porta `forces`"* saiu de eu ter visto `Consumes("inv_mass")` no `register_couplings`
    e ter completado o resto de cabeça. A verdade é `let w = scalar_to_n(rest, INV_MASS,
    n, 1.0)`: o `accel` vem do `state`, o **`inv_mass` vem do `rest`**. A cena inteira
    ficou com um `inv_mass` que ninguém lia, nove gates verdes, e o smoke a dizer *"tudo
    foi levado pelo vento"*. *Um `Coupling` diz QUE coluna o nó consome; ele não diz de
    que PORTA. Vá ao `eval`.*
53. ⚠️ **Uma sonda de simulação que não fecha o quadro mede ZERO — e o que a desmascara é
    corrê-la contra uma cena APROVADA.** O `pre` de um circuito sequencial só avança em
    [`Cook::advance_tick`], *"once per frame, after the frame's cook(s)"*; um laço que só
    `cook`a lê o mesmo tique quarenta vezes. A minha primeira sonda dizia que a `=75` não
    andava — e dizia o mesmo da `=71`, que o Enio já aprovara. *Uma sonda que acusa a cena
    boa está a acusar-se a si própria; ponha sempre um caso conhecido-BOM no varrimento.*

54. ⚠️ **Uma cena de smoke tem de usar o NÓ QUE O CATÁLOGO TEM, não o mais fácil de
    montar** (Enio, 2026-08-21: *"porque usar grid se temos nós de tecido?"*). Eu montei
    a cortina do `break_above` com uma `motion.grid` — pontos soltos — porque a fiação
    era mais curta. O custo não foi técnico, foi o EXEMPLO: uma nuvem de pontos que voa
    não mostra um pano a **rasgar**, mostra pontos a irem embora. Com o
    `motion.soft_body` vê-se a folha inteira soltar-se.
    ⚠️ **E o nó certo era também o mais fácil**, o que só se soube depois de olhar: o
    `soft_body` lê o `inv_mass` **e** o `accel` da MESMA cadeia (a de estado), então ali
    o pin cabe dentro do laço — enquanto com o `motion.integrate` ele tinha de ficar no
    caminho da arte e receber a carga por uma porta própria. *A cena feia estava a
    esconder que a arquitetura tinha um idioma melhor.*

55. ⚠️ **Um evento que acontece no PRIMEIRO quadro não se vê acontecer.** O rasgo do pin
    estava correcto — o gate media-o, a sonda media-o — e o smoke voltou com *"não
    rasga"*, porque com a carga acima do limiar desde o tique 0 o pano da direita já
    NASCE a voar: não se vê rasgar, vê-se um painel vazio. ⚠️ E a rajada não serviu de
    cura: o ruído do `force.wind` é **por instância**, então alguma das peças pregadas já
    nasce perto do pico — MEDIDO, todos os limiares até 5,5 cruzavam a **0,02 s**. A cura
    é uma carga que sobe com o TEMPO (`value.time → value.map_range → drive_param`), e o
    limiar no meio da subida. *Uma transição só é demonstrável se houver um ANTES na tela.*
56. ⚠️ **Uma janela de medição curta demais diz «não há diferença» sobre uma cena que a
    tem.** A `measure_scene_motion` corria 2 s; a rampa desta cena cruza o limiar aos
    2,05 s, e as duas metades saíam com o MESMO número. ⚠️ E o mesmo vale para o gate: a
    janela dele passou a ser **derivada da rampa** (`RAMP_SECS · 60 + 120`), nunca uma
    constante. *A janela de uma sonda temporal é um parâmetro da CENA, não da sonda.*

57. ⚠️ **O RÓTULO DE UMA CENA PROMETE, e o dele tem de ser o que o modelo entrega**
    (Enio, 2026-08-21: *"funciona mas não rasga o pano (os cubos não se separam)"*).
    Eu chamei a linha «RASGA» e o que ela faz é o **PINO romper** — a folha sai inteira.
    Partir o TECIDO é outra feature, e ela não é uma omissão do knob: o `motion.soft_body`
    guarda a forma por correspondência GLOBAL (Müller shape matching), **sem ligações
    uma-a-uma**, então não há aresta que se possa quebrar. Num solver de arestas
    (`motion.verlet_rope`) a pergunta faria sentido e é uma célula que ninguém abriu.
    ⇒ a linha passou a chamar-se **SOLTA**, e o smoke diz em voz alta o que NÃO acontece.
    *É a mesma lei da memória `feedback_a_label_must_promise_what_the_model_delivers`, e
    é a segunda vez que esta casa a paga.*

58. ⚠️ **Uma expressão enterrada num laço que precisa de meio app para correr é uma
    expressão que ninguém gateia.** A publicação da pose do objeto vivia dentro do
    `publish` do shell, que exige um `SimWorld`, um `TextureAtlas` e um `ObjectBake` — e
    a mutação *"troca `t.rotation` por `0.0`"* **sobreviveu**. Extraída para
    `pose_stream(&Transform) -> Stream`, ela tem um gate de três linhas e a mutação
    sangra. *Se montar a fixture custa mais que a lei, a lei está no sítio errado.*
    ⚠️ E uma segunda mutação sobreviveu por ser **EQUIVALENTE** (uma guarda de
    «pose ausente» que os dois `if let` já implementavam) — a terceira vez esta semana.
    A guarda saiu: ela LIA como uma lei e não era uma.

59. **Uma recusa MEDIDA fecha uma HIPÓTESE, nunca o item.** A célula do *trim/dash* levava
    ⛔ *"a cura foi refutada por medição"*, e a medição estava certa: o `marker::trim_path`
    devolve o caminho intocado em **42 das 47** formas da biblioteca. Só que ele é o recuo de
    ponta para dar lugar a uma seta — o *Trim Paths* do AE chama-se **`fx_trim`**, mede por
    arco exato, **abre** o contorno, e estava testado no repo desde a wave dos Live Path
    Effects, com o cabeçalho a dizer *"keyar `end` de 0 a 1 desenha a forma"*. A recusa
    apontava a função errada e ninguém reconferiu por dois dias. *Leia o que a recusa MEDIU,
    não o que ela CONCLUIU — e pergunte se a hipótese refutada era a única.*
60. **Uma frase sobre a ARQUITETURA envelhece como uma nota, e esta custava a arte inteira.**
    O `motion_shape_gen` dizia *"o nó não tem entradas, então o `ctx.param` dele não tem
    camada conduzida com que discordar"* — e confundia **porta de entrada** com **param
    conduzido** (doc 58), que é um fio para um PARAM e não precisa de porta nenhuma. Medido:
    `drive_param(shape, trim_end, …)` ⇒ o cook devolve contagem **0**, a forma desaparece,
    nada fica vermelho. A cura é o shell resolver o fio pela **porta do cook** (que memoiza,
    então não custa nada). *Uma afirmação de doc que autoriza a NÃO fazer uma coisa é a que
    mais precisa de um gate ao lado.*
61. **Uma premissa de gate DISSOLVE-SE sem o gate ficar vermelho.** O
    `every_node_fits_the_inspector_dock` afirmava *"o painel **não rola**"* — e o painel de
    params rola desde a wave da rolagem, com o `forwarding.rs` a interceptar a roda e um
    arch-gate a guardar a ligação. O gate continuou verde a medir a coisa errada até um param
    novo o acordar. *Um gate cujo corpo afirma um FACTO do produto tem de citar quem o
    mantém verdadeiro; senão ele vira uma nota com `assert!` à volta.* ⚠️ E a recalibração
    dele também errou à primeira: comparar `fundo − conteúdo` contra uma constante do censo
    assumia um crome uniforme, e o `motion.color_array` mede **110** contra os 114 dos outros
    115 nós, porque uma linha de PALETA fecha com outra folga. *Uma segunda aritmética sobre
    o layout diverge do layout* — o oráculo passou a ser **rolar de verdade e perguntar onde
    a última linha ficou**.
62. **Um controle que faz a ARTE SUMIR é pior que um controle morto, e pede o mesmo remédio.**
    O Trim sem traço não é um botão inerte: um contorno aparado é ABERTO, não tem interior, e
    a forma **desaparece**. O `ParamGate` não sabia perguntar *"apareça quando esta grandeza
    sair do zero"* (ele arredonda a inteiro, o que é exato para um `Enum` e inútil para um
    slider de `0..1`), então a família ganhou o irmão **`ParamGateAbove`**. *Quando o gate de
    visibilidade não exprime a condição, o gate é que está incompleto — não a condição.*
63. **Uma sonda que não pode ver a metade que o SHELL publica acusa a cena boa.** A
    `measure_scene_layout` cozinhava num `Cook::new()` virgem: uma cena de FORMA ou de TEXTO
    lê a geometria por canal externo, e um cook sem externals devolve zero instâncias — que a
    sonda imprimia como `VAZIA`. Terceira vez nesta linha (a sonda de movimento e o harness
    do texto pagaram as outras duas). *Um instrumento que não passa pela mesma porta do
    produto mede a si próprio.*

64. **Acrescentar uma PORTA a um nó com kernel RENOMEIA todos os acessores de leitura dele —
    e o compilador não vê nada.** O `codegen::accessor_suffix` qualifica pelo nome da porta
    **quando o nó tem mais de uma entrada**, e o corpo do kernel é uma STRING. Três nós desta
    linha ganharam portas em waves anteriores e ficaram com o corpo a chamar `read_falloff`
    num módulo onde ele passara a chamar-se `read_in_falloff`: **`motion.pin_constraint`**
    (`state`/`load`), **`field.index_range`** (`attr`) e **`force.attractor`** (`target`) —
    os três com o caminho de GPU **quebrado**, e nada vermelho até
    `every_registered_kernel_validates_across_the_whole_presence_space` correr. ⚠️ O `cargo
    check` é cego a isto por construção, e o `applicable` de dois deles escondia-o na maioria
    dos grafos. *Um kernel novo, uma porta nova ou um rename de porta obrigam a rodar o
    validador de WGSL — ele é o único compilador daquela string.*

65. **Um CACHE cuja chave pode mudar a 60 Hz não é um cache — é uma fuga com memória.** O
    smoke da `=76` matou o app: `wgpu error: Out of Memory` no **quadro 19706** (~5 min). O
    `trim_offset` conduzido pelo relógio dá uma chave de conteúdo nova por quadro, e o
    `ShapeBake` guardava **uma textura de GPU por `geometry_id`** sem nunca a libertar — o
    doc-comment dele dizia *"o antigo fica órfão até o `release`"* e o `release` **não
    existia**; o irmão `ObjectBake` pareia `acquire`/`release` desde que nasceu, e este
    assador herdou a frase sem a lei. ⚠️ **Eu tinha MEDIDO o crescimento e escrito o número
    errado**: a sonda contava as entradas do `VecPathStore` (~500 B cada, "30 KB/s") e não
    viu que quem paga é a **VRAM**. *Medir o cache barato e não o caro é medir e ainda assim
    não saber.* ⚠️ E a poda estava bloqueada por um `Vec` indexado por posição — o handle
    passou a ser um **contador** num `BTreeMap`, que remove no meio sem renomear ninguém.
    ⚠️ Terceira metade: o assador corria **sem consumidor**, e um tile por quadro é também um
    **readback de GPU** por quadro. Hoje ele pergunta pelo `fx.glow` pela mesma porta que o
    `present` usa.


---

## §5 — O ritual de cada célula (o que fazer, na ordem)

1. **Leia a célula inteira**, inclusive a coluna *"exprimível?"* — ela costuma trazer o
   mecanismo, e é onde as dez que envelheceram estavam erradas.
2. **Escreva uma SONDA `measure_*`** em `crates/ph2d-node-registry-init/tests/` que tenta as
   rotas de composição e **IMPRIME** (`#[ignore]`, `--nocapture`). Se ela mostrar que o
   catálogo já dá, a célula **envelheceu** — reescreva o veredito com o número e siga.
3. **Só então** escreva o param, com o default que **reduz** ao mundo de antes, e um gate que
   peça `==` sobre isso.
4. **CPU e GPU juntos**, com paridade. Se o nó tem kernel, o corpo WGSL é port linha-a-linha e
   a paridade é quem guarda a igualdade das duas cópias.
5. **Prova de mutação** — RED só conta sobre algo visto VERDE antes.
6. **Uma cena** por grupo, com **CONTROLE** dentro dela. Números que a mensagem cita vivem em
   `const` presos por um gate que lê o fonte da narração.
7. **Reconcilie a `Contagem`** da folha rodando o placar (ele **imprime e sai vermelho**;
   `--write` não existe).
8. **`CLAUDE.md §5` recebe UMA LINHA** — a narrativa vai no handoff.

---

## §6 — Comandos que esta linha usa

```bash
# inner loop
bash scripts/cargo-check-narrow.sh ph2d-node-motion-<nó>

# a suíte de uma crate (exit 0 verde · 1 teste vermelho · 2 não compilou)
bash scripts/cargo-test-narrow.sh ph2d-node-motion-<nó>

# a sonda de uma célula
CARGO_INCREMENTAL=0 cargo test -p ph2d-node-registry-init --test measure_<x> -- --ignored --nocapture

# paridade CPU×GPU (⚠️ skip gracioso NÃO é verde — confirme que o adapter apareceu)
CARGO_INCREMENTAL=0 cargo test -p ph2d-gpu-cook --test gpu_cpu_parity -- --ignored --test-threads=1 <filtro>

# o gate batched, 1× no fim do grupo
CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast

# a superfície de colisão, antes de fechar
bash scripts/collision-surface.sh main
```

---

## §7 — Os smokes que estão pendentes

### `=76` — a folha 14 inteira (o Trim, o tracejado, e o traço que apagava o miolo)

```
env PH2D_GPU_COOK_DEMO=76 cargo run -p ph2d-host-desktop --release
```

Três linhas, duas colunas. **BORDA**: a mesma estrela azul, à direita com borda laranja — e o
miolo continua azul (antes, `stroke_width > 0` deixava a forma OCA). **APARADO**: um anel
branco inteiro contra um trecho de 28% dele que **dá a volta** (o `trim_offset` é conduzido
por um fio — precisa de PLAY). **PICOTADO**: o mesmo retângulo com linha contínua e com linha
picotada. ⚠️ **Pôr o `Stroke Width` em 0 faz o Trim, o Dash e a cor da borda SUMIREM do
painel** — é o `ParamGateAbove`, e a razão é que sem traço aparar a forma a faz desaparecer.


### `PH2D_MOTION_OBJ_SMOKE=8` — a POSE do objeto (folha 14)

```
\
  env PH2D_MOTION_OBJ_SMOKE=8 cargo run -p ph2d-host-desktop --release
```

⚠️ **Não é uma cena `=N`, e não podia ser:** o `source.object` lê um objeto da CENA
(um external publicado do mundo ECS), não do grafo — então o smoke tem de spawnar o
objeto, e isso é o que a família `PH2D_MOTION_OBJ_SMOKE` faz.

### `=75` — duas das três da folha 03 (o pin que rasga, o bando que desvia)

```
\
  env PH2D_GPU_COOK_DEMO=75 cargo run -p ph2d-host-desktop --release
```

Duas linhas, rotuladas no canvas. ⚠️ **SÓ com o PLAY** — as duas são simulação. As
pedras são desenhadas de propósito.

⚠️ **Esta é a v4, e as três correcções vieram de smokes.** A v1 foi reprovada (*"tudo
foi levado pelo vento, nada rasgou"*): o pin estava no laço do integrador, que lê o
`inv_mass` do `rest`. A v2 foi reprovada por *"porque usar grid se temos nós de
tecido?"* — a cortina é agora um `motion.soft_body`. A v3 por *"não rasga"*: o rasgo
estava certo mas acontecia no primeiro quadro, e o vento passou a SUBIR com o tempo.
Leis **52** a **56**.

### `=74` — a folha 02 inteira (o alvo do atrator, e a mira)

```
\
  env PH2D_GPU_COOK_DEMO=74 cargo run -p ph2d-host-desktop --release
```

Duas linhas, rotuladas no canvas. ⚠️ **SÓ se julga com o PLAY** — uma força acumula em
`accel` e é o integrador que aplica; parada, as quatro nuvens são iguais. Os alvos são
os pontos **brancos**, desenhados de propósito: um alvo invisível faz a cena virar
adivinha.

### `=73` — as folhas 08 e 10 inteiras (quatro células, e um nó NOVO)

```
\
  env PH2D_GPU_COOK_DEMO=73 cargo run -p ph2d-host-desktop --release
```

Quatro linhas, cada uma com o nome escrito **no canvas** e uma pergunta de sim/não:
**CORTE** (a fileira acende até o fim?) · **BANDA** (as peças acesas ficam juntas ou
espalhadas?) · **RAMPA** (ela recomeça no meio?) · **FORMA** (o pentágono acende cheio
ou só a borda?). Esquerda = como era, direita = o knob novo.

⚠️ **Esta é a v2.** A v1 foi reprovada (*"tudo misturado e bagunçado"*) e a causa não
era feature nenhuma — era a colocação das bandas correr **depois** do campo, logo
mascarada por ele. Mecanismo e instrumento nas leis **43** e **44** do §4.

### `=72` — a folha 09 inteira + o pareamento do `motion.step`

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=72 cargo run -p ph2d-host-desktop --release
```

Três pares. ⚠️ **O par 3 é o único cujos dois lados têm de ficar IGUAIS** — a leitura é
*"as duas fileiras sobem em bloco"*, e o modo de falha a nomear é *"só a primeira peça da
fileira de baixo anda"* (era esse o defeito: o batimento global chegava ao elemento 0 e a
mais nenhum). ⚠️ **A quarta célula da folha 09 — o kernel de GPU do `color_array` — prova-se
pela AUSÊNCIA de diferença:** rodar de novo com `PH2D_GPU_COOK=0` na frente tem de dar a
mesma imagem.

### `=60` — o ESPAÇO do campo do `motion.noise`

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=60 cargo run -p ph2d-host-desktop --release
```

Quatro blocos em **2×2**, o **mesmo** ruído; muda só o espaço. Julga-se **PARADO**, e o
campo **É o tamanho do ponto** — cada bloco é um retrato dele, não um movimento.
Em cima: controle (manchas redondas) · rodado 45°. Em baixo: listras deitadas · listras na
diagonal. ⚠️ Se um bloco parecer ter pontos **maiores** que os outros, a cena perdeu o
controle — o que muda é ONDE o campo é amostrado, nunca quanto ele vale.

⚠️ **Esta é a v2.** A v1 foi reprovada no smoke (*"não tem nada girado nem na diagonal"*) —
o porquê, e os dois gates que nasceram dele, estão na lei **8** do §4.

---

## §8 — Onde ler

- **Estado do módulo:** `CLAUDE.md §5` (roteador, não história).
- **A conferência:** [`89_conferencia/README.md`](../89_conferencia/README.md) — 17 folhas; o
  placar é **derivado**.
- **O mecanismo das waves desta linha:** [`handoffs/README.md`](README.md) — o índice
  cronológico (⚠️ ele estava **oito** atrás em 18/08; se acrescentar um handoff, reconcilie a
  contagem lendo a pasta).
- **Processo:** DIRETRIZ §1.5 (Modo L) · §1.5.9 (fechar a linha).
