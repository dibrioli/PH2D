# Plano — Timeline geral do app: dope-sheet + graph editor + transporte

**Data:** 2026-07-08 · **Status:** PLANO COMPLETO v1 (ondas W0–W4 + backlog) · **Linha:** `line/anim`
**Regime:** esta linha executa tudo (Modo L); **integração ao main SÓ por ordem do Enio, no fim do
turno de todas as linhas** (regra vigente da jornada); smoke com o Enio no gate de cada onda.

**Base JÁ PRONTA e integrada (não re-planejar):**
- Dado: `ph2d-anim` — `Track`/`Key`/`Clip`/`AnimCurve`/`Easing`(~30 presets)/`Interp`
  (Hold/Linear/Eased/**Bezier** CSS-style)/`RationalTime` (OTIO). Traits congelados implementados
  (`AttributeEvaluator`/`AnimationCurveSampler`); golden + determinismo + dhat 0-alloc verdes.
- Relógio: `ph2d-core::Playhead` (time f64 + play/pause/seek/rate/frame) avançado 1×/tick no render
  loop; transporte por teclado (Space, `,`/`.`).
- Runtime de binding: `ph2d-timeline` — `SpriteAnimation`(ECS) + `SpriteProp` (AnimTarget opaco →
  propriedade do `Transform`) + `apply_sprite_animations` no frame; prova viva = o **painel Timeline**
  (W2/W3), mais headless (`playhead_drive`, `apply`). (Os scaffolds de smoke KeyB /
  `PH2D_TIMELINE_SMOKE=1` foram aposentados na W4.T5 — a autoria real os substituiu.)

> Escopo: a **timeline GERAL do app** — anima QUALQUER propriedade (sprite, layer do painter, param
> de motion node, vetor). NÃO é a timeline do módulo Motion Nodes (deferida lá; encaixe =
> `motion_timeline_slot` + ordem `socket > keyframe > literal`). Referência visual: theatre.js.

---

# Parte A — Fundamentos (pesquisa de campo, 2026-07-08)

## Tópico 1 — Timelines mais amadas e por quê

| Timeline | O que os usuários amam (a essência) |
|---|---|
| **After Effects** | **Graph editor**: controle fino do que acontece *entre* keys — tangentes bézier, overshoot, ease custom. Padrão-ouro de *timing*. |
| **Blender** | **Dope sheet = visão aérea**; keys como blocos que move/escala/duplica em massa. **Separação timing (dope) × valor (graph).** |
| **Spine** | Dope sheet mostra **muitas propriedades de uma vez** só com timing — legível quando o graph polui. |
| **Cavalry** | Mental model de AE com **curva de aprendizado bem menor** + procedural. Precisão sem rig. |
| **Rive** | **State machine sobre timelines**; **editor = runtime** (WYSIWYG real). |
| **Procreate Dreams** | **Timeline por gestos**, touch-first ("Performing" grava keys em tempo real). Menos intimidante. |
| **theatre.js** | **Editor visual DENTRO do app**, sobre objetos reais; graph editor expande por-faixa. |
| **Final Cut** (magnetic) | **Remove atrito estrutural** — a ferramenta some, a intenção fica. |

**Os 6 padrões transversais (o alvo):** (1) dope-sheet + graph = as 2 vistas canônicas — ter as duas
é inegociável; (2) **bézier de 1ª classe**; (3) **baixa intimidação** (gesto direto > setup);
(4) **WYSIWYG: editor = runtime**; (5) **manipulação em massa de keys**; (6) **fricção estrutural zero**.
**Anti-padrões:** graph poluído com N propriedades; spec ambígua (mostrar **segundos E frames**);
gap editor↔runtime.

## Tópico 2 — O layout canônico ("spreadsheet do tempo")

```
┌──────────────┬──────────────────────────────────────┐
│ [transporte] │  RÉGUA DE TEMPO (topo, sempre visível) │
├──────────────┼───────────╂══════════════════════════┤  ← playhead = linha vertical
│ track list   │  keyframes / barras (dope-sheet)       │    cruzando TODAS as faixas
│ (ESQUERDA,   │  ▸ faixa 1  ● ─────● ──────●           │
│  fixa,       │  ▸ faixa 2      ●────────●             │
│  colapsável) │  ...                                   │
└──────────────┴──────────────────────────────────────┘
```

1. Esquerda = lista de faixas fixa/hierárquica/colapsável; direita = área de tempo (rótulos parados,
   tempo rola). 2. Régua no topo com subdivisões. 3. Playhead = UMA linha vertical. 4. Dope↔graph no
   MESMO painel — **modelo escolhido: expand por-faixa (theatre.js)**, mais limpo que 2 modos e casa
   com `AnimCurve` por-faixa. 5. **Docado embaixo** (canvas em cima → WYSIWYG). 6. **Scrub
   bidirecional** com timecode em tempo real.

## Tópico 3 — Features das timelines importantes (condensado; status PH2D)

| Categoria | Features canônicas (exemplar) | PH2D |
|---|---|---|
| **Keyframe & interp** | Linear · Bézier/"Easy Ease" · Hold · presets (AE/Cavalry "Magic Easing") | ✅ dado |
| | Graph editor valor (AE/Blender F-Curves) · curva de velocidade | 🔜 UI (W3) · backlog |
| | Roving keys · interp espacial (motion path) | backlog |
| | **Edição em massa**: mover/escalar/duplicar/alinhar/copiar-colar | 🔜 (W0+W2/W3) |
| **Tempo & transporte** | Playhead + scrub bidirecional · play/pause · **loop range** · frame⇄timecode | ✅ parcial · loop 🔜 (W0) |
| | Markers/regions (Unity signals) · time remap | 🔜 (W4) · backlog |
| | Onion skin (raster frame-anim) | — (caso do Painter, não do core) |
| **Estrutura** | Tracks hierárquicos colapsáveis · clip nomeado · NLA/keyframe-layers blend · nesting | 🔜 (W2) · ✅ `Clip` · backlog · backlog |
| **Procedural/data** | Expressions/drivers · behaviours + **bake→keys** · data binding · state machine | — (Motion Nodes) · backlog-ponte |
| **Interação** | **Performing por gesto** (Dreams) · **auto-key** (Blender) · joystick 2D (Cavalry) | backlog-WOW · 🔜 (W4) · backlog |
| **Feedback** | Timecode/tooltip no scrub · live preview sempre | 🔜 (W2) · ✅ apply |

**Leitura:** o coração amado (interp+graph+dope+massa+transporte) é exatamente onde a fundação já
existe; a UI (W2–W3) + auto-key (W4) fecham o "funcional, poderoso e intuitivo". Pontes procedurais
ficam pro seam com Motion Nodes.

---

# Parte B — O plano

## B0. Princípios de design (guardrails — derivados da Parte A)

- **P1 Duas vistas, um painel:** dope-sheet default; graph por **expand por-faixa**. Nunca um "modo" global escondido.
- **P2 Bézier 1ª classe:** todo segmento é upgradável a bezier arrastando handles; presets a 1 clique.
- **P3 Baixa intimidação:** criar track = 1 clique com objeto selecionado; defaults bons (EaseInOut); auto-key opcional.
- **P4 WYSIWYG absoluto:** a cena responde ao playhead SEMPRE (apply já roda); zero gap editor↔runtime.
- **P5 Massa:** toda operação aceita multi-seleção (mover/deletar/copiar/escalar).
- **P6 Fricção zero + spec não-ambígua:** frame-snap; tempo exibido **em segundos E frames** juntos; nada de no-op silencioso (estado impossível =控 desabilitado com hint).
- **Anti-poluição do graph:** só faixas expandidas mostram curva; auto-fit vertical por faixa.

## B1. Aceitação v1 (CONGELADA — DoD do plano inteiro) + não-escopo

Feito = o Enio consegue, SÓ pela UI (mouse + atalhos):
1. Selecionar um sprite no canvas → **"+ Track"** → escolher propriedade (X/Y/Rotation/ScaleX/ScaleY/Opacity).
2. Inserir keys (K/duplo-clique), **mover/multi-selecionar/deletar/copiar-colar** no dope-sheet, com frame-snap.
3. **Scrub** na régua move o Playhead e a cena responde ao vivo; play/pause/loop range/go-to-start-end na barra; readouts em s **e** frames, editáveis (seek).
4. Expandir uma faixa → **graph editor**: arrastar tangentes bézier muda o easing visivelmente; presets por segmento (Hold/Linear/famílias In/Out/InOut).
5. **Auto-key**: armado, mover o sprite no canvas (gizmo/inspector) grava key no playhead (criando track se preciso).
6. **Undo/redo** cobre toda edição de timeline (1 passo por gesto).
7. **Salvar e reabrir** o projeto preserva animação e bindings.
8. Gates verdes: seam tests comportamentais (ui-testkit), wiring-parity, no_literal_color, i18n EN, LOC caps, dhat (paused = 0 alloc no caminho bridge), clippy/fmt; 60 fps na cena de referência (B5).

**Não-escopo v1 (backlog W5):** performing por gesto, speed graph, weighted/value-space tangents,
roving, NLA/keyframe-layers, time remap, multi-clip UI + nó `motion.clip`, markers→signals, MCP/Luau
(HR-10), bake procedural↔keys, export, onion skin.

## B2. Arquitetura (decisões fixadas)

1. **Relógio ÚNICO = `ph2d-core::Playhead`.** Loop range entra NELE (W0). `MotionTransport` passa a
   derivar/consumir o Playhead (W1.T7) — se a linha motion estiver viva e houver mesmo-símbolo,
   **reporta ao Enio** (§1.5.2.1); senão, esta linha faz sob o gate testado.
2. **Documento ≠ tool** (padrão vec_scene/MotionState): `AppGfx.timeline: TimelineState { doc,
   selection, history, flags }`. `TimelineDoc` vive em `ph2d-timeline` (crate-folha, sem codegen):
   `{ version, fps_display, clips: Vec<NamedClip>, active_clip, bindings, markers }` — v1 edita 1
   clip ("Main"); multi-clip é dado desde já, UI backlog.
3. **Bindings centrais no doc:** `TargetBinding { target: AnimTarget, entity: u64(bits) +
   wire_id(save), prop: PropKind }`. `AnimTarget` segue **opaco** em `ph2d-anim` (HR-8); resolução =
   consumidor. v1 resolve sprite (`Transform` + opacity); vetor/painter/nó = resolvers-irmãos futuros.
   Entity morta → track com badge "missing", **nunca** no-op silencioso.
4. **Apply doc-driven:** `apply_from_doc(world, doc, t)` (o caminho por-componente `SpriteAnimation`
   continua p/ uso programático).
5. **Graph editor SEM modelo novo:** handles de curva = manipulação do `Interp::Bezier{x1,y1,x2,y2}`
   **normalizado por segmento** (y fora de [0,1] = overshoot ✓). Weighted/value-space tangents =
   backlog. Zero mudança em contrato congelado.
6. **Dispatch:** `InteractiveState::TimelineSurface { kind: TimelineHitKind }` + canal
   `TimelineGesture` no WidgetStore — **molde `GraphSurface`/`GraphGesture`** (Motion M0.T2/T3;
   **verificar shape real antes**). Painel drena e interpreta; editor-core não conhece semântica.
7. **Painel↔bridge por canais estáticos** (molde `GraphViewSnapshot`/`GraphIntent`):
   `TimelineViewSnapshot` (tracks, keys, playhead, seleção, badges, epoch) ⇄ `TimelineIntent`
   (Scrub, Play, Pause, SetLoop, AddTrack, RemoveTrack, AddKey, MoveKeys, SetKeyValue, SetInterp,
   DeleteSelection, Copy/Paste, Bind, SetAutoKey…). Downcast só no bridge.
8. **Layout:** painel docado EMBAIXO (`timeline_slot` novo no HeroLayout, colapsável; toggle por
   chrome pill + atalho livre — verificar colisões em `handle_editor_key`); quando o split Motion
   estiver ativo, docka no `motion_timeline_slot` (W4).
9. **Persistência:** serde versionado (HR-14; postcard posicional → campo `version` explícito
   primeiro, sem confiar em `serde(default)`); entity→`wire_id` estável no save (investigar
   ADR-0037/SceneDoc). Bloqueio no save do projeto → sidecar documentado (B5).
10. **Contratos congelados tocados: NENHUM.** Sem variant novo em `PanelEvent` (canal próprio),
    sem mexer em `Tool`/`NodeOp`/`AnimValue`/`ParamSpec`.

**Crates/áreas tocadas:** `ph2d-anim` (autoria) · `ph2d-core` (Playhead.loop) · `ph2d-timeline`
(doc/bindings/apply/history) · `ph2d-panel-timeline` (NOVO) · editor-core (TimelineSurface +
layout slot + IconId + tokens) · shell (`timeline_bridge.rs`, app_state, input_handlers) ·
`ph2d-tokens`/tokens.json · `ph2d-motion-doc`/`motion_bridge` (só W1.T7/W4.T4, coordenado).

## B3. Ondas e tasks

### W0 — Autoria no dado (`ph2d-anim` + `ph2d-core`; headless, zero UI)

| # | Task | Notas |
|---|---|---|
| W0.T1 | `KeyId` estável por track (u64 monotônico) + API de autoria: `insert_key(t,value,interp)->KeyId` · `remove_key` · `move_key` (re-sort + invalida cursor) · `set_value` · `set_interp` · `key(KeyId)` | invariante ordenado preservado; testes de cada op + fuzz leve de sequências |
| W0.T2 | Helpers de mapeamento **handle⇄bezier normalizado** por segmento (px→(x,y) do `Interp::Bezier`) puros e testados | é a math do graph editor (W3) sem UI |
| W0.T3 | `Playhead.loop_range: Option<(f64,f64)>` — `advance` faz wrap determinístico; `set_loop/clear_loop`; testes bit-exatos + wrap com rate ≠ 1 | ph2d-core; é o "loop range" geral |
| W0.T4 | Serde versionado em `ph2d-anim` (Track/Key/Clip/Interp/Easing/RationalTime) + round-trip + teste de version-bump | HR-14; dep `serde` nova na crate (derives) |
| W0.T5 | Ops em massa no dado: `move_keys(&[KeyId], dt)` · `remove_keys` · `duplicate_keys` · `scale_keys(pivot, factor)` | base do P5; tudo re-sort seguro |
| W0.T6 | Re-rodar gates da crate: golden + determinismo + dhat 0-alloc (sample intocado) + clippy/fmt | fechamento W0 |

**Gate W0:** suíte `ph2d-anim`+`ph2d-core` verde; nenhuma UI. (Sem smoke visual — dado puro.)

### W1 — Documento, bindings gerais e ponte (runtime completo sem painel)

| # | Task | Notas |
|---|---|---|
| W1.T1 | `TimelineDoc` em `ph2d-timeline` (§B2.2) + `NamedClip` + `Marker {t, label}` + serde versionado + round-trip | markers são dado do doc (UI em W4) |
| W1.T2 | `PropKind` geral + `TargetBinding` (§B2.3); `SpriteProp` vira o resolver-sprite de `PropKind` | opacity: localizar o seam do componente `Sprite` (grep primeiro) |
| W1.T3 | `apply_from_doc(world, doc, t)` + liveness check + flags "missing" p/ snapshot | mantém `apply_sprite_animations` |
| W1.T4 | `TimelineState` no `AppGfx` + `TimelineHistory` (snapshot por gesto; molde vec-edit History/RECOLOR_PRE) | pan/zoom/seleção = estado do painel, não-undoable |
| W1.T5 | `render_loop/timeline_bridge.rs` (molde motion/vector_bridge): chama `apply_from_doc`, publica `TimelineViewSnapshot`, drena `TimelineIntent` → doc/Playhead/History | testável headless; buffers do snapshot REUSADOS |
| W1.T6 | Roteamento de undo: painel timeline focado/hover → Ctrl+Z vai pra `TimelineHistory` (investigar regra atual painter/image-edit; definir precedência + teste) | zero surpresa de undo errado |
| W1.T7 | **Relógio único:** `MotionTransport` deriva do Playhead (play/pause/seek/rate; tick continua inteiro do FixedStep) — coordenação §B2.1 | mesmo-símbolo com linha viva → PARE e reporte |
| W1.T8 | Persistência: mapping entity↔`wire_id` (investigar ADR-0037/SceneDoc) + save/load do doc no projeto (localizar o site de save; bloqueado → sidecar §B5) | teste save→load→sample idêntico |
| W1.T9 | Gate dhat: bridge com doc não-vazio, **paused = 0 allocs** (molde motion M0.T12) | HR-3 |

**Gate W1:** headless — intents dirigem doc+playhead e a cena muda (teste de integração no shell);
dhat verde. Smoke Enio opcional (à época, KeyB era a prova visual; aposentado na W4.T5).

### W2 — Painel Timeline v1: transporte + dope-sheet (a onda grande)

| # | Task | Notas |
|---|---|---|
| W2.E0 | Scaffold `ph2d-panel-timeline` (DIRETRIZ §3.B.1: crate + feature flag + push no registry-init + `EXPECTED_TYPED`+1) + `timeline_slot` bottom no HeroLayout + toggle (chrome pill + atalho livre — auditar `handle_editor_key`) + `panel_visibility` | DEFAULT_VISIBLE=false; gotcha: FloatingPanel não pinta |
| W2.E1 | **TimelineSurface foundational** (§B2.6): `TimelineHitKind {Background, Ruler, TrackHeader{i}, Key{track,key}, LoopBrace{side}, Marker{i}, …}` + canal `TimelineGesture` + arms nos dispatch pointer_down/move/up/scroll/key | VERIFICAR molde GraphSurface primeiro; testes de dispatch |
| W2.E2 | **Barra de transporte:** Play/Pause, GoStart/GoEnd, Prev/Next frame, chips **tempo (s) + frame** editáveis (seek; `set_number_range`+`mark_chip_no_stepper`/`link_slider_number` onde couber), fps readout do doc, Loop toggle, AutoKey toggle (arma flag; efetiva em W4) | IconId: auditar existentes (audio mixer) e adicionar faltantes em ordem alfabética |
| W2.E3 | **Régua:** paint (ticks s/frames adaptativos ao zoom) + hit + **scrub** (drag = Seek com frame-snap toggle) + tooltip timecode + linha do playhead cruzando as lanes | scrub↔playback bidirecional |
| W2.E4 | **Track list (esquerda):** rows por binding (alvo→prop), twirl colapsável por alvo, seleção de row, **"+ Track"** (popup de props do objeto selecionado; sem seleção = desabilitado com hint), remover track | nomes via snapshot (Name do entity) |
| W2.E5 | **Lanes de keys:** paint diamantes (cull ao viewport), ids dinâmicos fnv64 `"timeline_key/{track}/{key}"`, click/shift-select, **box-select** (drag em Background), **drag move multi** com frame-snap, Delete, K = key no playhead (tracks selecionadas), duplo-clique = key com valor amostrado | precedente ids: hierarchy/motion-params |
| W2.E6 | Zoom/pan do tempo: wheel ancorado no cursor (0.1×–20×), drag-pan, **F = fit** ao extent das keys; estado no `Panel::State` | consumir scroll antes do panel_scroll |
| W2.E7 | Copy/paste de keys (mesma track, offset no playhead) + duplicate | usa W0.T5 |
| W2.E8 | `TimelineViewSnapshot`/`TimelineIntent` completos + **seam tests ui-testkit** (scrub→seek; click key→seleção; drag→MoveKeys; Play→playing; +Track→binding) + `architecture_panel_wiring_parity` verde | DIRETIVA §2: todas as pontas JUNTAS |
| W2.E9 | **~14 ColorTokens** (ruler bg/tick, row/row-alt, key/key-selected/key-hold, playhead, loop-region, marker, curve/handle, badge-missing…) ×4 temas + i18n keys EN (`panel.timeline.*`) | gates no_literal_color + HR-15; lista exata na implementação |
| W2.E10 | Split de arquivos ≤700 LOC (paint/lanes/ruler/event/populate/state/intents) + behavioral-test gate do painel | `architecture_interactive_crate_has_behavioral_test` |

**Gate W2 (smoke Enio):** importar sprite → +Track X e Opacity → 3 keys → play/loop/scrub → mover
keys em massa → undo — tudo pela UI.
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop`

### W3 — Graph editor (expand por-faixa)

| # | Task | Notas |
|---|---|---|
| W3.E1 | Expand/collapse por row (twirl da faixa → row alta); múltiplas expandidas; hit `CurveHandle{track,key,which}` no TimelineHitKind | theatre.js model (B0.P1) |
| W3.E2 | Paint da curva: polyline adaptativa amostrando o próprio sampler + âncoras; **auto-fit vertical por faixa** + labels min/max; cull | anti-poluição |
| W3.E3 | **Handles bézier**: paint 2 handles/segmento (out de k0, in de k1) via W0.T2; drag → `SetInterp(Bezier)`; Hold/Linear/Eased viram Bezier ao arrastar (upgrade path) | overshoot = y fora de [0,1] ✓ |
| W3.E4 | Menu por segmento (R-click): Hold · Linear · famílias In/Out/InOut · Custom(bezier) | presets 1-clique (P2/P3) |
| W3.E5 | Edição de VALOR no graph: drag vertical da âncora = `SetKeyValue` (Float; Vec2 = par de faixas X/Y) | massa: multi-âncora |
| W3.E6 | Golden: mapping handle⇄interp bate o sampler bit-a-bit (headless) + seam test drag-handle→interp mudou | ASSERÇÃO-VERMELHA |

**Gate W3 (smoke Enio):** bola quicando — squash com overshoot editado NO graph, ao vivo.

### W4 — WYSIWYG total: auto-key, markers, loop na régua, fechamento

| # | Task | Notas |
|---|---|---|
| W4.T1 | **Auto-key:** mapear TODOS os sites de commit de Transform vindos de UI (gizmo end-drag, chips do Inspector, move tool — grep primeiro, memória "enumere todos os caminhos") → choke point único → armado: insere/atualiza key no playhead (undo agrupado no gesto) | sem choke point único → hook os enumerados + documenta gap (B5) |
| W4.T2 | Auto-create track no auto-key (opção no doc) — mover objeto sem track cria binding+track+key | P3 |
| W4.T3 | Loop braces arrastáveis na régua + **markers** (add/move/rename popup; dado W1.T1) | |
| W4.T4 | Dock no `motion_timeline_slot` quando o split Motion está ativo (slot h=0 → altura real; coordenação leve com motion) | verificar slot no layout atual |
| ~~W4.T5~~ ✅ | **Aposentado** `timeline_smoke.rs` (arquivo + `mod` + branch no render loop) + hook KeyB + helper `demo_spin_clip` (substituídos pela autoria real); comentários stale limpos (Cargo.toml/app_state/timeline_bridge); docs/CLAUDE §5 atualizados | tecla B **liberada** (cai no `_ => {}`) |
| W4.T6 | Persistência end-to-end no save real do projeto (fecha W1.T8) + migration test | HR-14 |
| W4.T7 | Fechar unificação do relógio (se W1.T7 ficou parcial) + remover transporte duplicado | |
| W4.T8 | Gate batched final da linha: nextest-impacted + clippy --all-targets + audit ≥2 lentes + DIRETIVA §3 template por claim + perf da cena de referência | fechamento do módulo |

**Gate W4 (smoke Enio = aceitação B1 inteira):** animar uma cena real só com mouse, salvar, reabrir, play.

### W5 — Backlog (pós-v1; ordem sugerida por valor)

Performing por gesto (Dreams-style, gravar durante o play) · ~~speed graph~~ ✅ **(landou
2026-07-11)** · weighted/value-space tangents · roving keys · keyframe-layers/NLA blend · time
remap · multi-clip UI + nó `motion.clip` (seam Motion) · markers→signals/eventos · API MCP/Luau da
timeline (HR-10) · bake procedural⇄keys (ponte Cavalry) · export.

**Speed graph (W5, FECHADO 2026-07-11):** 2ª vista do graph editor plotando velocidade
(`d(value)/dt`) — toggle **Speed** panel-local na barra de transporte. Math em `ph2d-timeline::speed`
(`sample_speed` diferencia a easing pura `Interp::remap` em espaço-u normalizado → fiel ao que toca,
sem spike de Hold; `speed_extent` sempre inclui a linha-zero; `out/in_handle_y_for_speed` inverte
velocidade→tangente mantendo a influência x). `graph_paint`/`graph::resolve_drag` ramificam em
`state.speed_view` reusando `CurveHandle`/`HandleDrag`. Arrastar um speed-handle reafina a tangente
do endpoint; segmento flat (dv=0) mantém o handle. `TIMELINE_SPEED` + `panel.timeline.speed`. Testes:
9 goldens de math + 3 seam (toggle · retune · flat) + mutação dirigida. **Weighted tangents** é o
follow-up natural (a edição de speed hoje mantém a influência fixa; tangentes com peso dariam o eixo
de influência completo no speed graph).

## B4. Verificação (como cada onda fecha)

- Por task: `cargo check -p <crate>`. Por onda: `scripts/nextest-impacted.sh` + clippy
  `--all-targets -D warnings` + `rustup run 1.95 cargo fmt` + audit ≥2 lentes com o TEMPLATE da
  DIRETIVA §3 (claim → traço file:line → **asserção-vermelha**).
- **Seam comportamental obrigatório** (ui-testkit) para todo controle novo — o teste é o entregável.
- dhat: sample 0-alloc (existente) · bridge paused 0-alloc (W1.T9) · sem novo contador global flaky
  (asserção por capacidade quando couber).
- Smoke visual com o Enio no gate de CADA onda (comando com `cd` incluso, W2 acima).
- **Integração ao main: SÓ por ordem do Enio** (fim do turno das linhas). Ship idem (ship.sh completo
  — a linha forkou antes de outras integrações; fmt-drift só o ship pega).

## B5. Riscos + kill-criteria (antes de construir — DIRETIVA §5)

| Risco | Critério/kill |
|---|---|
| Perf de paint do painel | Cena de referência: **50 tracks × 2.000 keys**, painel aberto, ≤ **1.0ms** de overlay (HR-4). 2 tentativas falhas → **corta escopo**: virtualização de rows + cap de faixas expandidas (≤4). |
| Topologia do dispatch (TimelineSurface) | Two-strikes: 2ª reconstrução → PARA e prova o modelo em teste antes da 3ª. Molde GraphSurface não encaixar = 1º strike. |
| Auto-key sem choke point | Fallback: hook só gizmo+inspector (enumerados); gap documentado em handoff. |
| Mesmo-símbolo com linha Motion (transport/bridge) | PARE e reporte ao Enio (§1.5.2.1). |
| Save do projeto inacessível/wire-id complexo | v1 salva `TimelineDoc` em sidecar do projeto (decisão documentada aqui); merge no save = W5. |
| Undo global conflita (painter/image-edit) | Regra de foco explícita + teste; conflito irresolvível → atalho dedicado (Ctrl+Alt+Z) temporário documentado. |

## B6. Arquivos críticos

- `crates/ph2d-anim/src/{track,curve,time}.rs` — autoria W0 (KeyId, massa, serde)
- `crates/ph2d-core/src/playhead.rs` — loop range W0.T3
- `crates/ph2d-timeline/src/{doc,bindings,apply,history}.rs` — W1
- `crates/ph2d-panel-timeline/` (NOVO) — W2/W3
- `crates/ph2d-editor-core/src/interaction/{state,dispatch}/*` — TimelineSurface (molde GraphSurface)
- `crates/ph2d-editor-core/src/screens/layout.rs` — `timeline_slot` (+ dock motion W4)
- `crates/ph2d-editor-core/src/icons.rs` + `docs/design/tokens.json` + `ph2d-tokens` — ícones/tokens W2.E9
- `shells/desktop/src/render_loop/timeline_bridge.rs` (NOVO) + `app_state.rs` + `input_handlers.rs`
- Referências vivas: `motion_bridge.rs`/`GraphViewSnapshot` (canais) · `vec-edit::History` (undo) ·
  `bgremoval_preview.rs` (bridge) · ADR-0037 (wire id)

## B7. Sequência e regime de execução

W0 → W1 → W2 → W3 → W4, nesta linha (`line/anim`), cada onda = commits locais + gate batched +
smoke do Enio; **nenhuma integração/ship sem ordem explícita**. W2 é a maior — se o Enio quiser
paralelizar, E1 (foundational dispatch) e E2–E7 (painel) são divisíveis em 2 agentes após E0/E1.
