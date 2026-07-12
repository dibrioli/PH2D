# Flip — Plano de implementação (waves + tasks)

> **Decisão:** [ADR-0114](../architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md) ·
> **Visão/nomes/reference:** [`00_README.md`](00_README.md) ·
> **Algoritmos Blender 5.2 (consultar SEMPRE):** [`02_referencia_algoritmos_blender_5.2.md`](02_referencia_algoritmos_blender_5.2.md).

## Regras permanentes (valem em TODA task)

1. **Consulte o Blender 5.2 antes de cada tópico** (`~/Downloads/blender-5.2-grease-pencil-ref/`, índice
   no README). Leia o algoritmo no `02_referencia`, vá ao fonte, **reimplemente do zero** — clean-room,
   nunca copie código GPL.
2. **Padrão-ouro sem custo** (§0.6 do CLAUDE.md): a melhor opção técnica vence cronograma. Gaps in-scope
   fecham na wave.
3. **Isolamento (linha paralela / Modo L):** desenvolva na sua worktree; foundational que você criar
   (`ph2d-flip`, `FlipObjectRef`) projete pra isolamento (módulo irmão / extensão append-only) e anote ids/
   consts novos no handoff. **NÃO integre nem pushe sozinho** — feche a linha, escreva o handoff
   (DIRETRIZ §1.5.9) e PARE.
4. **UI canônica:** zero hex, zero f32-literal de UI, zero string hardcoded — tokens (`ph2d-tokens`) +
   i18n. **Mas** labels/toasts do app são **inglês** (memória `feedback_app_ui_english_only`).
5. **Widgets pela Widget Gallery** (`ph2d-panel-widget-gallery`) — é a fonte de verdade; não improvise chrome.
6. **LOC cap 700/arquivo, 200/fn** (ADR-0105 + gates de painel): transborde = extraia módulo-irmão; rode
   `fmt` (pin) antes de medir.
7. **Inner loop = `cargo check -p <crate>`**; teste/clippy/gate **1× no fechamento da wave**.
8. **Ready-to-smoke:** toda feature nova nasce com exemplo no documento demo (auto-play), nunca peça pro
   Enio montar à mão (memória `feedback_ready_to_smoke_example`).
9. **HR-5 (determinismo):** cuidado com transcendentais (`sin/cos/pow/exp` do smooth binomial, Perlin,
   hardness) — cheque se caem sob gate; prefira a forma polinomial quando existir.

## Mapa de waves

| Wave | Tópico | Entregável | Gate de fechamento |
|---|---|---|---|
| **W0** | **Dados** | `ph2d-flip` (modelo doc) + entidade ECS na Hierarchy + undo/save + amostragem por playhead | teste headless: playhead→drawing ativo; undo/redo; round-trip serde |
| **W1** | **Render GPU** | pipeline wgpu de traço/fill (ultra-perf) + compositing de camada + bench | demo auto-play renderiza; bench em `--release` registrado |
| **W2** | **Tool + Painel** | `ph2d-tool-flip` (Draw/Erase/Select) + `ph2d-panel-flip` docado + camadas UX-Painter | smoke: desenhar c/ pressão, apagar, editar brush/cor/camadas |
| **W3** | **Frames · Ghost · Tween** | tira de frames + transport + Ghost Frames (onion) + Tween (inbetween) | smoke: 2 desenhos-chave, ghost on, tween, play |
| **W4** | **Fill** | balde interativo (flood + Moore + Gap Closure) + material fill | smoke: fechar região com gap e preencher |
| **W5** | **Reshape** | escultura de traço (Smooth/Push/Thickness/…) | smoke: remodelar um traço |
| **W6** | **Timeline** (DEFERIDA) | plugar frames do Flip na `ph2d-timeline`/dope-sheet global | smoke: animação dirigida pela timeline global + handoff |

Ordem obrigatória de dependência: **W0 → W1 → W2** (base). W3/W4/W5 dependem de W2 e podem ser
paralelizadas entre si se houver linhas. **W6 é sempre a última** (a timeline nasce noutra linha).

---

# W0 — Fundação de dados + Hierarchy

**Objetivo:** o modelo de documento `ph2d-flip` (layers→frames→drawings→strokes), a entidade ECS na
Hierarchy única, undo/save e a amostragem pelo playhead. **Sem render** — tudo testável headless.

**Padrão-ouro:** o dado de animação desenhada é **cel refcontado com hold** (Blender GPv3, Callipeg,
TVPaint): um "drawing" pode ser reusado por vários frames (ciclos), e a duração é implícita (segura até o
próximo). SoA por atributo (pronto pra GPU). **Referência:** `02_referencia` §1 (frames/hold/end-sentinel,
refcount, atributos+defaults). Fonte: `DNA_grease_pencil_types.h`, `BKE_grease_pencil.hh`, `grease_pencil.cc`.

**Tasks:**
- [ ] **T0.1 — Scaffold `crates/ph2d-flip`.** Cargo + `lib.rs`, no workspace, foundational-isolada.
      `forbid(unsafe)`. Aceite: `cargo check -p ph2d-flip`.
- [ ] **T0.2 — Tipos-núcleo.** `FlipDoc`, `FlipLayer`, `FlipFrame`, `FlipDrawing`, `FlipStroke` (SoA:
      `pos: Vec<Vec2>`, `width`, `opacity`, `color: Vec<Rgba>`; por-curva: `closed`, `caps`, `hardness`,
      `material`, `fill`), ids tipados (`DrawingId`, `LayerId`, `MaterialId`). `#[derive(Clone,Serialize,
      Deserialize)]`. Ver o esboço em `02_referencia` §1 (►Decisão). Aceite: compila + teste que constrói doc.
- [ ] **T0.3 — Mapa de frames + HOLD.** `frames: BTreeMap<i32, FlipFrame>`; `FlipFrame.drawing = Option`
      (None = end-sentinel); `drawing_at(frame) -> Option<DrawingId>` via `range(..=frame).next_back()` +
      regra de end-frame. Aceite: teste tabelado espelhando GP `{0:d0,5:d1,10:end,12:d2}` (d1 aparece 5..9,
      nada 10..11).
- [ ] **T0.4 — Refcount de drawing.** `users`, `add_user`/`remove_user`, `is_instanced`, e
      `remove_drawings_with_no_users` (compacta + **remapeia** todos os `drawing` dos frames). Aceite:
      teste de compactação com remap.
- [ ] **T0.5 — Ops de frame.** `insert_frame(layer, n, hold)` (Implicit vs Fixed(dur)+end-frame),
      `insert_duplicate(instance|deep)`, `remove_frame` (converte anterior em end-frame),
      `move_frame`/`move_duplicate`. Aceite: testes espelhando `02_referencia` §1.
- [ ] **T0.6 — API de stroke/pontos.** getters/setters SoA + defaults (tabela §1), `push_point`,
      `insert_point`. Aceite: teste de construção e defaults.
- [ ] **T0.7 — Serialização.** postcard + `FLIP_SCHEMA_VERSION`. Aceite: round-trip serde de um doc rico.
- [ ] **T0.8 — Componente ECS `FlipObjectRef(u64)`.** Em `ph2d-ecs`, espelhando
      `crates/ph2d-ecs/src/vec_path_ref.rs` (`Component + Clone + Serialize + Deserialize + SimComponent`,
      carrega só a identidade). **Registrar** em `crates/ph2d-ecs/src/scene/registry.rs:222`
      (`register_ecs_components`) **e bumpar** o `assert_eq!(reg.len(), N)` em `registry.rs:286`. Aceite:
      teste do registry verde.
- [ ] **T0.9 — Ponte objeto↔entidade.** `shells/desktop/src/flip_entities.rs` (spawn/despawn + `rebuild_map`),
      espelhando `vec_entities.rs`. Aceite: criar/remover objeto Flip cria/despawna a entidade.
- [ ] **T0.10 — `ProjectState` (undo).** Adicionar campo `flip: FlipDoc` (3º campo) em
      `shells/desktop/src/undo.rs:34`; `capture`/`restore`/`canonicalize` cobrindo o Flip (a geometria
      fora do ECS precisa entrar no diff — ver o gotcha de `canonicalize` em undo.rs:90-131). Aceite:
      undo/redo de uma edição de stroke funciona (sem passo espúrio por-frame).
- [ ] **T0.11 — Save/Load.** `shells/desktop/src/project.rs` inclui o `FlipDoc` na captura (mesmo padrão do
      `VecScene`). Aceite: save→load preserva o doc.
- [ ] **T0.12 — Amostragem por playhead.** dado `Playhead::time`→frame (por FPS do doc), resolver o drawing
      ativo por camada. Aceite: teste headless "em t, camada L mostra drawing D".

**Gate W0:** `cargo test -p ph2d-flip` + arch-gates de `ph2d-ecs`/registry; teste de integração headless
(constrói doc, "anima" frames, assere drawing amostrado); undo/redo + serde round-trip. **Sem render.**

---

# W1 — Render GPU (ultra-performance, tempo real)

**Objetivo:** o pipeline wgpu dedicado que põe o traço na tela — mesmo caminho pro editor e pro runtime do
jogo. Troca de quadro pelo playhead = rebind (zero re-tessellação). Compositing de camada. Bench.

**Padrão-ouro:** o draw engine do Grease Pencil — **expansão do traço no vertex shader** a partir de
buffers de ponto, com junções em screen-space e seção transversal com falloff de hardness no fragment. É
o que dá milhares de traços animados a 60/120 Hz. **Referência:** `02_referencia` §2 (layout 3+2 texels,
`gl_VertexID`+padding, fórmula de hardness, passes, depth, ►Decisão wgpu). Fonte: `draw_grease_pencil_lib.glsl`,
`draw_cache_impl_grease_pencil.cc`, `gpencil_frag.glsl`, `gpencil_engine_c.cc`.
**Chave PH2D:** ortográfico → a matemática 3D COLAPSA (sem perspectiva; `thickness_px = raio·zoom`).

**Tasks:**
- [ ] **T1.1 — Upload SoA→GPU.** Empacotar `FlipDrawing` em storage buffers WGSL (`positions`, `widths`,
      `opacities`, `colors`, tabela de strokes com offsets/closed/caps/hardness/material). Upload 1× por
      drawing editado (dirty flag). Aceite: buffers criados; teste de empacotamento.
- [ ] **T1.2 — Vertex shader (expansão).** WGSL: ponto→quad (fita), junção **miter/bevel/round** em
      screen-space, largura/opacidade/cor por-ponto, caps round/flat, padding de adjacência (1 antes/1
      depois, +1 se closed). Sem billboard/normal (ortho). Aceite: um traço reto e um com quina renderizam
      corretos num teste visual.
- [ ] **T1.3 — Fragment shader (hardness).** Portar `mask = smoothstep(0,1, pow(clamp(1-d,0,1), mix(0,10,
      1-hard)))`, `d = dist_eixo/(w/2)`, + AA sub-pixel `*= smoothstep(0,1, w_unclamped)`. Aceite: traço com
      hardness alto = borda dura; baixo = airbrush.
- [ ] **T1.4 — Passe Flip no compositor.** Novo passe wgpu em `shells/desktop/src/render_loop/present.rs`
      (após sprites; antes/junto do Vello). Alvos color+reveal (ou premul direto). Aceite: um drawing
      hardcoded aparece na tela do app.
- [ ] **T1.5 — Batching + ordem.** Agrupar por (camada, material); ordem 2D por índice de traço (depth
      `(stroke_id+2)·2e-7` com teste GREATER, ou o z-order inteiro do PH2D). Aceite: traços novos por cima.
- [ ] **T1.6 — Fill (triângulos).** Triangular região por-stroke (reusar CDT do `ph2d-vec-boolean`/kurbo)
      **fill-first** no mesmo caminho; render da cor de fill. Aceite: um traço fechado com fill renderiza.
- [ ] **T1.7 — Compositing de camada.** Reusar o compositor GPU 22-modos do Painter
      (`crates/ph2d-render/src/layer_compositor/`) para blend/opacity/mask por camada do Flip (em vez de
      reimplementar os 6 do GP). Aceite: 2 camadas com blend Multiply/opacity compõem certo.
- [ ] **T1.8 — Troca de quadro barata.** Ao avançar o playhead, só **rebind** do range do drawing ativo —
      instrumentar e provar **zero re-tessellação/CPU** por frame. Aceite: log/counter mostra 0 rebuilds ao
      reproduzir.
- [ ] **T1.9 — Bench.** N traços × M pontos animando; medir fps em `--release` (memória
      `feedback_measure_perf_symptom_scale`). Registrar números no handoff. Aceite: alvo 60/120 Hz batido
      ou gap quantificado.

**Gate W1:** um documento demo com **animação auto-play** renderiza no app; bench registrado; smoke visual.

---

# W2 — Tool de desenho + Painel Inspector

**Objetivo:** a ferramenta Flip (drop-crate) com modos **Select/Draw/Erase**, o painel docado com cara de
Inspector, e camadas no idioma do Painter. Nasce com demo ready-to-smoke.

**Padrão-ouro:** a mão de desenho premium (Procreate/Fresco) = amostragem com **active smoothing que
assenta o traço** (a cauda congela enquanto a ponta ajusta), pressão→tamanho/opacidade por curva, e um
painel simples com Brush/Color/Layers. **Referência:** `02_referencia` §5 (Draw/Erase) + o mapa de
registro abaixo. Fonte: `paint.cc`, `paint_common.cc`, `erase.cc`.

### Registro da TOOL (sites exatos)
- [ ] **T2.1 — Scaffold `crates/ph2d-tool-flip`** implementando `Tool` (`register`/`make`). Aceite: `cargo check`.
- [ ] **T2.2 — Design + ícone.** `docs/design/tools/flip.toml` (`[tool]` id/cluster/order/icon_slug) +
      `docs/design/icons/flip.svg`. Adicionar `IconId::Flip` **em ordem alfabética** no enum
      (`crates/ph2d-editor-core/src/icons.rs`) **e** no array `ALL_ICONS` (gotcha: fora de ordem quebra
      TODOS os ícones). Aceite: `enum_order_matches_svgs` verde.
- [ ] **T2.3 — `cargo run -p ph2d-tool-sync`** (regenera registry-init + os 2 testes hand-maintained).
      Adicionar dep no shell (`shells/desktop/Cargo.toml`). Aceite: staleness gate + cluster-order + icon-slug verdes.
- [ ] **T2.4 — Modos Select/Draw/Erase.** Espelhar a arbitragem do Vector (ADR-0112): **gizmo só no
      Select** (não publica `GizmoView` nos modos de desenho); pill alterna. Aceite: em Select o gizmo move
      o objeto; em Draw o clique desenha.
- [ ] **T2.5 — Entrega de ponteiro de canvas.** `CanvasPaintTool`/`on_canvas_pointer` (como o Painter);
      pos+pressão → amostras. Aceite: arrastar no canvas gera pontos.

### Desenho / borracha
- [ ] **T2.6 — Loop de amostragem (1º corte).** Override a <2px, subdivisão por espaçamento
      (`max(spacing%·raio, 0.25px)`), pressão→largura/opacidade via **falloff curve** (reusar a do Painter,
      `HANDOFF_painter_falloff_curve`). Aceite: traço com afinamento por pressão.
- [ ] **T2.7 — Active smoothing (o "assentar").** Janela ≥8 pontos: pré-blur 3 + fit + reamostra 32 +
      convergência 0.1px (a cauda congela). É a task de sensação premium — isolada de propósito. Aceite:
      o traço estabiliza atrás do cursor sem "engolir" cantos.
- [ ] **T2.8 — Pen-up.** trim pontas raio~0 → smooth → **simplify RDP** (reusar §4) → (opc.) fit poly→bezier.
      Aceite: traço final limpo, sem jitter.
- [ ] **T2.9 — Borracha.** modos **Soft** (reduz opacidade — default, mais "pintura") + **Hard** (corta) +
      **Stroke** (apaga traço inteiro). Aceite: os 3 modos funcionam.

### Painel docado (os 6 sites + seções)
- [ ] **T2.10 — Scaffold `crates/ph2d-panel-flip`** (`impl Panel`, `ID="flip"`, `NODE_ID=ids::FLIP_PANEL`,
      `DEFAULT_VISIBLE=false`). Criar node-id em `crates/ph2d-editor-core/src/ids/chrome/flip.rs`. Aceite: compila.
- [ ] **T2.11 — Registro do painel.** `cargo run -p ph2d-panel-sync`; adicionar bloco em `EXPECTED_TYPED`
      (`crates/ph2d-panel-registry-init/src/lib.rs:93`, hand-maintained); feature-proxy no shell
      (`shells/desktop/Cargo.toml`: dep opcional + `[features]` + `default`); z-order walk em
      `crates/ph2d-editor-core/src/screens/hero/paint.rs:269` (add `ids::FLIP_PANEL`). Aceite: staleness +
      `build_typed_registry_matches_enabled_features` verdes.
- [ ] **T2.12 — Bridge de visibilidade.** `shells/desktop/src/render_loop/flip_bridge.rs` espelhando
      `vector_bridge.rs:137` (visível só com a tool Flip ativa; alterna o inspector). Downcast concreto só
      aqui (allowlist). Aceite: painel aparece/some com a tool.
- [ ] **T2.13 — Seção Brush.** size, hardness, opacity, smoothing, spacing, jitter (sliders/NumberInput com
      range — memória `reference_number_input_register_range`). Aceite: sliders editam o brush ao vivo.
- [ ] **T2.14 — Seção Color.** swatch Stroke/Fill abrindo o **BlenderColorPicker OKLCH** compartilhado
      (`register_picker_swatch` + read-back no bridge, como o Vector). Aceite: escolher cor pinta o próximo traço.
- [ ] **T2.15 — Seção Layers (idioma Painter).** lista com add/delete/reorder/group + blend/opacity/
      visibility/lock por camada (mesma UX do `ph2d-panel-painter-layers`). Aceite: criar/reordenar/ocultar camada.
- [ ] **T2.16 — Seam Painel↔Tool.** `PanelEvent` (`SetValue`/`Click` por node-id) via `ToolPanelEvent`.
      Aceite: eventos do painel chegam à tool.
- [ ] **T2.17 — Ready-to-smoke.** Autorar um objeto Flip no documento demo (algumas camadas + um par de
      frames desenhados) pra abrir e desenhar na hora. Aceite: abrir o app já mostra o objeto Flip editável.

**Gate W2:** smoke — ativar Flip, desenhar com pressão, apagar (3 modos), mudar size/hardness/cor no
painel, criar/reordenar/ocultar camadas. Todos os gates de registro verdes. LOC caps respeitados (fatiar
`paint.rs`/painel como o Vector/Painter fazem).

---

# W3 — Frames · Ghost Frames · Tween

**Objetivo:** a tira de frames leve (própria do Flip, **não** a timeline global), transport, **Ghost
Frames** (onion) e **Tween** (inbetween). Nomes intuitivos (README).

**Padrão-ouro:** Procreate Animation Assist / Callipeg — **tira visual de quadros**, onion ligado por
default, add/duplicate/hold por gesto, play/loop/pingpong; tween = "desenhe A e B, o app faz o meio".
**Referência:** `02_referencia` §1 (onion settings+defaults) e §3 (algoritmo de tween). Fonte:
`gpencil_engine_c.cc`/`cache_utils.cc` (onion), `interpolate_curves.cc` + `interpolate.cc` (tween).

**Tasks:**
- [ ] **T3.1 — Tira de frames.** widget na parte inferior do painel Flip: célula por frame (miniatura ou
      número), quadro atual destacado, **Add / Duplicate / Delete / Reorder**, arrastar para ajustar
      **Hold** (duração). Padrão-ouro Procreate (drag reorder, hold slider). Aceite: manipular quadros pela tira.
- [ ] **T3.2 — Transport.** play/pause, loop, pingpong, **FPS** do doc; roda sobre o `Playhead` (local ao
      Flip por ora). Aceite: reproduzir a animação no viewport.
- [ ] **T3.3 — Ghost Frames (onion).** N antes/depois (default 1/1), tint before/after (verde/azul),
      **fade `1/|Δ|`**, opacity 0.5; passe GPU barato re-desenhando os drawings vizinhos com tint uniforme.
      Toggle + settings no painel. Nome **Ghost Frames** (não "onion skin"). Aceite: ver os quadros vizinhos
      esmaecidos ao desenhar.
- [ ] **T3.4 — Tween (dados).** correspondência de traço **por índice** + padding ao **MÁX** + reamostra
      por arco (reusar §4 resample) + **lerp** de pos/width/opacity/color; **auto-flip** geométrico. Aceite:
      dado A e B, gera inbetween coerente; teste tabelado.
- [ ] **T3.5 — Tween (UI).** selecionar frame A→B, **Add Tween** → N inbetweens; slider de quantidade +
      **easing** (reusar `ph2d-anim::Interp`/`Easing`!). Auto-flip on por default. Nome **Tween** (não
      "interpolate"). Aceite: gerar 3 inbetweens entre dois desenhos e reproduzir.
- [ ] **T3.6 — Edit Across Frames** (multiframe, opcional). Marcar como carry-over se apertar o escopo.

**Gate W3:** smoke — desenhar 2 desenhos-chave, ligar Ghost Frames, Add Tween, dar play.

---

# W4 — Fill (balde)

**Objetivo:** balde interativo robusto para line-art (com **Gap Closure**), material de fill, preview.

**Padrão-ouro:** o fill do GP e do Toon Boom — **preencher line-art mesmo com aberturas** via extensão de
linha, com preview antes de confirmar; grow/shrink e precisão ajustáveis. **Referência:** `02_referencia`
§5 (pipeline offscreen→flood+leak→Moore→smooth+simplify; Gap Closure por extensão). Fonte: `fill.cc`,
`draw_ops.cc`.

**Tasks:**
- [ ] **T4.1 — Material de fill.** cor de stroke + cor de fill por-stroke, agrupamento `fill_id`. Aceite:
      um stroke pode ter fill.
- [ ] **T4.2 — Pipeline de fill.** render offscreen dos traços de fronteira (reusa o passe de W1 num alvo) →
      flags → **flood-fill 4-conexo** com **leak filter 3px** → **traçado Moore** da fronteira → curva/região
      de fill. Aceite: clicar dentro de forma fechada preenche.
- [ ] **T4.3 — Gap Closure (extensão).** prolongar pontas + pontos de alta curvatura por `length` (ou
      círculos que se conectam); slider interativo de gap. Nome **Gap Closure** (não "extension"). Aceite:
      preencher forma com abertura pequena fechando o gap.
- [ ] **T4.4 — Grow/Shrink + Precision + Preview.** dilate/erode do preenchimento (**Grow/Shrink**),
      resolução (**Precision**), e **preview** do fill antes de confirmar (ganho de UX). Aceite: ajustar e
      ver o resultado antes de soltar.
- [ ] **T4.5 — Fill-on-draw** (toggle): a tool Draw produz fill além do traço. Aceite: desenhar já preenche.

**Gate W4:** smoke — desenhar forma com gap, preencher, ajustar Gap Closure e Grow/Shrink.

---

# W5 — Reshape (escultura de traço)

**Objetivo:** remodelar traços já desenhados com pincéis de raio+força+queda. Nome **Reshape** (não "sculpt").

**Padrão-ouro:** o sculpt do GP e o Liquify do Photoshop — pincéis diretos e previsíveis sobre os pontos.
**Referência:** `02_referencia` §6. Fonte: `editors/sculpt_paint/grease_pencil/sculpt_*.cc`.

**Tasks:**
- [ ] **T5.1 — Modo Reshape + pincel.** raio + força + queda sobre os pontos na região. Aceite: pincel afeta
      pontos sob o cursor.
- [ ] **T5.2 — Smooth.** suavização local (binomial, §4). Aceite: alisar um traço trêmulo.
- [ ] **T5.3 — Push/Grab + Thickness + Strength.** mover pontos / mudar raio / mudar opacidade sob o pincel.
      Aceite: cada um funciona.
- [ ] **T5.4 — Pinch/Twist/Randomize/Clone** (2º corte, opcional).
- [ ] **T5.5 — Reshape multiframe** (opcional).

**Gate W5:** smoke — remodelar um traço (smooth, engrossar, empurrar).

---

# W6 — Integração com a Timeline principal (DEFERIDA — última)

**Objetivo:** plugar os frames do Flip na `ph2d-timeline`/dope-sheet/`Playhead` globais. **Só depois de
tudo acima** (a timeline nasce noutra linha; ver CLAUDE.md §5 Timeline).

**Padrão-ouro:** os quadros do Flip aparecem como chaves no dope-sheet global; o scrub da timeline dirige o
playhead do Flip; markers/loop. **Referência:** `ph2d-timeline`/`ph2d-panel-timeline` (estado atual: W2/W3;
`PropKind` é enum fechado — coordenar com o dono da timeline).

**Tasks:**
- [ ] **T6.1 — Bind frames↔timeline.** expor os frames do Flip como faixa/keys no dope-sheet (novo tipo de
      faixa ou `PropKind` — **coordenar com o dono da timeline**, é enum fechado). Aceite: keys do Flip na timeline.
- [ ] **T6.2 — Playhead unificado.** o scrub/transport global dirige a amostragem do Flip (aposentar o
      transport local do W3 ou mantê-lo como atalho). Aceite: mover a timeline global troca o quadro do Flip.
- [ ] **T6.3 — Markers/loop** integrados. Aceite: marker/loop da timeline afeta o Flip.
- [ ] **T6.4 — Handoff de integração** (DIRETRIZ §1.5.9) da linha inteira. **PARAR** — não integrar/pushar.

**Gate W6:** smoke — animação do Flip dirigida pela timeline global; handoff escrito.

---

## Deferidos explícitos (pós-plano, não esquecer)

- **2.5D multiplane** (paralaxe por-camada sobre a `Camera2d`): um `parallax_factor: f32` por camada
  aplicado ao pan/zoom → o "brilho 3D" do multiplane sem motor 3D. Barato; entra depois (ADR-0114 §Decisão 3).
- **Materiais ricos de traço** (textura de ponta, dots/squares, gradiente de fill) — o GP tem; portar sob demanda.
- **Rig/skinning** — fora de escopo (ADR-0114 §Gaps 4).
- **Congelar o contrato do `ph2d-flip`** (gate de superfície) — follow-up quando o modelo assentar.

## Definition of Done (por wave)

1. Smoke real no app (não só unit) — a wave faz algo visível/utilizável.
2. `cargo test -p <crates>` + arch-gates relevantes verdes (registry, panel LOC, tokens, no-downcast).
3. LOC caps ok (fatiar antes de medir; `fmt` no pin).
4. Zero hex/f32-literal/string hardcoded de UI; labels em inglês.
5. Ready-to-smoke atualizado no documento demo.
6. Nada de `git push`/integração — commits locais + (no fim da linha) handoff.
