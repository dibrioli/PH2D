# 03 — UI/UX do painel (inspector direito)

> **Fonte da verdade = Widget Gallery** (`crates/ph2d-editor-core/src/widget/` + showcase).
> **Referência de implementação = painel Brush** (`ph2d-panel-painter-layers`). Nada de
> "minha variação compacta" (DIRETRIZ §5.2). Zero hex / zero f32-de-UI / tudo em tokens (HR-15).

## 1. Onde mora
`ph2d-panel-painter-layers` (inspector direito do Painter). É **mode-exclusivo**: quando
`brush.is_deform`, o corpo do painel é inteiramente o Deform (early-return em
`paint_brush_body`, exatamente como `is_selection` em
[`paint_brush.rs:52-54`](../../crates/ph2d-panel-painter-layers/src/paint_brush.rs)).
Header re-rotula para **"Deform"** (arm em `paint_brush_top::header_title`).

## 2. Widgets usados (todos já existentes na Gallery) — **usar SEMPRE a variante adaptativa**
| Papel | Widget (variante responsiva) | Anchor |
|---|---|---|
| Seletor de modo (Push/Twist/…) | **`SegmentedAdaptive`** — reflui em N linhas quando estreito | `widget/segmented_adaptive.rs:74` |
| Card com título | **`Card`** `.title()` (largura fluida `body_rect`) | `widget/card.rs:16` |
| Seção colável | **`SectionHeader`** via `paint_collapsible_section` | `paint_brush_top.rs:103` |
| Slider + chip (px/%) | **`paint_slider_with_chip_layout_adaptive`** + `link_slider_number` + `display_override` | `slider_with_chip.rs:237` |
| Toggle (Freeze on/off) | **`Toggle`** (Role::Switch) | `widget/toggle.rs` |
| Botões (Reset/Apply/Invert) | **`Button`** / `IconButton` (empilham no estreito, §7) | `widget/button.rs` |

**Nota:** `mark_chip_no_stepper` está **deprecado (no-op)** desde 2026-05-24 — **não chamar**;
todo chip pinta stepper e o gate `architecture_no_chip_without_steppers` cuida disso.
(CLAUDE.md/DIRETRIZ ainda citam a chamada antiga — ignorar aqui.)

## 3. Layout — cards (de cima para baixo)

### Card A — **Mode** (`Card` sem colapso, sempre visível)
`SegmentedAdaptive` com ícones, 6 segmentos (Wave 1), reflow adaptativo:
`Push · Twist · Pinch · Wrinkle · Fold · Reconstruct`.
Pinch é bipolar → o slider **Strength** (Card B) vira central (−suga / +infla); rótulo do
segmento mostra "Pinch" e o sinal do slider decide a direção (evita 7º segmento).

### Card B — **Brush** (colável, aberto por padrão)
Espelha os 4 do Procreate + o nosso extra:
| Row | Storage | display_override |
|---|---|---|
| **Size** | slider 0..1 | px (ex. "220") |
| **Pressure** | slider 0..1 | "%" |
| **Distortion** | slider 0..1 | "%" *(oculto em Reconstruct)* |
| **Momentum** | slider 0..1 | "%" *(oculto em Reconstruct)* |
| **Strength** *(nosso)* | slider bipolar 0..1 (centro=0.5) | "−100%…+100%" via `display_override` |

### Card C — **Freeze** (colável, fechado por padrão)
- `Toggle` **"Freeze selected area"** — liga o gate que preserva a região de seleção durante o warp.
- `Button` **"Invert freeze"** (protege o complemento).
- **Sem seleção ativa:** toggle **desabilitado** + hint "Make a selection to protect areas."
  (nunca no-op silencioso — DIRETIVA §2).

### Card D — **Deform actions** (colável, aberto)
- `Button` **Reset** — descarta a deformação da sessão (volta ao pristino).
- `Button` **Apply** — baka no sprite (`request_commit`, caminho `RasterEditTool` já existente).
- `Button` **Apply & Keep** — baka mas mantém a sessão de deform ativa (padrão dos stroke-editors).
- slider **Amount** — atenua a força total do que já foi deformado (fade pós-stroke, = Procreate "Amount").

*(Waves 2-3 adicionam: Card **Transform** com segmented Uniform/Free/Distort/Warp/Puppet,
`Rect2Editor`/`NumberInput` para X/Y/W/H/Rotation, stepper de densidade da malha,
botões Add pin / Clear pins.)*

## 4. Mockup (proporção do inspector, ~300px)

```
┌──────────────────────────────── Deform ──────────┐
│ ┌─ MODE ───────────────────────────────────────┐ │
│ │ [Push][Twist][Pinch]                          │ │  ← SegmentedAdaptive
│ │ [Wrinkle][Fold][Reconstruct]                  │ │     (reflow 2 linhas)
│ └───────────────────────────────────────────────┘ │
│ ▾ BRUSH                                    ⟲       │  ← SectionHeader + reset
│   Size        ▁▁▁▁▇▁▁▁▁▁▁   [ 220 ]                │  ← slider_with_chip
│   Pressure    ▁▁▁▁▁▁▇▁▁▁▁   [ 68% ]                │
│   Distortion  ▁▁▇▁▁▁▁▁▁▁▁   [ 20% ]                │
│   Momentum    ▇▁▁▁▁▁▁▁▁▁▁   [  0% ]                │
│   Strength    ▁▁▁▁▁●▁▁▁▁▁   [ +0% ]                │  ← bipolar (centro)
│ ▸ FREEZE                                           │  ← colapsado
│ ▾ DEFORM                                           │
│   [   Reset   ]     [       Apply       ]          │  ← Button
│   [        Apply & Keep        ]                   │
│   Amount      ▁▁▁▁▁▁▁▁▇▁▁   [ 82% ]                │
└───────────────────────────────────────────────────┘
```

### Variante estreita (tablet, ~200px) — mesmo conteúdo, reflui
```
┌──────── Deform ────────┐
│ ┌─ MODE ─────────────┐ │
│ │ [Push] [Twist]     │ │  ← segmentos refluem
│ │ [Pinch] [Wrinkle]  │ │
│ │ [Fold] [Reconstr.] │ │
│ └────────────────────┘ │
│ ▾ BRUSH           ⟲    │
│   Size                 │  ← label sobe p/ linha própria
│   ▁▁▁▇▁▁▁▁  [ 220 ]     │
│   Pressure             │
│   ▁▁▁▁▁▇▁▁  [ 68% ]     │
│ ▸ FREEZE               │
│ ▾ DEFORM               │
│   [     Reset     ]    │  ← botões empilham
│   [     Apply     ]    │
│   [  Apply & Keep  ]   │
└────────────────────────┘
```

## 5. Responsividade (iPad/tablet — larguras estreitas) — **requisito de 1ª classe**

Os painéis serão **mais estreitos em tablets**. Todo componente deve degradar com
elegância conforme a largura cai. **Regra:** nada de largura fixa em px de UI; tudo
relativo à `body_rect` do card, e todo controle usa a **variante adaptativa** que já
existe na Gallery. Nada de layout que estoure/corte no estreito.

| Componente | Comportamento largo → estreito |
|---|---|
| **Mode** (`SegmentedAdaptive`) | 6 segmentos numa linha → **reflui p/ 2-3 linhas**; abaixo de um mínimo por-segmento, vira `Dropdown` (fallback). `paint_segmented_adaptive` já reflui e devolve a altura (`segmented_adaptive.rs:74`). |
| **Slider + chip** | label · slider · chip na mesma linha → quando estreito, **label sobe p/ linha própria** e slider+chip ocupam a largura toda. `paint_slider_with_chip_layout_adaptive` (`slider_with_chip.rs:237`) já faz via `slider_with_chip_is_stacked` (`:202`); a altura vem de `slider_with_chip_height` (`:210`) — **nunca** assumir altura de row fixa. |
| **Strength bipolar** | idem slider; o `display_override` ("−100%…+100%") encurta p/ "±%" se faltar espaço no chip. |
| **Freeze** (toggle + label) | label + switch na linha → label trunca com elipse antes do switch encolher (switch tem tamanho mínimo tocável). |
| **Deform actions** (Reset / Apply) | 2 botões lado-a-lado → **empilham verticalmente** abaixo do breakpoint (largura total cada). Apply & Keep já é full-width. |
| **Cards** | `Card`/`SectionHeader` são fluidos por `body_rect`; só o padding interno é token, então acompanham a largura sem ajuste. |

**Breakpoints:** derivar de tokens de espaçamento + largura mínima tocável (não px mágico).
Um helper único `deform_is_narrow(body_w) -> bool` decide o modo empilhado, chamado por
cada row (fonte única, não `if w < 240.0` espalhado — literal-px proibido de qualquer forma
pelo gate `no_magic_numeric`; usar o mesmo padrão de `slider_with_chip_is_stacked`).

**Alvo tocável:** em tablet a interação é por **dedo**, não mouse — cada hit-rect
(segmentos, chip stepper, switch, botões) respeita a altura mínima tocável dos tokens;
o stepper-invisível do chip **não** pode ficar menor que isso no estreito.

## 6. Estilo (tokens)
- Superfícies de card: `ColorToken::Bg2` + borda 1px (via `paint_card`).
- Headers: `SectionHeader` uppercase, chevron, cor-dot atribuível (herda o padrão Brush).
- Espaçamentos: `Spacing::*`; raios: `Radius::*`; sem literais.
- Segmented ativo: token de acento; ícones via `IconId` (reuso — sem SVG novo p/ Wave 1).
- Acessibilidade: cada widget emite `Node` AccessKit (gate `hr12_widgets_a11y`).

## 7. Checklist de painel (Coord, antes de mergear — DIRETRIZ §5.2/§5.3)
- [ ] Cada slider+chip com `link_slider_number(_mapped)` no `populate`.
- [ ] Storage slider+chip no mesmo espaço `0..1`; unidade só em paint via `display_override`.
- [ ] `apply_event` é forwarder thin (sem mirror manual slider↔chip).
- [ ] Seam test (`ph2d-ui-testkit`) dirige o evento real → efeito observável, por control-shape.
- [ ] Paint e hit-test do mesmo widget gateados pela MESMA condição (`is_deform`).
- [ ] Freeze desabilitado sem seleção mostra hint (sem no-op).
- [ ] **Responsivo (§5):** cada row usa a variante `*_adaptive`; altura vem do helper (nunca fixa);
      hit-rects seguem a altura pintada em ambos os layouts (largo e empilhado). Testar em ≥2
      larguras (dock desktop ~300px e tablet estreito) — o gate `architecture_panel_wiring_parity`
      valida wiring, mas o **reflow** precisa de verificação visual no smoke em largura estreita.
