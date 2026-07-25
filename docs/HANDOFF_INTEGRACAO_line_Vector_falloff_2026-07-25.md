# HANDOFF DE INTEGRAÇÃO — `line/Vector` · FALLOFF + TWIST (2026-07-25)

**Estado:** linha FECHADA para DUAS waves da mesma sessão, aguardando ordem EXPLÍCITA de
integração. **NÃO integrei nem pushei** (§0.7 / [[feedback_integration_only_enio_command_end_of_all_lines]]).

- **Wave 1 — o FALLOFF** (commit **`772f830eb`**): **smoke APROVADO pelo Enio.** Detalhe abaixo.
- **Wave 2 — o TWIST** (commit **`afe16fce5`**): **pendente de smoke** (`PH2D_BUILD_SMOKE=29`). Seção
  própria no fim deste documento.

> Os números de commit e schema abaixo são da Wave 1; a Wave 2 **não move o schema** (variant
> apendado, `PROJECT_SCHEMA` fica 29). As duas waves tocam o MESMO conjunto de arquivos
> (`ph2d-vec-scene/src/effect*.rs`, `fx_warp_presets.rs`, `MAX_FX_KINDS`), então integram JUNTAS.

## O que é

A **10ª ferramenta da pesquisa `20_*`** — o **Falloff** do Cavalry, "a ideia mais exportável da
pesquisa inteira" (§2.2). Um **campo escalar espacial** que **não deforma nada sozinho**: produz
um peso `w(ponto) ∈ [0, 1]` (`1` = força cheia, `0` = efeito removido) e **modula a FORÇA do
deformador SEGUINTE na pilha** de Live Path Effects (ADR-0132). Desacopla *"onde há influência"*
(o campo) de *"o que a influência modula"* (o efeito). Um Bulge que some nas bordas, uma onda alta
só no centro: um campo, todos os deformadores.

**É um `PathEffect` de primeira classe** (o nó que a pesquisa pediu), apendado à pilha existente —
a arquitetura já era o nó. Quatro formas analíticas: **Radial · Linear · Rect · Sweep** (as quatro
do Cavalry que não pedem uma segunda geometria; a 5ª, *forma arbitrária*, fica deferida — ver
Aberto).

## A espinha (não re-litigar)

- **O campo entra POR DENTRO do deformador**, não por um lerp na pilha. O Warp e o Zig Zag
  **reamostram** por arco (a saída não tem a contagem da entrada), então um lerp `input↔output` na
  pilha não teria com o que alinhar. O deformador avalia `w` na posição ORIGINAL de cada amostra e
  escala o deslocamento: `lerp(orig, deformado, w(orig))`. Em `w = 0` a amostra fica onde estava ⇒
  a região sem influência reconstrói a curva de entrada. É o "pluga na entrada de força do nó",
  literal. Pega **Bloat, Warp e Zig Zag**; Trim/Repeat não têm força que um campo module
  (`PathEffect::takes_falloff()` = false), e um Falloff acima deles é **inerte na geometria** — o
  painel diz isso.
- **`falloff = None` é byte-idêntico** ao que era antes do Falloff existir (todos os testes velhos
  dos deformadores passam com `None`).
- **Neutro = `amount == 0`** ⇒ Add não move um pixel e o `Cow::Borrowed` do `cooked()` sobrevive.
- Dois Falloffs antes do mesmo deformador **compõem por produto** (interseção das influências).

## Arquivos

**Motor (`crates/ph2d-vec-scene/`):**
- **`src/fx_falloff.rs`** (NOVO): `FalloffShape`, `FalloffSpec` (params/get/set/is_neutral por
  forma), `Falloff` (o campo composto) + o avaliador por-forma.
- **`src/fx_falloff_tests.rs`** (NOVO): 12 gates (campo por forma + modulação pela pilha REAL,
  mutação-provada).
- **`src/effect.rs`**: variant `PathEffect::Falloff` apendado; `is_neutral`/`label`/`as_falloff`/
  `as_falloff_mut`/`takes_falloff`/`from_kind`/`kind_index`/`KINDS` (+4) atualizados; `apply` ganhou
  `falloff: Option<&Falloff>`; `run_stack` acumula o campo pendente e o consome no próximo efeito.
- **`src/effect_params.rs`** (NOVO): `params`/`get`/`set` extraídos por **teto de LOC**
  (`effect.rs` 789→575). É `impl PathEffect` continuado.
- **`src/fx_warp.rs` · `fx_warp_presets.rs` · `fx_zigzag.rs`**: cada `*_contour` ganhou
  `falloff: Option<&Falloff>` (aditivo; `None` = byte-idêntico). Call sites de teste passam `None`.
- **`src/lib.rs`**: `pub mod fx_falloff;`.

**Painel (`crates/ph2d-panel-vector/`):** `FxRowView` ganhou `falloff_role: FalloffRole` (novo
enum); `paint_effects.rs` pinta a linha de dica ("modulates the effect below" / "add a deformer
below"); `state.rs`/`lib.rs` re-exportam `FalloffRole`; `tests/seam.rs` atualizado.

**Ponte (`shells/desktop/src/fx_bridge.rs`):** `stack_view` computa `falloff_role` (o próximo
efeito ligado não-Falloff; falloffs compõem).

**Smoke (`shells/desktop/src/falloff_smoke.rs` NOVO + `build_smoke.rs`/`main.rs`):**
`PH2D_BUILD_SMOKE=28`.

## ⚠️ Foundational tocado (pontos de extensão append-only, para o integrador conferir colisão)

- **`crates/ph2d-editor-core/src/ids/chrome/vector.rs`**: `MAX_FX_KINDS` **13→17** (o cap que o
  menu Add registra; gate exige `>= PathEffect::KINDS.len()`). **Nenhum NodeId const novo** — os
  ids de Add são FNV em runtime (`vector.fx.add.{k}`), então não há colisão de const a resolver.
- **`crates/ph2d-i18n/src/lib.rs`**: +2 chaves (`panel.vector.fx.falloff.modulates` /
  `.inert`). App é inglês-only; sem outra locale a sincronizar.

## Contratos congelados (§6) — INTOCADOS

`NodeOp=2`/`OpResolver=1`/`NodeManifest=8` **não tocados** (conferido por grep — o Falloff é dado
de `VecPath.effects`, não do `NodeManifest`). O contrato do doc vetorial (`VectorOp`…) **intocado**.
**ZERO bump de schema:** `PROJECT_SCHEMA` fica **29**, `VEC_SCENE_SCHEMA_VERSION` fica **13** — um
variant apendado a `PathEffect` não move os índices postcard dos anteriores (a receita "acrescentar
um efeito" do `effect.rs`).

## Gates (batched, todos VERDES)

- `ph2d-vec-scene`: **313 + 12 falloff** (mutação: tirar o `w` do `bloat_contour`/`warp_contour`
  → RED em 2 gates, provado e restaurado).
- `ph2d-panel-vector` (seam incluído), `ph2d-host-desktop` fx_bridge (19) + fx_bridge_dispatch (9),
  reachability/teto (effect_tests), `architecture_workspace_file_loc_cap`, `file_loc_caps` (shell),
  `architecture_panel_wiring_parity`, i18n, clippy `--all-targets` limpo nas crates tocadas,
  `cargo check --workspace` limpo.

## Smoke

```
env PH2D_BUILD_SMOKE=28 cargo run -p ph2d-host-desktop --release
```

Três retângulos, o **MESMO Zig Zag**, campos diferentes: **esquerda** = sem campo (uniforme,
controle) · **meio (selecionado)** = **Falloff Linear** (cristas em rampa; a seção Effects mostra o
card do Falloff com a dica; desligar o olho dele devolve as cristas uniformes) · **direita** =
**Falloff Radial** (cristas altas no miolo, planas nas pontas). Render-and-look já confirmou a rampa
do Linear; a modulação de Warp/Bloat está **provada nos gates** (não encenada no smoke).

⚠️ **Gotcha de captura (não é bug):** o app pode abrir uma cena-demo/projeto por cima; e instâncias
zumbis de runs anteriores atrapalham `spectacle -f`. Mate as instâncias antigas (`pkill ph2d-host-desk`)
antes de olhar.

## Aberto (deferido, com motivo)

- **Falloff de forma ARBITRÁRIA** (a 5ª do Cavalry): precisa de um path de referência (o gesto
  "Pick Path" do Pattern/Texto) + teste dentro/distância. É uma feature própria, não um branch a
  mais no `weight()`.
- **Falloff na CONTAGEM do Repeater** (por-cópia, não por-ponto): outra semântica; hoje Repeat não
  consome o campo (documentado).
- O smoke mostra a modulação em Zig Zag (as duas formas de campo mais legíveis); Warp/Bloat ficam
  por conta dos gates. Se o Enio quiser ver o Bulge modulado ao vivo, é um `=28` com uma 4ª forma.

---

# WAVE 2 — O TWIST (commit `afe16fce5`, pendente de smoke)

## O que é

O **remoinho** — o último membro da família de path-operators do AE (ZigZag/Roughen/Bloat/Warp já
existiam; era o único que faltava). Cada ponto gira em torno do centro por um ângulo que cresce com
a distância: centro parado, borda gira o ângulo inteiro, ponta enrola mais — o *pinwheel* do
*Distort & Transform > Twist* do Illustrator / *Twist* do AE. Um variant novo `PathEffect::Twist`,
um param só (**Angle**, `-360..360` graus).

## ⚠️ A cerca, e por que esta volta é legítima (LEIA antes de julgar)

Houve um Twist **CORTADO em 2026-07-18** (a cerca vive no cabeçalho de `crates/ph2d-vec-scene/src/
fx_warp.rs`): a 1ª versão mapeava só os pontos de controle (o *"lowpoly"*), e quatro tentativas com
uma subdivisão adaptativa **anterior** rasgavam sobre formas com quinas — veredito de defeito do
MODELO, e a cerca pedia *render-and-look, não um palpite* para reabrir.

**Legitimidade desta volta, por construção:**
1. **Esqueleto maduro.** Os presets de Warp (que POST-datam o corte) trouxeram um esqueleto de
   reamostragem densa por ARCO + união com âncoras + Catmull-Rom que os campos não-afins usam SEM
   rasgar. Extraí-o para a **porta única `fx_warp_presets::resample_displace`**; o Twist rida ele. O
   `warp_contour` ficou **byte-idêntico** (os 9 goldens do warp passam — é *pure code motion*).
2. **A sonda de olhar preenche em nonzero-winding** — uma silhueta auto-intersectada (o que um
   twist forte faz de propósito) aparece TINTA CHEIA, não rasgada; o artefato de even-odd que
   confundia o diagnóstico morreu.
3. **Render-and-look feito.** `tests/fx_look.rs` ganhou duas linhas — Twist num **QUADRADO** (o caso
   de falha documentado) e num círculo, 0→360°. O quadrado sai **pinwheel de tinta cheia** (não
   facetado, não rasgado, sem buracos); o círculo continua círculo. É o que a cerca mandou provar.
   Rodar: `PH2D_FX_LOOK_DIR=<dir> cargo test -p ph2d-vec-scene --test fx_look --release -- --ignored`.

## Arquivos (todos no MESMO conjunto da Wave 1 — integram juntas)

- **`src/fx_twist.rs`** (NOVO): `TwistSpec` (1 param) + `twist_contour` (rida `resample_displace`).
- **`src/fx_twist_tests.rs`** (NOVO): 4 gates mutação-provados.
- **`src/fx_warp_presets.rs`**: extraída a `resample_displace` (pub(crate)); `warp_contour`
  byte-idêntico; nota do cabeçalho do Twist atualizada (a cerca apontava "não entra aqui ainda").
- **`src/effect.rs`**: variant `Twist` apendado; `is_neutral`/`label`/`as_twist`/`as_twist_mut`/
  `takes_falloff`(=true)/`from_kind`/`kind_index`/`apply`/`KINDS`(+1)/`TWIST_KIND` atualizados.
- **`src/effect_params.rs`**: param **Angle** + `get`/`set`.
- **`src/lib.rs`**: `pub mod fx_twist;`. **`src/effect_tests.rs`**: `PANEL_MAX_FX_KINDS` 17→18.
- **`tests/fx_look.rs`**: as 2 linhas de render-and-look do Twist.
- **`crates/ph2d-editor-core/src/ids/chrome/vector.rs`**: `MAX_FX_KINDS` **17→18** (append-only).
- **`shells/desktop/src/twist_smoke.rs`** (NOVO) + `build_smoke.rs`/`main.rs`: `PH2D_BUILD_SMOKE=29`.

## Contratos / schema — INTOCADOS

`NodeOp`/`OpResolver`/`NodeManifest` intactos (Twist é dado de `VecPath.effects`). **ZERO bump:**
variant apendado não move os índices postcard (`PROJECT_SCHEMA` **fica 29**, `VEC_SCENE` **fica 13**).

## Gates (todos VERDES)

- `ph2d-vec-scene`: **317 lib** (313 + 4 Twist) + reachability/neutral/round-trip/ceiling
  auto-cobrem o Twist. Mutações: identidade → RED nos 2 gates de rotação; escala → RED no de
  cisalhamento (provado e restaurado).
- warp goldens (9, byte-identidade da extração), panel-vector (70), shell fx_bridge(19)+dispatch(9),
  `file_loc_caps`+workspace LOC-cap, clippy `--all-targets` limpo, `cargo check --workspace` limpo.

## Smoke

```
env PH2D_BUILD_SMOKE=29 cargo run -p ph2d-host-desktop --release
```

Três quadrados: **esquerda** Twist 90° (remoinho suave) · **meio (selecionado)** Twist 200° (o card
Twist com o slider **Angle** na seção Effects — arraste e veja o giro apertar) · **direita** o MESMO
Twist 200° precedido de um **Falloff Radial** (só o miolo gira, as quinas ficam) — o campo da Wave 1
modulando o giro da Wave 2. Desligar o olho do Falloff devolve o remoinho cheio.

## Aberto (deferido, com motivo)

- **Um dropdown de forma para o Twist não faz sentido** (é 1 KIND, sem família — ao contrário de
  Warp/Falloff). Um `center` autorável (offset do pivô, como o AE) seria um 2º param — só se pedido.
- **Além de ~270° numa forma de quinas afiadas o twist fica extremo** (509° no canto a 360°) — é a
  natureza de um twist forte, não um defeito; o slider vai a ±360 (uma volta), e a fidelidade da
  reamostragem (`SAMPLES=128`) foi o recurso que definiu o teto, não um palpite.
