# 06 — Plano de implementação: dois slots de textura (Shape + Grain)

> **Fase 3 do** [`HANDOFF_shape_grain_dual_texture.md`](HANDOFF_shape_grain_dual_texture.md).
> Pré-requisitos: [`04_pesquisa`](04_pesquisa_shape_grain_procreate.md) + [`05_design`](05_design_dois_slots_textura.md).
> **Waves pequenas, cada uma compilável + testável isolada**, ordenadas para **minimizar risco de
> regressão** (back-compat byte-idêntico primeiro). Cada wave tem **teste e2e que prova no produto** —
> não só unit verde ([as 4 causas](../../docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)).
> **A implementação é uma rodada subsequente, com aval do Enio.** Este doc é o roteiro.

---

## §0 — Regras válidas para TODAS as waves

- **Inner loop:** `CARGO_TARGET_DIR=<slot> cargo check -p <crate>` (slot CoW). Teste/clippy/LOC **1× no
  fim da wave**, não por task ([CLAUDE.md §2](../../CLAUDE.md)).
- **Isolamento:** só as crates do painter (`ph2d-painter-brush`, `ph2d-tool-painter`,
  `ph2d-panel-painter-layers`, + ids em `ph2d-editor-core/src/ids/chrome/painter.rs`). Precisou de algo
  fora ⇒ **PARE e reporte**.
- **Commits locais** `--no-verify`, stage só os meus paths (`git add -- <paths>`). **Sem push/CI** (Coord).
- **HR-5** em todo sorteio novo: gated + ordem fixa + all-off byte-idêntico.
- **DoD por wave:** o **teste e2e da wave verde** + LOC/clippy do diff. Compile-verde **não** é pronto.
- **Ponto de retorno:** tag `painter-pre-shape-grain-2026-06-24` @ `928bd303` + `backups/painter_2026-06-24/`.

---

## §1 — W0: Fundação no engine (silhueta × grain, default byte-idêntico) — **a wave crítica**

**Objetivo:** introduzir o slot `shape` + `grain_depth` e a composição `silhueta × grain`, **provando
back-compat byte-a-byte**. Nada de Shape-de-imagem útil ainda — só a plumbing + o default neutro.

**Arquivos / símbolos:**
- `ph2d-painter-brush/src/spec.rs` — renomear `texture`→`grain`; add `shape: TextureSettings`,
  `grain_depth: f32` (default 1.0); helper `TextureSettings::depth()` se Depth morar lá, ou campo no
  spec (design §2b: **campo no spec**). Atualizar `Default`, `has_per_dab_rotation` (passa a olhar os 2
  slots), e os helpers que liam `self.texture`.
- `ph2d-painter-brush/src/dab.rs` — `stamp_band`: silhueta = shape **ou** falloff; grain com Depth.
  `DabCtx` ganha `shape`/`shape_image`. `stamp_dab_inner` resolve/recebe o `shape_basis`.
- `ph2d-painter-brush/src/stamp.rs` — `render_stamp_mask` assa `silhueta × grain` (shape no unit-coord).
- `ph2d-painter-brush/src/texture.rs` — `is_cacheable`/`is_canvas_cacheable` viram métodos que o
  **caller combina** (ou um helper `BrushSpec::mask_cacheable()` que faz o produto dos 2 slots, design §3a).
- `ph2d-tool-painter/src/tool/paint.rs` — `PaintState`: renomear `texture_image*`→`grain_image*`; add
  `shape_image`/`_pending`/`_version` + `shape_rake_dir`/`shape_rake_accum`. `stamp_dabs_inner`: condição
  das rotas vira produto de 2 slots (design §3a). As 4 fns de stamp resolvem o `shape_basis`.

**Riscos:** é o hot-path e o caminho de cache — **alto risco de regressão**. Mitigação: o default neutro
é byte-idêntico por construção (design §6); o teste byte-a-byte é o gate.

**Teste e2e da wave (o entregável que prova):**
- `shape_none_depth_one_is_byte_identical`: pinta o MESMO dab (vários brushes: liso, texturizado
  ViewPlane, Tiled, com Rake, com Random) **com o engine novo** e compara byte-a-byte contra um buffer
  de referência salvo do HEAD (ou contra `falloff×grain` recomputado pela fórmula antiga inline no teste).
- Re-rodar `cargo test -p ph2d-painter-brush --lib` e exigir **132 intactos** (nenhum muda de resultado)
  + `-p ph2d-tool-painter` **127** + `-p ph2d-panel-painter-layers` **21**.
- **Headless GPU** não se aplica (CPU path); mas rodar os testes de paridade cached↔per-pixel já
  existentes e exigir verde.

**DoD:** byte-idêntico provado + contagens intactas. **Sem isso, W1+ não começam.**

---

## §2 — W1: Caches do produto de 2 slots + Shape-de-imagem amostrável

**Objetivo:** fazer o Shape de **imagem** efetivamente desenhar a silhueta (substituindo o falloff) e
ajustar as 4 rotas para o produto de 2 slots, com paridade cached↔per-pixel.

**Arquivos / símbolos:**
- `ph2d-painter-brush/src/texture.rs` / `stamp.rs` — `render_stamp_mask` amostra a `ImageMask` do shape
  no unit-coord; **clamp-para-0 fora do footprint** (silhueta não tilea — design §2a/§2c). `sample_shape`
  (ou `sample` reusado com a convenção de silhueta).
- `ph2d-tool-painter/src/tool/paint/stamp_cache.rs` — `StampKey`/`CanvasKey` ganham `shape` +
  `shape_image_version`; `ensure_stamp_cache`/`ensure_canvas_cache` invalidam no shape; as 4 fns
  (`stamp_dabs_cached`/`_canvas_cached`/`_ramped`/`_per_pixel`) resolvem `shape_basis` (com
  `advance_rake` próprio do shape) e passam a `ImageMask` do shape.
- `ph2d-tool-painter/src/tool/paint.rs` — a condição de `stamp_dabs_cached` passa a aceitar
  `shape.is_active() ∨ grain.is_active()` (design §3a — Shape-only cacheia).

**Teste e2e:**
- `shape_image_paints_silhouette_not_disc`: brush com Shape = quadrado (ImageMask sintética 8×8 cheia),
  grain None, falloff Smooth; pinta um dab e afirma que os 4 cantos do quadrado têm cobertura > 0
  (o disco do falloff os zeraria) e que fora do quadrado é 0.
- `cached_matches_per_pixel_with_shape`: o MESMO brush shape+grain pela rota cached e pela per-pixel
  (forçando cada uma) produz buffers iguais (tolerância de 1 LSB do bilinear, como os parity tests atuais).

**DoD:** a ponta de imagem aparece no buffer **e** as rotas concordam.

---

## §3 — W2: Comportamentos do Shape (gated, HR-5)

**Objetivo:** os comportamentos que dão vida ao Shape, cada um *gated* e determinístico. **Ordem por
valor/risco** — fazer só os baratos-e-úteis primeiro; Count/Roundness são opcionais.

**Sub-tasks (cada uma é um commit isolado, byte-idêntico quando off):**
- **W2.1 — Shape Rotation (Follow-Stroke / Angle / Random):** reusa `advance_rake` (estado próprio do
  shape) + `angle_deg` + `random_angle` do slot shape. *e2e:* shape assimétrico (seta) segue uma curva
  (heading do dab) vs. fica fixo com Rake off.
- **W2.2 — Shape Scatter (posicional):** reusa/estende o `jitter` de posição aplicado **quando o shape
  está ativo** (ou um `shape_scatter` próprio se a semântica diferir). *e2e:* N dabs no mesmo ponto com
  scatter > 0 espalham; com 0 empilham.
- **W2.3 (opcional) — Flip X/Y:** nega `u`/`v` do `shape_basis`. *e2e:* shape espelhado.
- **W2.4 (opcional) — Count / Count-Jitter:** loop de N carimbos por passo, offset/rot de cada um do
  rng em ordem fixa. *e2e:* Count=4 + scatter ⇒ 4 carimbos/ponto; Count=1 ⇒ baseline.
- **W2.5 (opcional) — Roundness:** footprint elíptico (Size X/Y do dab, não só da textura). *e2e:*
  squash visível; 1.0 ⇒ redondo.

**HR-5:** cada sub-task tem o teste "off ⇒ RNG não avança ⇒ byte-idêntico" (padrão
[`jitter.rs` `all_off_draws_no_randomness`](../../crates/ph2d-painter-brush/src/jitter.rs)).

**DoD:** cada comportamento ativo tem e2e que o prova **e** o off-test byte-idêntico.

---

## §4 — W3: Painel — seções Shape + Grain (preservar Texture-LAYER)

**Objetivo:** UI das duas seções, costura ponta-a-ponta, **sem quebrar** o editor de Texture-LAYER.

**Arquivos / símbolos:**
- `ph2d-panel-painter-layers/src/paint_texture.rs` → **renomear para `paint_grain.rs`** (seção Grain =
  a Texture de hoje + **slider Depth**). Manter a assinatura `compact` e os **2 call-sites** (brush +
  Texture-LAYER) idênticos.
- `ph2d-panel-painter-layers/src/paint_shape.rs` — **novo**: seção Shape colapsável (source/Import,
  preview, Rake, Random, Angle, Offset, Size, [W2: Flip/Count/Roundness/Scatter]). **Sem** Mapping
  (ViewPlane implícito), **sem** Color Ramp.
- `ph2d-editor-core/src/ids/chrome/painter.rs` — `PAINTER_SHAPE_*` (um por controle) +
  `PAINTER_BRUSH_GRAIN_DEPTH`; adicionar às listas que o guard "dead-control" cobre (espelho de
  `PAINTER_BRUSH_RANDOMIZE_SLIDERS`).
- `populate.rs` / `event.rs` — register + dispatch de cada id novo (**os 7 sites** da
  [DIRETIVA §2](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md); template = Jitter Spacing).
- `ph2d-tool-painter/src/tool/paint/brush_settings.rs` (+ `shape_settings.rs` novo se LOC exigir) —
  `BrushSettings` snapshot ganha os campos do shape; setters `set_shape_*` + reset por seção;
  `handle_panel_event` roteia os ids do Shape.

**Riscos:** LOC (painel 600/arquivo, 200/função) + o reuso da Texture-LAYER. Mitigação: extrair
`paint_shape.rs` separado; rodar `architecture_panel_loc_cap` na wave; **não usar apóstrofo em
comentário** (quebra o parser — [HANDOFF §2](HANDOFF_shape_grain_dual_texture.md)).

**Teste e2e (seam, `ph2d-ui-testkit`):**
- Dirige um id do Shape (ex.: `PAINTER_SHAPE_ANGLE`, `PAINTER_SHAPE_RAKE`) via evento real e afirma o
  efeito em `BrushSpec.shape` (rotação/rake setados) — o padrão "evento real → efeito observável".
- Dirige `PAINTER_BRUSH_GRAIN_DEPTH` e afirma `grain_depth` no spec + o stroke resultante difere.
- **Regressão Texture-LAYER:** os 21 testes do painel verdes + um teste que edita uma Texture-LAYER e
  afirma que o roteamento ainda cai na layer (não no brush).

**DoD:** seam test verde para ≥1 id de cada seção + Texture-LAYER intacta + **smoke do Enio** (layout).

---

## §5 — W4: Carga de imagem nos 2 slots

**Objetivo:** o usuário atribui imagem ao Shape e ao Grain.

**Arquivos / símbolos:**
- `ph2d-panel-painter-layers/src/paint_shape.rs` / `paint_grain.rs` — botão **Import** por seção
  (design §5d, opção B); o shell abre o file-picker via o flag `*_image_pending` (espelha
  `texture_image_pending`).
- `ph2d-tool-painter` — handler que recebe os pixels e seta `shape_image`/`grain_image` + bump de
  version. Manter "Use as Brush Texture" (Hierarquia) → **Grain** (back-compat).
- Bridge/shell: publicar a luminância do shape para o **preview** (como já faz para o grain —
  `current_brush_texture_image`; add `current_brush_shape_image`).

**Teste e2e:** seam/integration: setar um `ImageMask` no slot Shape via o caminho do tool e pintar ⇒ a
silhueta importada aparece (reusa o e2e de W1 mas pelo caminho de carga real, não ImageMask sintética
injetada). Preview do Shape renderiza a imagem.

**DoD:** importar uma imagem como Shape e pintar a ponta dela, fim-a-fim, **smoke do Enio**.

---

## §6 — W5: Audit e2e + fechamento

**Objetivo:** provar que funciona no produto e fechar com gates batched.

- **Audit por lente** (template [DIRETIVA §3](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md),
  preenchido por claim): correção (silhueta×grain), wiring (os 7 sites de cada id), perf (FPS do brush
  texturizado **não** regride > 10% — medir em `--release` contra a tag), determinismo (replay).
- **Conjunto de aceitação congelado** ([05 §9](05_design_dois_slots_textura.md)) — todos os 8 verdes.
- **Gates batched 1×:** `scripts/nextest-impacted.sh` + clippy `--all-targets` + `architecture_*_loc_cap`
  + os arch-gates do painter, sobre o diff acumulado.
- **Smoke real do Enio:** pintar com Shape=imagem + Grain=Noise (Moving e Texturized), variar Depth,
  Rake, e **ver** a ponta + grão. SÓ ENTÃO reportar fechamento.

**DoD:** conjunto de aceitação verde + audit por-lente com asserção-vermelha por claim + smoke do Enio.
**Veredito condicional** ("APPROVE pending smoke") até o smoke voltar.

---

## §7 — Ordem, paralelismo e custo

| Wave | Depende de | Pode paralelizar? | Risco | Esforço relativo |
|---|---|---|---|---|
| **W0** fundação byte-idêntica | — | não (base de tudo) | **alto** (hot-path/cache) | médio |
| **W1** caches + shape amostrável | W0 | não | alto (cache parity) | médio |
| **W2** comportamentos shape | W1 | sub-tasks em paralelo | médio (HR-5) | médio (escalável) |
| **W3** painel | W0 (campos) | parcial c/ W2 | médio (LOC, Texture-LAYER) | médio |
| **W4** carga de imagem | W3 | não | baixo | baixo |
| **W5** audit + fecho | W0-W4 | — | — | baixo |

- **Caminho crítico:** W0 → W1 → (W2 ∥ W3) → W4 → W5. W2 e W3 podem rodar em paralelo após W1 (W3 só
  precisa dos **campos** do spec de W0, não dos comportamentos de W2) — mas **≤3 cargos simultâneos**
  (RAM 8 GiB).
- **MVP mínimo viável** (se o Enio quiser cortar): W0 + W1 + W3(sem W2 opcionais) + W4 já entrega
  "ponta de imagem + grain ortogonal + Depth + painel + import". W2.3-W2.5 (Flip/Count/Roundness) são
  incrementos puros depois.

---

## §8 — O que NÃO fazer (anti-goals desta linha)

- **Não** implementar Azimuth (Shape/Grain) — sem pipeline de tilt/azimuth ([04 §6](04_pesquisa_shape_grain_procreate.md)).
- **Não** adicionar Grain Blend Mode separado (grão↔cor) — o multiply default cobre; defer.
- **Não** versionar save — brush não é serializado ([05 §7](05_design_dois_slots_textura.md)).
- **Não** dar seção Shape à Texture-LAYER (uma layer não tem ponta).
- **Não** quebrar o reuso de `paint_grain_section` (ex-`paint_texture_section`) pela Texture-LAYER.
- **Não** copiar código do Procreate/Blender (GPL) — clean-room, só comportamento.

---

## §9 — Resumo de uma tela

```
HOJE:   cobertura = falloff(t) × texture(coord) × dyn        (1 slot = grain; silhueta = falloff)
ALVO:   cobertura = SILHUETA  × GRAIN(depth)    × dyn        (2 slots ortogonais)
            SILHUETA = shape_alpha  se shape atribuído  SENÃO  falloff(t)   ← Shape SUBSTITUI o falloff
            GRAIN    = grain(coord) com Depth  se atribuído  SENÃO  1.0     ← o slot de hoje, re-rotulado

slot novo: shape: TextureSettings (Copy) + shape_image em PaintState   |  grain_depth: f32
default neutro (shape=None, depth=1.0) ⇒ BYTE-IDÊNTICO a hoje (gate W0)
caches: 4 rotas viram produto lógico de 2 slots; topologia inalterada
painel: seção Shape (brush-only) + seção Grain (= Texture de hoje + Depth); Texture-LAYER = Grain intacta
save: brush não serializado ⇒ zero migração
```

→ **PARAR aqui. Reportar ao Enio. Implementação = rodada seguinte, com aprovação.**

---

## §10 — STATUS: IMPLEMENTADO (Enio aprovou full W0–W5, 2026-06-25)

Todas as waves landaram em loop implementação→auditoria. ADR formal:
[`0100-dual-texture-slots-shape-grain.md`](../architecture/decisions/0100-dual-texture-slots-shape-grain.md).

| Wave | Status | Evidência (testes) |
|---|---|---|
| **W0** fundação byte-idêntica | ✅ | `grain_depth_one_is_default_and_zero_disables` + suítes 137/129/21 intactas |
| **W1** caches + shape amostrável | ✅ | `shape_image_cached_mask_matches_per_pixel_silhouette` · `shape_with_tiled_grain_canvas_cached_matches_per_pixel` |
| **W2** Shape rotation (Angle/Rake/Random, HR-5) | ✅ | `shape_angle_rotates_the_silhouette` + `advance_rake` |
| **W3** painel (seção Shape c/ Falloff migrado + preview + Grain rename + Depth) | ✅ | `panel_events_drive_shape_and_grain_depth` (seam) + Texture-LAYER 21 intacta |
| **W4** import (Hierarquia 2 opções: Shape/Grain) | ✅ | `shape_image_paints_the_silhouette_end_to_end` (e2e via o mesmo setter do shell) |
| **W5** gates batched | ✅ | clippy limpo · LOC caps verdes (splits por responsabilidade, sem allowlist) · wiring/behavioral/contract gates verdes |

**Desvios do design (documentados):**
- O campo interno do Grain ficou `texture` (não renomeado p/ `grain`) — só o **label** virou "Grain"
  (menor churn/risco; ADR-0100 alt. rejeitada).
- "Clearly inactive" do Falloff = a seção Shape **substitui** o Falloff dropdown+curva pela imagem +
  preview + caption "Falloff inactive" quando há imagem (em vez de greying com scrim — sem primitivo de
  draw desabilitado; mais claro e sem dead-control).
- Import do Shape é via a **Hierarquia** (menu); não há botão "Import" in-panel (API pronta se quiser).
- Splits de LOC: `texture/shape.rs`, `paint/shape_settings.rs`, `paint/stamp_route.rs`, `paint_shape.rs`.

**Pendente:** smoke manual do Enio (pintar com Shape de imagem + Grain + Depth; ver preview, menu,
falloff inativo). Veredito condicional **APPROVE pending smoke** até lá.
