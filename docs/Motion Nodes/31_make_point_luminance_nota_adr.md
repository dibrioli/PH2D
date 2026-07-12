# 31 — Nota-ADR: Make Point + Luminance (M1 — adapters valor↔geometria↔cor)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: 2/1/8). Adiciona os **adapters** que faltavam do lote M1 (doc 01 §1.7):
**`motion.make_point`** (campos de valor → geometria) e **`motion.luminance`** (cor → campo de valor).

> **Achado importante (corrige a recomendação anterior):** eu vinha recomendando a **`motion.expression`**
> como "próxima self-contained". **Ela NÃO é self-contained** — uma fórmula editável precisa de um **param
> string**, e `ParamSpec { name, default: f32 }` é **f32-only** (verificado). Param tipado é o **M4.N1
> (ParamSpec tipado, `ParamValue{F32,Vec2,Color,Enum,Bool}`) — contrato congelado, EXIGE ADR** (§6). Então a
> expression é **ADR-gated**, não fan-out. Esta fatia pivota pros adapters, que são f32-only e self-contained.

---

## 1. O problema

O domínio de valor produzia campos (lfo/math/map_range/instance_field) mas nada **atravessava** entre os
domínios: não dava pra transformar um campo de valor em **geometria** (plotting/data-driven), nem ler a **cor**
de volta pra um valor (aparência → tamanho/posição). São as pontes que fecham o ciclo valor↔geometria↔cor.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.make_point` (valor → geometria):** o padrão-ouro é o construtor de ponto a partir de atributos —
Houdini `@P = set(x, y, 0)` / o "Add" SOP dirigido por atributo. Veredito:

- `P[i] = (x[i], y[i])` — os inputs `x`/`y` são **value fields** (length-1 broadcasta, length-N mapeia
  element-wise, ausente = 0); a contagem é o maior entre o `in` (carrier opcional, cujas colunas passam) e os
  dois campos. Alimente dois `value.lfo` com `phase_stagger` distintos → um **Lissajous**. HR-5: empacotar
  coordenada, sem matemática. `Effect::Pure`, Utility. Testado por: empacota x,y element-wise (falsifica
  transpor) · length-1 broadcasta · campo ausente = 0 · cook (in fixa a contagem).

**`motion.luminance` (cor → valor):** o padrão-ouro é a **luma Rec. 709** — `v = 0.2126·R + 0.7152·G +
0.0722·B` (pesos perceptuais padrão). Veredito:

- Lê a coluna `tint` (RGBA linear) e emite um **campo de valor** — output **tipado `VALUE`** (como
  `value.instance_field`), pra plugar direto em qualquer value input (o `t` de um color-ramp, um math). O
  inverso do color-ramp (valor→cor), fechando o ciclo: a aparência de uma instância pode dirigir seu
  tamanho/posição. Tint ausente = preto (0). HR-5: soma ponderada. `Effect::Pure`, Utility. Testado por:
  branco→1/preto→0/cinza→0.5 · **verde > vermelho > azul** (ordem Rec.709; falsifica média chapada) · ausente→0
  · cook.

**Decisão de tipo (lição do 1º smoke headless):** o `luminance` primeiro emitia `INST_VEC2` (passando a
geometria) — mas aí o `t` (VALUE) do color-ramp **não valida** contra `INST_VEC2`. O correto pra um adapter
cor→valor é **emitir VALUE puro** (só a coluna `v`), como o `instance_field`. A geometria segue na linha
principal; o valor é um ramo.

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-make-point` (drop-crate, VALOR→GEOMETRIA):** `(in?, x?, y?) → out`. Empacota campos de
valor em `P`. `Pure`, Utility. Display "Make Point".

**`ph2d-node-motion-luminance` (drop-crate, COR→VALOR):** `(in) → out(VALUE)`. Rec.709 luma do `tint` → campo
`v`. `Pure`, Utility. Display "Luminance".

**Cena boot — DUAS cenas** (`motion_demo_strobe.rs`, 13 nós):

```
ESQUERDA (make_point): grid → lfoX(stagger)/lfoY(stagger) → make_point → tint → move(−6) → output
DIREITA  (luminance):  grid → color_ramp(Rainbow) → luminance → color_ramp(t, Heat) → move(+6) → output
```

- **Lissajous** (x≈−6): o grid 8×8 fixa a contagem (64); dois `value.lfo` com `phase_stagger` 3/64 e 2/64 dão
  x e y por-instância → uma **figura de Lissajous 3:2** que o playhead anima.
- **recolor por brilho** (x≈+6): o grid é colorido por um Rainbow; o `luminance` lê o brilho por-instância → um
  campo `v` que indexa um ramp **Heat** → recolorido pela própria luminância.

**Testes (8 unit + 3 integração):** make_point (4: empacota, broadcast, ausente-0, cook); luminance (4:
branco/preto/cinza, G>R>B, ausente-0, cook). Integração no shell: `the_lissajous_is_plotted_and_animates` (64
pts + span da curva + anima + esquerda) · `the_grid_is_recoloured_by_luminance` (100 pts + **>5 cores** do Heat
[falsifica luminance morta] + direita) · `the_default_document_replays_deterministically`.

**Bug pego (costura não-testada — o alvo da DIRETIVA):** esqueci de conectar `make_point → tint → move →
output` (só liguei as entradas); o output ficou órfão → cozinhava **0**. **Validou** (nenhuma aresta inválida)
mas cozinhava vazio. O teste falsificável (`assert count == 64`) pegou na hora. Lição reforçada:
verde-de-validação ≠ costura viva.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-make-point`, tipo `motion.make_point` | nova | nome novo |
| crate `ph2d-node-motion-luminance`, tipo `motion.luminance` | nova | nome novo |
| `ph2d-node-registry-init` regenerado (68 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas, 13 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo (só path crates). Machete verde. Zero tofu.

## 5. O que fica (a cauda M1 quase fecha)

Adapters value↔geometria↔cor abertos. O que resta da cauda M1:
- **Adapters restantes (self-contained):** `make-line`/`make-vec2` (variações de make_point) · `threshold`/
  `gate` (mas `pulse.threshold` já cobre o Schmitt) · `value-to-color` (subsumido pelo `color_ramp` t input).
  A maioria é subsumida ou marginal.
- **`motion.expression` — ADR-GATED (M4.N1 ParamSpec tipado).** É o item de maior valor que resta do M1, mas
  precisa de **ordem/ADR do Enio** pra descongelar o contrato (param string). **Não fazer como fan-out.**
- **M2:** wiring do scrub-back · `delay` · `buoyancy`.
- **Cross-module:** `distribute-path` (vetor) · `slit-scan` (time-scope).
- **Fronteiras:** **M4** (Rig+FX, necks) · **M5** (GPU, ADR).

> Com make_point + luminance o ciclo valor↔geometria↔cor fecha. A cauda M1 self-contained está
> **essencialmente esgotada** — o que resta de alto valor (**expression**) é **ADR-gated**, e o resto é
> subsumido/marginal. É a hora clara de **integrar** as 16 fatias, ou pedir o ADR do ParamSpec tipado pra
> destravar a expression (aí vira uma linha foundational, não fan-out).
