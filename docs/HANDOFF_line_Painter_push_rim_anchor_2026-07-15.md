# HANDOFF — o aro do Push ancora no CORPO da tinta, não no círculo do gizmo (`line/Painter`, 2026-07-15)

> **FECHADO (`fd77f9c5`, pendente smoke do Enio).** O smoke REPROVOU o desenho do bow wave: *"é usada
> a circunferência do gizmo do brush para empurrar a massa e não o alpha do falloff"*. A mecânica da
> onda estava certa; **a ÂNCORA do aro estava errada** — nascia em `t = 1` (círculo geométrico) e num
> pincel macio a tinta termina bem antes. Corrigido: o aro nasce onde o CORPO termina. Este doc virou o
> registro de fechamento; a fila que sobra está no §8.

## 1. O que era o bug (o diagnóstico do Enio, exato)

O aro lateral (`bank_dab_push`) e o lóbulo da onda (`wave_lobe`), ambos em
`crates/ph2d-painter-brush/src/height_push.rs`, definiam seu domínio em `t = 1` — a borda GEOMÉTRICA
do dab (a circunferência do gizmo). Mas o alpha da tinta num falloff macio morre muito antes: `W_TAIL`
(0,35 de cobertura) cai em **t ≈ 0,61 no Smooth** (`docs/Painter/16_impasto_plano_implementacao.md`
§14.1). Num pincel Smooth de raio 40 a tinta visível termina a ~24 px e o aro nascia a 40 px — **16 px
de tela nua entre a tinta e um anel de massa com borda interna perfeitamente circular.** As zonas do
gate mediam PARA ONDE o volume vai (frente/lado/trás), nunca ONDE a borda interna nasce — gate verde,
contradito pelo olho, porque a fixture não continha o fenômeno.

## 2. O fix (a lei da casa, do outro lado)

A regra do filme já dizia *"um pincel que não deposita corpo não deposita tinta"* (`height_film`,
`W_TAIL`, `body_profile`). O aro ganhou a MESMA lei: **a tinta desloca para onde o CORPO termina.**

- **`height_film::body_edge_t(spec) -> f32`** (novo): o `t` onde a silhueta cruza `W_TAIL`, por
  bisseção sobre `spec.falloff_weight` (monótona), **1× por traço** (hoisted, não por dab). Falloff
  duro (Constant, hardness ≥ 1) devolve **exatamente 1.0** por um fast-path (`RIM_PROBE = 0.999`): a
  bisseção pararia um fio abaixo e deslocaria o aro sub-pixel; o fast-path garante byte-identidade.
- **`height_push::rim_t0(spec, has_shape) -> f32`** (novo): a porta única da âncora. `has_shape` ⇒
  `1.0` (uma Shape image/procedural não tem borda de corpo radial — carimbo tem borda dura). Senão,
  `body_edge_t`.
- **`height_push::rim_lift(t, t0, inv_reach)`** (novo, privado): o domínio+perfil compartilhado —
  `0` para `t ≤ t0`, senão `push_rim_weight((t − t0) · inv_reach)`. **`bank_dab_push` E `wave_lobe`
  chamam ele** ⇒ o aro lateral e o lóbulo da onda nascem no MESMO círculo. `inv_reach = radius/reach`
  intacto ⇒ o reach conta a partir de `t0` ⇒ **aro de largura constante** (recuar a âncora sem recuar
  o reach esticaria o aro num espículo — a lição do `push_reach_px`).
- **Chamadores** (`impasto.rs` deposit, `sculpt_session.rs` Conserve): calculam `rim_t0(brush,
  shape_active)` UMA vez antes do laço de dabs e passam a todos os `bank_dab_push`/`wave_lobe`.

## 3. ⚠️ O Conserve compartilha o motor — e o desenho MOVEU (re-smoke)

`bank_dab_push` é chamado pelo W5 (Scrape/Chisel + Conserve). O pincel de sculpt default é **Smooth
macio** (`state_default.rs`), então `t0 ≈ 0,60` e **o aro do Conserve também recuou pra dentro** — a
pilha agora encosta na tinta em vez de anelar o raio geométrico. Escolhi **UMA lei + re-smoke**
(a física é a mesma dos dois lados), não parametrizar a âncora. **O que NÃO mudou:** o ledger fecha < 1 %,
`OFF` é byte-idêntico, o re-stamp é idempotente. O render de referência (Scrape+Conserve, cena 6 da
sonda) segue bom, marginalmente mais colado. **RE-SMOKE do Conserve declarado.**

## 4. Gates (mutation-tested, ciclo verde→RED→verde)

Em `crates/ph2d-painter-brush/src/height_tests.rs`:

- **`the_rim_rises_from_the_body_edge_not_the_geometric_rim`** (novo): banco de UM dab dirigido, em
  ISOLAMENTO (plano zerado, sem mordida, sem vizinhos, `paint` zerado ⇒ sem supressão) — o 1º texel
  positivo na perpendicular é a âncora pura. Raio 40: borda interna ~24 px (`t0·r`), bem dentro do raio
  geométrico 40. O discriminador **não-autorreferente** é `inner < radius·0.75` (contra o raio, um
  fato fixo, não contra `t0`, que a mutação move). **RED** se a âncora volta a `t=1` (`rim_t0`→1.0 OU
  `rim_lift` `t≤t0`→`t≤1.0`).
- **`a_hard_falloff_anchors_the_rim_at_the_geometric_rim`** (novo): `rim_t0(Constant) == 1.0` exato +
  **fingerprint** (banco na âncora resolvida == banco em `t0=1.0` hard-coded, byte a byte) + borda
  interna ≈ raio. **RED** se o fast-path `RIM_PROBE` cai (Constant bisseciona pra 0,9999).
- **`the_ploughed_paint_waits_at_the_strokes_frontier`** + o tool-side
  **`the_deposits_wave_travels_through_the_real_stroke`**: as zonas agora medem contra a **FRONTEIRA
  DA TINTA** (`t0·radius`), não o raio geométrico — senão o aro encostado na borda do corpo seria
  contado como "dentro do canal". Ahead subiu (52 %/59 %), swath caiu (0,3 %/0,8 %). Com falloff duro
  (`t0=1`) a fronteira = raio e o split é exatamente o pré-fix.

Placar de mutação desta tarefa: **3 provadas** (rim_t0→1.0 · rim_lift `t≤1` · drop RIM_PROBE), cada uma
sobre o gate visto VERDE.

## 5. Perf

O `body_edge_t` é **1 bisseção de 24 passos por traço** (hoisted), não por dab ⇒ desprezível. O aro
varre o MESMO bbox (`radius + reach`) e a MESMA largura de aro (`reach`), só deslocada pra dentro ⇒
**perf-neutro por construção**. Não regatei os gates de perf (`--ignored --release`), mas nada no
caminho quente mudou de custo.

## 6. Superfície tocada (7 arquivos, `fd77f9c5`)

- `ph2d-painter-brush/src/height_film.rs` (`body_edge_t`) · `height_push.rs` (`rim_t0`, `rim_lift`,
  `t0` nos 2 kernels) · `height_tests.rs` (2 gates novos + zonas contra a fronteira).
- `ph2d-tool-painter/src/tool/paint/impasto.rs` (deposit: hoist + 3 call-sites) ·
  `sculpt_session.rs` (Conserve: hoist + 1 call-site) · `tests.rs` (tool-side wave gate → fronteira).
- `shells/desktop/src/render_loop/push_look_probe.rs` (cena 5 nova: pincel grande e macio que para na
  tinta grossa — onde o colar era mais gritante).
- **Contratos congelados intactos.** Nenhum schema. Nenhum arquivo de outra linha.

## 7. O INSTRUMENTO (o before/after já rodado)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_PUSH_LOOK_DIR=/tmp/push_look cargo test -p ph2d-host-desktop \
  probe_push_render_and_look -- --ignored
```
7 PNGs (`push_look_probe.rs`). **Cena 5 (`5_push_stops_big_soft`)** é a nova: pincel raio 45 macio,
Push=1, que para no meio da tinta grossa. No before (âncora `t=1`) o anel da ponta arredondada fica
DESTACADO da tinta, com um vão escuro; no after (âncora no corpo) ele encosta na tinta. Cena 6 é o
Scrape+Conserve de referência.

Smoke final (Enio): `PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop` → tinta grossa →
pincel MACIO menor, Push alto → atravessar e parar no meio. O certo: o aro/onda nascendo da borda MACIA
da tinta (sem tela nua entre tinta e colar, sem círculo perfeito). **E** re-smoke do Conserve (Scrape,
checkbox Conserve, raspar tinta grossa).

## 8. Aberto (a fila que sobra)

- **Smoke do Enio** deste fix (deposit Push) **+ re-smoke do Conserve** (o desenho moveu — §3).
- **Conserve p/ Flatten/Fill**: decisão de design (conservar quem ADICIONA exige decidir de onde o
  volume vem) — fora do v1.
- **Knob de `forward_share`?** hoje const `0.6` (`DEPOSIT_FORWARD_SHARE`).
- **Custom falloff não-monótono:** `body_edge_t` bisseciona assumindo monotonia; um curva Custom
  não-monótona ainda devolve um `t0` são (a raiz que a bisseção acha), mas "borda do corpo" é
  ambígua ali — não é um caso que a arte real produz; anotado, não gateado.
