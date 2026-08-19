# ARQUIVO — 25_avaliacao_gpu.md (história, 649 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`25_avaliacao_gpu.md`](../../../Painter/25_avaliacao_gpu.md) em 2026-08-18, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-402,486-500,750-772,879-886,961-1051,1122-1141,1171-1253` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

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
