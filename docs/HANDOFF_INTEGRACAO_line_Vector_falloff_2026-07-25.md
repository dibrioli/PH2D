# HANDOFF DE INTEGRAÇÃO — `line/Vector` · FALLOFF + TWIST + KNOT (2026-07-25)

**Estado:** linha FECHADA para TRÊS waves da mesma sessão, aguardando ordem EXPLÍCITA de
integração. **NÃO integrei nem pushei** (§0.7 / [[feedback_integration_only_enio_command_end_of_all_lines]]).

- **Wave 1 — o FALLOFF** (commit **`772f830eb`**): **smoke APROVADO pelo Enio.** Detalhe abaixo.
- **Wave 2 — o TWIST** (commit **`afe16fce5`**): **smoke APROVADO pelo Enio** (`PH2D_BUILD_SMOKE=29`).
  Seção própria mais abaixo.
- **Wave 3 — o KNOT** (commit **`d38cae8f2`**): **pendente de smoke** (`PH2D_BUILD_SMOKE=30`). Seção
  própria no fim deste documento.

> Nenhuma wave move o schema (variants apendados, `PROJECT_SCHEMA` fica **29**). As três tocam o
> MESMO conjunto de arquivos (`ph2d-vec-scene/src/effect*.rs`, `fx_*`, `MAX_FX_KINDS`), então
> integram JUNTAS. Estado no tip: `MAX_FX_KINDS` **19** (13 base+warp, +4 falloff, +Twist, +Knot);
> splits de LOC: `effect_params.rs` (params) + `effect_accessors.rs` (as_*+label).

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

---

# WAVE 3 — O KNOT (commit `d38cae8f2`, pendente de smoke)

## O que é

O **entrelace celta** — onde o caminho se cruza (auto-interseção OU entre contornos), a fita de
BAIXO ganha um VÃO e a de CIMA passa inteira. É o *Knot* LPE do Inkscape, o item "médio, e é lindo"
da pesquisa `20_*`. Um variant novo `PathEffect::Knot`, dois params: **Gap** (a espessura aparente,
% da forma) e **Swap** (quem passa por cima).

## A espinha (não re-litigar)

- **Detecta na POLIGONAL, corta na CURVA.** As travessias saem da poligonal densa (interseção
  reta-reta com a posição de arco de cada passagem); o vão é cortado na Bézier pela MESMA máquina de
  arco do Trim. Extraí `pieces_between`/`rebuild` do `fx_trim` para `pub(crate)` (**porta única**:
  Trim revela UM intervalo, Knot corta o COMPLEMENTO de vários) — o `fx_trim` fica **byte-idêntico**
  (goldens intactos).
- **Alternância = parece tecido.** `over` vira a cada ponta de cruzamento em ordem de arco (seguindo
  uma fita ela alterna cima/baixo — o "nó sem fim"). Garante **exatamente um vão por travessia**
  (nunca dois, que apagaria a fita; nunca zero, que a deixaria sólida); empate de paridade → mergulha
  a passagem de arco maior. `Swap` inverte todos.
- **Sem z-buffer:** as duas fitas têm o mesmo traço e a de cima tem tinta onde a de baixo tem o vão —
  o vão É a sombra (o método do Inkscape).
- **Whole-path** (como o Repeater, NÃO `apply_per_contour`): uma travessia pode ser entre dois
  contornos. Caminho sem travessia sai clonado (nada a tecer). `takes_falloff = false`.

## Arquivos (mesmo conjunto das waves 1-2 — integram juntas)

- **`src/fx_knot.rs`** (NOVO): `KnotSpec` + `knot_path` (detecção + alternância + corte).
- **`src/fx_knot_tests.rs`** (NOVO): 4 gates mutação-provados (pentagrama).
- **`src/fx_trim.rs`**: `Piece`/`pieces_between`/`rebuild` → `pub(crate)` (byte-idêntico).
- **`src/effect.rs`**: variant `Knot` + `is_neutral`/`takes_falloff`(=false)/`from_kind`/`kind_index`/
  `apply`(whole-path)/`KINDS`(+1)/`KNOT_KIND`. ⚠️ **Split de LOC:** os acessores `as_*`+`label` saíram
  para **`src/effect_accessors.rs`** (NOVO, irmão do `effect_params.rs`) — effect.rs 705→510.
- **`src/effect_accessors.rs`** (NOVO): `impl PathEffect` com os `as_*` (agora incluindo `as_knot`).
- **`src/effect_params.rs`**: params **Gap**+**Swap** + `get`/`set`.
- **`src/lib.rs`**: `pub mod fx_knot;`. **`src/effect_tests.rs`**: `PANEL_MAX_FX_KINDS` 18→19.
- **`tests/fx_look.rs`**: pentagrama + 2 linhas de render-and-look (Knot + Swap); ⚠️ as linhas de
  Twist passaram de `KINDS.len()-1` para `-2` (Knot é agora o último KIND).
- **`crates/ph2d-editor-core/src/ids/chrome/vector.rs`**: `MAX_FX_KINDS` **18→19** (append-only).
- **`shells/desktop/src/knot_smoke.rs`** (NOVO) + `build_smoke.rs`/`main.rs`: `PH2D_BUILD_SMOKE=30`.

## Contratos / schema — INTOCADOS

`NodeOp`/`OpResolver`/`NodeManifest` intactos. **ZERO bump** (`PROJECT_SCHEMA` fica **29**,
`VEC_SCENE` **13**).

## Gates (todos VERDES)

- `ph2d-vec-scene`: **321 lib** (317 + 4 Knot) + reachability/neutral/round-trip/ceiling auto-cobrem
  o Knot. Mutações: não-cortar-vão → RED em one-gap+swap; ignorar-swap → RED só no swap.
- fx_trim goldens (byte-identidade do `pub(crate)`), panel-vector, shell fx_bridge(19)+dispatch(9),
  `architecture_workspace_file_loc_cap`+`file_loc_caps`, clippy `--all-targets` limpo, `cargo check
  --workspace` limpo.

## Render-and-look (a folha de contacto)

`PH2D_FX_LOOK_DIR=<dir> cargo test -p ph2d-vec-scene --test fx_look --release -- --ignored` — as
linhas do Knot (pentagrama) mostram as 5 travessias abrindo vão ALTERNADO, e o Swap pondo o vão na
outra fita. (A folha preenche as fitas abertas — ruído do desenhador; o smoke usa **traço**, sem
fill, e lê como fita limpa.)

## Smoke

```
env PH2D_BUILD_SMOKE=30 cargo run -p ph2d-host-desktop --release
```

Três pentagramas traçados como FITA: **esquerda** Gap 6% · **meio (selecionado)** Gap 10% (card do
Knot com Gap + Swap na seção Effects; seguindo uma fita ela alterna cima/baixo) · **direita** Gap 10%
+ **Swap** (inverte quem passa por cima em toda travessia).

## Aberto (deferido, com motivo)

- **Toggle por-travessia** (o clique-para-inverter do Inkscape): hoje só o `Swap` global. Precisa de
  um gesto de canvas que aponte a travessia — é uma feature própria, não um branch no `dive_gaps`.
- **A alternância é por PARIDADE de arco** — perfeita no nó alternante (pentagrama, (5,2) torus); numa
  projeção não-alternante a regra garante um vão por travessia mas não a alternância ideal. O toggle
  por-travessia é o escape (item acima).
- **Detecção na poligonal** (`SAMPLES_PER_SEG=16`): a posição do vão tem precisão ~1/16 de segmento;
  o vão tem largura, então não aparece. Curva↔curva exata (Bézier-Bézier, até 9 raízes) seria um
  motor próprio, não justificado.
