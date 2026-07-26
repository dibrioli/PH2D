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
- **W5 — O FEATHER E O BEVEL (FECHADA, pendente de smoke):** os dois tipos que o campo de distância
  da W4 destravou (7 → 9). Ver §10. ⚠️ O feather chegou pelo caminho oposto ao previsto: em vez do
  `erf` analítico de Levien, ele é uma rampa sobre o MESMO campo do JFA — o primitivo já estava
  construído por outro motivo, e a wave inteira coube num braço de shader.
- **W6 — o que ainda pede maquinaria NOVA:** turbulência + deslocamento (o
  `feTurbulence`/`feDisplacementMap`, o eixo ORGÂNICO que ninguém no 2D vetorial entrega bem) e o
  blend mode POR DEGRAU (o Layer Style do Photoshop). Nenhum deles muda a pilha.

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
