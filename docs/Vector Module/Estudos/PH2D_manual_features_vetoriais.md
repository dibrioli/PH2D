# Manual de Implementação — Ferramentas Vetoriais Expressivas para o PH2D

> **Documento técnico / manual para agente de implementação.**
> Objetivo: mapear as features de desenho vetorial que ampliam a expressividade artística (traçado, cor, forma, deformação, não-destrutividade, assistência), identificar as implementações open-source de referência, e para cada uma fixar **um algoritmo principal + um alternativo**, com trechos de código e notas de integração no pipeline do PH2D (Rust / wgpu / lyon / parley, com Vello no chrome de UI).
>
> **Como usar este manual (para o agente):** cada seção é autocontida. Ao implementar uma feature, siga a ordem: (1) leia "O quê / por quê"; (2) escolha entre algoritmo principal e alternativo conforme a tabela de trade-offs; (3) use os projetos de referência como fonte de verdade para casos de borda; (4) siga as "Notas de integração PH2D". Nunca implemente do zero um solver de curva paralela ou um clipping booleano sem antes checar `kurbo` e `Clipper2` — ambos já resolvem os casos de borda difíceis.

---

## 0. Recomendação de stack (leia antes de tudo)

O ecossistema Rust já cobre quase toda a geometria necessária. A decisão arquitetural mais importante para o PH2D é **construir sobre `kurbo` + `lyon` + `Clipper2`**, não reimplementar.

| Necessidade | Crate recomendado | Papel |
|---|---|---|
| Curvas Bézier, arclength, offset, simplificação, **stroke expansion** | **`kurbo`** (linebender) | Núcleo de geometria. Estado da arte em curvas paralelas (Euler spiral). |
| Tesselação para GPU (fill/stroke → triângulos para wgpu) | **`lyon`** | Já no teu stack; alimenta o pipeline wgpu do canvas. |
| Booleanos de polígono + offset robusto (Shape Builder, Live Paint) | **`Clipper2`** (via `clipper2` / `clipper2-rust`) | Vatti robusto com coordenadas inteiras. |
| Vetorização de raster (Image Trace) | **`vtracer`** (visioncortex) | O(n), colorido, saída compacta. Rust puro. |
| Renderização vetorial de referência / chrome | **Vello** | Já pinado no teu ADR de UI. |

**Sinal forte de mercado:** o Graphite (editor vetorial node-based em Rust, o parente arquitetural mais próximo do PH2D) **migrou da própria `bezier-rs` para `kurbo` em 2025**, citando performance e correção superiores às implementações "ingênuas e não-otimizadas" da bezier-rs. Para o PH2D, isso é um atalho validado: adote `kurbo` como camada de curva e reserve implementação própria só onde os shader nodes exigem (o canvas criativo).

> ⚠️ **Distinção de pipeline crítica para o PH2D:** as features de *chrome/UI* podem sair prontas do Vello. As features do *canvas criativo* — onde o artista autora shaders arbitrários via nodes — precisam do pipeline wgpu bespoke. A regra prática: se a feature produz **geometria** (path de saída), ela pertence à camada `kurbo`/`lyon` e é agnóstica ao renderer; se produz **aparência** (pixels, cor, textura procedural), ela pertence ao pipeline wgpu. Muitas features abaixo são geométricas e podem ser implementadas uma vez e renderizadas por qualquer backend.

---

## 1. Traçado expressivo

### 1.1 Largura variável (Variable-Width Stroke / "PowerStroke")

**O quê:** o stroke ganha espessura modulada ao longo do path — pares `(posição, largura)` interpolados, gerando a sensação de pincel/nanquim. É a ponte mais direta entre gesto e vetor.

**Referência open-source:** **Inkscape — LPE PowerStroke** (`src/live_effects/lpe-powerstroke.cpp`, autor Johan Engelen). O mecanismo LPE guarda o path original em `inkscape:original-d`, aplica a matemática (via lib2geom) e escreve o resultado no atributo `d`. A largura é armazenada como lista de pares `(location, width)`; onde mudam, interpola suavemente entre eles.

O modelo de dados essencial (reconstruído a partir da implementação do Inkscape): três pontos de controle default, no início/meio/fim, cada um `Point(posição_ao_longo_do_path, largura)`:

```cpp
// Inkscape lpe-powerstroke.cpp — inicialização dos offset_points (paráfrase)
double width = style ? style->stroke_width.computed : 1.0;
Geom::Path const &path = pathv.front();
auto size = path.size_default();
points.push_back( Geom::Point(0.,        width) ); // início
points.push_back( Geom::Point(0.5*size,  width) ); // meio
if (!path.closed())
    points.push_back( Geom::Point(size,   width) ); // fim
```

O interpolador default do Inkscape moderno é **Centripetal Catmull-Rom** (evita overshoot/self-intersection que o Catmull-Rom uniforme produz); há também o "CubicBezierJohan" com parâmetro `beta` (0 = linear, 1 = suave). Junções ganham dois tipos novos além de bevel/round/miter: **Spiro** (arredondado, baseado nas curvas Spiro de Raph Levien) e **Extrapolated** (miter que segue melhor a trajetória de uma pena).

**Algoritmo principal — Stroke Expansion via Euler Spiral (kurbo / Levien-Uguray 2024).**
O output de uma largura variável é uma *curva paralela* (offset) de cada lado do esqueleto, com a distância de offset variando ponto a ponto. O problema difícil é gerar essas curvas paralelas com cúspides corretas e sem auto-interseção. O método de Levien & Uguray ("GPU-friendly Stroke Expansion", 2024, implementado em `kurbo`) usa **segmentos de Euler spiral como representação intermediária**: aproxima o Bézier de origem por spirais de Euler dentro de uma tolerância, e então calcula as curvas paralelas de cada spiral (que têm forma analítica tratável), baixando para segmentos de linha ou arco — ambos ideais para GPU. Cúspides aparecem exatamente onde o raio de curvatura iguala o offset; o método as detecta e subdivide ali.

```rust
// kurbo: expandir um esqueleto em outline preenchível
use kurbo::{stroke, Stroke, BezPath};
let skeleton: BezPath = /* path do gesto, já fitado */;
let style = Stroke::new(1.0)        // largura base; escale por-ponto para variável
    .with_caps(kurbo::Cap::Round)
    .with_join(kurbo::Join::Round);
let tolerance = 0.1;
let outline: BezPath = stroke(&skeleton, &style, &Default::default(), tolerance);
// `outline` é um path fechado → mande para lyon tessellar → wgpu.
```

Para largura *variável* (não uniforme), a estratégia é: amostrar o esqueleto por arclength, avaliar `width(s)` a partir dos pares de controle (interpolação Catmull-Rom centrípeta), gerar as duas bordas por deslocamento na normal, e fitar Béziers nas bordas (Seção 1.3). `kurbo` provê arclength e normais; a expansão paralela por Euler spiral cuida dos cantos.

**Algoritmo alternativo — Offset por amostragem de normais + refit (o método "clássico" do Inkscape/Graphics Gems).**
Mais simples de implementar e depurar: avalie o path em `t` uniformemente espaçados, para cada ponto calcule tangente e normal normalizadas, escale a normal pela largura local, construa segmentos de linha e **elimine pontos cujos segmentos se auto-intersectam** (remove overlaps em curvas apertadas); então refite um Bézier peça-a-peça sobre os pontos deslocados usando as tangentes exatas. É O(n) e robusto o suficiente para uso interativo; perde em qualidade de canto e mínimo de segmentos frente ao método Euler spiral.

**Trade-off:**

| | Euler spiral (kurbo) | Amostragem+refit |
|---|---|---|
| Qualidade de cúspide/canto | Excelente | Média |
| Nº de segmentos de saída | Mínimo | Maior |
| Facilidade de implementar | Usa crate pronto | Trivial do zero |
| GPU-friendliness | Alta (arcos/linhas) | Média |

**Notas de integração PH2D:** armazene o par `(location, width)` como dado do node não-destrutivo (não "queime" no path). O esqueleto permanece editável; a expansão roda no recompute do node graph. Para input de tablet, alimente `width(s)` diretamente da **pressão do Wacom/Apple Pencil** — foi exatamente a extensão que os autores do PowerStroke marcaram como "future work" e que tu podes entregar nativamente. Suporte largura assimétrica (esquerda ≠ direita) desde o início: guarde dois arrays de offset; o Inkscape só adicionou isso tarde e usuários pediam muito.

---

### 1.2 Pincéis vetoriais (Art / Pattern / Scatter brushes) — "Skeletal Strokes"

**O quê:** um traço aplica *arte* (um path, ou um pattern repetido) ao longo do caminho, deformando-a para seguir a curvatura. É o mecanismo por trás de art brushes, borders decorativos e texturas orgânicas.

**Referência open-source:** **Inkscape — LPE "Pattern Along Path"** e **"Bend"**. Modelo mental do Inkscape:

```
original path ──► LPE ──► output path
                   ▲
                   │ parâmetros (o "esqueleto" ou o "pattern")
```

- **Pattern Along Path:** o *pattern* é o path fixo, o *skeleton* é o parâmetro — o pattern é distribuído/repetido ao longo do esqueleto. Modos: `Single`, `Single stretched`, `Repeated`, `Repeated stretched`.
- **Bend (Path Along Path):** o path a dobrar é o original, e o *bend path* é o parâmetro — dobra a arte ao longo da curva.

**Algoritmo principal — Skeletal Strokes (Hsu & Lee, 1994): mapeamento por arclength + deslocamento na normal (frame de Frenet).**
Defina um sistema de coordenadas local ao longo do esqueleto: para o parâmetro de arclength `s`, tenha o ponto `P(s)`, a tangente unitária `T(s)` e a normal `N(s)`. Um ponto do pattern com coordenadas `(u, v)` (u = ao longo, v = transversal) mapeia para:

```
world(u, v) = P(u) + v · N(u)
```

onde `u` é reparametrizado por arclength (não por `t` da Bézier — são não-lineares entre si!). Isso "envelopa" o pattern na curva. O passo crítico e propenso a bug é a **reparametrização arclength→t**: pré-compute uma LUT (lookup table) de arclength cumulativa amostrando o Bézier, e inverta por busca binária + Newton.

```rust
// Esqueleto do algoritmo (pseudo-Rust sobre kurbo)
use kurbo::{ParamCurve, ParamCurveArclen, ParamCurveDeriv, Point, Vec2};

fn map_onto_skeleton(skel: &impl ParamCurveArclen, u: f64, v: f64, total_len: f64) -> Point {
    let t = skel.inv_arclen(u * total_len, 1e-3); // arclength → t (kurbo faz isso)
    let p = skel.eval(t);
    let tangent = skel.deriv().eval(t).to_vec2().normalize();
    let normal = Vec2::new(-tangent.y, tangent.x);  // rotação 90°
    p + normal * v
}
```

Para *repetição*, particione `[0, total_len]` em cópias e mapeie cada instância; para *stretch*, escale o eixo `u` do pattern para o comprimento disponível. Cuidado com o bug histórico do Inkscape: `floor(nCopies)` dá zero cópias quando o esqueleto é mais curto que o pattern — use `max(1, round(nCopies))`.

**Algoritmo alternativo — Warp por Coons/bilinear patch (envelope 2D).**
Em vez do frame de Frenet 1D, defina uma faixa (ribbon) 2D com quatro bordas Bézier (duas laterais = offsets do esqueleto, duas pontas) e faça um mapeamento bilinear/Coons do bounding box do pattern para dentro dessa faixa (ver Seção 5.1). Vantagem: lida melhor com largura variável do brush e com deformação em curvas fechadas; desvantagem: mais caro e pode dobrar (foldover) em curvas muito apertadas.

**Notas de integração PH2D:** este é o **art brush** que o artista solo espera do Procreate/Illustrator. Combine com 1.1: a largura variável define a faixa, o pattern preenche. Para *scatter brushes*, não deforme — apenas instancie o pattern em posições amostradas ao longo do path com jitter de posição/escala/rotação (mais barato, ótimo para folhagem/partículas estáticas). Guarde o brush como asset reutilizável referenciado pelo node, não copiado.

---

### 1.3 Curva suavizada / Pencil (fit de gesto → Bézier)

**O quê:** desenho à mão livre entra como uma nuvem densa de pontos (mouse/tablet); precisa virar poucos Béziers editáveis, com suavização ajustável, sem o artista tocar em handles.

**Referência open-source:** `burningmime/curves` (C#), `soswow/fit-curve` (JS), `volkerp/fitCurves` (Python) — todos implementam o mesmo clássico.

**Algoritmo principal — Schneider, "An Algorithm for Automatically Fitting Digitized Curves" (Graphics Gems, 1990).**
Pipeline em três passos:

1. **Pré-simplificação (Ramer-Douglas-Peucker):** remove pontos redundantes da polyline crua.
2. **Fit por mínimos quadrados:** com parametrização por comprimento de corda (`chord-length`), monte o sistema de Bernstein e resolva os pontos de controle que minimizam o erro quadrático. Se o determinante é zero ou os alphas ficam negativos, caia na heurística Wu/Barsky: `P1 = P0 + tan_esq·(len/3)`, `P2 = P3 + tan_dir·(len/3)`.
3. **Refinamento iterativo:** calcule a distância máxima ponto→curva; se exceder a tolerância, faça **reparametrização Newton-Raphson** (ajusta os `t` dos pontos) e refite; se ainda ruim, **subdivida no ponto de erro máximo** e fite recursivamente cada metade (impõe continuidade C1/G1 no ponto de split).

```rust
// Interface-alvo (existem crates; ou porte o Graphics Gems)
fn fit_curve(points: &[Point], max_error: f64) -> Vec<CubicBez>;
// internamente: rdp() → chord_length_param() → generate_bezier()
//               → compute_max_error() → reparameterize() (Newton) → recurse
```

**Algoritmo alternativo — total least squares / weighted least-squares com detecção de cantos (Yang et al.).**
Primeiro identifica pontos significativos e os classifica em **cantos** (descontinuidade) vs **junções** (suaves); então faz o fit ponderado forçando G1 nas junções e deixando cantos livres. Melhor para preservar cantos intencionais do gesto (útil em lettering/inking), ao custo de um passo extra de classificação. Alternativa moderna: usar o `kurbo::simplify` / `fit_to_bezpath` de Raph Levien, que chega perto do ótimo global em nº de segmentos.

**Notas de integração PH2D:** exponha a tolerância como "smoothing" na UI (um slider), não como valor mágico. Faça o fit *incremental* durante o traço (fit da cauda enquanto desenha) para feedback ao vivo — crucial na sensação de tablet. Alimente a pressão em paralelo como canal de largura (Seção 1.1), fitando `width(s)` com o mesmo esquema.

---

## 2. Cor e preenchimento

### 2.1 Gradient Mesh / Freeform gradient (Coons patch mesh)

**O quê:** malha de retalhos onde cada retalho tem cores nos 4 cantos e bordas Bézier; permite sombreamento pictórico/volumétrico dentro do vetor (o recurso mais "pintura" dos apps vetoriais).

**Referência open-source:** **Inkscape mesh gradients** (implementação de Tavmjong Bah, base do SVG2), e o excelente writeup **Rasterific (Haskell)** de renderização de Coons/tensor patches. Cairo também suporta (Type 6/7 shading do PostScript/PDF).

**Estrutura de dados:** um **Coons patch** = quadrilátero definido por 4 curvas Bézier cúbicas (12 pontos de controle) + 4 cores nos cantos. A cor de um ponto interior vem de uma **interpolação bilinear das cores dos cantos**, seguida do mapeamento geométrico definido pelos lados Bézier. Um **tensor patch** adiciona 4 pontos de controle internos (mais controle da geometria).

**Algoritmo principal — subdivisão recursiva de Coons patch (Yao & Rokne 1991) + rasterização por tiras de Bézier.**
Renderize subdividindo o patch em sub-patches até que cada um seja aproximadamente plano em cor e forma (aprox. um quad de cor quase constante), então preencha. A subdivisão de um Coons patch reusa o split de Bézier nas bordas; a contribuição do termo bilinear `S_B` é reinterpolada. Interpolação de cor:

- **Bilinear:** só depende das cores dos cantos. Simples, mas produz **Mach banding** (o olho é sensível à descontinuidade da derivada na fronteira entre patches).
- **Bicúbica:** usa também as derivadas de cor (por canto, ao longo de u e v), escolhidas para transição suave entre patches e sem extremos internos (interpolação monótona). Elimina o banding.

```haskell
-- Rasterific (paráfrase): média dos 4 pesos de cor do patch
meanValue :: ParametricValues (V2 CoonColorWeight) -> V2 CoonColorWeight
meanValue = (^* 0.25) . sumValues
-- Renderização: interpola ao longo das 4 Béziers numa direção,
-- rasteriza a curva formada pelos pontos correntes → cor final.
```

**Algoritmo alternativo — sombreamento Gouraud sobre malha de triângulos + tesselação fina.**
Triangule cada patch densamente e faça interpolação Gouraud (linear por triângulo) das cores nos vértices. Muito mais simples e **mapeia direto no pipeline wgpu** (é literalmente vertex colors + rasterização). Um Coons patch pode simular Gouraud e vice-versa. Trade-off: precisa de malha fina para esconder o banding, gastando mais triângulos; mas para GPU isso é barato e é a rota pragmática para o PH2D.

**Notas de integração PH2D:** **esta feature é o teu diferencial "pintura dentro do vetor".** Recomendação: represente a mesh como dado editável (grade de patches com handles Bézier), mas **renderize via wgpu com Gouraud + subdivisão adaptativa** em vez de um rasterizador de Coons na CPU. Como o teu canvas já tem shader nodes, um shader de mesh gradient (avaliação bicúbica no fragment) é natural e resolve o Mach banding sem explodir a contagem de triângulos. Isso é algo que Vello/Skia *não* fazem bem — justamente por isso o pipeline bespoke se paga aqui.

---

### 2.2 Live Paint / bucket por região

**O quê:** preencher regiões visualmente fechadas por interseções de traços, ignorando a estrutura de path subjacente (o artista vê "áreas", não "caminhos").

**Referência open-source:** **Inkscape** — a ferramenta *Paint Bucket* (flood fill no render rasterizado) e o *livarot*/planar map para a versão vetorial. Conceitualmente é a construção de um **planar subdivision** (arrangement) de todos os traços.

**Algoritmo principal — Planar map / arrangement de segmentos (subdivisão planar).**
1. Colete todos os segmentos/curvas do desenho.
2. Compute **todas as interseções** (sweep-line de Bentley-Ottmann) e quebre as curvas nos cruzamentos.
3. Construa a estrutura DCEL (doubly-connected edge list): vértices, meia-arestas, **faces**.
4. Cada *face* fechada é uma região preenchível. O clique do usuário faz point-location para achar a face; atribuir cor à face é não-destrutivo (a cor liga à região, não ao path).

**Algoritmo alternativo — flood fill em raster + revetorização da fronteira.**
Rasterize o desenho num buffer, faça um **scanline flood fill** a partir do pixel clicado até bater em traços (com tolerância de gap para "fechar" fendas), e então **revetorize a fronteira da região** (Potrace/marching squares → Seção 6). É o que o Paint Bucket do Inkscape faz. Muito mais simples e robusto a traços que "quase" fecham; desvantagem: resolução-dependente e a fronteira é aproximada. Ótimo default interativo; ofereça o planar-map como modo "preciso".

**Notas de integração PH2D:** para um app com GPU sempre presente, o flood fill pode rodar como **compute shader** (jump flooding / scanline no GPU) sobre o framebuffer do canvas — rápido e natural no wgpu. Guarde a região preenchida como um node que referencia os traços-fronteira, para que editar o traço reflua a cor (o "live" de Live Paint).

---

### 2.3 Recolor / Global swatches

**O quê:** reharmonizar toda a arte de uma vez; cores nomeadas ("global") ligadas, de modo que mudar o swatch propaga por todas as instâncias.

**Algoritmo principal — indireção por tabela de cores (color table / palette indirection).**
Não guarde RGBA nos objetos; guarde um **índice/ID de swatch**. O objeto resolve a cor via `palette[swatch_id]` no momento do render. "Recolor global" = editar uma entrada da tabela → todos os que apontam para ela mudam. Para "recolor" temático (reharmonização global tipo Illustrator Recolor Artwork), mapeie a paleta atual para um novo esquema preservando relações (matiz relativo, ordenação por luminância).

**Algoritmo alternativo — extração de paleta + remapeamento por clustering.**
Extraia a paleta efetiva da arte com **k-means/median-cut** no espaço de cor perceptual (OKLab, não RGB), agrupe cores próximas, e ofereça transformações de harmonia (rotação de matiz, mudança de temperatura) aplicadas por cluster. Necessário quando a arte *não* usa swatches (ex.: imagem traçada). OKLab é a escolha correta para distâncias perceptuais.

**Notas de integração PH2D:** adote **indireção de swatch desde o início** no modelo de dados — é barato e habilita recolor, temas (claro/escuro), e "variables/modes" tipo Figma (Seção 8). Faça toda mistura/interpolação de cor em **espaço linear ou OKLab**, nunca em sRGB gamma — erro comum que suja gradientes.

---

## 3. Construção de forma

### 3.1 Shape Builder (booleanos interativos por arrasto)

**O quê:** combinar/subtrair formas passando o cursor por cima das regiões — muito mais fluido que um Pathfinder booleano tradicional. Por baixo é união/interseção/diferença de polígonos, mas a UX opera em *regiões do arrangement*.

**Referência open-source:** **Clipper2** (Angus Johnson) — a lib de clipping mais robusta e usada; portes Rust: `clipper2` (FFI via `clipper2c-sys`) e `clipper2-rust` (Rust puro, feature-complete, 444 testes). Alternativa acadêmica: implementações Martínez-Rueda (`rust-geo-booleanop`).

**Algoritmo principal — Vatti (Clipper2).**
Sweep-line que suporta polígonos complexos auto-intersectantes, múltiplas fill rules (EvenOdd, NonZero, Positive, Negative) e produz hierarquia de buracos (PolyTree). Usa **coordenadas inteiras internamente** para robustez numérica (o `f64` da API é escalado para `i64`).

```rust
use clipper2::*;
let a: Paths = vec![(0.2,0.2),(6.0,0.2),(6.0,6.0),(0.2,6.0)].into();
let b: Paths = vec![(5.0,5.0),(8.0,5.0),(8.0,8.0),(5.0,8.0)].into();
let out = a.to_clipper_subject().add_clip(b).difference(FillRule::NonZero)?;
```

**Algoritmo alternativo — Martínez-Rueda (sweep-line de subdivisão).**
Também sweep-line, `O((n+k) log n)`, lida com buracos e múltiplos contornos, preservando topologia (hierarquia buraco/contorno). Costuma ser mais fácil de portar/entender e trabalha em ponto flutuante direto (sem escala inteira), mas historicamente menos endurecido contra casos degenerados que o Clipper2. Bom quando não se quer a dependência C++.

**Nota importante sobre curvas:** ambos operam em **polígonos** (segmentos de reta). Para paths com Béziers: **flatten** as curvas para polylines na tolerância desejada (kurbo/lyon fazem isso), rode o booleano, e **refite Béziers** no resultado (Seção 1.3) se quiser saída vetorial suave. Aceitar essa "ida e volta" flatten→boolean→refit é o padrão da indústria.

**Notas de integração PH2D:** a UX do Shape Builder = construir o **arrangement** (Seção 2.2) de todas as formas selecionadas uma vez, e conforme o cursor arrasta sobre faces, marcar faces como "add" ou "subtract"; no soltar, unir as faces marcadas via Clipper2. Guarde o resultado como node booleano não-destrutivo (as formas-fonte permanecem editáveis).

---

### 3.2 Blend / Interpolação (morphing e arrays graduais)

**O quê:** interpola entre duas formas (morph) e gera arrays intermediários graduais — sombreamento por steps, transições, repetição com variação.

**Referência open-source:** **Inkscape — LPE Interpolate** (interpola *dados de path*, não estilo). Nota do design do Inkscape: LPE só produz geometria; cor/gradiente entre blends precisa de tratamento separado.

**Algoritmo principal — correspondência de nós + interpolação linear de pontos de controle.**
1. **Normalize a topologia:** as duas formas precisam do mesmo nº de nós; insira nós (subdividindo Béziers sem mudar a forma) na de menos até igualar, e **case os nós** (alinhamento por ângulo/posição, tipicamente minimizando distância total ou por arclength).
2. Para cada step `α ∈ [0,1]`, interpole linearmente cada ponto de controle correspondente: `P_i(α) = (1-α)·A_i + α·B_i`.
3. Cor/atributos interpolam em paralelo (em OKLab).

**Algoritmo alternativo — interpolação intrínseca (ângulo/comprimento) para morphs sem "afundar".**
Interpolação linear de pontos pode fazer a forma encolher/cruzar no meio. A interpolação intrínseca representa cada aresta por `(comprimento, ângulo de virada)` e interpola *esses*, reconstruindo a forma — preserva rigidez e evita colapso. Mais complexo, mas dá morphs muito melhores para formas rotacionadas/dobradas. (Família "as-rigid-as-possible" shape interpolation.)

**Notas de integração PH2D:** o blend é um gerador procedural perfeito para um **node** ("blend node": entradas A, B, N steps) — casa com tua arquitetura de graph. Reuse o casamento de nós do morph para animação/tweening (é o mesmo problema do "Smart Animate" do Figma, Seção 8).

---

### 3.3 Offset path não-destrutivo (contornos paralelos)

**O quê:** gerar contornos paralelos automáticos (inflar/desinflar) — usado para outlines, molduras, e como primitiva de muitos efeitos.

**Algoritmo principal — curvas paralelas via Euler spiral (kurbo).**
Mesmo motor da Seção 1.1, mas offset constante. `kurbo` implementa o método de Levien: aproxima por spirais de Euler e gera a paralela com cúspides corretas. Para offset de *polígono* (não curva), **Clipper2 `InflatePaths`/`ClipperOffset`** com `JoinType` (Miter/Square/Bevel/Round) e `EndType`.

```rust
// Curva: kurbo
use kurbo::offset::CubicOffset;
let offset_path = kurbo::fit_to_bezpath(&CubicOffset::new(cubic, distance), 1e-3);

// Polígono: Clipper2
let inflated = paths.inflate(2.0, JoinType::Round, EndType::Polygon, 0.0);
```

**Algoritmo alternativo — offset por dilatação de campo de distância (SDF).**
Rasterize a forma num **signed distance field**, e o offset vira um simples threshold do SDF (`sdf < d`); revetorize a iso-linha por marching squares. Trivial de fazer em GPU e naturalmente robusto a auto-interseções (elas somem no SDF). Perde precisão vetorial e é resolução-dependente, mas para efeitos visuais em tempo real (glow, outline animado) é imbatível — e cai perfeitamente no teu pipeline wgpu.

**Notas de integração PH2D:** ofereça os dois: offset vetorial exato (kurbo/Clipper2) para geometria "cozida", e offset SDF em shader para preview/efeito ao vivo. Cúspides: quando `raio_de_curvatura == distância`, aparece cúspide — não ignore, o kurbo trata; um offset ingênuo por normais gera loops.

---

## 4. Simetria e repetição

### 4.1 Simetria em tempo real (radial / espelho / grade) e Repeat objects

**O quê:** mandalas, padrões e estruturas simétricas geradas **ao vivo** enquanto se desenha — cada traço é replicado pelas transformações de simetria imediatamente.

**Referência open-source:** **Inkscape — Tiled Clones** (`Edit > Clone > Create Tiled Clones`, com 17 grupos de simetria de wallpaper: P1, P2, PM, PG, CM, PMM, ... P6M) e a **LPE Mirror symmetry / Rotate copies**. Krita tem multibrush/mirror ao vivo como referência de UX.

**Algoritmo principal — grupo de transformações + instanciamento (clones referenciais).**
Modele a simetria como um **conjunto de matrizes de transformação** (o grupo). Cada objeto "fonte" é referenciado por N clones, cada um com sua matriz. No desenho ao vivo, todo ponto de input `p` gera `{ M_k · p }` para cada `M_k` no grupo. Para simetria radial de ordem n: `M_k = Rot(2πk/n)` em torno do centro; espelho: `M = Reflect(eixo)`; grade de wallpaper: gerador de translações + rotações/reflexões do grupo cristalográfico.

```rust
// Simetria radial ao vivo
fn symmetry_transforms(order: u32, center: Point) -> Vec<Affine> {
    (0..order).map(|k| {
        let a = std::f64::consts::TAU * k as f64 / order as f64;
        Affine::translate(center.to_vec2())
            * Affine::rotate(a)
            * Affine::translate(-center.to_vec2())
    }).collect()
}
// aplique cada transform ao stroke corrente antes de tessellar
```

**Algoritmo alternativo — instanced rendering na GPU (sem duplicar geometria).**
Em vez de materializar N cópias do path, **desenhe a geometria uma vez com instancing** no wgpu, passando as matrizes de simetria como buffer de instâncias. O vertex shader aplica `M_instance`. Escala para centenas de cópias (mandalas densas) praticamente de graça, e o "ao vivo" fica trivial (só atualiza o buffer). Trade-off: as cópias não são editáveis individualmente até "expandir" (materializar) — o que geralmente é o comportamento desejado.

**Notas de integração PH2D:** para o artista de tablet, **simetria radial/espelho ao vivo é um feature de deleite** (Krita/Procreate provam). Implemente via instancing GPU para o preview e permita "expandir para clones editáveis" ou "cozinhar em path único" (via união Clipper2) sob demanda. Guarde o grupo de simetria como um modificador de node.

---

## 5. Deformação

### 5.1 Envelope / Mesh warp / Puppet warp

**O quê:** dobrar, curvar e distorcer arte mantendo editabilidade (envelope de grade, warp por malha, e deformação por "pinos" estilo puppet).

**Algoritmo principal (envelope) — warp bilinear / Coons patch.**
Defina uma grade de controle (2×2 para bilinear, ou 4×4 Béziers para Coons). Cada ponto da arte em coordenadas normalizadas `(u,v)` no bounding box mapeia para dentro do envelope deformado pela mesma matemática de Coons da Seção 2.1 (só que aqui deformando *geometria*, não interpolando *cor*). Referência: "Bilinear Coons Patch Image Warping" (Heckbert, Graphics Gems IV).

**Algoritmo principal (puppet) — Moving Least Squares (MLS) ou As-Rigid-As-Possible (ARAP).**
Para deformação por pinos (o usuário crava âncoras e arrasta): **MLS deformation** (Schaefer et al., "Image Deformation Using Moving Least Squares") calcula, para cada ponto, a transformação rígida/afim que melhor casa o movimento das âncoras ponderado por `1/dist²`. ARAP resolve um sistema que preserva rigidez local — a base de rigging de malha (Rive/Live2D usam parentes disso).

```
// MLS rígido (esboço): para ponto v, âncoras p_i → q_i, pesos w_i = 1/|p_i - v|^(2α)
// 1. centróides ponderados p*, q*
// 2. monte matriz de rotação ótima (Procrustes ponderado)
// 3. v' = (v - p*) · R + q*
```

**Algoritmo alternativo — Free-Form Deformation (FFD) por lattice Bézier.**
Encaixe a arte num lattice de controle Bézier/B-spline 3D→2D; mover pontos do lattice deforma tudo dentro por avaliação polinomial. É o "Envelope Distort" clássico do Illustrator. Mais previsível que MLS para deformações globais suaves; menos natural para "puxar um ponto".

**Notas de integração PH2D:** deformação de malha sobre arte vetorial é **exatamente a ponte para o rigging estilo Rive** que aparece na tua tese (reduzir round-trip arte→engine). Implemente warp como node não-destrutivo (guarda a grade de controle; a arte-fonte fica intacta). Para performance, a avaliação do warp pode ir para o vertex shader (deformação na GPU), essencial se for animar em runtime.

### 5.2 Roughen / Wrinkle / Zigzag (irregularidade orgânica)

**O quê:** adicionar irregularidade controlada a formas rígidas — bordas rústicas, tremor de mão simulado, zigzag.

**Algoritmo principal — deslocamento por ruído coerente (Perlin/Simplex) ao longo da normal.**
Amostre o path por arclength; em cada amostra desloque na direção da normal por `amplitude · noise(s · frequência)`. Ruído coerente (Simplex) dá irregularidade *orgânica* (contínua), diferente de random puro (que dá serrilhado). Reinsira nós nas amostras e refite. Roughen do Illustrator = detalhe (frequência) + tamanho (amplitude), com modo "smooth" (ruído) vs "corner" (dentes).

**Algoritmo alternativo — subdivisão fractal (midpoint displacement).**
Recursivamente subdivida cada segmento e desloque o ponto médio por um valor aleatório que decai com a profundidade (como geração de terreno fractal). Dá bordas com auto-similaridade em múltiplas escalas; controle por dimensão fractal. Mais barato que amostrar ruído denso, ótimo para "coastline"/rachaduras.

**Notas de integração PH2D:** como é deslocamento na normal, reusa a infra de offset/normais (Seção 3.3). Seed determinístico por objeto para o efeito ser estável entre recomputes (nada pior que "roughen" que tremula a cada frame sem querer). Node não-destrutivo com `seed`, `amplitude`, `frequency`.

---

## 6. Assistência — Image Trace / Auto-vetorização

**O quê:** ponte rápida do rascunho raster (foto do caderno, iPad sketch) para vetor editável.

**Referência open-source:** **Potrace** (Peter Selinger, B&W) e **VTracer** (visioncortex, **Rust**, colorido, O(n)). VTracer é a escolha para o PH2D: Rust puro, aceita scans coloridos de alta resolução, saída compacta (estratégia de *stacking* evita shapes com buracos), e lida tanto com fotos quanto pixel art.

**Algoritmo principal — Potrace: decomposição em paths → polígono ótimo → suavização.**
1. Decompõe o bitmap em paths que formam as fronteiras B/W (vértices nos cantos entre 4 pixels de cor diferente).
2. Para cada path, acha o **polígono ótimo** — critério de otimalidade = **mínimo nº de segmentos**.
3. Converte o polígono numa **outline Bézier suave** (corrige vértices para casar o bitmap, junta segmentos Bézier consecutivos quando possível).
4. Recursão: remove o path fechado invertendo cores dentro dele e repete até não sobrar preto.

O fit de polígono do Potrace é `O(n²)`.

**Algoritmo principal alternativo — VTracer: clustering hierárquico + fit O(n).**
1. **Clustering de cor** (agrupamento hierárquico de regiões conexas; parâmetro de similaridade por distância Euclidiana em RGB — idealmente OKLab).
2. **Stacking:** empilha camadas por cor evitando buracos (saída mais compacta que o Image Trace do Illustrator).
3. **Curve fitting O(n)** com modos `pixel` (sem fit, retro/pixel art), `polygon` (arestas retas), `spline` (Béziers suaves — default para 95% dos casos). Suavização por *subdivide iterativo* até segmentos < `segment_length`; *splice_threshold* controla o ângulo mínimo para emendar splines.

```bash
# VTracer como referência de parâmetros (também é lib Rust)
vtracer -i sketch.png -o out.svg -m spline --preset photo
# em Rust: use o crate `vtracer` diretamente no pipeline de import
```

**Notas de integração PH2D:** integre `vtracer` como **crate no import pipeline** — o artista fotografa/importa um sketch e recebe paths editáveis (fecha o round-trip papel→engine, alinhado à tese). Exponha os 3 modos (`pixel`/`polygon`/`spline`) porque o pixel-art vira nicho forte no público iPad. Faça o clustering em **OKLab** em vez de RGB para separação perceptual melhor. Ofereça "trace ao vivo" com preview conforme o usuário mexe nos thresholds.

---

## 7. Não-destrutivo (o núcleo arquitetural) — Appearance Stack / Live Effects

**O quê:** o fio que amarra tudo. Illustrator (**Appearance panel**) e Affinity (**live filters/effects**) permitem empilhar múltiplos fills, strokes e efeitos num mesmo objeto, reordenáveis, **todos permanentemente editáveis** — um pipeline procedural sobre o vetor. É *literalmente* a lacuna "pintura ↔ ferramenta técnica" que o PH2D ocupa.

**Referência open-source:** **Inkscape — LPE stack.** Mecanismo:
```
original style ─────────────► output style
original path ──► LPE₁ ──► LPE₂ ──► ... ──► output path
                    ▲         ▲
                 params    params
```
O original é preservado (`inkscape:original-d`); efeitos são **encadeados em série**; cada LPE consome um path e emite um path. Limitação do modelo do Inkscape: LPE só transforma *geometria*, não estilo — o PH2D deve superar isso (ver abaixo).

**Algoritmo principal — DAG de nodes com avaliação lazy + cache por node.**
Isto **já é a arquitetura de node graph do PH2D** — e é a decisão certa. Cada feature deste manual vira um **node**:
- **Nodes geométricos** (path→path): PowerStroke, Offset, Roughen, Warp, Boolean, Blend, Pattern-along-path.
- **Nodes de aparência** (geometria→pixels): fills, strokes, gradient mesh, shader nodes, filtros.
- Ordem = ordem no grafo; reordenar = reconectar. Edição de um parâmetro invalida só o subgrafo a jusante (cache dos nodes a montante permanece).

Diferente do Inkscape, o PH2D **não deve separar geometria de estilo rigidamente**: como o canvas tem shader nodes, um node pode emitir tanto path quanto aparência. É o teu diferencial sobre o LPE stack.

```rust
// Modelo mínimo de node não-destrutivo
trait VectorNode {
    fn eval(&self, ctx: &mut EvalCtx, inputs: &[Value]) -> Value; // path e/ou aparência
    fn params(&self) -> &Params;      // editáveis, dirigem re-eval
    fn cache_key(&self) -> Hash;      // memoização; invalida só o necessário
}
// O grafo é um DAG; recompute topológico com dirty-propagation a jusante.
```

**Algoritmo alternativo — pilha linear de modificadores (modifier stack estilo Blender).**
Se o DAG completo for demais no início, uma **pilha linear** de modificadores por objeto (como o Modifier Stack do Blender ou os Live Filters do Affinity) entrega 90% do valor com muito menos complexidade: lista ordenada de efeitos, cada um `f(input) → output`, reavaliada de cima a baixo, com cache do topo estável. Migra para DAG depois sem quebrar o modelo de dados (a pilha é um DAG linear degenerado).

**Notas de integração PH2D:** esta seção **não é uma feature, é o esqueleto**. Todas as anteriores devem ser projetadas como nodes desde o dia 1: `(params editáveis) + (função pura path/appearance) + (cache key)`. Isso garante a promessa "permanentemente editável" que define a categoria e diferencia o PH2D de exportadores de sprite (Rive) e de pintura raster (Procreate). Combine com a indireção de swatch (2.3) e o rigging de warp (5.1) e tens o pipeline "vetor pintado → riggado → dirigido → tocado no engine" sem sair do canvas.

---

## 8. Ponte para Figma/Rive (features de sistema — resumo para roadmap)

Não são "desenho" no sentido estrito, mas caem no mesmo modelo de node/DAG e valem como fase 2:

- **Vector networks (Figma):** substitua o path hierárquico (sequência fechada) por um **grafo de vértices/arestas** onde uma aresta pode ramificar. Edição topológica mais livre; é a feature mais difícil de replicar de todo o Figma. Modele como half-edge/DCEL (mesma estrutura do arrangement da Seção 2.2).
- **Components/variants + Variables/modes (Figma):** instâncias com props + tokens de design resolvidos por contexto (tema/estado). Cai direto na indireção de swatch (2.3) generalizada para qualquer propriedade.
- **State machines + data binding (Rive):** grafo de comportamento com inputs (bool/number/trigger) dirigindo parâmetros **em runtime** — a eliminação do round-trip. No PH2D, os inputs dirigem parâmetros de nodes (incluindo warp/rig e shader nodes). É a síntese final da tese.

---

## 9. Tabela-resumo: algoritmo principal × alternativo

| Feature | Algoritmo principal | Alternativo | Crate/Ref |
|---|---|---|---|
| Largura variável | Stroke expansion Euler-spiral (Levien-Uguray) | Offset por normais + refit | `kurbo`; Inkscape PowerStroke |
| Pincel vetorial | Skeletal strokes (Frenet, arclength) | Warp bilinear/Coons | Inkscape Pattern-Along-Path |
| Curva suavizada | Schneider (Graphics Gems) | LSQ com detecção de cantos | `fit-curve`, `kurbo::simplify` |
| Gradient mesh | Coons subdiv + bicúbica | Gouraud sobre triângulos (GPU) | Inkscape mesh, Rasterific |
| Live Paint | Planar map / arrangement (DCEL) | Flood fill raster + revetorização | Inkscape livarot |
| Recolor/swatches | Indireção por tabela de cor | Extração de paleta k-means (OKLab) | — |
| Shape Builder | Vatti (Clipper2, i64) | Martínez-Rueda | `clipper2`, `clipper2-rust` |
| Blend/morph | Casamento de nós + LERP de controles | Interpolação intrínseca (ARAP) | Inkscape Interpolate |
| Offset path | Curva paralela Euler-spiral | Offset por SDF (GPU) | `kurbo`, Clipper2 offset |
| Simetria | Grupo de transformações + clones | Instanced rendering (GPU) | Inkscape Tiled Clones |
| Envelope/warp | Coons/bilinear warp | FFD por lattice | Graphics Gems IV |
| Puppet warp | MLS / ARAP | FFD | Schaefer et al. |
| Roughen | Ruído coerente na normal | Midpoint displacement fractal | Illustrator Roughen |
| Image trace | Potrace (poligono ótimo, O(n²)) | VTracer (clustering + fit O(n)) | `vtracer`, Potrace |
| Não-destrutivo | DAG de nodes, eval lazy + cache | Modifier stack linear | Inkscape LPE, Blender |

---

## 10. Ordem de implementação sugerida (dependências)

1. **Fundações geométricas:** integrar `kurbo` (curvas, arclength, offset, stroke expansion) e `lyon` (tesselação wgpu). Sem isso, nada acima funciona bem.
2. **Fit de gesto (1.3)** — a porta de entrada do tablet; habilita todo o input à mão livre.
3. **Modelo de node não-destrutivo (7)** — o esqueleto; defina `VectorNode` antes de escrever a segunda feature.
4. **Largura variável (1.1)** e **Offset (3.3)** — reusam o mesmo motor de curva paralela.
5. **Booleanos/Shape Builder (3.1)** via Clipper2 — desbloqueia Live Paint e simetria "cozida".
6. **Gradient mesh (2.1)** e **swatches (2.3)** — o diferencial "pintura", via wgpu + OKLab.
7. **Simetria (4.1)** e **Roughen (5.2)** — deleite rápido, baixo custo, reusam infra.
8. **Warp/rigging (5.1)** — ponte para animação estilo Rive (fase 2).
9. **Image trace (6)** via `vtracer` — fecha o round-trip papel→engine.
10. **Pincéis vetoriais (1.2)** e **Blend (3.2)** — expressividade avançada, dependem de 1.1/1.3.

---

## Referências principais

- Levien, R. & Uguray, A. (2024). *GPU-friendly Stroke Expansion.* Proc. ACM CGIT. (implementado em `kurbo`) — arXiv:2405.00127
- Levien, R. *Cleaner parallel curves with Euler spirals* e *Simplifying Bézier paths* (blog linebender).
- Schneider, P. J. (1990). *An Algorithm for Automatically Fitting Digitized Curves.* Graphics Gems (Academic Press).
- Hsu, S. C. & Lee, I. H. H. (1994). *Skeletal Strokes.* (base conceitual de pattern-along-path).
- Engelen, J. *PowerStroke LPE* — Inkscape `lpe-powerstroke.cpp` (LGM 2012).
- Bah, T. *Coons Patch Mesh Gradients in SVG* (base do SVG2 mesh).
- Heckbert, P. (1994). *Bilinear Coons Patch Image Warping.* Graphics Gems IV.
- Schaefer, S. et al. *Image Deformation Using Moving Least Squares.*
- Vatti, B. R. (1992). *A generic solution to polygon clipping.* CACM 35(7). (base do Clipper2)
- Selinger, P. (2003). *Potrace: a polygon-based tracing algorithm.*
- visioncortex. *VTracer* — vectorização colorida O(n) em Rust.
- Projetos Rust: `kurbo`, `lyon`, `clipper2`/`clipper2-rust`, `vtracer`, `flo_curves`, `bezier-rs` (arquivada; Graphite migrou para `kurbo` em 2025).
