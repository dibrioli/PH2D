# HANDOFF — 4 Image Tools em paralelo (drop-crate)

**Status:** ABERTO 2026-05-23. 4 sessões Implementador paralelas.
**Pre-work Coord (concluído):**

- `ccf0cf0` — 4 SVGs (Lucide-portados) + 4 `IconId` variants alfabéticos em `editor-core/src/icons.rs` + 4 `docs/design/tools/<slug>.toml`.
- `28e1761` + `0cd3017` + `b59384d` + `99c8a40` + `9a0cb80` — Fase 0 multi-select foundation (`GizmoStateGroup::extra_selection` + `iter_selected` + `add_to_selection`/`toggle_in_selection`/`replace_selection`/`clear_all_selection`, Hierarchy/canvas Shift/Cmd-click, rubber-band, group translate, multi-row highlight, smart click).
- `821683a` — 3 panel crate stubs (`ph2d-panel-color-equalization`, `ph2d-panel-equalize-sizes`, `ph2d-panel-upscale`) prontos pro Implementador preencher.

**Como o Enio dispara:** abre 4 sessões Claude Code, cola um dos 4 briefings abaixo em cada uma. Cada Implementador trabalha **só dentro de 2 pastas exclusivas** (tool crate + opcional panel crate). Zero colisão por construção (workspace `members = ["crates/*"]` glob; superfícies centrais reservadas pelo Coord acima).

**Convenções comuns às 4 sessões** (referência rápida):

- **Pasta tool:** `crates/ph2d-tool-<slug>/`.
- **Pasta panel (quando sabor 3):** `crates/ph2d-panel-<slug>/` (stub já criado).
- **Wiring (zero edit central):** `cargo run -p ph2d-tool-sync` regenera `ph2d-tool-registry-init` automaticamente. Gate de staleness em CI exige a regen.
- **Multi-sprite obrigatório:** todos os 4 leem `hero.gizmo.iter_selected()` no shell drain; pra tools "per-sprite" (Color EQ / Rasterize / Upscale), o `chrome/image_actions.rs` já faz broadcast (1 `OneShotImageOp` por sprite); pra tools "cross-sprite" (Equalize Sizes), 1 push e o drain itera.
- **Zero deps externas:** implementação 100% própria de cada algoritmo. Não puxe `image::imageops::resize` nem nada parecido. Os kernels (CLAHE, Mitchell-Netravali, Lanczos3, xBR) são canônicos e portáveis.
- **GPU avaliada por algoritmo:** sempre implemente o CPU path primeiro (correto, determinístico, testável). Para cada operação que é **embaraçosamente paralela por-pixel** (filter kernels, color adjusts, simple LUT), considere escrever um **WGSL compute shader** + **parity test** (CPU vs GPU output dentro de ε=0.5/255 por canal). Se o trabalho de WGSL for desproporcional ao ganho (algoritmo branchy, state cross-pixel, neighborhood-dependent), CPU-only com justificativa medida é aceitável.
- **Sabor §3.8.3** da DIRETRIZ é o seu template: (1) one-shot stateless = só `pub fn register`; (3) stateful + panel = `register` + `make` + crate-irmão panel.
- **`ImageEditTool` trait** está **definido mas não é o padrão atual** (§3.8.3.1 DIRETRIZ): nenhum tool de produção implementa hoje. BgRemoval e Padding usam métodos próprios da concrete type alcançados via `as_any_mut` downcast no shell. **Siga esse padrão**; só implemente `ImageEditTool` se for fazer uma migração-vertical do canal genérico (fora do escopo).
- **Smoke por tool:** ao terminar, reporte SHA local + `cargo test -p ph2d-tool-<slug>` + `-p ph2d-panel-<slug>` (quando aplicável) + `-p ph2d-tool-registry-init` (staleness gate).

---

## 1. BRIEFING — Color Equalization

```
═══════════════════════════════════════════════════════════════════
BRIEFING — tool-crate · slug: color_equalization  (sabor 3 stateful+panel)
═══════════════════════════════════════════════════════════════════

PASTAS EXCLUSIVAS:
  crates/ph2d-tool-color-equalization/         (você cria)
  crates/ph2d-panel-color-equalization/        (stub Coord; você preenche)

ANTES DE CODAR: leia
  - docs/IntegracaoMultiAgente/DIRETRIZ.md §3.8 (drop-crate fan-out)
  - docs/IntegracaoMultiAgente/DIRETRIZ.md §3.8.3 (sabores de tool)
  - crates/ph2d-tool-padding/src/* (template sabor 3 leve)
  - crates/ph2d-tool-bgremoval/src/* (template sabor 3 completo — preview cap + downcast)

O QUE FAZER (algoritmo + UX):

ALGORITMO — CLAHE (Contrast Limited Adaptive Histogram Equalization,
Zuiderveld 1994, *Graphics Gems IV*, pp. 474-485). Implementação
própria 100% (zero deps).

  1. Converte RGBA8 → luminância L (BT.709: 0.2126·R + 0.7152·G + 0.0722·B).
  2. Divide a imagem em tiles `tileGridSize × tileGridSize` (default 8×8).
  3. Para cada tile: histograma de 256 bins; aplica clipLimit (default
     2.0) — redistribui contagens > clipLimit equilibradamente entre
     todos os bins; computa CDF normalizada.
  4. Para cada pixel: interpola bilinearmente as 4 CDFs dos tiles
     vizinhos (canto/aresta usam apenas as válidas). O valor mapeado
     é aplicado à luminância; reconstrói RGB preservando matiz via
     scale = L_new / L_old (clamp anti-zero).
  5. Pós-CLAHE: brightness + contrast + saturation lineares em RGB
     espaço linear (sRGB → linear → adjusts → sRGB).
  6. Auto-WB Gray-World: média RGB do canal → gain = mean_gray /
     mean_channel, aplicado a cada pixel.

UI (sliders + Apply/Cancel, panel docado):
  - clip_limit (1.0–4.0, default 2.0).
  - tile_grid_size (4–16, default 8).
  - brightness (-1.0–+1.0, default 0).
  - contrast (0.5–2.0, default 1.0).
  - saturation (-1.0–+1.0, default 0).
  - Toggle: auto_wb (default off).
  - Botão: Apply (commit todos sprites selecionados).
  - Botão: Cancel (fecha panel sem aplicar).
  - Preview live: redesenha thumbnail RGBA do primary sprite a cada
    edit (cap 512×512 — mesmo padrão do BgRemoval).

GPU AVALIAÇÃO (induzido):
  - Histograma de tile: SERIAL por-tile (atomic counters em GPU
    funcionam mas com contenção alta). CPU vence: tile 8×8 com 64 bins
    úteis × 64 tiles = ~4 ms. Mantenha CPU.
  - Bilinear interp + apply do mapping LUT: TRIVIALMENTE PARALELO
    per-pixel. **WGSL compute shader** vale 10-20× speedup em 4K. Use.
  - Brightness/Contrast/Sat lineares: TRIVIALMENTE PARALELO. WGSL.
  - Auto-WB: reduce paralelo (sum por canal) + apply paralelo. GPU vale.
  - Parity test obrigatório: rode CPU e GPU sobre input fixo, asserte
    erro ≤ 0.5/255 por canal RGB (pixel-by-pixel).

ARQUIVOS DO TOOL CRATE:
  src/lib.rs              — MANIFEST (ToolManifest) + pub fn register + pub fn make
  src/manifest.rs         — opcional (se quiser separar)
  src/tool.rs             — impl Tool + handle_panel_event (rota NodeId → UiEdit)
  src/algorithm.rs        — clahe(rgba, params) -> Vec<u8> + adjusts + auto_wb (CPU)
  src/shader.wgsl         — apply LUT + adjusts (GPU compute) [se aplicável]
  src/icon.rs             — BezPath portado de docs/design/icons/color-equalization.svg
  src/params.rs           — ColorEqualizationUiEdit / Snapshot / Params
                            + apply_ui_edit (single-source-of-truth de clamps)

ARQUIVOS DO PANEL CRATE (preencha o stub):
  src/lib.rs              — Panel impl (ID, NODE_ID, paint, apply_event, populate)
  src/state.rs            — ColorEqualizationPanelState + UI snapshot mirror
  src/paint.rs            — render dos sliders + toggles + Apply/Cancel
  src/event.rs            — Click/SetValue/Toggled → PanelEvent → ToolPanelEvent
  src/populate.rs         — registrar NodeIds dos sliders/toggles/botões
  src/ids.rs              — NodeId consts (hash_node_id("color_eq.<chip>"))

MANIFEST (ToolManifest):
  id          = "color_equalization"
  label_key   = "tool.color_equalization.label"
  cluster     = "image_tools"
  zone        = Zone::TopRight
  order       = 90              (já reservado pelo Coord)
  a11y_role   = Role::Button
  handler     = ToolHandler::Stateful { on_activate, on_deactivate, on_panel_event }
  memory_budget = MemoryBudget::new(0, 0, 0)  (sem state global; preview cap fica
                  no estado interno do tool, contabilizado lá)
  touches_sim = false
  icon_fn     = color_equalization_icon_bezpath  (de src/icon.rs)
  mcp         = McpExposure::reserved()

VOCAB (params.rs, single-source-of-truth):
  pub enum ColorEqualizationUiEdit {
      ClipLimit(f32), TileGridSize(u32),
      Brightness(f32), Contrast(f32), Saturation(f32),
      AutoWb(bool),
  }
  pub struct ColorEqualizationUiSnapshot { /* mirror dos valores live */ }
  pub struct ColorEqualizationParams { /* idem + métodos clamp */ }
  pub fn apply_ui_edit(params: &mut ColorEqualizationParams, edit: ColorEqualizationUiEdit) {
      // clamps centralizados aqui — handle_panel_event roteia mas
      // NUNCA duplica os clamps.
  }

ÍCONE:
  - SVG já existe em docs/design/icons/color-equalization.svg (Coord).
  - Porte para BezPath em src/icon.rs (estilo Lucide 24×24 stroke).
  - NÃO mexa em editor-core/src/icons.rs — ColorEqualization variant
    já está registrado em ordem alfabética pelo Coord.

MULTI-SPRITE:
  - O shell drain faz broadcast: 1 OneShotImageOp por sprite em
    hero.gizmo.iter_selected(). Cada chamada do tool sobre 1 entity.
  - Apply commit: read source, run CLAHE+adjusts, write via
    texture_edit::commit_edited_texture. Iterar nada explícito no tool.

GATING:
  - PALETTE manifest cluster "image_tools" auto-gated por mode_on.
  - Painel docado abre quando o tool é ativado (set_active in shell
    drain de ActivateTool — já wired).

O QUE NÃO TOCAR:
  - Qualquer arquivo fora das suas 2 pastas.
  - crates/ph2d-editor-core/src/tool.rs (Tool / ImageEditTool /
    PanelEvent — contrato congelado ADR-0040 §7).
  - crates/ph2d-editor-core/src/action_bus.rs (EditorAction).
  - editor-core/src/icons.rs (variant já registrado).
  - Cargo.toml raiz (glob cobre).
  - ph2d-tool-registry-init (gerado por sync).

WIRING:
  cargo run   -p ph2d-tool-sync                   # regenera o wiring
  cargo test  -p ph2d-tool-color-equalization
  cargo test  -p ph2d-panel-color-equalization
  cargo test  -p ph2d-tool-registry-init          # staleness gate

NOMES (gates):
  manifest id = "color_equalization", único cross-crate.
  label_key   = "tool.color_equalization.label".
  panel NODE_ID = hash_node_id("panel.color_equalization") (já no stub).
  Vocab UiEdit ids = hash_node_id("color_eq.<chip>") — defina em ids.rs.

VALIDAÇÃO (codificação rápida):
  cargo check  -p ph2d-tool-color-equalization
  cargo test   -p ph2d-tool-color-equalization
  cargo clippy -p ph2d-tool-color-equalization --all-targets -- -D warnings
  cargo fmt    -p ph2d-tool-color-equalization

SE PRECISAR DE ALGO FORA DAS PASTAS (dep externa, mudança no contrato
congelado, variant novo em EditorAction): PARE e reporte ao Enio (§2.4).
Quase sempre significa que a tarefa não era fan-out puro.

QUANDO TERMINAR, reporte ao Enio:
  "Color Equalization pronto. Commit local: <sha>.
   cargo test -p ph2d-tool-color-equalization e -p ph2d-tool-registry-init verdes."
═══════════════════════════════════════════════════════════════════
```

---

## 2. BRIEFING — Equalize Sizes

```
═══════════════════════════════════════════════════════════════════
BRIEFING — tool-crate · slug: equalize_sizes  (sabor 3 stateful+panel)
═══════════════════════════════════════════════════════════════════

PASTAS EXCLUSIVAS:
  crates/ph2d-tool-equalize-sizes/             (você cria)
  crates/ph2d-panel-equalize-sizes/            (stub Coord; você preenche)

ANTES DE CODAR: idem briefing Color Equalization (DIRETRIZ §3.8, §3.8.3;
templates ph2d-tool-padding/, ph2d-tool-bgremoval/).

O QUE FAZER (algoritmo + UX):

ALGORITMO — Normalização de canvas size sobre N sprites selecionados.
A tool é a única **cross-sprite** das 4: o shell drain NÃO faz broadcast
1-por-sprite. O Apply lê iter_selected() e roda o loop interno aqui.

  1. Inputs do panel (snapshot):
     - target_mode: enum { MaxOfSelection, Fixed(u32, u32), GridUnit(f32) }
     - upscale_if_smaller: bool (revela algorithm dropdown se true)
     - upscale_algorithm: enum { Lanczos3, Nearest, Xbr }
     - rasterize_after: bool (cada sprite, bake do scale/rotation/flip)
     - arrange_on_grid: bool (futuro; em v1 deixe stub off)
  2. Computa target_w × target_h:
     - MaxOfSelection: max(sprite.size·scale) sobre cada sprite em iter_selected.
     - Fixed: usa W,H direto.
     - GridUnit: snap target ao grid (sprite.size·scale arredondado pro
       múltiplo de grid_unit).
  3. Para cada sprite em iter_selected():
     a. Se sprite.size × scale < (target_w, target_h) E upscale_if_smaller:
        chama o algoritmo de upscale (chame ph2d-tool-upscale como
        biblioteca pública OU duplique o kernel — escolha a forma menos
        invasiva; dependência cross-tool é permitida se ambos publicam
        as funções puras).
     b. Senão: aplica scale ratio para fit no target.
     c. Se rasterize_after: chama Mitchell-Netravali resample
        (ph2d-tool-rasterize ou duplique) e reseta sprite scale=1.
     d. Commit via texture_edit::commit_edited_texture.

  Caso degenerate (iter_selected vazio): no-op + toast "No sprites
  selected".

UI (panel docado):
  - Radio group target_mode (3 opções).
  - NumberInputs W + H (visíveis se target_mode == Fixed).
  - Slider grid_unit (visível se target_mode == GridUnit).
  - Toggle upscale_if_smaller.
  - Dropdown algoritmo (visível se upscale_if_smaller true).
  - Toggle rasterize_after.
  - Botão Apply.
  - Botão Cancel.
  - Preview: opcional — mostra "Final size: WxH px" string só.

GPU AVALIAÇÃO (induzido):
  - O algoritmo principal é orquestração (CPU trivial).
  - O custo está em upscale + rasterize internos (delegado às outras
    tools). Cada uma faz sua avaliação de GPU.
  - Em Equalize Sizes em si: CPU-only justificado, sem WGSL próprio.

ARQUIVOS DO TOOL CRATE:
  src/lib.rs              — MANIFEST + pub fn register + pub fn make
  src/tool.rs             — impl Tool + handle_panel_event
  src/algorithm.rs        — equalize_sizes(sprites_iter, params) -> Vec<EditedSprite>
  src/icon.rs             — BezPath de docs/design/icons/equalize-sizes.svg
  src/params.rs           — UiEdit/Snapshot/Params + apply_ui_edit

ARQUIVOS DO PANEL CRATE (preencha stub): mesma estrutura do Color Eq.

MANIFEST:
  id        = "equalize_sizes"
  label_key = "tool.equalize_sizes.label"
  cluster   = "image_tools"
  zone      = Zone::TopRight
  order     = 100       (Coord reservou)
  handler   = ToolHandler::Stateful { ... }
  memory_budget = MemoryBudget::new(0, 0, 0)
  touches_sim = false
  icon_fn   = equalize_sizes_icon_bezpath

MULTI-SPRITE: leia `hero.gizmo.iter_selected()` no algoritmo. NÃO faça
1-OneShotImageOp-por-sprite no chrome — é cross-sprite. O chrome
emite 1 push (OneShotImageOp{tool_id="equalize_sizes", entity_bits:primary}),
e o shell drain de equalize_sizes ignora o entity_bits e itera
iter_selected. (Implementador: você adiciona um arm específico no
shells/desktop/src/render_loop/image_edit.rs? **NÃO** — é fora da sua
pasta. Em vez disso, na sua tool, dispare via `ActivateTool` (que
abre o panel), e o Apply do panel já roda no path de stateful bake
do BgRemoval — você define o método na concrete type e o shell
alcança via downcast.) Ver bgremoval/padding como template — eles
têm `pub fn run_full_resolution()` na concrete type chamado pelo
shell pós-bake.

O QUE NÃO TOCAR: idem briefing 1.

WIRING / VALIDAÇÃO / NOMES / REPORT: idem briefing 1.
═══════════════════════════════════════════════════════════════════
```

---

## 3. BRIEFING — Rasterize

```
═══════════════════════════════════════════════════════════════════
BRIEFING — tool-crate · slug: rasterize  (sabor 1 one-shot stateless)
═══════════════════════════════════════════════════════════════════

PASTAS EXCLUSIVAS:
  crates/ph2d-tool-rasterize/                  (você cria)

(Sabor 1 — sem panel docado. Pill no chrome dispara algoritmo puro.)

ANTES DE CODAR: leia DIRETRIZ §3.8.3 (one-shot) + templates
ph2d-tool-trim-transparency/ + ph2d-tool-make-square/ + ph2d-tool-real-size/.

O QUE FAZER (algoritmo):

ALGORITMO — Bake do transform atual (scale + rotation + flip) no pixel
buffer do sprite, depois reseta scale=1.0 + rotation=0.

  1. Lê o sprite source (RGBA8) + o seu Transform atual.
  2. Compute novo tamanho output:
     w_new = (size.w * |scale.x|).round() as u32
     h_new = (size.h * |scale.y|).round() as u32
  3. Resample com kernel Mitchell-Netravali (B=1/3, C=1/3 — Mitchell &
     Netravali 1988, "Reconstruction Filters in Computer Graphics",
     SIGGRAPH'88). Suporte de filtro = 4 amostras (radius 2). Para
     cada pixel destino (x_d, y_d): mapeia para origem (x_s, y_s) =
     (x_d/scale.x, y_d/scale.y); soma kernel*source nos 4×4 pixels
     vizinhos. Implementação 100% própria; **zero dep externa** (não
     use image::imageops::resize).

     Fórmula do filtro Mitchell-Netravali (forma 1D, t = |x_s_real -
     x_pixel_src|):
       w(t) = (1/6) * [
         (12 - 9B - 6C) * |t|^3 + (-18 + 12B + 6C) * |t|^2 + (6 - 2B)
         se |t| < 1,
         (-B - 6C) * |t|^3 + (6B + 30C) * |t|^2 + (-12B - 48C) * |t|
            + (8B + 24C)
         se 1 <= |t| < 2,
         0 caso contrário.
       ]
     Com B=C=1/3, o filtro é o "Mitchell" canonical (compromisso ótimo
     entre ringing e suavização per Mitchell 1988).
  4. Aplica rotation: rotacione o resampled buffer pela transform.rotation
     (com nearest-neighbor pra OK simple ou Mitchell de novo p/ alta
     qualidade — escolha Mitchell).
  5. Aplica flip: espelhe horizontal/vertical se sign(scale.x|y) < 0.
  6. Commit: texture_edit::commit_edited_texture com o novo RGBA8.
     Reseta sprite.Transform: scale = (1.0, 1.0), rotation = 0.

UX:
  - Pill no chrome (TopBar Image Tools, order 110).
  - Click: dispara OneShotImageOp; shell broadcast 1 por sprite em
    iter_selected; cada sprite recebe seu próprio bake.

GPU AVALIAÇÃO (induzido):
  - Mitchell-Netravali kernel: TRIVIALMENTE PARALELO por-pixel destino.
    Cada pixel destino lê 4×4 = 16 amostras source independentemente.
    **WGSL compute shader vale a pena** — 8-30× speedup em sprites
    grandes (>1024 px lado).
  - CPU path obrigatório (correto, determinístico, testes).
  - Parity test: ε=0.5/255 por canal.
  - Considere implementar GPU-first como caminho rápido se a tool é
    chamada com sprites grandes; CPU como fallback.

ARQUIVOS:
  src/lib.rs              — MANIFEST + pub fn register
  src/algorithm.rs        — mitchell_resample(src_rgba, src_dim, dst_dim) + rotate + flip
  src/shader.wgsl         — Mitchell resample compute (opcional)
  src/icon.rs             — BezPath de docs/design/icons/rasterize.svg

MANIFEST:
  id        = "rasterize"
  label_key = "tool.rasterize.label"
  cluster   = "image_tools"
  zone      = Zone::TopRight
  order     = 110       (Coord reservou)
  handler   = ToolHandler::OneShot { on_click }
                — segue o template trim_transparency/make_square exatamente
  memory_budget = MemoryBudget::new(0, 0, 0)
  touches_sim = false

MULTI-SPRITE: chrome broadcast já cobre. NADA a fazer no tool.

O QUE NÃO TOCAR: idem briefing 1.

WIRING / VALIDAÇÃO / NOMES / REPORT: idem briefing 1.
═══════════════════════════════════════════════════════════════════
```

---

## 4. BRIEFING — Upscale

```
═══════════════════════════════════════════════════════════════════
BRIEFING — tool-crate · slug: upscale  (sabor 3 stateful+panel)
═══════════════════════════════════════════════════════════════════

PASTAS EXCLUSIVAS:
  crates/ph2d-tool-upscale/                    (você cria)
  crates/ph2d-panel-upscale/                   (stub Coord; você preenche)

ANTES DE CODAR: idem briefings anteriores.

O QUE FAZER (algoritmo + UX):

ALGORITMO — 3 algoritmos cobrindo o design space (Enio decisão):

  (a) Lanczos3 (default) — Duchon 1979. Sinc-based, suporte 3 (kernel
      6×6 = 36 amostras por pixel destino). Estado-da-arte pra fotos e
      sprites com gradientes. Fórmula:
        L(t) = sinc(t) * sinc(t/3)   para |t| < 3, senão 0
        sinc(x) = sin(π·x) / (π·x)   (1.0 em x=0)
      Aplique normalização: divide o output pela soma dos pesos do
      kernel pra cada pixel (evita brilho variável quando o kernel
      sai da borda).
      Implementação 100% própria.

  (b) Nearest — replicação de pixel (factor inteiro: zero filtragem).
      Trivial. Preserva o pixel grid exato — único algoritmo correto
      pra pixel art quando o user quer NN puro.

  (c) xBR — Hyllian 2011 ("Pixel Filters and Image Resizing"). Edge-
      aware com corner-blending. Pixel art enhanced. Apenas suporta
      factor inteiro (2×, 3×, 4×). Operação por pixel:
        - Para cada pixel source, lê o neighborhood 5×5.
        - Detecta arestas via color-distance threshold (~100 em luma).
        - Em cada quadrante do output (factor=2 → 2×2 saída), decide
          se é "edge corner" (blend dos dois) ou "interior" (replica
          source).
      Tabela de regras canonical de Hyllian — implementação ~300-500
      LOC; siga referência open-source xBRZ se inspirar, mas escreva
      a partir do paper (zero dep).

UI (panel docado):
  - Dropdown algoritmo: { Lanczos3, Nearest, xBR }.
  - NumberInput scale_factor: 1.0–16.0 (default 2.0). Lanczos3 e
    Nearest aceitam não-inteiros; xBR clamp pra inteiro 2/3/4 (visual
    feedback no UI).
  - Toggle preview_live (default on).
  - Botão Apply.
  - Botão Cancel.
  - Preview: redesenha thumbnail RGBA do primary sprite a cada edit
    (cap 512×512 — padrão BgRemoval).

GPU AVALIAÇÃO (induzido):
  - Lanczos3: TRIVIALMENTE PARALELO. Kernel é constante (sinc
    pre-computado em uniform buffer). **WGSL vale ~15-40× speedup**.
    Considere primário, CPU fallback.
  - Nearest: TRIVIALMENTE PARALELO. WGSL trivial. CPU é tão rápido que
    GPU adiciona overhead de upload; CPU OK como default.
  - xBR: COMPLEXO (5×5 read + branches densas + lookup table). Viável
    em GPU mas alta complexidade WGSL. **CPU primário; GPU opcional
    de Implementador (justifique se for fazer)**. Documente decisão.
  - Parity test obrigatório para cada algoritmo que tiver path GPU.

ARQUIVOS DO TOOL CRATE:
  src/lib.rs              — MANIFEST + register + make
  src/tool.rs             — impl Tool + handle_panel_event
  src/algorithm.rs        — pub fn lanczos3 / pub fn nearest / pub fn xbr
                            (cada uma stand-alone; pub para outras tools
                             reusarem se quiserem — eq.: Equalize Sizes
                             pode chamar lanczos3 diretamente).
  src/shader.wgsl         — Lanczos3 compute (opcional)
  src/icon.rs             — BezPath de docs/design/icons/upscale.svg
  src/params.rs           — UpscaleUiEdit / UpscaleUiSnapshot / UpscaleParams
                            + apply_ui_edit

ARQUIVOS DO PANEL CRATE: idem stub.

MANIFEST:
  id        = "upscale"
  label_key = "tool.upscale.label"
  cluster   = "image_tools"
  zone      = Zone::TopRight
  order     = 120       (Coord reservou — rightmost)
  handler   = ToolHandler::Stateful { ... }
  memory_budget = MemoryBudget::new(0, 0, 0)
  touches_sim = false

MULTI-SPRITE: chrome broadcast já cobre. Cada sprite passa pelo Apply
do panel; o método público da concrete type do tool roda 1× por sprite.

O QUE NÃO TOCAR: idem briefing 1.

WIRING / VALIDAÇÃO / NOMES / REPORT: idem briefing 1.
═══════════════════════════════════════════════════════════════════
```

---

## Smoke checklist (após os 4 fecharem)

Coord testa com Enio:

1. **Color Equalization** — seleciona 2 sprites diferentes; click pill;
   panel abre; ajusta brightness + saturation; Apply → ambos sprites
   recebem ajuste. Cancel sobre outro par → não aplica.
2. **Equalize Sizes** — seleciona 3 sprites de tamanhos distintos; click
   pill; panel abre; deixa target_mode=MaxOfSelection; Apply → todos
   ficam no tamanho do maior. Variante Fixed W×H → todos no tamanho
   fixado. upscale_if_smaller on → menores escalam ao destino.
3. **Rasterize** — seleciona 2 sprites com scale ≠ 1.0; click pill →
   ambos têm scale resetado a 1.0 e o pixel buffer reflete o tamanho
   visual anterior. Sprite com rotation → pixel buffer rotacionado +
   rotation = 0.
4. **Upscale** — seleciona 2 sprites; click pill; panel abre; dropdown
   Lanczos3; scale=2.0; Apply → ambos sprites dobrados. Dropdown
   xBR + scale=2.0 sobre pixel art → arestas suavizadas com corner
   blending visível. Nearest + scale=2.0 → replicação de pixel exata.

CI verde, push, babysit. Reporta link da run.
