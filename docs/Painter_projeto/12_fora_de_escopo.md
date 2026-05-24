# 12 — Fora de escopo (não-objetivos explícitos)

> Cada "não" aqui é uma **decisão consciente**, não esquecimento. Razões documentadas; reabrir um destes itens exige ADR novo.

> Filtro: o Painter v1.0 é raster, single-user, single-canvas, frame-based para anim, raster-only para todas as features. Tudo que vai além disso fica registrado aqui com a razão.

## 12.1 Vetor real

**Decisão:** Painter é 100% raster.

**Razão:** PH2D já tem vetor first-class via Vello + kurbo ([SKILL_Stack §11.2](../../SKILL_Stack_PH2D_Definitiva.md)). Vetor é responsabilidade de **outra ferramenta** (Vector Tool) sobre o mesmo canvas, idealmente futura. Misturar paradigmas (raster + vetor editável no mesmo tool) é o que Photoshop fez tarde demais e nunca ficou bom; ou Illustrator/Affinity Designer e Painter compartilham o canvas via layer-types, mas não a mesma tool.

**Quando talvez:** outra tool dedicada `ph2d-tool-vector` no futuro (W∞+). Compartilharia o canvas via "Vector layer" — layer-type novo que reuse os blend modes do compositor mas com geometry Bézier persistida.

**Mas no Painter:** importar SVG rasteriza (via `usvg` + Vello rasterizer); pintar com brush gera bitmap stamps; sem nodes editáveis pós-stroke (exceto via QuickShape Edit Shape mode, que ainda termina rasterizando).

## 12.2 Adjustment layers Photoshop-style (não-destrutivos)

**Decisão:** Adjustments são destrutivas (aplicam ao layer/selection). Sem adjustment layer stack que se possa reordenar/desligar/editar livremente.

**Razão:**
1. Espelha Procreate, mantendo o sabor.
2. Workflow lean — usuária aprende menos conceitos.
3. Duplicate Layer + Apply Adjustment = same outcome com 1 keystroke a mais (`Ctrl+J` antes da adjustment).
4. Implementação complexa (adjustment layer afetando N layers abaixo, com mask próprio, opacity, blend mode) — muito surface area; alto custo de manutenção.

**Quando talvez:** se o produto Painter for usado profissionalmente para print workflow Photoshop-replacement, adjustment layers viram demanda. Aí é ADR novo + nova UI mode (provavelmente "Painter Pro" mode toggle em Preferences).

**Workaround atual:** Live preview de adjustments durante drag — usuária vê resultado em tempo real e ajusta. Para preservar pré-state, Duplicate Layer (`Ctrl+J`).

## 12.3 CMYK first-class

**Decisão:** CMYK aparece como opção em canvas creation com aviso explicito, mas **não é first-class**. Internamente, render path é sempre RGB linear.

**Razão:**
1. PH2D não é app de print — é game engine 2D com Painter como Image Tool.
2. CMYK correto exige profile management complexo (ICC profiles, soft proofing, ink limits, dot gain). Sub-projeto inteiro.
3. Print workflow profissional usa Photoshop/Affinity para conversão CMYK no fim do pipeline; PH2D entrega RGB de altíssima qualidade.
4. Procreate "tem CMYK" mas com mesmas limitações (display sempre converte para sRGB/P3; adjustments operam em RGB). Mantemos o mesmo nível de "honesto, com aviso".

**Quando talvez:** ferramenta dedicada `ph2d-tool-print-export` no futuro, com proper CMYK conversion via lcms2 (ou crate Rust nativa). Esse tool seria separado do Painter — recebe Painter export RGB e converte com profile correto.

## 12.4 Procreate Dreams timeline (motion tracks, keyframes, audio)

**Decisão:** Painter tem **apenas Animation Assist** (frame-based, vide [10_animation_assist.md](10_animation_assist.md)). Sem timeline track-based, sem keyframes interpoláveis, sem motion tweening, sem audio sync.

**Razão:**
1. Procreate Dreams é app **separado** da Savage justamente porque mistura modelo (frame-by-frame de Procreate + tracks de animação) quebra a filosofia "canvas-first" de Procreate. Mistura impede que cada um seja excelente no seu domínio.
2. PH2D **já vai ter um nó de animação** (sistema de nós, vide [HANDOFF_node_system.md](../HANDOFF_node_system.md)). Motion/keyframe/timeline pertencem ao node-graph — onde acoplam com physics, lighting, scripting. Painter exportando ativos para esse pipeline é a integração natural.
3. Implementação Dreams-style é um projeto inteiro (12+ meses para uma equipe).

**Quando talvez:** um node "Animation Editor" no node-graph PH2D, ortogonal ao Painter, no roadmap pós-W∞.

## 12.5 Face Reference / ARKit FacePaint

**Decisão:** Reference Companion tem apenas Canvas mode + Image mode. **Não tem Face mode** (que Procreate oferece via ARKit).

**Razão:**
1. ARKit é iOS-only. Implementar features iOS-only quebra a regra "multi-plataforma DAY 1" (vide [README.md](README.md) §6).
2. Use case é nicho (face painting / makeup / cosplay design).
3. Implementação requer integração nativa profunda (Swift) + capture de camera + face tracking — surface área grande para benefit marginal.

**Quando talvez:** se demanda surgir, implementar como **shell extension iOS** independente, expondo Face mode apenas em Reference Companion quando shell iOS detecta a capability. Continua sendo opcional, não-bloqueante para outras plataformas.

## 12.6 Cloud sync nativo

**Decisão:** Sem cloud sync embutido. Arquivos são FS-local. Usuária sincroniza via Files.app / Dropbox / iCloud Drive / OneDrive / Google Drive / Git LFS / quaisquer ferramentas externas.

**Razão:**
1. Cloud sync é **infraestrutura complexa**: conflict resolution, large file transfer, encryption, multi-device authentication, billing.
2. PH2D não é serviço hospedado — é engine open-source-spirit + app local.
3. FS-local com files content-addressed (HR-6) já oferece reproducibilidade entre devices via copy.

**Quando talvez:** opcionalmente, integração WebDAV / S3 / git como **plugin** (sabor (3) stateful tool com sync logic) — explora pós-v1.0 se demanda emergir.

## 12.7 Macros / Actions / Scripts Photoshop-style

**Decisão:** Não há "Action recorder" ou Photoshop-Actions. Para automação, usar **Luau scripts via MCP** (HR-10).

**Razão:**
1. PH2D já tem Luau como linguagem de scripting first-class ([SKILL_Stack §11.7](../../SKILL_Stack_PH2D_Definitiva.md), ADR-0019). Toda API exposta a Lua é exposta a MCP automaticamente. Painter expõe brushes/layers/strokes via `#[lua_export]`.
2. Action recorder seria uma UI alternativa para o mesmo subset — duplicação de surface.
3. Power-users que querem automação em Painter aprendem Luau (ou usam MCP via LLM agent) — mais poderoso e composável que Action recorder linear.

**Quando talvez:** Action recorder visual estilo Photoshop poderia ser desenvolvido como sugar layer sobre Luau scripting (UI gera script Luau equivalente). Pós-v1.0, se demanda alta.

## 12.8 Smart Objects, Layer Linking, Instâncias Clone

**Decisão:** Sem layer linking, sem smart objects, sem instâncias compartilhadas.

**Razão:**
1. Paradigm fits do node-graph PH2D — duplicação de surface se feita no Painter também.
2. Procreate clássico não tem isso (mantém sabor).
3. Clone tool é destrutivo (clona pixels com offset, sem "live link").

**Quando talvez:** layer linking não vai entrar no Painter; entra no node-graph como nodes "Instance of X" que referenciam outros nodes.

## 12.9 Text avançado no Painter

**Decisão:** Sem text tool sofisticado no Painter. Text é responsabilidade de **Text Tool** separado (futuro), usando `parley` ([SKILL_Stack §11.3](../../SKILL_Stack_PH2D_Definitiva.md)).

**Razão:**
1. Texto editável (cursor, IME, BiDi, complex scripts, OpenType features) é sub-projeto inteiro com parley.
2. Procreate text tool é raster-friendly mas limitado; usuárias profissionais usam Photoshop/Illustrator para text e importam.
3. Painter aceita text como **layer raster importada** — Text Tool produz raster do texto, layer entra no canvas Painter.

**Quando talvez:** Text Tool dedicada (sabor (3) stateful + panel) com parley as core. Painter trabalha sobre o output.

**Importing**: PSD com text layers → Painter flattenes para Raster (com warning, vide [09 §9.4](09_export_interop.md)).

## 12.10 3D painting (Materials Metallic + Roughness)

**Decisão:** Painter é **2D-only**. Sem 3D model painting, sem Materials channels.

**Razão:**
1. PH2D é game engine **2D**. 3D explicitamente fora de scope ([SKILL_Stack §3 Não-objetivos](../../SKILL_Stack_PH2D_Definitiva.md)).
2. Procreate tem 3D Materials para ilustradores, mas é nicho.

**Quando talvez:** **nunca** no PH2D. Se o usuário precisa pintar 3D, use Blender / Painter 3D / Substance Painter.

**Mas:** Painter v1.0 pode pintar **normal maps** e **emission masks** como layers raster RGB normais, exportar como PNG e o usuário aplica em outro pipeline (Painter não interpreta normal maps; é texture data). Use case real para o PH2D engine usar (lighting via normal maps em sprites 2D).

## 12.11 Animação live (motion graphics, particles)

**Decisão:** Animation Assist é puramente frame-by-frame. Sem motion graphics, sem particles, sem motion paths.

**Razão:**
1. Particles + motion = node-graph PH2D + ferramenta de animação dedicada (futuro).
2. Painter foca em **ilustração** (incluindo animação leve frame-by-frame).

## 12.12 Symmetry 3D / Mandela complexa

**Decisão:** Symmetry guides (V/H/Quadrant/Radial) é suficiente. Sem hyperbolic symmetry, sem 17-wallpaper groups, sem 3D mirror.

**Razão:**
1. Cobre 95% dos casos.
2. Wallpaper groups requerem matemática especializada — sub-projeto.

**Quando talvez:** ferramenta dedicada `ph2d-tool-mandala` no futuro, plugin opcional.

## 12.13 Tools de painting baseadas em ML (style transfer, autocomplete, upscale)

**Decisão:** Sem features ML embedded no Painter v1.0.

**Razão:**
1. Modelos ML são pesados (modelos GB+), aumentam app size dramaticamente.
2. Requerem GPU/NPU capability não uniforme entre devices.
3. Espaço evolui rápido — features hardcoded ficam desatualizadas.
4. PH2D MCP first-class permite **LLM externo** fazer essas tarefas via API exposta — bg removal AI, style transfer, etc. ficam em serviços externos chamados via MCP.

**Quando talvez:** features ML **opcionais via plugin** (`ph2d-tool-painter-ml-pack`) que usuária instala se quiser. Plug-in baixa modelo on-demand. Pós-v1.0.

## 12.14 Multi-user collaborative painting

**Decisão:** Single-user, single-canvas. Sem live collab (Figma-style).

**Razão:**
1. Networking sub-projeto não-trivial (CRDT, presence, conflict resolution).
2. PH2D networking é gameplay-oriented (rollback netcode); paint collab requer modelo completamente diferente.
3. Procreate não tem; manter o sabor.

**Quando talvez:** se demanda alta, plugin separado com WebRTC / WebSocket sync. Pós-v1.0+.

## 12.15 Plugin system

**Decisão:** Sem plugin system formal no v1.0. Funcionalidade extra entra como **crate satélite** no workspace (fan-out drop-crate, DIRETRIZ §3.8).

**Razão:**
1. Plugin system formal exige API estável + sandboxing + delivery mechanism — sub-projeto.
2. Modelo PH2D já é "fan-out de crates" — uma "extensão" é uma tool/painel nova.
3. Luau é o caminho user-facing para customization.

**Quando talvez:** plugin system formal (DLL/dylib loading + sandbox) só se PH2D virar plataforma comunitária com ecossistema third-party. Pós-v1.0+.

## 12.16 GIF reading com edição frame-a-frame

**Decisão:** Import GIF → carrega como Animation Assist com cada frame virando layer. Export GIF re-exporta. Mas **sem timeline editor sofisticado** (entries por frame, easing entre cores).

**Razão:** Animation Assist é frame-based simples — cobre o use case. Sofisticação adicional é Dreams territory.

## 12.17 RAW image format support

**Decisão:** Sem suporte direto a CR2/NEF/ARW/etc. (camera RAW).

**Razão:** RAW workflow é Lightroom/RawTherapee/darktable territory. Painter aceita PNG/JPEG/TIFF/EXR/WebP — usuária converte RAW para esses fora do Painter.

**Quando talvez:** se demanda significativa, importer plugin via `image` crate features ou plugin separado.

---

## 12.18 Resumo

Tudo nesta página tem **uma razão concreta**. Quando alguém perguntar "por que Painter não tem X?", aponte essa razão. Se o raciocínio mudar (mercado, hardware, demanda), revisite via ADR.

**Princípio operacional:** estamos construindo a **versão essencial do Painter**. Excelência em cada feature in-scope > breadth shallow em features out-of-scope. Procreate venceu Photoshop em iPad fazendo menos coisas, melhor. Painter PH2D segue a mesma lógica.

**Continua em:** [13_referencias.md](13_referencias.md) — fontes oficiais Procreate + papers + referências PH2D.
