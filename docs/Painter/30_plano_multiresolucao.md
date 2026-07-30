# 30 — MULTI-RESOLUÇÃO: o fluxo é grosso, o pigmento é da tela

> ## ⚠️ A FASE 1 RODOU E REESCREVEU O DESENHO — leia esta seção antes do resto
>
> A F1 existia para medir o risco #1 (§2.4: *a redução pode comer o ganho*).
> Ela mediu, e **derrubou três coisas que este plano afirmava** — as três
> minhas. Números reproduzíveis (3 corridas apertadas), poça de 1,66 M células
> a 4096², sonda `ph2d-wet-paint/tests/measure_flow_reduction.rs`:
>
> | grandeza | medido |
> |---|---|
> | `build_flow_field` | **9,88 ms** |
> | `smooth_velocity` | 0,44 ms |
> | `project` | 1,05 ms |
> | **MÉDIA** de 8 planos, `rf=4` — `O(finas)` | **3,69 ms** |
> | **AMOSTRA** de 8 planos, `rf=4` — `O(grossas)` | **0,29 ms** |
> | ablação: o **backrun** dentro do `build_flow_field` | **+0,06 ms** |
> | ablação: o **fingering** | +0,04 ms |
>
> **(1) A REDUÇÃO É A ROTA ERRADA — a certa é AMOSTRAR.** Mediar é `O(finas)`
> por construção (lê toda célula fina) e por isso **não encolhe com `rf`**: a
> 3,69 ms ela custa *mais* que os dois passes que ela alimentaria (1,49 ms
> somados). Amostrar — ler UMA célula fina por bloco — custa **0,29 ms e
> encolhe por `rf²`**, 12,7× mais barato. ⚠️ Não é a mesma resposta (uma gota
> de 1 px pode cair ENTRE dois pontos de amostra), mas o campo de fluxo é
> **suave por física** — é a premissa inteira do inkwash —, e a pergunta que
> sobra é de **APARÊNCIA**, decidida por render-and-look, não por aritmética.
>
> **(2) O `build_flow_field` NÃO PRECISA SER FATORADO.** A §2.5 chamava a
> fatoração de *"o item de maior risco da wave"* e a punha na Fase 3, na teoria
> de que o backrun (que espalha PIGMENTO) prendia o passe na grade fina.
> **Medido: o backrun custa 0,6% do passe** e o fingering 0,4% — **99,4% é o
> NÚCLEO** (leveling · capilar · viscosidade · freio), que é exatamente a parte
> que quer ser grossa. A fatoração dissolve: o backrun fica onde está.
>
> **(3) O GANHO É 1,3×, NÃO 1,7× — e eu errei pelo motivo que o doc 28 §5.40 já
> tinha documentado.** A §1.4 abaixo soma os passes **sem a CADÊNCIA**, e o
> `build_flow_field` roda **÷4**. Amortizado:
>
> | | hoje | depois (`rf=4`, alimentado por amostra) |
> |---|---|---|
> | `build_flow_field` ÷4 | 2,47 ms | 0,23 |
> | `smooth_velocity` ×¾ | 0,33 | 0,10 |
> | `project` ÷3 | 0,35 | 0,04 |
> | **lado do fluxo** | **3,15 ms/passo** | **0,37** |
>
> ⇒ nesta poça **10,3 → 7,5 ms (1,37×)**; na escala do PRODUTO (62 ms/passo)
> **62 → ~47 ms (1,32×)**. E o que sobra é `advect` (26,2) + `drying_pass`
> (15,5) = **88%**, os dois FINOS e os dois já nomeados como *"não ganham
> nada"*.
>
> **⇒ ESTA WAVE NÃO É SOBRE VELOCIDADE.** O `Grid Size` que já shipou compra
> **9,1×** na razão 4 — 25× mais que isto — e o preço dele é o pigmento grosso,
> que é *exatamente a foto do Enio*. A entrega aqui é **a BORDA FINA com o
> fluxo barato**; o 1,3× é troco. O plano segue válido; a justificativa muda de
> lugar, e é honesto que ela mude.

> **Estado: as fases F2..F6 em construção.** Ordem do Enio (2026-07-30, com foto):
> *"Ainda não temos o AA funcionando! … Fique muito esperançoso com a
> possibilidade de grade grossa só para velocidade/pressão, pigmento e wetness
> na resolução da tela. Mas que cada ajuste desses seja colocado na UI junto ao
> nosso slider. Planeje, salve o plano e prepare-se para implementar."*
>
> Antecedentes: doc 28 §5.41 (a razão única de grade, que shipou) · a pesquisa
> de estado da arte de 2026-07-29 · ADR-0146 (a GPU, em proposta).

---

## 0. Por que o AA não funcionou, e por que isso APONTA para esta wave

Duas curas foram construídas e as duas estão certas no que fazem — e nenhuma
podia resolver a foto:

* **smoothstep no upsample** — remove a quebra de derivada na emenda das
  células (os *blocos quadrados*), e removeu: a ampliação 6× confirma;
* **cobertura na entrada** (supersampling da silhueta) — remove a decisão
  binária *"esta célula está dentro do pincel?"*, e remove: um dab isolado ganha
  uma célula de penumbra.

⚠️ **Mas a borda da FOTO não é a borda do pincel — é a borda da ÁGUA depois de
correr.** Ela é uma **isolinha de um campo que só existe na grade grossa**, e
sobre essa isolinha os dois AAs são impotentes por construção:

> Interpolar um campo que vai de 255 a 0 **em uma célula** espalha a transição
> por `ratio` px, mas a POSIÇÃO em que ela cruza meia-tinta continua saltando de
> `ratio` em `ratio`. Suavizar a rampa não endireita a escada — **a informação
> de sub-célula não existe no dado**.

A pesquisa é categórica no mesmo ponto, e por isso esta wave é a resposta certa
em vez de um terceiro AA:

> **⛔ Não faça upsample do campo de pigmento. Nenhuma app documentada faz.**
> (§6/§7 da pesquisa — a indústria **evita** o problema; não achei um único caso
> de sharpening/edge-preserving upsample de pigmento em app de pintura.)

⇒ **A cura é não ter o problema:** o pigmento nunca é upsampled porque nunca é
grosso. É exatamente o pedido do Enio.

---

## 1. Estado da arte — e o que foi TENTADO e abandonado

### 1.1 O desenho que estamos copiando: inkwash (WebGL2, medido a 60 fps num telefone)

Campos, com a resolução de cada um ([about](https://johnowhitaker.github.io/inkwash/about)):

| campo | resolução | por quê |
|---|---|---|
| `Ink` RGBA16F | **~2048** | *"that's where edges, granulation and fine linework live"* |
| `Fixed` RGBA16F | ~2048 | idem |
| `Wet` R16F | ~2048 | idem |
| `Velocity` RG16F | **~256** | *"fluid motion is inherently smooth"* |
| `Pressure` R16F | **~256** | *"the pressure solve (the expensive part) scales with cell count"* |

E a frase que resume a arquitetura inteira:

> *"The sleight of hand of the whole app is **sampling a blurry, cheap flow
> field to push around a sharp, expensive ink field**."*

⚠️ **E é por isso que lá não existe problema de serrilhado:** o único campo que
sobe de resolução é a velocidade, que (a) é suave por física, (b) é consumida
por advecção semi-lagrangiana, que **já** amostra em posições fracionárias com
bilinear, e (c) **não tem borda** — não existe isolinha de velocidade a
antialiasar.

### 1.2 Quem mais desacopla, e como

| sistema | grade de sim | saída | fonte |
|---|---|---|---|
| **Curtis 1997** (o pai do nosso modelo) | *"lower-resolution"* | ampliada p/ display | ✅ paper |
| **Stuyck/Adobe 2016** (óleo, iPad) | **1024×768** | **2048×1536** | ✅ Tabela 2 |
| **inkwash** | vel/pres **256** · pigmento **2048** | 2048 | ✅ about |
| **Rebelle** | = canvas (CPU) | export ML até 32k | ✅ NanoPixel |
| **Expresii** | modesta | 12k+ (hybrid vector-raster) | ✅ SIGGRAPH |
| **Procreate / ArtRage** | **não simulam** | — | ✅ |

⚠️ **Ninguém documenta rodar a sim completa a 1 célula/pixel em 4096².** Nós
somos o único caso, e é por isso que o problema apareceu aqui.

### 1.3 O que foi TENTADO e abandonado — as três que importam

1. **Adobe, WetBrush → Fresco: a porta direta falhou.** *"WetBrush required
   high-end hardware… we couldn't get it to run fast enough for deployment"* —
   e a saída foi **simulação por TILES**: *"It's about the **communication among
   tiles**, to allow them to flow"* (Byungmoon Kim). Ou seja: quando a
   resolução não cabe, a indústria **fatora o domínio**, não acelera o kernel.
2. **Krita, Instant Preview** — proxy de baixa resolução para o feedback do
   traço, com o preço documentado por eles mesmos: *"**popping** when the stroke
   is finished"*. ⚠️ **É a doença `seed ≠ sample` que esta linha pagou quatro
   vezes.** Não vamos por aí: nosso proxy não pode ser *outro* resultado.
3. **Wavelet Turbulence (Kim et al. 2008)** — sintetizar detalhe de alta
   frequência a partir da sim grossa. ⛔ **Mau encaixe, e o motivo é preciso:**
   ele sintetiza **turbulência no campo de velocidade**; nosso artefato é a
   **borda de um escalar**. Adotá-lo seria resolver o problema errado com o nome
   certo.

### 1.4 ⚠️ E o achado que a MEDIÇÃO derruba: o "expensive part" do inkwash é 1,3% do nosso

Decomposição por passe, poça do PRODUTO (4096², r=100, janela 8,42 M células,
1,61 M ativas — `measure_what_a_step_of_the_products_puddle_is_made_of`):

| passe | ms | % | o que ele é |
|---|---|---|---|
| `build_flow_field` | **60,9** | **42,9 %** | produz `flow` de `vel`+`film`+`wet` · **e o backrun espalha pigmento** |
| `drying_pass` | **46,6** | **32,8 %** | `susp` ↔ `sett` — **pigmento puro** |
| `advect` | 26,2 | 18,5 % | move `film`+`susp` pelo `flow` |
| `rebuild_active_region` | 4,9 | 3,5 % | lê `film`/`susp`, escreve `active` |
| **`project`** | 1,9 | **1,3 %** | ← *o pressure solve, "a parte cara" do inkwash* |
| **`smooth_velocity`** | 1,1 | **0,8 %** | só velocidade |
| `apply_boundaries` | 0,2 | 0,1 % | — |

⚠️ **Um split ingênuo — "velocidade e pressão vão para a grade grossa" — renderia
2,1 %.** A extrapolação que a pesquisa fez do perfil da Adobe (Velocity 10 % +
Height Field 30 % = 40 %) **não transporta**: a nossa cadência já roda o
`project` ÷3 e ele é Jacobi barato, enquanto os nossos caros são passes de
CAMPO que tocam pigmento. *A razão de outra pessoa não é a nossa* — a mesma
lição que a §5.40 pagou com duas fixtures "grandes" incomparáveis.

**O ganho REAL vem de perguntar por PASSE, não por plano:**

| passe | pode rodar grosso? | por quê |
|---|---|---|
| `build_flow_field` | **SIM** (42,9 %) | o produto dele **é** o campo suave; o backrun é efeito lateral a fatorar |
| `project` | **SIM** (1,3 %) | pressão sobre `film` — o caso canônico |
| `smooth_velocity` | **SIM** (0,8 %) | gather de velocidade |
| `advect` | **NÃO** | é o *"sharp ink field"*; fica fino, **amostrando** o flow grosso |
| `drying_pass` | **NÃO** (32,8 %) | pigmento puro — não ganha nada |
| `rebuild_active_region` | híbrido | a máscara viva é do pigmento (fina) |

⇒ **~45 % do passo cortado por `rf²`.** Com `rf = 4`: 45 % → 2,8 % ⇒ o passo cai
a ~58 % ⇒ **1,7×** — **e o pigmento fica na resolução da tela**, que é a metade
que a foto pede.

⚠️ **E o `drying_pass` (32,8 %) vira o novo topo**, sem ganho nesta wave. Isso
fica NOMEADO agora para ninguém descobrir depois (candidatos futuros: a cadência
dele já é adaptativa; o §5.40 mostrou que ele reconverte `susp_rgb`).

---

## 2. O desenho

### 2.1 A ideia em uma frase

**Dois grids, e a fronteira entre eles é UMA função.** O grid FINO (pigmento,
água, wetness, papel — a resolução do canvas ÷ `Grid Size`, que o slider atual
já controla) e o grid de FLUXO (velocidade, pressão — o fino ÷ `Flow Ratio`).
O passe de fluxo lê o fino **reduzido** e escreve o grosso; a advecção lê o
grosso **amostrado** e move o fino.

### 2.2 As portas ÚNICAS (duas portas divergem em silêncio)

| pergunta | porta única | quem chama |
|---|---|---|
| de que tamanho é cada grid? | `grid_map::grid_dims` (já existe) + `flow_dims` | o nascimento da sessão |
| onde, em células FINAS, está este ponto de canvas? | `px_to_cell` (já existe) | a rota do dab |
| onde, em células de FLUXO, está esta célula fina? | **`fine_to_flow`** (novo) | os passes de fluxo |
| **que célula FINA esta célula de fluxo amostra?** | **`flow_probe`** (novo — a rota que a F1 escolheu) | `build_flow_field`, `smooth_velocity`, o momento |
| qual é o fluxo NESTA posição fina? | **`FlowSample::at`** (novo) | o `advect` |
| que pixel de canvas é o centro desta célula fina? | `cell_center_px`/`_texel` (já existe) | silhueta, Grain, Paper |
| de que células finas sai este pixel? | `SampleU::at` (já existe) | o composite |

⚠️ **`fine_to_flow` e `flow_probe` são INVERSAS** (`fine_to_flow(flow_probe(c)) == c`),
e é essa identidade que substitui a que a §2.4 pedia da redução: um passe que
lê a célula-amostra de um bloco e escreve o fluxo daquele bloco tem de
concordar sobre QUAL bloco, senão o campo sai deslocado meia célula — a doença
`seed == sample`, a mesma que as quatro portas do `grid_map` já pinam.

⚠️ **`FlowSample::at` é a porta que decide a wave.** Ela é chamada pelo `advect`
(que precisa do fluxo na posição da partícula) e pelo `project` (que devolve
velocidade). Se as duas amostrarem com leis diferentes, a água anda num campo e
a pressão corrige outro — **e nada nos números denuncia**: a poça fica
"estranha" e ninguém sabe dizer por quê.

⚠️ **`fine_to_flow` é a INVERSA de `FlowSample::at`**, pela mesma razão que
`px_to_cell` e `cell_center_px` têm de ser inversas (a lição
`seed == sample`): a redução escolhe *de onde* o fluxo lê, a amostragem escolhe
*para onde* ele volta, e meia célula de discordância é uma poça que deriva.

### 2.3 O que fica ONDE

| plano | grid | por quê |
|---|---|---|
| `susp`, `susp_rgb`, `sett`, `sett_rgb` | **FINO** | *"edges, granulation and fine linework live here"* — é a foto |
| `wet` | **FINO** | é a máscara que a tinta respeita; grossa, a borda do molhado degrau |
| `paper` | **FINO** | o dente do papel é textura de canvas |
| `film` | **FINO** | ⚠️ ver §2.4 — é a decisão mais delicada |
| `active`, `bloom` | **FINO** | a máscara viva segue o pigmento |
| `vel_x`, `vel_y` | **FLUXO** | suave por física |
| `flow_x`, `flow_y` | **FLUXO** | idem |
| pressão (transiente do `project`) | **FLUXO** | *the expensive part scales with cell count* |

### 2.4 ⚠️ A decisão delicada: o `film` — **DISSOLVIDA pela F1**

> A F1 mediu e o dilema desta seção **não existe**: com a rota de AMOSTRA
> nenhum plano fino é reduzido, então o `film` simplesmente **fica FINO** e o
> passe de fluxo lê a célula-amostra do bloco. Não há cópia grossa a manter, e
> por isso não há a pergunta *"de quem é a verdade sobre a poça?"*. A seção
> fica como registro do que foi pesado.
>
> ⚠️ **O que a F1 abriu no lugar, e é REAL:** o `advect` **escreve `vel` por
> célula FINA** (`flow` amostrado na fonte + `gravidade × film LOCAL`). Com
> `vel` residente no grosso, essa escrita não tem para onde ir ⇒ a
> **atualização de momento migra para um passe COARSE próprio**, e o `advect`
> fino passa a transportar só MASSA. É literalmente o desenho do inkwash (*um
> campo de fluxo borrado e barato empurrando um campo de tinta nítido e caro*)
> e é o que muda os números — o re-pin do fingerprint (§2.6) é dele.

### 2.4-histórico — o dilema, como estava escrito

O `film` (profundidade de água livre) é **os dois**: é a fonte do gradiente de
pressão (que quer ser grosso) **e** a borda visível da poça no véu + o que o
`rebuild_active_region` usa (que quer ser fino).

**Recomendação: FINO, com uma REDUÇÃO para o passe de fluxo.** Razões:

1. a borda da poça é exatamente o artefato da foto — pô-la grossa reintroduz o
   problema no véu e no `rebuild`;
2. o `advect` move `film` junto com `susp`, e separá-los faria a água e a tinta
   viajarem em resoluções diferentes — **duas respostas para "onde a poça
   está"**;
3. a redução `film` fino → grosso é uma **média de `rf²` células**, que é
   exatamente o que o gradiente de pressão quer ver (a pressão é uma média).

⚠️ **Custo desta escolha, nomeado:** o `build_flow_field` passa a REDUZIR
`film`/`wet`/`susp` antes de rodar. A redução é `O(células finas)` — ou seja, ela
**não** desaparece com `rf`. O ganho fica em `O(finas)` de redução +
`O(finas/rf²)` de trabalho, contra `O(finas)` de trabalho hoje. **Isto tem de
ser MEDIDO na Fase 1 antes de qualquer outra coisa** — se a redução custar o que
o passe custava, a wave morre aí e é barato descobrir.

### 2.5 O que fatorar no `build_flow_field` — **DISSOLVIDO pela F1**

> A fatoração desta seção era *"o item de maior risco da wave"* e a F1 a mediu:
> **backrun +0,06 ms · fingering +0,04 ms · núcleo 9,83 ms.** O que prendia o
> passe na grade fina custa **0,6%** dele. O passe inteiro roda grosso e o
> backrun vai junto — ele lê e escreve na célula AMOSTRADA, o que muda o
> desenho do padrão de backrun (menos sítios de nucleação, mais espaçados) e é
> uma pergunta de **aparência para o smoke**, não de custo. A seção fica como
> registro.

### 2.5-histórico — a fatoração, como estava escrita

Ele faz hoje **três** coisas distintas:

1. o **freio de look-ahead** (lê o `wet` VIVO, `sqrt`, sonda de acesso aleatório);
2. o **backrun** (espalha `susp[nb]`/`sett[nb]` — **pigmento**);
3. o **fingering** (`libm::sin/cos`, gated à borda de avanço).

⇒ **(1) vai para o grid de fluxo; (2) FICA no fino** (é pigmento, e é o que dá o
padrão de *backrun* que o artista reconhece); (3) é da borda, fica fino.

⚠️ **Isto é uma REFATORAÇÃO do passe, não uma mudança de dimensão de buffer** —
e é o item de maior risco da wave. O plano o isola na Fase 3, depois de a Fase 1
já ter provado o ganho com `project` + `smooth`.

### 2.6 O fingerprint

⚠️ **Esta wave MUDA os números** (o fluxo passa a ser calculado em média sobre
`rf²` células), então o `tests/fingerprint.rs` do ADR-0134 **tem de ser
re-pinado**. O protocolo já existe e já foi exercido duas vezes nesta linha:

* doc 23 (`wetLift`): pin novo + **o pin ANTIGO virou um gate** — `wetLift = 0`
  é o modelo anterior AO BYTE;
* doc 24 (tabela sRGB): o caminho default ficou byte-idêntico.

⇒ **O molde aqui: `Flow Ratio = 1` é o modelo de hoje AO BYTE**, com gate
provando, e o pin novo com justificativa. É a mesma lei que fez a razão 1 da
§5.41 ser byte-idêntica: **a identidade não se pede a um épsilon**.

---

## 3. Contrato congelado (§6) e schema — a prova

| superfície | encosta? | prova |
|---|---|---|
| `NodeOp` / `OpResolver` / `NodeManifest` | **não** | outro módulo (nodegraph) |
| `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` | **não** | a wave não muda assinatura de trait — o slider entra pelo `PanelEvent::SetValue` que já existe |
| `VectorOp` etc. | **não** | outro módulo |
| `PROJECT_SCHEMA` | **não** | a razão é estado de TOOL (`WetPaintState`), não serializado — como `grid_ratio` hoje |
| `FLIP_SCHEMA` / `DOC_VERSION` / `VEC_SCENE` | **não** | outro módulo |

**A conferir por grep ANTES do primeiro commit** (não por auto-relato — a lei
que a §5 aplica em toda integração):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
cargo test -p ph2d-editor-core --release --test architecture_tool_contract_surface
cargo test -p ph2d-editor-core --release --test architecture_contract_surface
git grep -n "PROJECT_SCHEMA: u32" -- shells/desktop/src/project.rs
```

⚠️ **A superfície pública que MUDA é a da `ph2d-wet-paint`** (`Grid` ganha
dimensões de fluxo; `build_flow_field`/`project`/`advect` ganham parâmetro), e
ela **não é contrato congelado** — mas o `Engine::on_dab` já precisou de `+ Send`
uma vez, então a regra vale: **aditivo, com default neutro**.

---

## 4. A UI — as 4 condições, por controle

O Enio pediu: *"que cada ajuste desses seja colocado na UI junto ao nosso
slider"*. ⇒ a seção Wet Paint abre com um **grupo de resolução**, e o slider de
hoje passa a ser o primeiro de dois:

```
WET PAINT
  Grid Size (px)     [  1 ]   ← já existe: px de canvas por célula FINA
  Flow Grid (x)      [  4 ]   ← NOVO: células finas por célula de FLUXO
  ─────────────────
  [Paint][Erase][Smear][Blend][Wet][Dry][Blow]
  …
```

⚠️ **Dois números, não um.** Eles respondem perguntas diferentes — *"quão fino é
o pigmento?"* e *"quão grosso é o fluxo?"* — e colapsá-los num só seria a falha
de duas-portas ao contrário: um controle que governa dois fatos independentes.
O **readout derivado** (§4.1) é o que impede que isso confunda.

### 4.1 O readout que torna os dois legíveis

Uma linha de texto sob os dois, **derivada, nunca autorada**:

```
   fluido 4096² · fluxo 1024²        (com Grid 1, Flow 4)
```

⚠️ Sem ele o artista não tem como saber que *Grid 2 + Flow 4* dá uma grade de
fluxo de 512² — e a lei desta linha é que **um limite que não se vê é um limite
que o artista descobre por acidente** (a duração autorada da timeline, o teto do
impasto).

### 4.2 As 4 condições, por controle (independentes)

| condição | `Flow Grid` | como se prova |
|---|---|---|
| **o componente EXISTE** | `PAINTER_WETPAINT_FLOW` + `WetPaintState.flow_ratio` | gate de modelo |
| **é pintado e registrado** | `card_row` + entra em `PAINTER_WETPAINT_FIELDS` (8→9) | `architecture_panel_wiring_parity` + o gate de posição (abaixo do Grid Size, acima das tools) |
| **o clique chega ao barramento** | `PanelEvent::SetValue` → `set_wet_flow_ratio` | seam com `click_at` **real** (não `WidgetEvent` sintético — a lição das 36 células da física) |
| **a SEQUÊNCIA leva a algum lugar** | mover o slider → a sessão encerra → o próximo traço nasce com o fluxo grosso → **o passo fica mais barato e a borda não muda** | gate de comportamento + o readout refletindo |

⚠️ A quarta é a que pega o caso *"tudo gateado e o gesto não leva a lugar
nenhum"* — aqui ela é dupla: o passo **fica mais barato** E a borda **continua
fina**. Um dos dois sozinho não é a wave.

---

## 5. Os gates, red-first, e a fixture que contém o fenômeno

| # | gate | fixture / oráculo | mutação que sangra |
|---|---|---|---|
| G1 | **`Flow Ratio = 1` é o modelo de hoje AO BYTE** | o fingerprint antigo, congelado como gate (o molde do doc 23) | qualquer redução ativa em `rf = 1` |
| G2 | **a borda do PIGMENTO é fina em qualquer `rf`** | ⚠️ **render-and-look + a métrica de ESCADA por PERÍODO** (§5.1) | pigmento no grid de fluxo ⇒ a escada volta |
| G3 | as duas portas são inversas (`fine_to_flow` ∘ `FlowSample::at`) | varredura de razões, como o gate de inversão da §5.41 | meia célula em qualquer uma |
| G4 | **a água vai para o MESMO lugar** com `rf` 1 e 4 | centroide + alcance do escorrido após N passos | redução com peso errado (a poça deriva) |
| G5 | o passo fica mais barato — **RAZÃO**, não wall-clock | mesma poça, `rf` 1 vs 4 | o passe de fluxo rodando fino |
| G6 | a UI: existe · pintado · clicável · leva a algum lugar | 4 gates independentes (§4.2) | cada um o seu |
| G7 | **o readout diz a verdade** | `Grid 2 + Flow 4` ⇒ "fluido 2048² · fluxo 512²" | readout derivado de um só dos dois |

### 5.1 ⚠️ A fixture do G2 é o ponto mais difícil, e já errei três vezes

A serra RMS do contorno **não serve** — medido, ela dá **2,28 px na razão 1** e
0,45 na razão 4, porque mede a granulação esparsa do banco de cerdas (~5 % de
cobertura/célula), não a estrutura de grade. *Número no lugar errado diz o
contrário da foto.*

**O oráculo que serve** é a **periodicidade no período `r`**: a escada de grade é
uma ondulação do contorno **no período exato da célula**, e a granulação é banda
larga. ⇒ o gate mede a energia do contorno na frequência `1/r` (uma correlação
simples, sem FFT), com **a razão 1 como CONTROLE** (onde essa energia tem de ser
piso de ruído).

⚠️ E o gate **tem de nascer VERMELHO** contra o build de HOJE (o Grid Size
sozinho), que é onde o fenômeno está. Se ele passar no estado atual, ele está
medindo outra coisa — e essa foi exatamente a armadilha das três tentativas.

---

## 6. As fases (nenhuma depois da 1 é reversível de graça)

| fase | o que | o que ela decide | estado |
|---|---|---|---|
| **F1** | **medir** como alimentar a grade de fluxo | MÉDIA (3,69 ms) × AMOSTRA (0,29) · o backrun não prende nada · o ganho é 1,3× | ✅ **feita** — ver o topo |
| **F2** | a grade de FLUXO EXISTE e `Flow Ratio = 1` é **byte-idêntico** | a rede de segurança de tudo que vem depois (fingerprint intacto) | |
| **F3** | `project` + `smooth_velocity` na grade de fluxo | os dois mais simples (não escrevem plano fino nenhum) | |
| **F4** | `build_flow_field` grosso + o **passe de MOMENTO** que sai do `advect` | o que muda os números ⇒ o re-pin | |
| **F5** | o `advect` amostra o fluxo grosso (`FlowSample::at`) | o acoplamento fino↔grosso, e é onde a poça pode derivar | |
| **F6** | a UI (os dois sliders + o readout), o re-pin do fingerprint, os gates | — | |

⚠️ **A ordem é a da SEGURANÇA, não a do ganho:** a F2 não entrega um
milissegundo e é a fase mais importante — ela é o que torna toda fase seguinte
falsificável por um gate de byte-identidade contra o modelo que shipa.

---

## 7. O que esta wave NÃO é

* ⛔ **não é um terceiro AA** — o smoothstep e a cobertura ficam (eles curam o
  que curam, e o de entrada **se paga**: 0,280 ms/move a 8:1 contra 2,081 a
  1:1). A multi-resolução torna os dois **desnecessários no pigmento**, o que é
  a confirmação de que o desenho é o certo;
* ⛔ **não substitui o `Grid Size`** — ele continua sendo o controle de *quão
  fino é o pigmento*, e é ele que dá os 2,7-9× de hoje. O `Flow Grid` é
  ortogonal;
* ⛔ **não é a GPU** (ADR-0146) — e ⚠️ **ela muda o cálculo de lá**: com o fluxo
  grosso, os passes que o ADR-0146 nomeia como *"93,1 % que mudam os números"*
  encolhem, então o ADR terá de ser re-precificado depois desta wave. *Quem move
  o número reconfere a nota.*

---

## 8. Riscos nomeados

1. **A redução pode comer o ganho** (§2.4) — F1 mede.
2. **A poça pode derivar** se `fine_to_flow` e `FlowSample::at` discordarem —
   G3/G4, e o modo de falha é silencioso.
3. **O fingerprint muda** — protocolo do doc 23, com o pin antigo virando gate.
4. **A borda do véu** (show-wet) lê `film` — que fica FINO, então ela melhora
   junto; mas o **menisco** lê `film[i±1]`, e num `film` fino o gradiente muda de
   escala. ⚠️ Verificar por render-and-look, não por número.
5. **`drying_pass` (32,8 %) não ganha nada** e vira o topo — nomeado agora.
