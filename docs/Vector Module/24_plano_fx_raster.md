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

- **W1 — A FORMA FILTRADA (esta):** a orquestração isolar→filtrar→recompor + `VecFilter` +
  **Blur · Outer Glow · Drop Shadow** (ride o blur gaussiano que já existe) + painel + smoke. É o
  Rive-headline e estabelece a costura que todo filtro futuro pluga.
- **W2 — O GRAFO COMPONÍVEL (além do Rive):** color-matrix (tint/aberração/duotone), displacement +
  turbulence (noise procedural), morphology (dilate/erode), lighting/bevel. Passes novos com o
  template `PreviewPremul`; múltiplos filtros compõem numa cadeia (o `Vec<VecFilter>` ou uma pilha
  própria — decisão da W2).
- **W3 — O FEATHER ANALÍTICO (igualar o Rive onde ele é forte):** soft edge resolution-independent
  via erf da distância (o Levien `draw_blurred_rounded_rect` generalizado, ou SDF via o JFA in-repo
  do motion-nodes). O primitivo premium, quando a nitidez em zoom extremo importar.
