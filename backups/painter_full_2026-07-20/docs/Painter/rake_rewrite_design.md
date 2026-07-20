# Rake rewrite — design (2026-06-26)

> **Decisor:** Enio (veredito "temos um rake que nunca funciona", handoff `HANDOFF_rake_rewrite.md`).
> **Implementação:** clean-room reescrita. ADR: [`0101-rake-heading-on-dab-length-weighted-ema.md`](../architecture/decisions/0101-rake-heading-on-dab-length-weighted-ema.md).
> **Crates:** `ph2d-painter-brush` (motor), `ph2d-tool-painter` (consumidor). Brush NÃO é contract-gateado.

## 1. Causa-raiz (por que o Rake nunca funcionou)

A matemática do frame de textura (`texture::dab_basis`) sempre esteve **correta** — o branch Rake só
faz `u = normalize_or(dab_dir, angle)`. O bug era a **FONTE** de `dab_dir`.

O Rake antigo (`tool/paint/rake.rs::advance_rake` + `brush_settings::dab_tangent`) **reconstruía** a
direção a jusante, no tool, a partir da **corda entre centros de dabs consecutivos**. Mas:

- os dabs ficam a **~3 px** um do outro, sobre um spline **Catmull-Rom suavizado pelo estabilizador**
  (lazy-mouse). A *direção* de uma corda de 3 px é dominada por curvatura local + lag do estabilizador
  = **ruído**.
- Suavizar a *saída* (lerp v1, acumulador long-baseline v2) **não recupera** uma direção cuja *entrada*
  já está corrompida. v1 ficou anárquico; v2 ainda dependia da densidade de dab (o acumulador re-mira
  por nº de cordas, não por distância real) → comportamento mudava com tamanho/spacing do pincel.
- Havia **dois** rakes paralelos (Shape + Grain, 4 campos em `PaintState`), cada um com seu acumulador.

**Prova empírica:** a `advance` da heading, alimentada com a corda crua por-segmento (a classe de entrada
antiga), faz o teste de arco (`arc_stroke_heading_tracks_the_tangent_and_rotates_monotonically`)
**FALHAR** na asserção de rotação monotônica — a heading oscila dab-a-dab. Com o EMA, passa.

## 2. Pesquisa — como motores maduros fazem "rotação segue o traço"

Resumo (fontes no fim). Dois modelos:

| Motor | Modelo de heading | Length-weighted? | Fallback parado | Eixo |
|---|---|---|---|---|
| **MyPaint** | **EMA do vetor velocidade**, `fac = 1−exp(−‖Δ‖/T)`, depois `atan2` | **SIM** (por distância) | coasta (fac→0) | — |
| **Krita** | `atan2(p2−p1)` instantâneo + `lastAngle`; "Fade" = escala-de-comprimento **proporcional ao tamanho do pincel** | parcial (Fade) | `lastAngle` | — |
| **Blender** | resample por distância-gate (`r`=20px / 4px no pre-roll), segura `last_rake_angle` | gate, não EMA | segura último | **+π/2 = atravessa** |
| **Procreate** | Shape Rotation "Follow Stroke" + Azimuth (comportamental) | — | — | ao longo |
| **Photoshop** | Angle Jitter → Control = Direction / Initial Direction | — | — | ao longo |

**Tirada:** o modelo **MyPaint** (EMA do vetor, length-weighted) é o mais limpo e o único independente da
densidade de dab. Krita confirma amarrar a escala-de-comprimento ao **tamanho do pincel**. Blender
confirma o fallback "segura a última heading" no parado.

## 3. A decisão

**A heading do traço é propriedade de primeira-classe do `Dab`, computada UMA vez no motor**, onde a
geometria do caminho é conhecida e a tangente é limpa. O tool só **lê** `d.dir`.

### Algoritmo — EMA da tangente, length-weighted, em espaço de vetor (`heading.rs`)

```
// por passo de comprimento de arco `step_len` (px) com tangente unitária `t`:
α       = step_len / (step_len + L)          // racional, length-parametrizado (HR-5: sem exp/atan2)
mixed   = heading + α·(t − heading)
heading = mixed / ‖mixed‖                     // sqrt permitido
L       = max(½·diâmetro, 8px)                // Krita's Fade: brush-relative, com piso
```

Decisões e *porquês*:

1. **Espaço de VETOR, não ângulo.** Lerp de vetor é **wrap-safe**: numa reversão ~180° o vetor passa por
   comprimento-zero (snap para a nova tangente), nunca gira "pelo caminho longo" como um lerp de ângulo
   faria. Elimina a "chicotada" sem caso especial de wrap.
2. **Length-weighted (`α = Δs/(Δs+L)`).** O blend depende da **distância percorrida**, não do nº de
   dabs ⇒ o mesmo arco físico dá a mesma heading em qualquer spacing/tamanho. (Era o defeito do v2.)
3. **No MOTOR (`stroke.rs::walk_space`).** Ali o `dir` do segmento do spline já é limpo e `to_next` (o
   incremento de comprimento de arco até o próximo dab) é conhecido. `dab_at` carimba `self.heading`.
4. **HR-5 transcendental-free.** `α` é racional; só `sqrt` (normalização). Zero RNG → determinístico,
   reprodutível por seed.
5. **Unifica Shape + Grain.** `d.dir` é propriedade do caminho, não do slot. Os **dois** `dab_basis`
   leem o mesmo `d.dir` ⇒ os 4 campos de estado de rake e os 2 acumuladores **somem**.

### Casos-limite

- **Início do traço:** `heading = [0,0]` (reset em `begin`/`fill_*_preview`). `dab_basis` cai no Angle
  base (`normalize_or` já trata `[0,0]`). O primeiro travel faz `advance` dar snap na 1ª tangente.
- **Parado/lento (`step_len≈0`):** segura a heading estabelecida (igual ao Blender).
- **Reversão ~180°:** lerp passa por ~zero → snap na nova tangente (sem spin).
- **Anchored** (não passa por `dab_at`/EMA): heading = direção do arraste (`cursor − anchor`), setada
  explicitamente; `[0,0]` antes de arrastar.
- **Line/Curve/Circle/Polygon preview:** Line salva/restaura `heading` na tupla de preview (senão deriva
  entre re-stamps); as formas resetam `heading=[0,0]` no início do fill (re-miram pela própria geometria).

### Convenção de eixo

Mantida a existente: `u = base = direção do traço` (ao longo), conforme o branch Rake intacto de
`dab_basis`. É cosmético (handoff §3) e não houve mudança de UX pedida; flip para "atravessa" (Blender
`+π/2`) é follow-up de 1 linha se o Enio preferir.

### Cache

Inalterado: `is_cacheable`/`is_canvas_cacheable` continuam exigindo `!rake` (cada dab tem seu frame numa
curva). Cache mais esperto por-heading é follow-up, não bloqueia o MVP.

## 4. O que foi arrancado vs reescrito

**Arrancado** (cirúrgico, sem `git revert`): módulo `tool/paint/rake.rs` inteiro (`advance_rake` +
consts + teste); `brush_settings::dab_tangent`; os 4 campos `rake_dir`/`rake_accum`/`shape_rake_dir`/
`shape_rake_accum` em `PaintState` + 4 inits + 4 resets; os 8 writebacks + 4 chamadas `advance_rake` nos
2 loops de `stamp_cache.rs`; o `mod rake;` e o `use super::rake::advance_rake;`.

**Reescrito:** novo `heading.rs` (EMA puro, testável isolado); campo `dir:[f32;2]` no `Dab`; campo
`heading` em `Stroke` (update em `walk_space`, reset em `begin` e nos fills de forma, save/restore no Line
preview, set explícito em `anchored_dab`); os 2 loops de `stamp_cache.rs` passam `d.dir` aos dois
`dab_basis` (preservando `footprint` e `extra_rot` intactos).

## 5. Como foi provado (e2e, não só unit-verde)

- **`heading.rs` (8 testes):** snap no início, segura no parado, sempre unitário, converge, estável em
  reta, **longer-step puxa mais** (length-weighting), reversão dá snap, `smooth_len` brush-relative.
- **Arco e2e (`stroke/tests.rs`):** pinta um quarto-de-círculo COM estabilizador 0.5 (o caso que
  corrompia o rake antigo) e asserta (a) cada `d.dir` alinha com a tangente do arco (dot>0.9), (b)
  **rotação monotônica, zero reversões** (a "anarquia" antiga), (c) varre ~90° (começa +y, termina −x).
  Reta dá heading ~constante. **Spacing-independence:** mesma posição de arco, dense vs sparse, headings
  concordam (dot>0.96). **Falha-com-o-antigo provada:** trocando o EMA pela corda crua, o teste de arco
  falha na asserção de monotonia.
- **Byte-identity Rake-OFF (`texture/tests.rs`):** `dab_basis` com Rake off é IGUAL para heading
  arbitrária vs `[0,0]` (= o frame do Angle puro) — a heading nova não toca o brush não-Rake.
- **Coexistência Jitter-Rotate:** com Rake on, `dab_basis` com `extra_rot` não-identidade = a heading
  multiplicada-complexa pelo jitter (compõe POR CIMA, não substitui); identidade deixa a heading nua.

## 6. Fontes da pesquisa

- MyPaint (EMA de direção, length-weighted): `mypaint-brush.c` / `brushsettings.json` — github.com/mypaint/libmypaint
- Blender `paint_calculate_rake_rotation` (gate `r=20/4`px, `+π/2`, segura último): `blenkernel/intern/paint.cc`
- Krita Drawing Angle / Fade / `directionBetweenPoints`: docs.krita.org tablet_sensors + `kis_algebra_2d.cpp`
- Procreate Follow-Stroke/Azimuth: help.procreate.com brush-studio-settings
- Photoshop Angle Jitter "Direction"/"Initial Direction": helpx.adobe.com adding-dynamic-elements-brushes
