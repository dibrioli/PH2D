# Handoff de integração — `line/Painter`, o CARIMBO (2026-08-03 · 2026-08-04)

> **29 commits.** ⚠️ **PENDENTE DE SMOKE** — nada aqui foi aprovado na tela. A jornada tem três
> metades: o depósito de pigmento fica **10-13× mais rápido na CPU** e depois passa a rodar **no
> dispositivo** quando vale; o Enter do Wet Paint ganhou o gate da metade que ninguém tinha pinado; e
> em **04/08** as duas exclusões que sobravam caíram — o **cap de Accumulate** entrou na rota em banda
> e a **contagem de bandas passou a sair do trabalho**, o que tirou do serial todo dab de raio 64-181
> (o pincel comum). Detalhe nas §8 e §9.

## 1. O que o report do Enio era, e o que ele não era

*"Não houve melhora real nem no modo digital simples. Wet Paint regride para digital ao usar os
strokes vivos."*

⚠️ **A segunda metade estava REFUTADA antes de uma linha ser escrita** — o meio sobrevive aos cinco
métodos de shape (gate `the_paint_media_survives_every_live_shape_method`, e os 14 gates de
`wetpaint_commit` já passavam). O que o artista vê como *"regride"* é o **preview flat estático** do
doc 21: sob autoria o esboço é pintado pelo pipeline normal e só **derrete** no commit. É desenho, e
está gateado.

A primeira metade era real e tinha causa: o `PH2D_PAINT_PERF` — o flag que NOMEIA performance de
pintura — era **estruturalmente cego ao carimbo**. O flush coalescido roda antes do `cpu_start`, então
o log reportava `dispatch p50=0.0` tanto num traço de graça quanto num que custa 300 ms. *Um
instrumento silencioso é pior que um ausente: ele tranquiliza.*

## 2. O depósito divide o LOTE, não o dab (CPU)

O kernel de um dab já se divide entre os núcleos, e o piso dele (`PARALLEL_MIN_AREA`, ~131 k px) está
calibrado **para um dab** — um dab de pincel comum cobre 2,3 k px e nunca o alcança, corretamente.

Só que os métodos de **re-stamp** (Line · Curve · Ellipse · Polygon · Free Hand) não carimbam um dab:
eles re-carimbam a **figura inteira** a cada quadro. Medido, uma elipse de raio 400 são **525 dabs
sobre 722 k px de união** — 5,5× o piso como LOTE, e nenhum deles perto dele sozinho. O depósito rodava
**em um núcleo de trinta e dois**.

⚠️ **E o depósito não é ineficiente, ele é REPETIDO:** o mesmo comprimento de caminho custa **1,02×**
à mão livre e por re-stamp. Por isso a cura é *dividir o lote*, não *acelerar o dab*.

A banda é uma **tela virtual** (a fatia recebe a altura da banda como altura da tela e o centro
deslocado), então não há kernel novo — é o do produto com outra moldura, e o resultado é
**byte-idêntico** (linhas disjuntas, cada banda percorre TODOS os dabs na ordem da lista).

⚠️ **A régua é a SOMA DAS PEGADAS, não a área do bbox**, e a diferença é medida: os dabs de um traço
se sobrepõem ~10×, então perguntar pelo bbox mandava 9,1 ms de trabalho real para a rota serial.

## 3. E então o carimbo foi para o DISPOSITIVO (doc 33)

Crate nova **`ph2d-paint-gpu`**. ⚠️ **Ela não depende da `ph2d-painter-brush`, e isso é a wave inteira
em uma linha:** sem alcance ao `falloff_weight` ela **não CONSEGUE** ter opinião sobre a lei do dab. O
que sobe é a **TABELA** que a CPU encheu com a função que já shipa — a cura do LUT especular do
`ImpastoLightPass`, e ela é **estrutural, não disciplinar** (arch-gate sobre o `Cargo.toml`).

O tool publica dado simples e **não tem device**; a ponte é do shell (o molde do
`denoise_ml_with_progress`). A contenção corta nos dois sentidos.

### 3.1 O predicado ESTREITA, e cada cláusula é uma lei

A rota em banda já exclui Shape, Grain, imagem e o cap de Accumulate. `eligible` tira mais três:
**blend `Mix`** (os outros 23 são 23 leis) · **pigmento RYB zero** (o crossfade lê o alfa do destino
por texel) · **Smooth Edges desligado** (o AA é um passe de nove amostras, não um valor tabelável).

⚠️ **`deposits_height` NÃO está entre elas, de propósito:** o filme é função escalar pura da
silhueta, então ele entra na **tabela** (`film_coverage(body, falloff(t))`) e o depósito do Impasto vem
junto de graça. *Quem pode ser tabelado não precisa ser excluído.*

### 3.2 Três coisas que a medição corrigiu em mim

1. **Transcrevi a função errada.** O shader fazia `stamp_rgba`; o produto nesta rota chama
   `blend_over(Mix, …)`. Previ que a troca fecharia a divergência de 14-300 bytes (a multiplicação
   IEEE-754 é comutativa e **não associativa**). **Refutado — os seis números saíram idênticos.** A
   divergência inteira era a resolução da TABELA: 1 024 reprova · 16 384 → 71 · 65 536 → 18 ·
   262 144 → 8. A tabela foi a 65 536 (256 KB, preenchida uma vez por traço) e a paridade caiu 20×
   (**0,015%**, pior delta 1 nível). A transcrição certa fica pelo motivo que sempre valeu — *é a
   função que shipa* —, não por um ganho que ela não deu.

2. **A escolha de fiação foi medida:** bbox + cópia **2,80 ms** contra linhas de largura cheia
   **7,70**. A cópia dos dois sentidos custa **0,18 ms** — eu havia estimado 1,4.

3. **O eixo que a fronteira escondia: a REDUNDÂNCIA.** ⚠️ Este é o item que teria shipado um defeito.

### 3.3 O piso, e por que sem ele a wave PIORA o produto

As duas rotas escalam com grandezas **diferentes** — a fronteira com a **área da REGIÃO** (sobe e
desce a mesma janela), o carimbo da CPU com as **VISITAS** (`Σ` pegadas). Medido pela porta do artista
(`measure_product_stamp`, 4096², pincel r=155):

| figura | região | visitas | redundância | CPU | device | ganho |
|---|---|---|---|---|---|---|
| 300 | 1,36 M | 8,56 M | **6,3×** | 7,21 ms | 2,75 ms | **2,62×** |
| 600 | 4,05 M | 17,04 M | **4,2×** | 15,83 | 10,66 | **1,48×** |
| 1200 | 13,77 M | 34,00 M | 2,5× | 38,31 | 53,89 | **0,71×** |
| 1900 | 16,78 M | 5,63 M | 0,3× | 18,25 | 82,23 | **0,22×** |

⚠️ **Sem piso, uma figura grande fica 4,5× MAIS LENTA** — e o 6,55× da S2 não dizia isso porque a
fixture dela comparava um passe sobre a **bbox** do artista (1,75 M px) contra o **trabalho** do
artista (17,3 M visitas): uma redundância de 9,9× embutida sem ninguém ter escrito o eixo. *Um número
medido sobre dois lados que escalam diferente não é uma razão, é um ponto de uma curva.*

`wants_device` é a porta única (o padrão do `wants_bands`, um nível acima) e pergunta o trabalho ao
**mesmo `batch_work`** que a rota em banda usa. Piso **4,0**, ajustado das duas retas (~3 ns/px de
região no device contra ~1 ns/visita na CPU) e posto na ponta ALTA de propósito: superestimar o
device manda o lote duvidoso para a rota que já shipa. Com ele, as duas figuras grandes ficam na CPU
a **1,00-1,01×** — sem regressão.

⚠️ **E três gates ficaram VERMELHOS no instante em que o piso entrou:** a figura deles (pincel r=40
numa elipse larga) estava abaixo do piso, então o lote ficava legitimamente na CPU e os gates da ponte
viravam vácuo. **Foi a fixture que deixou de conter o fenômeno, não o produto que quebrou.**

### 3.4 O modo de falha é LENTO, nunca errado

`StampPass::run` **DECLINA** (`Option`) em vez de devolver a base. A v1 devolvia a base num readback
perdido, o que o chamador escreveria de volta como se fosse tinta — **o traço sumiria em silêncio**.
Gate: uma ponte que recusa devolve o lote à CPU **byte a byte**.

**S4 (residência no device) NÃO é necessária** — a v1 que não muda posse já ganha, e o `canvas_rgba`
segue autoritativo na CPU.

## 4. O Wet Paint — VERIFICADO, e o gate que faltava

Pedido: *"depositando o pigmento e a água (como um traço normal) para iniciar simulação ao apertar
Enter"*. **Funciona:** `film = 3262`, `susp = 5,82 M` logo depois do Enter; após os ticks `sett` sai de
0 para **122.218** (a tinta assenta) e o `film` cai para 3189,8 (a água evapora).

⚠️ **Mas a metade da ÁGUA não estava gateada, e é a que o pedido nomeia:** o gate do commit afirmava
`susp + sett > 1` — pigmento. Um commit que despejasse tinta com `film = 0` passaria nos 14 gates da
suíte, e o artista veria tinta que **não escorre**.

⚠️ **Duas lições de mutação** (as duas sobre camadas):
- A água chega ao filme por **duas portas** (a célula virgem e a passada geral), então matar uma só
  deixa o gate verde. Com as duas mortas: `film` **3262,3 → 0,051**, o piso de traço.
- Matar o `g.sett[i]` do `drying.rs` **não** sangra a metade *"a sim anda"*: o `dry_cell` tem **duas
  rotas** (Gauss-Seidel e a independente de ordem do ADR-0147) e eu mutei primeiro a que o produto
  não toma — *um corpo, dois walkers*, a lição que o ADR-0145 já pagou. O que sangra é a porta que o
  gate afirma: **o tick nunca entregar o motor ao worker**.

## 5. O que a integração precisa saber

- **Zero schema** — `PROJECT_SCHEMA` intocado. **Zero contrato congelado** (`Tool=12` /
  `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4`, conferidos por gate). **Nenhum ADR novo**
  ⇒ esta linha fica **fora** de toda disputa de número desta janela.
- **Crate nova `ph2d-paint-gpu`** (deps: `ph2d-gpu`, `wgpu 28`, `bytemuck` — as três já no workspace;
  **nenhuma dep externa nova**). Dev-deps `ph2d-painter-brush` + `ph2d-tool-painter` **só para os
  oráculos**, o `src/` não os toca ⇒ **machete-safe**.
- **`Cargo.toml` tocados: 2** — a crate nova e a aresta do shell.
- **API pública nova:** `ph2d_tool_painter::{DeviceDab, DeviceStamp, DeviceStampJob}` +
  `PainterTool::{set_device_stamp, has_device_stamp}`; `band_diag::DepositDiag` ganhou os campos
  `device`/`visits`/`deliveries` e as quatro fases.
- **Gates de GPU são `#[ignore]`** e precisam de adapter — rodados na RTX: `parity` 2/2,
  `measure_boundary` 2/2, `measure_product_stamp` 1/1. **Sem adapter fazem skip gracioso, que não é
  verde.**
- ⚠️ **Rode a suíte do Painter em DEBUG também** — a linha tem precedente registrado.

## 6. O smoke

```
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, Digital, pincel grande, e **desenhe uma ELIPSE viva** (arraste sem soltar — é o
re-stamp que esta jornada ataca). No log:

- **`deposito:`** diz a rota — `N no DEVICE, M em BANDA, K serial(is)`. Numa figura compacta o device
  tem de aparecer; numa enorme ele fica em zero **de propósito** (abaixo do piso).
- **`re-stamp por entrega:`** dá as quatro fases; o **CARIMBO** é a que esta wave move.
- ⚠️ **Nenhum número deste log significa nada com o `load average` acima de ~5.**

E o Wet Paint:

```
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

Escolha **Wet Paint** no dropdown, desenhe uma elipse e aperte **Enter** — o esboço chapado tem de
**derreter** num traço molhado que escorre, não ficar parado.

## 7. Aberto, com o número ao lado

- O piso **4,0** é ajustado de duas retas medidas **nesta máquina** (RTX, PCIe): ele é uma razão, o
  que o torna mais portável que um absoluto, mas o joelho de outra placa pode diferir. O eixo está
  escrito; o número é reconferível pela sonda.
- A rota do **DEVICE** serve o **pincel de falloff puro**. Shape, Grain, imagem, o cap de Accumulate,
  os 23 blends e o Smooth Edges seguem na CPU — cada um é uma pergunta própria e entra quando for
  medida, não por simetria. ⚠️ **A frase acima é sobre o DEVICE e passou a ser fácil de ler errado
  em 04/08:** o cap de Accumulate **tem** rota rápida hoje, a em BANDA na CPU (§8); o que ele ainda
  não tem é o WGSL, porque o kernel do device não transcreve a lei do cap.
- O `prewarm` **não semeia** os planos da luz (limitação nomeada de antes desta jornada, doc 28
  §4.8.2), e o custo dela é decisão de produto.

---

## 8. 2026-08-04 — o cap de Accumulate entra no lote em banda

O `stamp_plain_dabs_banded` excluía o cap com uma razão escrita no próprio módulo: *"estado
compartilhado (a máscara canvas-shaped)"*.

⚠️ **Compartilhado entre DABS, não entre LINHAS.** A máscara é lida e escrita **por-texel**, no índice
do próprio pixel; bandas são linhas **disjuntas**, então nenhuma banda lê um byte que outra escreve —
exatamente o invariante do ADR-0109 que o `buf` já satisfazia. Uma fatia paralela (`stride` para a
tinta, `stride / 4` para a máscara) e a rota vale para os dois.

⚠️ **E o alcance é maior que o impasto:** `stroke_cover_wanted` dispara em **`strength < 1`**, ajuste
comum de pincel digital. Com um shape editor o lote são centenas de dabs pequenos, nenhum perto do
piso do kernel — e ele rodava num núcleo de trinta e dois.

Três peças:

- **O predicado PARTIU.** O `plain` respondia duas perguntas — *a banda consegue carregar isto?* ×
  *o WGSL transcreve todas as leis?* — e o veto do device tirava a CPU junto.
- **`stamp_dab_textured_masked_with` deixou de ser `#[cfg(test)]`.** O lote pede `usize::MAX` ao
  kernel porque **o paralelismo é do lote OU do dab, nunca dos dois**; aninhar é alcançável e tem
  número (a 4096² com r = 512 uma banda recebe `128 × 1024`, que **é** o `PARALLEL_MIN_AREA` antigo).
- O `serial` virou `fn`: um `&dyn Fn` não segura um `&mut [u8]`.

**3 gates, 3 mutações, 3 sangram, cada uma no seu** — e o par identidade/oráculo não é redundante:
dobrar o passo da fatia sangra só a identidade (as escritas seguem dentro da pegada), perder o `y_top`
sangra os dois.

⚠️ **Três defeitos de fixture meus, os três achados MEDINDO:** `arc(2, 40)` não sobrepõe · a premissa
afirmava o piso do *produto* onde a fixture força `min_area = 0` · e o teto da máscara é **76**
(`coverage × strength`), não 128 — e um dab sozinho já o alcança, então contar texels saturados não
distinguia sobreposição de dab único. A premissa honesta é o **efeito**: com o cap a tinta sai
diferente de sem ele.

## 9. 2026-08-04 — a contagem de bandas sai do TRABALHO (o piso era o knob errado)

Medindo o que a §8 comprou, a sonda `measure_route_cost::what_the_banded_batch_buys_when_the_cap_is_on`
mostrou o lote pagando **2,0× a 3,8× ABAIXO do piso** — e a leitura natural (*"o piso do LOTE está
alto, desacople-o do kernel"*) foi **invertida pela metade (C)**, que mediu o piso do **kernel** pela
primeira vez: ele nasceu escolhido e tem **o mesmo break-even** (~25 k visitas contra 131 072).

⇒ O doc do `BATCH_MIN_AREA` está **CERTO** (*"a pergunta não muda por quem a faz"*). Quem estava
errado é **o número**, e para os dois. Desacoplar teria consertado metade do defeito e enterrado a
outra sob uma justificativa que parecia boa.

**O mecanismo:** `available_parallelism()` devolve o mesmo número para um dab de 7 k visitas e para um
de 500 k, então o pequeno pagava **32 spawns** por 3 µs de trabalho cada — e a única defesa era um
piso que o mandava INTEIRO para a rota serial. *Um cliff é o que se constrói quando o knob certo não
existe.*

**A porta única `ph2d_painter_brush::band_count(area, rows, min_area)`:** o ótimo de
`T·c_spawn + area·c_visita/T` é **`T* = √(area / SPAWN_EQUIV_VISITS)`**, com o `808` saindo da razão
MEDIDA entre 10,5 µs de spawn e 13 ns de visita. ⚠️ **O modelo foi conferido contra o produto antes de
virar código:** para 110 224 visitas em 32 threads ele prevê **381 µs** e a sonda mediu **382**. O
piso passou a ser **derivado** (`SPAWN_EQUIV_VISITS * 4`, onde a raiz alcança 2).

**SEIS cópias da mesma regra viraram uma** — `band_split`, `parallel_band_cached`, três sítios do
`accumulate_batch` (um deles **sem piso nenhum**), o `stamp_banded` do lote e o `stamp_color_dynamic`,
que **re-declarava a constante como literal privado de um bloco**.

Medido pela porta do produto, `serial ÷ banda`:

| | antes | agora |
|---|---|---|
| KERNEL, raio 64 (17 k px) | **serial** | 1,80-1,91× |
| KERNEL, raio 90 (33 k px) | 1,33× | 2,82-2,95× |
| KERNEL, raio 128 (67 k px) | 2,64× | 4,39-4,74× |
| KERNEL, raio 181 (133 k px) | 4,29× | 6,55× |
| LOTE, 2 dabs (14 k) | 0,58× | 1,42-1,55× |
| LOTE, 8 dabs (55 k) | 2,01× | 3,02-3,08× |
| LOTE, 16 dabs (110 k) | 3,77× | 4,25-4,38× |

Raio 20 / 2 dabs dá **1 banda, 1,00×** — corretamente serial. **Nenhuma linha da tabela piorou**, e o
ponto de operação do editor de figura (1,69 M visitas/lote) já saturava os núcleos: a wave não o move,
ela move tudo o que está **abaixo** dele.

⚠️ **E ZERO byte se move:** bandas são linhas disjuntas e cada uma percorre todos os dabs na mesma
ordem, então o resultado é bit-idêntico ao serial **para qualquer contagem de bandas** (HR-5) — a
invariante que o doc do `band_split` já declarava.

**Três gates pinavam o piso antigo e foram RE-DERIVADOS contra a medição, não afrouxados.** O que o
piso protege não é o traço à mão livre (medido, ele **paga** 1,5-2×) e sim o lote que não enche duas
bandas. Os dois controles agora **DECLARAM de que lado da cerca estão**, que é o que os fez falhar
alto em vez de seguirem verdes medindo o outro lado. **5 gates novos, 5 mutações, 5 sangram.**

⚠️ **E a sonda pegou uma regressão de 10× que EU introduzi e que nenhum gate de identidade podia ver:**
pus o `available_parallelism()` — um **syscall** — na frente do early-out, e o lote o chama **uma vez
por dab POR BANDA** (128 × 32 = 4096 chamadas). A rota em banda foi de **0,99 para 9,79 ms** com a
MESMA contagem de bandas. Curado por `OnceLock` + a recusa antes de perguntar à máquina, com
**arch-gate sobre a fonte** (a única forma de o próximo `cores()` escrito à mão nascer vermelho em vez
de custar outro smoke).

### O que a integração precisa saber (adendo à §5)

- **Zero schema, zero contrato congelado, zero ADR, nenhuma dep nova, nenhum `Cargo.toml` tocado**
  nestas duas waves.
- **API pública nova em `ph2d-painter-brush`:** `band_count` e `PARALLEL_MIN_AREA` (este **mudou de
  valor**: `131 072 → 3 232`, e passou a ser derivado). `stamp_dab_textured_masked_with` deixou de ser
  `#[cfg(test)]`. `SPAWN_EQUIV_VISITS` e `band_count_with` são `pub(crate)` — não atravessam a crate.
- ⚠️ **`ph2d_tool_painter::…::wants_bands` mudou de assinatura** (recebe o `work`): ela é
  `pub(super)`, não sai da crate.
- **Dívida de typos drenada:** um subjuntivo pt-BR de *depositar*, num doc-comment do
  `wetpaint_commit` (commit `18c9b1f47` desta mesma linha), colidia com o inglês *deposit* — reescrito
  para *"volte a depositar"*, porque o critério do `.typos.toml` é *allowlistar só quando isso NÃO
  pode esconder um typo real*, e este repo diz "deposit" o tempo todo. ⚠️ O gate de typos **não roda
  no fechamento por crate**, só no `ship.sh` — e ⚠️ a primeira versão desta linha **repetia a palavra
  literal** e reprovava o gate que ela documenta.

### Aberto, com o número ao lado

- **O `SPAWN_EQUIV_VISITS = 808` é desta máquina** (32 núcleos, `std::thread`). Ele é uma **razão**
  entre dois custos, o que o torna mais portável que um absoluto, e o ótimo é **plano** em volta dele
  (errar por 2× custa ~2%) — mas o número é reconferível pela sonda em outro hardware.
- ⚠️ **A evidência do 10,5 µs foi consumida pela própria cura:** o piso chato de 0,33-0,39 ms que toda
  linha pequena mostrava era `32 × 10,5 µs`, e ele não existe mais nas tabelas de hoje. Quem quiser
  re-derivar o número tem de **ablacionar a contagem**, não reler a tabela.
- **Quatro sítios ainda chamam `available_parallelism()` fora da porta** — `stamp_color_cache`,
  `compositor::compose`, `selection_overlay` e `warp/transform_float`. Eles são por-OPERAÇÃO (um
  composite, um warp), não por-dab-por-banda, então o syscall ali é ruído; mas eles têm **o mesmo
  cliff de contagem constante** que esta wave curou, e nenhum foi medido.
- **A rota do DEVICE segue exigindo `!accumulate_cap`** — o kernel WGSL não transcreve a lei do cap.
  É a decisão que sobra do §7, e agora ela custa menos, porque a CPU deixou de ser a rota lenta.
