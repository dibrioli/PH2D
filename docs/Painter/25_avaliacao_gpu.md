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
