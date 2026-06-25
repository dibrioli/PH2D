# ADR-0100 — Dois slots de textura no brush: Shape (silhueta) + Grain (textura)

**Status:** Accepted (implementado 2026-06-25; aguarda smoke manual do Enio para fechar como Done).
**Contexto/decisor:** Enio, 2026-06-24 (pesquisa+plano) → 2026-06-25 (aprovação + implementação W0–W5).
**Substitui/relaciona:** estende o brush clean-room ([nota CLAUDE.md §5 "a PINTURA voltou"]); o brush **não** é contract-gateado ([project-painter-brush-came-back-cleanroom]), então esta ADR registra a **decisão arquitetural**, não um contrato congelado.
**Docs de detalhe:** [`docs/Painter/04_pesquisa_shape_grain_procreate.md`](../../Painter/04_pesquisa_shape_grain_procreate.md) · [`05_design`](../../Painter/05_design_dois_slots_textura.md) · [`06_plano`](../../Painter/06_plano_dois_slots_textura.md).

## Contexto

O brush tinha **um** slot de textura — que, na prática, é o **Grain** do Procreate (a textura *dentro*
do dab) — e a silhueta do dab era sempre o `falloff` redondo procedural. O Procreate separa **Shape**
(a ponta/alpha do carimbo) de **Grain** (a textura dentro da silhueta), ortogonais. Faltava-nos a
"ponta de imagem": carimbar uma silhueta importada (estrela, folha, textura de pincel) era impossível.

## Decisão

1. **Generalizar a cobertura do dab** de `falloff × grain` para **`silhueta × grain × dinâmica`**, com
   **default neutro** em cada slot (silhueta→falloff, grain→`g_eff=g` com Depth=1) ⇒ baseline
   **byte-idêntico** ao engine pré-Shape.
2. **Shape SUBSTITUI o falloff** quando uma imagem é atribuída (fidelidade: formas duras crocantes até
   a borda do footprint); **sem imagem, a silhueta é o falloff** (back-compat). O falloff migrou para
   dentro da seção **Shape** do painel e fica **inativo** quando há imagem.
3. **Reusar `TextureSettings`** para os dois slots (um dono só): campo `texture` (= Grain, nome interno
   mantido por baixo-custo de churn; **label visível "Grain"**) + novo campo `shape` + `grain_depth`.
   Pixels do Shape vivem em `PaintState.shape_image` (espelha `texture_image`, fora do `Copy BrushSpec`).
4. **Caches:** as 4 rotas (`stamp_dabs_cached`/`_canvas_cached`/`_ramped`/`_per_pixel`) viram **produto
   lógico de 2 slots** (`BrushSpec::dab_mask_cacheable`/`shape_silhouette_active`/`shape_has_per_dab_rotation`);
   `render_stamp_mask` assa `silhueta × grain`. Topologia inalterada.
5. **Comportamentos do Shape:** Rotation **Angle / Rake (segue o traço, reusa `advance_rake`) / Random**
   — gated + ordem de draw fixa shape→grain (HR-5; off ⇒ não puxa RNG ⇒ byte-idêntico).
6. **Import:** a opção única da Hierarquia "Use as Brush Texture" virou **duas** — "Use as Brush Shape"
   + "Use as Brush Grain".
7. **Brush não é serializado** ⇒ **zero impacto de SCHEMA_VERSION/save**. Texture-LAYER = Grain (não
   ganha Shape; o reuso de `paint_texture_section`/`paint_grain` é preservado).

## Alternativas rejeitadas

- **Estender o slot único** (um modo "shape" no mesmo slot): não dá ortogonalidade (shape+grain
  simultâneos) — o objetivo inteiro.
- **Shape MULTIPLICA o falloff:** erode formas duras na borda do footprint; infiel ([04 §4](../../Painter/04_pesquisa_shape_grain_procreate.md)).
- **`ShapeSettings` separado:** duplicaria frame/helpers; rejeitado por "um dono só".
- **Renomear `texture`→`grain` no código:** ~30+ sites de churn (tool+panel) para nome interno; rejeitado
  em favor de manter `texture` interno + só o label "Grain" (menor risco de regressão).

## Consequências

- +1 slot `Copy` (`shape`) + `grain_depth` em `BrushSpec`; +1 buffer em `PaintState`; +1 seção de painel
  (brush-only) + 1 opção de menu. Back-compat provada por teste byte-a-byte (`grain_depth_one_is_default_…`)
  + as suítes 137/129/21 intactas. Parity cached↔per-pixel com Shape provada (`shape_image_cached_mask_…`,
  `shape_with_tiled_grain_canvas_cached_…`). e2e do produto: `shape_image_paints_the_silhouette_end_to_end`.
- Split de LOC por responsabilidade: `texture/shape.rs`, `paint/shape_settings.rs`, `paint/stamp_route.rs`,
  `paint_shape.rs` (painel) — sem novas entradas em `FILE_OVERAGE_OK`.

## Fora de escopo (follow-ups)

- **Azimuth** (Shape/Grain) — sem pipeline de tilt/azimuth de caneta.
- **Grain Blend Mode separado** (grão↔cor base) — o multiply default cobre; defer.
- Shape **Count / Roundness / Flip / Scatter dedicado** — W2 opcionais marcados; Scatter posicional já
  disponível via o `jitter` de posição existente.
- File-picker de imagem direto para o Shape (hoje o import é via Hierarquia) — API pronta se um botão
  "Import" for adicionado à seção Shape.

## Kill-criterion (era pré-build; resultado)

"Se após W0 o stamp não for byte-idêntico com shape=None+depth=1, ou o FPS de um brush texturizado
regredir >10%, a feature não existe nesta forma." → **Byte-identidade confirmada** (default neutro por
construção + suítes intactas); a composição é o mesmo hot-path + 1 branch de silhueta + 1 lerp de Depth
short-circuitado em `depth>=1` (sem custo no caminho default). Sem regressão estrutural.
