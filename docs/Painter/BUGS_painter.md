# Bugs do módulo Painter — registro + soluções

> Log vivo de bugs **não-triviais** do Painter (sintoma → causa-raiz → tentativas que falharam → solução).
> O objetivo não é listar todo fix (isso o git já faz), mas registrar os bugs cuja **causa enganava** —
> aqueles em que a aparência levou a vários rounds na pista errada. Cada entrada termina em **lições
> generalizáveis** pra não repetir o erro de diagnóstico.

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--offset-de-curva-as-quinas-não-ficavam-paralelas-nem-cruzavam) | Offset de curva — quinas (não-paralelas, depois não-cruzavam) | Stroke shape-editor (Curve/Circle/Polygon/Free Hand) | ✅ Resolvido | 2026-06-29 |
| [2](#bug-2--per-layer-color-fps-despenca--artefatos-retangulares-retângulo-virtual) | Per-Layer Color — FPS despenca + artefatos retangulares ("retângulo virtual") | Stamp path (CPU) + GPU preview slot | ✅ Resolvido | 2026-06-29 |
| [3](#bug-3--queda-de-fps-warp--shapes-booleanas--todo-arraste-interativo) | Queda de FPS (Warp · Shapes booleanas · todo arraste interativo) | Bridge preview + selection recompose + warp mesh | ✅ Resolvido em CPU (2 rodadas 2026-07-04: Transform smoke-OK; per-layer texture-color + overlay booleanas fechados na 2ª — pendente smoke) | 2026-07-04 |
| [4](#bug-4--simplify-curve-degenerava-o-schneider-fit-não-fecha-loops) | Simplify Curve degenerava (curva → 2 pontos idênticos / triângulo) | Selection curve simplify (`fit_curve` fechado) | ✅ Resolvido (DP fechado + Catmull-Rom corner-aware) | 2026-07-05 |
| [5](#bug-5--offset-de-curva-densa-amontoava-os-pontos-após-convert) | Offset amontoava os pontos de uma curva densa após Convert (perda de perfeição) | Stroke offset (movia os pontos de controle) | ✅ Resolvido (offset DRAWING-ONLY, modelo da Seleção) | 2026-07-05 |
| [6](#bug-6--simplify-quase-bom--offset-arredondava-as-quinas--refit--vértice-reconstruído) | Simplify "quase bom" + offset arredondava as quinas | Simplify/Merge (`curve_refit`) | ✅ Resolvido (refit Schneider corner-split + vértice por interseção de bordas) | 2026-07-05 |
| [7](#bug-7--aquarela-grave-queda-de-fps--build-profile--composite-2frame--loops-seriais-não-os-algoritmos) | Aquarela: "grave queda de FPS" — build profile + composite 2×/frame + loops seriais, NÃO os algoritmos | Watercolor render-path + dev profile + heartbeat do shell | ✅ Resolvido (60fps em release com todos os knobs; validado pelo Enio via frame profiler) | 2026-07-07 |
| [8](#bug-8--aquarela-borda-duraserrilhada-nas-junções--retângulo-no-preview--6-fixes-verdes-sem-efeito-o-harness-reproduzia-o-mecanismo-não-o-contexto) | Aquarela: borda dura/serrilhada nas junções (Charge<1 / Rewet) + "retângulo no preview" — 6 fixes verdes sem efeito | Watercolor mixer + render (blend do pigmento, estilo por dono) | ✅ Resolvido (smoke Enio 2026-07-09; clareamento preservado, fronteira orgânica) | 2026-07-09 |
| [9](#bug-9--preview-de-umidade-retângulo-na-união--o-pour-re-molhava-o-vizinho-dentro-do-bbox) | Preview de umidade: "retângulo maldito/gigante" na união de traços úmidos | Moisture pour (`pour_canvas_wet`) + overlay do shell | ✅ Resolvido (pour por-footprint-dona; blur do véu foi tentativa errada) | 2026-07-11 |
| [10](#bug-10--borda-dura-na-junção-ao-mudar-params-de-wash-e-cruzar-traço-úmido--params-por-dono-degrauavam) | Borda dura na junção ao mudar params de Wash (Body/Concentration/Edge/Opacity/RaggedEdge) e cruzar traço úmido | Watercolor render (params por-dono discretos) | ✅ Resolvido (campo suavizado `build_style_field`; grad 118→13) | 2026-07-11 |
| [11](#bug-11--per-layer-color-linhas-retangulares-intermitentes-aberto) | Per-Layer Color — "linhas nas bordas de retângulos" nas cores do brush, **intermitente** | Preview (produtor CPU↔GPU / overlay) — **NÃO** o composite CPU | 🔎 **ABERTO** (dormente; composite CPU provado limpo, espaço de busca reduzido, **armadilha armada**) | 2026-07-11 |
| [12](#bug-12--panicsigsegv-ao-apertar-rake-com-um-traço-per-layer-color-vivo) | **PANIC/SIGSEGV** ao apertar Shape **Rake** com um traço Per-Layer Color vivo | Roteamento de stamp (troca de rota **no meio do traço**) | ✅ Resolvido (guard de forma único; RED verificado nas 2 direções) | 2026-07-12 |
| [13](#bug-13--varredura-a-família-do-12-o-guard-que-pergunta-existe-em-vez-de-que-forma-tem) | **Varredura**: +1 PANIC (trocar de sprite com tinta molhada) + 4 vazamentos silenciosos entre sprites | Lifecycle de rebind de documento · compositor · aquarela | ✅ 3 fixes (RED verificado em cada) | 2026-07-12 |
| [15](#bug-15--impasto-os-chips-do-rig-de-luzes-pintam-e-não-clicam-aberto) | **Impasto**: os chips do rig de luzes **pintam e não clicam** (nem o checkbox) | Seam da UI (painel ↔ tool) — **não** a matemática | 🔎 **ABERTO** (fila: amanhã, gate do seam PRIMEIRO) | 2026-07-12 |
| [14](#bug-14--impasto-a-tinta-extravasava-o-relevo-o-suporte-batia-e-a-foto-estava-errada) | **Impasto**: "a tinta extravasa o relevo" — 3 rodadas; o gate ficava **verde** e a foto do Enio, errada | Depósito de pigmento (alpha do dab) × corpo × luz | ✅ Resolvido (o FILME + opacidade Beer-Lambert; névoa 52% → 13,5%) | 2026-07-12 |
| [17](#bug-17--a-tinta-atravessando-a-máscara-saía-craquelada-a-proteção-era-um-fato-sobre-o-mouse-e-depois-um-teto-que-erodia) | **A tinta atravessando a máscara saía CRAQUELADA** — a força da proteção era um fato sobre o MOUSE, e depois um teto que ERODIA | Gate de proteção/seleção (`stamp_dabs` → o pull-back) | ✅ Resolvido em 2 rodadas (smoke Enio 2026-07-25) | 2026-07-25 |
| [16](#bug-16--aquarela-borda-dura-pixelada-o-aa-alimentado-na-densidade-era-comido-pela-saturação-óptica) | **Aquarela**: "borda dura pixelada" em traço fino — o 1º fix foi verde nos gates e invisível no produto | Watercolor composite (hardening + óptica exponencial) | ✅ Resolvido (forma × sombreamento: fração como ALPHA linear na aparência; estendido a todo traço + Ragged Edge alto) | 2026-07-20 |

## Bug #17 — A tinta atravessando a máscara saía CRAQUELADA: a proteção era um fato sobre o MOUSE, e depois um teto que ERODIA

**Área:** o gate de proteção/seleção do stamp (`paint/stamp_route.rs::stamp_dabs` → `paint/mask.rs`),
não a máscara e não o pincel.
**Estado:** ✅ Resolvido em **duas rodadas** (smoke do Enio 2026-07-25: 1ª rodada *"sanou quase 85% do
problema"*, 2ª rodada OK). Gates em `mask_gate_tests.rs`; sondas em `mask_probe_gate.rs`.
**Detalhe técnico:** [`25_avaliacao_gpu.md`](25_avaliacao_gpu.md) §13.11 (diagnóstico) → §13.12 (o `keep`
vale uma vez) → §13.13 (o TETO).

### Sintoma (Enio, 2026-07-25, com duas fotos)

> *"A máscara agora é desenhada corretamente, mas sofre novamente com bordas craqueladas na pintura quando
> muitas pinceladas são dadas repetidamente. Existe algum problema no algoritmo de mascaramento que gera
> baixa resolução nas áreas com alpha na máscara?"*

Pintar COR atravessando uma proteção de orla macia deixava a fronteira **craquelada / em degraus
retangulares** — e só depois de MUITAS pinceladas repetidas. Uma passada parecia boa.

⚠️ **A pergunta do Enio nomeava o suspeito errado, e a medição inocentou a máscara na primeira linha:** a
serra do contorno DELA é **0,040 px** (o controle da sonda). Quem craquelava era a TINTA.

### Causa-raiz — DUAS, uma por rodada

**(1ª) O `keep` era composto uma vez por BATCH.** `restore_protected_region` carimbava o traço normalmente
e depois puxava os texels protegidos de volta contra o snapshot **daquele batch**. Logo o que sobrevivia a
`N` batches era `(1−keep)^N` — e **`N` é a taxa de polling do mouse**, não uma propriedade do gesto:

| | 4 eventos/traço | 60 eventos/traço |
|---|---|---|
| tinta que sobrevive em `keep = 0,5` | 0,886 | **0,992** |
| serra do contorno da TINTA | 0,061 px | **0,164 px** |
| posição do contorno | — | andava **4 px** |

E o que sobrava visível — a fronteira `keep ≈ 0` — era recortado pelos **RETÂNGULOS dos batches** (cada um
puxava de volta só a sua região), o que produz o degrau axis-aligned da foto. **É a mesma doença que esta
linha curou 4× no relevo** (*a lei é função do CAMINHO, nunca de quão fino o motor amostrou o caminho*),
agora no gate de proteção — a **5ª instância**.

**(2ª, os 15% que sobraram) Cada TRAÇO era escalado por `keep`.** Curada a 1ª causa, o Enio reportou 85%.
O resto tinha causa exata: com o `keep` aplicado por traço, `N` traços deixam passar `1 − (1−keep)^N`. Duas
consequências:

- **o PENTE:** o contorno de meia-tinta senta em `keep` **0,2929** para `N=2` e **0,2063** para `N=3`, e
  `N` varia com a linha (quantos traços vizinhos a cobriram) ⇒ `Δkeep / |∇keep|` **é** o pente. Previsão
  puramente aritmética **1,64 px** contra **1,68 medido** (e 0,49 vs 0,60 com a máscara esfregada).
- **A PROTEÇÃO ERODIA**, e ninguém tinha posto isso num número: em `keep = 0,522`, `N=1` deixava passar
  0,522 · `N=4` → **0,949** · `N=8` → **1,000**. Oito passadas e a máscara não protegia mais nada — sob
  **literalmente o gesto que o Enio reportou**.

### Tentativas que falharam (não repetir)

| tentativa | por que falhou |
|---|---|
| **§13.6** envelope cross-stroke na COBERTURA da máscara (`600a79606`) | matou o endurecimento, mas o `min` deixava **linhas brancas nos cruzamentos** (union em vez de soma). Revertida. |
| **§13.7** o TETO por ÉPOCA (`38c1f725b`) | a semântica estava CERTA; o **ciclo de vida** a matou — 22 escritores estrangeiros de canvas commitados à mão, e um que ninguém listou tinha os pixels projetados por cima. Revertida. |
| **§13.9** a lei do canal (Wash/Alpha-Darken do Krita) na cobertura da máscara | curava o endurecimento nos números e **REPROVADA na tela**: sem a saturação do produto o traço sai em **CONTAS**. Revertida. |
| **cura mínima:** puxar de volta contra a base do TRAÇO em vez do batch | conserta o dente-de-serra (0,164 → 0,039 px) e **inverte a dependência, piorando a magnitude** (0,886→0,992 vira 0,667→**0,141**). O ponto fixo dela ainda é função do sampling. **As duas referências erram**: puxar de volta por batch é a doença, não a referência escolhida. |
| **duas explicações "fáceis" para os 15%** | (a) *a tinta livre ondula e o gradiente raso amplifica* — a tinta livre na fronteira é **1,000 ± 0,000**, zero ondulação a amplificar; (b) *o ombro da máscara tem contas e o contorno as herda* — a ondulação do `keep` vale **0,07 px** contra um pente de 1,68, **24× pequeno demais**. As duas refutadas por medição antes de a verdadeira aparecer. |

### Solução

**O `keep` é aplicado UMA vez por texel, sobre a tinta acumulada LIVREMENTE, e a época dura o que a
PROTEÇÃO durar.** `GateSession { base, free, preview_patch, layer, scratch_gen, witness }` mora no
`PainterTool`, ao lado do `canvas_rgba` e dos 3 planos de relevo:

- **`base`** = o canvas como a proteção o encontrou (`Arc` clone ⇒ refcount, não cópia).
- **`free`** = o que a pintura **irrestrita** teria produzido, **trocado para dentro do `canvas_rgba`**
  durante o stamp (o mesmo truque do scratch da máscara) ⇒ **toda rota** — cor, smear, blur, clone,
  composite — pinta o que pintaria sem gate nenhum: *o gate não tem voto sobre O QUE é pintado, só sobre o
  que APARECE*.
- o que se vê é `free·keep + base·(1−keep)`, com `keep = proteção × seleção` — o produto que as duas
  portas antigas, rodadas em sequência contra um snapshot, compunham **exatamente** (conferido na álgebra).

⚠️ **É a época do §13.7, e a diferença é a MÁQUINA que a fecha:** os **22 sítios enumerados à mão** viraram
**UMA pergunta** no topo de todo batch — *algo mudou debaixo de mim?* — respondida por **três testemunhas**:
a **camada**, a **geração do scratch** (`mask_scratch_gen`, bumpada por todo escritor do scratch) e o
**`pixel_clock`** (que todo escritor de canvas move via `mark_dirty`). *Enumeração apodrece; testemunha não.*

⚠️ **O teto não é uma PAREDE:** `free` acumula livremente e o `keep` pousa no RESULTADO, então o brush ainda
**constrói até** o teto. Um cap aplicado ao BRUSH faria o 2º traço ser um no-op silencioso — a forma exata
de *"o brush parou de funcionar"*.

⚠️ **A reposição do plano livre num re-stamp mora DENTRO do `restore_region`**, não nos 5 chamadores dele
(regra escrita uma vez por chamador é regra que o 6º nasce sem), e restaura um **patch por-batch**, não o
`base` — a época atravessa traços, então zerar para `base` deletaria tudo o que os traços anteriores
deixaram.

**Resultado medido:**

| | antes (1ª rodada) | depois do §13.12 | depois do TETO (§13.13) |
|---|---|---|---|
| tinta em `keep≈0,5` a 4 vs 60 eventos | 0,886 vs 0,992 | **0,800 nas duas** | 0,800 nas duas |
| erosão em `keep=0,522` após 8 traços | 1,000 | 1,000 | **0,522** |
| pente da fronteira (máscara fresca) | 1,68 px | 1,68 px | **0,05 px** |
| rampa da TINTA ÷ rampa da MÁSCARA | 0,876 | 0,876 | **1,000 (idênticas)** |
| serra do contorno | 0,164 px | 0,082 px | **0,042 px** |

A rampa da tinta virar **exatamente** a da máscara é a assinatura da lei: com a tinta livre saturada, o
display é função **pura** do `keep`, então a fronteira da tinta **é** o contorno do `keep`. Confirmado por
**render-and-look** (as bordas internas dentadas ficaram limpas).

### Lições generalizáveis

1. **Uma lei que depende de em quantos pedaços o motor recebeu o gesto é um bug, sempre.** 5ª instância
   nesta linha (mordida do bow wave · cápsula do relevo · smear · aro · agora o gate). O sinal diagnóstico é
   barato: **rode a MESMA cena a duas taxas de polling e imprima as duas linhas lado a lado**. Iguais = a lei
   é do gesto; diferentes = é do mouse.
2. **Um revert com a SUA fórmula dentro pode diferir só no tempo de vida — e o que falhou pode ser a
   MÁQUINA que fecha o escopo, não a lei.** Ler o *diff* do revert confirma que a fórmula é a mesma e faz
   você concluir o oposto do que a evidência sustenta; leia o **motivo**.
   ([[feedback_a_reverted_attempt_may_differ_only_in_lifetime_read_the_revert_reason]])
3. **Refute as explicações fáceis com número antes de aceitar a difícil.** As duas primeiras hipóteses do
   resíduo eram plausíveis e **erradas por 24×**; a verdadeira ficou provada porque a previsão puramente
   aritmética (1,64 px) casou com a medição (1,68) em 2%. *Uma causa que prevê o número é um diagnóstico;
   uma que só o explica é uma história.*
4. **Um gate que vigia um NÚMERO mudar não pode falhar pelo motivo que alega.** A metade da testemunha
   afirmava *"o `witness` mudou"* — ele muda nas nossas próprias escritas também, e a mutação **sobreviveu**.
   O oráculo virou ***o Fill SOBREVIVE***, e aí a mutação sangra reproduzindo o vazamento original
   (verde 0 → 224).
5. **Defesa em camadas precisa de gate por camada.** A metade da "edição do scratch" passava com a geração
   congelada porque a esfregada de máscara **também** move o `pixel_clock`; a camada que só a geração
   testemunha é o **Modifier** (`mask_canvas_op` nunca chama `mark_dirty`) — gate próprio, sangra 0,122 →
   0,878. ([[feedback_layered_defenses_need_per_layer_gates]])
6. **Um plano que é jogado fora e reconstruído todo frame não pode ser observado errado.** Uma mutação
   sobrevivente (restaurar o plano livre da fonte errada) denunciou um bug MEU: o `mark_dirty` do próprio
   `restore_region` disparava a testemunha de escrita estrangeira, então **todo método de re-stamp** (Drag
   Dot, Line, todo shape editor) re-semeava a época por frame de preview — o teto revertia em silêncio para
   por-gesto naquela família inteira, e cada frame pagava um clone de canvas.
7. **Depois de uma edição em massa num arquivo de teste, CONTE os gates.** Uma substituição por âncoras
   engoliu a região entre dois testes e **deletou o gate central da wave**; a suíte ficou verde sem ele.
   Verde não prova que o gate que você escreveu ainda existe.

## Bug #16 — Aquarela: "borda dura pixelada" — o AA alimentado na DENSIDADE era comido pela saturação óptica

**Área:** watercolor composite (`watercolor_render.rs` / `watercolor_field.rs::aa_coverage`).
**Estado:** ✅ Resolvido (smoke Enio 2026-07-20: *"Traços finos perfeitos!"* → estendido a todo traço
por ordem; Ragged Edge alto fechado na mesma rodada). Gates em `watercolor_aa_tests.rs`.

### Sintoma (Enio, 2026-07-20, screenshots)

Traço **fino** de aquarela (e de impasto) com borda dura serrilhada ("pixelada"), enquanto o painter
simples tem borda perfeita em qualquer tamanho. Traço grosso parecia bom em todos os modos.

### Causa-raiz (em camadas — cada rodada descobriu uma)

1. **O hardening mora em espaço de COBERTURA** (`smoothstep(SS0=0.12, SS1=0.60, cov)`): a largura da
   transição em texels de tela é proporcional ao raio — num traço fino a silhueta cruza a janela em
   menos de um texel e vira degrau binário. O painter simples usa o falloff cru como alpha
   (rampa ≈ 0.4·raio texels, sempre ≥ 1 texel → AA sub-pixel de graça).
2. **A óptica a jusante é EXPONENCIAL** (fringe `edge_gain·(cw−inner)` + Beer–Lambert): satura o
   escuro logo acima de `cw≈0.2`. MEDIDO no raio 10: a cobertura tem rampa de ~2 texels e a borda
   ainda renderiza `255, 190, 1` — binária. **Alimentar a fração de AA na densidade não é AA**: o
   exponencial come a fração.
3. **Ragged Edge alto** (warp até 48): o warp desloca as posições de amostragem de pixels VIZINHOS em
   até `1 + amp·0.19` texels (medido) — a banda inteira do hardening é saltada entre dois pixels, e
   nenhum centro amostrado jamais vê gradiente. Warp 48 = 226 cliffs com todo o resto verde.

### Tentativas que falharam (não repetir)

- **v1 — supersampling da cobertura alimentado na densidade** (commit `4783ddf2`): gates verdes
  (fixture raio 4, métrica de cliffs), produto inalterado ("nenhuma melhoria"). Dois erros: o gate de
  steepness não disparava no raio do produto (~10), e a fração morria na saturação. Lição dupla:
  *teste com os números do PRODUTO* e *o oráculo tem de ser a aparência final*.
- **v2a — alpha com sombreamento da amostra central**: double-fade — o texel do aro renderiza o wash
  DILUÍDO e ainda leva alpha, enquanto o vizinho interno já saturou: o penhasco só muda de lugar
  (`255, 223, 28`).
- **lerp de BYTES no pixel armazenado**: quebrou `flatten(camada transparente sobre branco) ==
  pintar no branco` (o gate `watercolor_ground_*` pegou) — o L straight-alpha não é aparência; o AA
  tem de compor em luz linear ANTES do un-premultiply.
- **alpha = média crua dos subsamples**: com Dilution o corpo inteiro senta no meio da banda e o
  scallop do feather (platô 0.92→1.0) mantém `grad>0` em todo o interior → o corpo desbotava (~0.8)
  e a junção de donos degrauzava (o gate da cruz pegou: 42 vs limite 15).
- **probe dilatado de resgate para centros em região chata** (Ragged Edge): construído e MEDIDO
  MORTO — o mesmo scallop garante `grad>0` em praticamente todo o wash; removido (código morto com
  docstring confiante vira mentira).

### Solução (o split clássico de rasterizador: forma × sombreamento)

`aa_coverage` devolve `(cw, aa_alpha)` por texel de transição (`grad>0`; platô saturado/papel ficam
amostra única = byte-idênticos):

- **Sombreamento** = o wash da fração coberta (o **MAX** dos 9 subsamples ±0.667 texel) — pode
  saturar à vontade.
- **Forma** = fração de área **relativa ao nível de wash presente** (`média ÷ máx` — a razão leva o
  interior diluído a ~1 e deixa a borda verdadeira fracionária).
- O alpha compõe a **APARÊNCIA** (wash-sobre-ground vs base-sobre-ground) em luz linear, antes do
  un-premultiply; `cov_a` escala junto (a silhueta alfa da camada transparente desvanece igual).
- **Os subsamples atravessam o warp COMPLETO** (avaliado em sub-offsets de OUTPUT): warp 48 cliffs
  226 → 0. Offsetar em espaço já-warpado lê um footprint até `1+amp·0.19`× pequeno demais.

Perfis (borda superior, papel→corpo): r=10 `255,190,1` → `255,188,84,5`; r=40 `255,97,30` →
`255,222,170,118,61,25` (fringe preservado). Perf: pior caso patológico (dilution 0.6 + warp 48 +
r40) ≈ 4,7 ms/composite, sob a barra de 8 ms.

**Desfecho (ideia do Enio, mesmo dia): os DOIS modos coexistem** — checkbox **Smooth Edges** no card
Wash (`BrushSpec::smooth_edges`, default **true**). OFF restaura o hard edge pré-AA **byte a byte** —
o fingerprint original (`0xc5ebf8cf645fb6f6`), aposentado quando o AA virou universal, voltou como o
oráculo do modo duro (`smooth_edges_off_is_the_pre_aa_render_byte_for_byte`; a mutação "render ignora
o flag" sangra nele). O modo é lido UMA vez por composite (mistura por-dono costuraria o rim onde
duas silhuetas se encontram).

### A metade do IMPASTO (mesmo dia, `impasto_smooth_edges` + checkbox no card Body)

O filme do impasto (`height_film::film_of`) endurece a silhueta numa banda FIXA de `t` ⇒ binário em
todo raio (r=10 saltava `255→0`; r=20 `255→226→2`). Diferenças estruturais que mudaram a cura:

- **O alpha do pigmento compõe LINEARMENTE** (sem Beer–Lambert) ⇒ a fração de área entra direto no
  `w` — mas tem de atravessar a **porta única** (`FilmAa::film_at` em `height_film.rs`) nos DOIS
  consumidores (funil do pigmento em `dab.rs` + envelope de cobertura da luz em `height.rs`, cada um
  com a própria cadeia de silhueta — disco vs cápsula varrida) ⇒ pigmento e luz concordam sobre onde
  a tinta termina por construção. A banda é resolvida por **bissecção por-dab** (o padrão
  `body_edge_t`); Shape image fica duro por design (o precedente do carimbo).
- **O build-up entre dabs re-satura a fração** (0.64→0.94 com dabs a 1px) — o cap de Accumulate-OFF
  existia mas só era armado com `Strength < 1` (premissa falsificada pelo AA). Armado sob AA com a
  fórmula `cap = w·coverage` e `add = a_dab·(1−m/cap)`: em `cap = 1` (interior) reproduz o maskless
  **byte a byte**; no aro capeia pela ÁREA. ⚠️ A 1ª fórmula (salto pro alvo `w·g·cov`) impôs o cap do
  Grain a traços full-strength e mudou TODO interior granulado — os gates de material (wax/shine)
  pegaram; a bisseção mostrou que nenhum dos 3 mecanismos era o culpado, era o ARMAR do mask.
- **Quantização no toe**: fração < 2 quanta corta pra 0 na porta (pigmento e byte do filme
  arredondam independentes; um vai a 0 e o outro a 1 = luz em canvas nu).
- **Perf**: o supersampling era pago onde o filme já saturou — `film_of` é sub-quantum de 1.0 em
  sil≈0.58, muito antes do `W_SOLID=0.75`; resolver a banda no ponto de SATURAÇÃO (não no corpo
  sólido) cortou o anel 2.2× (r=100: 28→12.5 texels; +2.0 → +0.9-1.2 ms/move, todas as seções sob o
  kill 8). O custo restante mora no pass de ALTURA (serial); o do pigmento é absorvido pelas bandas
  paralelas.

Oráculos do modo OFF: **4 fingerprints** pré-AA (r=4/6/10/20), byte a byte. Perfis com AA:
r=4 `255→115→0` · r=10 `255→185→67→0` · r=20 `255→182→55→3`. Três fixtures de material foram
**afiados** (não afrouxados): wax/shine medem em `cov==255` (a população pálida do aro diluía o
agregado sem tocar o mecanismo) e o gate de canvas-nu exige vizinhança-8 nua (o aro AA é sempre
adjacente a tinta; o overshoot de 26px que ele caça tem miolo cercado de nu).

### Lições generalizáveis

1. **Uma melhoria medida num estágio intermediário é invisível se um estágio NÃO-LINEAR a jusante a
   satura.** Aplique a correção depois do estágio saturante (alpha linear na aparência final) e
   gateie nos PIXELS renderizados, nunca na grandeza intermediária.
2. **AA é forma × sombreamento**: a fração de cobertura nunca deve modular a substância (densidade,
   pigmento) — só a mistura final. Modular substância = double-fade ou saturação.
3. **O fixture tem de usar os números do produto** (raio ~10 do smoke, não o raio 4 conveniente) — o
   v1 foi verde por medir onde o bug não morava.
4. **"Existe texel cinza" não é oráculo de AA** — o pré-fix tinha um 190 antes do penhasco e passaria;
   o oráculo é o TAMANHO DO DEGRAU máximo na descida.
5. **Um remap não adiciona resolução espacial**: alargar a janela do smoothstep não cria um segundo
   texel de rampa; só supersampling (ou alpha por fração de área) cria.
6. **Deslocamento por-pixel (warp) quebra gates de gradiente**: quando a amostragem é warpada, os
   subsamples têm de atravessar o MESMO warp em offsets de output — e o gate de "onde preciso de AA"
   pode nunca ver a borda que corre ENTRE os centros amostrados.

**Área:** seam da UI (painel `ph2d-panel-painter-layers` ↔ `ph2d-tool-painter`). **Não** é a matemática
do rig — essa tem 6 gates e 3 mutações vermelhas (`16_impasto_plano_implementacao.md` §18).
**Estado:** 🔎 **ABERTO** — fila de amanhã, por ordem do Enio.

### Sintoma (Enio, 2026-07-12, print)

*"UI não funciona, nem o checkbox nem se pode selecionar outra luz."*

Os chips `1 2 3 4` do card **Lighting** **pintam** (o print mostra `1` selecionado e `2· 3· 4·`
apagados — os pontinhos são a marca de "desligada", então **o snapshot chega certo no painel**) e
**não respondem ao clique**. O checkbox **Enable** também não; mas isso pode ser *consequência*: ele só
é pintado quando a lâmpada selecionada é ≠ 1, e não dá pra selecionar outra.

### Causa — NÃO IDENTIFICADA (e não vou adivinhar)

Duas hipóteses levantadas e **descartadas na leitura**:

1. **Colisão de id**: passei `PAINTER_IMPASTO_LIGHT_1` como `group_id` do segmented **e** como id da
   opção 1. → **Descartada**: `paint_segmented_adaptive` **ignora** o `group_id` (só mapeia
   `widget.options` para `paint_segmented_group_adaptive`).
2. **Falta de `store.register` em `populate.rs`** ([[feedback_panel_populate_register]]). →
   **Descartada**: os segmentos de **Depth Source** / **Draw To** também não estão em `populate.rs` e
   funcionam.

**Candidatos ainda NÃO checados:**

- **A altura do `card_frame`.** O segmented **reflui** (4 chips num painel estreito podem virar 2
  linhas), mas eu dimensionei o card por uma contagem **fixa** de linhas (`rows = 6`, ou 7 com o
  Enable). Se o conteúdo estoura o card, o **card seguinte é pintado por cima** — e os hit-rects dele
  ganham. O print reforça: o card parece **curto demais**, terminando logo abaixo dos chips.
- A **ordem dos arms** em `event.rs::handle_event`.

### A LIÇÃO — e é a terceira vez que ela cobra

Gatei a **MATEMÁTICA** do rig com 6 gates e 3 mutações vermelhas, e escrevi **ZERO gates no seam da
UI**. O `ph2d-ui-testkit` existe exatamente para isso: um teste headless que **clica no chip 2** e
afirma que `impasto_rig.selected == 1` teria saído **vermelho antes de o Enio abrir o app**.

É [[feedback_painted_is_not_populated_paint_gate]] (*pintado ≠ populado: teste a PINTURA... e o
CLIQUE*) e [[feedback_tool_unit_green_integration_dead]] (*unit-verde ≠ funciona no produto*) outra vez.
**Um widget novo não está pronto quando pinta — está pronto quando um teste clica nele.**

### Ordem de amanhã (não negociável)

1. **Escrever o gate do seam PRIMEIRO.** Headless: clica o chip 2 → `selected == 1`; clica Enable →
   `lights[1].on`. **Ele nasce VERMELHO.** Sem ele, qualquer fix é chute.
2. Só então diagnosticar (candidatos acima).
3. Consertar. É **UI pura**: não toca a matemática, e nenhum dos 6 gates do rig deve se mexer.

---

## Bug #14 — Impasto: "a tinta extravasa o relevo" (o suporte batia, e a foto estava errada)

**Área:** depósito de pigmento (o alpha do dab) × corpo do impasto × passe de luz.
**Estado:** ✅ Resolvido em 3 rodadas — cada uma corrigiu uma coisa **real** e as duas primeiras não
mataram o sintoma. **Aberto e adiado (ordem do Enio, 2026-07-12): a TINTA EMPURRADA (Push) — fim da
fila, depois de toda a implementação.**

### Sintoma (Enio, três smokes seguidos)

1. *"o efeito leva em consideração os limites do pincel e não o peso do relevo. Este falloff (smooth)
   pinta tinta fora do relevo. Usando o falloff **Sphere** fica mais preciso e a tinta corresponde ao
   relevo."*
2. *"regrediu ao deixar a tinta extravasar o relevo e não resolveu a distância da tinta levantada."*

Uma névoa de vermelho pálido em volta do tubo iluminado — **tinta sem forma**.

### Causa-raiz — TRÊS camadas, e as duas primeiras não bastavam

**(a) O pigmento não conhecia o corpo.** Duas coisas já concordavam sobre onde a tinta deixa de ter
corpo: o **relevo** (`body_profile`, zero abaixo de `W_TAIL = 0,35` de cobertura) e a **luz** (pesa a
sombra pela *mesma* curva, pra não branquear o papel — o halo de um smoke anterior). O **pigmento** não
sabia de nada disso: depositava até o **limite geométrico do disco**.

A largura da saia é **função pura do falloff**, e é *por isso* que o falloff parecia o culpado:

| falloff | `W_TAIL` cai em | saia |
|---|---|---|
| `Smooth` (default) | t = 0,61 | **39% do raio** (16 px num pincel de 40) |
| `Sphere` | t = 0,94 | 6% |

**Sphere não era mais preciso — ele só não tem saia.**

**(b) A luz morria em pressão parcial.** A luz pesava por `body_profile(cover)` com `cover` = tinta
**crua** (`silhueta × dinâmica`) — as **dinâmicas dentro da curva**, onde podem matá-la de fome: a
Strength 0,3 o argumento cai sob a cauda em **todo texel** e a luz não modela nada em traço nenhum.
Invisível com mouse (que sempre aperta a 1,0); com **caneta**, é o mesmo bug de volta.

**(c) ⚠️ A que de fato o Enio via: OPACIDADE NÃO É ESPESSURA.** Depois de (a) e (b) o gate estava
**verde** — *"o pigmento existe exatamente onde a luz modela"*, provado, mutação-vermelha — e a foto
**continuava errada**. Medido, no pincel do próprio smoke:

| t | tinta | sombra |
|---|---|---|
| 0,38 | **227** | 103 |
| 0,47 | 133 | 49 |
| 0,55 | 15 | 7 |

A tinta e a sombra somem **juntas** (suporte idêntico!) **ao longo de 8 px de rampa suave**. E uma rampa
suave de vermelho pálido, sem forma 3D, **é** uma névoa. **Um gate de igualdade-de-conjuntos não
distingue uma parede de um banco de neblina.**

A física errada: **a opacidade de um filme satura muito antes que a espessura dele** (Beer–Lambert).
Tinta a óleo com um décimo da espessura já é praticamente **opaca** — é por isso que uma espátula deixa
uma **borda**, não um gradiente. Modelar o alpha como *proporcional ao corpo* era modelar tinta como
**vidro**.

### Como a causa (c) foi finalmente encontrada — RENDERIZANDO E OLHANDO

Duas horas de teoria (build velho? outra rota de depósito? pressão do mouse?) contra um teste
`#[ignore]` que **pinta o traço num PNG** e um `Read` da imagem: a névoa estava lá, **no próprio
harness**, idêntica à foto. A medição transversal dizia "limpo"; a imagem dizia "névoa". **A imagem tinha
razão.**

### A solução — uma função, três consequências

**O FILME** (`ph2d-painter-brush/src/height_film.rs`, módulo-irmão novo):

> **Um pincel que não deposita corpo não deposita tinta** — e a tinta que ele deposita é **opaca**.

```rust
film_of(sil)      = film_opacity(body_profile(sil))   // early-out nos 2 extremos constantes
film_opacity(d)   = 1 - (1 - d)^8                     // satura rápido; HR-5 (3 quadrados, zero exp)
solid_paint(s, d) = d * film_of(s)                    // o alpha do filme = o peso da LUZ
```

**Onde o corte mora: na SILHUETA** — nem no grão, nem nas dinâmicas. Os dois foram pagos com vermelho:

- **Não nas dinâmicas:** a Strength 0,5 o pico do dab é 0,25 < `W_TAIL` ⇒ a curva zera **todo** texel e o
  traço **não deposita nada**. Um pincel que não pinta não é um fix. *A borda de um filme é da ponta, não
  da força com que se aperta.*
- **Não no grão:** o `cover` que a luz pesa não tem grão de propósito (grão texturiza o pigmento, não
  escava o corpo). Cortar através dele faz os vales perderem a tinta **mantendo o corpo cheio** ⇒ luz
  total sobre papel nu (**124 níveis / 1694 px**, medido).

Assado nas **2 máscaras cacheadas** (`StampKey` ganha `lays_body`) + 1 sítio por-pixel; tudo a jusante
(grão, dinâmicas, teto do Accumulate-OFF, rampas, cor por-camada) herda a silhueta remodelada **sem
aritmética**. O relevo segue derivando da tinta **crua** ⇒ Depth/Body/Source/Smoothing/Push **vivos**.

**Isto também explica o Sphere:** silhueta quase plana até a borda ⇒ o filme dele já alcançava corpo
cheio em 1–2 px. **Ele já fazia isto, por acidente de forma.** Agora todo falloff faz.

### Verificação — o gate certo mede ÁREA, não suporte

`impasto_paint_has_an_edge_not_a_fringe`: *de toda a tinta do traço, quanta não é nem sólida nem
ausente?*

| | opaca | translúcida | **névoa** |
|---|---|---|---|
| sem filme (o bug original) | 6122 | 6620 | **52%** |
| filme ∝ espessura (o 1º corte) | 5108 | 2036 | **28,5%** |
| **filme com opacidade** | **6396** | **1000** | **13,5%** |

…e a área **opaca CRESCE** enquanto a névoa cai: **a tinta não encolheu, virou sólida.** As duas
mutações são vermelhas. O suporte não separava nenhuma das três versões; a **área** separa as três.

Outros gates com RED provado: `the_film_never_starves_the_brush_at_low_strength` (MUT cortar as
dinâmicas → **0 px pintados**) · `the_film_binds_only_a_brush_that_lays_body` (Impasto OFF /
`DrawTo::Color` / Depth 0 = byte-idênticos) · `the_light_models_a_faint_stroke` (MUT dinâmica de volta na
curva → **0 níveis** a Strength 0,5).

Perf: **3,18 ms/movimento @2048² · 3,27 @4096²** (alvo ≤4) — o `film_of` corta os dois extremos
constantes, e a cauda é a maior parte da bbox de um dab, então ficou **mais rápido que antes da curva**.

Commits: `877f8080` (o filme) · `b7ce38cc` (a luz) · `8769b0a3` (a opacidade). Detalhe completo:
[`16_impasto_plano_implementacao.md`](16_impasto_plano_implementacao.md) §14–§16.

### ⚠️ ABERTO (adiado por ordem do Enio, 2026-07-12) — **a tinta EMPURRADA**

*"a tinta empurrada ainda não resolveu. Adiar para o final de toda essa implementação. Fim da fila."*

O **Push** (conservação de volume, §13 do plano) é real-time, conservativo, vivo e idempotente — a crista
sobe sob o pincel e a soma fecha em zero. Mas o **desenho** da tinta deslocada ainda não convence. Não
foi diagnosticado: **fica no fim da fila**, depois de todo o resto do Impasto.

### Lições generalizáveis

1. **Um gate VERDE não prova que você mediu a coisa certa.** *"O pigmento existe exatamente onde a luz
   modela"* era verdade, provada, com mutação vermelha — e cega para o sintoma. Igualdade-de-**suporte**
   não vê *quanta* tinta e *quanta* forma há em cada pixel.
2. **Quando o Enio contradiz um gate verde: RENDERIZE E OLHE.** Um teste `#[ignore]` que despeja um PNG e
   um `Read` da imagem mataram em um minuto o que a teoria não matou em duas horas. **O pixel é o
   oráculo; a métrica é uma sombra dele.**
   ([[feedback_render_and_look_when_a_green_gate_is_contradicted]])
3. **Sintoma visual ⇒ métrica de ÁREA ou CONTRASTE**, não de suporte. "Quanta tinta é neblina" separa as
   três versões (52% / 28% / 13,5%); "onde há tinta" não separava nenhuma.
4. **O workaround do usuário é uma pista da física.** O `Sphere` "mais preciso" não era precisão: era uma
   silhueta que já saturava a opacidade num pixel. Quando um preset acidental conserta o bug, **descubra
   o que ele faz** — é a lei que falta.
5. **Um limiar pertence à forma; a dinâmica multiplica depois.** Errei isso duas vezes, em lados opostos
   do mesmo cano (o corte do pigmento e o peso da luz), e as duas vezes o sintoma foi um **knob morto**
   em pressão/força parcial.

---


## Bug #13 — VARREDURA: a família do #12 (o guard que pergunta "existe?" em vez de "que FORMA tem?")

> **Estado: ✅ 3 bugs RESOLVIDOS na linha `line/Painter` (2026-07-12).** Varredura pedida pelo Enio após o
> #12, com 4 lentes independentes. Achados **abertos** listados no fim — nenhum é crash.

### A raiz única

O #12 não era um bug isolado: era uma **espécie**. O guard de reuso pergunta *"já inicializei?"*
(`pre.is_empty()`, `cov.len() != n`, `is_empty()`, `cuts.contains_key(&id)`) quando deveria perguntar
**"esse dado ainda pertence às entradas que o produziram?"**. Quatro lentes independentes convergiram nela.

**O caso que corrompe em silêncio é o SPRITE DO MESMO TAMANHO** — é justamente quando o guard de
comprimento *bate*.

### #13.a — PANIC: trocar de sprite com a tinta ainda molhada (`2e5e8444`)

`reset_transient_edit_state` (`tool/paint/lifecycle.rs`) é o choke point **declarado** do "abandone tudo em
progresso ao trocar de documento" — o doc-comment dele diz que *"fecha essa classe inteira"*. Foi escrito em
2026-07-02 e lista 8 itens. **Três subsistemas nasceram depois e nunca se registraram nele:** a sessão
molhada da aquarela, o **Deform** e a **Seleção**. Todos guardam estado canvas-sized que **sobrevive ao gesto**.

- **PANIC:** `canvas_wet` é o único buffer canvas-sized que sobrevive ao pen-up (seca no heartbeat, ~10 s).
  `dry_canvas_wet` guardava com `is_empty()` e então indexava com o stride do sprite **atual** e um
  `canvas_wet_rect` em coordenadas do sprite **antigo**. Pintar aquarela → clicar num sprite **maior** dentro
  da janela de secagem → **SIGSEGV no tick seguinte**, sem mais nenhuma interação.
- **Deform:** `deform.pre` é um snapshot "pristino" validado só pelo **comprimento** ⇒ sprite do mesmo tamanho
  o mantinha, e o próximo reshape reamostrava o canvas novo **a partir dos pixels do sprite antigo**.
- **Seleção:** é tool-global (não está no `StashedDoc`, que guarda a seleção de *camada*), e
  `selection_restricts_paint()` só pergunta *"a máscara está não-vazia?"*. O sprite novo herdava a seleção do
  antigo e **toda pincelada fora dela era revertida** — o clássico *"não pinta e eu não sei por quê"*.

**Fix:** registrar os 3 no choke point + guard de **forma** em `dry_canvas_wet` (defesa em profundidade,
bounds-only, física intacta). Os dois fecham o panic **independentemente**.

### #13.b — Editar o Paper com wash molhado não re-texturizava a poça (`8a0de875`)

`wet_substrate` (o memo do dente do papel) é chaveado pelo **pixel do canvas e nada mais**; sua única
invalidação é o `NaN`-reset no **pen-down**, sob a premissa escrita no doc do campo: *"o papel não pode mudar
no meio do traço"*. **A feature live-editable wash (2026-07-11) tornou essa premissa falsa** — ela existe
justamente para re-renderizar quando um param de Paper/Grain se move. Como `fill_substrate_cache` só preenche
os `NaN`, todo pixel já memoizado mantinha o papel **antigo**: a poça re-renderizava e o papel não mudava.
**A feature derrotava a si mesma no slot Paper.** (O Grain é amostrado ao vivo — por isso a metade que
funcionava passou no smoke.)

> **★ LIÇÃO DE ORÁCULO (o teste ficou VERDE com o bug vivo):** meu 1º RED afirmava *"os pixels mudam"*.
> Passou — porque o re-render suja uma região **maior**, e os pixels recém-preenchidos (`NaN` → valor) mexem
> bytes **mesmo com 100% dos já-memoizados stale**. O oráculo certo é o **memo**, não o sintoma: comparar só
> os pixels que **já estavam** memoizados. Com o oráculo certo: **2304/2304 stale**.
> Ver [[feedback_oracle_must_model_appearance_not_implementation]] — aqui a variante: *oráculo largo demais
> fica verde por um caminho que não é o do bug*.

### #13.c — O cache de cut-points do compositor sobrevivia à troca de sprite (`3ef7be66`)

O compositor cacheia um **cut point** por camada de Adjustment: o acumulador **abaixo** dela, um buffer
dimensionado para **aquele** canvas. `set_source` constrói um `LayerStack` novo — e `LayerStack::new()`
**reinicia `next_id` em 1**, então os ids do documento novo **colidem com os do antigo por construção**. O
cache nunca era limpo, e o guard só pergunta *"existe um cut para esse id?"*.

- sprite **maior** ⇒ indexa além do fim do acumulador antigo (**panic**);
- sprite do **mesmo tamanho** ⇒ a Adjustment do sprite novo compõe sobre as **camadas-de-baixo cacheadas do
  sprite antigo** — preview errado, e **silencioso**.

**A assimetria denunciava o bug:** `restore_doc` (o rebind irmão) **sempre** limpou os 4 campos; `set_source`
nunca limpou nenhum. *Mesmo seam, duas portas, uma trancada.*

### Lições generalizáveis

1. **Um guard de reuso deve validar a PROCEDÊNCIA, não a existência.** "Já aloquei?" ≠ "isto pertence às
   entradas de agora?". Comprimento é a dimensão mais fraca da forma — **o mesmo tamanho é exatamente o caso
   que corrompe calado**.
2. **Um comentário que declara uma premissa é uma dívida com data de vencimento.** *"O papel não pode mudar
   no meio do traço"* era verdade quando foi escrito; a feature seguinte o desmentiu e **ninguém releu o
   comentário**. Toda premissa escrita num campo deveria virar `debug_assert` ou gate.
3. **Um choke point só protege quem se registra nele.** `reset_transient_edit_state` foi criado para matar
   esta classe, e a classe voltou por **três subsistemas novos que não sabiam que ele existia**. Choke point
   precisa de um **gate que force o registro**, não de boa vontade.
4. **Assimetria entre irmãos é o cheiro mais barato de bug.** `restore_doc` reseta 6 campos, `set_source`
   resetava 2 — os dois são o MESMO seam. Comparar caminhos-irmãos acha bug mais rápido que ler qualquer um
   deles a fundo.
5. **Auditores concordando não é prova.** Duas lentes apontaram o `wet_substrate`; meu primeiro probe
   **refutou** (mediu errado) e o segundo **confirmou**. Sempre execute — dos dois lados.
6. **★ Um comentário desatualizado é pior que nenhum comentário — ele MENTE com autoridade.** Duas vezes
   nesta varredura: (a) o `stamp_route` afirmava que os shape-editors *"fall through to the plain deposit"*
   com Watercolor ligado — **falso desde o doc 13 #3** (eles rodam a ótica por `stamp_drag_preview_watercolor`),
   e eu quase reportei ao Enio uma condição de UI errada por causa dele; (b) os setters do Paper Rake,
   mantidos *"for the API"*, fizeram uma auditoria inteira concluir que havia botões mortos no painel —
   **não havia botão nenhum**. Corolário: **código/comentário sem consumidor não é neutro, é uma armadilha
   armada** — quem chegar depois vai acreditar. Se removeu a UI, remova o encanamento.
7. **"Está morto" e "está vivo" são AMBOS afirmações que exigem teste.** Esconder um knob morto exige provar
   que ele é inerte (byte-identidade); manter um knob exige provar que ele muda a saída. O gate
   `under_the_wash_accumulate_is_inert_but_strength_is_not` pina os dois lados — senão o painel mente numa
   direção ou na outra.

### ⚠️ ABERTOS na varredura (nenhum é crash) — precisam de decisão ou fila

| Achado | Gravidade | Nota |
|---|---|---|
| ~~**Paper Rake / Paper Random**~~ | ✅ **FECHADO** (`970d47c8`) | **O auditor errou:** os BOTÕES nunca existiram — o painel já os tinha removido, com o raciocínio certo (o papel é substrato ancorado no canvas; não há dab, logo "a rotação segue o traço" não tem o que rodar). O que sobrou foram **ids + setters + arms + campos mantidos "for the API"**, e o encanamento morto virou **armadilha**: a auditoria leu os setters sobreviventes como knobs vivos. Removidos os 6 sites. **Lição: código morto não é neutro — ele MENTE para o próximo leitor.** |
| ~~**Tiling + Random Angle: a costura não fecha**~~ | ✅ **FECHADO** (`a6da10de`) | As cópias wrapped sacavam do RNG **uma vez cada** ⇒ os dois lados da costura sorteavam ângulos diferentes. Fix: `tiled_dabs_grouped` + `DabRng` reproduzem o stream **por dab original**. Aplicado nas 3 rotas de pintura **e** nas 2 passagens do wash. Sem tiling ⇒ byte-idêntico. **Armadilha de oráculo:** a 1ª versão do RED tinha off-by-2 no mapeamento canvas→local (o centro da cópia é `cx+span`) e falharia em QUALQUER implementação — um **falso RED**. |
| ~~**Accumulate visível sob o wash**~~ | ✅ **FECHADO** (`under_the_wash_…`) | `brush.accumulate` é lido **só** pelo `accumulate_cap`, dentro do roteamento de stamp — e a aquarela faz short-circuit **antes** dele. Checkbox pintado-mas-inerte. É também **redundante por construção**: a cobertura do wash é max-blend (um envelope), que já **É** "não acumula dentro do traço". Escondido sob `watercolor_active` (o predicado REAL: `watercolor && Paint && !eraser` — em Eraser/Mask/Inpaint o depósito comum volta e o Accumulate volta a valer). **`Strength` FICA** — provado vivo: o engine o assa no `Dab.coverage` e o wash o lê como o pico do depósito. |
| ~~**Grain Rake/Random dentro da aquarela**~~ | ✅ **FECHADO** (`330339ae`) | **Provados inertes** (washes byte-idênticos). Na aquarela o slot Grain **É** o mapa de granulação — substrato ancorado no canvas, não um carimbo que o dab carrega. Escondidos sob `watercolor_active`; voltam em Eraser/Mask/Smear, onde o dab carimba. |
| ~~**Jitter Rotate morto em Smear/Blur/Clone**~~ | ✅ **FECHADO** (`c22b7d42`) | `has_per_dab_rotation()` exigia textura ativa, mas o Jitter Rotate gira o **footprint**. Dab achatado sem Shape/Grain parecia constante → máscara cacheada → mesma elipse em todo dab. |
| ~~**Grain `Random Offset` no cache constante**~~ | ✅ **FECHADO** (`c22b7d42`) | O `per_dab_dynamic` esqueceu `randomises_offset()`. Trocado pelo predicado canônico `!texture.is_cacheable()` — um lugar só, o buraco não reabre. |
| ~~**Opacity por-camada inerte no achatado**~~ | ✅ **FECHADO** (`f0e686bf`) | `flatten()` rodava **antes** de `set_layers_meta` (opacidades ainda 1.0) e `set_opacity` nunca re-achatava. O termo `op` dentro do `flatten()` era código **inalcançável**. Virou choke point. |
| ~~**`ColorStampKey` sem `granulation`**~~ | ✅ **FECHADO** (`c22b7d42`) | O bake a assa; a chave não a carregava. |
| ~~**`wet_editable_tex` / `AppearanceSig` incompletos**~~ | ✅ **FECHADO** (`330339ae`) | Paper Depth e Granulation vivem no `BrushSpec`, **fora** de `TextureSettings` → arrastar Paper Size re-renderizava a poça e Paper Depth, o slider ao lado, não fazia nada. `AppearanceSig` não via Ramp Alpha nem Tiling — os setters dizem *"só afeta stamps futuros"*, verdade para o canvas, **falso** para uma forma aberta (cujo preview É um stamp que ainda não pousou). |
| ~~**Kernels `pub` só com `debug_assert`**~~ | ✅ **FECHADO** (`8a1cb4c0`) | `debug_assert` **some** no build que o artista roda. Os kernels agora **bailam** (`None`) em vez de indexar fora — batch perdido é bug que se vê; OOB é crash na máquina dele. |
| ~~**Per-Layer Color OFF→ON no meio do traço apaga tinta**~~ | ✅ **FECHADO** (`8a1cb4c0`) | `pre` é o canvas PRÉ-traço; o dab pintado com o modo OFF não está nele nem nos mapas → o recomposite o **evaporava**. O `fits()` valida a FORMA dos mapas, não a **continuidade** do traço. O toggle agora dropa a acumulação. |
| **Watercolor OFF→ON no meio do traço** | 🔎 **ABERTO** | Mesmo mecanismo suspeito (o `watercolor_base` é congelado no pen-down). **NÃO corrigido de propósito:** não consegui construir um RED — o dab plano nem chega a pintar no harness, então não sei o que estou corrigindo. Regra do projeto (e ordem do Enio: *não ferir a aquarela*): **sem RED refutável, não se mexe**. O fix tentado (re-congelar o ground no toggle) foi **revertido**. |
| **Gates de paridade banda-vs-serial dependem da máquina** | cobertura | Num runner de 1 core os gates "bit-identical to sequential" comparam serial contra serial — verdes e vazios. Nenhum gate força a contagem de bandas. |

---

## Bug #12 — PANIC/SIGSEGV ao apertar Rake com um traço Per-Layer Color vivo

> **Estado: ✅ RESOLVIDO na linha `line/Painter` (2026-07-12).** Fix + 2 gates. Aguarda integração ao main.

### Sintoma (Enio, 2026-07-12)

Pintando com Per-Layer Color, traço freehand desenhado na tela **em tempo real**, ao apertar **Rake** na
seção Shape:

```
PH2D PANIC frame=34949 location="crates/ph2d-painter-brush/src/stamp_color/accumulate_batch.rs:347"
message="range end index 3911680 out of range for slice of length 1048576"
… encerrada pelo sinal SIGSEGV
```

### Causa-raiz — **as rotas do Per-Layer Color usam mapas de tamanhos de elemento DIFERENTES**

As três rotas alocam o mesmo `PerLayerStroke.cov`, mas com **bytes/pixel diferentes**:

| Rota | `len` de cada mapa | Kernel |
|---|---|---|
| `stamp_dabs_cached_color` | `w·h` — **1 B/px** (cobertura alpha-only) | `accumulate_color_stamps_fused_batch` |
| `stamp_dabs_cached_color_rgba` · `stamp_dabs_per_layer_dynamic` | `w·h·4` — **4 B/px** (premul RGBA) | `accumulate_*_rgba_batch` |

E o guard de reuso **nunca perguntava o tamanho do elemento**. Ele perguntava só:
`pre.is_empty()` (*"já inicializei este traço?"*) e `cov.len() != n` (*"mudou o número de camadas?"*).

**Rake troca a rota** (`shape_has_per_dab_rotation` → `per_dab_dynamic`, `stamp_route.rs:429`) — e o traço
**continua vivo**, então `pre` não está vazio e `n` não mudou ⇒ **nenhuma re-alocação**. A rota dinâmica
então fatiou os mapas de `w·h` como se fossem `w·h·4`:

```
gy0*w*4 .. gy1*w*4   →   3911680 = linha 955 × stride 4096
len do mapa          →   1048576 = 1024² × 1 B/px      ⇒  fora do slice
```

**A aritmética fecha exatamente com o crash do Enio:** canvas 1024², rota cacheada, lida com stride de 4.

**O flip REVERSO (Rake desligado) era pior porque era SILENCIOSO:** os mapas de 4 B/px são *grandes demais*,
nunca estouram o índice — o recomposite cacheado apenas lê **bytes premul-RGBA como se fossem cobertura**.
Corrupção visual sem panic. Mesma causa-raiz; se o fix só olhasse a direção que crashou, o bug teria
*migrado* em vez de morrer.

### A solução — um guard de FORMA, num ponto só

1. **`PerLayerStroke::fits(n, len)`** (`stamp_color_cache.rs`) — *"os mapas já estão na forma desta rota?"*.
   **O tamanho do elemento passou a fazer parte da forma**, que é justamente o que faltava.
2. **`PainterTool::ensure_per_layer_stroke(incremental, n, len)`** — **as três rotas passam por aqui**. Um
   choke point: a próxima rota que alguém adicionar **não consegue** esquecer o guard.
3. No flip, a **cobertura acumulada** é descartada (os mapas das duas rotas não são o mesmo dado) mas a
   **base do traço (`pre`) sobrevive** — o recomposite segue reconstruindo a partir do canvas pré-traço, em
   vez de compor em cima de si mesmo. Os métodos de fill re-estampam a forma inteira a cada move, então
   **não perdem nada**.

### Verificação (RED verificado nas DUAS direções — DIRETIVA §3)

- `per_layer_color_route_flip_mid_stroke_reshapes_the_maps` — cacheada → Rake → dinâmica.
  **RED confirmado** desligando o fix: `index out of bounds: the len is 4096 but the index is 6792`
  (o mesmo mecanismo do crash do Enio; num canvas de teste pequeno o batch cai no kernel **serial**
  `accumulate.rs:256` em vez do **bandado** `accumulate_batch.rs:347` — mesma causa, kernel diferente).
- `per_layer_color_route_flip_back_reshapes_the_maps_too` — dinâmica → Rake off → cacheada.
  **RED confirmado**: mapas ficavam em 16384 B onde deviam ter 4096, e o pixel saía errado.
- Suítes: **544** `ph2d-tool-painter` + **231** `ph2d-painter-brush` verdes; clippy `--all-targets` 0 warnings.

### Lições generalizáveis

1. **Guard de reuso de buffer tem que checar a FORMA INTEIRA, não a identidade.** `pre.is_empty()` responde
   *"já aloquei?"*, não *"aloquei do jeito que EU preciso?"*. Toda cache reusada por rotas diferentes
   precisa que **o layout faça parte da chave** — contagem **e** tamanho de elemento.
2. **Editar o brush no meio de um traço vivo é um estado real, não um caso de canto.** O painel e o canvas
   estão vivos ao mesmo tempo; qualquer knob que **troque de rota** (Rake, Random, Jitter, Grain Tiled,
   Randomize Color) pode fazê-lo com buffers de meio-traço na mão. **Todo roteador de stamp deve assumir
   que a rota pode mudar entre dois batches do MESMO traço.**
3. **Corrija a CLASSE, não a direção que crashou.** O flip reverso não estourava índice — corrompia calado.
   Um fix que só olhasse o panic teria deixado metade do bug vivo, e a metade pior (silenciosa).
4. **Um choke point > N guards corretos.** Três cópias do mesmo guard já tinham divergido (o incremental nem
   checava `n`). Uma função por onde todas as rotas passam é o que impede a 4ª rota de nascer quebrada.

---

## Bug #11 — Per-Layer Color: linhas retangulares intermitentes (ABERTO)

> **Estado: ABERTO e DORMENTE.** Nada foi corrigido. A caçada de 2026-07-11 **não achou a causa**, mas
> **eliminou quase todo o espaço de busca** e deixou uma **armadilha re-ativável** (§Armadilha). Leia a
> tabela de descartados ANTES de tentar de novo — ela economiza rounds inteiros.

**Sintoma (Enio 2026-07-11, smoke em `--release` LIMPO):** ao usar **Per-Layer Color** com **shapes
dinâmicas** (Free Hand / Ellipse / Polygon), aparecem **linhas nas bordas de retângulos**, **nas cores do
próprio brush** (não em cor de chrome). Enio: *"parecem os retângulos da umidade que foram resolvidos
(Bug #9), mas aparecem como linhas nas bordas dos retângulos."* Na screenshot: um pretzel free-hand já
desenhado + um editor de **Ellipse ativo por cima**, sendo editado, com um **círculo-fantasma deslocado**
à direita.

**O fato que domina tudo: é INTERMITENTE.** Apareceu; depois **3 runs seguidas sem reproduzir** (inclusive
COM Free Hand, o método que o Enio suspeitava ser o gatilho). Isso mata a abordagem "reproduz e bissecta"
e é a assinatura clássica de **memória não-inicializada** (Bug #2 lição #4) *ou* de uma condição de
timing/ordem (a troca de produtor CPU↔GPU).

### O que foi DESCARTADO (com o método — não repita)

| Suspeito | Veredito | Como foi descartado |
|---|---|---|
| **Composite CPU** (canvas + cache `composited`) | ❌ **DESCARTADO** | **9 testes** (`per_layer_*` em `tool/paint/tests.rs`): o cache parcial (`composite_region`+`blit_region`) é **byte-idêntico** a um recompose CHEIO em shrink, forma que se move, multi-shape, Free Hand auto-sobreposto, **multi-move-por-frame**, parked-freehand+ellipse-ativa, caminhos **cached E dinâmico** (Randomize Color) |
| **Upload parcial GPU** (`preview_upload_bbox`) | ❌ DESCARTADO | `PH2D_PAINT_FULL_UPLOAD=1` → o artefato **PERSISTIU** |
| **Tiling / Repeat Image** (`draw_repeat_image`) | ❌ DESCARTADO | Enio confirmou **Tiling OFF** (a função faz early-return) |
| **Slot GPU não-inicializado** | ❌ Já corrigido (Bug #2) | `clear_all_mips_transparent` presente em `individual.rs::create_entry_empty` |
| **Upload de camada por versão (GPU)** | ❌ DESCARTADO | `pixel_clock` **incrementa** a cada `bump_layer_pixels`; `ensure_slice` sobe a camada **inteira** quando a versão muda |
| **Resíduo no canvas** (restore/recomposite) | ❌ DESCARTADO | `dab_bbox` e a footprint do accumulate usam a **mesma** fórmula (`floor(c−r)..ceil(c+r)+1`); `restore_region` **marca dirty** |
| **Produtor GPU** (`painter_gpu_preview::try_drive`) | ⚠️ **RESTA** | Intestável no harness CPU; **o `FULL_UPLOAD` não o toca** |
| **Overlay** desenhado por cima | ⚠️ **RESTA** | Não passa pelo composite nem pelo upload. Candidatos: `draw_overlays` (symmetry / ellipse / polygon / **stencil**), `draw_selection_overlay` |
| **Tamanho do canvas** | ⚠️ **Condição provável** | Quando apareceu, os dirty bboxes chegaram a `(227,56,635,893)` ⇒ canvas **≥ ~862×949**. As 3 runs limpas foram em **512×512** |

### A pista mais forte que sobrou (leia antes de tudo)

O `PH2D_PREVIEW_DIAG` provou que **as edições de shape rodam no produtor CPU** (`gpu_owns=false`), MAS o
log tinha um bloco de **~2710 frames `gpu_owns=true`** no meio (um **arraste de slider** — o produtor GPU
assume o slot para sliders rápidos). Ou seja: **o preview ALTERNA de produtor** durante a sessão. A
troca CPU↔GPU é o único caminho que (a) o harness headless não alcança, (b) o `FULL_UPLOAD` não cobre, e
(c) depende de timing/ordem — casando com a intermitência. **Comece por aí.**

### Armadilha (re-ativável — já commitada, custo ZERO desligada)

Duas metades em [`painter_bridge.rs`](../../shells/desktop/src/render_loop/painter_bridge.rs):

```bash
# 1) Qual produtor tem o slot + o bbox do upload parcial, por frame:
PH2D_PREVIEW_DIAG=1 ./target/release/ph2d-host-desktop 2>/tmp/diag.log

# 2) O composite CPU exato que vai subir (ANTES de qualquer overlay), 1 PNG por frame:
mkdir -p /tmp/dump && PH2D_PREVIEW_DUMP=/tmp/dump ./target/release/ph2d-host-desktop
```

**Como usar quando o artefato reaparecer:** reproduza **no sprite GRANDE** com o dump ligado e **feche o
app no instante em que o retângulo aparecer**. Então:
- **Retângulo NOS PNGs** ⇒ está no composite ⇒ os 9 testes estão errando alguma condição do gesto real;
  compare o frame ruim contra o que o teste gera.
- **PNGs LIMPOS enquanto o artefato está na tela** ⇒ o composite é inocente ⇒ é **overlay** ou o
  **produtor GPU**. (Este é o desfecho que a evidência atual favorece.)

### Lições (já pagas — não repita)

1. **9 verdes no harness ≠ bug inexistente.** É a [[feedback_harness_reproduces_mechanism_not_context]] de
   novo: gastei 9 tentativas headless reproduzindo o *mecanismo* (restore/recomposite) sem o *contexto*
   (produtor GPU, canvas grande, timing). O doc já mandava parar em 1-2 e **instrumentar o app** — e foi a
   instrumentação (`gpu_owns`) que produziu a única pista real. **Pare o harness mais cedo.**
2. **Bug intermitente: a NÃO-reprodução não é prova de correção.** Enio: *"alguma coisa que vc fez deve ter
   resolvido"* — o `git diff` provou o contrário: **+21 linhas, todas dentro de `if env::var_os(...)`**, zero
   mudança de comportamento. É o falso-negativo do Bug #2 **invertido**: lá um binário stale fez um fix certo
   parecer morto; aqui a não-reprodução faz um bug vivo parecer morto. **Sempre cheque o diff antes de
   aceitar "resolveu".**
3. **Eliminar tem valor mesmo sem resolver.** Esta entrada não tem solução — tem um **espaço de busca
   reduzido a 2 suspeitos** e uma armadilha armada. Registrar isso é o que evita o próximo round começar do
   zero (é literalmente para isso que este doc existe).
4. **Compare contra o ORÁCULO certo.** Comparar gesto-vs-gesto **cancela** um bug geometria-dependente (os
   dois lados passam pela mesma via parcial). O oráculo que vale é **cache parcial vs recompose CHEIO** do
   mesmo estado — é exatamente a diferença que o `FULL_UPLOAD` **não** consegue corrigir.

---

## Bug #10 — Borda dura na junção ao mudar params de Wash e cruzar traço úmido — params por-dono degrauavam

**Sintoma (Enio):** pintar um traço, mudar **Body/Concentration/Edge/Opacity/RaggedEdge** e cruzar o traço
ANTERIOR ainda úmido imprimia uma borda dura na junção; e o **Warp** do traço NOVO re-warpava a junção do
VELHO (artefato). Regra do Enio: *o traço antigo NUNCA muda por config do novo.*

**Causa-raiz:** a **mesma classe do Bug #8 (lição #4)**, que a gente só tinha resolvido pro `wet`. Os OUTROS
params por-dono contínuos (fill/Body, depth/Concentration, edge_gain/Edge, opacity/Opacity [novo do #17],
warp/RaggedEdge) eram lidos DISCRETOS via `style_at` (recência por disco) → **degrauavam na fronteira de
posse** (a junção). O `wet` já era CAMPO borrado (`build_wet_field`); os 5 outros faltavam.

**Fix:** `build_style_field` generaliza o `blur(param·mask)/blur(mask)` MASCARADO por posse pros 5 params —
o traço velho mantém os SEUS params, a fronteira suaviza. Detalhes que evitam regressão:
- **Só quando os donos DIFEREM** (`params_differ`) ⇒ senão discreto, **byte-idêntico** (o blur uniforme
  reproduz o valor). Sessão de um estilo / params iguais = zero mudança.
- **Mascarado por posse** (só pixels DONOS contribuem, owner-0 do brush vivo NÃO) ⇒ não vaza pros pixels
  de um wash que **não se toca** com o novo (o guard `..._do_not_touch_baked_washes` segue verde). Um
  descuido meu incluiu o owner-0 e vazou o brush novo por cima do gap — o guard pegou na hora.
- **Warp lê PRÉ-warp** `(lx,ly)` (a amplitude vem antes do deslocamento); os outros no warped `(sx,sy)`.
- **Geometria/cor** (`color`, `core_r`, `spread_*`, paper/grain) ficam DISCRETOS (lição #4: campo pra
  grandeza física contínua, discreto pro que é realmente discreto).
- RED→GREEN: gradiente máx da junção com params trocados **118 → 13 bytes/px** (medido desabilitando o campo).

**Junto:** as bordas da umidade (mesma classe de boundary-step) ganharam um blur GENTIL do véu (r=4) — agora
SEGURO porque a raiz do retângulo (#9) foi corrigida (o over-blur anterior só espalhava o retângulo do
pour-união, que não existe mais). Só o preview; o mapa de umidade fica intacto. E **undo agora limpa a
umidade** (`dry_session_now` no `restore_model` — o canvas mudou de identidade, a sessão molhada é stale).

**Lição:** quando um fix por-dono (o `wet` do #8) resolve UM parâmetro, **todos os params da mesma família
que alimentam termos contínuos têm o mesmo bug latente** — generalize o mecanismo, não trate caso a caso.

---

## Bug #9 — Preview de umidade: "retângulo" na união — o pour re-molhava o vizinho dentro do BBOX

**Sintoma (Enio, cruz/blob de traços úmidos):** um retângulo escuro axis-aligned na união quando um traço
cruza/cobre outro traço AINDA ÚMIDO — sumia no pen-up-e-secar, "aparecia e desaparecia". O `Preview=0`
(slider da Wetness) matava → **é o overlay de umidade** (não o wash).

**Pista falsa (custou 1 take):** dumpei o mapa de umidade + o wash tool-side numa junção FRESCA → **cruzes
limpas**, zero retângulo. Conclusão precipitada: "o tool está certo, o bug é no draw do shell". Errado — o
teste não tinha SECAGEM entre os traços (o contexto real). Reproduzindo COM `paint_tick` entre A e B o
retângulo apareceu no dump: os braços de A recuados (secagem edges-to-center) + a região de B a 255.

**Causa-raiz:** `stroke_coverage` é a UNIÃO da sessão. O `pour_canvas_wet` iterava o RECT do traço
(`wet_stroke_dirty`) e despejava a cobertura da UNIÃO → **re-molhava a 255 os pixels do wash ANTERIOR que
caíam dentro do bbox do traço novo** = um retângulo (o bbox) preenchido com a umidade do vizinho. O fix
anterior (#4, rect por-traço em vez do cumulativo) só ENCOLHEU o retângulo pro bbox do último traço; a raiz
é despejar a UNIÃO em vez da FOOTPRINT do traço.

**Tentativa ERRADA (revertida):** achei que era degrau fresh-vs-decaído e **borrei o alpha do véu**
(`box_blur_f32`). O blur só ESPALHOU o retângulo real → "ficou gigante" (Enio). Sintoma tratado, causa
intacta.

**Fix:** `pour_canvas_wet` restringe à **footprint DONA** — `wet_styles.owner[i] == cur_o` (recência). A
sobreposição que o traço de fato molhou entra (o owner-map já é a footprint por recência); os pixels
só-do-vizinho não são tocados. `has_owner` falso ⇒ pour do rect todo (comportamento pré-owner-map).
RED→GREEN: sonda de A dentro do bbox do B-diagonal mas FORA da footprint de B — **179 → 255** sem o
owner-check, **fica 179** com (`watercolor_overlapping_bbox_does_not_rewet_the_neighbour_wash`).

**Lições:**
1. **Reproduza o CONTEXTO, não só o mecanismo** (de novo, [[feedback_harness_reproduces_mechanism_not_context]]):
   a cruz-limpa mentia porque faltava a SECAGEM. O "tool-side limpo" só vale se o teste tem o mesmo estado
   (aqui: um vizinho parcialmente seco). Quando o dump contradiz o report, **cheque o que o teste NÃO tem**.
2. **Retângulo axis-aligned = operação por RECT, não por forma.** Sempre que o artefato é um retângulo, o
   suspeito é um `for y in rect { for x in rect }` que devia ser por footprint/cobertura. Aqui o pour.
3. **Blur é tratar sintoma.** Se o campo tem um degrau/patch ERRADO, borrar só o espalha. Ache a fonte do
   patch (o pour da união) antes de suavizar.
4. **Owner-map É a footprint por recência** — o jeito canônico de dizer "só o que ESTE traço pintou" na
   sessão única; reusável pra qualquer op que não pode tocar o vizinho (mesma família do #13/#4).

---

## Bug #8 — Aquarela: borda dura/serrilhada nas junções + "retângulo no preview" — 6 fixes verdes sem efeito: o harness reproduzia o MECANISMO, não o CONTEXTO

**Sintomas (Enio, cruz de traços):** (1) borda dura/pixelada na junção entre traços, persistindo
após pen-up — gatilho "só ao reduzir Charge" (mixer on), e voltando ao subir Rewet; (2) um
"retângulo" no preview com Charge 1 + Dilution > 0, sumindo no mouse-up.

**A armadilha (a parte que enganou por UMA SEMANA de takes):** SEIS fixes consecutivos, todos com
mecanismo provado RED→GREEN no harness — e o Enio reportando "nenhuma diferença" em todos. A
causa não era nenhum dos mecanismos: era a LACUNA harness×app. Todo repro rodava
`Falloff::Constant`, hardness 1, **warp 0, granulation 0, sem Paper, radius 12-14, pressão fixa
e sem `paint_tick`** — o app roda o preset Watercolor (feather, warp 6-12, gran 0.3, PaperCold),
radius 40-100 e o heartbeat por-frame (soak/secagem vivos). Fixes provados num motor que o app
não executa.

**O método que virou o jogo (doc 12, takes 7-10):**
1. **Instrumentação no app real** (`[wet-diag]`: spec efetivo por pen-down com 4 casas decimais +
   estado da sessão por pen-up + salto de pickup do mixer) — 1 gesto do Enio colado do terminal
   valeu mais que os 6 takes de mecanismo.
2. **Bissecção por toggle** (`PH2D_PAINT_FULL_UPLOAD=1`, já existia do Bug #2) — separou
   shell-upload de composite-CPU sem escrever código.
3. **Perfil 1D + sondas por-pixel no harness com os params EXATOS do dump** — o penhasco estava
   no eixo que ninguém tinha escaneado (a fronteira do footprint do traço novo DENTRO da tinta
   velha): degrau inteiro em 1-2 px onde a borda orgânica espalha por 15-25 px. As sondas
   (`[maps]`/`[wetmaps]`) mataram os suspeitos óbvios um a um: coverage/deplete/water PLANOS no
   penhasco.

**Causas-raiz (dois portadores independentes + um já morto):**
- **"Retângulo" (sintoma 2): já estava resolvido** pelo take 6 (água só interage com tinta SECA —
  o re-molho retroativo da união era o retângulo). Os reports "sem diferença" eram
  build/condição velha. Confirmado não-reproduzível com e sem o toggle.
- **Portador A (Charge<1 seco):** o alpha do color-buffer rampa ~20 bytes/px na borda do
  footprint e a janela de confiança `COL_LO..COL_HI` (8..32) era atravessada em ~1 px espacial —
  o flip cor-crua↔cor-depositada imprimia a linha (serrilhada pelo warp nearest). **Fix:** lerp
  proporcional `ca8/255` no blend do pigmento (7.5 → 1.6 bytes/px).
- **Portador B (Rewet difere entre traços):** o `wet` do DONO entrava **binário** em todos os
  termos wet-driven (thinning do interior — o maior clareador —, boost do edge, mix, granulação)
  via o mapa de dono (recency por disco). **Fix: molhado é CAMPO, não estilo** —
  `build_wet_field`/`sample_wet_field` (blur mascarado por posse, `blur(wet·m)/blur(m)`, raio
  fixo 8 px < gap do guard de não-contato) — 11.5 → 1.9 bytes/px, clareamento intacto.
- **Desvio no caminho (take 9, revertido):** um clamp de magnitude no depósito do mixer matou a
  borda MAS matou junto o clareamento da junção — que era o look desejado. Veto do Enio
  ("perdeu o efeito de clareamento") → a spec correta é **clareia E suave**, codificada no guard
  `watercolor_junction_lightening_is_soft_and_preserved` (clareia > +2 bytes E grad ≤ 4
  bytes/px, com Rewet 0 e 1).

**Lições generalizáveis:**
1. **Smoke contradiz fix provado ⇒ pare de iterar mecanismo e feche a REPRODUÇÃO** — dump dos
   params reais no app (1 eprintln por evento) + réplica 1:1 no harness. É a
   [[feedback_harness_reproduces_mechanism_not_context]], agora com o protocolo que funcionou.
2. **Meça o PERFIL espacial, não só o valor:** "borda dura" = contraste÷distância. O degrau de 8
   bytes em 1 px é uma linha dura; os mesmos 8 bytes em 20 px são orgânicos. O teste de dureza
   certo é o gradiente máximo de 1 px num scan que cruza a fronteira — e há mais de um eixo pra
   escanear.
3. **Limiar fixo em bytes sobre um campo com rampa espacial variável = linha dura latente** (a
   janela COL_LO..HI). A versão suave de um limiar de confiança é o lerp proporcional.
4. **Params por-traço resolvidos por mapa de dono degrauam na fronteira de posse** quando o
   parâmetro alimenta termos contínuos — para grandezas físicas espaciais (umidade), o campo
   borrado (mascarado por posse, pra não vazar pra tinta que não se toca) é a representação
   certa; o estilo discreto fica pro que é realmente discreto (cor, geometria).
5. **Registre o look ANTES de "corrigir" um efeito visível:** o clareamento parecia bug e era
   feature. Fix de aparência sem o veredito do dono do look = risco de take extra.

---

## Bug #7 — Aquarela: "grave queda de FPS" — build profile + composite 2×/frame + loops seriais, NÃO os algoritmos

**Sintoma (Enio 2026-07-07, em 3 rodadas):** (1) brush >200px com recursos de aquarela = queda severa de
FPS; (2) após os primeiros fixes, "ainda lento"; (3) **mesmo brush pequeno** (16px) com Bleed/Ragged
Edge/Rewet/Smudge/Pigment no máximo = queda grave. A aparência acusava "algoritmo de aquarela pesado" —
e essa pista estava **quase toda errada**.

**Os VERDADEIROS culpados (por ordem de impacto, todos medidos antes de corrigir):**

1. **O build profile — o maior de todos.** O smoke rodava via `cargo run` = **dev opt-0**; o motor
   custava **10,9 ms/frame em debug vs 2,9 ms em release** (~4×) com os knobs no máximo, e o resto do
   app (Vello/compositor/painel) somava o próprio overhead de debug ⇒ estourava os 16,7 ms e o vsync
   amplificava (perde o vblank → 22-28 fps). O brush comum não sentia porque é ~memcpy; a aquarela é
   matemática por-pixel real (LUTs, blurs, paper procedural) — exatamente o que opt-0 massacra. **Fix:**
   `[profile.dev.package.*] opt-level=2` só nos 4 crates de paint-math (idiom do `ci-test`; dev
   10,9→3,75 ms) + smoke de feel SEMPRE em `--release` (`058dabf0`).
2. **Composite 2×/frame (achado da instrumentação do shell).** Durante o gesto, o Move flush compunha a
   janela E o heartbeat (`on_tick`) compunha DE NOVO — o `grow_wet_soak` forçava `stamped=true` todo
   tick. O frame profiler mostrou `stamps` e `tool-tick` carregando ~4-5 ms CADA. **Fix:** `stamped |=
   parked` — movendo, o soak só folda no dirty (o próximo composite ≤1 frame o inclui); parado (quando o
   bleed crescer sob a ponta É o efeito), segue ao vivo. Bake byte-idêntico; tick em movimento
   3,75→0,01 ms (`4e00e8e8`).
3. **Loops seriais O(janela) num caminho embaraçosamente paralelo.** O composite por-pixel, o `box_blur`
   e o fill dos campos rewet são funções puras por-pixel/linha sem redução — rodavam single-thread por
   disciplina "sem rayon" (que existe pro replay determinístico). **Fix:** exceção sancionada
   **ADR-0109** (3 invariantes: sem redução entre pixels · sem estado mutável compartilhado · sem RNG ⇒
   bit-idêntico independente de nº de threads): composite ∥ (`d775c31c`), box_blur ∥ por transposição
   (`93c14b94`), fill dos campos rewet ∥ (`8ac5be35` — o Rewet era o último knob furando 60fps porque
   com Bleed ≤12 os campos rodam em resolução CHEIA, ds=1). R220 "tudo": frame max 51→10 ms, commit
   238→44 ms.
4. **Paper procedural recomputado todo frame.** PaperCold = ~28 hashes inteiros/pixel, canvas-anchored
   (mesmo pixel ⇒ mesmo valor o traço inteiro) — recalculado a cada composite. **Fix:** memoização
   `wet_substrate` (f32/px, NaN=não-computado, reset no pen-down — o papel não muda no meio do traço,
   então não existe invalidação in-stroke pra errar). `+paper` virou grátis após o 1º toque
   (`2e19c9a0`).
5. **(Não-bug, mas o fato que explica o "mesmo brush pequeno"):** com Rewet+soak o custo por frame é
   **pad-dominado** — a janela recomposta = dirty + 2·(2·Bleed + Ragged) ≈ ±144 px por lado com os
   knobs a 48, **independente do raio do brush**. É a física pedida pelos knobs (1 dab influencia
   ±144 px), não desperdício: o algoritmo já era assintoticamente correto (dirty-rect incremental +
   downsample ds + bake 1×).

**Pistas falsas REFUTADAS por medição (tão importantes quanto os fixes):**
- **"É churn de alocação"** (17 Vecs/frame ∝ janela): implementado o reuso de buffers, medido — **zero
  ganho** (números idênticos dentro do ruído). Os spikes eram compute-bound; o alocador já reciclava
  bem. Revertido no ato — mudança que não se paga não fica.
- **"É a GPU / o upload / o painel":** frame profiler provou GPU 0,8-4 ms (inocente), upload já parcial
  (dirty-bbox), `hero-paint` estável ~1-3 ms.
- **"É o mixer (Charge/Dilution/Pull)":** ablação mostrou ~0,4 ms — desprezível.
- **"O motor está lento" (rodada 3):** ablação reversa com a config exata do Enio provou o motor a
  **2,9 ms/frame em release** — dentro do orçamento com folga; o problema era o item 1.

**Ferramentas que resolveram (e ficam):** sonda de **ablação reversa** (tudo-no-máximo, desligando 1
knob por vez — isola o culpado em uma rodada) · **frame profiler do shell** estendido com
`tool-tick`/`stamps`/`hero-paint` (`PH2D_FLUID_PROFILE=1`, `07d079b9`) — foi ele que pegou o composite
2× que nenhuma sonda unitária pegava, porque só o app real tem a ordem Move-flush→tick do frame ·
sondas temporárias em `--release` sempre revertidas após medir.

**Estado final (validado pelo Enio no app, release):** ~60 fps com todos os recursos; Rewet-only medido
1,1-2,2 ms/frame no motor (R16-R60 × Bleed 7-48). Restante conhecido: pen-down carrega o
`build_wet_backdrop` full-canvas (~15-25 ms 1×/traço @2048² — o S4/backdrop-regional da auditoria doc
12 é o fix mapeado, byte-idêntico, se o soluço de início de traço incomodar).

**Lições generalizáveis:**
1. **"App lento" começa no build profile, não no algoritmo.** Confira `opt-level` ANTES de otimizar
   código — 4× de graça. Corolário: feel-test é em `--release`; e per-package `opt-level` no dev
   profile dá smoke realista sem custar o build inteiro.
2. **Sonda unitária mede o motor; só instrumentar o SHELL pega interação entre fases.** O composite
   2×/frame era invisível em qualquer teste da crate — existia na ordem real Move→tick do frame. Quando
   o probe diz "rápido" e o app diz "lento", instrumente o pipeline inteiro e deixe o split apontar.
3. **Meça para REFUTAR, não só para confirmar.** A teoria da alocação era plausível e caiu em uma
   medição; sem ela, teríamos complexidade permanente (thread-locals) por zero ganho. Fix que não se
   paga, reverte.
4. **Paralelismo byte-idêntico existe e é auditável:** função por-pixel pura + fatias disjuntas + zero
   redução ⇒ bit-igual em qualquer nº de threads. Mas política de determinismo se fura por ADR com
   cerca explícita (ADR-0109), nunca por conveniência local.
5. **Custo pad-dominado engana:** "até brush pequeno é lento" parecia bug e era a janela de influência
   dos knobs (2·Bleed+Ragged). Entender O QUE escala com O QUÊ (ablação por knob) evita otimizar o
   termo errado.

---

## Bug #5 — Offset de curva densa amontoava os pontos após Convert

**Sintoma (Enio 2026-07-05):** depois do **Convert to Curve** (que passou a gerar curvas densas de múltiplos
pontos, ~16px de espaçamento, no P6), aplicar **Offset para dentro** amontoava as âncoras (uma elipse de 24
âncoras encolhida pra raio ~10 ficava com âncoras a **2.5px** umas das outras) → pontos sobrepostos + artefatos
+ "perda da perfeição da curva". Regressão introduzida pela densificação do Convert (antes o Convert dava
poucos pontos, que não amontoavam).

**Causa-raiz:** o offset do stroke **movia os pontos de controle** — `offset_curve_refined` reconstruía a
curva inteira (densify adaptativo + join CAD) e o overlay/fill exibiam a curva OFFSETADA. Num offset pra
dentro de uma curva densa, as âncoras encolhem proporcionalmente ao raio e se amontoam; o offset em si é fiel
(o spine bate no raio certo), mas os pontos de controle exibidos ficam sobrepostos.

**Tentativa que falhou (e a lição):** re-fluir a curva offsetada pra densidade uniforme quando *bunched*.
Corrige o amontoado, mas (a) a 16px de espaçamento os handles Catmull-Rom sub-estimam um círculo pequeno e
encolhem 26%; a 8px fica OK — mas (b) fundamentalmente é **remendo do sintoma**: os pontos ainda se movem e a
curva editável é reconstruída a cada frame. Enio apontou o caminho certo: **"veja como é feito em Seleção —
offset sem mover os pontos, movendo apenas o desenho."**

**Tentativa #2 que falhou (importante):** offsetar o **spine** como polilinha via `line_offset::offset_polyline`
(miter). Mantém os pontos pristinos, mas a curva de desenho fica **imperfeita** — o offset por miter numa
polilinha não é a curva paralela verdadeira (Tiller–Hanson é exato só pra reta+círculo). Enio: *"o resultado
ficou pior, o offset produz curvas imperfeitas. temos documentado como conseguir a curva perfeita."* A **curva
perfeita** já existia: `offset_curve_refined` (reconstrução CAD adaptativa sub-pixel, Levien 2022) — o header
do `curve_offset.rs` a documenta.

**Solução final (drawing-only puro, modelo da Seleção):** o EDITOR inteiro fica na curva **PRISTINA** — âncoras
+ handles + gizmo + a **linha-guia** (`curve_overlay.spine` = flatten da curva pristina, SEM offset) — e **só
o desenho pintado** (os dabs pretos, `curve_fill`/re-stamp de parked) sofre o offset. O desenho usa a curva
paralela PERFEITA: `offset_curve_spine` roda `offset_curve_refined` (CAD adaptativo) e guarda só o spine
achatado. `bake_curve_offset` virou **no-op** (o offset nunca materializa nos pontos; Apply-&-Keep só acumula
o valor). Resultado: nada no editor se move ou amontoa (ponto e linha parados na fonte da verdade) + a pintura
= a parametrização paralela exata. Como a linha-guia é pristina, o "bico"/artefato que aparecia no guia
offsetado **some** (o amontoado e o cruzamento só existiam nas âncoras/guia offsetados, nunca na fonte).

**Lição generalizável (a saga inteira):** três tentativas, uma pista decisiva. (1) re-flow das âncoras
offsetadas = remendo do sintoma. (2) offset por miter da polilinha = rebaixou a curva perfeita. (3) certo:
**a fonte da verdade (âncoras + linha) fica pristina; só o resultado renderizado sofre o offset** — exatamente
como a Seleção offseta a máscara e não a curva. A pista do Enio ("offset movendo apenas o desenho, ponto e
linha parados") era literal: o problema nunca foi o ALGORITMO de offset (o CAD já era perfeito no spine
pintado), foi **exibir/editar sobre a geometria reconstruída**. Quando um offset "move os pontos" gera
artefatos, não troque o offset — pare de mover a fonte.

**Coda — o "bico nos pontos convertidos" (2026-07-05, agentes):** mesmo com o offset drawing-only, o Convert
ainda **assava o offset na geometria** — `stroke_state_to_curve_state` (multi-shape, ramo Curve) rodava
`offset_curve_refined_kinds` e gravava o resultado direto nos pontos de controle; num canto côncavo/over-offset
o join CAD **divide o vértice em dois pontos que se cruzam** (de propósito — o Trim corta a "orelha" no
DESENHO), mas gravado como âncora vira um V auto-cruzado editável. `bake_ellipse_offset` no Convert single
também assava (limpo, mas assava). Enio: *"por que a Seleção faz a mesma coisa e sai perfeita?"* — porque a
Seleção **nunca assa offset nas âncoras** (o offset dela é na máscara). Prova: converter elipse (mesmo
excêntrica rx=120/ry=20), círculo e polígono é **byte-perfeito** (0 cruzamentos, desvio < 0.0004) — o bico só
vinha do assado. **Fix:** o Convert agora produz a geometria **PRISTINA** e o offset **persiste** como
transform de desenho (nada assado, slider não reseta) — `offset_curve_refined_kinds` deletada. Toda âncora
fica exatamente na forma. (Método: 2 agentes — um comparou Stroke vs Seleção linha-a-linha e isolou o ramo
Curve; a densify/split_cubic/ellipse-math provou-se idêntica e correta nos dois. Nota operacional: o agente de
worktree vazou tests `dbg_` no arquivo principal — limpos manualmente.) Bônus: botão **Simplify** estava
escondido em curva convertida (gate só `FreeHand || added_point`); agora aparece em qualquer curva fechada
editável.

**Coda 2 — o "bico" no MERGE (2026-07-05):** o Convert já pristino, mas o **Merge** ainda cuspia bicos nas
cinturas côncavas (peanut). Causa: o Merge (`merge_open_shapes_to_curves` / `selection_merge_curves`) fitava o
contorno traçado com `to_closed_curve_precise` (Schneider tight erro 1.0 + densify 16px) → **muitos** pontos
que, numa cintura pinçada, viravam uma agulha auto-cruzada (turn ~1.94 = quase 180°). O próprio Enio achou o
fix: *"Simplify resolve; reduzir os pontos gerados melhora."* Novo helper `to_closed_curve_smooth` roda o
mesmo redutor robusto do Simplify (`simplify_closed_smooth` — DP + Catmull-Rom corner-aware): âncoras a menos
(62→32 no pinch), zero auto-cruzamento, cintura limpa. Ambos os Merges (stroke + seleção) usam. Lição:
tracing-de-máscara + fit-tight amplifica o staircase do contorno em bicos; o redutor DP-fechado é a saída.
Merge usa tolerância própria `MERGE_SIMPLIFY_TOL_PX=1.0` (mais densa que Simplify).

---

## Bug #6 — Simplify "quase bom" + offset arredondava as quinas → REFIT + vértice reconstruído

> Desenho final consolidado (algoritmos, constantes, testes-âncora):
> [`09_curvas_convert_merge_simplify_offset.md`](09_curvas_convert_merge_simplify_offset.md).

**Sintoma (Enio 2026-07-05):** "não ficou bom o simplify… faça pesquisa de como gerar curvas simplificadas
perfeitas, com números de pontos bem reduzidos. Descubra os tipos de handles adequados." E depois do fix do
Simplify: "os vertex das quinas fazem um offset ruim (arredonda as quinas no offset)."

**Simplify FINAL — refit por mínimos quadrados (pós-pesquisa):** duas iterações anteriores
(decimação DP + handles Catmull-Rom; depois Visvalingam-20% + kinds Symmetric/Vector derivados) produziam
curvas "quase" — porque decimar pontos e derivar handles genéricos **não é fit**. A pesquisa (Schneider 1990,
o pipeline do Inkscape/paper.js `simplify()`; Levien 2023) manda: (1) **detectar quinas** (cusps) no spine
denso (janela de ±3px de arco, giro ≥ ~70°, supressão não-máxima > 2×janela — senão UMA quina vira duas);
(2) **fit cúbico por mínimos quadrados** (Schneider, `fit_curve`) em cada trecho ABERTO entre quinas (aberto
= zero risco do colapso do Bug #4); (3) **kinds do fit**: junção suave = **Aligned** (braços colineares de
comprimentos independentes — o fit carrega a forma nos comprimentos; Symmetric os igualaria e distorceria),
quina = **Free** (braços fitados independentes; Vector os apontaria pros vizinhos e mataria a curvatura de
aproximação). Anel sem 3 cusps ganha seams artificiais nos terços (re-suavizados pra Aligned). Progressivo:
cada aperto escala a tolerância (0.5px ×1.7…) até ~20% das âncoras caírem. Módulo novo `curve_refit.rs` — o
funil único de Simplify E Merge. Resultado medido: círculo denso 16 âncoras → **3 Aligned** (spine a <1.5px
do círculo real); pentágono 15 → **5 Free exatas**. Lições: (a) simplificar curva = REFIT, nunca decimação;
(b) o raio de supressão do detector de quinas tem de exceder o span de resposta (2×janela); (c) anel fechado
precisa ≥3 seams — um cúbico engole meio-anel dentro da tolerância e o assembly degenera.

**Quinas × offset (2026-07-05, follow-up):** com o Merge refitado, o offset **arredondava as quinas**. Causa:
o trace da máscara é SUAVIZADO (média móvel), então a ponta da quina chega ~2px arredondada; o fit ancorava a
quina EM CIMA da ponta arredondada com tangentes estimadas nos primeiros samples (borrados) — e **o offset
amplifica o arredondamento por |d|** (ponta raio 2px offsetada 20px = arco visível de raio 22px). No merge
denso antigo, vários pontos na região reproduziam a quina apertada — por isso "estava bom antes". **Fix
(`CORNER_TRIM_PX`/`corner_vertex` no `curve_refit`):** apara ~3px de arco de cada lado da quina REAL (a ponta
suavizada) e re-ancora os runs do fit na **interseção das duas retas de borda** (medidas num baseline limpo de
3-9px) — vértice-navalha na curva, miter exato no offset. Medido: quadrado com pontas arredondadas → quinas
reconstruídas a <1.2px do vértice verdadeiro; offset de 12px alcança o ápice do miter a <1.5px (arredondado
ficaria ~5px aquém). Lição: quando um consumidor AMPLIFICA erro (offset × curvatura de ponta), a fonte tem de
reconstruir a geometria ideal, não reproduzir fielmente o dado degradado (o trace suavizado).

---

## Bug #4 — Simplify Curve degenerava: o Schneider fit NÃO fecha loops

**Sintoma (Enio 2026-07-05):** ao pedir **Simplify Curve** numa seleção convertida (ex.: um retângulo →
curva densa de 8 pontos), a curva **colapsava**: uma região virava 2 pontos idênticos (`[[58,58],[58,58]]`),
outra virava um triângulo de 3 pontos. A cobertura da seleção sumia. O `Simplify` antigo só rodava com UMA
curva (`selection_shapes.len() == 1`), então isso nunca fora exercido no caminho denso do P6.

**Causa-raiz (o que enganava):** o *spine* achatado estava **perfeito** — 25 pontos formando um quadrado
limpo, `spine[0] == spine[24]`. O problema é o **fitter**: `ph2d_painter_brush::fit_curve` (Schneider,
Graphics Gems 1990) fita **polilinhas ABERTAS** — preserva os extremos e estima as tangentes das pontas por
`p1−p0` / `p_{n-2}−p_{n-1}`. Num **loop fechado** onde `start == end` (ou start≈end, mesma aresta), as duas
tangentes brigam no ponto de costura e o least-squares/reparam de Schneider **colapsa a curva inteira num
único cubo degenerado**. É por isso que o Free Hand funciona (a captura da caneta é **aberta** e só depois é
marcada `closed`) e o Offset/Apply-&-Keep "funcionava" (alimenta contornos **densos** traçados, muitos pontos
→ o fit sobrevive exceto num segmento na costura). Uma curva já-fechada limpa e esparsa (saída do Convert)
não tem pontos suficientes pra mascarar o colapso.

**Tentativas que falharam:** (a) achatar **aberto** (sem duplicar a costura) e re-fechar → ainda degenera:
start e end na mesma aresta a ~9px, o fitter aproxima o quadrado inteiro por 1 cubo. (b) densificar antes do
fit → a degeneração é de **tangente na costura**, não de densidade; não resolve pro caso esparso.

**Solução (`curve_geom::simplify_closed_smooth`):** trocar o fitter por um redutor **closed-loop-correto**:
achatar pro spine denso → **Douglas–Peucker fechado** (`selection_trace::simplify_closed`, tolerância 3px →
âncoras precisas + poucas) → atribuir a cada âncora sobrevivente uma tangente **Catmull-Rom** (⅓ da corda
adjacente por lado), **colapsando pra quina dura** quando o giro local ≥ 60° (`dot(dir_in, dir_out) ≤ 0.5`). Um
retângulo volta a ser 4 quinas afiadas exatas; um laço orgânico fica tão suave quanto um Free Hand.
Transcendental-free (dot + sqrt). O `Simplify` agora roda em **TODAS** as curvas Freehand da lista (antes ou
depois do Merge), não só quando há exatamente uma.

**Lição generalizável:** um fitter de curva "de alta qualidade" pode ser **estruturalmente incapaz** de fechar
loops — o Schneider assume extremos distintos. Antes de reusar um fitter aberto em geometria fechada, cheque
o caso `start==end`; a densidade do input pode **mascarar** o colapso (Offset passava; Convert-esparso pegava).
Para curvas fechadas, DP-fechado + tangentes por vizinhança é robusto e preserva quinas por design.

---

## Bug #3 — Queda de FPS: Warp, Shapes booleanas, e TODO arraste interativo

**Crates/arquivos:** [`shells/desktop/src/render_loop/painter_bridge.rs`](../../shells/desktop/src/render_loop/painter_bridge.rs),
[`tool/paint/selection_shapes.rs`](../../crates/ph2d-tool-painter/src/tool/paint/selection_shapes.rs) +
[`selection_raster.rs`](../../crates/ph2d-tool-painter/src/tool/paint/selection_raster.rs),
[`tool/paint/warp/transform_mesh.rs`](../../crates/ph2d-tool-painter/src/tool/paint/warp/transform_mesh.rs).
**Método:** auditoria de performance **multi-agente, 4 lentes** (Warp · Shapes booleanas · Composite/GPU ·
Alocação), medida em `--release`, correções cruzadas verificadas.

### Sintoma
Queda séria de FPS ao (a) arrastar o gizmo do **Warp**, (b) editar **múltiplas shapes de seleção com
operações booleanas**, e — latente — em qualquer arraste de pintura. Bench-verde escondia tudo.

### Causa-raiz (as 4 lentes convergiram)

- **★ Transversal (afeta Warp, pintura, seleção): deep-copy do canvas inteiro por-move.** O bridge do desktop
  segurava um `Arc::clone(canvas_rgba)` **entre frames** e a detecção de upload GPU era chaveada no
  **ponteiro do Arc** → o `Arc::make_mut(canvas_rgba)` do tool via `strong_count == 2` e **copiava o canvas
  inteiro** (16,8 MB @ 2048², **escala com o CANVAS**, não com a região editada) TODO move. Invisível aos
  benches (que não seguram o Arc entre moves) — o clássico bench-vs-live gap. Também penalizava o Per-Layer
  Color (Bug #2), num eixo que o harness §1.R nunca exercitou.
- **Shapes booleanas:** cada Move do gizmo **re-rasterizava TODAS as N shapes** no canvas inteiro (O(N·A),
  só uma mudou) **e** chamava `invalidate_composite()` → **derrubava o composite + upload GPU do canvas
  inteiro** por-move — apesar de a máscara de seleção **não** entrar no composite (compositor sem nenhuma
  referência a seleção; a marquee é overlay por-frame).
- **Warp:** a grade **pristina** era re-subdividida (Catmull-Rom) todo move (constante durante o arraste).

### Smoke Enio 2026-07-04 (noite) — o eixo TRANSFORM está fechado; per-layer/booleanas NÃO
- **Transform whole-image (P1): ✅ RESOLVIDO, smoke OK** — as bandas paralelas do composite
  (6,4–8,5 ms/move medidos) resolveram a lentidão em todos os 4 sub-modos.
- **Per-Layer Color (P3): causa live achada e fechada em CPU (2ª rodada, mesma noite).** O harness
  antigo setava cor custom em TODAS as camadas → media só o caminho CACHED. O uso real (camadas
  capturadas SEM pick) é **Texture Color, o default** → roteava o kernel DINÂMICO serial
  (`accumulate_shape_layer_rgba`): medido **354 ms/move (N3) e 1,87 s/move (N16)** @2048²·r100
  (`per_layer_perf_live`) — o "FPS 60→10". Três fixes: (1) `take_preview_arc` recompunha o bbox da
  shape inteira por-frame no stack não-trivial — `composite_region_linear` + `encode` agora em bandas
  paralelas (31 → 5 ms); (2) kernel dinâmico batched+banded+layer-fused (354→39 N3 · 1874→181 N16);
  (3) **rota nova `stamp_dabs_cached_color_rgba`**: Texture Color com orientação constante (sem
  Rake/Random/Jitter/Randomize/grain canvas-fixed) assa cada camada num stamp premul-RGBA COM a cor
  (o `render_color_stamp_mask` já sabia) e blita 4-canais fused/banded → **13,1 ms/move (N3, 27×) ·
  54,8 ms (N16, 34×)**; recomposite RGBA compartilhado com o dinâmico, também em bandas. N16×r100
  segue caso-GPU (documentado). Teste de comportamento novo:
  `per_layer_texture_color_paints_each_layers_own_rgb`.
- **Booleanas multi-shape (P4): causa live achada e fechada.** Não era o recompose (cacheado, 5 ms,
  coalesced): era o **overlay de marching-ants** — `selection_overlay_rgba` reconstrói um RGBA do
  canvas INTEIRO **todo frame** (o `phase` anima as ants ⇒ nunca cacheia; roda até parado) =
  **9,9 ms/frame serial @2048²·8 shapes**. Agora em bandas paralelas (per-pixel puro, bit-idêntico):
  **1,35 ms/frame** (7,3×). Harness: `perf_selection_overlay_frame`.

### Regressão 2026-07-04 ("booleanas multi-shape lentas DE NOVO") + fix definitivo
O cache por-shape (`a914a772`) estava INTACTO (re-medido: 5,0 ms/move cache vs 34,0 full). A lentidão live
era **entrega por-evento bruto**: o modo Selection nunca entrou no coalescing por-frame
(`coalesces_canvas_motion` só olhava o stroke method), então um mouse de alta Hz pagava o recompose de
~5 ms VÁRIAS vezes por frame — a mesma tempestade do Bug #2, no eixo da seleção. **Fix:** Selection
coalesce por-frame (gizmo/Rectangle/Ellipse/Automatic agem só na última posição; **Freehand lasso fica de
fora** — captura o path e precisa de todo evento). Guard estendido em
`coalesces_canvas_motion_is_true_only_for_restore_based_fill_methods`.

**No mesmo dia, o eixo Transform/Warp (que o revert do Fix A devolveu ao estado lento) foi fechado por
outro caminho:** medição com o Arc retido (`perf_transform_whole_image_table`) mostrou whole-image 2048² =
**188–218 ms/move** com o loop de gather = ~99% e o deep-copy do Arc = só ~1,3 ms — ou seja, o Fix A mirava
1% do problema (por isso "estritamente melhor" no papel e irrelevante na prática). Fix real: bandas de
linhas paralelas + fast-paths exatos no `over` + strips fora do `affected` viram memcpy + cache da
subdivisão pristina do Warp → **6,4–8,5 ms/move** (29×), byte-idêntico (bandas disjuntas), SEM tocar o
bridge. Mesma alavanca aplicada ao kernel per-layer (95 → 7,9 ms/move; ver
`HANDOFF_per_layer_color_perf_artifacts`). Lição nova: **um custo por-move IGUAL em máquinas muito
diferentes (M2 8GB vs 9950X 128GB) = trabalho serial O(canvas)** — paralelize antes de teorizar sobre
caches/uploads.

### Solução
- **Bridge (`2c64ba80`) — ❌ REVERTIDO (`461dcafd`, 2026-07-04).** A ideia era: `needs_upload` do sinal
  `preview_dirty` em vez do ponteiro do Arc + soltar o clone após o upload → `make_mut` in-place. **Smoke do
  Enio mostrou o oposto: regrediu Warp E Per-Layer Color JUNTOS.** Dois tools não relacionados piorando em
  sincronia = mudança no caminho de display compartilhado, e essa era a única edição local no
  `painter_bridge.rs`. O ganho in-place nunca foi confirmado visualmente e na prática *piorou* — revertido
  por inteiro. **Lição atualizada abaixo (nº 5).** O eixo Per-Layer Color vai pra **GPU** (não mais CPU) —
  ver [`HANDOFF_per_layer_color_perf_artifacts`](../HANDOFF_per_layer_color_perf_artifacts.md) §4.2.
- **Seleção (`a914a772`) — ✅ mantida.** **cache por-shape** da cobertura (chaveado por valor da geometria, auto-validante;
  `Raster` por `Arc::ptr_eq`) → um arraste re-rasteriza **só a shape que moveu** — **medido 34,3 → 5,1 ms/move
  (6,8×)** com 8 shapes em 2048². E **removido o `invalidate_composite()`** da derivação da máscara (o
  composite é comprovadamente independente da seleção) → sem drop de composite/upload por-move.

### Lições
1. **Bench-verde ≠ live-green (o bench-vs-live gap é literal aqui):** o custo dominante (deep-copy do canvas)
   só aparece quando um clone do Arc é retido **entre frames** — exatamente o que o bridge faz e o harness
   não. Sempre modele o retentor real (ver o bench `perf_anchored_drag_per_move_cost` com `hold_preview`).
2. **Detecção de mudança por ponteiro é frágil + load-bearing:** chavear upload no `Arc::as_ptr` fazia o
   `make_mut` (que troca a alocação) parecer "mudou" — o desperdício estava sustentando a correção. Use o
   sinal semântico explícito (`preview_dirty`), não a identidade do Arc.
3. **Invalidação estrutural (`invalidate_composite`) num edit que NÃO toca o composite** = full upload grátis
   por-frame. Antes de invalidar, prove que a saída depende do que mudou (`grep` no compositor fechou isso).
4. **Multi-agente por lentes convergiu na mesma causa raiz** vista de 3 ângulos (Warp/Boolean/Alocação todos
   apontaram o deep-copy) — a triangulação deu confiança pra mexer no caminho de display. **MAS** (ver nº 5)
   convergência de análise estática ≠ prova; o benefício era teórico.
5. **★ Otimização de análise-estática sem smoke visual do caminho de display = aposta.** O bridge fix parecia
   estritamente melhor no papel (mata uma cópia de 16 MB/move) e ainda assim regrediu 2 tools. **Regra:**
   qualquer mudança no caminho de display **compartilhado** (`painter_bridge.rs`, upload GPU, lifecycle do
   Arc de preview) exige smoke visual **por-tool** (Warp *e* pintura *e* seleção) ANTES de considerar
   landada — o commit até se auto-marcou "NEEDS VISUAL SMOKE / revert is one commit", e foi exatamente isso.
   Dois tools piorando em sincronia ⇒ suspeite PRIMEIRO do caminho compartilhado, não de cada tool.

---

## Bug #1 — Offset de curva: as quinas não ficavam paralelas (nem cruzavam)

**Crates/arquivos:** [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/) →
[`tool/paint/curve_offset.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_offset.rs),
[`tool/paint/curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (novo),
[`tool/paint/curve_trim.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_trim.rs),
[`tool/paint/curve.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve.rs).
**Feature:** o slider **Offset** (card Offset + checkbox **Trim**) do editor de traço — gera a curva paralela
de Curve / Circle-convertido / Polygon-convertido / Free Hand, pra fora e pra dentro, aberta e fechada.

### Sintoma (e como ele evoluiu — o próprio sintoma enganou)

O Offset funcionava nos **trechos curvos** mas falhava nas **quinas**, e a descrição do sintoma mudou a cada
round porque cada fix resolvia uma camada e expunha a próxima:

1. *"O Polygon convertido em curva não offseta direito; parece o algoritmo antigo."*
2. *"Funciona com lados retos; só ao criar ponto novo e curvar é que piora."*
3. *"Lados curvos ficam mais distantes que os lados retos."* (quinas **encurtadas** vs. curvas)
4. *"O handle Free/Aligned/Symmetric piora; Auto/Vector é melhor."*
5. *"As quinas ficam pontudas e **não se cruzam**."* (sintoma final, decisivo)

### Causa-raiz (a verdadeira, achada só no fim)

Havia **duas** causas, em camadas:

- **Camada A — undershoot da quina.** O offset deslocava cada âncora pelo **normal médio normalizado** (unitário)
  × `d`. Num vértice suave isso é exato, mas numa **quina** (descontinuidade de tangente — handle colapsado/Free)
  a curva paralela verdadeira fica na **interseção das duas arestas offsetadas**, a `d / cos δ`, não a `d`.
  Resultado: a quina ficava **mais curta** que os trechos curvos por um fator `cos δ`. Isso explica os sintomas
  3 e 4 (Auto/Vector mantêm tangente contínua → sem descontinuidade → sem undershoot; Free/Aligned criam a
  descontinuidade → undershoot).

- **Camada B — a quina nunca cruzava.** Mesmo corrigindo a distância (miter na interseção), o algoritmo ainda
  produzia **um único vértice por quina**. E **um ponto único nunca se auto-cruza.** O padrão-ouro CAD é
  **offset-then-trim**: cada aresta é offsetada de forma independente, numa quina **côncava** as duas arestas
  **ultrapassam** uma a outra (cruzam), e um passo de **Trim** corta a orelha. Fundir a quina num ponto (mesmo
  na distância certa) **evita** justamente o cruzamento que o resultado pro precisa. Esse é o sintoma 5.

### Tentativas que falharam — e por quê (as lições estão aqui)

| # | Tentativa | Por que pareceu certo | Por que falhou |
|---|---|---|---|
| 1 | Offsetar âncoras ao longo da **tangente** Bézier (não do chord) | A teoria do "chord dá distância desigual" era correta | **Nenhuma mudança visível**: o `offset_curve` já roda sobre pontos **densificados**, onde tangente≈chord. O fix estava num lugar que já era no-op. |
| 2 | **Polyline offset** (offset por segmento de reta + miter join) | Deu "distância correta" | Perdeu as âncoras Bézier/pontos visíveis (Enio rejeitou) **e** ainda artefatava nas quinas. |
| 3 | Restaurar **densificação CAD** com pontos visíveis | Resolveu "ver os múltiplos pontos" nas curvas | Não tocava nas quinas: a densificação refina **dentro** de spans suaves; a quina é uma **junção entre** segmentos. |
| 4 | **Miter** simétrico: `vertex_normal` devolve `(n₁+n₂)/(1+n₁·n₂)` (a interseção) com miter-limit | Corrigiu o undershoot (Camada A); zero regressão em suave/círculo | Ainda **um vértice único** → continuava pontudo, sem cruzar (Camada B intacta). |
| 5 | Miter **assimétrico** (convexa clampa, côncava alcança a interseção sem clamp) | Distâncias 100% corretas em todos os casos | Ainda **um vértice único** por quina → **não cruzava**. Um ponto não se auto-cruza, ponto final. |

**A lição-mãe:** "distância visualmente correta" **não** é prova de que o algoritmo está certo. As tentativas 4 e 5
acertavam a distância e ainda assim estavam erradas na **topologia** (sem cruzamento). Só o sintoma reformulado
pelo Enio — *"não se cruzam"* — revelou que o problema era de **estrutura de saída** (1 ponto vs. 2), não de
posição. Ver [feedback_measure_perf_symptom_scale](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_measure_perf_symptom_scale.md)
e [feedback_tool_unit_green_integration_dead](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_tool_unit_green_integration_dead.md).

### A solução final (offset-then-trim, padrão CAD)

A arquitetura **já estava pronta** para o cruzamento e ninguém tinha percebido: em
[`curve.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve.rs) o **Trim age só no *spine* pintado**
(`trim_offset_spine`), deixando as **âncoras livres pra cruzar** (comentário explícito: *"the anchors may
cross; the crossed loop just isn't painted"*). Faltava o `offset_curve` **produzir** o cruzamento.

Criei [`curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (irmão do `curve_trim`),
onde o novo `offset_curve` decide **por quina**:

- **Vértice suave** (`n_in ≈ n_out`, dot > `SMOOTH_DOT`): **1 âncora** no normal unitário → círculos / Auto /
  Vector ficam byte-idênticos (sem regressão).
- **Quina convexa** (lado de **fora** da curva — um gap): **1 âncora** no **miter** `(n₁+n₂)/(1+n₁·n₂)`,
  clampada a `MITER_LIMIT` (um espinho convexo não se auto-cruza, então o Trim não o limparia → tem que ser
  limitado).
- **Quina côncava** (lado de **dentro** — as arestas se sobrepõem): **DIVIDE em 2 âncoras** `P_in = V+d·n_in`
  e `P_out = V+d·n_out`. Na côncava elas caem em **lados opostos** do vértice → as duas arestas offsetadas
  **ultrapassam** → o `flatten_spine` gera um spine **auto-cruzado** → o **Trim corta a orelha**. O conector
  reto (handles colapsados) entre `P_in` e `P_out` é exatamente a orelha.

Convexo vs. côncavo é decidido pelo sinal do giro vezes o sinal de `d`:
`côncavo ⇔ (n₁×n₂)·d < 0` (com gate `dot < SMOOTH_DOT` pra um bend suave nunca fragmentar).

**Plumbing do `remap`.** Como uma quina côncava agora vira **2 âncoras**, a saída do `offset_curve` tem tamanho
variável. Ele devolve um `remap: Vec<usize>` (índice de saída → índice de entrada; o split mapeia 2 saídas →
1 entrada), e o [`offset_curve_refined`](../../crates/ph2d-tool-painter/src/tool/paint/curve_offset.rs) **compõe**
esse `remap` com o `origin` da densificação, pra o **bake** continuar carregando handle-kinds + seleção através
do split. O bake materializa o cruzamento na curva editável — o usuário **vê** os pontos cruzados.

**HR-5 (transcendental-free):** tudo é produto vetorial + a rotação complex-multiply do `SegXform`. Nada de
`atan2`/`sin`/`cos`. Não puxa kurbo (usa transcendentais; e há o gate `vello_kurbo_only_in_ph2d_vector`).

### O que NÃO era a causa (red herrings registrados)

- **Miter-join no convexo.** Necessário e correto, mas **insuficiente** — só corrige a distância (Camada A),
  não a topologia (Camada B).
- **Mais densidade de pontos na quina.** Não resolve: a quina é uma **junção**, não falta de amostras. Mais
  pontos só agrupa amostras perto do vértice (errado). Confirmado na literatura (Levien: subdivisão e junções
  são problemas **separados**).
- **Tipo de handle.** Free/Aligned "pioravam" só porque criavam a descontinuidade de tangente que disparava o
  undershoot; não era um bug do handle.

### Arquivos e commits (ordem cronológica da saga)

| Commit | O que fez |
|---|---|
| `3a3f6071` | (tentativa 1) offset ao longo da tangente — no-op por causa da densificação |
| `803a7c76` | (tentativa 2) polyline offset — revertido (perdia Bézier; artefatos) |
| `d9e6e5ab` | (tentativa 3) densificação CAD com pontos visíveis; sem simplificação automática |
| `c6e600ab` | (tentativa 4) miter simétrico corrige o undershoot da quina |
| `7d7d7a7d` | (tentativa 5) miter assimétrico: convexo clampa, côncavo alcança a interseção |
| `99f3aef0` | **solução** — `curve_join.rs`: côncava **divide em 2 âncoras** → spine cruza → Trim corta |

### Verificação

Testes em [`curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (`cargo test -p ph2d-tool-painter --lib curve_join`):
`a_concave_corner_splits_into_two_overshooting_anchors` (prova: 4 âncoras, `remap=[0,1,1,2]`, `P_in`/`P_out`
em lados opostos; convexo no mesmo canto = 3 âncoras), `a_convex_corner_stays_one_sharp_miter_anchor`,
`offsetting_a_circle_stays_concentric...` (suave nunca fragmenta), `the_convex_miter_reaches_the_true_distance_then_clamps`,
`a_smooth_vertex_miter_stays_unit`, `side_normals_follow_the_handle_tangent_not_the_chord`.
**Smoke do Enio (2026-06-29):** "Perfeito tanto para fora quanto para dentro! Tanto curvas como quinas! Curvas
abertas ou fechadas!"

### Lições generalizáveis

1. **Reformule o sintoma antes de iterar.** O salto de "quinas curtas" para "quinas não cruzam" mudou a classe
   do problema (posição → topologia). Cada round na pista errada custou um commit.
2. **"Parece certo" ≠ "está certo".** Distância visualmente correta escondeu um defeito topológico por 2 fixes.
3. **Junção ≠ amostragem.** Offset de traçado = problema de *stroking*: a parte suave é subdivisão; a quina é
   uma **junção** (miter/round/bevel ou split-and-trim). São mecanismos distintos.
4. **Cheque o que a arquitetura já permite.** O Trim-só-no-spine já deixava as âncoras cruzarem; a correção era
   *upstream* (produzir o cruzamento), não mexer no Trim/dispatch.
5. **Saída de tamanho variável precisa de `remap`.** Ao trocar 1↔N na saída de uma função no meio de um
   pipeline, propague um mapa índice→origem pra os consumidores a jusante (bake/seleção/kinds) não quebrarem.

---

## Bug #2 — Per-Layer Color: FPS despenca + artefatos retangulares ("retângulo virtual")

**Crates/arquivos:**
- **Perf:** [`ph2d-painter-brush/src/stamp_color/accumulate.rs`](../../crates/ph2d-painter-brush/src/stamp_color/accumulate.rs)
  (kernel **fundido** `accumulate_color_stamps_fused`), [`tool/paint/stamp_color_cache.rs`](../../crates/ph2d-tool-painter/src/tool/paint/stamp_color_cache.rs);
  coalescing de ponteiro no shell ([`input_dispatch/painter_canvas_input.rs`](../../shells/desktop/src/input_dispatch/painter_canvas_input.rs)
  + [`render_loop/mod.rs`](../../shells/desktop/src/render_loop/mod.rs)) + `StrokeMethod::coalesces_canvas_motion`.
- **Artefato:** [`ph2d-render/src/individual.rs`](../../crates/ph2d-render/src/individual.rs) —
  `clear_all_mips_transparent` em `create_entry_empty` (clear-on-alloc do slot de preview).

**Feature:** Per-Layer Color (camadas-como-pincel) — N camadas capturadas como Shape, cada uma com sua cor,
compostas em z-order e estampadas ao longo do traço.

Dois sintomas reportados **juntos**, com **causas-raiz diferentes** (essa foi a primeira armadilha):

1. **FPS despenca** ao desenhar (9 FPS) **e** o contador **"Raw" SOBE enquanto o FPS cai** (paradoxo).
2. **Artefatos retangulares:** fatias da imagem do brush **aparecem e somem** em "cantos de retângulos invisíveis".

### Problema A — Perf (estrutural, bem-comportado)

**Medição primeiro** (harness `per_layer_perf` em [`tool/paint/tests.rs`](../../crates/ph2d-tool-painter/src/tool/paint/tests.rs),
`--release`): split de fases revelou **um único kernel = 96.5%** do custo por-Move — `accumulate_color_stamp_coverage`,
`O(D·N·S)` (D dabs × N camadas × footprint (2r)²), refeito pra forma inteira a cada pointer-Move. **Refutou** a
teoria do handoff (que culpava bbox/recompose/upload — D/H≈1.0 provou que **não** é bbox-bound). O "Raw sobe" é a
assinatura: as estampas rodam **fora** da janela de encode que o Raw mede, então `frame_cpu_ms` (Raw) **cai**
enquanto o wall-clock total (FPS) **sobe**.

**Fix:** (1) **kernel fundido alpha-only** — todos os N stamps compartilham `size`, então as coords bilineares
são computadas **1×/pixel** (não ×N) e só o canal alpha é amostrado (o caminho descarta o RGB) → **3.2–4.5×**,
byte-idêntico (gate `fused_per_layer_accumulate_is_bit_identical_to_sequential`). (2) **Coalescing de ponteiro
por-frame** dos métodos de forma (Curve/Line/Circle/Polygon) — colapsa o storm de re-estampa por-evento bruto em
1/frame (incrementais resamplam o segmento, ficam de fora por design). **Limite aceito:** com pincel grande × N16
× canvas grande uma estampa única ainda é ~110 ms — o caso extremo fica para a **migração GPU** do accumulate
(decisão do Enio: sem mitigações CPU de spacing/pincel/camada).

### Problema B — Artefatos (a causa que enganou por ~5 rounds)

A **descrição do sintoma evoluiu** e cada reformulação reposicionou a causa:

1. *"Listras retangulares ao desenhar."* → suspeita: upload parcial de GPU (`preview_upload_bbox`).
2. *"Persiste com `PH2D_PAINT_FULL_UPLOAD=1`."* → upload parcial **descartado**; reclassifiquei como **tearing
   por perf** (§3-D) — **errado** (tearing seria persistente, não "primeiras vezes").
3. *"Fatias da forma, transientes, só nas primeiras vezes; depois nunca mais."* (mockup do Enio) → re-suspeita:
   base stale no cache `composited` CPU. Implementei `reseed_preview_base` (full recompose no início de cada
   sessão de forma). **Não resolveu.**
4. *"Existe um **retângulo virtual** onde o traço é feito; ele sofre o artefato só na **PRIMEIRA vez** que aquela
   região é desenhada na sprite; depois fica limpo pra sempre, mesmo redesenhando."* (a observação decisiva).

A observação 4 é a assinatura inequívoca de **leitura de memória GPU não-inicializada**: garbage até a região ser
escrita a 1ª vez; uma vez escrita, válida para sempre. E **imune ao FULL_UPLOAD e ao reseed** porque ambos mexem
em buffers **CPU já semeados** — e se o stack é GPU-elegível, o `gpu_owns_preview` **desliga o caminho CPU inteiro**.

### Causa-raiz (a verdadeira) + a saga do falso-negativo

A assinatura é **leitura de memória GPU não-inicializada** (retângulo virtual; garbage só na 1ª vez que a região é
desenhada; limpo pra sempre depois — e **não-determinístico**: memória não-inicializada às vezes calha de ser
transparente/preta, às vezes lixo visível). Trace exaustivo: **todos** os buffers semeados **EXCETO um** — o slot do
[`IndividualTextureStore`](../../crates/ph2d-render/src/individual.rs) (a textura que o sprite amostra via
`PreviewOverride`) era criado em `create_entry_empty` **sem clear** (texturas wgpu nascem com lixo). O caminho
GPU-preview adquire esse slot **vazio** (`acquire_empty`) e o preenche por **cópia de região** depois → uma região
amostrada antes da 1ª cópia lê garbage. Retângulo = a região; primeira-vez = antes do 1º write; limpo-pra-sempre = a
textura persiste escrita.

**O falso-negativo que custou 3 rounds.** O clear-on-alloc do slot foi a 1ª hipótese certa — mas o teste do Enio logo
após disse *"alarme falso, ainda existe"*, o que me fez **descartar** a hipótese e caçar `out`/premul (que verifiquei
limpos) e reprodução runtime. **Era um binário stale**: o `play.command` daquele momento rodou um build **sem o clear
compilado** (ou pegou um cache), então o artefato (não-determinístico) ainda aparecia. Num **rebuild limpo** (`play.command`
sem env, depois do ship), o clear-on-alloc está ativo e o artefato **não voltou em vários testes**.

### Tentativas / a ordem real (incluindo o falso-negativo)

| # | Passo | Resultado |
|---|---|---|
| 1 | `PH2D_PAINT_FULL_UPLOAD` (upload full do slot CPU) | Persistiu → não é cobertura do upload; e no stack GPU-elegível o `gpu_owns_preview` desliga o caminho CPU. |
| 2 | Reclassificar como **tearing por perf** (§3-D) | Errado — "primeiras vezes, depois nunca" contradiz tearing (seria persistente). |
| 3 | `reseed_preview_base` (full recompose por sessão de forma) | Re-semeia o `composited` **CPU**; defensivo correto, mas não era o buffer (GPU). |
| 4 | **Clear-on-alloc do slot** (`clear_all_mips_transparent`) — **O FIX** | Falso-negativo (binário stale) me fez achar que falhou → descartei. |
| 5 | Verificar `out`/premul (shaders) | Limpos (escrevem todo texel) — não eram a fonte. Confirmou que o pipeline todo estava semeado **menos o slot do passo 4**. |
| 6 | Rebuild limpo + re-teste (Enio) | **Artefato resolvido.** O passo 4 era o fix o tempo todo. |

### A solução final (clear-on-alloc)

`clear_all_mips_transparent` ([`texture_clear.rs`](../../crates/ph2d-render/src/texture_clear.rs), chamado em
`individual.rs::create_entry_empty`): render-pass `LoadOp::Clear(TRANSPARENT)` sobre **todos** os níveis de mip (o
sampler trilinear lê qualquer nível e `regen_mips` só roda após o 1º upload — então cada nível precisa nascer limpo,
não só o 0). Custo: uma vez na alocação do slot. Agora qualquer amostragem-antes-do-write mostra **transparente** (e
deterministicamente), não lixo.

### O que NÃO era a causa (red herrings registrados)

- **Upload parcial de GPU / `preview_upload_bbox`** (§3-A): cobertura provada consistente; FULL_UPLOAD descartou.
- **Tearing por perf** (§3-D): contradito pelo "primeiras vezes, depois nunca".
- **Cache `composited` CPU stale / drag-preview restore:** auto-consistentes (trail-freedom verdes).
- **`out`/premul (compositor GPU):** `cs_flat` parte de `acc=vec4(0)` e escreve todo texel; `cs_main` (premul) idem,
  canvas inteiro. Ambos totalmente escritos cada frame — não eram a fonte.

### Verificação

- **Perf (✅):** harness `per_layer_perf` (`#[ignore]`, `--release`) + gate de paridade byte
  `fused_per_layer_accumulate_is_bit_identical_to_sequential`; **3.2–4.5×**.
- **Artefato (✅):** guard `acquire_empty_slot_reads_back_transparent_not_garbage` (slot vazio lê all-zero, antes
  garbage); 6/6 `individual_readback` verdes.
- **Smoke do Enio (2026-06-29):** "Testei várias vezes, o bug/artefato não voltou a aparecer" (`play.command`, rebuild limpo).

### Lições generalizáveis

1. **Verifique um REBUILD LIMPO antes de declarar um fix morto.** O falso-negativo ("ainda existe") foi um binário
   stale — eu **descartei o fix certo** e gastei 3 rounds caçando o buffer errado. Bug não-determinístico + build
   incremental = "ainda aparece" pode ser só o binário antigo. Force o rebuild (toque o crate / `--release` limpo)
   antes de abandonar a hipótese.
2. **"Não mudou" não autoriza reclassificar a causa** — só prova que **aquele** buffer/build estava ok. Vale dobrado
   quando o sintoma é não-determinístico (memória não-inicializada).
3. **Texturas wgpu nascem com lixo.** Toda textura amostrável-antes-do-1º-write-completo precisa de clear-on-alloc;
   limpe **todos** os níveis de mip (o `regen_mips` só roda depois do 1º upload).
4. **"Primeira vez, depois nunca" = leitura não-inicializada.** Escrito-uma-vez-fica-válido aponta direto pra um
   buffer sem clear-on-alloc (foi a pista que cravou o slot).
5. **Meça antes de culpar (perf).** O split de fases (96.5% num kernel) refutou a teoria do handoff em uma medição.
   Ver [feedback_measure_perf_symptom_scale](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_measure_perf_symptom_scale.md).

---

## Como adicionar um bug aqui

Uma seção `## Bug #N — <título>` + linha na tabela do topo. Foque nos bugs cuja **causa enganou** (vários rounds
na pista errada); fix trivial fica só no git. Sempre termine em **lições generalizáveis**.
