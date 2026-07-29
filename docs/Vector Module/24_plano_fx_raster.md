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

## §7 — W2: a PILHA (o que se construiu, e por quê)

**A pesquisa matou o DAG.** O `<filter>` do SVG é um grafo de primitivas (`feGaussianBlur`/
`feOffset`/`feComposite`/`feMerge`/`feDisplacementMap`…) — poderosíssimo, e **abandonado como
interface** por todo mundo que o tentou: Photoshop (Layer Styles), After Effects (effect stack) e
Figma (Effects) convergiram numa **lista ordenada** de efeitos por objeto. O DAG sobrevive no
*runtime* (o arquivo SVG), nunca na mão do artista. E nós já tínhamos a resposta em casa: a pilha
de Live Path Effects (ADR-0132) é exatamente isto no eixo da GEOMETRIA.

**O Rive não tem pilha nenhuma** (feather + blend, com sombra e brilho DERIVADOS do feather). Poder
encadear *sombra → borrão → brilho*, nessa ordem, com o resultado de um alimentando o seguinte, é o
que esta wave entrega e ele não.

### O invariante, e a consequência que ele força

**Todo op é imagem → imagem, premultiplicada, do MESMO tamanho.** É a mesma frase que governa a
pilha de geometria (*"um efeito é `VecPath -> VecPath`, puro — é POR ISSO que a pilha compõe"*), e
a consequência não é cosmética: **Glow e Drop Shadow compõem o halo POR BAIXO da entrada DENTRO do
próprio op**, em vez de pedirem ao compositor que desenhe algo atrás da forma. Um op que devolvesse
*duas* camadas não poderia alimentar o seguinte ⇒ o `FxMode::Below`/`Replace` da W1 **MORREU**, e o
`dispatch` ficou mais simples do que era.

⚠️ **Mudança de comportamento nomeada:** uma forma com Glow/Drop Shadow agora é desenhada
INTEIRAMENTE a partir da textura (a W1 desenhava o vetor por cima). O scratch rasteriza na escala
EXATA da tela e o retângulo é alinhado ao pixel, então é 1:1 — mas é o olho do smoke que decide.

### As portas únicas

| Pergunta | Porta |
|---|---|
| *quanto a pilha espalha?* | `ph2d_render::stack_reach` (as reaches SOMAM; assimétrica p/ a sombra) |
| *que meia-largura o shader percorre?* | `kernel_half` — o `stack_reach` e o shader perguntam à MESMA |
| *mundo → pixel* | `fx_live::resolve_ops` (a câmera é conhecida ali e em mais lado nenhum) |
| *este id do painel é de quê?* | `fx_live::hit_of` — os TRÊS sítios da ponte (comando · valor · alvo do picker) |
| *esta pilha desenha alguma coisa?* | `VecFilter::is_active` |

### Os intermediários são `Rgba16Float`, e isso não é luxo

Entre ops a imagem é premultiplicada. Guardá-la em `Rgba8Unorm` e des-premultiplicar depois
**quantiza justamente a borda macia** que o borrão existe para produzir (alfa baixo ⇒ a divisão
amplifica o erro). `rgba16float` é formato de storage do baseline do WebGPU ⇒ nem uma feature custa.

### O teto, MEDIDO

`VecFilter::MAX_OPS = 6`, e **o recurso que aperta é a TELA do painel, não a GPU** — medido na RTX
(`fx_stack_gpu::the_cost_of_a_stack_is_linear_in_the_number_of_ops`, 512×512, sigma 8 px):

| degraus | 0 | 1 | 2 | 3 | 4 | 6 |
|---|---|---|---|---|---|---|
| ms | 0,082 | 0,084 | 0,149 | 0,220 | 0,336 | **0,429** |

Linear, ~0,07 ms por degrau; uma pilha CHEIA custa **2,6 % de um frame de 60 fps**. Cada degrau, em
compensação, é um card de 4-6 linhas no painel — seis já enchem a coluna.

### Gates (e as duas lições)

8 de GPU + 5 no shell + 4 no modelo + 6 de seam + 3 de dispatch. O gate que carrega a wave é
`the_order_of_the_stack_changes_the_picture`.

⚠️ **A 1ª mutação estava ERRADA e sobreviveu:** *"todo op vira o primeiro"* não reproduz *"a ordem
é ignorada"* — produz outra coisa errada (`[glow,blur]` vira glow-glow e `[blur,glow]` vira
blur-blur, que **continuam diferentes**). A mutação honesta é **ORDENAR a pilha**: aí os dois lados
ficam idênticos, `0 bytes diferentes`, RED.

⚠️ **O seam nasceu VERMELHO e apontou um erro real:** a swatch de cor tinha sido registada como
`button()`, e **um id só pode ter UM tipo de widget no store** — o Down abria o picker e nenhum
`Click` saía. É a mesma lição que o `vector_fx_toggle_id` já documentava. Ela tem gate próprio
agora (pintada + no conjunto de picker).

⚠️ **O `node_id_collisions` não cobria nem `vector_fx_*` nem a família nova**, que partilham o
prefixo `vector.f…` — "os nomes são diferentes" era uma afirmação por provar exatamente onde é
duvidosa. As duas entraram no MESMO conjunto.

### Aberto

- **Só três tipos.** O `apply_op` é a porta por onde um tipo novo entra com um braço de shader e um
  `kind_name`: color-matrix (tint/duotone), morphology (dilate/erode), displacement + turbulence,
  bevel. Nenhum deles muda a pilha — é isso que a wave comprou.
- **`Radius` é slider em unidades de MUNDO** (`FILTER_RADIUS_MAX = 2.0`) — fração-do-tamanho seria
  mais robusto para formas de tamanhos diferentes (a mesma nota que o Contour faz do Offset).
- **O deslocamento da sombra é arredondado ao PIXEL** (o halo é amostrado por `textureLoad`, sem
  sampler). Invisível numa sombra; nomeado por honestidade.
- **`MAX_HALF = 96`** no kernel (sigma ≈ 32 px de tela): acima, o borrão satura — limite de CUSTO
  do passe, não de produto.

## §8 — W3: o CATÁLOGO (o que se construiu, e por quê)

**A W2 comprou um multiplicador e esta wave o gasta.** O `apply_op` é a porta por onde um tipo novo
entra com um braço de shader e um nome; nada aqui mexe na pilha, na margem-como-conceito, no memo,
no registro de textura ou na costura de render. Três tipos viraram **sete**:

| # | tipo | o que desenha | raio | offset | cor | cresce | passes |
|---|---|---|---|---|---|---|---|
| 0 | Blur | borra o que chegou | Radius | — | — | 3σ | 2 |
| 1 | Glow | halo tingido POR BAIXO | Radius | — | ✓ | 3σ | 2 |
| 2 | Drop Shadow | o Glow deslocado | Radius | ✓ | ✓ | 3σ + offset | 2 |
| 3 | **Inner Shadow** | a sombra que cai **para dentro** | Radius | ✓ | ✓ | **0** | 2 |
| 4 | **Inner Glow** | a Inner Shadow sem deslocamento | Radius | — | ✓ | **0** | 2 |
| 5 | **Outline** | contorno de borda **DURA** | **Width** | — | ✓ | **σ+1** | 2 |
| 6 | **Color Overlay** | repinta sem borrar | — | — | ✓ | **0** | **1** |

### As três decisões que decidem o resto

**(a) Fora da textura é TRANSPARENTE, não `clamp`.** A W2 grampeava a coordenada, o que *estica* o
texel da borda para dentro do kernel. Trocar por *fora não há imagem* é a extensão correta de uma
imagem premultiplicada sobre campo transparente — e é ela que dá **margem zero** aos degraus de
dentro: para eles, "fora da textura" É "fora da forma", que é exactamente o que o alfa invertido
precisa de ler. Nos degraus de fora as duas respostas coincidem (a margem garante borda
transparente), então **nada do que a W1/W2 desenhava se move**.

⚠️ **Um gate da W2 ficou vermelho com isso, e a culpa era da FIXTURE:** ela punha uma barra opaca
ocupando a altura inteira de uma textura de 8 px — flush contra a borda, que é a única situação em
que as duas respostas diferem, e que o `stack_reach` **nunca produz**. Medido: o platô caía de 255
para 242 (o kernel perdia 4,9 % do peso pelo topo/base). A fixture ganhou margem e ficou mais forte
do que era (agora afirma também que o miolo continua opaco).

**(b) O Outline é o mesmo kernel com um CORTE, e a largura é uma promessa medida.** Para uma aresta
reta a silhueta borrada vale `Φ(−d/σ)` a `d` px para fora, então cortar em `Φ(−1) = 0,1587` põe a
borda **exactamente a σ px**. Medido: alcance **3,5 px** para largura 4 e **7,5 px** para 8 (o meio
pixel é a convenção da última coluna acima do limiar), com transição de **≤1 px** — contra os 5 e 10
px de banda de um Glow do mesmo σ. A meia-banda de anti-aliasing é **derivada** do gradiente do
perfil ali (`φ(1)/σ`), não escolhida. Por isso o slider dele se chama **Width** e não Radius: é a
tabela que dá o rótulo.

**(c) O Color Overlay é PONTUAL, e isso aparece no custo.** Um dispatch, sem vizinho nenhum, margem
zero. Medido a 512²: **6 overlays 0,282 ms contra 0,646 ms de 6 borrões**. O `passes_of` é porta
única — quem escreve os globals e quem despacha perguntam à mesma, porque as duas varreduras andam
em lockstep sobre a mesma lista e um `if` duplicado as descasaria em silêncio.

### UMA tabela, quatro consumidores

`ph2d_ecs::FxOp::SPECS` responde *o que este tipo é* para: o **painel** (que rows oferecer, e o
rótulo do raio), o **passe** (quanto espalhar, quantos dispatches), os predicados
`tints`/`displaces`, e o **WGSL** — cujos códigos são **gerados** (`kind_consts_wgsl`) em vez de
repetidos do outro lado da fronteira de linguagem. Com três tipos o painel decidia por `kind == 2`
espalhado pelo `paint`; com sete isso apodrece na primeira adição, e o modo de falha é um knob morto
que nenhum gate vê.

O `FilterRowView` **perdeu o campo `label`**: o nome só pode vir da tabela porque não existe outro
sítio de onde ele possa vir. Divergência inexprimível > divergência testada.

### Gates (e as três lições)

6 no catálogo de GPU + 6 de seam + 6 no modelo + os 8 da W1/W2 intactos. O gate que carrega a wave
é **`every_kind_draws_something`**: varre `FxOp::KINDS` e exige que TODO tipo mude a imagem — o
antídoto do modo de falha desta wave (um tipo entra na tabela, ganha botão, card e defaults, e o
shader cai no `else`). Um tipo novo entra nele no mesmo commit em que entra na tabela.

⚠️ **A fixture do Color Overlay não continha o fenômeno.** Ela testava só a força cheia, e ali
`alfa × k` com `k = 1` **é** o alfa — então a mutação que escreve a força no canal de cobertura era
a identidade, e o gate ficava verde sobre ela. Agora varre `[1,0 · 0,6 · 0,25]`.

⚠️ **Um "sobrevivente" era o HARNESS, não um buraco:** eu filtrava com `--ignored` um gate que é
puro (roda sem device) ⇒ **zero testes rodaram** e o verde era *nada aconteceu*
([[feedback_a_negative_search_needs_a_positive_control]]).

⚠️ **O seam varre a tabela INTEIRA, presença e ausência.** A versão da W2 comparava dois tipos
escritos à mão (Blur × Drop Shadow) e teria ficado **verde** sobre os quatro tipos desta wave: um
Color Overlay com slider de Radius, ou um Outline sem cor, passariam sem nada falhar.

7 mutações, 7 sangram.

### Aberto

- **`MAX_HALF = 96`** continua o teto do kernel; para o Outline ele limita a largura a ~32 px de
  tela, o que é muito mais do que um contorno pede.
- **O Outline arredonda quinas convexas** — é um corte no nível de uma Gaussiana isotrópica, logo
  aproxima a dilatação por DISCO (o que um pincel redondo faz). Uma dilatação exata seria `O(r²)`
  por texel, ou um EDT; não se justifica sem um pedido.
- **Sem blend mode por degrau** (o Layer Style do Photoshop tem um por efeito) — é W5, e não mexe na
  pilha: é um campo a mais no `FxOp` e um `mix` a mais no finalize.

## §9 — W4: a REVISÃO (o rim, a lei da sombra e a quina)

O smoke da W3 foi aprovado com **três observações**, e elas são de duas classes: uma era um BUG, as
outras duas eram o MODELO. A revisão pediu auditar os sete tipos, e ela achou um quarto defeito que
ninguém tinha visto.

### 1. O rim claro de 1 px (BUG)

O halo dos degraus de dentro era composto como uma **CAMADA por cima**
(`halo + over*(1 − halo.a)`), e isso **SOMA alfa**: na borda anti-aliased, `over.a = 0,5` com
`halo.a = 0,25` dava **0,625**. Como o `resolve` des-premultiplica, dividir por um alfa maior
**CLAREIA** — o rim era essa divisão, não uma cor.

**Um efeito de DENTRO tinge o que já está lá; ele não é uma camada nova.** A lei virou
`mix(over, tint·over.a, s)`, que deixa o alfa EXATAMENTE onde estava. Gate:
`an_inner_op_never_moves_the_coverage` — byte a byte, numa fixture de alfa em RAMPA, porque o
fenômeno vive na fatia fracionária e o gate antigo só olhava o miolo (255) e o lado de fora (0),
que estão certos **mesmo com o bug**.

### 2. A AUDITORIA achou um quarto: opacidade 0 apagava a forma

O Blur fazia `borrado × opacidade`, então opacidade 0 não era *este efeito não contribui* e sim **a
forma desaparece**. Os outros seis já eram no-op por construção. Agora é `mix(over, borrado, op)`, e
o gate **varre a tabela**: `an_op_at_zero_opacity_is_a_no_op_for_every_kind`. Foi a varredura que
separou os dois casos — escolher um tipo teria acertado 6 em 7.

### 3. A sombra de dentro não entrava nas reentrâncias — o MODO

O modelo (o do Photoshop) mede a **PROXIMIDADE do lado de fora**: o alfa invertido, borrado. Numa
reentrância o "fora" subtende um ângulo pequeno, então o número é pequeno **mesmo encostado na
borda** — e numa parte fina tudo está perto de fora, então ela escurece INTEIRA. É por isso que a
estrela tinha sombra só nas pontas.

A outra lei é a **DISTÂNCIA à borda**, que não tem ângulo nenhum: é 0 em todo ponto do contorno.
Medido numa cruz (as duas sondas à MESMA distância da borda, senão o gate compararia distâncias e
não leis):

| modo | reentrância | aresta reta |
|---|---|---|
| `Proximity` | **219** | 155 |
| `Contour` | **115** | 104 |

Os dois modos ficam (o Enio pediu os dois); **`Contour` é o default**. O campo vem de um **JFA
limitado** (`cs_sdf_seed` + `n` saltos), com `n = bits(w)` — 4 passes para uma banda de 8 px. ⚠️ Os
offsets são guardados em `rgba16float` e f16 representa inteiros **até 2048 exatamente**, então o
campo é exato na faixa que interessa; não é "aproximado porque é f16".

### 4. A quina do contorno: a derivação matou o pedido e a medição salvou metade dele

O pedido era *"opção de arredondar ou não"*. **Miter é impossível a partir do alfa**, e isso é uma
derivação, não uma preferência: numa quina de ângulo interno `θ` a ponta do miter fica a
`w/sin(θ/2)` do vértice — numa ponta de estrela (`θ ≈ 36°`), **3,24 × w**. Toda dilatação é uma soma
de Minkowski `A ⊕ S`; para esticar 3,24 w na quina o `S` teria de conter um ponto a 3,24 w naquela
direção, e aí engordaria 3,24 w **na aresta reta também**. Quem decide um miter são as DIREÇÕES das
duas arestas, e isso é geometria (`VecOffset { join }`, a pilha de Effects), não pixels.

**Mas a medição achou um defeito real no caminho:** o corte num campo BORRADO **não é uma
dilatação** — ele encolhe na quina convexa. Medido numa cunha de 36° com largura 10: a ponta
recebia **0,0 px** de contorno contra 10,5 px na aresta. O contorno passou a ser uma dilatação de
verdade sobre o mesmo campo de distância (`d ≤ w`): a ponta agora recebe **9,0 px**, e a largura é
a que o slider promete (medido 3,5 px para 4 e 7,5 px para 8, na convenção da última coluna acima
do limiar).

⚠️ **A assimetria que custou dois gates vermelhos:** *fora da textura* é semente para quem mede a
distância ao FORA (os degraus de dentro) e **não** para quem mede a distância à FORMA (o contorno).
Semear os dois igual fazia o contorno crescer a partir da borda da CENA: medido, 63 px de halo numa
largura de 4.

⚠️ **Meio texel, derivado:** o JFA mede até o CENTRO do texel semente e a fronteira geométrica está
0,5 px dele. Sem a correção o contorno sai 1 px mais fino do que a largura pedida — e o gate
**não pegava**, porque a tolerância dele (±1,5 px) era maior que o erro. O bar apertou para ±0,25.

### Gates

10 no catálogo de GPU (4 novos) + 7 de seam (1 novo) + 7 no modelo (1 novo). **13 mutações, 13
sangram.** Lições:

- **Uma fixture de borda DURA não distingue limiares de semente** — a mutação do limiar sobreviveu,
  e a resposta certa não foi um gate a mais e sim MEDIR: numa aresta diagonal com AA, a banda tem
  **0 níveis** de oscilação em 60 texels. O limiar não é load-bearing dentro da rampa de AA, e um
  bar que fingisse o contrário seria um gate que não pode falhar pelo motivo que alega.
- **O gate da diagonal nasceu VERDE sobre o nada:** a sonda caía a 6,4 px, onde a banda (de 8) já
  morreu — media `245..245`. Agora ela cai DENTRO da banda e há um **controle positivo** que exige
  isso.
- **Um `str.replace` sem asserção é no-op silencioso depois do `rustfmt`** — foi ele que me fez
  medir o lugar errado por duas rodadas.

### Aberto

- **Miter/bevel no contorno**: geometria, não raster (ver a derivação acima). O caminho é um
  *Stroke* na pilha de Effects, onde `VecOffset { join }` já vive.
- **O modo `Contour` custa `2 + bits(w)` passes** contra 2 do borrão — 6 para uma banda de 16 px.
  Barato (o JFA lê 9 texels por passe), mas é o tipo mais caro do catálogo.

## §10 — W5: o FEATHER e o BEVEL (o que o campo de distância destravou)

A W4 construiu o campo por um motivo (a sombra que respeita o contorno) e ele pagou dois tipos por
outro. **Nenhum dos dois precisou de maquinaria nova** — os dois são braços do `cs_op_field`, e o
painel não mudou uma linha, porque a TABELA o dirige. 7 → **9 tipos**.

### Feather — a borda amacia, o miolo NÃO

É o headline do Rive, e é o que um Blur **não** faz: um borrão mistura a COR também. Medido, com
listras dentro da forma: **contraste do miolo 195 no feather contra 1 no borrão** (195 nu). A rampa
é **centrada na fronteira** — a forma ganha alfa para fora e perde para dentro (medido: 24 a 2 px
fora, 151 na borda, 255 a 6 px dentro) —, e é isso que a separa de um recorte.

⚠️ **Fora da forma o pixel não tem cor própria:** ele herda a do texel de borda mais próximo, que é
exatamente para onde o campo aponta. E **dentro** da banda cada texel mantém a cor DELE: o feather
muda a COBERTURA, não a cor (há gate; sem ele, "pinte tudo com a cor da borda" passava).

### Bevel — o rebordo ganha luz

`off` aponta para a borda mais próxima, então **ele É a normal 2D do rebordo**: `dot(n, luz)` acende
a face virada para a luz e escurece a oposta, com o efeito morrendo para o miolo. Medido sobre
cinza: **rim 225 / 30 contra miolo 128**, e trocar a luz troca os dois.

⚠️ **O par de offset quer dizer coisas DIFERENTES conforme o tipo** — numa sombra é um
DESLOCAMENTO (amostra-se o campo mais adiante), num bevel é uma DIREÇÃO. Deslocar por ela moveria o
relevo inteiro em vez de o iluminar (foi o 1º corte, e o bevel saía inerte). Por isso a tabela
passou a **ROTULAR** cada knob (`offset_labels`, `color_label`) em vez de só dizer que ele existe:
o card do Bevel diz **Light X / Light Y** e **Shadow**, o da Drop Shadow diz **Offset X / Y**.

### A semente é escolhida pelo que o op PRECISA

| quem | semente | porquê |
|---|---|---|
| Inner Shadow / Glow / Bevel | os texels de FORA | *a que distância estou de deixar de existir* — a medida exata do que eles perguntam |
| Feather / Outline | a CASCA (a 1ª fileira de dentro) | precisam do campo dos DOIS lados |

⚠️ Tentei unificar tudo na casca e **a medição recusou**: na quina CÔNCAVA a casca de um lado só
estima ~0,6 px pior, e a reentrância é justamente onde o modo Contour existe para acertar (o gate
foi de 115 para 132 contra os 104 da aresta). O campo simétrico é melhor para quem sai da forma e
pior para quem fica dentro — então cada um semeia o seu.

### Gates

12 no catálogo de GPU (2 novos) + os 7 de seam e 7 de modelo, que cobriram os tipos novos
**sozinhos** (varrem a tabela). **21 mutações na jornada, 21 sangram.** Três lições, todas de
FIXTURE:

- **Uma forma BRANCA não tem para onde clarear:** o gate do bevel ficou vermelho sobre um produto
  correto, porque o realce saturava em 255. Fixture cinza.
- **Além do alcance do JFA o campo não existe**, então duas mutações eram a IDENTIDADE exatamente
  onde as sondas olhavam (o miolo). As sondas desceram para DENTRO da banda, e as duas metades que
  faltavam ("a cor não se move" e "o relevo decai") entraram junto.
- **`signed` é palavra reservada no WGSL** — o shader inteiro falhou a compilar, e o erro só
  aparece no `create_shader_module` (todos os 10 gates de GPU caíram de uma vez).

### Aberto

- **`MAX_FILTER_KINDS` foi de 7 para 9**, e o gate que o pega é o seam que CLICA cada "Add": o
  `.take(MAX_FILTER_KINDS)` do paint deixaria os dois últimos tipos sem botão em silêncio.
- **O bevel não tem "size" separado da profundidade** (o `Depth` governa os dois). O Photoshop tem
  Size + Soften; se o smoke pedir, é um knob a mais, não um modelo a mais.

## §11 — W5b: os três artefatos, e o que cada um ensinou

O smoke da W5 voltou com **pente** no feather e no bevel, **serrilha interna** no contorno e **as
pontas ceifadas** quando a forma tem traço. Três causas diferentes, e a primeira coisa que a wave
fez foi descobrir que **o gate que devia pegar tudo isso media a si mesmo**.

### 0. O gate media a SONDA, não o campo

O gate da banda andava "paralelo à aresta" numa grade de texels, o que obriga a **arredondar o y** —
e ±0,5 px de sonda sobre uma banda de ~32 níveis/px são **±16 níveis de oscilação inventada**. Ele
media 34 níveis sobre um campo que estava perfeito. E a fixture era a **45°**, o único ângulo onde a
discretização do campo some por simetria (ao longo de `x + y` constante o texel-semente mais próximo
é o mesmo para todos).

O oráculo virou um **BUCKET**: agrupa todos os texels cuja distância VERDADEIRA (analítica) cai numa
fatia estreita e exige que a sombra deles concorde — *à mesma distância, a mesma sombra*. Sem
arredondamento nenhum, e num ângulo **oblíquo** (21,8°).

### 1. O PENTE era a DIREÇÃO, não a distância

Com o bucket, o campo mediu **0 níveis**: a distância estava exata. O que penteava era o que se
derivava dela:

- o **bevel** tomava a normal do rebordo como `normalize(off)` — e `off` aponta para UMA semente,
  então ele salta na fronteira entre células de Voronoi. Agora a normal vem do **gradiente do
  campo** (diferença central de uma grandeza que já é suave).
- o **feather** amostrava a cor da metade de fora em `off` truncado, e a amostra oblíqua às vezes
  pousava num texel ainda transparente — cada buraco desses é um dente. Agora arredonda, entra meio
  texel e **tem fallback** de mais um.

### 2. A SERRILHA do contorno era a rampa de AA suposta com 1 px

No meio do contorno o alfa media 255..255; **na borda dele, 24 níveis** entre texels à mesma
distância. A semente sub-texel estimava a fronteira como `a − 0,5` do centro, o que supõe uma rampa
de exatamente 1 px — numa aresta oblíqua ela é mais larga (~`|nx|+|ny|`), e o erro chega a ~0,09 px.
Numa borda DURA isso lê como serrilha. A inclinação real está no próprio gradiente
(`|∇a|/2`), então a distância é **`2(a − 0,5)/|∇a|`** (Gustavson–Strand). **24 → 0 níveis.**

### 3. A PONTA CEIFADA era o bbox do scratch contra a junta MITER

`path_screen_bounds` inflava por **meia largura** de traço. Mas numa quina de ângulo `θ` a ponta do
miter fica a `½w / sin(θ/2)` do vértice — numa ponta de estrela, **3,24 × ½w** — e a kurbo só a
corta no `miter_limit` (4). A ponta era recortada contra a borda da textura, e o corte era reto:
exatamente o que o smoke mostrou. O bbox passou a inflar por `½w × miter_limit`, **lido do MESMO
construtor de traço que o renderer usa** (não de uma segunda constante).

⚠️ É a **terceira** vez nesta linha que `1/sin(θ/2)` decide alguma coisa: ele proibiu o miter no
contorno raster, explicou por que o corte-de-Gaussiana encolhia na quina, e agora dimensiona o
scratch.

### Gates

+2 de GPU (o bucket do contorno dos dois lados) e +1 puro na `ph2d-vec-render` (a ponta do miter
cabe no bbox, com a ponta calculada ANALITICAMENTE). **22 mutações na jornada, 22 sangram.**


## §12 — W6a: a LEI DE MISTURA por degrau

**Multiplicador, não um décimo tipo.** As vinte leis vezes os degraus que já existem: Inner Shadow
em `Multiply` escurece em vez de lavar, Inner Glow em `Screen` acende, Bevel em `Overlay` lê como
material, e Color Overlay em `Color` troca a matiz preservando a luminosidade — **o tint/duotone que
esta própria fila listava como item à parte, e que sai daqui sem uma linha de kernel nova**.

### A lei vem do arquivo que o compositor de camadas compila

`blend_sep`/`blend_hsl` (W3C Compositing L1, 22 modos) viviam dentro do `layer_composite.wgsl`,
pinados bit a bit contra o Rust. Ganharam um SEGUNDO consumidor ⇒ saíram para
`shaders/blend_modes.wgsl`, e a `composite_source()` os re-concatena. **Movimento de código puro**
— os 16 gates do compositor, incluindo os dois de naga, passaram sem uma alteração de asserção.

⚠️ **A porta de montagem é ÚNICA dos dois lados** (`composite_source` no compositor,
`module_sources` na pilha de FX): um gate que montasse a própria concatenação validaria a si mesmo
e ficaria verde sobre um produto que deixou de prefixar o bloco. Mutação: `blend = ""` ⇒ os dois
gates de naga vermelhos.

### QUEM toma a lei, e por que os outros não (medido)

A lei pesa pelo alfa do FUNDO — a fórmula do W3C, `Cs' = (1−ab)·Cs + ab·B(Cb,Cs)`. Um halo
**EXTERNO** entra POR BAIXO da entrada (`over + halo·(1−ab)`), então:

| onde | `ab` | a lei alcança | o halo aparece |
|---|---|---|---|
| fora da forma | 0 | nada (não há com que misturar) | sim |
| rampa de AA | ~0,5 | **0,25** (o pico) | metade |
| dentro | 1 | tudo | **nada** |

O produto `ab·(1−ab)` pica em **0,25 exatamente na rampa de anti-aliasing**. Um controle cujo
efeito inteiro é uma orla de 1 px lê como quebrado — e quem TINGE alcança **1,0** no MIOLO, quatro
vezes mais. Daí a lista: **Inner Shadow · Inner Glow · Bevel · Color Overlay**. Blur e Feather não
têm cor própria (a saída deles É a entrada transformada).

⚠️ **`takes_blend` é campo PRÓPRIO da `FxKindSpec`, e não `!grows`** — hoje as duas listas
coincidem, e coincidem por ACIDENTE: uma pergunta *preciso de margem na textura?*, a outra *a minha
cor encosta na de baixo?*. Um *Satin* futuro (espalha para fora E tinge por dentro) responderia sim
às duas, e derivar uma da outra o faria nascer sem o controle, em silêncio. Há gate que pina a
coincidência **como coincidência**, com a mensagem a dizer que divergir é legítimo.

### VINTE leis, e o `BlendMode` tem 22

`Behind` e `Clear` (20/21) são operações de **COBERTURA** — o próprio `apply` do Rust as desvia
antes da função de mistura. Um degrau aplica a lei dele exactamente onde a cobertura já está
decidida pela lei DELE (o `inner_tint` existe *para não a mover*, e foi um bug real resolvido).
Oferecê-las e depois dobrá-las em Normal no dispositivo seria a opção que despacha e mente.

### As portas

| pergunta | porta | consumidores |
|---|---|---|
| este tipo tem lei a honrar? | `FxOp::takes_blend` | o painel (OFERECER) · o produtor (HONRAR) |
| que código vai ao dispositivo? | `FxOp::blend_code` | `resolve_ops` |
| como duas cores se combinam? | `blend_modes.wgsl` | o compositor de camadas · a pilha de FX |
| como a lei entra num degrau? | `fx_blend` (WGSL) | `inner_tint` (os três de dentro) · Color Overlay |

⚠️ **`inner_tint` ser porta única deu o blend aos três de dentro numa linha** — o Bevel de graça (a
cor dele já é escolhida antes: branco na face iluminada, o tint na oposta).

### Normal é byte-idêntico, e o early-out é load-bearing

`mix(x, x, a)` é `x·(1−a) + x·a`, que em ponto flutuante **não é exactamente `x`** (gate
`the_normal_early_out_is_load_bearing_because_mix_is_not_the_identity` — medido, diverge em muito
mais que 1% dos pares). Sem o `return` antecipado no `fx_blend`, o caminho default da pilha inteira
deixaria de ser byte-idêntico e a wave passaria a mexer na aparência de toda arte já autorada, um
nível de cada vez.

### O que o processo de mutação achou (três coisas, todas por não sangrar)

1. **Neutralizar `is_hsl` deixava tudo verde** — as quatro leis não-separáveis não tinham cobertura
   nenhuma, e `Color` é a que justifica a wave. Gate novo.
2. **Jogar fora o peso do fundo (`mix → b`) também** — a fixture era um quadrado de borda DURA, sem
   cobertura parcial, e o peso só é observável onde `ab` varre `(0,1)`. **A fixture não continha o
   fenômeno**; agora há uma RAMPA (252 → 204 → 128).
3. **O oráculo do HSL nasceu VERMELHO sobre produto CORRETO** — media Rec.709 sobre bytes sRGB, e a
   lei preserva `0,3/0,59/0,11` sobre LINEAR. Dois pesos, dois espaços; preservar um não preserva o
   outro. Medido certo: base **0,2159** → Color **0,2163**.

E um gate que **não podia falhar** (comparava blend 0 com blend 0) foi deletado em vez de
contrabandeado.

### Schema

**`PROJECT_SCHEMA` 37→38** — `FxOp` ganhou `blend` APENDADO, e postcard é posicional.
`serde(default)` não salva: o formato não tem NOMES de campo, e um buffer que acaba cedo é erro de
decode. `FLIP_SCHEMA_VERSION`/`VEC_SCENE_SCHEMA_VERSION` **intactos** (a lei é do componente ECS; a
geometria do caminho não mudou uma vírgula). ⚠️ O número se CONTA a partir do `main` do dia.

### Smoke

**`PH2D_BUILD_SMOKE=34`** — quatro PARES, em cada um a mesma cor e a mesma opacidade, só a lei
diferente. Números medidos pela sonda `measure_the_smoke_scene_pairs` antes de a mensagem os
afirmar (ver o commit da cena: o par do Inner Shadow foi medido no lugar errado e depois com uma
cor errada, e as duas rodadas estão registadas).

### Aberto

- **A lei de um halo EXTERNO contra a CENA** (o Drop Shadow em Multiply do Photoshop) exigiria que
  a textura de saída do FX carregasse uma lei para o composite da cena — outra camada, outro dono.
  Nomeado, não construído.
- **O Bevel tem UMA lei para as duas faces.** O Photoshop tem duas (Highlight: Screen · Shadow:
  Multiply). Uma só já é coerente (Multiply mata o realce e mantém a sombra; Screen o inverso), e o
  par é refino de produto.


## §13 — W6b: a TURBULÊNCIA (o eixo orgânico)

**O pedido do plano:** *turbulência + deslocamento, o eixo que ninguém no 2D vetorial entrega bem.*

### A pesquisa decidiu a FORMA da feature, não só a matemática

O `<filter>` do SVG separa **`feTurbulence`** (gera um campo de ruído Perlin, com `baseFrequency`,
`numOctaves`, `seed` e `type ∈ {fractalNoise, turbulence}`) de **`feDisplacementMap`** (usa dois
canais dele para deslocar uma imagem). Todo mundo que veio depois **FUNDIU os dois**:

| Ferramenta | Como embrulha |
|---|---|
| **After Effects** | *Turbulent Displace* — o ruído mora DENTRO do efeito (Amount · Size · Complexity · Evolution) |
| **Photoshop** | *Displace* com um mapa em ARQUIVO — a interface que ninguém usa, e o motivo de todos os outros a evitarem |
| **Illustrator** | *Roughen* é VETORIAL (move âncoras) — outro eixo, que a nossa pilha de LPE já cobre |
| **Rive** | nada |

**E na nossa arquitetura a fusão não é conveniência, é obrigatória:** a pilha é uma LISTA em que
*todo op é imagem → imagem*. Um degrau que só GERASSE ruído teria de escrever a saída dele por cima
da imagem que o degrau seguinte espera receber — ele apagaria o trabalho anterior. Um tipo só.

### O desenho, e as portas únicas

| Pergunta | Porta |
|---|---|
| *este degrau lê um campo de ruído?* | `FxKindSpec::noise_labels` (o painel OFERECE, o produtor HONRA — o molde do `takes_blend`) |
| *quantas oitavas ele de fato soma?* | `FxOp::detail_clamped` |
| *como este degrau é executado?* | `plan_of` → `Plan::Warp` (**pelo TIPO, antes do modo**) |
| *quanto ele espalha?* | `op_reach` = o próprio Amount |
| *onde a grade do ruído está ancorada?* | `stack_reach(ops).0/.1` — a MESMA função que dimensiona o scratch |

**A colisão de modos que isto expôs:** o `mode` é um índice na lista DO TIPO, então `1` é
`MODE_CONTOUR` num degrau de dentro e `MODE_CREASED` na turbulência. O `plan_of` roteava por
*"tem modos, e escolheu o 1?"* — uma turbulência *Creased* cairia no campo de distância e desenharia
outra coisa, sem erro nenhum. O tipo passa a decidir primeiro, com gate nos dois modos.

**A ancoragem é o ponto não-óbvio.** A coordenada do ruído é `(pixel − org)/escala_px`, com `org` = a
margem que a pilha reservou. Sem esse termo a grade fica presa ao canto do *scratch* — e a margem é
função de TODA a pilha, então mexer no raio de um Glow faria o padrão inteiro **andar** por baixo da
forma. Com ele, a coordenada é `(mundo − caixa_da_forma)/escala_mundo`: invariante ao zoom e imune
aos outros degraus. Gate: `the_noise_is_pinned_to_the_shape_not_to_the_scratch` (**0,0000 px**).

### Os números, medidos (RTX)

- **`MAX_DETAIL = 6` NÃO é teto de custo** — 1 a 12 oitavas movem o passe de 0,058 para 0,12 ms a
  512², que é a própria dispersão da medição. É teto de **REPRESENTAÇÃO**: a 7ª oitava move a borda
  **0,019 px** (a 6ª move 0,044; a 10ª, 0,002).
- **`op_reach` = o Amount**, não `3σ`: o campo vive em `[-1,1]`, então nenhum texel viaja mais que
  isso — e `3σ` seria margem paga por um borrão que não existe.
- Amount 4/8/16 px → amplitude de borda 1,03/2,07/2,97 px (o desvio-padrão é uma FRAÇÃO do Amount,
  porque o campo tem média zero).

### Duas coisas que a medição corrigiu em mim

1. **A fixture continha OUTRA coisa.** As linhas do topo e da base do quadro amostram FORA do
   scratch (o `dy` puxa de onde não há nada), e os cruzamentos espúrios delas inflavam a amplitude
   em 3× — `[17,75; 65,28]` com o miolo INTEIRO em 63,3. **Um** defeito de fixture reprovava TRÊS
   gates, e eu teria "consertado" três coisas certas.
2. **A semente por-oitava é load-bearing, e o mecanismo que eu escrevera era outro.** Eu dizia que
   ela impede os zeros de se alinharem — falso (Perlin vale zero em todo nó da própria grade, em
   qualquer semente). O que ela impede é as oitavas lerem a MESMA tabela de gradientes em células
   relacionadas: sem ela a rugosidade do modo **Smooth** sobe de 0,419 para 0,609 e **encosta na do
   Creased** (0,602) — o modo liso deixa de ser liso e os dois modos desenham a mesma coisa.

### Schema

`PROJECT_SCHEMA` fica em **38**, o mesmo bump da W6a: o `FxOp` ganhou `scale`/`detail`/`seed` na
mesma janela, e um save v37 já é recusado pelo 38 — um 39 jogaria fora exatamente os mesmos arquivos
e custaria mais um degrau para ninguém. **Uma linha, um bump.**

### Smoke

**`PH2D_BUILD_SMOKE=35`** — quatro pares (Amount · Size · Detail · Modo), cada estrela com um
CONTORNO por baixo da turbulência: é a linha fina que torna visível um deslocamento de poucos pixels,
e de quebra é a prova de que o degrau COMPÕE.


## §14 — W7: GROW / SHRINK (a morfologia)

A última família de primitivas do SVG que faltava ao catálogo — a lista que o §7 abriu
(*color-matrix · morphology · displacement+turbulence · bevel*) fica com **um** item aberto — que a
§15 (W8) fechou.

### A pesquisa decidiu a FORMA, e a RÉGUA

| ferramenta | interface | elemento estruturante |
|---|---|---|
| SVG `feMorphology` | `operator="erode\|dilate"` + radius | **retângulo** (quinas quadradas) |
| Photoshop *Minimum/Maximum* | dois itens de menu | os dois (*Preserve: Squareness/Roundness*) |
| AE *Simple Choker* | **um slider com sinal** | disco |
| Blender *Dilate/Erode* | **uma "Distance" com sinal** | disco |
| Illustrator *Offset Path* | **um valor com sinal** | disco (com joins) |

**O modal é a interface ANTIGA.** Um enum é barato num formato *declarativo* (o SVG) e o Photoshop
herdou dois itens de menu da era dos filtros de 1990. Todo mundo que desenhou isto como **CONTROLE**
convergiu no bipolar, e por um motivo: crescer e encolher são a mesma operação em sinais opostos, e
quem afina um *choke* quer atravessar o zero sem trocar de modo.

E a régua sai de graça: com o campo de distância euclidiano do W4 o conjunto novo é `{d ≤ r}`, ou
seja um **DISCO**. Medido na quina de uma caixa — crescer 10 px alcança **10,00 px** na diagonal,
contra os **14,14** que um retângulo alcançaria.

### O desenho, e as portas únicas

| pergunta | porta | consumidores |
|---|---|---|
| este tipo engorda/afina? | `FxKindSpec::grow_label` | o painel (OFERECER) · o produtor (HONRAR) |
| **contra o QUÊ ele mede?** | `FxOp::measures_the_image` | `plan_of` (a semente) · o writer dos globals (o `n_segs`) |
| de que cor é a área nova? | `straight_colour` (WGSL) | o feather · a morfologia |
| quanto ele espalha? | `op_reach` | `stack_reach` → o tamanho do scratch |

**Não há kernel novo, e é o desenho inteiro:** `a = clamp(sdist + r + 0.5, 0, 1)`. A rampa é linear
e de um texel porque a cobertura de uma aresta reta a distância `d` do centro **é** `d + 0,5`
recortado — a mesma lei que todo renderer de SDF usa.

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

### O oráculo, e o número que NÃO é desta operação

O gate que fecha a wave não compara contra o ideal: **`Grow(r)` e `Outline(r)` descrevem o MESMO
conjunto** (o comentário do contorno já dizia *"isto é uma DILATAÇÃO de verdade (`d ≤ w`)"*), logo
têm de pôr o contorno no mesmo lugar. Medido: **71,992 contra 71,992** — e o gate de catálogo, que
não sabe nada disto, conta os **mesmos 1152 texels** para os dois.

⚠️ **MEDIDO e nomeado:** o campo semeado pelo raster põe a fronteira **~0,5 px adiante** numa aresta
DURA alinhada aos eixos quando o JFA propaga longe. **Não é desta operação** — um Outline de 8 px
mede `+8,494` no mesmo caminho contra `+8,000` pelo pé exato da geometria. A fixture é o pior caso do
estimador de sub-texel (rampa de AA de exatamente 1 texel, com a diferença central a ler amostras
saturadas). Como a morfologia mede a IMAGEM de propósito, ela paga essa régua **sempre**.

### Gates — 14 novos, 10 mutações, **10 sangram**

⚠️ **Uma mutação exigiu TRÊS iterações de fixture** até conter o fenômeno (colapsar os dois braços
de `straight_colour`): numa forma **monocromática** *"a minha cor"* e *"a cor da borda"* são o mesmo
número; num texel do **miolo** o JFA não alcança e o braço de vizinhança amostra a si próprio; só o
**ENCOLHER** — onde os texels sobreviventes estão dentro do alcance do salto — os separa.

⚠️ E o **gate de catálogo pegou a wave**: `every_kind_draws_something` constrói cada tipo com o
*raio* visível, e o knob visível deste é outro campo ⇒ ele entrava no ponto NEUTRO e "não desenhava
nada". A fixture passou a perguntar à TABELA, como a linha do `offset` ao lado já fazia.

### Schema

**`PROJECT_SCHEMA` fica em 38** — a terceira leva da mesma linha (`blend`, depois
`scale`/`detail`/`seed`, agora `grow`). Um save v37 já é recusado pelo 38; pôr cada leva num número
próprio jogaria fora exatamente os mesmos arquivos. **Uma linha, um bump.**

### Smoke

**`PH2D_BUILD_SMOKE=36`** — quatro pares: o SINAL · o ELEMENTO (as pontas arredondam) · **a ORDEM**
(`Outline → Grow` contra `Grow → Outline`, o headline) · e o USO (o *choke* clássico: encolher antes
de borrar).

### Aberto

- **Falta o `feColorMatrix`** (tint/duotone/saturate/`luminanceToAlpha`) — o último item da lista do
  §7. O Color Overlay com as vinte leis de mistura já cobre a maior parte do que ele entrega.
- **Os joins do offset**: o Illustrator oferece miter/round/bevel no *Offset Path*. Aqui a quina é
  sempre redonda, porque a régua é a distância; um miter exigiria geometria, e a pilha de LPE
  (`VecOffset { join }`) já tem essa resposta no eixo certo.

## §15 — W8: COLOR ADJUST (o `feColorMatrix`) — e a lei NÃO é nova

O último item da lista que o `apply_op` da W2 nomeou (*color-matrix · morphology ·
displacement+turbulence · bevel*). Com ele o catálogo fecha.

### A pesquisa decidiu a FORMA — e depois o REPO decidiu a LEI

**O `type` de quatro valores do SVG é interface de formato DECLARATIVO.** O `feColorMatrix` tem
`matrix` / `saturate` / `hueRotate` / `luminanceToAlpha`; quem o desenhou como **CONTROLE**
convergiu, sem exceção, na ficha **Hue / Saturation / Brightness**: Photoshop, After Effects
(*Hue/Saturation*), Krita, Blender (*Hue Saturation Value*). Três sliders com o neutro no meio.

| `feColorMatrix type` | onde ele pousa |
|---|---|
| `saturate` | o slider **Saturation** |
| `hueRotate` | o slider **Hue** |
| `matrix` (a parte que um artista pede) | o slider **Brightness** |
| `luminanceToAlpha` | **NÃO ENTROU**, e é decisão: ele converte luminância em COBERTURA, e a pista pontual existe precisamente para não mover cobertura. É outro verbo, não um quarto slider. |

**E então a pergunta mudou de lugar.** Procurando onde escrever a rotação de matiz, ela já estava
escrita: `AdjustmentKind::HueSaturationBrightness`, no `ph2d-painter-effects`, rotulada
literalmente *"Hue/Saturation"*, com lei de CPU (`apply_hsb`) **e** kernel de GPU (o `case 0u` do
`layer_composite.wgsl`) — a camada de ajuste que o Painter ship há waves. Escrever uma segunda
seria dar ao app **duas respostas para *"o que o slider de matiz faz?"***, divergindo no único
lugar onde ninguém lê um número: uma cor.

⚠️ **E o repo já tinha decidido QUAL lei, pagando por isso.** O doc do `apply_hsb` diz porque a
matiz roda em **OKLab** e não em HSL: *"HSL hue is numerically unstable for near-gray pixels …
the colored speckle Enio hit on the gray background"*. A escolha não é gosto, é uma cicatriz.

### A wave, então, é uma EXTRAÇÃO — a segunda vez que este módulo a faz

`oklab_from_linear` / `oklab_to_linear` / o corpo do `case 0u` saíram para
`shaders/colour_adjust.wgsl`, prefixado pelo `composite_source()` do compositor **e** pelo
`module_sources()` da pilha. É, ao pé da letra, o movimento que o cabeçalho do `blend_modes.wgsl`
já descreve — *"extraído do `layer_composite.wgsl` quando ganhou o SEGUNDO consumidor (a pilha de
FX raster)"* — e o gate novo é o irmão exacto do que aquela extração deixou.

| pergunta | porta única | consumidores |
|---|---|---|
| o que matiz/saturação/brilho fazem a uma cor? | `adjust_hsb` (WGSL compartilhado) | o compositor de CAMADAS · a pilha de FX |
| este degrau ajusta cor? | `FxKindSpec::adjust_labels` | o painel (OFERECER) · o produtor (HONRAR) |
| ele está no neutro? | `FxOp::adjust_is_neutral` | o gate · o kernel |
| quanto ele espalha? | `op_reach` → **0** (pontual, a pista do Color Overlay) | o `stack_reach` |

### O oráculo não é uma tolerância: é a OUTRA implementação

`the_adjust_is_the_law_the_painter_already_ships` roda o degrau na GPU e o `apply_adjustment` do
`ph2d-painter-effects` na CPU, sobre as mesmas cores e os mesmos knobs. **Pior divergência: 1
nível de byte**, em 5 combinações × 9 cores. A força do oráculo é ele não ter sido escrito para
esta wave.

### O que a medição corrigiu (três vezes, e as três eram afirmações minhas)

1. **"o brilho move um pixel de qualquer cor"** — falso. A fixture da varredura do catálogo é
   BRANCA, e `+brilho` é `out + (1−out)·b`, que em branco é branco: **0 de 12800 texels**. Um
   ajuste pontual tem pontos FIXOS por construção. A varredura passou a empurrar o brilho para
   BAIXO, e nasceu o gate `an_achromatic_pixel_is_untouched_by_hue_and_saturation` para pinar onde
   eles estão.
2. **"a rotação preserva o croma"** — falso para cor viva: o vermelho da paleta cai a **0,641**
   num quarto de volta. A rotação é rígida em OKLab, mas o resultado pode sair do gamut do sRGB e
   a viagem de volta a 8 bits **corta**. Reescrevi para *"nas duas cores que ficam no gamut"* — e
   isso **também** era falso, porque eu tinha medido UM ângulo: o âmbar cai a 0,817 a ⅜ de volta e
   o azul a 0,736 a −⅛. **Estar no gamut é propriedade do par (cor, ângulo).** A fixture que
   contém o fenômeno é uma cor de croma BAIXO — medida no giro inteiro, razão **0,989..1,010**.
3. **"o early-out do neutro é load-bearing para a byte-identidade"** — falso, e foi uma MUTAÇÃO
   que o mostrou: removendo o ramo, uma rampa sRGB completa sai com **0 de 4096 bytes diferentes**.
   O erro do ida-e-volta OKLab em `f32` fica sob meio nível e a quantização o come. O ramo é
   exactidão no FLOAT (que compõe numa pilha longa) e CUSTO — a mesma frase que o `apply_hsb` do
   Painter já usava, e que eu tinha lido sem a ler.

### Gates — 12 novos, 12 mutações, **11 sangram**

O sobrevivente é o (3) acima: ele não expôs um buraco de gate, expôs uma frase errada em três
doc-comments, que foram corrigidos com o número.

⚠️ **E o que a mutação M5 ensinou sobre a divisão de trabalho:** apagar o braço do Color Adjust do
`cs_op_point` mata **8** gates desta wave e **não** mata o `every_kind_draws_something` — sem o
braço, o degrau cai na pista do Color Overlay, que repinta com o `tint` da varredura e portanto
*desenha alguma coisa*. A varredura pergunta **se** um tipo desenha; os gates dedicados perguntam
**o quê**.

### Schema, contratos, ids

- **`PROJECT_SCHEMA` fica em 38** — a quarta leva da mesma linha (`blend`, `scale`/`detail`/`seed`,
  `grow`, agora `hue`/`sat`/`bright`). **Uma linha, um bump.**
- **Contrato congelado §6: INTACTO** (conferido por gate: `architecture_contract_surface` 3 ✓ ·
  `architecture_tool_contract_surface` 4 ✓).
- **`MAX_FILTER_KINDS` 11 → 12**; ids novos `filter_{hue,sat,bright}_id{,_num}` (6).
- **`Globals` 96 → 112 bytes** (uma linha de 16: matiz/saturação/brilho + pad).
- Superfície pública nova: `FxOp::COLOR_ADJUST` · `FxOp::{hue,sat,bright}` ·
  `FxKindSpec::adjust_labels` · `FxOp::{reads_adjust,adjust_is_neutral}` ·
  `FxOpGpu::{hue,sat,bright}`. `FxOpGpu` **mudou de módulo** (`fx_stack_op`, mesmo caminho de
  import, teto de LOC).

### O que ficou de fora, nomeado

- **O DUOTONE de duas pontas** (mapear a luminância de uma cor escura a uma clara — o *Tint* do AE,
  o *Gradient Map* do Photoshop). O **Color Overlay com a lei `Color`** já entrega o tingimento
  monocromático (troca a matiz preservando a luminosidade), e o Painter já tem um `gradient_map`
  próprio; o que falta é só a PONTA ESCURA ser de outra matiz. É um degrau com uma **segunda cor**,
  não um knob — wave própria, se o uso a pedir.
- **`luminanceToAlpha`** — muda cobertura (acima).

---

## §16 — W9: DUOTONE (duas pontas) e LUMA TO ALPHA — as duas leis que leem o BRILHO da arte

Pedido do Enio, com o eixo de prioridade nomeado na mesma frase: *"quero boa qualidade, mas quero
principalmente **performance em tempo real em runtime para games**"* + `duotone de duas pontas` +
`luminanceToAlpha`.

### §16.1 — A prioridade responde ANTES do desenho

Os dois tipos são **PONTUAIS**: um dispatch, sem vizinho, margem ZERO, sem textura intermediária
extra. É a classe mais barata da pilha, e o número é medido
(`the_pointwise_op_costs_much_less_than_a_blur`, RTX):

| | custo |
|---|---|
| moldura (a pilha vazia) | 0,058 ms |
| **6 degraus pontuais** | **+0,022 ms** (≈ 0,004 ms cada) |
| 6 borrões | +0,575 ms (≈ 0,096 ms cada) |

Um degrau destes custa **0,02 % de um quadro de 60 fps**. A prioridade não teve de ser negociada
contra a qualidade porque a operação que o pedido descreve já é, por natureza, a barata.

### §16.2 — A pesquisa, e o que ela decidiu

**Duotone.** O `feColorMatrix` do SVG não tem duotone; quem o desenhou como CONTROLE convergiu numa
**rampa de dois stops**: a *Gradient Map* do Photoshop (o Duotone de impressão é a mesma coisa com
tintas), o **Tint** do AE (`Map Black To` / `Map White To` + `Amount to Tint`), o *Colorize* do
Krita. Nenhum deles pede um gradiente completo para o caso de duas pontas — e é o caso de duas
pontas que o artista usa. Duas swatches, e a "quantidade" é a Opacity que todo degrau já tem.

**Luma to Alpha.** Aqui há uma referência exacta (`type="luminanceToAlpha"`) e nós **divergimos dela
de propósito** — ver §16.4.

### §16.3 — A RÉGUA: o `L` do OKLab, e não o `lum()` das leis de mistura

As duas leis fazem a MESMA pergunta (*quão claro é este texel?*) e o repo tem **duas** funções que
parecem respondê-la. Elas não são intercambiáveis:

- `lum()` (`blend_modes.wgsl`) é a **luminosidade do W3C**, definida para os modos `Color` /
  `Luminosity`. Ela opera em luz LINEAR.
- `oklab_from_linear(...).x` é a **lightness perceptual**, e é literalmente a definição de *onde
  neste eixo claro↔escuro o texel senta*.

**Medido** (`measure_the_two_candidate_rulers_for_the_ramp`), no cinza sRGB 128:

| régua | valor |
|---|---|
| `lum` sobre luz linear | **0,216** |
| `L` do OKLab | **0,600** |

Com o `lum`, o meio-tom cairia a **um quinto** do caminho da rampa e a arte inteira se empilharia na
ponta escura. O `L` casa com o que Photoshop e AE desenham. E os coeficientes do `L` **somam 1**,
então preto puro vale 0 e branco puro vale 1 — é isso que faz as duas swatches significarem
exactamente o que o rótulo diz (gate próprio).

### §16.4 — A divergência do SVG, e por que ela é a metade que faz o efeito servir

A matriz do `feColorMatrix` escreve `A' = luma(cor RETA)` e **ignora o alfa que estava lá**. Num
pipeline premultiplicado isso ENDURECE a orla anti-aliased: a cor reta da orla é a MESMA do miolo,
então a rampa de cobertura vira um DEGRAU. **Medido** (mutação com a lei literal instalada): um texel
com **4/255** de cobertura salta para **180/255**.

A nossa lei **ESCALA** (`A' = A · luma`), o que preserva a rampa, e **preserva a cor** em vez de a
zerar. O argumento decisivo não é estético, é de composição:

> **encadear recupera o SVG, e o contrário é impossível.** `Luma to Alpha` → `Color Adjust
> (Brightness −1)` dá o matte PRETO exacto da matriz; nenhuma ordem de degraus devolve a cor que já
> foi apagada. A lei que **guarda informação** é a que compõe.

Há gate para as duas metades (a orla que sobrevive · a cadeia que reproduz o SVG).

### §16.5 — As portas únicas

| pergunta | porta |
|---|---|
| que rótulos tem a segunda swatch? | `FxKindSpec::color_b_label` (a tabela, como todo o resto) |
| qual PONTA o picker abriu? | `fx_live::colour_target(id) -> (linha, é_a_segunda)` |
| a cor de um degrau em bytes | `fx_live::colour_bytes` (nasceu com a 2ª ponta: dois chamadores) |
| como desenhar uma swatch de filtro | `filter_color_swatch(id, cor, rótulo, y)` — recebe o id, e é
  isso que faz a segunda ponta ser a PRIMEIRA outra vez |

### §16.6 — O que a wave encontrou de errado no que já estava lá

1. **Uma família de acessores `pub` sem UM chamador.** `reads_noise` / `reads_grow` / `reads_adjust`
   (das W6b/W7/W8) — cada um com um doc-comment a afirmar *"porta única com dois consumidores: o
   painel a consulta para OFERECER, o produtor da GPU para HONRAR"*. **A frase era falsa nos dois
   lados:** o painel não alcança o `ph2d-ecs` (lê o `FilterKindView` publicado) e o produtor copia
   os campos incondicionalmente — quem HONRA é o ramo por `kind` dentro do shader. Os três foram
   **removidos**, e o quarto (o meu) não chegou a existir. Achado por uma **mutação que
   SOBREVIVEU**.
2. **O `node_id_collisions` estava cego a metade da seção.** Ele enumera os ids de linha à mão, e as
   três waves anteriores acrescentaram **catorze** sem entrar na lista. Acrescentar só o meu teria
   continuado a rotina; agora são 32 por linha + os modos + as opções de mistura.
3. **Um doc-comment órfão a mentir um número.** *"64 bytes de propósito"*, pendurado num `use`, com o
   struct em 112. A nota foi para o campo de padding que ela descreve, com o número certo.
4. **A varredura de tipos não podia conter o fenômeno.** A fixture dela é uma CHAPA branca, e sobre
   branco puro o Luma to Alpha é a IDENTIDADE — ela reportaria *"não desenha nada"* sobre um produto
   correto. A varredura ganhou fixture própria (um DEGRADÊ); as outras ficaram na chapa, porque os
   comentários delas estão calibrados nela.

### §16.7 — Estado

`PROJECT_SCHEMA` **fica em 38** (a política que o próprio 38 declara: uma linha, um bump — ele já
carrega a turbulência, a morfologia e o ajuste, e um save v37 já é recusado). `MAX_FILTER_KINDS`
12 → **14**; `Globals` 112 → **128 bytes**. Zero `Cargo.toml`, zero crate nova, zero ADR, contrato
congelado intacto.

**9 gates** no arquivo novo (oráculo em CPU independente: pior delta **1 nível de byte**), **10
mutações, 9 sangram**. Smoke: **`PH2D_BUILD_SMOKE=38`**.

---

## §17 — W10: o ATLAS DE RASTER — de `n` renders do Vello para UM

A primeira wave deste plano que **não acrescenta um tipo**. Ela responde a pergunta que o eixo do
Enio faz — *"performance em tempo real em runtime para games"* — e que nenhum número deste
documento tinha respondido: **todos eles são de UMA pilha sobre UMA forma.**

### §17.1 — A medição veio antes do desenho, e derrubou a minha hipótese

Num editor o artista mexe numa forma de cada vez, então o memo protege tudo. **Numa cena de jogo as
formas filtradas ANIMAM**, logo erram o memo **todas, todo frame** — e o que multiplica é o custo
por-FORMA, que nunca tinha sido medido (`ph2d-render/tests/fx_scene_scale_cost.rs`, RTX):

| N formas | frame (ms) | só o raster | por forma |
|---|---|---|---|
| 1 | 0,30 | 0,20 | 0,27 |
| 4 | 0,80 | 0,54 | 0,20 |
| 16 | 3,14 | 2,23 | 0,20 |
| **32** | **6,40** | **4,03** | **0,20** |

⚠️ **A minha hipótese estava errada, e a medição a matou numa linha.** O `VelloPass::ensure_size`
compara por **igualdade** (`size == self.last_size`) enquanto o irmão dele, o
`FxStackPass::ensure_work`, **cresce e guarda** (`>=`) — então uma cena de formas de tamanhos
diferentes realocaria a textura intermediária uma vez por forma. Medido: formas **uniformes** e
**variadas** custam o **mesmo** (6,16 contra 6,40 a N=32). A realocação não é o custo.

**A causa é o custo FIXO de um render do Vello** — ele corre a cadeia de binning/tiling inteira e
submete, seja qual for o conteúdo. A **mesma área de arte**, medida dos dois jeitos:

| N | `n` renders | **1 render (atlas)** |
|---|---|---|
| 32 | **3,82 ms** | **0,39 ms** |

O fixo é **~0,12 ms por render** e não depende do conteúdo (0,20/0,21/0,39 ms para 4/16/32 estrelas
num atlas só).

**E o segundo eixo foi medido junto:** uma corrida da PILHA custa **0,031–0,042 ms** com o degrau
mais BARATO que existe (um pontual, onde o trabalho real é ~0,003) ⇒ **~0,03 ms de fixo** por forma
(encode + bind groups + `submit`). Somando: **73 % do frame de FX de uma cena de 32 formas era
overhead fixo por-forma.**

### §17.2 — O que shipou, e a porta única que já existia

Todas as formas que erram o memo são rasterizadas **numa passagem só**, num ATLAS, cada uma na
célula que o empacotador lhe deu; a pilha de cada uma lê a **própria célula**.

⚠️ **O comentário do `run` já tinha metade da resposta escrita:** *"o INGEST é o que põe a fonte no
espaço de trabalho, então depois dele **nenhum op volta a ler `src`**"*. Logo há exactamente **um**
sítio a deslocar — o `cs_ingest` — e o resto do módulo continua a falar em coordenadas **locais da
forma** (as work textures, os segmentos de silhueta, a origem do ruído, o `dst`).

| Pergunta | Porta |
|---|---|
| *onde, na fonte, começa a célula desta forma?* | `Globals.src_org`, lido **só** pelo `cs_ingest` |
| *como a pilha lê uma célula?* | `FxStackPass::run_from` (o `run` delega com `[0,0]` ⇒ byte-idêntico) |
| *onde cabe cada célula?* | `shells/desktop/src/fx_atlas.rs::pack` — pura, sem `wgpu` |

O empacotador é de **prateleiras**, determinista, com textura **apertada** (área desperdiçada custa
preenchimento, que é a metade barata; o que se compra é o número de RENDERS) e **divide em lotes**
quando não cabe — devolver *"não coube"* faria a forma **desaparecer**, que é o modo de falha errado
para um limite de recurso.

**Medido ponta a ponta, o mesmo trabalho pelos dois caminhos:**

| N | ontem | hoje | ganho |
|---|---|---|---|
| 4 | 0,79 | 0,45 | 1,76× |
| 16 | 3,26 | 1,16 | **2,81×** |
| 32 | 6,00 | 2,22 | **2,7×** (2,58–2,82 em quatro corridas) |

**32 formas filtradas: 39 % de um quadro de 60 fps → 14 %.**

### §17.3 — Os três defeitos que os gates acharam, todos meus

1. ⚠️ **O `tap_img` limita-se a `g.dims`**, que é o tamanho da **FORMA**. Isso é *correto* para as
   work textures — o `ensure_work` **cresce e nunca encolhe**, então fora de `dims` moram os pixels
   de uma forma maior de um frame anterior, e é exactamente esse limite que os torna invisíveis — e
   *errado* para a fonte, onde a célula vive em `src_org .. src_org + dims`, logo **metade dela cai
   fora de `dims`** e era lida como vazio. O gate de paridade nasceu **VERMELHO com 8806 bytes
   diferentes**, a forma cortada na coluna `dims.x`. Cura: `tap_src`, com limite **próprio**
   (`textureDimensions(t0)`) — nenhum uniform novo.
2. ⚠️ **Uma forma maior que o teto era posta na prateleira corrente** e espalhava o próprio excesso
   pela textura do **lote inteiro**: os vizinhos, que cabiam, passavam a viver numa textura que o
   device recusa (`lote 9000x104 passou do teto 8192`). Agora ela vai sozinha, e a textura sai do
   tamanho dela.
3. ⚠️ **O atlas introduz um modo de falha NOVO e MUDO:** arte que passe da caixa calculada (um traço
   mais largo, uma junta miter comprida) cairia **dentro da célula da vizinha** — com um render por
   forma, a borda da textura a descartava. Cada célula é **RECORTADA**; **não é otimização, é a
   reposição de um limite que já existia**, e o que apareceria é arte a mais numa forma que não a
   pediu, no z dela, com a cor dela.

### §17.4 — Gates

- **2 de paridade de GPU** (`fx_stack_atlas_gpu`) — ⚠️ **o oráculo é o PRODUTO ANTIGO**, byte a
  byte: **0 de 27648 bytes diferem**. Um limiar de tolerância aceitaria justamente a classe de erro
  que um deslocamento errado produz (a forma inteira arrastada por alguns texels). A fixture tem
  **cobertura parcial** de propósito — num texel opaco ou vazio um deslocamento errado ainda pode
  acertar por acaso; é na RAMPA da borda que ele se denuncia.
- **6 no empacotador** (headless, sem GPU) — duas células sobrepostas pintam uma forma dentro da
  outra e **o resultado ainda parece arte**, que é o que uma foto não mostra.
- **2 arch-gates de shell** — o `cook_batch` exige janela + device + um renderer do Vello, e nenhum
  teste de unidade o alcança.

**8 mutações, 8 sangram.**

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

## §18 — W11: o GRADIENT MAP — a rampa de N stops

**O que a wave entrega:** um degrau que mapeia a claridade da arte numa rampa de até **8 stops**,
com o trilho de autoria no card (barra de preview, punho por stop, `+`/`−`, e a swatch do stop em
foco pelo MESMO picker OKLCH das duas pontas do Duotone).

### §18.1 — A wave é uma SUBSUNÇÃO, e é isso que a justifica

Um Gradient Map de **dois stops nas pontas é o Duotone ao byte** — medido no dispositivo, **0 de
6144 bytes diferem**, em três opacidades. Duas leis vizinhas na mesma pilha não podem discordar
sobre *"quão claro é este texel"*: o artista teria duas fichas que desenham coisas diferentes com os
mesmos números, e ninguém lê um número numa screenshot.

**A LEI DA RAMPA é a que o app já ship** — paridade com o `gradient_map_lut` do
`ph2d-painter-effects` (a camada de ajuste do Painter, escrita por outra wave, para outro consumidor):
**1 nível de byte** no pior caso, em três rampas × dois modos.

⚠️ **A RÉGUA divergem de propósito, com o número medido.** O Painter mede claridade em **Rec.601
sobre bytes de display**; esta pilha mede em **`L` do OKLab** (a régua que o Duotone e o Luma to
Alpha já usam, e a única perceptualmente uniforme das três que o app carrega). Medido em sRGB 128:
Rec.601 dá **0,502** e o OKLab **0,600**. A paridade afirmada é sobre *"que cor vive na posição
`t`"*, não sobre *"que `t` este pixel tem"*.

### §18.2 — A porta única: o documento guarda a AUTORIA, o consumidor ORDENA

`FxOp::ramp_for_device` ordena uma cópia e empacota nos dois `vec4` do uniform. O documento guarda a
ordem de **autoria**, o que dá **índice estável** a cada alça — arrastar um stop por cima do vizinho
não re-liga o gesto a outro stop. É o precedente do `move_gradient_stop` do Painter, e o gate
`the_authoring_order_of_the_stops_does_not_change_a_byte` o pina.

### §18.3 — As três leis do trilho

| Gesto | Lei | Por quê |
|---|---|---|
| **arrastar** um punho | move só a POSIÇÃO | a cor é da swatch; o índice é o de autoria |
| **`+`** | stop novo no maior **VÃO**, com a cor que a rampa **já tem ali** | ele não pode mudar o desenho — o artista pediu um ponto de controle, não uma edição (é o que Photoshop e Illustrator fazem) |
| **`−`** | remove o em foco, **piso em DOIS** | abaixo de dois a rampa muda de **LEI**, não de forma (§18.5) |

### §18.4 — O que a medição derrubou: quatro notas minhas

1. ⚠️ **`mode 1` num tipo pointwise era varrido para o plano do campo de distância**, e o degrau saía
   **no-op completo** (saída byte-idêntica à fonte, com o Linear correto ao lado). A regra do plano
   perguntava *"tem modos, e escolheu o 1?"* — enumeração disfarçada de regra, **terceira
   instância**: o `1` da turbulência é *Creased* (isento por um `if` à mão), o `1` do Gradient Map é
   *Smooth*. Hoje a pergunta é feita ao TIPO (`FxOp::mode_selects_the_distance_plan`), derivada da
   MESMA declaração de modos que a UI pinta — um falloff novo entra por construção, um vocabulário
   próprio nunca é varrido.
2. ⚠️ **Um Gradient Map novo nascia em SMOOTH** — o `BLANK` compartilhado tem `mode: MODE_CONTOUR`
   (= 1), o default bom da família do falloff, onde `1` significa *Contour*. **Quarta instância da
   mesma doença, agora no DEFAULT.** E o corolário medido: em Smooth o `+` **não pode** ser neutro
   (o easing é por SEGMENTO, dividir um segmento reforma a curva — 25 níveis de byte); em Linear ele
   é neutro ao byte. `mode: 0` agora é **declarado**.
3. ⚠️ **Eu escrevi que a rampa preto→branco era *"a IDENTIDADE em luma"*.** É **falsa**: o `t` sai
   do `L` do OKLab e a mistura acontece em luz LINEAR, então **cinza 129 entra e 204 sai**. Um
   Gradient Map é um **RECOLORIDOR** como o Duotone e o Color Overlay — adicioná-lo muda o desenho
   por desenho (o default do Photoshop também), e o que o default tem de ser é **PREVISÍVEL**.
4. ⚠️ **Eu escrevi que o registro do `CurvePoint` tinha lag de 1 frame.** Não tem — o
   `seed_and_publish` é a **Fase B do MESMO `paint`**.

### §18.5 — O degenerado NÃO é o default, e o gate o PINA

`stop_count == 0` cai no ramo vazio do `gradient_sample` (`srgb_to_linear(t)`, que trata o `t` como
valor de display) e **difere do default de dois stops em 73 níveis de byte** — porque duas pontas
preto→branco linearizam e devolvem `t` em luz linear. **Herdamos as duas leis verbatim do Painter**,
que tem exatamente a mesma descontinuidade. O gate as pina: quem "consertar" um dos ramos para casar
com o outro passa a divergir da crate que é o nosso oráculo. É por isso que o `−` tem piso em dois —
o degenerado é o caso bem-definido de um degrau em branco, nunca um estado de autoria.

### §18.6 — Uma mutação minha sobreviveu, e o defeito era do meu GATE

O readback do picker ramificava dentro do `render_frame` (window-gated), coberto por um **arch-gate
sobre o FONTE**. A mutação que dobrava o stop na ponta escura **manteve o nome
`ColourSlot::SelectedStop` num braço inalcançável** e passou **verde**: uma varredura de fonte vê
FORMA, não comportamento.

**A cura foi mover a decisão, não reforçar o gate:** a rota vive em `fx_live::apply_picked_colour`
— pura, e portanto observável por um gate de unidade (`each_colour_slot_gets_the_picked_colour_and_only_it`,
onde a mutação sangra num `assert_eq!`). O arch-gate ficou com a **costura**, que é o que só o fonte
mostra. ⚠️ E o `bool` do readback (*"é a segunda?"*) virou `ColourSlot` de três — o comentário dele
**já previa esta wave**: *"derivar a ponta do `kind` faria a segunda escrever na primeira em qualquer
tipo que ganhasse uma rampa depois"*.

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

### §18.8 — Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=39 cargo run -p ph2d-host-desktop --release
```

Oito estrelas em quatro pares — a cena **imprime o que montou**. O par 1 é o que carrega a wave (o
Duotone contra a rampa de dois stops: **indistinguíveis**); o 2 traz quatro stops e a MESMA rampa
**fora de ordem**; o 3 é Linear × Smooth; o 4 é a força por-stop e a rampa **posterizada**. E o
passo que fecha: **no painel**, arrastar um punho por cima do vizinho, pintar um stop pela swatch, e
os dois botões.

### §18.9 — O punho não se movia, e o ladrão era outro painel

Report do Enio, duas rodadas: *"não é possível arrastar os pontos de cor"* → *"ainda não posso
mover"*. **Reproduzido pelo diagnóstico** (`PH2D_FX_RAMP_DIAG=1`), e a causa não estava no trilho.

O log disse tudo em três linhas:

```
[ramp] linha 1: barra x=1626 w=240 y=431 · 4 punho(s) · store armado: true
[hero] unhandled event: Click(NodeId(9745551770827700970))
```

O `NodeId` é `vector.filter.stop.1.3` — o punho da direita. Ele **estava** pintado, hit-registrado e
com `InteractiveState::CurvePoint` no store; o `Click` no fim é ruído esperado (o `apply_click` cai
no genérico para um `CurvePoint`). O que **faltava** era o meu `[ramp] painel entregou:` — e não
havia `unhandled event: ValueChanged`, ou seja **alguém consumiu** o `ValueChanged` do trilho.

**O ladrão:** o `route_value_changed` do `ph2d-panel-painter-layers` drenava o stash de arrasto para
**qualquer** `ValueChanged` — tomava o gesto e *só então* procurava a camada dona — e devolvia
`Consumed` mesmo sem achar nenhuma. O `HeroScreen::apply_event` pergunta a **todo** painel do
registry, visível ou não, e para no primeiro `Consumed`: o painel do FX pintava os punhos, o dispatch
calculava a posição nova, e o gesto era engolido por um painel de camadas que **nem estava na tela**.

⚠️ **O `take` é irreversível, e o próprio código já o dizia:** o `motion-params` carregava o
comentário *"Not our editor — put it back is impossible (take)"*. Um canal **global** que se drena
antes de perguntar de quem é o gesto não tem conserto a jusante.

**A cura é estrutural, não um guard a mais:** a pergunta virou **parte da chamada**.
`WidgetStore::take_curve_point_drag()` **não existe mais** — só
`take_curve_point_drag_if(|parent| …)`, que deixa o stash **INTACTO** quando a resposta é não. Os
nove sítios de drenagem passam a nomear o id do próprio editor (conferidos **contra a registração**
do `CurvePoint`, não assumidos), e o compilador recusa quem não responde.

**Gates:** `seam_curve_drag_ownership.rs` (RED-first: nasceu `Consumed`) prova que o arrasto de outro
painel **sobrevive e não é consumido**, com o CONTROLE de que o próprio continua a ser drenado ·
`a_curve_point_drag_is_only_taken_by_the_editor_it_belongs_to` prova que a recusa é
**não-destrutiva** · `architecture_curve_drag_asks_whose_gesture.rs` recusa a **tautologia literal**
(`|_| true`), com controle positivo de sítios lidos.

⚠️ **Duas lições sobre os MEUS gates, e as duas são de medição:**

1. **A quarta condição de UI não é implicada pelas outras três.** Os três gates que eu tinha
   provavam que os punhos são *pintados* e que `+`/`−` *despacham*; **nenhum dirigia o gesto**. E
   quando escrevi o que faltava, um deles — `both_handles_…_are_draggable` — media só a metade do
   **DISPATCH** (o punho responde ao mouse) e **não** que o painel **encaminha**: ele passava com a
   mutação *"o vetor pergunta pela linha errada"* instalada, sobre um punho que não move nada. Agora
   afirma as duas metades, e a mutação sangra nos três.
2. **Uma mutação SOBREVIVEU e está nomeada:** trocar o id da rampa de *textura* do Painter pelo da
   rampa de *Shape* não sangra gate nenhum — a suíte daquele módulo **não cobre o forward do arrasto
   de rampa dele**. É vão pré-existente, de outro dono; os cinco ids que esta wave escreveu lá foram
   verificados **lendo a registração** (`parent: ids.edit`), que é o que se faz quando não há gate.

**Três dívidas latentes dos commits anteriores do trilho, corrigidas junto** — e a família é a mesma
que já custou caro nesta linha: **estes gates moram na `ph2d-editor-core`, então `cargo test -p
<painel>` nunca os roda**. O `paint_filters.rs` estava em **760 > 600 LOC já no HEAD** (split por
responsabilidade: `paint_filters_ramp.rs` = o trilho · `paint_filters_blend.rs` = a lei de mistura) ·
o `no_literal_color` exige o marcador na **MESMA linha** e o meu estava na de cima (logo, vermelho no
HEAD também) · e o `hr12_widgets_a11y` ganhou entrada com a dívida **nomeada**: os punhos de rampa
não têm nó AccessKit — gap pré-existente, compartilhado com o ramp widget do próprio Painter, cujo
único a11y é um `use ph2d_a11y::NodeId` (import de **tipo**, que satisfaz o scanner sem registrar
nada).
