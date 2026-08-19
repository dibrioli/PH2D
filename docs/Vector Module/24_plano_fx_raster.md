# Plano — FX raster de alta qualidade para formas vetoriais

> `line/Vector`, 2026-07-25. O pedido do Enio: **"efeitos FX de alta qualidade … existe algo
> melhor do que o Rive e compatível com o que temos? Buscamos o estado da arte."** A pesquisa
> respondeu (§1); este é o desenho e a divisão em waves. **Autorizado a seguir ("faça como achar
> melhor"); mesmo assim o plano vem primeiro, é a lei da casa.**

## §1 — Pesquisa (o estado da arte, e o veredito)

**O Rive é a fronteira num eixo ESTREITO e fraco em largura.** Fatos verificados na fonte:

- **Licença MIT pura** (`rive-app/rive-runtime/LICENSE`) — sem não-competição. Podemos referenciar
  E reimplementar; o ADR-0108 já manda reimplementar nativo, não vendorizar.
- **O FX central do Rive é UM: o Vector Feathering.** A opacidade da borda macia é a **integral
  analítica da normal (erf)**, avaliada por vértice via LUT gaussiana (`@featherTexture`,
  `FEATHER_COVERAGE_BIAS`) + a cobertura por *winding* da tesselação. Documentado no blog técnico e
  no gist do CTO; shaders em `renderer/src/shaders`. Drop shadow / glow / blur de vetor no Rive
  **derivam do feather** (um preenchimento deslocado e feathered) — não são passes separados.
- **Duas pegadinhas decisivas:** (a) o algoritmo é **acoplado ao renderer de tesselação do Rive**
  (`@featherTexture` + cobertura por triângulo). Nós renderizamos por **Vello/kurbo** — herdamos a
  *matemática* (soft edge = erf da distância), não os shaders. (b) O menu de FX do Rive é
  **estreito**: feather + blend, e nada mais. Sem displacement, sem noise procedural, sem
  color-matrix, sem lighting/bevel.
- **O que É melhor que o Rive e compatível:** o **grafo de filtros componível** (o modelo
  SVG-filter — `feGaussianBlur`/`feDropShadow`/`feColorMatrix`/`feDisplacementMap`/`feTurbulence`/
  `feMorphology`/`feDiffuseLighting`), que é a largura que o Rive não tem, componível como a nossa
  pilha de LPE, e **a direção ATIVA do próprio Vello** ([tmil-23], nov/2025: Gaussian Blur, Drop
  Shadow, Flood + "initial filter effects" no `vello_cpu`). ⚠️ Esses filtros do Vello estão no
  backend `vello_cpu`, **não no `vello` 0.8 GPU que pinamos** — herdá-los direto = migração de
  backend (grande). O caminho compatível HOJE é **implementar o passe raster nós mesmos**, e já
  temos a infra: o `ph2d-render` tem blur gaussiano separável, bloom/glow, chroma e S-H
  texture→texture (o compositor do Painter).

**Veredito:** o play de estado-da-arte que bate o Rive é o **híbrido** — grafo de FX raster
componível (largura), com a **sombra/glow/blur analítico** de qualidade-Rive como o corte inicial
(onde igualar o Rive importa). É melhor por construção (superset) e maximamente compatível (roda no
nosso renderer + reusa o compositor que o Painter já tem).

## §2 — O desenho (portas únicas)

**A costura arquitetural nova, e o inegociável:** um FX raster produz **PIXELS**, não um `VecPath`,
então **NÃO é um `PathEffect`** da pilha (ADR-0132) — o `effect::run_stack` é `VecPath -> VecPath`,
puro, dentro da `ph2d-vec-scene` (sem kurbo, sem GPU). Isto é a MESMA lei que o doc-comment do
`VecOffset` já grava para a geometria: o `cooked()` responde *"o que este documento desenha?"* e não
pode depender de um instalador de runtime GPU. Logo o FX raster é um **post-pass orquestrado no
SHELL** (onde o GPU + `VelloPass` + `LayerCompositor` estão em escopo), e o atributo é um
**componente ECS por-forma** — o precedente exato de `VecOffset`/`VecTextPath`/`VecEnvelope`.

O pipeline de render é UMA `vello::Scene` compartilhada → UMA textura intermediária → compositor.
Para filtrar UMA forma é preciso **isolá-la na própria textura primeiro**, e isso **não pode** viver
no `ph2d_vec_render::dispatch` (função de encode pura, sem handle de GPU). As portas únicas:

| Pergunta | Porta única |
|---|---|
| *Esta forma tem filtro?* | componente ECS **`VecFilter`** (`ph2d-ecs`), lido pelo shell na MESMA fase que monta o `LiveGeometry` |
| *Onde a forma isolada é rasterizada?* | `VelloPass::render_to_intermediate` (já existe) numa textura-scratch; o `dispatch` **PULA** as formas filtradas (deixa o buraco) |
| *Como o filtro roda?* | `LayerCompositor` via `inject_slice_from_texture` + um op `SpatialAdjustment` (blur/glow existem) — os passes exóticos novos clonam o shape `PreviewPremul::run(gpu, src) -> &Texture` |
| *Como volta ao z?* | `VectorScene::draw_image_rgba_premultiplied_transformed` (já existe) — re-injeta a textura filtrada no z da forma, e flui pelo `VelloPass`→`Compositor` normal |

⚠️ **Uma pergunta, uma porta:** a mesma `render_to_intermediate` que o present usa, o mesmo
`LayerCompositor` que o Painter usa, o mesmo `draw_image` que os overlays usam. Nenhum passe de GPU
novo para blur/glow/shadow — só orquestração. Passe novo só para color-matrix/displacement/noise
(Wave 2), e o template é o `PreviewPremul`/`ImpastoLightPass`.

**Sombra e glow a partir do blur que existe:** Drop Shadow = isola a forma → borra o alfa → desloca
→ tinge com a cor da sombra → compõe **ABAIXO** do original. Outer Glow = o mesmo sem deslocamento,
cor de brilho, compõe abaixo (ou `screen`). Blur = a forma borrada no próprio lugar. Os três são
"borra a forma isolada e recompõe" — a razão de serem UMA wave.

## §3 — Contrato congelado (§6) e schema — a prova por grep

- **NÃO toca o §6.** O contrato congelado do vetor é `ph2d-vector-doc`+`-traits`
  (`VectorOp`/`Vertex`/`Segment`/… — gate `architecture_vector_contract_surface`). O `VecFilter`
  mora em `ph2d-ecs` (componente), e a orquestração no shell + `ph2d-render`. Nenhum toca o doc
  congelado. (Grep de fechamento: `architecture_vector_contract_surface` verde intacto.)
- **NÃO é `PathEffect`** ⇒ `MAX_FX_KINDS`/`VEC_SCENE_SCHEMA_VERSION` (13) **intactos** (o §2 explica
  por quê — raster não é `VecPath -> VecPath`).
- **ZERO bump de `PROJECT_SCHEMA` (29).** Componente ECS novo cunha `stable_type_id` próprio
  (`blake3(NOME)[..8]`), não move layout posicional de nada — o precedente exato do `PhysicsJoint`/
  `VecTextPath` (registro do `ph2d-ecs` sobe; ⚠️ **o contador é TRÊS** — os espelhos em
  `ph2d-render` e `ph2d-script` sobem junto, a família vermelho-latente das integrações de 21 e 23
  de julho).

## §4 — A UI (as 4 condições independentes)

1. **O componente EXISTE:** `VecFilter` em `ph2d-ecs`, registrado no `ComponentRegistry` (gate
   `registers_every_component`, o contador ×3).
2. **Pintado e registrado:** seção **"Effects (FX)"** nova no `ph2d-panel-vector` (`VECTOR_SECTIONS`
   +1 ao FIM), com o seletor de tipo (None/Blur/Glow/Drop Shadow) + os params do tipo (radius,
   offset X/Y, color, opacity) — cada widget com id registrado (`node_id_collisions`).
3. **O clique chega ao barramento:** o `vector_bridge` drena os edits e escreve/remove o `VecFilter`
   na entidade selecionada (padrão `VecOffset`), com seam que CLICA cada row.
4. **A sequência leva a algum lugar:** editar um param → o shell orquestra o post-pass → o pixel
   muda na tela (arch-gate de shell — a orquestração exige janela/GPU, então é gate sobre a fonte +
   um gate e2e headless com wgpu real, o padrão do `the_gpu_producer_shows_what_the_cpu_producer_shows`).

## §5 — Gates (red-first, a fixture contém o fenômeno)

- **A forma filtrada é rasterizada ISOLADA** (arch-gate: o `dispatch` pula a forma com `VecFilter`;
  mutação = não pular → a forma aparece DUAS vezes, nítida e filtrada).
- **O blur borra** (e2e headless com wgpu: uma forma dura vira uma rampa; mutação radius→0 = borda
  dura de volta, RED). O oráculo é a **largura da rampa medida nos pixels**, não a regra.
- **A sombra fica ABAIXO e DESLOCADA** (a cor da sombra aparece no offset, atrás do original;
  mutação = compor acima → a sombra come a forma, RED).
- **`VecFilter` ausente é byte-idêntico ao mundo de hoje** (o caminho comum não paga nada — a forma
  sem componente flui pelo `dispatch` intacta).
- **Painel:** presença E ausência das rows por tipo (Blur não mostra offset; Drop Shadow mostra) +
  seam que clica cada uma.
- **Contrato:** `architecture_vector_contract_surface` + `VEC_SCENE_SCHEMA_VERSION==13` +
  `PROJECT_SCHEMA==29` intactos.

## §6 — Smoke (números MEDIDOS pela sonda headless antes da mensagem)

`PH2D_BUILD_SMOKE=3X` — três estrelas idênticas: **nítida** | **drop shadow** (offset 8px, blur 6px,
preto 60%) | **outer glow** (blur 12px, ciano). A sonda headless mede a largura da rampa de alfa e a
posição do centroide da sombra, e a mensagem traz os números.

## Waves

- **W1 — A FORMA FILTRADA (FECHADA, smoke aprovado 2026-07-26):** a orquestração
  isolar→filtrar→recompor + `VecFilter` + **Blur · Outer Glow · Drop Shadow** + painel + smoke. É
  o Rive-headline e estabelece a costura que todo filtro futuro pluga. ⚠️ Reescrita **GPU-resident**
  a meio caminho (o 1º corte era CPU-first e vazava o atlas do Vello — 37→793 ms num smoke parado);
  e o `override_image` foi trocado por RE-REGISTRO no resize (dims estáveis) depois do "panic ao
  zoom".
- **W2 — A PILHA COMPONÍVEL (FECHADA, smoke aprovado 2026-07-26):** ver §7 abaixo.
- **W3 — O CATÁLOGO (FECHADA, smoke aprovado 2026-07-26):** os degraus de DENTRO, o CONTORNO e a COR —
  `Inner Shadow` · `Inner Glow` · `Outline` · `Color Overlay`, três tipos → **sete**. Ver §8.
- **W4 — O CAMPO DE DISTÂNCIA (FECHADA, pendente de smoke):** a revisão que as três observações do
  smoke do Enio pediram — o rim de 1 px, o modo `Contour` dos degraus de dentro e o contorno que
  deixou de encolher na quina. Ver §9.
- **W5 — O FEATHER E O BEVEL (FECHADA, smoke reprovado → §11):** os dois tipos que o campo de distância
  da W4 destravou (7 → 9). Ver §10. ⚠️ O feather chegou pelo caminho oposto ao previsto: em vez do
  `erf` analítico de Levien, ele é uma rampa sobre o MESMO campo do JFA — o primitivo já estava
  construído por outro motivo, e a wave inteira coube num braço de shader.
- **W5b — OS ARTEFATOS DO CAMPO (FECHADA, pendente de smoke):** o pente do feather/bevel, a serrilha
  do contorno e a ponta ceifada pelo traço. Ver §11.
- **W6a — A LEI DE MISTURA POR DEGRAU (FECHADA, pendente de smoke):** o blend mode do Layer Style,
  em **quatro** dos nove tipos. Ver §12.
- **W6b — A TURBULÊNCIA (FECHADA, smoke aprovado 2026-07-28):** o `feTurbulence` +
  `feDisplacementMap` num degrau só — o eixo ORGÂNICO. **Não mudou a pilha**, como previsto. §13.
- **W7 — GROW / SHRINK (FECHADA, pendente de smoke):** o `feMorphology` (dilate/erode) num knob
  **com sinal** — 10 → **onze** tipos. Nenhum kernel novo: é um LIMIAR no campo de distância do W4.
  Ver §14.
- **W8 — COLOR ADJUST (FECHADA, smoke aprovado):** o `feColorMatrix` — e a lei **não era nova**, a
  wave é uma EXTRAÇÃO. Ver §15.
- **W9 — DUOTONE + LUMA TO ALPHA (FECHADA, smoke aprovado 2026-07-29):** as duas leis que leem o
  BRILHO da arte. Ver §16.
- **W10 — O ATLAS DE RASTER (FECHADA, pendente de smoke):** a primeira wave que não acrescenta um
  tipo — ela responde *"quanto custa uma CENA de formas filtradas?"*, que é o eixo que o Enio
  nomeou. Ver §17.
- **W11 — GRADIENT MAP (FECHADA, pendente de smoke):** a rampa de **N stops** — e ela **SUBSUME o
  Duotone** (dois stops nas pontas são ele **ao byte**), então a wave é uma generalização em vez de
  um 12º tipo que responde à mesma pergunta. Ver §18.

---

# O QUE SEGUE ABERTO, E O QUE FOI MEDIDO E REVERTIDO

> ⚠️ **Recorte de 2026-08-18.** O corpo das waves **W2 a W11** (§7-§18 — *o que se construiu, e
> por quê*) foi movido **verbatim** para
> [`docs/archive/docs-2026-08-18/Vector Module/24_plano_fx_raster.md`](../archive/docs-2026-08-18/Vector%20Module/24_plano_fx_raster.md).
> Ficou aqui a pesquisa e o desenho (§1-§6), a lista de waves com o que cada uma entregou, **os
> `Aberto` de cada wave**, as **⚠️ leis** que o campo de distância comprou, a **⛔ reversão medida
> do §17.6** e o **custo nomeado do §18.7**. ⛔ Nada foi resumido — as duas metades remontam o
> original byte-a-byte (sha256).

## §7 (W2 — a PILHA) — Aberto

- **Só três tipos.** O `apply_op` é a porta por onde um tipo novo entra com um braço de shader e um
  `kind_name`: color-matrix (tint/duotone), morphology (dilate/erode), displacement + turbulence,
  bevel. Nenhum deles muda a pilha — é isso que a wave comprou.
- **`Radius` é slider em unidades de MUNDO** (`FILTER_RADIUS_MAX = 2.0`) — fração-do-tamanho seria
  mais robusto para formas de tamanhos diferentes (a mesma nota que o Contour faz do Offset).
- **O deslocamento da sombra é arredondado ao PIXEL** (o halo é amostrado por `textureLoad`, sem
  sampler). Invisível numa sombra; nomeado por honestidade.
- **`MAX_HALF = 96`** no kernel (sigma ≈ 32 px de tela): acima, o borrão satura — limite de CUSTO
  do passe, não de produto.

## §8 (W3 — o CATÁLOGO) — Aberto

- **`MAX_HALF = 96`** continua o teto do kernel; para o Outline ele limita a largura a ~32 px de
  tela, o que é muito mais do que um contorno pede.
- **O Outline arredonda quinas convexas** — é um corte no nível de uma Gaussiana isotrópica, logo
  aproxima a dilatação por DISCO (o que um pincel redondo faz). Uma dilatação exata seria `O(r²)`
  por texel, ou um EDT; não se justifica sem um pedido.
- **Sem blend mode por degrau** (o Layer Style do Photoshop tem um por efeito) — é W5, e não mexe na
  pilha: é um campo a mais no `FxOp` e um `mix` a mais no finalize.

## §9 (W4 — o CAMPO DE DISTÂNCIA) — Aberto

- **Miter/bevel no contorno**: geometria, não raster (ver a derivação acima). O caminho é um
  *Stroke* na pilha de Effects, onde `VecOffset { join }` já vive.
- **O modo `Contour` custa `2 + bits(w)` passes** contra 2 do borrão — 6 para uma banda de 16 px.
  Barato (o JFA lê 9 texels por passe), mas é o tipo mais caro do catálogo.

## §10 (W5 — FEATHER e BEVEL) — Aberto

- **`MAX_FILTER_KINDS` foi de 7 para 9**, e o gate que o pega é o seam que CLICA cada "Add": o
  `.take(MAX_FILTER_KINDS)` do paint deixaria os dois últimos tipos sem botão em silêncio.
- **O bevel não tem "size" separado da profundidade** (o `Depth` governa os dois). O Photoshop tem
  Size + Soften; se o smoke pedir, é um knob a mais, não um modelo a mais.

## §12 (W6a — a LEI DE MISTURA) — Aberto

- **A lei de um halo EXTERNO contra a CENA** (o Drop Shadow em Multiply do Photoshop) exigiria que
  a textura de saída do FX carregasse uma lei para o composite da cena — outra camada, outro dono.
  Nomeado, não construído.
- **O Bevel tem UMA lei para as duas faces.** O Photoshop tem duas (Highlight: Screen · Shadow:
  Multiply). Uma só já é coerente (Multiply mata o realce e mantém a sombra; Screen o inverso), e o
  par é refino de produto.

## §14 (W7 — GROW/SHRINK) — as duas leis que a wave comprou

### ⚠️ Ela mede a IMAGEM, não a FORMA

As outras quatro do campo (contorno, feather, bevel, os de dentro em Contour) são efeitos de **borda
da SILHUETA** e querem o pé exato da geometria. O `feMorphology` dilata **a entrada dele** — e é isso
que faz `Outline → Grow` **engordar o traço** em vez de o recortar de volta à silhueta.

Com geometria disponível o produtor resolveria pela forma: a resposta certa para quatro tipos e a
errada para o quinto, **sem erro nenhum**. Daí a porta única, e daí o `n_segs` do uniform ter passado
a ser derivado do PLANO — semear o raster e deixar o finalize consultar a geometria construiria um
campo que ninguém lê.

⚠️ **A fixture do gate TEM de trazer geometria.** Sem segmentos o produtor já semeia pela cobertura,
o defeito não existe e o gate ficaria verde sobre nada.

### ⚠️ `seeds_shell()` ENUMERAVA os leitores, e apodreceu na primeira adição

Ela dizia `FEATHER || OUTLINE || BEVEL || GLOW`. A morfologia lê o campo dos **dois** lados (crescer
olha para fora, encolher para dentro) e nasceu a cair no `else` — o ramo que semeia só os texels de
FORA, a medida que os degraus de dentro pedem. O campo saía definido de um lado só, e **quatro gates
ficaram vermelhos de uma vez**.

A pergunta certa já estava escrita uma linha acima: **os de DENTRO são a exceção**, todo o resto quer
os dois lados. Hoje é `!is_inner()`, equivalente ao byte no dia da troca, e o próximo tipo do campo
nasce certo.
## §14 (W7 — GROW/SHRINK) — Aberto

- **Falta o `feColorMatrix`** (tint/duotone/saturate/`luminanceToAlpha`) — o último item da lista do
  §7. O Color Overlay com as vinte leis de mistura já cobre a maior parte do que ele entrega.
- **Os joins do offset**: o Illustrator oferece miter/round/bevel no *Offset Path*. Aqui a quina é
  sempre redonda, porque a régua é a distância; um miter exigiria geometria, e a pilha de LPE
  (`VecOffset { join }`) já tem essa resposta no eixo certo.

## §17 (W10 — o ATLAS DE RASTER)

### §17.5 — Estado, e o que fica aberto com o número

`Globals` 128 → **144 bytes** (pin atualizado). `PROJECT_SCHEMA` **fica em 38**, contrato congelado
intacto, **zero `Cargo.toml`**, zero crate nova, zero ADR.

### §17.6 — ⛔ O SEGUNDO EIXO foi CONSTRUÍDO e REVERTIDO pela medição — não refaça

A §17.1 mediu **~0,03 ms de fixo por corrida da pilha** e a §17.5 anunciou, com esse número, que um
encoder/`submit` para as `n` formas valeria **~1,0 ms numa cena de 32**. **Está errado, e a wave que
o testaria já foi escrita inteira** (`FxStackItem` + `run_batch` + `seg_org` nos globals + o lote na
shell, com gate de paridade byte a byte e 3 mutações a sangrar). Medido pelas duas rotas, o mesmo
frame:

| N formas | `n` submissões | **1 submissão** | "ganho" |
|---|---|---|---|
| 4 | 0,58 ms | 0,47 ms | 1,24× |
| 16 | 1,25–1,28 | 1,33–1,36 | **0,92–0,95×** |
| 32 | 2,31–2,41 | 2,80–3,06 | **0,75–0,86×** |

**A submissão única é MAIS LENTA, e pior quanto maior o lote** (três corridas, reprodutível).

⚠️ **Por que o número da §17.1 enganou, e a lição é sobre a SONDA:** ele saiu de uma medição com o
degrau **mais barato que existe** — escolhido de propósito para isolar o fixo —, onde não há mais
nada a acontecer, então encode+submissão *são* a amostra. Numa pilha real o fixo **sobrepõe-se a
trabalho de GPU** e deixa de ser aditivo. *Um custo fixo medido em isolamento não se soma ao custo
de um sistema que o esconde.*

⚠️ **E o mecanismo do prejuízo é do próprio desenho:** as work textures são partilhadas, então num
encoder só o wgpu insere barreira entre **cada par de formas** — o lote **serializa** o que `n`
submissões deixam o driver pipelinar. Dar work textures próprias a cada item removeria a barreira ao
preço de `n×` a VRAM, e isso é uma wave diferente sobre uma hipótese que **esta medição não
estabelece** (ninguém mostrou que a GPU está ociosa).

**O que sobreviveu:** nada de código — o experimento foi revertido inteiro para não deixar uma porta
pública (`run_batch`) que ninguém deve chamar. O que fica é este número.

**Smoke:** `PH2D_BUILD_SMOKE=33` (as 16 estrelas filtradas) com **`PH2D_FX_PERF=1`** — a linha tem de
dizer **`em 1 render(es)`**, e o desenho tem de ficar **igual ao de antes**.

---

## §18 (W11 — o GRADIENT MAP)

### §18.7 — O que NÃO foi extraído, e o custo nomeado

O trilho do Painter (`paint_adjust.rs::paint_gradient_map`) é um editor completo e **não** foi
extraído para um widget compartilhado. **O que É compartilhado é o GESTO** —
`InteractiveState::CurvePoint`, o dispatch 2D que o editor de falloff do Painter e a curva do
motion-params já usam —, e esse é o precedente deste repo: reusar o primitivo de arrasto e pintar
localmente. Extrair o **pintor** exigiria parametrizar dois `PaintCtx` diferentes, dois esquemas de
id e duas fontes de *"qual stop está em foco"*.

**O custo de não extrair:** as constantes de geometria do trilho (barra, punho, caixa de agarre,
largura do botão) existem em **duas** cópias, hoje batendo valor a valor, cada uma citando a outra.
É a mesma dívida que o editor de curva do motion-params já carrega. **Quem quiser fechá-la:** o
alvo é um `paint_ramp_widget` no design system tomando um snapshot (`&[([u8;4], f32)]` + o índice em
foco) e um provedor de ids, com os dois painéis a delegar.
