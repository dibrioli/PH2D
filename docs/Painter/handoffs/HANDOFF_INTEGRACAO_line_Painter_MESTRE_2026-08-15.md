# HANDOFF DE INTEGRAÇÃO — `line/Painter` MESTRE (2026-08-15)

> **Estado:** linha **FECHADA**, aguardando **ordem explícita do Enio** para integrar.
> A linha não pusha, não integra, não roda `scripts/foundational-integrate.sh`.
>
> ⚠️ **Este documento SUPERSEDE o [`HANDOFF_INTEGRACAO_line_Painter_linha_procedural_2026-08-15.md`](HANDOFF_INTEGRACAO_line_Painter_linha_procedural_2026-08-15.md)
> apenas como *o que integrar agora*.** O detalhe de mecanismo das waves W1..W6 do plano 38 continua
> **LÁ** e **não foi copiado** para cá — aquele handoff foi escrito no meio da jornada e descreve a
> linha procedural com uma profundidade que este resumo não repete.

---

## 1. A superfície de colisão — MEDIDA, e é a primeira coisa a ler

| | |
|---|---|
| Branch | `line/Painter` · HEAD `48dc0ce14` |
| Commits | **83** contra o `main` de 2026-08-15 |
| Diff | **191 arquivos, +28.120/−2.477** |
| `PROJECT_SCHEMA` | **70 — INTOCADO** (`git diff main...HEAD -- shells/desktop/src/project.rs` é **VAZIO**) |
| `VEC_SCENE` · `FLIP_SCHEMA` · `DOC_VERSION` | **intocados** (fora do diff) |
| Contrato congelado (§6) | **INTACTO** — `ph2d-core/src/tool.rs` e `ph2d-nodegraph/` com diff **vazio** |
| Registro do `ph2d-ecs` | **INTOCADO** ⇒ os **três** espelhos (`ph2d-ecs`/`ph2d-render`/`ph2d-script`) também |
| `Cargo.toml` | **UM** — `crates/ph2d-painter-brush/Cargo.toml` (o `rayon` do ADR-0158) |
| Pacote externo novo | **NENHUM** — o `Cargo.lock` ganha só a **aresta** `ph2d-painter-brush → rayon` |
| Crate nova | **NENHUMA** |
| ADR | **UM: [ADR-0158](../../architecture/decisions/0158-solid-fill-running-sum-is-row-disjoint-rayon-exception.md)** ⚠️ ver §1.1 |
| Ids novos | **todos `hash_node_id`** ⇒ **nenhum gate de contagem**, nenhum id numérico disputado |
| Scrollbar id | **nenhum novo** |
| i18n | **intocado** |

**Crates tocadas:** `ph2d-tool-painter` · `ph2d-painter-brush` · `ph2d-panel-painter-layers` ·
`ph2d-editor-core` (só `src/ids/chrome/`) · `ph2d-render` (só o passe de luz do impasto) ·
`shells/desktop`.

### 1.1 ⚠️ O ADR-0158 é PROVISÓRIO

**0157 é o teto do `main` de hoje** (conferido: `git ls-tree -r main -- docs/architecture/decisions/`),
então 0158 é o próximo livre. Mas **número de ADR escolhido numa linha paralela é provisório**: se
outra linha reivindicar o mesmo na mesma janela, **renumere na integração** — já aconteceu **oito**
vezes neste repo ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

⚠️ **Se renumerar, o rewrite do token é ESCOPADO aos arquivos que a LINHA mudou** — nunca do número
nu sobre a árvore (o `Cargo.lock` carrega números dentro de checksums)
[[feedback_a_token_rewrite_scopes_to_the_changed_files_not_the_whole_tree]]. As citações vivem em:
o próprio ADR, `crates/ph2d-painter-brush/Cargo.toml`, `crates/ph2d-painter-brush/src/solid.rs`,
`docs/Painter/39_auditoria_solid_e_tracos.md` e este handoff.

### 1.2 O que MOVEU e o integrador precisa saber

- **`ph2d-painter-brush` ganhou a 4ª exceção de `rayon` do repo** (as duas do `ph2d-wet-paint`, a da
  `ph2d-sdf`, e esta). A **cerca do ADR-0109 está escrita no `Cargo.toml`**: todo uso novo desta dep
  exige ADR novo.
- **O fingerprint da aquarela** (`smooth_edges_off_is_the_pre_aa_render_byte_for_byte`) **moveu duas
  vezes**: `0xc5ebf8cf645fb6f6` → `0xe59f2fb788ce5874` → `0x9744233f9f852066`. As duas movidas estão
  justificadas com números ao lado do pino, protocolo do doc 23 — **nunca em silêncio**.
- **O fingerprint do Wet Paint (ADR-0134) NÃO se move**: `crates/ph2d-wet-paint/` está **intocada**.

---

## 2. ⚠️ O ponto de merge sensível — UM, e ele PANICA se for fundido mal

**`crates/ph2d-render/src/shaders/impasto_light.wgsl` + `impasto_light.rs`.**

O bit de presença do substrato (`paper_body`) **COMEU a última vaga de padding** do `Globals`
(`pad2`). O uniform **não tem mais folga**: o próximo bit paga um bloco de 16 bytes dos **dois**
lados, de propósito.

⚠️ **O modo de falha não é um teste vermelho, é um PANIC no dispatch** — se outra linha acrescentar
um campo àquele uniform e o merge deixar as duas metades em desacordo, o `wgpu` recusa o bind. Quem
pega é **`the_wgsl_globals_measures_exactly_the_rust_globals`**, e ele existe precisamente porque já
nasceu de um panic idêntico. **Rode-o na árvore combinada.**

Os outros pontos são listas compartilhadas onde a regra é **só ADICIONAR**
([[feedback_a_shared_list_is_merged_against_todays_main]]):

| arquivo | o que a linha faz |
|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/mod.rs` | +2 `pub mod` (`painter_line`, `painter_substrate`) |
| `shells/desktop/src/main.rs` | +2 `mod` (`line_smoke`, `substrate_smoke`) |
| `shells/desktop/src/render_loop/mod.rs` | +2 blocos de roteamento de smoke (+60 linhas) |
| `crates/ph2d-painter-brush/Cargo.toml` | `[dependencies]` era **vazia**; ganha o `rayon` |

---

## 3. O que a jornada entrega

Oito blocos. Os cinco primeiros **nunca tiveram handoff** (a jornada correu por reports do Enio, não
por plano); o sexto tem handoff próprio; os dois últimos são de hoje.

### A — a cauda do smoke anterior

- **A cauda do taper sai de TODOS os modos; fica o início** (o widget também parou de invadir a
  borda direita da seção).
- **Sair do Grid Stamp pousa no método daquele MEIO**, não em `Dots`. ⚠️ **A metade óbvia do report
  já funcionava** — o `settle_stroke_method` já tirava o Grid Stamp desde 09/08; o defeito era o
  **DESTINO** (o assento saía de `offered.first()`, que é a ordem do **dropdown** — uma decisão de
  apresentação — em vez do `BrushSpec::default()`) e a **memória** (Digital/Watercolor/Impasto
  **compartilham** o slot do `PaintMode::Paint`, então os três tinham UM `stroke_method` entre si).
  Nasce `method_by_media[4]`, irmão exato do `last_non_shape_method`.

### B — o SUBSTRATO: o dente do papel vira SUPERFÍCIE para qualquer meio

O emboss do Wet Paint é um gradiente do campo de **massa**. O Digital não tem massa (medido: desvio
do alfa **0,00** no miolo de um traço duro), então a transplantação literal é um bisel de silhueta e
nada mais. O estado da arte responde outra coisa e **converge** — Krita (Phong Bumpmap), ArtRage
(Canvas Lighting), Corel Painter (a tinta assenta nos picos do grão). **Este repo já tinha metade**
(o slot Grain modula o depósito); a wave acrescenta a que faltava.

- O campo é o `BrushSpec::paper` **que já existia e já é neutro** ⇒ **zero linha de aquarela**, ao
  contrário do que a doc 19 estimou.
- O sombreamento é o `impasto_shade::Rig`, que **já é relativo** (dente plano multiplica por 1).
- **A normal do dente SOMA a da tinta** — uma luz só, nunca duas respostas para *"de onde vem a luz?"*.

**Medido** (excursão de luminância, tela nua): `depth` 0,25/0,50/0,75/1,00 com `rough` 0,5 →
**18 / 37 / 56 / 75** níveis. ⚠️ `MAX_TOOTH_PX = 1,0 px` **ancorado no teto que o próprio Wet Paint
declara**; o primeiro valor que escrevi, 3,0, mediu **128** — metade da faixa inteira, chapa
ondulada, e a medição o reprovou.

⛔ **MEDIDO E REJEITADO — não refaça:** um **realce especular** no papel. A Roughness como expoente
do brilho nasceu com **0 texels movidos** (o realce plano é subtraído e clampado; num dente de ~1 px
é nulo em qualquer expoente).

Três correções que o smoke e a auditoria do próprio diff produziram:

1. **O dente atravessa o produtor da GPU.** *"Paper parece não funcionar para Digital"* — e a causa é
   a que esta casa já nomeou várias vezes: **gate verde sobre produto vermelho**, porque a fixture
   exercitava o **outro** produtor. Este app tem dois, e um documento que a GPU compõe — o caso
   normal — **nunca passa pelo laço da CPU**. A cura é **porta única** (`ReliefFields::height_at`),
   não um segundo sítio. ⚠️ **E o gate e2e novo achou uma segunda metade:** o dente era amostrado
   **FORA do canvas** na CPU (um padrão contínuo responde em `x = -1`), onde a GPU só sabe clampar —
   **1149 bytes** de divergência, um anel de um texel começando em (0,0). *Um fold cujo domínio é o
   canvas tem de ser lido no canvas, mesmo quando a fonte sabe responder fora dele.*
2. **O confinamento tem de cair.** *"Retângulos que marcam por onde o pincel passa"* e *"ao aumentar
   o Relief o papel não atualiza em tempo real"* são **o mesmo mecanismo**: o dente cobre **todo**
   texel e nenhum dos knobs que o produzem derrubava o dirty-rect. ⚠️ **A cura é uma TESTEMUNHA, não
   uma lista de setters** — os knobs são nove e sete deles moram num arquivo sobre a **aquarela**;
   uma linha de `invalidate_composite` por setter é a regra que o décimo nasce sem.
3. **Mais três, todos consequência da wave:** o papel **IMAGEM** não alcançava o dente (o terceiro
   consumidor do slot nasceu esquecido, e um dente constante tem gradiente zero) · o papel **não
   sobrevivia a pegar outra ferramenta** (o doc do `set_paper_field` diz *"o papel é do CANVAS; o
   slot é do PINCEL"*, e os oito escritores do slot não o honravam — medido: pegar a Faca devolvia
   `Paper Size` de 5 para 1 e a excursão caía de **166 para 75** níveis) · e um **vermelho-latente de
   LOC** (`impasto_light.rs` em 743 > 700; o gate mora em `ph2d-editor-core/tests/` e um `cargo test
   -p ph2d-tool-painter` **não o alcança** — a família que esta casa já documentou três vezes).

⚠️ **Terceiro report do smoke — `undo/redo` — foi MEDIDO e NÃO construído**, porque a metade que é
deste módulo já funciona e a outra não é deste módulo: com o papel ligado o Ctrl+Z devolve a tinta
**AO BYTE** (residual 0, e agora é gate); e o Relief/Roughness não entram na fila de undo **porque
nenhum knob de ferramenta deste app entra** — medido lado a lado, o card Lighting do impasto (que
shipou e foi smokado) comporta-se **exatamente igual**. Sonda `probe_substrate_undo` põe os três lado
a lado; **a decisão é do Enio**.

### C — o FILME: o depósito de pigmento vira relevo

*"Criar o Relief para a deposição do pigmento com Shape exatamente como faz Wet Paint"* (Enio, com a
foto ao lado). O Digital não tem campo de massa, então a pergunta era **qual campo ele tem** — e a
medição decidiu o desenho inteiro: a **cobertura SATURA** (0,992..1,000 dentro do traço) e um
gradiente sobre um platô é zero (a primeira versão mediu **0,21 nível**). Quem carrega a estrutura é
o **envelope de CARGA**, logo o filme é uma **ALTURA**.

Duas diferenças em relação ao impasto, as duas medidas:

- **NÃO escala com o raio.** Herdar a escala do `derive_height` rende **14,04** num raio 5 e **96,39**
  num raio 40 — sete vezes. O filme é plano: 14,04 / 14,46 / 13,68 / 16,13.
- **NÃO passa pelo `deposits_height`.** Aquele predicado não significa *"escreve altura"*, significa
  *"este pincel deixa um CORPO"*, e quatro sítios do caminho de **cor** o leem para cortar o pigmento
  na borda. Pô-lo lá mudava a **SILHUETA** da tinta ao subir o slider: **61 níveis** que não tinham
  nada a ver com relevo.

**Medido** (`set_substrate_paint`, níveis no pior texel, raio 10): 0,25 → **3,06** · 0,50 → **7,13** ·
1,00 → **14,46**. E o filme **não INVENTA textura, revela a que o pincel já deposita**: redondo macio
**0,21**, Grain Noise **2,00**, Shape Stripes **14,46** — é por isso que o pedido diz *"com Shape"*.

**O smoke aprovou a feature e reprovou três coisas em volta**, e as três fecharam: o **nome** e a
**casa** (`Paint` na seção Paper vira **`Relief` na seção Shape**, com id/setter/estado/código
mudando de casa junto) e o **Shine** — que **já existia** e é o mesmo `BrushSpec::impasto_shine` do
card Material, pelo **mesmo setter**: duas vistas, um valor. ⚠️ A segunda vista existe porque o card
Material vive dentro da seção Impasto e **some quando o meio não é Impasto** — sem esta row, o
material da tinta que o Digital agora deposita seria **inalcançável**, que é o defeito de 2026-07-19
acontecendo de novo. Medido antes de construir: papel nu **0,00** nível; filme com Shine
0,25/0,50/0,70/1,00 → **1,00/1,79/2,57/3,36**.

⚠️ **E a medição pegou o meu próprio gate over-claiming:** eu pedia as rows nos **quatro** meios, e o
filme vale Digital **14,46** / Impasto **1,21** / Watercolor **0,00** / Wet Paint **0,00** — a aguada
e o fluido têm render próprio e nunca cruzam o `derive_height`.

### D — a PERF do depósito, quatro waves

| o quê | antes | depois |
|---|---:|---:|
| **pen-down** do depósito (2048², 7º traço) | 5,62 ms | **1,26** |
| **plano de material**, 1º traço de um documento (4096²) | 21,97 ms | **6,25** |
| **passe de altura**, move a raio 100 com Shape | 4,617 ms | **0,763** |
| alocado por traço (2048², **contado** com dhat) | 83,0 MB | **36,4** |

- **O pen-down pedia os cinco planos da TELA a cada traço.** ⚠️ **O relógio deu a FORMA e ela excluiu
  os dois suspeitos óbvios** (plano no raio: 0,62× e 1,90× entre r10 e r100, onde uma pegada preveria
  100×; plano na tela: 5,62 a 2048² contra 5,69 a 4096²) ⇒ **setup por gesto**, não o primeiro dab
  nem a cópia de canvas. ⚠️ **Qual setup não se responde com um relógio** — as mesmas cinco alocações
  em sequência deram **0,008 / 0,028 / 7,586 ms**, três ordens de grandeza. Quem respondeu foi uma
  **CONTAGEM**. Causa: `reset_stroke_height` faz `clear()` (preserva a capacidade) e o primeiro dab
  do traço seguinte a joga fora numa linha — **duas linhas discordando sobre o mesmo buffer**.
- **O plano de material era preenchido por UM núcleo.** ⚠️ **O Enio autorizou pagar memória em todo
  bind e o preço não foi necessário:** o custo nunca foi o **tamanho**, foi o número de núcleos. O
  `size_to` tem dois ramos e só um tinha sido examinado — o do zero já era de graça
  (`alloc_zeroed`), o do **valor** era um `Vec::resize` serial. ⚠️ **O limiar NÃO foi herdado**: uma
  cópia lê e escreve, um preenchimento só escreve sobre memória recém-alocada; os 32 MB da cópia
  deixavam o plano de material a 2048² (**29,4 MB, a um fio**) na rota serial, que é precisamente o
  caso a curar ⇒ `FILL_PAR_MIN_BYTES = 8 MB`.
- **O passe de altura percorria a pegada numa thread.** O depósito de **cor** blita um carimbo em
  cache; o de **altura** re-deriva a silhueta por texel — **3,75 contra 26-58 ns/texel, 15×**. A
  cura é o primitivo que a cor já usa (`dab::band_count`). ⚠️ **E a fusão que a auditoria prescreveu
  NÃO existe:** o `t` da altura é o da **cápsula** e o da cor é o do **disco** — dois números
  diferentes de propósito (a lei anti-corrugação). ⚠️ **Os contadores da LUT eram por-thread com a
  premissa *"a crate não tem rayon"*** — uma banda é outra thread, e os dois gates de fiação leram
  **ZERO**.
- **O pen-up NÃO tem defeito em regime**, e o resultado da wave é *dizer por quê*: decomposto no
  código que **shipa** (`cfg(test)` nas fronteiras que os blocos já tinham), tudo é **plano na tela**,
  nenhum bloco passa de meio milissegundo, e o total (~1 ms) acontece **uma vez por traço**.
  ⛔ **MEDIDO E REJEITADO, não refaça:** encolher a margem de 28 px do `grow_region` compra **zero**
  (o commit é plano na janela: 2,23 / 2,13 / 2,30 ms com a janela a crescer 2,6×); e preencher o
  plano de material por **duplicação** mede 17,5 contra 18,7 ms — **6%**, porque os dois estão no
  mesmo teto de banda (117 MB em 17 ms = 6,9 GB/s).

⚠️ **Um doc foi CORRIGIDO onde estava:** a seção da banda afirmava que o pen-down do filme era *"a
cópia de canvas"*. É falso — a cópia de canvas é o que o **Digital** paga (1,10 → 3,66 ms de 2048²
para 4096², plane-bound); o que o **filme** acrescenta é plano na tela e plano no raio, logo setup
por gesto. E a tabela do veredito apresentava colunas vindas de **fixtures diferentes** sem dizer
qual era qual.

### E — a AQUARELA: o AA, o entalhe, e o que saiu

**O AA degrauava o CORPO da lavagem.** O `aa_coverage` reconstruía a silhueta em toda vizinhança com
`grad > 0`, e **o doc dele ADMITIA que isso é em toda parte** (*"the feather's plateau scallop keeps
`grad > 0` across virtually the whole wash"*) — **a frase estava escrita como tranquilizante e era o
defeito**. Sobre o corpo, `cw = mx` é uma **dilatação 3×3**, cujo contorno segue a grade discreta:
escadinha por construção. A **Dilution decide se isso é visível** (`flow = 1 - dilution`, então a
0,45 o corpo pousa em ~0,55, **dentro** da janela em vez de saturado acima dela).

Três rodadas, e a terceira é a que vale ler:

1. **O portão do VÃO** (`mx - mn < AA_SPAN_MIN`) — o interior sai de **67 para 0** degraus com
   Dilution. ⛔ **MEDIDO E REJEITADO:** o portão *"o footprint toca o lado de fora"* (`mn == 0`)
   rendia os dois modos **byte-idênticos** e derrubava quatro gates.
2. **O portão DURO fabricava o degrau que devia remover** — um limiar duro sobre uma grandeza
   contínua faz dois texels vizinhos serem calculados por **leis diferentes**. Medido sobre tinta
   seca a Dilution 0,45, o pico: **32,7** sem portão / **81,7** com o duro / **112,7** com o AA
   desligado. ⚠️ **O limiar é uma CURVA DE TROCA medida, não um ajuste** — nenhum valor entrega as
   duas cenas, *o que prova que o vão não é a variável que as separa*; **0,20 é a única linha que não
   é pior que o mundo pré-cura em nenhum dos dois eixos**.
3. ⚠️ **E o Enio corrigiu a raiz:** o falloff do modo watercolor é **fixo** e é o
   `Falloff::Watercolor` (planalto até `t ≤ 0,62`) — **não** o `Falloff::Constant` que as minhas
   quatro rodadas de sonda usavam (Size 0,14 = raio **72**, contra os 26 que eu media). Com o pincel
   **real** o diagnóstico é uma **identidade de bytes**: o portão duro tornava **Smooth Edges um
   CONTROLE MORTO** sob Dilution — `929937a4cb8c423e` é o hash do canvas com o AA **desligado** e com
   o portão duro **ligado**, ao bit. ⚠️ **Os dois gates que eu tinha acabado de escrever foram
   REMOVIDOS, não recalibrados** — eles mediam `Falloff::Constant`, e *um gate sobre um pincel que o
   produto não faz pina ficção*.

**O entalhe no cruzamento** (foto do Enio: cunhas brancas nas quatro quinas côncavas de uma cruz).
⚠️ **O oráculo não precisa de imagem de referência:** duas faixas ortogonais, um ponto na bissetriz a
`s` px de cada eixo recebe de cada faixa o mesmo `f(s)` que receberia dela sozinha ⇒ união ⇒ axila ==
ombro solitário; composição ⇒ `2f − f²`. **Digital COMPÕE** (ao milésimo). **Aquarela não** — e são
**dois** mecanismos: `(a)` a cobertura é `max` por dab (a união, o vinco do FLIP) e `(b)` o **ARO**
(`inner = blur(cov)`: numa quina côncava o borrão vê mais interior, o edge desaparece). ⚠️ **O
contraste que decide o diagnóstico:** dois traços e **um traço cruzando a si mesmo** dão a **mesma
tabela** aos três decimais — no FLIP os dois casos eram diferentes, e *essa diferença era o
diagnóstico dele*. Curado `(b)`, em três passos, o último dos quais é o que fechou: **a régua do aro
sai da COBERTURA** (o mínimo entre a EDT e `(cov − COV_HALF)/|grad cov|`) — resultado nos quatro
cantos **−9% → +3% · −33% → −13% · −11% → −4% · −14% → −4%**, e os **buracos sumiram** (0,0 px de vão
cercado de tinta nas quatro escalas). ⚠️ **A propriedade que ela estabelece NÃO é *"a quina mede a
distância à frente mais próxima"*** — essa frase era minha e o gate a derrubou: a cobertura é um
`max`, logo a propriedade certa é ***mesma cobertura, mesmo aro***. `(a)` fica **não construída** —
muda a aparência de toda cruz/laço/hachura já pintada e **é decisão de produto**.

**E duas coisas SAÍRAM da aquarela:** a **Strength** (*"não é adequado para watercolor. Tire essa
ligação e esconda o slider"*) — ⚠️ eram **três** consumidores, não um, e a ligação foi cortada
**antes** de a row sumir; e o **accumulate do relevo**, construído como **integral de arco** e
**reprovado pela segunda vez** no smoke (o doc 20 e o novo [doc 35](../35_accumulate_vs_blender.md)
guardam a medição contra o Blender para ninguém o reconstruir).

### F — o plano 38: a LINHA PROCEDURAL (W0..W6 + Rough)

⚠️ **O detalhe está no [handoff `linha_procedural`](HANDOFF_INTEGRACAO_line_Painter_linha_procedural_2026-08-15.md)**
e não é repetido aqui. Em uma linha cada: o **card Line** · **Speed Shapes** · **Sketchy** ·
**Wire** · **Spray** (que **não é um tipo de linha** — é um multiplicador de emissão e mora no card
Jitter) · a **FAIXA** (dois trilhos e travessas) · e o **Rough**.

O que aconteceu **depois** daquele handoff e que ele **não** cobre:

- **O pen-up da fita não acrescenta nada** (ordem do Enio). ⚠️ **Revoga uma cerca de Chesterton, e
  ela fica ESCRITA** — a cauda soltava a coleira e percorria até assentar (o *follow-through*, e o
  que o Alchemy e o Dyna do Krita fazem), custando **71 dabs / 84 px** no peso que shipa. ⚠️ **É a
  terceira vez que este pen-up desenha uma reta indesejada, cada vez por outro mecanismo:** a mola
  presa ao cursor (369 px), o cursor envenenado pelo settle, e o crescimento em si.
- **Sem gesto, sem tempo** — uma mola que converge para um alvo **parado** anda em linha reta, e isso
  vale para **qualquer** instante em que a mão para com o botão preso. Ablação pela porta do produto:
  a pausa acrescentava **18.013 px** de tinta, 5.715 deles **escuros**.
- **A fita tem UM caminho** — o `Stroke::settle` era o **segundo** percorredor e saltava até o cursor.
  ⚠️ **Duas cegueiras de fixture, cada uma bastando sozinha:** as cinco fixtures cravavam
  `stabilizer: 0.0` (o valor exato que faz o settle sair no 2º `if`, sobre um produto cujo default é
  0,5 — a irmã exata do `hardness = 1.0` do smear) e as sondas simulavam a pausa entregando um `Move`
  na **mesma posição**, o que põe `moved_this_frame = true` ⇒ **o settle nunca corria**.
- **A faixa avança o que os DOIS trilhos avançaram** (o leque das travessas: **148,5 → 0,0 px**).
- **A fita ganha a tinta de fio, um seam e um diag que conta fios**; e **o card Line ganha seam** —
  os sliders shiparam com id, row, `populate`, encaminhamento e setter e **nenhum gate os
  exercitava**. ⚠️ **O `architecture_panel_wiring_parity` cobre duas condições e é cego às outras
  duas:** *o clique chega ao tool?* e *a sequência leva a algum lugar?*

### G — o Solid usa o PINCEL

Três frases do Enio que são **uma**: *"Solid deve usar o pincel com o falloff e espessura do traço
como no modo flip"* · *"todos os types devem ser compatíveis com solid"* · *"Symmetry e Tiling devem
ser compatíveis"*.

> **`Style: Solid` deixa de ser *a região EM VEZ do traço* e passa a ser *a região SOB o traço*** — o
> modelo do Flip, onde um `FlipStroke` tem `fill: Option<Fill>` e **continua** a ter largura e dureza.

Isso entrega as três metades de uma vez porque as três eram a mesma. ⚠️ **Revoga a §1.1 do plano 38,
e a revogação é do mesmo autor** — os dois gates que pinavam a lei antiga foram **substituídos**, não
apagados.

⚠️ **As duas transações não podem ser uma:** o preenchimento é **sempre** re-carimbo (a cada ponto o
polígono inteiro muda), mas o traço não — metade dos métodos deposita cumulativamente —, e o
`drag_preview` é **um slot só**. ⚠️ **A ordem entre a mancha e o traço não é observável**, e é esse
fato que **permite** as duas famílias usarem transações diferentes: o `over` de duas fontes da mesma
cor é **comutativo** em cor e em alfa.

Depois, dois reports com foto:

- **A mancha segue a TINTA.** Ela era o polígono de `ev.pos` — o percurso da **mão** —, mas metade
  dos tipos existe justamente para pôr a tinta **longe** da mão. **Medido: apenas 8,4%** da tinta de
  um gesto de Speed caía dentro da mancha que ele preenchia, contra **58,4%** do mesmo gesto sem
  efeito. Agora: **58,3% contra 58,4%**. ⚠️ *Um traço tem RAIO, então metade da tinta cai por fora
  mesmo quando as duas coincidem — é por isso que o gate compara com o CONTROLE.*
- **A corda de fechamento leva o pincel.** ⚠️ Os dabs saem do **pincel**, não do último dab do lote:
  a corda é um trecho que a mão nunca percorreu, e herdar pressão/heading a faria **afinar** quando o
  gesto acabasse leve — *o taper de uma ponta que não existe*.

**E o Speed do Alchemy foi ESTUDADO e MEDIDO** (*"em alchemy o traçado de speed é muito melhor"*).
A lei já é a dele, **verbatim do manual**. As duas diferenças foram isoladas uma de cada vez e **as
duas curas ficam**: tirar a **rampa** dá vão/diâmetro **2,77** (uma fileira de arcos desconectados —
**o look que o Enio já reprovou em 13/08**), e trocar a **mira** pelo heading cru dá **vão idêntico**
com raio médio pior — *trocar uma cura medida por um empate é o palpite que a §0 proíbe*. ⚠️ **O que
de facto faltava landou no commit anterior** (a mancha seguir a tinta); o que sobra é **decisão de
produto**, com os números na mão.

### H — a AUDITORIA do Solid e dos traços (ordem do Enio, hoje)

*"Auditoria completa dos traços e solid. Atenção especial para performance em Symmetry Circular +
Tiling"* — e depois *"siga e corrija os abertos"*.

**Dois defeitos de correção, os dois mutation-proven:**

1. **A mancha FECHA o evento — a teia deixa de ser apagada.** Sob Sketchy/Wire com Solid sobravam
   **11,9%** da teia (261 de 2186 texels). O mecanismo é a **ordem do ciclo de traço**: os fios são
   tinta **cumulativa** e caíam **fora** do instantâneo que o `peel` do evento seguinte restaura. Sob
   simetria circular esse retângulo é a **tela inteira**. ⚠️ **O invariante:** *o instantâneo tem de
   conter toda a tinta cumulativa do evento e nenhuma transitória* ⇒ a mancha é a **última** escrita.
   **11,9% → 100,0%.**
2. **O retângulo cobre a pegada REAL da corda sob Tiling.** ⚠️ **O Tiling tem régua PRÓPRIA:** um
   laço é replicado quando a **caixa** passa a costura; um dab, quando `centro ± raio` passa. Um
   caminho colado à borda tem a caixa **dentro** da tela e dabs de corda cuja pegada passa dela — a
   cópia envolvida cai **a um span inteiro** do retângulo salvo, fora de qualquer folga de raio, e
   cada evento deixava um fantasma ⇒ **o desenho passava a depender da TAXA DE EVENTOS**, a lei que
   este módulo já pagou quatro vezes no relevo. **197 → 0.**

**A performance, medida pela porta do produto** (1024², circular 12 + Tiling nos dois eixos, evento
96 de um traço):

| peça | início | depois do `over` | depois do ADR-0158 |
|---|---:|---:|---:|
| construir os laços | 0,029 | 0,029 | 0,028 |
| `solid::fill_coverage` | 1,472 | 1,414 | **1,007** |
| `write_solid` (o `over`) | **1,647** | **0,260** | 0,153 |
| `save` + `restore` | 0,130 | 0,147 | 0,196 |
| **TRANSAÇÃO** | **4,232** | 2,926 | **2,291** |

**1,85×**, em duas waves, as duas por **linhas disjuntas** (ADR-0109). ⚠️ **A rota paralela do `over`
shipava SEM GATE** — toda fixture de Solid roda a 256² (65 k texels) contra um piso de pool de
~131 k, então os 15 gates existentes exercitavam **só a rota serial**.

**O ADR-0158** está resumido na §1 e detalhado no próprio ADR. ⛔ **O depósito das ARESTAS fica
serial pelo mecanismo, não por preguiça** — e a alternativa (pré-binar as arestas por banda) está
**NOMEADA e não construída**, com o número que a reprova hoje: **930 k testes de intervalo, medidos
mais caros que o passe serial inteiro**.

**Mais dois abertos fechados:**

- **O `Rough` nasce VIVO.** `rough_amount`/`rough_bowing` nasciam em 0,0 ⇒ escolher `Rough` saía
  **byte-idêntico** ao `None`. ⚠️ **A nota que justificava o zero contradizia a que está SEIS LINHAS
  ACIMA dela, no mesmo arquivo** (a do Ribbon: *"um tipo escolhido tem de FAZER alguma coisa"*).
  Agora **0,4** — e o gate **não menciona o valor** (pergunta ao predicado que o motor consulta):
  *um default só é testado por um teste que não o menciona*.
- ⛔ **Decimar o caminho de tinta: MEDIDO E REJEITADO.** ⚠️ **O oráculo é a COBERTURA, não a
  contagem** — mover a fronteira `t` px muda a área do texel de borda em `255·t` níveis. Só `tol = 0`
  é de graça (e compra 3% dos pontos); a `0,01` o corte é de 69% mas o pior delta é **12 níveis em
  1724 texels** — **doze vezes o piso do `u8`**. E o ganho encolheu: depois do ADR-0158, decimar
  levaria **~0,27 ms de uma transação de 2,29**.

**E o censo dos seis tipos sob Solid** (gesto em C, canvas 256, r=4): **nenhum tipo é apagado pelo
Solid** — a coluna *tinta COM* é ≥ a do `None` em todos. ⚠️ **Os três zeros têm três naturezas e
nenhuma é defeito da mancha:** o **Wire** dá 0 porque os laços cortam a **concavidade** do C, que é
exatamente a região preenchida — *tinta da mesma cor dentro de uma região dessa cor é invisível por
construção*; o **Speed** dá 0 nas **duas** colunas (fixture: o arremesso é `v·T` e um arco de passos
curtos com o estabilizador ligado quase não tem `v`); e o **Ribbon** pinta 44 texels **com e sem**
Solid ⇒ *o Solid não é a variável* — é pergunta da **fita**, com sonda própria, **nomeada e não
perseguida**.

---

## 4. Mudanças de comportamento — nomeadas

| # | o quê | quem julga |
|---|---|---|
| 1 | **`Style: Solid` desenha o traço** (falloff + espessura), e a mancha é a região **sob** ele | smokado ✅ |
| 2 | **A mancha segue a TINTA**, não o ponteiro — um tipo que joga a tinta para o lado joga a **fronteira** junto | smokado ✅ |
| 3 | **Sketchy/Wire sob Solid mantêm a teia** (era apagada a cada movimento do rato) | ⚠️ **pendente** |
| 4 | **Solid + Tiling perto da borda não deixa fantasmas** | ⚠️ **pendente** |
| 5 | **O `Rough` desenha no default** — é decisão de **LOOK** | ⚠️ **pendente** |
| 6 | **A fita não cresce depois da soltura** (o *follow-through* morreu) | smokado ✅ |
| 7 | **Todo aro de aquarela muda um pouco** (2,4% dos bytes, pior delta 18/255, tudo na banda do aro) e **move num traço RETO de propósito** | smokado ✅ |
| 8 | **A Strength some da aquarela** e o **accumulate do relevo não existe** (2ª reprovação) | smokado ✅ |
| 9 | **O papel sobrevive a pegar outra ferramenta**; a seção Paper abre a **todo** meio | smokado ✅ |
| 10 | **Sair do Grid Stamp pousa no método daquele meio** e o meio **lembra** o que o artista deixou lá | smokado ✅ |

⚠️ **Integrar não é aprovar.** Os itens 3, 4 e 5 são de hoje e **não foram smokados**.

---

## 5. O gate de fechamento — medido em `48dc0ce14`

| | |
|---|---|
| `ph2d-painter-brush` | **407 passed** |
| `ph2d-tool-painter` (release, `--test-threads=1`) | **1234/1234** |
| `ph2d-tool-painter` (**debug**, `--test-threads=1`) | **1232/1232** |
| clippy (as duas crates) | **limpo** |
| `architecture_workspace_file_loc_cap` | **2/2** |
| `shells/desktop` `file_loc_caps` | **2/2** |
| `architecture_adr_numbers_are_unique` · `no_tofu_glyphs` · `no_magic_numeric` | **verdes** |
| `architecture_tool_contract_surface` · `architecture_panel_wiring_parity` | **verdes** |
| `cargo check -p ph2d-host-desktop` | **limpo** |

⚠️ **Rode a suíte do Painter em DEBUG também** — precedente registrado nesta linha (o
`ph2d-flip-colorize` panicava só em debug), e ela **passa**. A diferença 1234/1232 está **contada, não
suposta**: as duas listas de teste são **idênticas** (1526 nomes nos dois perfis) e os dois testes a
menos são exactamente os que carregam `#[cfg_attr(debug_assertions, ignore)]` —
`watercolor_smudge_gate_tests.rs:106` (razão de perf) e `wetpaint/offthread_tests.rs:277` (barra de
wall-clock). *Um bar de relógio mede o PERFIL do build, não o código.*

⚠️ **Os `--ignored` desta crate exigem `--test-threads=1` com a máquina calma.** Sob `load 41` cinco
kills de relógio dão vermelho e **nenhum é código** — o mesmo binário mede 11,36 ms e 5,50 ms para o
mesmo passe. **Nenhuma leitura de relógio desta workstation significa alguma coisa acima de
`load ~5`.**

⚠️ **Gates de GPU:** `crates/ph2d-render/tests/impasto_light_gpu.rs` é `#[ignore]` e precisa de
adapter — **rode-o na RTX** (`cargo test -p ph2d-render --release --test impasto_light_gpu --
--ignored`). Sem adapter ele faz *skip gracioso*, **que não é verde**, e é ele que prova o
`the_paper_alone_survives_the_gpu_producer` e o `the_wgsl_globals_measures_exactly_the_rust_globals`
da §2.

---

## 6. Os smokes

| env | o que perguntar |
|---|---|
| **`PH2D_LINE_SMOKE=1`** | o card **Line**: escolha cada `Type` no dropdown. ⚠️ **A cena dá o material e NÃO arma tipo nenhum** — o `Type` é o seam que ela existe para provar. ⚠️ **`--release` não é preferência** (a densidade cheia do Sketchy põe ~16 mil px de fio num traço de 312 px) |
| **`PH2D_SUBSTRATE_SMOKE=1`** | o **dente do papel**: tela branca, Digital, **Relief em 0 como CONTROLE** |
| **`PH2D_IMPASTO_SMOKE=1`/`=2`** | o controle do relevo/undo |
| **`PH2D_WETPAINT_SMOKE=1`** | o controle do fluido (a crate está **intocada** — tem de ficar igual) |
| **`PH2D_TAPER_SMOKE=1`** | a cauda do taper saiu; o **início** fica |
| **`PH2D_MASK_SMOKE=1`** | o controle da máscara |

**As três perguntas de hoje**, todas em `PH2D_LINE_SMOKE=1` com **Style: Solid** marcado:

1. **Sketchy ou Wire, traço grande:** a teia tem de **sobreviver** ao movimento seguinte do rato.
   Com **Symmetry Circular** ligada é onde o defeito era total.
2. **Perto da borda, com Tiling ligado:** desenhe um laço colado à margem. **Nenhum fantasma** do
   outro lado da tela.
3. **Rough, sem tocar num slider:** o traço tem de **vaguear**. Se o desvio estiver forte ou fraco
   demais, o número é `rough_amount` — é **LOOK**, e a decisão é sua.

---

## 7. Aberto, com o preço ao lado

- ⛔ **A composição da cobertura da aquarela (`(a)` do doc 36)** — muda a aparência de **toda** cruz,
  laço e hachura já pintada. Sobra um vão de **2 px** no cruzamento; os outros 3 já foram. **Decisão
  de produto.**
- ⛔ **O caráter do Speed sem rampa** (o arremesso por PONTO, como no Alchemy) — só é honesto depois
  de o `fill_thrown_gap` fechar o vão que ele abre, e **reabre um look que o Enio já recusou uma
  vez**. Wave própria, com os números na mão.
- ⛔ **Pré-binar as arestas do `fill_coverage` por banda** — vale os 0,94 ms que restam seriais;
  medido, o pré-filtro ingênuo é **mais caro que o passe inteiro**. Counting sort por linha é
  possível e é **wave própria**.
- **`raio 110 · spread 7`** (razão 0,06) não melhora com a régua nova, e **a causa está medida**:
  naquele regime quem está degenerado é o **modelo do aro em pincel grande**, não a quina.
- **A fita e a mancha discordam sobre onde o traço está, e por desenho** — o Ribbon deixa a tinta
  até 800 px atrás do dedo. **O smoke decide.**
- **Sob impasto a mancha é PLANA** e só o traço tem corpo. Coerente com o Flip.
- **Com `Strength < 1` a faixa onde a mancha e o traço se somam fica mais escura** que o miolo —
  inerente a pintar as duas coisas (o Grease Pencil tem o mesmo); o default de 1,0 não o mostra.
- **O undo de knobs de painel** (Relief/Roughness/Shine) não existe — **e nenhum knob de ferramenta
  deste app entra na fila**. Sonda `probe_substrate_undo` põe os três lado a lado; **decisão do
  Enio**.
- **Os 18 ms do plano de material** só saem se ele deixar de ser canvas-shaped ou for pago noutro
  momento — a mesma decisão de produto que o `prewarm` da luz já tem em aberto, pelo mesmo preço.

---

## 8. Documentos que a jornada acrescentou

| doc | o que é |
|---|---|
| [`35_accumulate_vs_blender.md`](../35_accumulate_vs_blender.md) | o accumulate medido contra o Blender — **a 2ª reprovação, com o desenho, para ninguém o reconstruir** |
| [`36_o_entalhe_no_cruzamento.md`](../36_o_entalhe_no_cruzamento.md) | os dois mecanismos do entalhe, a tabela por escala e as duas curas com o preço de cada |
| [`37_pesquisa_tracos_procedurais.md`](../37_pesquisa_tracos_procedurais.md) | Alchemy + o estado da arte (clean-room: comportamento, nunca expressão) |
| [`38_plano_linha_procedural.md`](../38_plano_linha_procedural.md) | o plano do card Line, W0..W6 |
| [`39_auditoria_solid_e_tracos.md`](../39_auditoria_solid_e_tracos.md) | a auditoria de hoje: os dois defeitos, as duas waves de perf, o censo, e os três abertos fechados |
| [`BUGS_painter.md` #23](../BUGS_painter.md) | **a fita divergiu e o processo comeu 90,2 GB** — um teto que limitava a RESOLUÇÃO em vez do TRABALHO |
| [ADR-0158](../../architecture/decisions/0158-solid-fill-running-sum-is-row-disjoint-rayon-exception.md) | a 4ª exceção de `rayon` do repo |

---

*Handoff escrito pela linha em 2026-08-15. A linha PARA aqui.*
