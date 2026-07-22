# 22 — A UI COMPLETA do Wet Paint: tuning, tilt, tools e o painel lateral

> **Pedido (Enio, 2026-07-22):** trazer para a engine toda a interface de ajustes finos do app
> modelo (`docs/Painter/ph2d_wet_paint`): no painel do Painter ficam os ajustes básicos — o
> **Tilt** (cópia perfeita do dial), os básicos do pincel, **Wet Canvas / Dry Canvas / Fast Dry /
> Show Wet**, o checkbox **Paper** (a textura vira visualmente parte da pintura) e um checkbox novo
> **Tuning** que abre um **segundo painel na lateral** com todos os ajustes finos
> (PAINT / WATER / PHYSICS / TOOLS / PAPER / EXPERIMENTAL). Tudo integrado ao sistema basal.

## §1 — O que o estudo estabeleceu (não re-derive)

- O engine (`ph2d-wet-paint`) **já portou os 53 knobs** — `tuning/defs.rs::KNOB_DEFS` é a casa
  ÚNICA de `{key, group, min, max, step, default, rebuild}`. 40 são dos grupos visíveis
  (Paint 13 · Water 7 · Physics 10 · Tools 4 · Paper 6) e 13 são as extensões §17 (`Hidden`,
  default-neutras, **sem UI — no modelo também não têm**).
- As 7 tools (`Tool::{Paint,Erase,Smear,Blend,Wet,Dry,Blow}`) estão portadas e o dispatch
  por-tool existe (`finish_dispatch`); **só Paint/Erase têm superfície de produto** (lanes +
  erase shaped). As lane doors JÁ são blend-aware no plumbing (`begin_direct_stroke` escolhe
  `TrailMode::Blend` por `engine.tool`; `direct_segment` cobre Paint|Blend).
- Tilt existe no engine (`Sim.tilt_on/tilt_dir_x/tilt_dir_y/tilt_scale`), boot ON/(0,1)/1.0 —
  **exatamente** o boot do dial do modelo (ring 4, spoke 3 = reto pra baixo).
- Ações prontas: `action_wet_canvas` / `action_dry_canvas` / `action_fast_dry`; flags prontos:
  `engine.show_wet`, `engine.km_glaze`, `sim.km_mixing`.
- O render de produto é `render_pigment_only_region` (pigmento puro, alfa real — SEM os termos
  de papel). O `render_region` completo pinta a FOLHA (opaca) — errado para camada.
- Mecanismo canônico de knob (W3): estado AUTORADO no tool (`WetPaintState.knobs`, **f64**),
  reconcile no batch E no tick via `Engine::set_knob` (reage a `Rebuild`), boot-equivalência
  EXATA gateada.

## §2 — Decisões de arquitetura

1. **UMA casa para os valores**: `WetKnobs` vira o armazém CHEIO —
   `{ water: f64, erase: f64, knobs: [f64; KNOB_COUNT] }` (sliders do engine + todos os knobs
   do registry). Os 5 curados que eram campos (`pigment`/`pickup`/`dry_speed`/`edge_darkening`/
   `gravity`) viram **acessores** sobre o array — a seção básica e o painel Tuning leem/escrevem
   **o mesmo valor** (duas vistas, um rádio; a alternativa — overlay separado — é a doença das
   duas portas). Defaults do array = `KNOB_DEFS` via const fn (engine é a fonte).
   Clamp de escrita = `safe_clamp(def.min, def.max)` — a "2ª cópia deliberada" de ranges do W3
   fica restrita às consts PINTADAS do painel.
2. **Ids do painel Tuning são DERIVADOS por chave em runtime** (`wet_tuning_*_id(key)`), o
   precedente das famílias dinâmicas do `node_id_collisions`; a unicidade é gateada no crate do
   painel sobre as chaves REAIS. Sem 117 consts, sem drift painel↔tabela.
3. **O painel constrói-se da tabela** (a lei do modelo: *"the panel builds itself from this
   table"*): as seções iteram `KNOB_DEFS` filtrando grupo; labels de UI são a única tabela local
   (vocabulário, gateada como completa sobre os grupos visíveis).
4. **Tilt** = estado autorado `{tilt_on, tilt_ring 0..8, tilt_spoke 0..11}` (boot `true, 4, 3` =
   boot do engine), reconciliado em `sim.tilt_*`. Direção: cardinais (spoke 0/3/6/9) em valores
   EXATOS (senão o reconcile de boot escreveria `cos(π/2) = 6.1e-17` sobre o `0.0` do engine);
   demais spokes via `libm`. Escala = `ring/4` (anel 4 = 1.0 = a magnitude do knob Gravity).
   Dial: cópia do modelo — 8 anéis × 12 raios, snap à grade, arrastar liga, toggle preserva
   direção. Interação = `InteractiveState::CurvePoint` (drag 2D normalizado) → polar no event.
5. **Wet tools** = `WetPaintState.tool: WetTool` {Paint,Smear,Blend,Wet,Dry,Blow} + o botão
   **Erase** que é a OUTRA VISTA do chip Eraser do rail (o precedente do impasto: escolher usa;
   Erase → wire "eraser", os demais → wire "brush"). Reconcile escreve `engine.tool` (dá o
   `TrailMode::Blend` e o `sim_should_run` do Blow de graça; erase força `Tool::Paint` no engine
   — a sim pausa sob a borracha como no modelo).
   Rotas de dab: Paint = lanes (INTOCADO, byte-idêntico) · Blend = lanes com porta nova
   `dispatch_pressure_dab_lane_blend` (accumulate/transfer_blend; TOOL_HARDNESS, intensity ≤ 3)
   · Wet/Dry/Blow/Smear = porta nova `dispatch_pressure_dab_tool(tool, …, prev)` com prev
   POR LANE (o prev singular do engine interleaved com cópias de Symmetry daria deslocamentos
   gigantes); Smear pula o 1º dab (sem deslocamento — lei do modelo), Blow usa prev=self no 1º.
   As tools usam o falloff do ENGINE (TOOL_HARDNESS fixo) — **fidelidade ao modelo**, que ignora
   o shape do pincel nas tools; silhueta do Painter continua dirigindo APENAS o depósito Paint.
6. **Ações de canvas** (one-shot no Click): sem sessão viva, **Wet Canvas cria uma** (molhar a
   folha para o próximo traço sangrar É o caso de uso); Dry/Fast Dry sem sessão = no-op honesto.
   Depois da ação: `mark_dirty_full` + composite.
7. **Show Wet** = bool autorado; o overlay (escurecimento frio + brilho de menisco, fórmulas
   EXATAS do `render_region`) roda TOOL-SIDE no `wetpaint_composite`, sobre o resultado
   composto. **O bake exclui o véu**: encerrar sessão com Show Wet ligado força um composite
   final limpo (gate: bake com/sem véu é byte-idêntico).
8. **Paper (checkbox)** = bool autorado `paper_visual` (default OFF — o look atual aprovado não
   muda; ON = a textura entra na PINTURA): o composite chama a variante nova
   `render_pigment_region_visual` que soma o offset de granulação `v` (+ emboss, ambos ×
   `PaperVisibility`/`VisualGrain`/`Emboss`) DENTRO das cores do pigmento, alfa intacto — a
   metade "parte da tinta" do render do modelo, sem a FOLHA (que numa camada seria papel opaco
   por cima da arte). OFF delega ao caminho de hoje (mesmo corpo, byte-idêntico).
   O header do grupo PAPER no painel Tuning tem o olhinho (0 ↔ último visível) = o MESMO bool.
   Contrast/Fibres/Grooves (rebuild `Paper`) só aparecem quando o Paper SLOT do artista NÃO está
   armado (com slot armado o tile do engine não é a fonte — knob morto se escondem, lei 3);
   rebake pós-reconcile só quando `paper_key.is_none()`.
9. **EXPERIMENTAL**: `km_mixing` → `sim.km_mixing` (mistura K–M em toda a sim);
   `km_glaze` → empilhamento por reflectância do par settled→suspended DENTRO do render de
   pigmento (a folha é o canvas do app — divergência documentada do modelo, onde a base é o
   papel). Ambos bools autorados, default false.
10. **Painel lateral** = crate nova `ph2d-panel-wet-tuning` (ID `wet_tuning`), rect derivado do
    slot do inspector (mesma altura, à ESQUERDA do painter-layers), scroll próprio, visibilidade
    dirigida pelo `painter_bridge` (painter ativo ∧ armado ∧ `tuning_open`), `bump_panel_z` no
    edge. Recebe `BrushSettings` inteiro (o padrão `set_current_brush`). Eventos pelo canal
    genérico `ToolPanelEvent` de sempre → `route_brush_wetpaint_event` (estendido).
11. **Zero contrato tocado**; todas as portas de engine são NOVAS (o fingerprint pina uma sessão
    scriptada que não as chama — seguro por construção; gate roda no fechamento).

## §3 — Waves

- **W1 (engine)**: `dispatch_pressure_dab_lane_blend` · `dispatch_pressure_dab_tool` ·
  `render_pigment_region_visual` (paper `v`+emboss+master, braço glaze; off = delega) +
  gates de paridade (porta nova ≡ `finish_dispatch` com o tool armado, bit-exato) +
  fingerprint/aceitação intactos.
- **W2 (tool)**: `WetKnobs` cheio + tilt + `WetTool` + ações + show_wet/paper_visual/km +
  reconcile estendido (tool→engine.tool, tilt→sim, rebake gateado por paper_key) + rotas de
  evento (ids novos + famílias dinâmicas) + snapshot → `BrushSettings` + gates (boot EXATO em
  53 knobs; cada família de rota; véu não baka; off byte-idêntico).
- **W3 (seção básica)**: Tools ·7 (segmented, vistas do rádio) · card Tilt (dial fiel) ·
  4 botões de canvas · checkbox Paper · checkbox Tuning + seams que CLICAM.
- **W4 (painel lateral)**: crate + registry/panel-sync + features do shell +
  `SHELL_DRIVEN_PANELS` + bridge + z-order + rows das 5 seções + EXPERIMENTAL + resets
  por-knob e por-grupo + seams.
- **W5**: gate batched + mutações + handoff (smoke: `PH2D_WETPAINT_SMOKE=1`).

## §4 — Fora de escopo (nomeado)

Tooltips ricos (KNOB_DOCS) · input numérico que excede o range do slider (nossa casa clampa) ·
camadas múltiplas do engine wet (o produto usa 1) · presets Cold/Rough/Hot do papel do engine
(o Paper SLOT do artista é o sistema canônico; o tile interno segue como fallback) · Undo/Redo
internos do engine (o undo é do app).
