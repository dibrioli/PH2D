# 25 — O Painter na GPU: avaliação medida dos quatro modos

> **Estado:** AVALIAÇÃO. Nada aqui foi construído. Pergunta do Enio (2026-07-23):
> *"é a hora de levar o módulo Painter para GPU (tudo que for possível). Vamos tentar
> superar apps PRO como o Procreate em performance. Faça avaliação geral da
> possibilidade. Todos os modos de pintura."*
>
> **Método:** censo medido antes de qualquer proposta (`CLAUDE.md` §0 — *meça antes de
> limitar*). Harness em `ph2d-tool-painter::tool::paint::measure_gpu_frontier` (lado CPU)
> e `ph2d-render/tests/layer_compositor_gpu.rs::measure_the_stack_depth_on_the_device`
> (lado GPU), os dois `#[ignore]`d. Máquina: RTX 5060 Ti 16 GB / 32 cores / tier
> `workstation`. Release.

---

## 1. A resposta curta

*"Levar o Painter para a GPU"* não é **uma** decisão — são quatro, com preços que diferem
por duas ordens de grandeza. E a medição inverte a ordem intuitiva:

- **O maior ganho disponível hoje não exige residência em GPU nenhuma.** O compositor
  GPU **já existe** e é **66–107× mais rápido** na composição — e **até 885× num arrasto de
  slider de ajuste** — e **recusa o documento comum**. Um único checkbox (máscara de camada)
  despenca a composição de **0,665 ms para 70,9 ms** num documento 4K de duas camadas, e o
  arrasto de um HSB de **0,234 ms para 170,8 ms** a 2048² (**652,9 ms** a 4096² — 1,5 fps).
  Consertar as recusas é uma wave de op-list, não de arquitetura.
- **Um dos quatro modos é genuinamente plane-bound e só a GPU o salva:** o Wet Paint custa
  **16,1 ms/frame a 4096² com a caneta LEVANTADA**. Os outros três são footprint-bound —
  o custo deles não conhece o tamanho da tela.
- **O modo Digital — o que o Procreate faz — já está em 1,2 ms/move, plano no canvas.**
  Portá-lo para a GPU compra ~1 ms/frame e custa a reescrita de residência inteira. É o
  pior trade da lista.

Recomendação em uma linha: **três waves que não exigem residência, medidas e ordenadas por
ganho, e só depois o ADR de residência — que é exigido por Wet Paint e por mais nada.**

---

## 2. O que JÁ roda na GPU (inventário, não promessa)

| peça | onde | estado |
|---|---|---|
| **Compositor de camadas** (22 blend modes, ajustes per-pixel + espaciais, grupos) | `ph2d-render::layer_compositor` — 1397 LOC de WGSL | vivo, com gates de paridade |
| **Luz do impasto** (a óptica do relevo) | `ph2d-render::ImpastoLightPass` | landou 2026-07-18, paridade `worst delta 0` |
| **Premultiply do preview** | `ph2d-render::PreviewPremul` | vivo |
| **Motor de compute genérico** (scan, reduce, grade espacial, JFA, ping-pong de buffers, **zero readback**) | `ph2d-gpu-cook` | vivo — construído pelas linhas de Motion Nodes |

⚠️ **O `ph2d-gpu-cook` não é do Painter, e é o ativo mais subestimado desta avaliação.**
Ele já resolve, em produção, exatamente os problemas que um Painter GPU-residente encontra:
colunas em storage buffers, cadeia de passes num único submit, `pre` (estado do tick
anterior) como refcount de buffer em vez de cópia, e uma política de determinismo escrita
(ADR-0126/0127: a CPU é canônica, a GPU é preview reconciliada por ε). **Não é preciso
inventar a disciplina — ela existe e foi paga por outra linha.**

E o que **não** roda na GPU:

- o **depósito** (a rasterização do dab) — CPU, em todos os quatro modos;
- os quatro **motores de mídia** (wash óptico, campos do impasto, sculpt/deform/smear, o
  solver de fluido);
- o **bake do pen-up** em Watercolor e Impasto;
- e o **canvas em si**: CPU-residente desde o [ADR-0096](../architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md).

---

## 3. O censo medido

### 3.1 Por-move e por-frame, os quatro modos

Traço reto, passo **constante** de 40 px, pincel r=100, mediana de 24 moves.

⚠️ *A primeira rodada deste harness usava um passo que **escalava com a tela** e reportou
todo modo como plane-bound — o fixture andava o dobro da distância a 4096². A tabela abaixo
é a corrigida.*

| modo | move 2048² | move 4096² | razão | composite/move | pen-up 2048² | pen-up 4096² | **tick ocioso** 1024²/2048²/4096² |
|---|---|---|---|---|---|---|---|
| **Digital** | 1,200 | 1,147 | **1,0×** | 0,000 | 1,16 | 1,16 | 0 / 0 / 0 |
| **Watercolor** | 3,075 | 2,999 | **1,0×** | 0,000 | 10,21 | 10,80 | 0 / 0 / 0 |
| **Impasto** | 2,551 | 2,557 | **1,0×** | **2,82** | 19,85 | **31,62** | 0 / 0 / 0 |
| **Wet Paint** | 2,332 | **14,670** | **6,3×** | 0,000 | 2,46 | 15,27 | **3,24 / 4,03 / 16,10** |

Leitura (ms):

- **Três dos quatro modos são footprint-bound.** Quadruplicar a área da tela não move o
  custo do traço. É a propriedade certa e ela já está lá.
- **Wet Paint é o único plane-bound** — e o número que decide é o da última coluna: **com a
  caneta levantada, a 4096², o fluido come 16,1 ms/frame**, o orçamento inteiro de 60 fps,
  para sempre. Nenhum outro modo tem custo ocioso.
- `composite/move` é 0,000 nos três primeiros porque a pilha trivial devolve o `Arc` do
  canvas (zero cópia). O Impasto força o caminho de composição — é ele que paga 2,82 ms.

### 3.2 Como cada modo escala com o PINCEL (2048²)

| modo | r=16 | r=50 | r=100 | r=220 |
|---|---|---|---|---|
| Digital (move) | 0,198 | 0,586 | 1,158 | **0,441** |
| Watercolor (move) | 0,618 | 1,487 | 3,018 | **8,568** |
| Impasto (move) | 0,636 | 1,430 | 2,559 | 5,813 |
| Impasto (composite) | 0,156 | 0,850 | 2,828 | **9,394** |
| Wet Paint (move) | 0,842 | 1,462 | 2,218 | 2,853 |

- O Digital **cai** em r=220 porque o espaçamento é fração do raio: pincel maior = menos
  dabs no mesmo percurso. Os dois termos competem, e o pico fica no meio da faixa.
- **Impasto a r=220 custa 15,2 ms/move** (5,8 depósito + 9,4 composição) — sozinho, um
  frame de 60 fps.
- **Watercolor a r=220 custa 8,6 ms/move**, e o pen-up dele custa mais 10 ms.

### 3.3 A composição de pilha cheia: CPU × GPU, mesma carga

Recomposição **completa** (não o dirty-rect do traço — é o que o artista sente ao alternar
uma camada, arrastar uma opacidade, trocar um blend mode, abrir um documento).

| camadas | tela | **CPU ms** | **GPU ms** | ganho |
|---|---|---|---|---|
| 2 | 2048² | 19,16 | **0,198** | **97×** |
| 2 | 4096² | 70,88 | **0,665** | **107×** |
| 4 | 2048² | 25,37 | **0,311** | **82×** |
| 4 | 4096² | 97,31 | **1,104** | **88×** |
| 8 | 2048² | 36,83 | **0,527** | **70×** |
| 8 | 4096² | 148,82 | **1,968** | **76×** |
| 16 | 2048² | 63,29 | **0,955** | **66×** |
| 16 | 4096² | **254,02** | ⛔ **RECUSADO** (cap 8) | — |

### 3.4 O arrasto de slider de ajuste — o pior número do módulo inteiro

⚠️ **A CPU tem cache próprio aqui, e ele foi honrado na medição.** Uma mudança só-de-param
deixa intocada toda camada ABAIXO do ajuste, então o caminho CPU reinicia de um *cut-point
cache* (`composite_with_cache`) em vez de recompor a pilha a frio. Comparar a GPU contra uma
recomposição fria seria comparar contra um caminho que o produto não toma. A tabela abaixo é
a do caminho quente:

| pilha | tela | **CPU (cache quente)** | **GPU** | ganho |
|---|---|---|---|---|
| 1 raster + HSB | 1024² | 35,86 | **0,078** | **460×** |
| 4 raster + HSB | 1024² | 37,39 | **0,113** | **331×** |
| 1 raster + HSB | 2048² | **170,78** | **0,208** | **821×** |
| 4 raster + HSB | 2048² | 166,56 | **0,372** | **448×** |
| 1 raster + HSB | 4096² | **652,92** | **0,738** | **885×** |
| 4 raster + HSB | 4096² | 657,39 | **1,362** | **483×** |

**653 ms por frame = 1,5 fps** para arrastar um slider de matiz num documento 4K. E o cache
quase não ajuda porque o HSB é a operação de cima: o cache salva as camadas de baixo, e o
custo é o próprio HSB — `cbrt` de OKLab por pixel, O(tela), que nada abaixo dele evita.

⚠️ **Isto só acontece quando a pilha cai da GPU.** Com um raster simples + HSB, o
`flatten_for_gpu` aceita e a GPU faz em 0,208 ms. Basta **uma máscara em qualquer camada**
para o mesmo arrasto passar a custar 170,8 ms.

---

## 4. Os quatro achados, em ordem de custo

### A. ⛔ O compositor GPU existe e RECUSA o documento comum — 107× de penhasco por um checkbox

`painter_gpu_flatten::flatten_for_gpu` devolve `None` — e o documento inteiro cai na CPU —
no instante em que **qualquer** camada tem:

- uma **máscara** (`layer.mask.is_some()`),
- **clipping** (`layer.clipping`),
- é **camada de referência** (`layer.is_reference`),
- um **ajuste mascarado**,
- ou um ajuste de tipo **não portado** — e a lista real é **seis de vinte e quatro**:
  `ColorBalance`, `GradientMap`, `PhotoFilter`, `SelectiveColor`, `ChannelMixer`,
  `BlackAndWhite`. Todo o resto já tem código: 12 por-pixel (`gpu_code` 0..11, inclusive
  Noise/Halftone/ColorLookup) + 6 espaciais (`gpu_spatial_code` 0..5).

⚠️ **Dois doc-comments MENTEM sobre isso e vão enganar quem for fazer a wave** — a lição
[[feedback_stale_comment_and_dead_code_lie]] em duas instâncias vivas:

1. `painter_gpu_flatten.rs`, no cabeçalho: *"a kind with neither a per-pixel `gpu_code()` nor
   a spatial `gpu_spatial_code()` (**e.g. Bloom / Noise / Halftone / ColorLookup /
   ShadowsHighlights**)"* — **os cinco têm código hoje**;
2. `adjustments/mod.rs`, três linhas ACIMA do `match` que os retorna: *"`Bloom` /
   `ShadowsHighlights` … stay `None` (CPU fallback) until their kernel ships"* — e o `match`
   logo abaixo devolve `Some(4)` e `Some(5)`. **O código contradiz o próprio comentário**, e
   há um gate de perf (`gpu_bloom_drag_perf`) que passa exercitando o caminho que o
   comentário diz não existir.

Nada disso é exótico. **Uma máscara é como se pinta dentro de uma forma** — é a operação
mais comum de um app de pintura depois do próprio traço. Medido:

| pilha | 2048² CPU | 4096² CPU | GPU (se aceito) |
|---|---|---|---|
| 2 camadas, simples | 19,16 | 70,88 | 0,665 |
| 2 camadas, **uma com MÁSCARA** | 18,74 | **74,02** | ⛔ recusado |
| 2 camadas, **uma com CLIPPING** | 20,08 | **72,23** | ⛔ recusado |
| 6 camadas, simples | 35,49 | 133,41 | ~1,5 |
| 6 camadas, **uma com MÁSCARA** | 33,90 | **134,67** | ⛔ recusado |

⚠️ **A máscara não torna a CPU mais lenta — ela troca de produtor.** O custo não muda de
forma; muda de ordem de grandeza porque o trabalho vai para a máquina errada. E é
**invisível**: nada na tela diz ao artista que aquele checkbox acabou de multiplicar por
107 o custo de cada mudança de camada — e por **885** o de cada quadro de arrasto de ajuste.

E o custo real da recusa não é a composição — é o **arrasto de ajuste** (§3.4): a mesma
máscara leva um slider de HSB de **0,208 ms para 170,8 ms** a 2048², e de **0,738 ms para
652,9 ms** a 4096². **885× no pior caso medido.**

Este é o maior ganho medido da avaliação inteira, **e não requer residência em GPU, nem
tocar o depósito, nem tocar mídia nenhuma.** É op-list: `Layer` ganha um slot de máscara,
`PushGroup/PopGroup` ganham clipping, e os seis ajustes restantes ganham `gpu_code()`.

### B. ⛔ O teto de 8 camadas a 4K é uma const de 512 MB, não o hardware

```rust
pub const LAYER_CACHE_BUDGET_BYTES: u64 = 512 * 1024 * 1024;
```

A 4096² uma fatia RGBA8 são 67,1 MB ⇒ **8 camadas**. Na nona, o compositor recusa e o
documento volta para os **254 ms** da CPU. A placa tem **16 GB, 14 GB livres**.

⚠️ E o próprio doc-comment da const está **desalinhado com a aritmética**: ele diz *"A 4K
RGBA8 slice is ~33.2 MB, so this 512 MB budget holds ~15 layers at 4K"* — 33,2 MB é
4096×2048 (um frame de vídeo "4K"), não uma tela quadrada de 4096². Para o documento
quadrado que um ilustrador de fato abre, o teto é **8**, não 15.

Isto é exatamente o padrão que o `CLAUDE.md` §0 nomeia: **o caminho mais lento definindo o
teto do mais rápido.** O orçamento deve sair do dispositivo (`adapter.limits()` +
`memory.free`), não de um literal escrito para outra classe de máquina.

### C. ⚠️ Wet Paint é o único plane-bound — e o ADR-0134 já nomeou a cura

Medido: **16,1 ms/frame ocioso a 4096²**; o move sobe 6,3× de 2048² para 4096². O solver
percorre a grade, não a pegada.

E ele é **serial por semântica**, não por preguiça — o [ADR-0134](../architecture/decisions/0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md)
mede e explica: *"o brake do flow lê `wet` VIVO escrito por células anteriores do mesmo
passe, e o drying lê o vizinho esquerdo pós-update"*. Isso é **Gauss-Seidel** — ler as
próprias escritas —, e nenhuma máquina paralela o reproduz bit a bit.

⚠️ **Mas o ADR já escreveu a saída, e ninguém a tomou:** *"o caminho nomeado é redesenho do
solver com snapshot de `wet` no brake (quebra paridade ⇒ re-aceitação §18 inteira)"*.
Snapshot = **Jacobi**. E o achado que muda a economia desta decisão:

> **A mesma mudança que destrava a GPU destrava os 32 núcleos da CPU.** Sem o
> read-your-own-writes, a lei de bandas do [ADR-0109](../architecture/decisions/0109-rayon-exception-watercolor-composite.md)
> volta a valer e o solver vira `par_chunks_mut` sobre linhas disjuntas — **mensurável
> antes de escrever uma linha de WGSL.**

O preço é honesto e não é técnico: **quebra o fingerprint da sessão** e exige re-aceitar a
suíte §18 inteira. O port é 1:1 do JS de propósito; trocar Gauss-Seidel por Jacobi é dizer
que a referência deixou de ser a referência. **É decisão do Enio, não minha** — e é a única
coisa nesta avaliação que precisa de um ADR próprio antes de qualquer código.

### D. ✅ O modo Digital já está em nível PRO — a GPU não é onde está o ganho dele

1,20 ms/move a 2048², **1,15 ms a 4096²** (plano), tick ocioso **0,000**, composição de
pilha trivial **0,000** (devolve o `Arc`). Um documento de uma camada com o pincel digital
já é essencialmente gratuito.

Portar o depósito para a GPU compraria ~1 ms/frame e custaria a residência inteira (§6).
**É o pior trade da lista** — e a razão é estrutural, não circunstancial: o depósito é
limitado pela **pegada do pincel**, e a pegada não cresce com o documento. A GPU ganha
onde o trabalho cresce com a tela, e o depósito não é esse trabalho.

---

## 5. Veredito por modo

| modo | o que domina | porta bem? | por quê |
|---|---|---|---|
| **Digital** | depósito footprint-bound, já ~1 ms | ⚪ **baixa prioridade** | já está rápido; o ganho é ~1 ms/frame contra o custo da residência |
| **Watercolor** | wash óptico per-pixel (8,6 ms/move a r=220) + bake de 10 ms | 🟢 **excelente** | é um **map puro por-pixel** sobre uma janela — literalmente a forma que o [ADR-0109](../architecture/decisions/0109-rayon-exception-watercolor-composite.md) já sancionou para paralelizar, e um map puro é um compute pass |
| **Impasto** | composição (9,4 ms a r=220) + bake (31,6 ms a 4096²) + sculpt/deform/smear (3–8 ms/move) | 🟢 **muito bom, com precedente** | a **luz já porta** (ADR de 2026-07-18, `worst delta 0`); o que falta é o *fold* e os kernels de campo, e o EDT/dilatação/box-blur do sculpt são separáveis — a forma canônica de compute pass |
| **Wet Paint** | solver plane-bound, 16 ms/frame ocioso a 4096² | 🔴 **exige ADR primeiro** | Gauss-Seidel; Jacobi destrava CPU **e** GPU mas quebra o fingerprint do port |

E o compositor, que é transversal aos quatro: 🟢🟢 **já está na GPU, 66–107× na composição
e até 885× no arrasto de ajuste, e recusa o documento comum** — a maior distância medida
entre onde estamos e onde a máquina consegue chegar.

---

## 6. O que a residência custa de verdade

Se o canvas passar a viver em VRAM, isto é a superfície que muda:

- **211 referências a `canvas_rgba` em 51 arquivos** do `ph2d-tool-painter`. Não são todas
  hot path — mas cada uma é um leitor ou escritor CPU que precisa de resposta: readback,
  espelho, ou migração para o device.
- **Os leitores que não são o traço** e não desaparecem: undo (`ModelSnapshot` copia o
  canvas por passo — 67 MB a 4096²), save/persistência, eyedropper, balde de tinta,
  traçado de seleção, `Apply` (o bake no sprite), o preview de stamp, o clone, o inpaint.
- **Os planos por-camada do impasto**: `heights: f32` + `covers: u8` + `mats: [u8;7]` =
  12 B/px por camada pintada — a 4096² são **201 MB por camada**, e o undo os congela.
- **Três regimes de determinismo diferentes** convivendo no módulo, e um port GPU tem de
  dizer a qual pertence:
  1. **fingerprint de sessão bit-exato** (`ph2d-wet-paint` — o port do JS);
  2. **gates de aparência** com literais pinados por gate CPU-only + ε documentado por gate
     `#[ignore]` de device (o template que a luz do impasto estabeleceu);
  3. **replay-hash do CI** (a disciplina "sem rayon", com as exceções sancionadas por ADR).

⚠️ **A boa notícia estrutural: nada disso precisa ser resolvido para colher as Ondas 1, 2
e 3** (§7) — ou seja, os achados **A** e **B** inteiros e os passes de Watercolor/Impasto.
O compositor GPU **já vive com o canvas CPU-residente**: ele faz upload por-camada com cache
versionado (re-envia uma fatia só quando a `version` dela muda) e **não faz readback** — o
resultado vai direto para o slot de preview que o sprite amostra. A residência é uma
pergunta **separada**, exigida por **um** modo (Wet Paint) e por uma otimização que a
medição diz não valer a pena (o depósito Digital).

---

## 7. O plano proposto (ondas, não um port)

Cada onda é fechável sozinha, tem um número medido como alvo, e nenhuma delas depende da
seguinte.

### Onda 1 — **Fechar as recusas do compositor** 🟢 maior ganho, menor risco
Máscara e clipping como ops (`Layer{mask_key}`, `PushGroup{clip}`), os **seis** ajustes que
faltam ganham `gpu_code()`, camada de referência decidida (portar ou nomear como recusa
deliberada), e os **dois doc-comments obsoletos** do achado A corrigidos no mesmo commit.
**Alvo: arrasto de ajuste num 4K com máscara de 652,9 ms → <1,5 ms** (e composição de
74 ms → <1 ms). Não toca residência, não toca depósito, não toca mídia nenhuma; os gates de
paridade CPU↔GPU do compositor já existem e cobrem o caminho novo.

### Onda 2 — **O orçamento sai do dispositivo** 🟢 barato
`LAYER_CACHE_BUDGET_BYTES` vira função de `adapter.limits()` + VRAM observada, com piso para
device pequeno. **Alvo: 16 camadas a 4096² param de cair para os 254 ms da CPU.**
⚠️ E o teto novo tem de vir **medido** — é uma const que a §0 do `CLAUDE.md` proíbe escrever
sem tabela ao lado.

### Onda 3 — **Watercolor e Impasto: os passes que já são maps puros** 🟢
Na ordem do medido: composição do impasto (9,4 ms a r=220) · bake do pen-up do impasto
(31,6 ms) · wash óptico do watercolor (8,6 ms/move) · bake do watercolor (10 ms) ·
sculpt/deform/smear. Todos operam sobre um retângulo delimitado, sem redução entre pixels —
o precedente é a luz do impasto, com o mesmo template de paridade.
⚠️ **O *fold* fica na CPU**, como a luz decidiu: quais camadas, em que z-order, o teto de
vidro. Um shader que o re-derivasse seria a segunda resposta a *"como camadas de tinta se
empilham"*.

### Onda 4 — **O ADR do Wet Paint** 🔴 exige decisão do Enio antes de qualquer código
A pergunta é uma só: *o fingerprint do port JS continua sendo o contrato, ou o produto passa
a ser o dono da física?* Se a resposta for a segunda, o passo 1 é **Jacobi na CPU com
rayon** (mensurável em horas, sem WGSL) e só depois a GPU.

### Onda 5 — **Residência do canvas** ⚪ o ADR grande, e o último
Só se depois das ondas 1–4 o perfil ainda apontar para o upload/readback. Hoje **não
aponta** — e propor a reescrita de 211 sítios antes de colher um ganho de 107× que não a
exige seria a otimização prematura que a memória do projeto proíbe.

---

## 8. Procreate: o que a comparação de fato significa

O Procreate é GPU-residente (Metal), com canvas em tiles, e o teto de camadas dele é
**função declarada do tamanho da tela e da memória do device** — exatamente o trade que o
nosso `LAYER_CACHE_BUDGET_BYTES` faz, só que o deles é derivado do aparelho e o nosso é um
literal (achado **B**).

O que a comparação diz, honestamente:

- **No eixo em que o Procreate compete** — pincel digital sobre pilha de camadas — o nosso
  **depósito já está lá** (1,15 ms/move, plano no canvas; pilha trivial de custo zero) e o
  nosso **compositor é mais rápido que precisa ser** (0,665 ms para 4K/2 camadas). O que nos
  separa não é potência: é o **penhasco de roteamento** do achado A.
- **No eixo em que ele não compete**, nós temos quatro mídias onde ele tem uma: relevo com
  material per-pixel e luz de quatro lâmpadas, aquarela óptica Kubelka–Munk, e um solver de
  águas rasas. Nenhuma delas existe no Procreate. **É por isso que "superar em performance"
  não pode significar só ser mais rápido no que ele faz** — significa que os três modos que
  ele não tem não podem ser o motivo de o app parecer lento.
- E é aí que o **tick ocioso** da §3.1 é o número mais importante desta avaliação: com a
  caneta levantada, três dos nossos modos custam **0,000 ms** e um custa **16,1 ms**.

---

## 9. Recomendação

**Sim, é a hora — mas a ordem certa não é a intuitiva.**

1. Faça a **Onda 1** primeiro. Ela é a maior distância medida entre o produto e a máquina
   (107×), é a que o artista sente todo dia, custa uma op-list e **não tem risco
   arquitetural nenhum**.
2. **Onda 2** junto, porque é o mesmo arquivo e o mesmo dia.
3. **Onda 3** em seguida, na ordem que a §3.2 mediu (impasto antes de watercolor).
4. **Onda 4** só depois da sua decisão sobre o fingerprint.
5. **Onda 5** só se o perfil, depois de tudo isso, ainda apontar para lá.

⚠️ E uma nota que vale para todas: o precedente de engenharia **já está pago**. A luz do
impasto provou o template de paridade CPU↔GPU deste módulo; o `ph2d-gpu-cook` provou que a
casa constrói cadeias de compute sem readback. **Isto não é um salto arquitetural — é
estender duas coisas que já funcionam.**

---

## Apêndice — reproduzir os números

```bash
cd Worktrees/line-Painter

# lado CPU (os quatro modos, o tick ocioso, o penhasco da máscara)
cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1

# lado GPU (a mesma carga no device)
cargo test -p ph2d-render --release --test layer_compositor_gpu -- --ignored --nocapture \
  measure_the_stack_depth_on_the_device
```
