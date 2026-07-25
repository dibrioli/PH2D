# 25 — O Painter na GPU: avaliação medida dos quatro modos

> **Estado:** a avaliação virou WAVE. **Ondas 1 e 2 CONSTRUÍDAS** (2026-07-23) —
> resultado medido na §10. O resto (Onda 3 em diante) segue por fazer.
> A análise abaixo é o censo que decidiu a ordem, e fica como está: ela é o
> *porquê*, e os números dela são o "antes". Pergunta do Enio (2026-07-23):
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


---

## 10. O que as Ondas 1 e 2 entregaram (2026-07-23)

### 10.1 O resultado, medido dos dois lados

O documento **mascarado** — o caso do achado A, e o mais comum que existe depois
do próprio traço:

| documento | tela | **antes (CPU)** | **agora (GPU)** | ganho |
|---|---|---|---|---|
| 2 camadas, uma **mascarada** | 2048² | 18,74 ms | **0,213 ms** | **88×** |
| 2 camadas, uma **mascarada** | 4096² | 74,02 ms | **0,741 ms** | **100×** |
| 6 camadas, uma **mascarada** | 4096² | 134,67 ms | **1,641 ms** | **82×** |
| 1 raster + **HSB mascarado** | 2048² | 170,78 ms | **0,239 ms** | **715×** |
| 1 raster + **HSB mascarado** | 4096² | **652,92 ms** | **0,831 ms** | **786×** |

E o número que resume a wave: **a máscara passou a custar +9%**, não uma troca de
máquina — 0,741 ms contra 0,680 ms do mesmo documento sem ela a 4096².

O teto de camadas (achado B), na mesma tela de 4096²:

| camadas | antes | agora |
|---|---|---|
| 8 | 2,05 ms | 2,05 ms |
| **16** | ⛔ **recusado → 254 ms na CPU** | **3,88 ms** (**65×**) |

### 10.2 O que mudou

1. **A recusa de `is_reference` era um bug puro** — o compositor CPU, que é a
   referência, **nunca lê o flag** (é *geometry source for ColorDrop*, §2.9). Um
   documento inteiro ia para o produtor lento por algo que não muda um pixel.
2. **Máscara e clipping viraram OPS.** `LayerOp::Layer` ganhou os dois
   modificadores de cobertura, `LayerOp::Adjustment` ganhou a máscara, e a ordem
   de dobra é a da CPU exatamente: *decode → máscara → clip → opacidade*.
3. **O orçamento de camadas vem do dispositivo** (1 GiB discreta / 512 MiB
   compartilhada), com o número **medido** e o recurso **nomeado**.

### 10.3 As decisões que a implementação teve de tomar

Todas espelham a referência CPU, e cada uma tem um gate:

- o **clip base é o alpha CRU** — antes da máscara *dela* e da opacidade *dela*;
- **clipping ENCADEIA**: duas camadas clipadas seguidas leem a MESMA base;
- **grupo e ajuste QUEBRAM** a cadeia, e um grupo abre a própria ⇒ o `clip_base`
  é um array **por profundidade**, não um escalar;
- a **máscara do ajuste multiplica a FORÇA**, nunca a cobertura — um ajuste
  mascarado desvanece o efeito, não fura a arte;
- **máscara não-servível = SEM máscara**, não erro (a CPU cai em *fully visible*).

⚠️ **Grupo mascarado/clipado CONTINUA recusado, de propósito.** O braço de Group
da referência CPU **não lê nenhum dos dois**; honrá-los na GPU faria a mesma arte
depender de qual produtor ganhou o frame — roteamento invisível para o artista, o
pior resultado disponível. É cerca de Chesterton com o motivo escrito e um teste,
e o marcador para quem for fechar o buraco: **conserte a CPU primeiro.**

### 10.4 Higiene que veio junto

Três doc-comments **mentiam** sobre o roteamento e foram corrigidos com o porquê
(o flatten LÊ essas funções para decidir GPU-vs-CPU, então prosa velha ali é
orçamento de frame perdido): a lista de recusa nomeava *"Bloom / Noise / Halftone
/ ColorLookup / ShadowsHighlights"* como sem op GPU — **os cinco têm**; o
`gpu_spatial_code` dizia que Bloom/S-H *"stay `None` until their kernel ships"*
três linhas acima do `match` que devolve `Some(4)`/`Some(5)`; e o
`feathers_coverage` se dizia *"BROADER"* que o conjunto espacial, sendo
subconjunto estrito. Agora as três pertinências são pinadas por **enumeração** de
`AdjustmentKind::ALL`.

E o doc do orçamento errava a **própria aritmética**: *"~33.2 MB a slice, so 512 MB
holds ~15 layers at 4K"* — 33,2 MB é 4096×2048, um frame de **vídeo**; a tela
quadrada que um ilustrador abre custa 64 MiB, e a resposta real era **8**.

### 10.5 Gates

5 de paridade no **device real** (rampa de máscara nas duas polaridades · cadeia
de clip com a base *usando* máscara — o único fixture que distingue alpha cru de
mascarado · cadeia por profundidade através de grupo · máscara de ajuste que NÃO
mexe na cobertura · máscara não-servível), **cada um com controle positivo** —
comparar duas implementações passa igualmente bem quando as duas ignoram o
modificador. Mais 3 de flatten, 2 de elegibilidade no shell, 2 de POD (tamanho
**e ordem de campos**: trocar `mask_slot` com `flags` mantém 32 bytes e lê índice
de slice como bitfield) e 1 do orçamento por-dispositivo.

**7 mutações, 7 sangram, zero sobreviventes.**

### 10.6 O que segue aberto

- **Onda 3** (Watercolor/Impasto) e **Onda 4** (o ADR do Wet Paint) intocadas.
- Os **6 ajustes** não portados seguem sendo recusa: `ColorBalance`,
  `GradientMap`, `PhotoFilter`, `SelectiveColor`, `ChannelMixer`, `BlackAndWhite`.
  ⚠️ Nem todos cabem no orçamento de 3 escalares do `AdjParams` — PhotoFilter e
  BlackAndWhite querem mais, e o GradientMap é literalmente um LUT de 256
  entradas, ou seja **já cabe na máquina de `adj_luts` que o Curves/Levels usa**.
  É a próxima peça de melhor razão custo-benefício.
- Um **grupo mascarado/clipado** exige fechar o buraco na CPU primeiro.
- Subir o orçamento além de 1 GiB exige **alocação falível**
  (`push_error_scope(OutOfMemory)`), não um literal maior.
- Uma **máscara de ajuste ESPACIAL** ainda cai na CPU: o passo de combine do
  pass-graph não tem entrada de máscara.

## 11. Onda 5a — a pintura para de copiar o canvas inteiro por movimento (2026-07-24)

Smoke do Enio: *"imagem 2048x2048 brush size de 0.5, pintura simples, com queda de
FPS. parece dependente de CPU."* Pincel pequeno + tela grande + queda de FPS = custo
por-frame **O(canvas), independente da pegada** — se fosse o depósito, um pincel
minúsculo não derrubaria nada. **Nada disto é o compositor** das Ondas 1–2: um traço
simples numa camada é trivial e nem chega ao compositor GPU; ele toma o caminho **CPU**
(`take_preview_arc` + upload), que é exatamente o *"parece dependente de CPU"*.

### 11.1 O que a medição mostrou

O depósito é barato; o custo era um **`Arc::make_mut` do canvas inteiro por
movimento**, forçado porque a shell segurava um clone do `canvas_rgba` vivo do tool (em
`painter_preview.rgba`) atravessando o frame — então o próximo `stamp_dabs` via um 2º
dono e copiava o plano todo antes de carimbar o dab.

| canvas | raio | segurando (bug) | soltando (tool dono único) | razão |
|---|---|---|---|---|
| 2048² | 1 px | 0,340 ms/move | 0,003 | **132×** |
| 2048² | 16 px | 0,441 | 0,095 | 4,6× |
| 4096² | 1 px | **10,285** | 0,003 | **3851×** |
| 4096² | 16 px | 9,779 | 0,091 | 107× |

O custo *segurando* é **plano no raio** (0,34/0,33/0,37/0,44 ms a 2048² para raios
1..16) — é a cópia do canvas, que não liga para o dab. A coluna *soltando* é o alvo.

### 11.2 A causa é a MESMA que o ADR-0124 já curou no áudio

A shell segurava o clone para detectar mudança por **identidade de ponteiro** (o
`make_mut` devolvia ponteiro novo numa mudança). É o anti-padrão *"pergunte a versão,
nunca o ponteiro."* Fix:

- **`PainterTool::canvas_version()`** — contador monotônico, bumpado 1× por drain sujo.
- A shell **possui o próprio buffer de preview** (`own_preview_buffer`: cópia cheia no
  seed, `buffer anterior + patch da região suja` num frame parcial) e **solta** o `Arc`
  drenado. O tool fica **dono único** do `canvas_rgba` → escrita **in place**.
- Sem mudança de struct: o `arc_token` do slot (token opaco de mudança, compartilhado
  com o bgremoval) carrega a **versão** para o painter; `0` segue o sentinela do
  produtor GPU (força upload cheio no handoff GPU→CPU).

### 11.3 Resultado, medido no caminho REAL (`own_preview_buffer`)

`a_plain_stroke_is_footprint_bound_when_the_shell_owns_its_buffer`:

```
OWN  (fix)  2048²=0,096 ms  4096²=0,097 ms   razão 1,0x   (footprint-bound)
HOLD (bug)  2048²=0,441 ms  4096²=9,834 ms   razão 22,3x  (o plano copiado)
```

A 4096² um traço simples vai de **9,8 → 0,1 ms/move (~100×)** e passa a ser **plano no
tamanho da tela**. O depósito não foi tocado ⇒ a **aparência é byte-idêntica**; esta
onda é custo, não pixel.

⚠️ **A residência de canvas na GPU (Onda 5) segue por fazer e segue não sendo
necessária para ESTE problema** — a medição mostra o depósito CPU já barato (0,003–0,6
ms/move) para todo pincel realista uma vez removida a cópia. A Onda 5 vira otimização de
escala extrema / liberar a CPU, não o conserto do FPS reportado.

### 11.4 Gates (5, mutação-provados; + 1 lever consertado)

- `the_shell_owns_its_preview_buffer_never_the_tools_canvas` — buffer INDEPENDENTE +
  byte-fiel (mut `Arc::clone(drained)` → `ptr_eq` RED).
- `canvas_version_advances_on_a_dirty_drain_and_holds_on_an_idle_one` (tool) — mut
  dropar o `+= 1` → `0 -> 0` RED.
- `the_paint_drain_owns_its_preview_buffer` — arch-gate sobre o fonte do drain (roteia
  pelo helper, nunca `rgba: drained`).
- `a_plain_stroke_is_footprint_bound...` — razão perf (`#[ignore]`), fix vs o controle
  que segura o Arc.
- Os gates de pipeline foram religados à **versão** + a um **oráculo de verdade
  INDEPENDENTE** (um bug de patch da região não se esconde atrás de um slot derivado do
  próprio mirror).
- ⚠️ **Latente da Onda 2, que o gate de handoff GPU-adapter nunca rodou:** o lever dele
  era *"máscara flipa elegibilidade para a CPU"*, mas a Onda 2 tornou máscara
  **representável**. Movido para um **ajuste não-portado** (`ColorBalance`) — o `ship.sh`
  não roda gate GPU-adapter, então ele ficou verde-latente por exatamente uma onda.

## 12. Onda 5b — o compositor GPU re-envia só a região suja da camada (2026-07-24)

Smoke do Enio: o brush (Onda 5a) segurou, mas a MESMA queda de FPS persistiu em três
cenas de máscara — **pintar a máscara**, **pintar com máscara**, e **pintar após limpar
a máscara**. As três tornam a pilha **NÃO-trivial**, então saem da pista trivial da CPU
que a Onda 5a consertou e vão para o **produtor GPU**.

### 12.1 A medição (wgpu real, traço com máscara)

| canvas | antes | depois | ganho |
|---|---|---|---|
| 2048² | 1,001 ms/move | **0,213** | ~4,7× |
| 4096² | **5,789 ms/move** | **0,470** | **~12×** |
| razão 4096/2048 | 5,8× (plano) | 2,2× | — |

Causa: o cache de camadas do compositor re-enviava a camada **inteira** por movimento
(`ensure_slice` → `upload_slice` = `write_texture` cheio, **64 MiB @ 4096² para um dab**)
— a cópia de staging na CPU por trás do *"parece dependente de CPU"*. A Onda 5a só tocou
a pista trivial da CPU; todo traço não-trivial a deixa.

### 12.2 A cura

`LayerPixels` ganhou **`dirty: Option<Region>`**. Numa fatia **RESIDENTE** cuja versão
mudou, `ensure_slice` re-envia **só** aquela sub-região (`upload_slice_region`: `origin`
+ `bytes_per_row` de largura cheia, **sem gather na CPU**); fatia nova/não-residente ou
`None` → envio cheio (o seed honesto). O tool fornece a região suja da camada **ativa**:
`take_preview_dirty` guarda o `dirty_rect` acumulado (unido por `mark_dirty`, então cobre
toda mudança desde o último upload mesmo sob frames perdidos) em `preview_dirty_region`,
e `preview_layer_pixels` o devolve como tupla (a ponte constrói o `LayerPixels.dirty`,
mantendo o tool desacoplado do `ph2d-render`). Camada não-ativa → `None` → cheio
(undo/fonte nova).

O **composite continua cheio** (GPU compute, rápido — o resíduo de 0,47 ms / razão 2,2×);
a imagem não muda. O composite parcial fica **nomeado** como a próxima alavanca se algum
dia 0,47 ms/move incomodar.

### 12.3 Gate

`gpu_partial_layer_upload_patches_only_the_dirty_region` — seed uma fatia, depois um
buffer que difere em TODO lugar com só uma sub-região declarada suja; o composite tem de
mostrar a cor NOVA DENTRO e a cor do SEED FORA (mutação: upload cheio → fora vira nova,
RED). Os gates e2e de handoff já dirigem o provider REAL com regiões sujas e seguem
byte-exatos (3/3, paridade 37/37).

**Foundational:** `ph2d-render::LayerPixels` +1 campo (`dirty`), 5 sítios de construção
atualizados (flip + 3 testes = `None`). Sem schema, sem contrato congelado.

## 13. Onda 5c — o traço de MÁSCARA para de fazer recompose cheio por quadro (2026-07-24)

A Onda 5b não resolveu a queda de FPS que o Enio reportava nos três cenários de máscara
(pintar a máscara · pintar com máscara · pintar após limpar). A razão: o caminho DELE
nem chegava ao produtor GPU da 5b — ficava 100 % na CPU.

### 13.1 O diagnóstico nomeou o braço exato

`PH2D_PAINT_PERF` foi partido em sub-fases (`preview`/`panel`/`overlay`/`upload`) e depois
ganhou o **braço do drain** (`take_preview_arc`) + os dois predicados que decidem o braço.
A linha do quadro caro, num traço de máscara:

```
frame p50=25.5 dispatch p50=24.2 [preview 17.3 panel 0.0 overlay 0.0 upload 6.8]
WORST: CPU 2048x2048 branch=FULL-composite impasto=false mask_scratch=true ... trivial=true
```

Ou seja: `panel`/`overlay` = 0 (as hipóteses anteriores morreram), o custo é **`preview`
≈ 17 ms + `upload` ≈ 6,9 ms**, na CPU, no braço **`FULL-composite`**, com **`mask_scratch=true`**.
A flag `trivial=true` enganava — ela só reporta `is_trivial_stack()`, e o caminho rápido de
verdade exige TAMBÉM `!mask_scratch_active() && !impasto_visible()`.

### 13.2 A causa

`take_preview_arc` fazia `force_full = mask_scratch_active()` (runtime.rs), mandando **todo
quadro pintado com um scratch de máscara vivo** pro braço de recompose de tela inteira +
upload cheio de 16 MiB — para uma mudança do tamanho de um dab. O comentário justificava:
*"um blit parcial não consegue re-tingir a área não-tocada"*. Mas:

- `apply_mask_overlay` é **PER-PIXEL** — o tint de um texel depende só da própria cobertura
  e da cor do filme, **sem termo global**;
- um dab de máscara muda a cobertura só dentro do próprio `dirty_rect` (o stamp edita o
  scratch trocado dentro do `canvas_rgba`, `stamp_dabs_mask`);
- as mudanças de tint genuinamente globais (swatch de cor, canvas-op Expand/Contract/…,
  o primeiro scratch) **todas** já chamam `invalidate_composite()` → o braço cheio.

Logo o `force_full` era pura super-cautela.

### 13.3 A cura

Removido o `force_full`. O braço parcial re-tinge **só a região do dab** via o novo
`apply_mask_overlay_region` — byte-idêntico ao re-tint cheio ali, porque ele **compartilha
o kernel per-pixel `tint_pixel`** com o `apply_mask_overlay` (duas cópias divergiriam num
texel, que é exatamente a classe de bug que o braço parcial não pode introduzir). O seed
(primeiro dab do scratch) segue cheio pelo `invalidate_composite` de `ensure_mask_scratch`;
os dabs seguintes tomam a via parcial ⇒ `preview_upload_bbox = Some` ⇒ a via de upload
parcial (5b) também dispara.

Cobre os TRÊS cenários: Mask mode (`stamp_dabs_mask`), pintura-com-proteção
(`restore_protected_region`, `paint_mode != Mask`), e pós-Clear (scratch branco vivo).

### 13.4 Gate

`a_mask_stroke_takes_the_partial_lane_byte_identical_to_a_full_recompose` (mutação-provado):
seed com o 1º dab (full), 2º dab noutro ponto tem de tomar `PartialComposite`, e o resultado
tem de bater byte-a-byte com um recompose cheio da MESMA cena. Mutações que sangram:
(a) re-por `force_full` → o 2º dab vai pro braço cheio (assert de branch) · (b) tirar o
`apply_mask_overlay_region` → a região do dab perde o tint, `partial != full` (assert de byte).

**Sem schema, sem contrato congelado.** Só a `ph2d-tool-painter` (mask.rs + runtime.rs).

### 13.5 O smoke do Enio: o composite parcial é limpo, o UPLOAD parcial não (no device)

O smoke aprovou o FPS mas reprovou a QUALIDADE: a máscara saiu com **bordas em bloco +
retângulos translúcidos** (as regiões do upload parcial). Bissecção com o interruptor
`PH2D_PAINT_FULL_UPLOAD=1` (força `UploadPlan::Full` sem tocar o composite):

- **com upload cheio** → máscara **lisa** e ainda **60 fps** (dispatch ~8 ms: composite
  parcial ~2 ms + upload cheio ~6 ms, sob os 16,7 ms @ 2048²);
- **com upload parcial** → os retângulos.

⚠️ **O cache da CPU é byte-idêntico ao recompose cheio** — provado por gate E por uma
sonda quadro-a-quadro que simula o upload parcial (0/65536 bytes diferem, pincel macio,
arrasto). Logo o defeito é **só no device** (o `write_texture` de sub-região do overlay
translúcido), mecanismo ainda não isolado. O `regen_mips` roda dos dois lados a partir do
nível 0 (correto), então não é mip; o `replace_individual_pixels_region` é o MESMO da
pintura normal (que é limpa) — o overlay translúcido é que torna a costura visível.

**Decisão (commit `2da916c99`):** a máscara mantém o **composite PARCIAL** (o ganho de
17 ms → ~2 ms) mas **força upload CHEIO** (`preview_upload_bbox = None` com scratch vivo).
Byte-idêntico à referência, 60 fps @ 2048². Gate estendido: o drain de máscara reporta
`bbox=None` (mutação: `Some(bbox)` incondicional → RED).

**Aberto (o "melhores que Procreate"):** (a) isolar a costura do upload parcial no device
para o caminho rápido ser limpo TAMBÉM — importa a 4096², onde o upload cheio estoura o
orçamento; (b) a qualidade da borda da máscara em si (a versão lisa é a referência antiga
e o Enio quer superá-la) — **FECHADO na §13.6**.

## 13.6 A borda da máscara endurecia sob MUITAS passadas — o build-up era um PRODUTO

O Enio mostrou (zoom) que a máscara fica **serrilhada/mosqueada** após MUITAS passadas no
mesmo lugar (uma passada = lisa). Eu não via porque dava poucas passadas.

**Diagnóstico (3 agentes em paralelo, prova aritmética):** a cobertura da máscara acumulava
como um **PRODUTO** entre passadas — `valor = 255·m^N` após N passadas — batendo texel a
texel com a medição (`199→6`, `247→156`). Isso empurra a borda de 50% para a **cauda
estreita do falloff**, então a banda do feather encolhe ~1/N e serrilha numa curva.
⚠️ **NÃO é** o caminho parcial, o upload, a rasterização (centros f32, falloff suave, sem
snapping) nem o 8-bit. É o **build-up per-dab compartilhado**, então a pintura normal
endurece **byte-idêntica** (medido); a máscara só REVELA porque o overlay é translúcido
(tinta opaca esconde a borda dura). A **UMA passada é uma INTEGRAL DE LINHA** suave e está
CORRETA — dabs mais densos CONVERGEM nela, não a afiam; o colapso é toda a re-multiplicação
ENTRE traços. É a mesma "doença do produto sobre a lista de dabs" que a linha curou 3×
(smear/bow-wave/cápsula) — a cura é sempre um **ENVELOPE (max)**, não um produto.

**Fix (escolha de produto do Enio: Envelope):** cada traço compõe exatamente como hoje (⇒
UMA passada byte-idêntica, o fingerprint da pintura intacto), e os **traços** se combinam
por **envelope** em vez de produto. Um traço Paint/Erase carimba num buffer POR-TRAÇO a
partir do neutro (255 Paint / 0 Erase — o produto within-stroke de sempre), e cada batch o
funde no scratch committed por `min` (Paint, mais proteção) / `max` (Erase, mais des-
proteção). A fusão é **idempotente entre traços** (o produto within-stroke é monotônico),
então N passadas idênticas dão o MESMO feather suave de uma — a borda **nunca endurece**
(render confirmou: 15 passadas == 1, liso). **Escopo: só a rota da máscara**
(`stamp_dabs_mask` + o buffer por-traço); o caminho per-dab compartilhado e o build-up
cross-stroke da pintura normal ("passar por cima pra aprofundar") ficam intactos — os dois
meios querem comportamento cross-stroke OPOSTO. Blur/Smear são ops espaciais no committed
(não re-multiplicam) e seguem pintando-o direto.

**Trade documentado:** passadas rápidas idênticas agora CONVERGEM na profundidade de uma —
aprofunde com pincel mais forte/lento ou traços sobrepostos.

**Gate (red-first, mutação-provado):** `the_mask_feather_does_not_harden_across_passes` —
15 passadas idênticas têm de deixar a cobertura byte-idêntica a 1 (envelope idempotente);
RED sob o produto antigo (37764 bytes diferem, delta máx 197); mutação `envelope=false`
re-sangra. Sem schema, sem contrato congelado; `paint.rs` mantido no teto de 700 LOC.

## 13.8 ⚠️ §13.6 (envelope) e §13.7 (teto por época) foram REVERTIDAS — a máscara vai ser REESCRITA

O smoke do Enio (2026-07-25) reprovou as duas: o **envelope** do §13.6 matou o hardening no mesmo
ponto mas o `min` deixava **linhas brancas nos cruzamentos** (union em vez de soma); o **teto** do
§13.7 (que era para o axis DIFERENTE de pintar-cor-através-da-proteção) **vazou no brush normal**
(a proteção persiste ao trocar de ferramenta ⇒ o teto capava a tinta comum). Ambas revertidas
(`1d390d926`, `569149dfc`/`7e26fa833`); a máscara está no **depósito original** (produto por-dab)
+ o fix de FPS (§13.5), com o brush normal **byte-idêntico** ao aprovado.

**Decisão do Enio:** reescrever a cobertura da máscara do zero, com referência de alta qualidade
(pesquisa). O serrilhado sob muitas passadas é a doença "product-over-dabs" (o `255·mⁿ` afia a
cauda do falloff); a cura precisa somar-como-tinta E ser idempotente no mesmo ponto — o candidato é
o **Wash/opacity mode** (cap por-traço + aditivo entre traços), provavelmente do Krita. Plano e
armadilhas completos em **[`../HANDOFF_line_Painter_mask_rewrite_2026-07-25.md`](../HANDOFF_line_Painter_mask_rewrite_2026-07-25.md)**.
Os §13.6/§13.7 ficam como HISTÓRICO do que já foi tentado e reprovado — não reconstrua.

## 13.9 ⚠️ REVERTIDA no mesmo dia — leia a §13.10 ANTES desta seção

> A lei do canal descrita abaixo (o envelope Wash do Krita) foi construída, medida, **renderizada e
> REPROVADA pelo Enio** horas depois: *"péssimo resultado. A máscara deve pintar exatamente como o brush
> digital normal"*. O que segue vale como **registro do que foi tentado** — a pesquisa, os números e o
> raciocínio seguem corretos e úteis; a CONCLUSÃO (adotar o envelope) não. O estado que ficou, o motivo
> e a foto estão na **§13.10**.

## 13.9 A cobertura da máscara reescrita — a lei do canal é o Wash do Krita, não o build-up do pigmento (2026-07-25, REVERTIDA)

**Ordem do Enio:** *"ainda temos o problema dos artefatos após múltiplas pinceladas. Creio que o melhor
será reescrever do zero, baseado em código de alta qualidade como referência."*

### 13.9.1 A pesquisa (as duas referências discordam, e a discordância é a resposta)

- **GIMP, paint core** (`gimppaintcore.c` + `gimp-gegl-loops.cc`): no modo `GIMP_PAINT_CONSTANT` (o
  default, "Incremental" desmarcado) a cobertura do dab acumula num **buffer por-traço**
  (`core->canvas_buffer`) e o resultado é aplicado **a partir do snapshot de undo**
  (`gimp_applicator_set_src_buffer(applicator, undo_buffer)`), com `paint_opacity` como **teto**. A
  aritmética, verbatim, é `if (opacity > dest) dest += (opacity - dest) * mask * opacity` — o perfil do
  dab é uma **TAXA** rumo ao teto.
- **Krita, modo Wash** (internamente o *Alpha Darken*): a opacidade é do **TRAÇO**, não do dab, e a
  regra *"não deixa o alfa DIMINUIR"* — um `max`. A própria doc diz para que serve: *"ensures the line
  doesn't get darker when you cross it again and again"*, sem *"the circular pattern you can see in
  Build-Up"*.

⚠️ **A arquitetura das duas é a MESMA** (buffer por-traço + aplicar sobre o estado congelado) e a **lei
é diferente**: taxa (GIMP) × alvo (Krita). Com opacidade 100% o teto do GIMP é vácuo, então **todo app
endurece a borda nesse regime** — é o Wash que existe justamente para não endurecer. Um canal de
cobertura quer o Wash; pigmento quer o build-up (*"passe por cima para aprofundar"*).

### 13.9.2 O que JÁ existia aqui — e por que a máscara não o usava

O motor já tinha o buffer por-traço (`PaintState.stroke_mask`, threaded pelas rotas per-pixel/ramped
quando **Accumulate OFF**), e a lei dele era **exatamente a do GIMP** (`m += w·(cap − m)`). Mas o
predicado que o armava era `!accumulate && (strength < 1 || film_aa)` — e o pincel de máscara tem
`strength = 1`, onde o teto é vácuo ⇒ **a máscara caía no produto per-dab puro**.

### 13.9.3 A medição do defeito (sonda `mask_probe.rs`, render-and-look)

Pincel default da máscara (r = 10, hardness 0, Smooth, spacing 10%, strength/flow 1):

| | 1 passada | 15 | 20 | 45 |
|---|---|---|---|---|
| band 0.9→0.1 (reta) | 3.53 px | 1.38 | 1.43 | — |
| band (arco r = 70) | 3.95 px | 1.65 | — | — |
| sawtooth do contorno (arco) | 0.035 px | 0.106 | — | — |

E o pior caso não precisava de 15 pincelada nenhuma: **ESFREGAR sem soltar a caneta** (4 pernas num
pen-down) já levava o corpo 118 níveis mais escuro e colapsava a band **3.53 → 1.88 px dentro do mesmo
gesto**. Renderizado, o arco de 15 passadas é uma borda dura serrilhada; o de 1 passada é macio.

⚠️ **Uma hipótese MORREU na medição:** a lei já era função do CAMINHO no que depende do polling — o
mesmo traço entregue em 5 ou 100 eventos de ponteiro dá `max |Δcov| = 0.0000`. O que a envenenava era a
CONTAGEM de dabs sobre o texel (o spacing), não o batching.

### 13.9.4 A cura

**`ph2d-painter-brush::stroke_cover`** — `StrokeCoverLaw { BuildUp, Envelope }` + `StrokeCover { buf,
law }`, e `cover_add` com a aritmética das duas leis num lugar só (o `BuildUp` é *pure code motion* das
duas ramas que estavam inline no `bands.rs`, com gate comparando as duas expressões termo a termo).

**Uma porta no tool:** `PainterTool::stroke_cover_law(brush)` responde *"este traço rastreia cobertura, e
por qual lei?"* — e é a MESMA porta que (a) escolhe a rota (um traço rastreado não pode usar os caches,
que não têm buffer para threadar) e (b) threada o buffer nas duas rotas que o aceitam. Antes o predicado
estava escrito em **três** lugares.

**O `a = add/(1 − m)` que já estava ali é a outra metade do GIMP, de graça:** ele telescopa exato
(`Π(1−a_k) = 1−m_n`), então o canvas sempre vale `pre_traço·(1−m) + cor·m` — o depósito pousa no estado
em que o traço começou, **sem guardar cópia do canvas**. É por isso que traços consecutivos SOMAM
(`c' = c + m(1−c)`) e o vale entre dois vizinhos enche — o oposto da união do §13.6.

### 13.9.5 O resultado (mesma sonda)

| | antes | depois |
|---|---|---|
| band, 1 passada (reta) | 3.53 px | **6.21 px** (feather analítico do pincel: 5.40) |
| band, 15 passadas | 1.38 px | **2.10 px** |
| ESFREGAR num pen-down (corpo) | 118 níveis · band 3.53→1.88 | **2 níveis · band 6.21→6.18** |
| arco, 15 passadas | 1.65 px, borda dura | **2.36 px, rampa macia** |
| custo por move (pincel r = 60) | — | **0,9 ms médio / 2,5 ms pior, IGUAL @2048² e @4096²** |

### 13.9.6 Os trades, MEDIDOS e nomeados (é o que o smoke vai julgar)

1. **Uma passada é mais MACIA** — ela agora deposita o feather do próprio pincel em vez de um
   pré-endurecido. Quem quiser borda dura tem Hardness/falloff, que é onde isso mora.
2. **Uma passada deixa o miolo em 0.984, não 1.000** (o centro de um dab nunca cai exatamente no centro
   de um texel). A segunda passada dá 1.000 exato.
3. **Rabisco de lanes a 1 raio de distância ondula na PRIMEIRA varredura** (interior 0.75 contra 0.98) —
   é o perfil macio somando honestamente. A 2ª varredura fecha (0.94) e a 3ª some (0.98); a 0.6 raio já
   nasce fechado (0.95 → 0.996). Sob a lei antiga o interior nascia chapado **porque tudo endurecia**.
4. ⚠️ **A rampa AINDA aperta com muitas passadas, como `N^(−1/2)`** — 6.21 / 2.10 / 1.94 / 1.55 px a
   1 / 15 / 30 / 45. **Não existe lei que dê build-up entre traços E borda invariante**: a única que
   congela a borda é a união entre traços, que não enche o vale (o §13.6, reprovado). O que o envelope
   compra é ~1,5× a rampa em QUALQUER contagem de passadas, e o esfregar sair de graça. Isto está escrito
   no gate, no smoke e aqui de propósito — não é invariância, e prometer invariância seria mentira.

### 13.9.7 Dois achados da auditoria (as duas lentes)

- **Uma rota escapava da porta:** o caminho de **Per-Layer Color** é decidido ANTES das ramas de
  ramp/cache e nunca consultava o predicado ⇒ com ele armado, o traço de máscara voltava ao build-up
  (**medido: esfregar movia 16 níveis com a rota aberta, 0 com ela fechada**) e ainda tirava as cores da
  PILHA de camadas, num buffer que `stamp_dabs_mask` força a preto/branco justamente para ser só
  cobertura. A máscara não toma essa rota (o roteamento do pigmento fica intocado). ⚠️ **Dois oráculos
  foram tentados e reprovados antes:** o *feather* não responde (armar camadas instala a silhueta de
  Shape, que tem borda dura por desenho) e a *cor* não responde (a rota resolve para cinza ali, então o
  gate passava COM o bug). Quem responde é o ESFREGAR — a propriedade de que a lei trata.
- **Um checkbox virou morto:** `Accumulate` não é lido em modo máscara (a lei vem do MODO), então a row
  se esconde ali — 3ª vez que essa row precisa se esconder, e cada vez por um motivo diferente
  (aquarela = provadamente inerte · impasto = governa metade da tinta · **máscara = o campo nunca é
  lido**). O comentário do painel afirmava *"em Eraser/Mask/Inpaint o Accumulate volta a significar
  algo"*: ficou FALSO para a Mask no mesmo commit, e foi corrigido junto.

### 13.9.8 Aberto, NOMEADO, e fora desta wave

**Os métodos de SHAPE (Line/Curve/Ellipse/Polygon/Free Hand) em modo máscara não pintam nada** — o
roteador de shape intercepta o Down antes do `paint_begin`, então `ensure_mask_scratch` nunca roda e o
scratch fica com **0 bytes** (medido, sonda 4). É pré-existente e ortogonal à lei; consertar direito
exige também uma base congelada por traço (senão o re-stamp por frame re-multiplica o scratch, e o
resultado passa a depender da taxa de quadros). Wave própria.

### 13.9.9 Gates (10) e mutações (6, todas sangram)

`mask_tests.rs`: o esfregar é inerte · uma passada deposita o feather analítico · a rampa sobrevive a 15
passadas · vizinhos SOMAM no vale · só a máscara pede a lei do canal · custo não segue o canvas (razão) ·
duas passadas protegem 100% · undo é 1 passo e o traço seguinte deposita normal · Per-Layer Color não
sequestra o traço. `stroke_cover_tests.rs`: o `BuildUp` é a aritmética que shipou (termo a termo) · o
envelope é um `max` (ordem-invariante, repetição inerte) · o ombro sobrevive onde o do build-up colapsa
(com controle: o build-up CRESCE com sampling mais denso) · Strength/Grain seguem valendo.
`seam.rs`: a row Accumulate se esconde na máscara (presença E ausência).

**Mutações:** lei da máscara → `BuildUp` (4 gates RED) · buffer não reseta por traço, i.e. união global
(2 RED, reproduzindo o vale claro do §13.6: 0.573 contra 0.984) · envelope reinicia por BATCH (3 RED) ·
`add = target` sem subtrair o que já está lá (2+1 RED) · rota Per-Layer Color reaberta (1 RED, 16
níveis) · `!is_mask` fora da condição da row (1 RED).

**Sem schema, sem contrato congelado.** `ph2d-painter-brush` não é superfície congelada (os ABIs de
pintura foram revogados pelo ADR-0099); `Tool`/`CanvasPaintTool`/`PanelEvent` intactos. Smoke:
**`PH2D_MASK_SMOKE=1`**.


## 13.10 A lei do canal foi REPROVADA na tela — a máscara pinta exactamente como o brush digital (2026-07-25)

**Ordem do Enio, depois do smoke do §13.9:** *"péssimo resultado. A máscara deve pintar exatamente como o
brush digital normal."* Com a foto: o traço de máscara saía em **CONTAS** — uma fileira de discos ao longo
da pincelada, mais as emendas claras nos cruzamentos.

### 13.10.1 O mecanismo (medido e renderizado, não inferido)

A mesma sonda reproduziu as contas headless em um traço só, com o pincel grande, e o A/B entre as duas
leis nomeou a causa:

> **O produto per-dab SATURA a estrutura por-dab; o envelope a deixa à vista.**

Sob o produto, o interior do traço vai a ~1,0 em qualquer texel que 2+ dabs cruzem, então a modulação
*"onde exatamente estão os centros dos dabs"* desaparece. Sob o `max`, cada texel guarda o pico do perfil
do dab mais próximo — e o perfil tem pico no CENTRO do dab, então a cobertura ondula com o período do
espaçamento. A ondulação é pequena em amplitude (**pico-a-pico 5 níveis de 255**, contra 3 sob o produto) e
**grande na percepção**, porque é periódica sobre um campo quase-sólido — 2% de contraste repetido é
visível, e é isso que a tela mostrou.

⚠️ **A medição que me enganou, e a lição:** eu medi a modulação **no EIXO** do traço, achei 6 níveis e
escrevi "invisível" (§13.9.6, item 3, e no comentário que dizia que a esparramada de 0,05 px era
desprezível). O eixo satura em qualquer lei — as contas vivem no **ombro e no interior fracionário**. Um
número no lugar errado disse o contrário do que a foto dizia
([[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]] é a irmã disto: aqui não
foi a mutação, foi a COLUNA).

### 13.10.2 O que ficou

**A máscara não tem lei própria.** O `StrokeCoverLaw`/`StrokeCover` foram removidos; `stroke_cover.rs`
guarda apenas a aritmética do cap de Accumulate (o que sempre shipou), numa cópia só, com o gate que a
compara termo a termo com as expressões que estavam inline. A porta do tool virou
**`stroke_cover_wanted(brush) -> bool`** — ela **não olha o MODO**, e é isso que "pinta como o brush
digital" significa em código. As três cópias do predicado seguem colapsadas em uma (o único ganho de
higiene que sobreviveu à reversão, e ele é real).

Saíram com a lei: o guard que desviava a máscara da rota de Per-Layer Color (o desvio a faria pintar
DIFERENTE do brush, que é o que a ordem proíbe — o achado fica nomeado no §13.9.7 como pré-existente) e o
esconde-esconde da row **Accumulate** (o campo é lido outra vez em modo máscara, então esconder passaria a
ser o controle FALTANDO).

### 13.10.3 O gate que pina a ordem

`the_mask_lays_exactly_what_the_digital_brush_lays`: o MESMO traço, pintado uma vez em modo Mask (lendo o
scratch) e uma vez em modo Paint com tinta preta sobre branco (lendo o canvas), tem de dar campos
**byte-idênticos**. Mutação medida: forçar o buffer por-traço na máscara + a lei `max` faz **3020 texels
divergirem, pior delta 120 de 255**.

⚠️ **Não existe gate numérico das CONTAS**, e a razão está medida acima: 3 níveis contra 5 é a mesma ordem,
porque o que o olho vê é a ondulação periódica e não a amplitude. Um bar de pico-a-pico em 4 seria um gate
que não pode falhar pelo motivo que alega. O oráculo das contas é o RENDER
(`probe_mask_beading_along_the_axis`), e o gate que de fato as impede é o de byte-identidade: **o brush
digital não faz contas, então a máscara não faz**.

### 13.10.4 O defeito original segue ABERTO, e a cura não é a lei da cobertura

O endurecimento da borda sob muitas passadas é real e está medido num teste executável
(`the_documented_hardening_is_still_there_and_this_is_its_number`: **3,53 px de rampa numa passada, 1,38 em
quinze**). As **duas** leis possíveis para o acúmulo já foram tentadas e cada uma tem seu artefato:

| lei | mata o endurecimento? | artefato |
|---|---|---|
| produto per-dab (a que ship a, = o brush digital) | não | a rampa aperta com as passadas |
| envelope `max` por-traço (Wash do Krita) | sim | **contas** por-dab, reprovadas na tela |

Então a próxima hipótese tem de estar em outro lugar. As três que sobram, sem nenhuma medição ainda:
**(a)** o OVERLAY (ele pinta `(1−cov)·0,8` de um filme sólido — uma rampa de cobertura fica visualmente
mais dura do que é, e o realce podia ser desenhado de outro jeito); **(b)** os DEFAULTS do pincel de
máscara (falloff/hardness/spacing — o endurecimento é função do nº de dabs por texel, que o Spacing
governa); **(c)** aceitar o endurecimento como o produto (é o que o brush digital faz, e é o que a ordem
atual pede). Nenhuma delas mexe na lei do acúmulo — e é justamente isso que o §13.10 fixa.

## 13.11 A TINTA atravessando a proteção — a força da máscara depende da taxa de POLLING (2026-07-25, ABERTO)

**Reporte do Enio (2ª rodada, com duas fotos):** *"A máscara agora é desenhada corretamente, mas sofre
novamente com bordas craqueladas na pintura quando muitas pinceladas são dadas repetidamente. Existe algum
problema no algoritmo de mascaramento que gera baixa resolução nas áreas com alpha na máscara?"*

**Sim, existe — e não é resolução.** A máscara está lisa (é o controle desta medição: dente-de-serra do
contorno dela **0,040 px**). Quem craquela é a TINTA, e a causa é onde a proteção é aplicada.

### 13.11.1 O mecanismo, medido

`restore_protected_region` puxa os texels protegidos de volta **uma vez por BATCH** (por evento de
ponteiro), contra o snapshot daquele batch. Então o fator `keep` é composto `N` vezes, onde `N` é quantos
batches cruzaram o texel — e `N` é a taxa de polling do mouse, não uma propriedade do gesto:

| referência do pull-back | tinta onde `keep = 0.5`, 4 ev/traço | 60 ev/traço | serra do contorno da tinta |
|---|---|---|---|
| o snapshot do BATCH (o que ship a) | 0,886 | **0,992** | 0,061 → **0,164 px** |
| a base do TRAÇO (`stroke_undo.canvas_rgba`) | 0,667 | **0,141** | 0,077 → 0,039 px |

A máscara manda passar **metade** e o produto entrega 89 % ou 99 % — ou, com a outra referência, 67 % ou
14 %. O contorno da tinta anda **4 px** só por trocar o número de eventos. É a MESMA doença que esta linha
curou 4× no relevo (*"a lei é função do CAMINHO, nunca de quão fino o motor amostrou o caminho"*), agora no
gate de proteção.

**Por que isso lê como "baixa resolução":** no feather a tinta satura (99 %), então o que sobra visível é a
fronteira onde `keep ≈ 0` — e essa fronteira é recortada pelos **RETÂNGULOS** dos batches (cada um puxa de
volta só a sua região), o que produz degraus axis-aligned. O olho lê degrau retangular como "baixa
resolução da máscara".

### 13.11.2 A cura mínima foi implementada e REFUTADA (não repita)

Trocar a referência do pull-back pela base do TRAÇO (que já existe e tem dono: o `canvas_rgba` do snapshot
de undo que o `paint_begin` tira) **conserta o dente-de-serra** (0,164 → 0,039 px, abaixo do da própria
máscara) **e inverte a dependência do polling, piorando a magnitude** (0,886→0,992 vira 0,667→0,141). O
ponto fixo dela é `base·(1−keep)/(1−keep + keep·a)`, onde `a` é a opacidade POR BATCH — ou seja, ainda uma
propriedade do sampling. **Ambas as referências erram**; o problema é puxar de volta por batch, não qual
snapshot se usa.

### 13.11.3 A lei certa, e por que é uma WAVE e não um patch

O que a máscara significa é `final = lerp(base_do_traço, tinta_LIVRE, keep)`, aplicado **uma vez** — a
semântica de máscara de camada do Photoshop, e o que o §7 do handoff da máscara já chamava de
*composite-time*. Para calcular isso é preciso que a tinta acumule **sem interferência** (livre) e que o
que se MOSTRA seja o lerp — duas coisas diferentes onde hoje existe uma. As três arquiteturas possíveis,
com o preço de cada uma:

1. **Buffer livre por-traço** (swap do buffer no stamp, como o `stamp_dabs_mask` já faz com o scratch; o
   canvas segue sempre exibível, então nenhum consumidor muda). Custo: +1 buffer canvas-sized durante um
   traço protegido (16 MB @2048²) e +1 passe por região. **Exato.**
2. **Proteção no PREVIEW** (o canvas guarda tinta livre, o lerp entra na porta de publicação). Sem buffer
   novo, mas o COMPOSITE lê o canvas — ele veria tinta desprotegida —, e todo caminho de teardown teria de
   assar o lerp ou a tinta livre vaza para o commitado. **Enumeração de teardown = a classe de bug que este
   repo mais paga.**
3. **Cobertura por-traço da TINTA** (`m_free` num buffer de 1 B/px, que já existe como `stroke_mask`): o
   display é `lerp(base, cor, m_free·keep)`. Barato, mas só vale para traço de cor CONSTANTE — Randomize
   Color, ramps e texturas quebram a premissa. É o parente da tentativa do §13.7, que vazou no brush.

**Recomendado: (1).** ⚠️ E qualquer uma delas mexe no caminho de depósito, onde a lei desta linha é
*"brush normal byte-idêntico"* — então a wave começa pelo gate de fingerprint do pigmento, não pelo fix.

### 13.11.4 O que já está no repo para a próxima sessão

`probe_paint_through_the_protection` (em `mask_probe.rs`, `#[ignore]`) monta a cena exata do reporte —
proteção com orla macia + N traços de tinta cruzando — e imprime a tabela acima, incluindo o controle
(o contorno da máscara). Ela é o red-first da wave: hoje ela MEDE o defeito; depois do fix, a tinta em
`keep = 0.5` tem de dar **0,5 nas duas colunas**.

## 13.12 A cura: a proteção é aplicada UMA vez por texel — a sessão por-TRAÇO (2026-07-25)

**Ordem do Enio:** *"veja nos commits se já não foi tentado sem sucesso. Muita coisa já foi tentada sem
sucesso. se é uma abordagem nova então vá"*. Foi conferido antes de escrever uma linha, e a resposta é:
**a LEI já foi tentada; o ESCOPO dela é novo.**

### 13.12.1 O que já tinha sido tentado, e por que isto não é aquilo

A §13.7 (`38c1f725b`, revertida em `569149dfc`) construiu exatamente `canvas = ref·(1−keep) + free·keep`.
Ela foi revertida porque o tempo de vida era a **ÉPOCA** — *enquanto a declaração de proteção existir* —,
o que atravessa traços e atravessa troca de ferramenta: a proteção virava um **TETO cross-stroke** e
**vazou no brush normal** (o teto capava tinta comum depois de você trocar de tool; §13.8).

| | §13.7 (revertida) | esta wave |
|---|---|---|
| vida da sessão | a declaração de proteção | **UM traço** (nasce no 1º batch gateado, morre no fecho) |
| entre traços | teto (N passadas convergem) | **build-up, como o brush digital** — intocado |
| sítios de commit a enumerar | **22** (toda edição de keep-source + todo escritor estrangeiro de canvas) | **0** — nada escreve o canvas no meio de um gesto |
| planos no `ModelSnapshot` | sim (o undo tinha de carregá-los) | **não** — transiente, o undo só derruba a sessão |
| gêmeo do plano livre no preview | sim (`PreviewPatch::free_pixels`) | **não** — a porta única `restore_region` o repõe |
| o vazamento que a matou | possível por construção | **impossível por construção** (não há o que vazar) |

⚠️ **O que fez a diferença toda foi perguntar *"por quanto tempo?"*, não *"qual fórmula?"*.** A fórmula
estava certa desde 25/07 de manhã; o custo dela era o tempo de vida.

### 13.12.2 O desenho

`GateSession { base, free, dirty }` mora no `PainterTool`, **ao lado do `canvas_rgba` e dos três planos de
relevo** (é um plano canvas-shaped, não um ajuste de pintura — e `paint.rs` está no teto de 700 LOC).

- **`base`** = o canvas como o traço o encontrou. `Arc` clone ⇒ **refcount, não cópia** (o 1º `make_mut`
  do canvas o forka uma vez, e a sessão fica com o lado pristino).
- **`free`** = o que a pintura **IRRESTRITA** teria produzido. É trocado para dentro do `canvas_rgba`
  durante o stamp (o MESMO truque que o sub-brush da máscara usa para o scratch), então **toda rota** —
  cor, smear, blur, clone, composite — pinta exatamente o que pintaria sem gate nenhum: *o gate não tem
  voto sobre O QUE é pintado, só sobre o que APARECE*.
- **`dirty`** = a união do que já foi carimbado no `free`: exatamente onde ele difere do `base`.

Depois do stamp, a região do batch do canvas visível é **re-derivada do zero** como
`free·keep + base·(1−keep)`, com `keep = proteção × seleção` (as duas portas antigas, rodadas em sequência
contra um snapshot, compunham **exatamente** esse produto — conferido na álgebra, não assumido).

⚠️ **Um traço de UM batch é byte-idêntico à porta antiga**, e o argumento é uma linha: `base` *é* o canvas
que o código antigo snapshotava, `free` *é* o que ele carimbava, e o blend é a mesma expressão termo a
termo. É só do SEGUNDO batch que as duas divergem — que é o bug.

### 13.12.3 A porta única que evita a próxima enumeração

Um método de **re-stamp** (Drag Dot e todo shape editor) devolve o canvas ao pristino e re-carimba a cada
frame de preview. O plano livre tem de voltar com ele — senão toda posição pela qual o artista ARRASTOU
fica nele e a projeção segue mostrando um leque de fantasmas.

Isso mora **dentro do `restore_region`**, não nos cinco chamadores dele: uma regra escrita uma vez por
chamador é uma regra que o sexto chamador nasce sem. (É o argumento do `reset_stroke_height`, uma linha
acima do primeiro chamador, um plano ao lado.) E o reset abrange o `dirty` **da sessão**, não o `rect` do
chamador: os dois batem na prática, mas são computados por funções diferentes (`dab_bbox` vs
`dab_batch_region`), e *"na prática"* é como um resíduo de 1 texel sobrevive para ser reportado como
rastro fraco.

As mortes da sessão são as MESMAS quatro do sculpt, no mesmo lugar: `paint_begin` (defensivo),
`close_stroke`, `commit_drag_preview`, `restore_model` (undo).

### 13.12.4 O resultado, medido pela sonda que diagnosticou

`probe_paint_through_the_protection`, a MESMA cena do reporte, a duas taxas de polling:

| lei | tinta em `keep ≈ 0.5`, 4 ev | 60 ev | serra do contorno | contorno médio |
|---|---|---|---|---|
| pull-back por BATCH (o bug) | 0,886 | **0,992** | 0,061 → **0,164 px** | andava **4 px** |
| pull-back contra a base do traço (cura mínima, refutada em §13.11.2) | 0,667 | **0,141** | 0,077 → 0,039 px | — |
| **plano livre por-traço** (hoje) | **0,800** | **0,800** | **0,082 px** nas duas | **x = 73,36** nas duas |

O controle da própria sonda — a serra do contorno da MÁSCARA — é **0,040 px**, então os 0,082 px são a
ordem do traçado, não resíduo do gate.

### 13.12.5 O custo, medido e NÃO otimizado (nomeado de propósito)

|  | pen-down | por move |
|---|---|---|
| 2048² sem proteção | 3,02 ms | 0,46 ms |
| 2048² **com** proteção | **7,43 ms** | **1,20 ms** |
| 4096² sem proteção | 11,26 ms | 0,41 ms |
| 4096² **com** proteção | **24,53 ms** | **1,13 ms** |

O **move é plano na tela** (1,20 vs 1,13): a projeção é limitada pela PEGADA, e é isso que o gate de perf
afirma como RAZÃO. O **pen-down é canvas-proporcional e sempre será** — ele aloca e enche um plano livre
canvas-sized, uma vez por traço. ⚠️ A 4096² isso é um quadro perdido no início de um traço protegido — e
**já era um antes desta wave** (o snapshot de undo força o próprio fork do canvas), então agora são dois.
O recurso tem nome: **largura de banda de memória + o zero-fill de uma alocação**.

**Não otimizado aqui, e a receita fica escrita:** semeadura **lazy por TILE** (um bitmap de tiles semeados;
uma cópia base→free por tile tocado ⇒ custo proporcional à pegada) + **reuso da alocação** entre traços
(mata o page-fault; preço = um plano canvas-sized residente enquanto o Painter tem documento, que é uma
pergunta de HR-13 e portanto de ADR-0117). É wave de perf com gates próprios; a wave de CORREÇÃO não
carrega uma otimização que nenhum smoke pediu.

### 13.12.6 Gates (6 novos) e mutações (4, todas sangram)

| gate | o que pina |
|---|---|
| `the_gate_lets_through_exactly_the_keep_it_declares` | **A LEI**: a tinta que passa é `keep × a que teria caído`, medida contra um **run de controle** do gesto idêntico com a orla fora do caminho |
| `the_protection_is_a_fact_of_the_mask_not_of_the_polling_rate` | **O SINTOMA REPORTADO**: seis passadas a 4 e a 60 eventos pintam a MESMA figura, e o contorno não anda |
| `repeated_strokes_through_the_feather_build_up_instead_of_converging` | **o guard anti-§13.7**: passar de novo aprofunda, como o brush digital — sequência plana = a sessão vazou |
| `a_session_is_born_only_under_a_gate_and_dies_with_the_stroke` | presença E ausência, as duas **no meio do traço** (depois do pen-up a resposta é `None` de qualquer jeito) |
| `a_restamp_preview_leaves_no_ghost_in_the_free_plane` | a porta única do `restore_region` |
| `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` | razão 1024²/2048² + kill 8 ms |

Mutações: **M1** re-semear por batch (= a porta antiga) ⇒ erro 0,235 / diferença 0,467, mata os 2 primeiros
· **M2** tirar o `restore_gate_free` ⇒ **960 texels** lembram uma posição arrastada · **M3** a sessão
sobrevive ao traço ⇒ `[0,678, 0,678, 0,678]`, a sequência plana que É o teto do §13.7, mata 2 · **M4** a
projeção percorre o plano ⇒ a razão dispara (2,20 vs 4,26 ms).

⚠️ **As duas portas antigas MORRERAM** (`snapshot_region`, `restore_protected_region`,
`restore_deselected_region`) — código morto e comentário obsoleto MENTEM, e os 5 doc-links que as citavam
foram reapontados. Zero id, zero token, zero contrato congelado, **nenhum schema** (`PROJECT_SCHEMA` 29):
a sessão é transiente por construção, e o gate de undo prova isso.

**Aberto:** o custo do pen-down (§13.12.5) · o **endurecimento da borda da máscara** segue aberto e é
outro eixo (§13.10.4 — as duas leis de acúmulo já foram tentadas) · smear/blur/clone atravessando a
proteção agora leem o plano LIVRE em vez do display, o que é a semântica de máscara de camada e é
**mudança de comportamento**: arrastar por cima de uma zona protegida carrega a tinta que o gate esconde.
O desenho antigo lia o display, mas o que ele lia dependia da taxa de polling, então não era referência
estável. **O smoke decide.**

## 13.13 Os 15% que sobraram: o build-up entre traços ERODE a proteção (2026-07-25, MEDIDO)

**Enio, depois do fix:** *"sanou quase 85% do problema"*. Os 15% restantes têm causa única e exata, e ela
**não é o gate** — é a semântica que o gate agora reproduz fielmente.

### 13.13.1 Duas explicações fáceis, as duas REFUTADAS por medição

| hipótese | previsão | medido | veredito |
|---|---|---|---|
| a tinta LIVRE ondula e o gradiente raso do `keep` amplifica | ondulação de `free` na fronteira | **1,000 ± 0,000** | ✗ a tinta é perfeitamente lisa ali |
| o OMBRO da máscara tem contas por-dab (§13.10) e o contorno as herda | `Δkeep/∇keep` = **0,07 px** | pente de **1,68 px** | ✗ 24× pequeno demais |

### 13.13.2 A causa: `1 − (1−keep)^N`

Cada traço é escalado por `keep`, então depois de `N` traços o texel guarda `1 − (1−keep)^N`. Duas
consequências, e a segunda é a grave:

**(a) o PENTE.** O contorno de meia-tinta senta em `keep` diferente para `N = 2` (0,2929) e para `N = 3`
(0,2063); a distância entre os dois dividida pelo gradiente **é** o pente, e `N` varia com a linha (quantos
traços vizinhos a cobriram). Previsão puramente aritmética contra a medição:

| | previsto | medido |
|---|---|---|
| máscara fresca (grad 0,0529/px) | **1,64 px** | **1,68 px** |
| 15 passadas (grad 0,1784/px) | **0,49 px** | **0,60 px** |

**(b) A PROTEÇÃO ERODE.** Num texel de `keep = 0,522`: `N=1` → 0,522 · `N=2` → 0,773 · `N=3` → 0,890 ·
`N=4` → 0,949 · **`N=8` → 1,000**. Oito passadas e a máscara não protege mais nada. ⚠️ **E a queixa do Enio
é literalmente esse gesto** (*"quando muitas pinceladas são dadas repetidamente"*).

### 13.13.3 Isto é um FORK de produto, e as duas referências discordam

- **Sculpt mask do Blender** (o que o código declara, e o que ship a): a máscara escala a força de cada
  traço ⇒ repetir constrói ⇒ erode. É consistente com *"pinta como o brush digital"*.
- **Layer mask / alpha lock** (Photoshop, Krita — e o que a pesquisa do §13.7 já tinha achado, *"tudo cuja
  promessa é PROTEÇÃO é um TETO que nunca endurece"*): o `keep` é aplicado UMA vez sobre a tinta acumulada
  livremente ⇒ **pente exatamente zero** e a proteção nunca erode.

⚠️ **O gate `repeated_strokes_through_the_feather_build_up_instead_of_converging` PINA a erosão como
correta.** Ele foi escrito como guarda contra reintroduzir o vazamento do §13.7, e nessa função está certo —
mas se o teto for a resposta, ele pina a lei de produto errada ([[feedback_inherited_affordance_must_be_rederived]]).

**O que o teto custa:** um plano por-MÁSCARA (a época do §13.7), cujo revert foi por **VAZAMENTO de ciclo de
vida** (a época sobrevivia à troca de ferramenta e capava tinta comum), não por a semântica estar errada. O
padrão da sessão agora existe e o commit obrigatório em toda edição do scratch é a peça que faltava — mas é
**wave própria**, e é a 3ª rodada neste mesmo eixo, então **exige ordem do Enio**.

### 13.13.4 A DECISÃO do Enio: o TETO (2026-07-25, construído)

Ele escolheu a **layer-mask / alpha-lock**: o `keep` é aplicado UMA vez sobre a tinta acumulada
livremente, e a proteção nunca erode.

⚠️ **Isto É a época do §13.7** — a semântica nunca foi o que estava errado com ela. O que a matou foi o
**CICLO DE VIDA**: ela tinha de ser commitada à mão em **22 escritores estrangeiros de canvas**, e um
escritor que ninguém listou tinha os pixels projetados por cima no batch seguinte (*"o teto capava a tinta
comum"*, §13.8). **Aqui os 22 sítios colapsam em UMA pergunta** feita no topo de todo batch — *algo mudou
debaixo de mim?* — respondida por **três testemunhas**: a **camada**, a **geração do scratch** e o
`pixel_clock` (`witness`). Enumeração apodrece; testemunha não.

| | §13.7 (revertida) | esta wave |
|---|---|---|
| lei | `lerp(ref, free, keep)` | **a mesma** |
| vida | a declaração de proteção | **a mesma** |
| commit em escritor estrangeiro | 22 sítios à mão | **1 testemunha** (`pixel_clock`) |
| commit em edição do scratch | sítios à mão | **1 geração** |
| planos no `ModelSnapshot` | sim | **não** (a época morre no undo; ADR-0117) |
| gêmeo do plano livre no preview | `PreviewPatch::free_pixels` | **o patch por-batch**, na porta única `restore_region` |

**Resultado medido:**

| | build-up (antes) | TETO (agora) |
|---|---|---|
| erosão em `keep = 0,522` após 8 traços | **1,000** | **0,522** |
| pente, máscara fresca | 1,68 px | **0,05 px** |
| pente, 15 passadas | 0,60 px | **0,12 px** |
| rampa da TINTA vs rampa da MÁSCARA | 49,89 / 56,94 (razão 0,876) | **56,94 / 56,94 — idênticas** |
| serra do contorno | 0,101 px | **0,042 px** |

A rampa da tinta virar **exatamente** a rampa da máscara é a assinatura da lei: com a tinta livre saturada,
o display é função PURA do `keep`, então a fronteira da tinta É o contorno do `keep`. E o pente residual
(0,05–0,12 px) casa com a ondulação própria do campo `keep` medida na §13.13.1 (0,05 / 0,13 / 0,14) — não
sobra nada do gate. Confirmado por **render-and-look**: as bordas internas dentadas ficaram limpas.

### 13.13.5 O defeito que a MUTAÇÃO SOBREVIVENTE achou

⚠️ Uma mutação (restaurar o plano livre de `base` em vez do patch por-batch) **não sangrava** — e a razão
era um bug meu: o `mark_dirty` do próprio `restore_region` move o `pixel_clock`, ou seja **a época disparava
a própria testemunha de escrita estrangeira**. Consequência: todo frame de preview de um método de
**re-stamp** (Drag Dot, Line, todo shape editor) re-semeava a época — o teto revertia em silêncio para
por-gesto naquela família inteira, e cada frame pagava um clone de canvas. Curado re-testemunhando no fim do
`restore_region`; gate próprio (`the_ceiling_holds_for_a_restamp_method_too`, mutação sangra com a erosão
subindo até 0,608 contra um `keep` de 0,522).

**Um plano livre que é jogado fora e reconstruído todo frame não pode ser observado errado** — é por isso que
a mutação passava. [[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]], na
variante em que ela acusa o PRODUTO.

### 13.13.6 Gates (8) e mutações (5, todas sangram)

| gate | o que pina |
|---|---|
| `the_protection_never_erodes_no_matter_how_many_strokes_cross_it` | **A LEI**: 12 traços não passam do `keep` — e a tinta ainda BUILD-UP até ele (as duas metades: teto não é parede) |
| `the_boundary_of_repeated_strokes_is_the_keep_contour_not_a_comb` | o pente ≤ 0,6 px (era 1,68) |
| `the_ceiling_holds_for_a_restamp_method_too` | a família dos shape editors, onde o preview undo disparava a testemunha |
| `the_epoch_outlives_the_stroke_and_dies_with_the_protection` | 5 metades: ungated não aloca · sobrevive ao pen-up · edição do scratch (traço) · **Modifier** (a camada que só a geração testemunha) · **escrita estrangeira** (o Fill SOBREVIVE) · undo |
| os 4 da §13.12 | polling-independência, custo, fantasmas, byte-identidade com o brush |

⚠️ **Duas metades desse gate nasceram FALSAS e foram reescritas:** a da escrita estrangeira afirmava que o
NÚMERO da testemunha mudava — e ele muda nas nossas próprias escritas também, então o gate não podia falhar
pelo motivo que alegava (a mutação sobreviveu). O oráculo virou *o Fill SOBREVIVE* (mutação: verde 0 → 224,
o vazamento do §13.7 reproduzido). E a da edição do scratch passou com a geração congelada porque a
esfregada de máscara move o `pixel_clock` — **defesa em camadas**; a camada que só a geração testemunha é o
**Modifier** (`mask_canvas_op` nunca chama `mark_dirty`), e o gate novo dela sangra com 0,122 → 0,878.

⚠️ **E o gate central desta wave foi DELETADO pela minha própria edição** (uma substituição por âncoras
engoliu a região entre dois gates) — a suíte ficou **verde sem ele**. Pego contando os nomes dos testes, não
pelo verde. *Depois de uma edição em massa em arquivo de teste, conte os gates.*

**Sondas no repo:** `probe_what_is_left_after_the_gate` (a rampa da tinta rastreia a da máscara, razão
0,876/0,875/0,847) · `probe_the_comb_on_the_boundary` (as duas refutações) ·
`probe_the_comb_is_the_cross_stroke_buildup` (a previsão aritmética + a tabela de erosão).
