# 05 — Design: dois slots de textura (Shape + Grain), paridade Procreate

> **Fase 2 do** [`HANDOFF_shape_grain_dual_texture.md`](HANDOFF_shape_grain_dual_texture.md).
> Pré-requisito: [`04_pesquisa_shape_grain_procreate.md`](04_pesquisa_shape_grain_procreate.md).
> **Entregável:** arquitetura dos 2 slots respeitando as 8 restrições reais do código + back-compat
> **byte-idêntico** + o **ADR-0100** (decisão). **Não implementa** — o código é a rodada seguinte.

---

## §0 — Princípio organizador

Generalizar a cobertura de um dab de **`falloff × grain`** (hoje) para **`silhueta × grain`** (alvo),
onde **cada slot tem um default neutro** que reproduz o comportamento atual:

```
cobertura(pixel) =  SILHUETA(pixel)  ×  GRAIN(pixel)  ×  dinâmica/flow

SILHUETA(pixel) =  falloff_weight(t)             se shape.kind == None   (DEFAULT)
                =  shape_alpha(footprint_coord)   se shape atribuído

GRAIN(pixel)    =  1.0                            se grain.kind == None   (DEFAULT)
                =  grain_value(coord) com Depth    se grain atribuído
```

- **Shape None + Grain None ⇒ exatamente `dab.rs` de hoje** (falloff × 1.0). Baseline byte-idêntico.
- O **slot atual `texture` vira o slot `grain`** (mesma struct, conceito idêntico — sem reescrever).
- O **slot novo `shape`** é o que destrava a "ponta de imagem".

Esta é a decisão central, justificada em [04 §4](04_pesquisa_shape_grain_procreate.md): **Shape
SUBSTITUI** o falloff quando atribuído; **sem Shape, a silhueta é o falloff** (back-compat).

---

## §1 — Modelo de dados (restrição #1: `BrushSpec` é `Copy`, sem pixels)

### 1a. Reuso do tipo: `TextureSettings` serve aos DOIS slots
`TextureSettings` ([`texture.rs:280`](../../crates/ph2d-painter-brush/src/texture.rs)) já é `Copy`, sem
pixels, e tem tudo que um slot precisa: `kind`, `mapping`, `angle_deg`, `rake`, `random_angle`,
`offset`, `size`, `params[6]`. **Não criar um tipo paralelo** ([feedback: um dono só]). O slot de Shape
reusa `TextureSettings` com **semântica de silhueta** (o sampler interpreta o valor como alpha do
footprint, não como modulador). Diferença de comportamento, não de tipo.

> **Por que não um `ShapeSettings` separado?** Um segundo struct duplicaria angle/rake/random/offset/
> size/params e os helpers (`dab_basis`, `is_cacheable`). O que muda entre Shape e Grain é **como o
> valor amostrado entra na composição** (silhueta vs modulador) e **quais kinds fazem sentido**, não os
> parâmetros de frame. Reusar `TextureSettings` mantém um dono só e zero divergência de helpers.

### 1b. `BrushSpec`: renomear `texture`→`grain` + adicionar `shape`
```rust
pub struct BrushSpec {
    // ...
    pub grain: TextureSettings,   // era `texture` — o slot atual, semântica inalterada (Grain)
    pub shape: TextureSettings,   // NOVO — a silhueta de imagem; kind==None ⇒ usa o falloff
    // ...
}
```
- `shape` default = `TextureSettings::default()` (kind `None`) ⇒ silhueta = falloff (back-compat).
- **Rename `texture`→`grain`:** mecânico, ~grep-e-troca; `BrushSpec` **não é serializado** (restrição
  #6 confirmada), então o rename **não tem custo de wire/save**. Manter um alias `pub fn texture()` só
  se algum consumidor externo o exigir (provavelmente o painel/tool, que editamos juntos — sem alias).
- **`is_active()`/discriminantes wire-st_able de `TextureKind`/`TextureMapping`** ficam idênticos — o
  Shape e o Grain compartilham a mesma tabela de kinds. *Restrição de produto:* o Shape **só faz
  sentido como `Image`** (uma ponta importada) — mas tecnicamente um kind procedural também produz uma
  silhueta; o painel **filtra** os kinds oferecidos por slot (ver §5), o engine aceita ambos.

### 1c. Buffer de imagem do Shape em `PaintState` (restrição #2)
Espelhar exatamente o par que já existe para o grain
([`paint.rs:142-146`](../../crates/ph2d-tool-painter/src/tool/paint.rs)):
```rust
// PaintState — ao lado de texture_image / texture_image_version (renomeados grain_image/_version):
shape_image:         Option<BrushTextureImage>,   // pixels do Shape (luminância → alpha)
shape_image_pending: bool,                         // shell abre file-picker
shape_image_version: u64,                          // invalida o cache do mask quando muda
```
- `BrushTextureImage::as_mask()` → `ImageMask` (já existe) serve aos dois slots sem mudança.
- **Rename `texture_image`→`grain_image`** acompanha o rename do campo no spec (consistência).

---

## §2 — Composição no dab (restrição #3: hot-path cirúrgico)

### 2a. A mudança em `stamp_band` ([`dab.rs:386-486`](../../crates/ph2d-painter-brush/src/dab.rs))
Hoje (linha ~396-429), simplificado:
```rust
let mut w = ctx.spec.falloff_weight(t);          // silhueta = falloff
if w <= 0.0 { continue; }
if let Some(b) = ctx.tex {                        // grain
    let s = sample(&ctx.spec.texture, b, px, py, center, radius, image);
    w *= s;                                        // grain multiplica
}
```
Alvo:
```rust
// 1) SILHUETA: shape de imagem substitui o falloff; senão, o falloff (default).
let mut w = if ctx.shape.is_some() {
    let sa = sample_shape(&ctx.spec.shape, ctx.shape_basis, px, py, center, radius, ctx.shape_image);
    // O falloff vira (opcional) feather do shape: por default NÃO multiplica (estrela crocante).
    // Modo "Shape softness" (futuro) faria `sa * falloff_weight(t)`.
    sa
} else {
    ctx.spec.falloff_weight(t)
};
if w <= 0.0 { continue; }
// 2) GRAIN: idêntico a hoje, com Depth aplicado (lerp 1↔grão).
if let Some(b) = ctx.grain {
    let g = sample(&ctx.spec.grain, b, px, py, center, radius, ctx.grain_image);
    let g_eff = 1.0 + (g - 1.0) * ctx.spec.grain.depth();   // Depth=1 ⇒ g_eff = g (hoje)
    w *= g_eff;
}
```
- **`sample_shape`** é `sample()` com a convenção de que o valor **é** a cobertura da silhueta (mesma
  função; o "shape" não tem Depth nem ramp). Para `kind==Image`, é a luminância da `ImageMask`
  (já bilinear+tiling). Fora do tile do shape o valor cai a 0 (a ponta tem extensão finita) — ⚠️
  **decidir o wrap do Shape:** silhueta **não tilea** (clamp para 0 fora de `[-1,1]` do footprint),
  diferente do grain que tilea. Ver §2c.
- **Depth default = 1.0** ⇒ `g_eff = g` ⇒ **byte-idêntico** ao `w *= s` de hoje. Depth é um `params`
  slot ou um campo novo em `TextureSettings` (ver §2b).
- O `t = dist/radius` continua computado (barato; precisa para o falloff e para o early-out do bbox).
  Para shape-de-imagem, `t` deixa de ser a silhueta mas o **early-out `w<=0`** ainda vale.

### 2b. Onde mora o **Depth** do grain
Duas opções:
- **(A)** Campo novo `grain_depth: f32` em `BrushSpec` (claro, fora do `params`). **Recomendado** —
  Depth é universal (todo grain), não um shape-knob per-kind.
- **(B)** Reusar um `params` slot. Rejeitado: `params[0]/[1]` são Contrast/Brightness universais e
  `params[2..]` são per-kind; Depth não cabe limpo.
> **Recomendação: (A)** `grain_depth: f32` default `1.0`. Back-compat: `1.0` ⇒ comportamento atual.

### 2c. Frames separados por slot
Cada slot resolve seu **próprio** `TexDabBasis` via `dab_basis` (já existe,
[`texture.rs:402`](../../crates/ph2d-painter-brush/src/texture.rs)):
- **`shape_basis`** = `dab_basis(&spec.shape, shape_dir, &mut rng, canvas, shape_extra_rot)`.
- **`grain_basis`** = `dab_basis(&spec.grain, grain_dir, &mut rng, canvas, grain_extra_rot)` (hoje).
- ⚠️ **Ordem de draw do RNG (HR-5):** se ambos sortearem (Random rotation / Random offset), a ordem
  **shape→grain** é fixa e gated por-feature (cada `dab_basis` só puxa do rng quando o slot é Random).
  Documentar no módulo, espelhar [`jitter.rs`](../../crates/ph2d-painter-brush/src/jitter.rs).
- **Rake do shape:** reusa `advance_rake` ([`stamp_cache.rs:396`](../../crates/ph2d-tool-painter/src/tool/paint/stamp_cache.rs)),
  com **estado de rake próprio** (`shape_rake_dir`/`shape_rake_accum` no `PaintState`, ao lado dos do
  grain) — os dois headings são independentes.

### 2d. O `DabCtx` ganha os campos do shape
`DabCtx` ([`dab.rs:362-381`](../../crates/ph2d-painter-brush/src/dab.rs)) passa a carregar
`shape: Option<&TexDabBasis>`, `shape_image: Option<ImageMask>` ao lado de `tex`(→`grain`)/`image`. O
ramped-path (Color Ramp) **só se aplica ao grain** (a silhueta não tem ramp) — sem mudança no ramp.

---

## §3 — Caches (restrição #3: manter as 4 rotas corretas)

As regras de elegibilidade viram um **produto lógico dos 2 slots**. Definições atuais em
[`texture.rs:333-352`](../../crates/ph2d-painter-brush/src/texture.rs).

### 3a. StampMask (dab-relativo, scale-invariante) — rota `stamp_dabs_cached`
Hoje assa `falloff × View-texture` ([`stamp.rs:38-60`](../../crates/ph2d-painter-brush/src/stamp.rs)).
**Generaliza para `silhueta × grain`** assando os DOIS slots no mesmo `u8` mask, desde que **ambos
sejam dab-relativos-constantes**:
```
mask_cacheable  ⟺  shape_is_view_static  ∧  grain_is_view_static
   shape_is_view_static = (shape.kind==None)                         // falloff puro (hoje)
                        ∨ (shape ViewPlane ∧ !rake ∧ !random ∧ !per-dab-rot ∧ !scatter ∧ !count)
   grain_is_view_static = (grain.kind==None)                         // sem grain
                        ∨ (grain ViewPlane ∧ !rake ∧ !random ∧ !per-dab-rot)
```
- `render_stamp_mask` passa a amostrar **shape** (substituindo o `falloff_weight` quando shape
  atribuído) **e** grain, no unit-coord `[-1,1]` (já é o que faz para o grain via `sample_unit`).
- **Novidade importante:** hoje a rota `cached` exige `texture.is_active()`
  ([`paint.rs:405`](../../crates/ph2d-tool-painter/src/tool/paint.rs)). Com Shape, um brush **Shape-only
  sem grain** TAMBÉM deve cachear (a silhueta de imagem é constante). Logo a condição da rota vira
  `(shape.is_active() ∨ grain.is_active()) ∧ mask_cacheable ∧ …`. Um brush redondo liso (nada ativo)
  segue indo ao per-pixel (como hoje).

### 3b. Canvas-cache (Tiled/Stencil, canvas-fixo) — rota `stamp_dabs_canvas_cached`
O grain **Texturized (Tiled)** é canvas-fixo; a silhueta do shape é sempre **dab-relativa**. Então:
```
canvas_cacheable  ⟺  grain_is_canvas_static  ∧  shape_is_dab_relative_cheap
   grain_is_canvas_static = grain (Tiled|Stencil) ∧ !rake ∧ !random
   shape_is_dab_relative_cheap = shape estático dab-relativo (sem per-dab rot/scatter/count)
```
- `blit_canvas_cached` ([`stamp.rs`](../../crates/ph2d-painter-brush/src/stamp.rs)) já computa o
  **falloff por-pixel** (dab-relativo, barato) e lê o grain do cache de canvas. Passa a computar a
  **silhueta** por-pixel (falloff **ou** shape_alpha) e multiplicar pelo grain cacheado. Shape barato
  dab-relativo + grain caro canvas-fixo = combinação correta.

### 3c. Per-pixel (fallback) — rota `stamp_dabs_per_pixel`
Qualquer coisa que torne **um dos slots per-dab** (Rake/Random/Jitter-Rotate/Scatter/Count em shape ou
grain) ou o Accumulate-cap → per-pixel, resolvendo `shape_basis` e `grain_basis` por dab. É a rota
correta-por-construção; só fica mais cara.

### 3d. Chaves de cache ganham o shape
`StampKey` e `CanvasKey` ([`stamp_cache.rs:29-47`](../../crates/ph2d-tool-painter/src/tool/paint/stamp_cache.rs))
passam a incluir `shape: TextureSettings` + `shape_image_version: u64`. Trivial; invalida o mask quando
o shape muda.

> **Resumo:** a topologia das 4 rotas **não muda**; só as *condições* viram produto de 2 slots e a
> bake assa 2 slots. É generalização, não reescrita — alinha com [feedback: pipeline-inject-don't-cap].

---

## §4 — Determinismo (restrição #4, HR-5)

- **Gating estrito:** cada sorteio do Shape (Random rotation, Scatter posicional, Count/Count-Jitter)
  só puxa do `rng` **quando aquela feature está ativa** — espelha `per_dab` em
  [`jitter.rs:45-72`](../../crates/ph2d-painter-brush/src/jitter.rs). Um brush "tudo off" **não avança o
  RNG** e é byte-idêntico ao baseline (o teste `all_off_draws_no_randomness` é o padrão a replicar).
- **Ordem fixa de draw** (a declarar e testar): por dab, **(1) jitter de posição/scale/rotate existente
  → (2) shape_basis (random rot / random offset) → (3) grain_basis (random rot / random offset) →
  (4) shape scatter/count**. A ordem é arbitrária mas **congelada**; mudar a ordem muda o replay.
- **Transcendental-free:** o frame do shape usa o mesmo `rotate_by_degrees`/`random_unit`/`next_f32`
  da `texture.rs`/`jitter.rs` (só `+ - * / floor sqrt` + passo 1° baked). Nada novo aqui.
- **Count** (se entrar): N carimbos por passo é um loop determinístico; o offset/rotação de cada cópia
  vem do rng em ordem fixa. Gated: Count==1 ⇒ um carimbo ⇒ baseline.

---

## §4.bis — REFINAMENTO de UI (Enio 2026-06-25, aprovação)

O Enio aprovou e precisou a UI. Isto **reforça** a decisão §4 (Shape substitui o falloff) e
reorganiza o painel:

1. **O Falloff É a fonte procedural da silhueta.** Hoje o Falloff (dropdown + curva) mora no topo do
   painel ([`paint_brush.rs:197-215`](../../crates/ph2d-panel-painter-layers/src/paint_brush.rs)). Ele
   **migra para dentro de uma nova seção `Shape`** — porque o falloff *é* o Shape procedural default.
2. **Imagem atribuída ⇒ falloff claramente INATIVO.** Quando o Shape recebe uma imagem
   (`shape.kind == Image`), os controles de Falloff (dropdown + curva) ficam **visualmente
   desabilitados** (greyed) — a silhueta passa a ser a imagem. Sem imagem, o falloff é o silhueta.
3. **Preview do Shape**, igual ao preview da textura (`paint_texture_preview`): a seção Shape mostra a
   silhueta atual (a imagem, ou um preview do disco do falloff).
4. **Menu da Hierarquia: duas opções.** "Use as Brush Texture" vira **"Use as Brush Shape"** +
   **"Use as Brush Grain"** (`CTX_MENU_HIER_USE_AS_BRUSH_TEXTURE` →
   [`context_menu_overlay.rs:315`](../../crates/ph2d-editor-core/src/screens/hero/context_menu_overlay.rs) /
   [`menus.rs:150`](../../crates/ph2d-editor-core/src/ids/menus.rs) /
   [`hierarchy/event.rs:81`](../../crates/ph2d-panel-hierarchy/src/event.rs) /
   `EditorAction::HierUseAsBrushTexture` em [`action_bus.rs:194`](../../crates/ph2d-editor-core/src/action_bus.rs)).
   ⚠️ **Toca crates fora do painter** (editor-core + panel-hierarchy + shell) — autorizado pelo Enio
   (dono) para esta feature; mudança cirúrgica + testada.
5. **Renomear "Texture" → "Grain"** (label visível) para clareza do sistema (Shape vs Grain).

**Modelo de dados implicado:** os campos `falloff`/`custom_falloff`/`hardness` de `BrushSpec`
continuam sendo a **silhueta procedural** (default), e `shape: TextureSettings` (kind None→falloff,
Image→imagem) é o override. O Shape **não** expõe kinds procedurais no picker (o falloff *é* o
procedural); a única fonte de imagem é `Image`. Mapping do Shape é sempre ViewPlane (dab-relativo).

## §5 — Painel (restrição #5: não quebrar a Texture-LAYER; vigiar LOC)

### 5a. Duas seções colapsáveis: **Shape** e **Grain**
Hoje há uma seção "Texture" (`paint_texture_section`,
[`paint_texture.rs:37`](../../crates/ph2d-panel-painter-layers/src/paint_texture.rs)). Plano:
- **Renomear a seção atual "Texture" → "Grain"** (label + ids `PAINTER_BRUSH_TEXTURE_*` podem ficar com
  o nome interno; o **label visível** vira "Grain"). É o slot que já existe; semântica idêntica.
- **Adicionar uma seção "Shape"** acima da Grain, com um subconjunto de controles:
  - **Shape source** (Kind picker — na prática **Image** + um botão "Import"; os procedurais ficam
    disponíveis mas o foco é Image), **preview**, **Mapping** implícito ViewPlane (silhueta é sempre
    dab-relativa — esconder o dropdown Mapping no Shape), **Rake** (Follow-Stroke), **Random**
    (rotação), **Angle**, **Flip X/Y** (W2), **Roundness/Count/Scatter** (W2). **Sem** Depth, **sem**
    Color Ramp (a silhueta não colore).
  - **Grain** mantém tudo que a "Texture" tem hoje (Mapping View/Tiled/Stencil, Rake, Random, Angle,
    Offset, Size, params, Color Ramp) **+ Depth** (slider novo).

### 5b. Reuso seguro pela Texture-LAYER (o ponto crítico anti-regressão)
`paint_texture_section(..., compact: true)` é chamado pelo **editor de Texture-LAYER**
([`paint_texture.rs:227-247`](../../crates/ph2d-panel-painter-layers/src/paint_texture.rs)). Uma
Texture-LAYER é um **grão de cobertura total** (mapeia ao conceito **Grain**, não Shape). Plano:
- A função que a Texture-LAYER reusa passa a ser **a seção Grain** (é o que ela já é). O **Shape é
  brush-only** — a Texture-LAYER **não ganha** seção Shape (uma layer não tem "ponta").
- **Refactor de baixo risco:** extrair a seção em `paint_grain.rs` (= o `paint_texture.rs` atual,
  re-rotulado) e um `paint_shape.rs` novo. A Texture-LAYER chama `paint_grain_section(..., compact)`
  exatamente como hoje. **Os dois call-sites (brush vs layer) preservados** — gate de regressão: os 21
  testes do painel + smoke.
- ⚠️ **LOC:** `paint_texture.rs` tem ~524 linhas (sob 600). Renomear p/ `paint_grain.rs` + extrair
  `paint_shape.rs` mantém cada arquivo sob o cap. **Vigiar `architecture_panel_loc_cap`** na wave do
  painel (não no fim). Cuidado com apóstrofo em comentário (quebra o parser de função do painel —
  [HANDOFF §2](HANDOFF_shape_grain_dual_texture.md)).

### 5c. Ids novos (padrão Jitter Spacing, costura ponta-a-ponta)
Novos `PAINTER_SHAPE_*` em
[`ids/chrome/painter.rs`](../../crates/ph2d-editor-core/src/ids/chrome/painter.rs) — um por controle do
Shape (kind, import, rake, random, angle, offset x/y, size x/y, params, flip x/y, …) **+** o
`PAINTER_BRUSH_GRAIN_DEPTH`. Cada id atravessa os **7 sites de costura**
([DIRETIVA §2](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)):
`id → register em populate.rs → paint+hit-index em paint_shape.rs → arm em event.rs → ToolPanelEvent →
handle_panel_event → setter`. O Jitter Spacing
([`painter.rs:189`](../../crates/ph2d-editor-core/src/ids/chrome/painter.rs) → `paint_stroke.rs:393` →
`populate.rs` → `event.rs:534` → `jitter_settings.rs`) é o **template ponta-a-ponta** a copiar.

### 5d. Carga de imagem nos 2 slots (restrição implícita; W4)
Hoje "Use as Brush Texture" (menu da Hierarquia) seta `texture_image`. Com 2 slots, o usuário precisa
escolher **Shape** ou **Grain** como destino. Opções:
- **(A)** Dois itens de menu: "Use as Brush **Shape**" / "Use as Brush **Grain**". Simples, explícito.
- **(B)** Um seletor de slot ativo no painel + um botão "Import" por seção.
> **Recomendação: (B)** botão **Import** dentro de cada seção (Shape/Grain) — local, óbvio, e não polui
> o menu da Hierarquia. Manter "Use as Brush Texture" → Grain por back-compat.

---

## §6 — Back-compat: a prova byte-idêntica (restrição #6, regra de ouro)

**Invariante:** com `shape.kind == None` **e** `grain_depth == 1.0`, o caminho de stamp produz **bytes
idênticos** ao HEAD para qualquer brush/dab/textura.

**Por que se sustenta (por construção):**
1. `SILHUETA = falloff_weight(t)` quando `shape.kind==None` — **idêntica** linha de
   [`dab.rs:396`](../../crates/ph2d-painter-brush/src/dab.rs).
2. `g_eff = 1 + (g−1)·1.0 = g` quando `grain_depth==1.0` — **idêntico** a `w *= s`.
3. `render_stamp_mask` com shape None assa `falloff × grain` — **idêntico** ao `falloff × texture`
   de hoje ([`stamp.rs:52-56`](../../crates/ph2d-painter-brush/src/stamp.rs)).
4. Ordem de draw do RNG: shape None ⇒ `shape_basis` não puxa do rng ⇒ a sequência consumida pelo grain
   e pelo jitter existente é **inalterada**.

**Teste de regressão W0 (inegociável):** um teste de mesa que pinta o MESMO dab (mesmo seed, mesma
textura/grain) **antes e depois** do refactor e compara o buffer **byte-a-byte**. Plus: re-rodar os
132/127/21 e exigir contagem idêntica (nenhum teste muda de resultado). Plus: um teste
`shape_none_and_depth_one_is_byte_identical_to_falloff_times_grain`.

---

## §7 — Persistência (restrição #7) — confirmado: ZERO impacto

Grep nas crates do painter: `BrushSpec` deriva apenas `#[derive(Clone, Copy, Debug, PartialEq)]`
([`spec.rs:30`](../../crates/ph2d-painter-brush/src/spec.rs)); **não há `Serialize`/`postcard`** em
`ph2d-painter-brush` nem em `ph2d-tool-painter` para o brush. O brush é **estado de ferramenta**, não
entra no save. As **Texture-LAYERS** (que *são* serializadas) usam `TextureLayer` (kind/params/size/
offset/ramp) e mapeiam ao **Grain** — **não ganham Shape**, então **`SCHEMA_VERSION` (3) não muda**.
→ Nenhuma migração de save. (Se um dia o brush virar preset persistido, aí sim entra versionamento — é
follow-up, fora desta linha.)

---

## §8 — ADR-0100 (decisão arquitetural)

> Arquivo a criar:
> [`docs/architecture/decisions/0100-dual-texture-slots-shape-grain.md`](../architecture/decisions/0100-dual-texture-slots-shape-grain.md).
> Esboço abaixo (o ADR final é curto; o detalhe técnico vive neste doc).

**Título:** Dois slots de textura no brush — Shape (silhueta) + Grain (textura), paridade Procreate.
**Status:** Proposto (aguarda aval do Enio para implementar).
**Contexto:** o brush tem **um** slot de textura, que na prática é o Grain; a silhueta é sempre o
falloff redondo procedural. O Procreate separa **Shape** (ponta/alpha do carimbo) de **Grain** (textura
dentro), ortogonais. Falta-nos a "ponta de imagem".
**Decisão:**
1. Generalizar a cobertura para `silhueta × grain × dinâmica`, cada slot com **default neutro**
   (silhueta→falloff, grain→1.0) ⇒ baseline byte-idêntico.
2. **Shape SUBSTITUI** o falloff quando atribuído (fidelidade: formas duras crocantes); falloff segue
   como silhueta default e edge do brush redondo.
3. Reusar **`TextureSettings`** para os 2 slots (um dono só); renomear `texture`→`grain`, adicionar
   `shape` + `grain_depth`. Buffer de pixels do Shape em `PaintState` (espelha `texture_image`).
4. Caches: as 4 rotas viram **produto lógico de 2 slots**; `render_stamp_mask` assa os 2; topologia
   inalterada.
5. Brush **não serializado** ⇒ sem impacto de save. Texture-LAYER = Grain (sem Shape).
**Alternativas rejeitadas:**
- **Estender o slot único** (um modo "shape" no mesmo slot): não dá ortogonalidade (não dá shape+grain
  simultâneos) — o objetivo inteiro.
- **Shape MULTIPLICA o falloff:** erode formas duras na borda do footprint; infiel ([04 §4](04_pesquisa_shape_grain_procreate.md)).
- **`ShapeSettings` separado:** duplica frame/helpers; rejeitado por "um dono só".
**Consequências:** +1 slot de knobs `Copy` + 1 buffer em `PaintState`; cache rules mais ricas (testar
paridade cached↔per-pixel); painel ganha 1 seção (brush-only); back-compat provado por teste byte-a-byte.
**Kill-criterion (restrição "alvo irrefutável", [DIRETIVA §5](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)):**
se, após a W0, o caminho de stamp **não** for byte-idêntico com shape=None+depth=1, ou se o FPS de um
brush texturizado regredir **> 10%** em um stroke de referência (medir antes/depois), a feature **não
existe nesta forma** — voltar à tag `painter-pre-shape-grain-2026-06-24`.

---

## §9 — Conjunto de aceitação concreto (congelado antes do build)

A implementação (rodada seguinte) só fecha quando **todos** verdes:
1. **Byte-idêntico:** teste de mesa `shape_none_depth_one_byte_identical` + contagem 132/127/21 intacta.
2. **Shape de imagem pinta a silhueta:** e2e `paint_begin→extend→finish` com um Shape de teste (ex.:
   quadrado/estrela) e asserção de que o footprint pintado tem a forma (pixels dentro on, fora off),
   **não** o disco do falloff.
3. **Grain ortogonal:** e2e shape=estrela + grain=Noise ⇒ a estrela tem grão dentro (cobertura
   modulada), bordas da estrela preservadas.
4. **Moving vs Texturized do grain** seguem corretos (parity cached↔per-pixel; já coberto, re-verificar
   com shape ativo).
5. **Depth:** Depth=0 ⇒ grão some (cobertura = silhueta cheia); Depth=1 ⇒ hoje; intermediário lerp.
6. **Determinismo:** all-off não avança RNG; mesmo seed → mesmo buffer (replay).
7. **Painel:** seam test (`ph2d-ui-testkit`) dirige um id novo do Shape (ex.: Import/Angle/Rake) e
   afirma o efeito no `BrushSpec.shape`; Texture-LAYER **inalterada** (21 testes + smoke).
8. **LOC + clippy + nextest-impacted** verdes; smoke do Enio (pintar e VER a ponta de imagem).

→ **Plano de waves em** [`06_plano_dois_slots_textura.md`](06_plano_dois_slots_textura.md).
