# Flip — animação desenhada quadro-a-quadro (o meio "Grease Pencil" do PH2D)

> **Decisão de arquitetura:** [ADR-0114](../architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md).
> **Plano de implementação (waves + tasks):** [`01_plano_waves.md`](01_plano_waves.md).
> **Referência de algoritmos (Blender 5.2, consultar antes de cada tópico):** [`02_referencia_algoritmos_blender_5.2.md`](02_referencia_algoritmos_blender_5.2.md).

## O que é

**Flip** é o quarto meio de criação do PH2D, ao lado de **Painter** (raster), **Vector** (vetor exato) e
**Motion Nodes** (procedural). É o meio do **cel / animação tradicional**: você **desenha quadro a
quadro**, com traços expressivos (largura e opacidade por pressão), *Ghost Frames* (onion skin) para
ver os quadros vizinhos, *Tween* para gerar inbetweens automáticos, *Fill* (balde) e *Reshape*
(escultura de traço). É a estética de Cuphead, Skullgirls, Dragon's Crown — que nenhum sprite-sheet
estático entrega.

A referência de estado-da-arte é o **Grease Pencil** do Blender (reescrito no 5.x como GPv3). Portamos
a **essência 2D** dele, **clean-room** (só comportamento, nunca código — é GPL), e **sem viewport 3D**
(o engine é 2D por design; o próprio modo 2D do Blender ignora a profundidade). O racional completo das
três perguntas (é valioso? dá pra portar? precisa de 3D?) está no ADR-0114.

## Princípios deste módulo (inegociáveis)

1. **Mais intuitivo e fácil que o Blender.** A UX/UX-writing se aproxima dos apps de artista
   (Procreate, Callipeg, apps de vetor), não do jargão de DCC 3D. Ver a **tabela de nomes** abaixo.
2. **Integrado à Hierarchy desde o primeiro commit.** Um objeto Flip é uma entidade ECS na árvore
   única (como sprite/vetor), com `Transform` próprio e o gizmo de sprite. Nada de sistema paralelo.
3. **Painel com a cara do Inspector.** O painel docado do Flip segue o modelo visual dos inspetores de
   Sprite e Painter (seções empilhadas, tokens, widgets da Widget Gallery). Zero hex, zero f32 de UI.
4. **Camadas no idioma do Painter.** Blend/opacity/visibility/lock/grupo iguais aos do Painter — é o
   que "integrar ao sistema de camadas da sprite" significa na prática (ver ADR-0114 §Gaps).
5. **Ultra-performance por wgpu, tempo real no runtime.** O traço é expandido em GPU (vertex shader),
   troca de quadro = rebind de range (zero re-tessellação). Animação de traço roda no jogo a 60/120 Hz.
6. **Timeline principal fica pro fim.** O Flip começa com uma tira de frames própria e leve; a
   integração com a `ph2d-timeline`/dope-sheet global é a ÚLTIMA wave (a timeline nasce noutra linha).
7. **Consulte o Blender 5.2 antes de cada tópico.** Está tudo em `~/Downloads/blender-5.2-grease-pencil-ref/`
   (ver §Referência). Reimplemente do zero a partir do comportamento; nunca copie código GPL.

## Tabela de nomes — Flip vs. Grease Pencil (mais intuitivo)

| Blender (jargão) | **Flip** (intuitivo) | Por quê |
|---|---|---|
| Grease Pencil | **Flip** | a metáfora do *flipbook* — o artista entende "animação quadro-a-quadro" na hora; curto e amigável |
| Onion Skin | **Ghost Frames** | "fantasmas" dos quadros vizinhos — claro na hora |
| Keyframe (que guarda um desenho) | **Frame** / **Drawing** | é literalmente um quadro desenhado |
| Implicit Hold / Exposure | **Hold** | por quantos quadros o desenho permanece |
| Interpolate / Inbetween | **Tween** | termo do Adobe Animate, universalmente entendido |
| Sculpt Mode (nos traços) | **Reshape** | artistas conhecem *reshape*/*liquify* |
| Multiframe editing | **Edit Across Frames** | direto |
| Vertex Color | **Stroke Color** (por-ponto) | direto |
| Drawing Plane / Stroke Placement | *(removido — é 2D)* | não existe em 2D |
| Dope Sheet | **Frames** (tira) → depois Timeline | a tira leve vem antes da timeline global |

Ferramenta única **Flip** com modos (espelhando o Vector Select/Node/Pen): **Select** (default, gizmo) ·
**Draw** · **Erase** · **Fill** · **Reshape**. (Detalhe e alternativas de nome no plano.)

## Layout de crates (drop-crate, ADR-0075/0040)

| Crate | Papel |
|---|---|
| `ph2d-flip` | modelo de documento puro (layers/frames/drawings/strokes), serializável. Foundational-isolada. |
| `ph2d-flip-render` | pipeline wgpu dedicado (expansão de traço + fill + onion); pode viver em `ph2d-render`. |
| `ph2d-tool-flip` | a tool (drop-crate): modos Draw/Erase/Fill/Reshape/Select. |
| `ph2d-panel-flip` | painel docado no slot do Inspector (aparência dos inspetores Sprite/Painter). |
| componente `FlipObjectRef` | ponte entidade↔documento (espelha `VecPathRef`), em `ph2d-ecs`. |

## Referência Blender 5.2 (consulta obrigatória, clean-room)

O recorte cirúrgico do Grease Pencil 5.2 vive **fora do repo** (GPL-2.0; o PH2D é proprietário — mesma
regra do `reference/blender-texture-paint/`), em:

```
~/Downloads/blender-5.2-grease-pencil-ref/
```

**Regra:** é referência de **comportamento, nunca de código**. Leia o algoritmo, entenda, reimplemente
do zero em Rust. O doc [`02_referencia_algoritmos_blender_5.2.md`](02_referencia_algoritmos_blender_5.2.md)
já traz os extratos comentados com `arquivo:linha`.

**Índice dos arquivos-chave** (todos sob `source/blender/`):

| Tópico | Arquivo(s) |
|---|---|
| Modelo de dados | `makesdna/DNA_grease_pencil_types.h` · `blenkernel/BKE_grease_pencil.hh` · `blenkernel/intern/grease_pencil.cc` |
| Frames/camadas (editor) | `editors/grease_pencil/intern/grease_pencil_frames.cc` · `..._layers.cc` |
| **Render GPU** (ultra-perf) | `draw/intern/draw_cache_impl_grease_pencil.cc` · `draw/intern/shaders/draw_grease_pencil_lib.glsl` · `draw/engines/gpencil/` (`gpencil_engine_c.cc`, `gpencil_cache_utils.cc`, `shaders/gpencil_vert.glsl`, `gpencil_frag.glsl`) |
| Desenho / borracha | `editors/sculpt_paint/grease_pencil/paint.cc` · `paint_common.cc` · `erase.cc` |
| Fill (balde) | `editors/sculpt_paint/grease_pencil/fill.cc` · `trace.cc` · `trace_util.*` · `blenkernel/intern/grease_pencil_fills.cc` |
| Tween (interpolação) | `geometry/intern/interpolate_curves.cc` · `editors/sculpt_paint/grease_pencil/interpolate.cc` |
| Reshape (sculpt) | `editors/sculpt_paint/grease_pencil/sculpt_*.cc` |
| Curva (smooth/simplify/fit/resample/fillet) | `geometry/intern/{smooth,simplify,fit,resample,fillet}_curves.cc` |

**Re-obter em outra máquina / worktree** (per-máquina, gitignorado — reproduz o recorte):

```bash
DEST="$HOME/Downloads/blender-5.2-grease-pencil-ref"
git clone --filter=blob:none --no-checkout --depth 1 -b blender-v5.2-release \
  https://github.com/blender/blender.git "$DEST"
cd "$DEST" && git sparse-checkout init --no-cone && git sparse-checkout set \
  '/source/blender/makesdna/DNA_grease_pencil_types.h' \
  '/source/blender/blenkernel/intern/grease_pencil*.cc' \
  '/source/blender/blenkernel/BKE_grease_pencil*.hh' \
  '/source/blender/draw/engines/gpencil/**' \
  '/source/blender/draw/intern/shaders/draw_grease_pencil_lib.glsl' \
  '/source/blender/draw/intern/draw_cache_impl_grease_pencil.cc' \
  '/source/blender/editors/grease_pencil/**' \
  '/source/blender/editors/sculpt_paint/grease_pencil/**' \
  '/source/blender/geometry/**' && git checkout
```

> Nota: o núcleo numérico do Schneider fit (`extern/curve_fit_nd/`) **não** vem nesse recorte; o PH2D
> já tem um refit de Schneider próprio (`curve_refit.rs` no Painter) — reusar esse.

## Como este plano está organizado

- **Waves W0..W6** em [`01_plano_waves.md`](01_plano_waves.md), cada uma com tasks pequenas e
  critério de aceite. Ordem: **dados → render → tool → frames/ghost/tween → fill → reshape → timeline**.
- Cada tópico traz o **padrão-ouro** (o que os bons apps fazem) + **ponteiro pro Blender 5.2** + a
  **decisão PH2D**.
- **Isolamento (linha paralela / Modo L):** este módulo é desenvolvido numa linha própria por
  worktree, com handoff de integração no fim (DIRETRIZ §1.5). **Não integrar/pushar sozinho.**
