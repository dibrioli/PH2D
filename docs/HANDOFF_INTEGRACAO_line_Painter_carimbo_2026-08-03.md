# Handoff de integração — `line/Painter`, o CARIMBO (2026-08-03)

> **13 commits.** ⚠️ **PENDENTE DE SMOKE** — nada aqui foi aprovado na tela. A jornada tem duas
> metades: o depósito de pigmento fica **10-13× mais rápido na CPU** e depois passa a rodar **no
> dispositivo** quando vale; e o Enter do Wet Paint ganhou o gate da metade que ninguém tinha pinado.

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
- A rota do device serve o **pincel de falloff puro**. Shape, Grain, imagem, o cap de Accumulate, os
  23 blends e o Smooth Edges seguem na CPU — cada um é uma pergunta própria e entra quando for
  medida, não por simetria.
- O `prewarm` **não semeia** os planos da luz (limitação nomeada de antes desta jornada, doc 28
  §4.8.2), e o custo dela é decisão de produto.
