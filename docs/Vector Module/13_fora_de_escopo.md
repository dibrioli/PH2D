# 13 — Fora de escopo (não-objetivos explícitos v1.0)

> Spec dos **não-objetivos deliberados** do Vector Module v1.0. Cada "não" é decisão consciente, com razão técnica + "quando talvez" + alternative. Reverter exige ADR amendment.
>
> **Princípio:** spec opinionada (vide [README §4](README.md)). Cada OUT economiza complexidade e foco.

## 13.1 3D vector / parametric 3D primitives

### Razão

PH2D é **2D engine** ([SKILL_Stack §3](../../SKILL_Stack_PH2D_Definitiva.md) Não-objetivo #1). Adicionar 3D primitives no Vector Module fragmenta o foco do projeto.

### Alternative

- 2.5D effects (parallax, normal maps) via Painter (raster) + motion nodes.
- glTF import para 2.5D normal maps OK (vide SKILL_Stack §11.10 asset pipeline).

### Quando talvez

PH2D 2.0+ se houver demanda real (provavelmente nunca — engine é deliberadamente 2D-focused).

---

## 13.2 Print production CMYK first-class

### Razão

PH2D foca em **digital art e games**, não impressão profissional. CMYK é caro: ICC profiles + soft proofing + spot colors + ink limits. Hoje, mercado de impressão profissional usa Adobe ecosystem; PH2D não compete nesse vertical.

### Alternative

- Internal color space: OKLCH (perceptual uniform) + sRGB / Display P3 detection per-device (vide Painter [ADR-0051](../architecture/decisions/0051-color-profile-pipeline.md)).
- Export-side tool dedicado se demanda real surgir: `vector-export-cmyk` node que applica ICC profile + CMYK separations em export.

### Quando talvez

Se artista profissional pedir explicitly. Roadmap V2.0+ com ICC profile support — não-prioritário.

---

## 13.3 DTP / page layout (InDesign-class)

### Razão

InDesign é **multi-page document layout** com master pages, story flow, paragraph styles, table of contents. Completamente fora do escopo de "ferramenta vetorial de ilustração + arte de jogo".

### Alternative

- Single-canvas focus em v1.0.
- Multi-canvas Gallery somente W11+ (vide [README §11](README.md) decisão estrutural).

### Quando talvez

Nunca — PH2D não vai pivotar para DTP.

---

## 13.4 Live web-collab Figma-style (servidor)

### Razão

CRDT data model (vide [01 §1.5](01_data_model.md) + ADR-0057) já habilita **multi-agente local** (agent ↔ designer). Web-collab cross-internet via **servidor** é orthogonal e exige infrastructure não-trivial (auth, WebSocket transport, conflict resolution at scale).

### Alternative

- CRDT local agent ↔ designer **IN W1** — habilitada para multi-agente local sem servidor.
- File-based merge (e.g., dois designers committam ao mesmo `.ph2d-vector` em git; CRDT merge produces convergent result via `git merge`).

### Quando talvez

V2.0+ se PH2D ecosystem cresce para colaboração cross-team. CRDT data model já está pronto — apenas servidor + transport layer.

---

## 13.5 Vector AI / motion-tween-by-prompt como única autoria

### Razão

LLM-as-graph-node ([§09](09_scripting_mcp.md), Inovação #4) emite **strokes editáveis** (não substitui artista). Sem "magic generate full illustration" sem editing.

### Anti-princípio: "AI faz tudo, artista é supérfluo"

PH2D rejeita essa visão. LLM = ajuda, não substitui. Outputs LLM são editáveis downstream. Artista permanece central.

### Alternative

- LLM emite vector network estruturado editável.
- Motion-tween-by-prompt via animation graph (motion nodes + state machine), não LLM auto-generation.

### Quando talvez

Nunca em v1.0+. Se demanda surgir em V3.0+ para "AI agent autonomously edits canvas", seria novo workflow opt-in com explicit user authorization (HR-11 governance).

---

## 13.6 ExtendScript / CEP / proprietary plugin SDK

### Razão

ExtendScript (Adobe) é horrível (ES3, single-thread, no async, freezes host UI). PH2D usa **Luau / WASM / MCP** somente (HR-8 + HR-10 + HR-11).

### Alternative

- Luau custom modifiers (`vector-luau-script` node).
- WASM plugins (futuro stretch).
- MCP tools (LLM agent integration).

### Quando talvez

Nunca. ExtendScript / CEP é anti-pattern.

---

## 13.7 Compatibility 100% com Adobe Illustrator AI nativo

### Razão

AI format é PDF wrapped com Adobe-proprietary extensions. Round-trip lossless impractical sem reverse-engineering Adobe internals.

### Alternative

- Import AI lossy via PDF subset (vide [01 §1.10.2](01_data_model.md)). Documented gaps.
- Export via SVG (round-trip lossless dentro do PH2D v1.0 subset) OR PDF.

### Quando talvez

Nunca. Lossy import + log gaps é trade-off aceito.

---

## 13.8 PDF authoring (criar PDF arbitrário)

### Razão

PDF authoring full requires text engine, font embedding, color profiles, JavaScript embedded, form fields, etc. Adobe Acrobat-level. Out of scope.

### Alternative

- Export PDF subset (paths + gradients + text). Read via lopdf.
- Web/iPad/Android targets já oferecem "print to PDF" via OS dialog.

### Quando talvez

Nunca em v1.0. V2.0+ se demanda real, only subset.

---

## 13.9 Mesh gradients hand-author (Illustrator-style mesh patches)

### Razão

Hand-authoring mesh patches é tedioso (Illustrator) e UI complexa. **Substituído por diffusion curves** (vide [05 §5.6](05_procedural_fill.md), Inovação #2).

### Alternative

Diffusion curve via Poisson PDE: autor desenha curve + cores em ambos lados; GPU diffunde o resto. Vastly simpler UX + GPU-resident.

### Quando talvez

Nunca. Diffusion curves são strictly superior.

### Import path

AI / SVG com mesh gradient → conversion para diffusion curve approximation (lossy). Logged em import.

---

## 13.10 SVG Filters DOM-level (feGaussianBlur etc)

### Razão

SVG filters DOM-level são limitados (filter chain hard-coded) e renderizados via raster path em browsers. **Substituído por procedural shader graph** ([05](05_procedural_fill.md)) — mais flexível + GPU compute.

### Alternative

- ProceduralShader fill graph com Noise / Voronoi / Ramp / Mix / Bump nodes.
- Export to SVG: bake fill graph to first-frame raster image OR skip filter.

### Quando talvez

Nunca em runtime. Export bake é solução standard.

---

## 13.11 Multi-line text composition

### Razão

Multi-line text (paragraphs, alignment, justification) é trabalho de DTP (vide §13.3). Vector Module foca em **text-on-path** (single line per path).

### Alternative

- Text-on-path em W14.
- Multi-line text como múltiplos vector layers sequenciais (manual layout).

### Quando talvez

V2.0+ se demanda real surge. Não-prioritário.

---

## 13.12 Macros / Actions / Photoshop-style automation

### Razão

Macros style Photoshop são single-purpose (record + replay UI actions). Vector Module habilita superior via **Luau scripting** (HR-10) + node graph automation.

### Alternative

- Luau scripts (record actions e save como `.luau` script).
- Node graph templates (save subgraph como Symbol).

### Quando talvez

Nunca. Luau >> Macros.

---

## 13.13 Smart objects / layer linking / instâncias clone

### Razão

Smart objects (Photoshop) e instâncias clone (Illustrator Symbols) — Vector Module entrega via **Symbol system** (vide [04 §4.8](04_tools.md), Inovação Cuttle-style).

### Alternative

PH2D Symbols **são parametric** (Cuttle-style — sliders driving geometry, beats Figma components 10×). Edit master → sync all instances.

### Quando talvez

PH2D Symbols **already cover** este use case. Layer linking simples (alias) NÃO é v1.0 — somente Symbols.

---

## 13.14 Cloud sync nativo

### Razão

Cloud sync via servidor PH2D-specific = infrastructure cara + dependency. PH2D usa **file-system based** assets; sync via OS tools (iCloud / Dropbox / Drive / git) é orthogonal.

### Alternative

- `.ph2d-vector` files em qualquer file system.
- git para versioning (CRDT data model permite merging em git).
- Cloud sync via Apple Files.app / Dropbox / Google Drive — user choice.

### Quando talvez

Nunca como serviço PH2D-hosted.

---

## 13.15 Vector Filters em runtime (estilo Adobe vector filters animados)

### Razão

Vector Module entrega procedural fill graph + animation system + state machine — combina tudo isso = "vector filters animados" emergente. **Não há filter system separado.**

### Alternative

Filter visual desejado → graph com modifier + fill + animation. Mais flexível que filters fechados Adobe-style.

### Quando talvez

Nunca. Filter system seria redundante.

---

## 13.16 OpenGL / D3D11 / Vulkan 1.0-1.2 backends

### Razão

Vello requires WebGPU compute. PH2D SKILL_Stack §4 explicitly cuts legacy GPUs (Vulkan 1.3 minimum em Android, Metal 3 em iOS, etc.). Adicionar backends legacy = massive maintenance burden sem return on investment.

### Alternative

- Vello CPU SIMD fallback (existing) para devices sem compute.
- Hardware abaixo da matriz §4 não é alvo (feature won't fix).

### Quando talvez

Nunca. Matrix decisão consciente.

---

## 13.17 Servidor de licenças / DRM

### Razão

PH2D é Rust-based engine open-source-friendly. Sem DRM, sem servidor de licenças. Distribution model é direct binary OR Apple Store / Google Play / etc.

### Alternative

- Open licensing (MIT / Apache-2.0).
- Apple Store etc. handles licensing on their platforms.

### Quando talvez

Nunca.

---

## 13.18 Mobile-specific gestos exclusivos

### Razão

PH2D é multi-platform desde W1 (vide README §6). Gestos exclusivos de mobile (e.g., shake-to-undo iPad-only) discriminam desktop / Android.

### Alternative

- Gestos cross-platform sempre (e.g., 2-finger undo funciona em iPad + Android).
- Hardware-specific features (Apple Pencil Pro squeeze) detected runtime + graceful degrade.

### Quando talvez

Nunca. Cross-platform consistency é princípio.

---

## 13.19 Auto-trace runtime no jogo

### Razão

Auto-trace (raster → vector) é operação **editor** (W12), não runtime de jogo. Game runtime tem `.ph2d-vector` asset already-vectorized.

### Alternative

- Pre-process raster em editor com `vector-auto-trace` node.
- Ship `.ph2d-vector` asset para jogo.

### Quando talvez

V3.0+ se demanda real (e.g., "user generated content em jogo where players upload raster + vectorize live"). Não é v1.0.

---

## 13.20 Conversor universal de qualquer vector format

### Razão

Format universe is vast (SVG / AI / PDF / EPS / DXF / SWF / Lottie / WMF / EMF / VSDX / etc.). PH2D cobre SVG + AI subset + PDF subset + `.ph2d-vector` native.

### Alternative

- SVG full subset import / export (W18).
- AI lossy import via PDF subset.
- Outros formats: user converts externamente (Inkscape supports DXF / EPS / WMF).

### Quando talvez

V2.0+ adicionar Lottie import. Outros formats não.

---

## Fim da OUT-list

20 não-objetivos com razões + alternatives + "quando talvez". Cada um é decisão consciente; reverter exige ADR amendment.

**Próxima leitura:** [`14_inovacoes_extraordinarias.md`](14_inovacoes_extraordinarias.md) (o que ESTÁ em scope = as 7 inovações).
