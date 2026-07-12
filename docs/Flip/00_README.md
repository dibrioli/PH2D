# Flip — animação desenhada quadro-a-quadro (o meio "Grease Pencil" do PH2D)

> **Decisão de arquitetura:** [ADR-0114](../architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md).
> **Este diretório é a fonte de conhecimento do módulo** — atualizado 2026-07-12 com o estudo
> exaustivo da referência (21 relatórios: 14 leitores do fonte do GP 5.2 + 6 pesquisas
> primárias na web + análise adversarial do traço em 3 lentes).

## O mapa dos docs (o que ler para cada tarefa)

| Doc | O que contém | Leia quando |
|---|---|---|
| [`01_plano_waves.md`](01_plano_waves.md) | plano por waves (WT→W3→W4→W5→W6), **decisões cravadas**, deferidos, não-objetivos, DoD | SEMPRE — é o roteiro |
| [`02_referencia_algoritmos_blender_5.2.md`](02_referencia_algoritmos_blender_5.2.md) | os algoritmos do GP 5.2 por subsistema, com pseudocódigo, constantes e `arquivo:linha`: dados · engine · shader · AA · tween · curvas · draw/erase · fill · sculpt · frames/onion · materiais · VFX · seleção/cíclicas · GL→wgpu | antes de CADA tópico de implementação |
| [`03_traco_rasterizacao.md`](03_traco_rasterizacao.md) | o doc definitivo do traço: o tripé, o invariante, **a mordida (mecanismo provado + a evidência de que o GP tem o mesmo artefato, ABERTO)**, o fix de 4 peças que a matou, o oráculo de aparência + a bateria, kill-criteria, AA | antes de tocar `ph2d-flip-render` |
| [`04_alem_do_blender.md`](04_alem_do_blender.md) | estado da arte além do GP (com fontes): stroke rendering, inbetweening (BetweenIT/espiral log), fill (LazyBrush/trapped-ball), a paisagem dos apps (TVPaint/Toonz/Harmony/Krita), lições do redesign GPv3, tabela lib GP→crate Rust | ao decidir "faz como o Blender ou melhor?" |
| [`05_frames_ghost_tween.md`](05_frames_ghost_tween.md) | o doc da W3: o modelo de tempo (chave/hold/sentinela/vão), os ciclos, o algoritmo dos Ghost Frames, o autokey por-tool (e por que a borracha SEMPRE duplica), o tween (pareamento/padding/auto-flip), a tira | antes de tocar frames/tempo/tween |
| [`BUGS_flip.md`](BUGS_flip.md) | **os bugs cuja causa ENGANAVA** (sintoma → causa-raiz → becos → solução → lições): a mordida, o oráculo verde-com-bug, o AA da linha fina, o NaN do ponto duplicado, o grid que perdia vizinhos | ANTES de caçar um bug parecido — e depois de resolver um |
| `../HANDOFF_flip_impl.md` | tracker do que LANDOU (W0-W2 + WT + a saga das 8 rodadas do traço) | para saber o estado real do código |
| `../HANDOFF_flip_NEXT.md` | onboarding do próximo agente da linha (Modo L + primeira tarefa) | ao abrir a linha |

## O que é

**Flip** é o quarto meio de criação do PH2D, ao lado de **Painter** (raster), **Vector** (vetor
exato) e **Motion Nodes** (procedural). É o meio do **cel / animação tradicional**: desenho
quadro a quadro com traços expressivos (largura/opacidade por pressão), **Ghost Frames** (onion
skin), **Tween** (inbetweens automáticos), **Fill** (balde para line-art) e **Reshape**
(escultura de traço). A estética de Cuphead, Skullgirls, Dragon's Crown.

A referência é o **Grease Pencil** do Blender 5.2 (GPv3), portado **clean-room** (só
comportamento, nunca código — é GPL) e **sem viewport 3D** (2D-ortográfico puro: a matemática
3D do GP colapsa — sem perspectiva, `thickness_px = raio·zoom`, o plano do traço É a tela).
Racional completo no ADR-0114. **Onde a referência é comprovadamente fraca, divergimos com
justificativa registrada** — ex.: a quina do traço macio (o GP convive com o artefato,
issue #140075; nós o matamos — `03`), o pareamento do tween por índice (upgrade especificado
no `04 §2`), o pós-processo do fill (Schneider > smooth 20×).

## Estado (2026-07-12)

- **W0 (dados) + W1 (render GPU) + W2 (tool+painel+borracha+Select/gizmo): ENTREGUES e
  integrados ao main.** Detalhe: `../HANDOFF_flip_impl.md`.
- **Wave WT (o traço) FECHADA em 2026-07-12** — a "mordida" morreu: a cobertura é a **união
  global da polilinha** num único passe (janela `p0`/`p3` + vizinhos geométricos por broadphase
  + `capsule_dn` única + par clamp/fade sub-pixel). 15 testes GPU, 5 mutações provadas, custo
  real de 1.7 ms num traço de 4000 pontos. Smoke **aprovado pelo Enio**. Detalhe:
  [`03`](03_traco_rasterizacao.md).
- **Wave W3 (Frames · Ghost Frames · Tween) FECHADA em 2026-07-12** — o Flip virou app de
  ANIMAÇÃO: tira de frames com exposição, transporte + ciclos por camada, Ghost Frames,
  autokey por-tool, flip por desenho (↑/↓) e tween com auto-flip. Detalhe: [`05`](05_frames_ghost_tween.md).
  **Pendente o smoke do Enio.**
- Próximas waves: **W4 (Fill)** → W5 (Reshape) → W6 (Timeline global, **adiada até a timeline
  principal ficar pronta** — Enio 2026-07-12).

## Princípios deste módulo (inegociáveis)

1. **Mais intuitivo e fácil que o Blender.** UX de app de artista (Procreate, Callipeg), não
   jargão de DCC. Tabela de nomes abaixo.
2. **Integrado à Hierarchy desde o primeiro commit.** Objeto Flip = entidade ECS na árvore
   única, `Transform` próprio, gizmo de sprite (ADR-0111). ✓ feito no W0/W2.
3. **Painel com a cara do Inspector.** ✓ feito no W2.
4. **Camadas no idioma do Painter** (blend/opacity/visibility/lock; compositor 22-modos
   compartilhado). ✓ feito no W1/W2.
5. **Ultra-performance por wgpu, tempo real no runtime.** Traço expandido na GPU; troca de
   quadro = rebind (zero re-tesselação). Lição histórica do GP (T57829): 1 batch por
   objeto/camada, NUNCA estado por-stroke. Budgets numéricos no plano (§Decisões).
6. **Timeline principal fica pro fim** (W6; a tira própria do W3 vem antes).
7. **Consulte o Blender 5.2 antes de cada tópico** — e o `04` antes de aceitar a solução do
   Blender como teto.
8. **Oráculo modela a APARÊNCIA, não a implementação** — a lição mais cara da saga do traço
   (7 rodadas): teste visual deriva da definição do objeto, fica vermelho antes do fix, e as
   mutações têm de sangrar (`03 §5`).

## Tabela de nomes — Flip vs. Grease Pencil (mais intuitivo)

| Blender (jargão) | **Flip** (intuitivo) | Por quê |
|---|---|---|
| Grease Pencil | **Flip** | flipbook — entendido na hora |
| Onion Skin | **Ghost Frames** | claro na hora |
| Keyframe (que guarda desenho) | **Frame** / **Drawing** | é um quadro desenhado |
| Implicit Hold / Exposure | **Hold** | por quantos quadros o desenho segura |
| Interpolate / Inbetween | **Tween** | termo universal (Animate) |
| Sculpt Mode | **Reshape** | artistas conhecem reshape/liquify |
| Multiframe editing | **Edit Across Frames** | direto |
| Fill extension | **Gap Closure** | diz o que faz |
| fill_factor | **Precision** | idem |
| dilate/erode | **Grow/Shrink** | idem |
| Self Overlap (material flag) | **Self Overlap** (flag de pincel, futura) | ok como está |
| Drawing Plane / Stroke Placement | *(removido — é 2D)* | não existe em 2D |

Ferramenta única **Flip** com modos (espelhando o Vector, ADR-0112): **Select** (default,
gizmo) · **Draw** · **Erase** · *(W4)* **Fill** · *(W5)* **Reshape**.

## Layout de crates (drop-crate, ADR-0075/0040) — como está no main

| Crate | Papel |
|---|---|
| `ph2d-flip` | modelo de documento puro (objects→layers→frames BTreeMap→drawings refcount→strokes SoA), serializável |
| `ph2d-flip-render` | pipeline wgpu dedicado do traço+fill (o tripé; ver `03`) |
| `ph2d-tool-flip` | a tool (modos Select/Draw/Erase) |
| `ph2d-panel-flip` | painel docado (Mode/Brush/Color/Layers) |
| `ph2d-ecs::FlipObjectRef` | ponte entidade↔documento |
| shell | `flip_draw/erase/layers/entities/transform/gizmo_view/demo` + `render_loop/{flip_bridge,flip_pass,flip_pass_cache}` |

## Referência Blender 5.2 (consulta obrigatória, clean-room)

O recorte vive **fora do repo** (GPL-2.0; o PH2D é proprietário), em:

```
~/Downloads/blender-5.2-grease-pencil-ref/
```

**Regra:** referência de **comportamento, nunca de código**. O `02` traz os extratos
comentados com `arquivo:linha`; os relatórios completos do estudo (2026-07-12) estão
sumarizados nos docs 02-04.

**Índice dos arquivos-chave** (todos sob `source/blender/`):

| Tópico | Arquivo(s) | Doc |
|---|---|---|
| Modelo de dados | `makesdna/DNA_grease_pencil_types.h` · `blenkernel/BKE_grease_pencil.hh` · `blenkernel/intern/grease_pencil.cc` | 02 §1 |
| Engine de render | `draw/engines/gpencil/{gpencil_engine_c,gpencil_cache_utils,gpencil_draw_data}.cc` · `draw/intern/draw_cache_impl_grease_pencil.cc` | 02 §2 |
| **Shader do traço** | `draw/intern/shaders/draw_grease_pencil_lib.glsl` · `engines/gpencil/shaders/gpencil_{vert,frag}.glsl` | 02 §2b + **03** |
| Antialiasing | `engines/gpencil/gpencil_antialiasing.cc` + shaders | 02 §2c + 03 §7 |
| Draw / borracha | `editors/sculpt_paint/grease_pencil/{paint,paint_common,erase,draw_ops}.cc` | 02 §5 |
| Fill | `editors/sculpt_paint/grease_pencil/fill.cc` · `grease_pencil_image_render.cc` | 02 §6 |
| Tween | `geometry/intern/interpolate_curves.cc` · `sculpt_paint/grease_pencil/interpolate.cc` | 02 §3 |
| Reshape (sculpt) | `sculpt_paint/grease_pencil/sculpt_*.cc` + `paint_common.cc` | 02 §7 |
| Curvas | `geometry/intern/{smooth,simplify,resample,fit,fillet}_curves.cc` · `grease_pencil_segments_geom.cc` | 02 §4 |
| Frames/onion/primitivas/undo | `editors/grease_pencil/intern/grease_pencil_{frames,layers,primitive,undo,utils}.cc` | 02 §8 |
| Materiais (superfície de render) | `gpencil_shader_shared.hh` · `gpencil_draw_data.cc` | 02 §9 |
| VFX | `gpencil_shader_fx.cc` · `shaders/gpencil_vfx_frag.glsl` | 02 §10 |
| Seleção/multiframe/cíclicas | `grease_pencil_select.cc` · `grease_pencil_utils.cc` | 02 §11 |

**Re-obter em outra máquina** (per-máquina, gitignorado):

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

> Notas: o núcleo do Schneider (`extern/curve_fit_nd/`) não vem no recorte — o PH2D já tem
> DOIS Schneiders próprios (reusar; ver 04 §6). O `paint_stroke.cc` genérico (estabilizador)
> também não vem — o algoritmo é descrito no 02 §5.

## Como este plano está organizado

- **Waves WT + W3..W6** em [`01_plano_waves.md`](01_plano_waves.md) — cada task com critério
  de aceite; decisões cravadas no topo (não re-litigar).
- Cada tópico traz o **padrão-ouro** + o **ponteiro pro Blender** + a **decisão PH2D** — e,
  onde divergimos do Blender, o `04` tem a evidência.
- **Isolamento (Modo L):** linha própria por worktree; handoff no fim (DIRETRIZ §1.5).
  **Não integrar/pushar sozinho.**
