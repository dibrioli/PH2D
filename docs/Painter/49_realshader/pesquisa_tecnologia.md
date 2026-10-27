# 49 — Pesquisa: a tecnologia por trás do RealShader/SoftShadows e o estado da arte

> 26/10/2026. Só textos públicos (blog, manual, imprensa, artigos). Nada do binário, shaders ou mapas do
> Rebelle foi lido. Complementa o [README](README.md) §1–§4 (medições M1–M11). Veredito do dono que motiva
> esta pesquisa: (1) pixels brancos nas bordas; (2) a sombra parece *drop shadow* de borda, sem sombra de
> crista DENTRO da tinta; (3) o relevo não sobe com a tinta acumulada.

## 1. O que a Escape Motions disse (tudo o que se achou)

| Data | Fonte | Citação (verbatim) |
|---|---|---|
| 05/06/2025 | [Entrevista «10 anos de Rebelle»](https://www.escapemotions.com/blog/10-years-of-rebelle-interview-with-developers) (Fapšo, Sedlák, Surovček, Obuch) | Surovček: «This led to a brand-new oil material, based not only on the new NanoPixel…»; «Material realism also requires realistic shadows». Obuch: «NanoPixel rendering with OpenGL». NanoPixel «now powers real-time scaling, transformations, and our liquify tool». |
| 11/06/2025 | [Blog «Realistic Oil Shader with Soft Shadows»](https://www.escapemotions.com/blog/rebelle-8-realistic-oil-shader-with-soft-shadows) | «using real photos as environment maps»; «Instead of calculating real-time reflections, it uses a pre-made image that represents the surrounding environment»; «we captured a photo of a gray-painted sphere and used it to create one of the texture maps»; «oil brushstrokes can cast realistic shadows, reflect light based on their shape and color»; «white strokes reflect more light and appear brighter, while darker tones absorb more light». Direcção da luz: staff nos comentários, «would be not possible at the moment». |
| 11/07/2025 | [Digital Production](https://digitalproduction.com/2025/07/11/oil-gets-real-rebelle-8s-realshader-and-softshadows-put-ray-tracing-on-the-canvas/) | «Rebelle 8 integrates real-time ray tracing directly into the canvas»; «Every brush stroke, no matter how thick or thin, will cast soft shadows onto the paint below». |
| 08/10/2025 | [Digital Production](https://digitalproduction.com/2025/10/08/rebelle-8-digital-paintings-physical-upgrade/) | «As paint thickens, it casts micro-shadows and occlusion into crevices»; «The more layers of pigment you stack, the more dimensional your canvas becomes»; «Every ridge, lump, and valley now catches highlights and shadows based on a 3D environment map». |
| manual 8.2 | [Visual Settings](https://escapemotions.com/products/rebelle/manual/8.2/interface/panel-visual-settings/) | RealShader liga OpenGL; «Turn RealShader off to use the CPU instead». SoftShadows: «Shadows based on raytracing». Shadow Altitude: «The higher the altitute, the more pronouced the shadows are» (o slider é invertido face ao ângulo físico — casa com M10/W0b: slider 10 = 42°). Impasto Depth: «visibility of the structure». Gloss: «illumination of the paint structure». |
| 30/07/2026 | [Blog 8.3](https://www.escapemotions.com/blog/rebelle-83-with-new-features-out-now) · [DP](https://digitalproduction.com/2026/08/05/rebelle-8-3-adds-exr-heightmap-export/) | «export the physical depth information of your impasto paint strokes»; «saved in the 16-bit EXR format», que «supports a much wider range of depth values than standard 8-bit images». |

**Não existe** artigo, patente ou palestra técnica da Escape Motions sobre isto (busca por patentes com
Blaškovič/Fapšo/Surovček: nada; o [SIGGRAPH](https://history.siggraph.org/experience/rebelle-real-watercolor-and-acrylic-painting-software-by-blaskovic-and-fapso) só lista a demo de 2015). LinkedIn dos devs: nada técnico indexado.
**Inferência** a partir das citações: (a) um campo de **altura/«profundidade física» por pixel, em ponto
flutuante e sem tecto curto** (EXR 16-bit, «more layers … more dimensional») — a altura ACUMULA; (b)
RealShader = IBL clássico (esfera cinza fotografada = *light probe*/matcap); (c) «ray tracing» = marcha de
raio no campo de altura num *fragment shader* OpenGL (há fallback de CPU sem ele) — não RT por hardware;
(d) «micro-shadows and occlusion into crevices» = um termo de **oclusão** separado da sombra direccional
(confirma M7: o escuro da fenda vem do shader, não da sombra).

## 2. Linhagem e estado da arte

| Técnica | O que faz | Custo | Sombra DENTRO da tinta? | Licença |
|---|---|---|---|---|
| Max 1988 *Horizon mapping*; Sloan & Cohen 2000 (EGWR; [resumo](https://en.wikipedia.org/wiki/Horizon_mapping)) | Guarda, por texel, o ângulo do horizonte em N azimutes; sombra = luz abaixo do horizonte | pré-cálculo O(N·R) por texel; consulta O(1) | Sim (exacta por azimute) | artigos; sem patente activa conhecida |
| [Snyder & Nowrouzezahrai 2008](https://diglib.eg.org/handle/10.2312/CGF.v27i4pp1275-1283) | Horizontes de campo **dinâmico** extraídos de uma **pirâmide multi-resolução** da altura; visibilidade por cunhas em SH ordem 4 | 256² a 57 Hz numa 8800 Ultra; «key light» precisa de ~3 azimutes (vs 16–32 p/ ambiente) | Sim, macia | MSR; uso livre da ideia |
| [Timonen & Westerholm 2010](https://diglib.eg.org/handle/10.2312/CGF.v29i2pp723-731) | Horizontes **exactos** por varrimento de linhas em N azimutes, O(1) amortizado por texel | 1024² a 23 fps, 64 direcções (GTX 280) | Sim, com luz de ambiente 256×64 | artigo |
| SDF soft shadow (Quilez, `min(k·h/t)`) | Penumbra por razão distância/percurso | K passos | Sim, mas `h` num campo de altura é distância VERTICAL, não euclidiana: penumbra larga e mole em degraus, quase nula em cristas baixas | domínio público de facto |
| [Relaxed cone stepping](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-18-relaxed-cone-stepping-relief-mapping) (Policarpo & Oliveira, GPU Gems 3, 2007) | Pré-calcula um cone livre por texel; o raio salta distâncias grandes | ~8–15 passos; pré-cálculo caro (refazer na região suja) | Sim | livro; sem patente conhecida |
| Screen-space shadows ([Bend/Days Gone](https://www.bendstudio.com/blog/inside-bend-screen-space-shadows/), SIGGRAPH 2023) | Marcha por pixel até à luz sobre a profundidade | ~1 marcha por pixel | Sim (detalhe fino) | código de exemplo público; licença não verificada |
| HBAO / [GTAO](https://www.iryoku.com/downloads/Practical-Realtime-Strategies-for-Accurate-Indirect-Occlusion.pdf) (Jimenez et al. 2016; rev. 2019 com GTSO) | AO por horizonte em fatias; GTAO acrescenta **multi-bounce em função do albedo** (ajuste cúbico) e oclusão especular | 2–4 fatias × 8–16 passos + filtro | AO de fenda | artigo Activision |
| IBL split-sum ([Karis 2013](https://cdn2.unrealengine.com/Resources/files/2013SiggraphPresentationsNotes-26915738.pdf)), irradiância SH9 ([Ramamoorthi & Hanrahan 2001](https://graphics.stanford.edu/papers/envmap/)) | Ambiente pré-filtrado por rugosidade + LUT; difuso em 9 coeficientes | 1–2 consultas | Não (precisa de AO/sombra à parte) | livre |
| [IMPaSTo](https://gamma.cs.unc.edu/IMPASTO/) (Baxter, Wendt, Lin, NPAR 2004) | Cada camada é um **campo de altura** (1 húmida + N secas «baked»); KM por espessura `d` de camada; «per-pixel dot-product bump-mapping and Blinn-Phong specular» ([texto](https://math.vu.nl/~eliens/demo/local/documents/impasto.htm)) | GPU 2004, interactivo | **Não** (sem sombra nem AO) | académico (eqs 8–9 com errata) |
| [Chu, Baxter, Wei, Govindaraju 2010](https://microsoft.com/en-us/research/wp-content/uploads/2010/06/PaintModel_NPAR_2010.pdf) | Pickup map 2D; «a separate texture to drive paint thickness accumulation and removal» para cristas | 145 fps numa iGPU | Não; o próprio artigo admite «shallow ridges» | MSR |
| [Corel Painter Impasto](https://product.corel.com/help/Painter/540111162/Corel-Painter-en/Corel-Painter-Impasto-lighting-and-depth.html) | «you accumulate depth information as you paint»; luzes direccionais na esfera; «Amount» = visibilidade | barato | Não documentado (aparência de bump) | proprietário |

Anti-cintilação especular: [Toksvig 2005](https://developer.download.nvidia.com/whitepapers/2006/Mipmapping_Normal_Maps.pdf)
(comprimento da normal média → alarga/escurece o brilho), [LEAN](https://www.cse.chalmers.se/edu/year/2011/course/TDA361/Advanced%20Computer%20Graphics/LEANMapping.pdf),
e AA especular **geométrico** por derivadas de ecrã da normal ([Kaplanyan, GDC 2016](https://developer.download.nvidia.com/gameworks/events/GDC2016/akaplanyan_specular_aa.pdf)).

## 3. Respostas directas

**(a) Crista que sombreia a tinta vizinha (não só a borda).** A sombra tem comprimento `L ≈ Δh·cot(alt)`.
Hoje o interior da nossa tinta tem `Δh` minúsculo: a massa é **suavizada** antes de virar altura (a crista
some) e os fios valem ±1,3 px contra ~15 px do degrau da borda (W0c). Qualquer algoritmo — marcha, horizonte,
cone — só pode sombrear o interior se o campo TIVER cristas de alguns px. Duas condições, ambas necessárias:
(1) **campo com cristas de amplitude real** (sem passa-baixo sobre a altura que vai para a sombra; a
suavização só para a normal); (2) **marcha que vê ocluidores próximos e longe**: passos curtos perto (1 px)
e crescentes longe, ou consulta a uma **pirâmide MAX da altura** (Snyder 2008: multi-resolução), e sem
*bias* de partida que salte a crista vizinha. A penumbra deve vir do **ângulo** (fracção do disco solar acima
do horizonte, como Timonen/Snyder), não de `k·Δh/t` vertical — é o `Δh/t` que faz o degrau da borda dominar.
Mais: a fenda escura do Rebelle (M2/M7) é **oclusão** com multi-bounce dependente do albedo (o modelo GTAO
dá exactamente `escuro ∝ albedoᵏ`), não sombra — logo «sombra dentro da tinta» = AO forte nas fendas + sombra
direccional das cristas.

**(b) Relevo que sobe com a tinta.** Modelo dos sistemas publicados: altura = **volume por célula /
área** (linear), **acumulada por camadas** — IMPaSTo soma camadas secas «baked» à húmida; Corel «accumulate
depth»; Rebelle exporta «physical depth» em EXR 16-bit e diz «more layers … more dimensional». Nada indica
saturação. Atenção: um planalto que sobe por igual **não muda a normal** — só se vê subir pela borda (sombra
mais longa, M10), pela AO e pela textura. Por isso: (i) altura **sem tecto** e linear na massa acumulada
(seca + húmida), (ii) amplitude da textura de cerdas/faca **proporcional à espessura local** (tinta grossa =
sulcos fundos; Chu 2010 usa uma textura para «thickness accumulation and removal»), (iii) **acumulação na
borda** (*edge pile-up*: a espátula empurra massa para os lados — é o degrau que o Rebelle sombreia, M6/M9).
Saturação (`tanh`) só como curva de **visualização** opcional, nunca no campo.

**(c) Sem pontos brancos na borda.** Causa provável no nosso: na borda a altura cai de ~h a 0 em 1–2 px →
o Sobel dá normal quase horizontal → GGX + Fresnel em ângulo rasante = brilho saturado, e o papel sem tinta
recebe especular. Curas publicadas e padrão: altura **pesada pela cobertura** (`h·α`, com α suave) antes
do gradiente; especular e IBL **multiplicados pela cobertura**; normal tirada de uma altura filtrada (mip 1
ou gaussiana 3×3) na borda; **Toksvig/LEAN** (rugosidade sobe onde as normais divergem) e **AA geométrico**
(rugosidade += variância das derivadas da normal, Kaplanyan 2016); *clamp* da inclinação; e oclusão especular
(GTAO) para que a crista na sombra/fenda não brilhe (M7: a sombra «apaga brilhos de cristas à sombra»).

## 4. Recomendação (por ordem)

1. **Corrigir o campo antes da luz** — altura linear e sem tecto na massa acumulada (seca+húmida), ganho
   pelo slider *Profundidade*, sem suavização na altura usada por sombra/AO; textura de ferramenta com
   amplitude ∝ espessura; *edge pile-up* nas espátulas. *Porquê:* veredito (2) e (3) nascem aqui, não no
   shader. *Fontes:* IMPaSTo, Chu 2010, Rebelle 8.3 EXR, DP 08/10/2025.
2. **AO por horizonte com multi-bounce dependente do albedo (GTAO) aplicada em linear** — `ao' = f(ao, albedo)`
   e lei M2 (`k≈2`). *Porquê:* é o escuro saturado das fendas (M1–M3, M7). *Fonte:* Jimenez et al. 2016.
3. **Sombra por horizonte na direcção da luz com penumbra ANGULAR** (fracção do disco da luz acima do
   horizonte), passos crescentes / pirâmide MAX da altura, sem bias que salte a crista vizinha. *Porquê:*
   sombra de crista dentro da tinta + comprimento ∝ `cot(alt)` (M10) com custo fixo e barato na Radeon (W0a).
   *Fontes:* Snyder & Nowrouzezahrai 2008; Timonen & Westerholm 2010.
4. **Cobertura em tudo:** altura·α antes do Sobel, especular·α, normal filtrada na borda. *Porquê:* veredito
   (1). *Fonte:* prática padrão; Toksvig 2005.
5. **AA especular (Toksvig + geométrico) e oclusão especular** (`spec *= visibilidade`). *Porquê:* tira
   cintilação e o brilho de cristas à sombra (M7). *Fontes:* Toksvig; Kaplanyan 2016; GTAO.
6. **IBL split-sum + SH9** a partir de uma foto NOSSA de esfera cinza / HDR CC0, rodável. *Porquê:* brilho
   que segue a crista (M4/M9), método declarado pela Escape Motions. *Fontes:* blog 11/06/2025; Karis 2013;
   Ramamoorthi & Hanrahan 2001.
7. **Sombra cai no papel** (só o termo sombra) e escala linear no slider. *Porquê:* M6. *Fonte:* medição.
8. **Recusar** relaxed cone stepping por agora: o pré-cálculo do cone tem de ser refeito a cada pincelada na
   região suja — a pirâmide MAX dá o mesmo salto com actualização O(log) local. *Fonte:* Policarpo & Oliveira 2007.
