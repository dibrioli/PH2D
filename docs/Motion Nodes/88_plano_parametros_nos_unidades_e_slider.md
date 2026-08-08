# Doc 88 — PLANO: os parâmetros dos nós ganham unidades, slider dual e o conjunto PRO

> **Handoff / super-prompt** para o agente que assume `line/motion-value` numa janela nova.
> Escrito 2026-08-05 pelo agente anterior (que acabou de fechar a wave "acelera", HEAD `ecb5232f2`).
> Ordem do Enio: *"melhorar os parâmetros dos nós e como são desenhados no painel"* — com
> **unidades** (Pixel/Metros/Graus), **slider dual** (o slider trava num range, a caixa de texto vai
> muito além, com Clamp), e **cada nó com o conjunto de params do essencial ao estado-da-arte**.
> Os dois exemplos que o Enio citou como mal-feitos: **LookAt** (sem alvo por nome/mouse) e
> **Color Array** (13 sliders crus, sem swatch).

---

## FASE 0 — ONDE VOCÊ ESTÁ (execute ANTES de ler qualquer código; MODELO_TROCA_DE_AGENTE)

Você começa na RAIZ do repo (=`main`). Os MESMOS paths existem aqui e na sua worktree —
editar a árvore errada compila e commita sem erro, e só aparece na integração.

```
cd Worktrees/line-motion-value && pwd && git branch --show-current
   -> pwd TEM de terminar em /Worktrees/line-motion-value
   -> a branch TEM de ser line/motion-value
git log --oneline -3    # o topo deve ser ecb5232f2 (a wave "acelera", ja commitada)
git rebase main         # obrigatorio no inicio da jornada; conflito FORA dos seus
                        # arquivos = colisao de mesmo-simbolo -> PARE e reporte ao Enio
cargo check -p ph2d-node-registry   # a base nova nao te quebrou (1o build pode ser frio)
```

⚠️ **Modo L, sempre:** todo comando de Bash começa com o `cd` da worktree — a cwd escorrega
para o primário ([[feedback_bash_cwd_resets_and_slips_to_the_primary]]). Você **NÃO integra nem
pusha** — fecha a wave, escreve o handoff de integração, e PARA (§0.7 do CLAUDE.md). Commit local
em fast mode é `git commit --no-verify -- <seus paths>` (nunca `-A`).

## FASE 2 — LEIA, nesta ordem, antes de tocar em código

1. **CLAUDE.md §0 (medir antes de limitar), §5 (Motion Nodes — o padrão side-metadata/text-param
   citado 5×), §6 (contrato CONGELADO).** É o núcleo.
2. **A pesquisa de estado-da-arte JÁ FEITA** (você NÃO recomeça):
   - `docs/Motion Nodes/referencia_pesquisa_cavalry.md` §B — os **conjuntos de params PRO** dos
     nós-chave do Cavalry (Duplicator, Falloff, Oscillator, Noise, Sound, Text, Look At, Stagger…).
   - `docs/Motion Nodes/referencia_catalogo_nodes_minicavalry.md` — o catálogo do nosso vocabulário
     (params, portas, gotchas, combos).
   - `docs/Motion Nodes/85_gradient_editor_nota_adr.md` — o editor de gradiente multi-stop (o
     primitivo de UI que o Color Array vai reusar).
3. **Este doc, §§1–6 abaixo** — a superfície exata (file:line) e o desenho.
4. **DIRETIVA_IMPLEMENTACAO.md** — releia a cada passo, como ela manda.

---

## §1 — A LEI que não se toca (contrato §6) e a que se estende

**CONGELADO** (`NodeManifest=8`/`NodeOp=2`/`OpResolver=1` — gate `architecture_contract_surface`):
- `crates/ph2d-nodegraph/src/node.rs:52` — `struct ParamSpec { name: &'static str, default: f32 }`.
  **Só nome + default f32.** Range, step, rótulo, widget, unidade **NÃO** moram aqui.
- `node.rs:101` — os 8 campos do `NodeManifest`. `node.rs:141` — `trait NodeOp`.

**A superfície que você ESTENDE** — tudo side-metadata do `NodeRegistry`, cada mapa com **default
vazio** (nó sem entrada = neutro). O mecanismo está em `crates/ph2d-node-registry/src/lib.rs:31`
(a struct) e `:336` (o `KernelResolver`, onde os canais "nascem" via `unwrap_or(&[])`):
- `ParamUiHint { param, label, min, max, step, widget }` — `ui.rs:174`. O `max` é o teto **SOFT**
  (arrasto do slider). Registrado por `register_param_ui(id, &[ParamUiHint])`.
- `ParamHardMax { param, max }` — `ui.rs:200`. O teto **HARD** (valor DIGITADO). Tabela SEPARADA
  (não campo do hint) **por razão mecânica: o `ParamUiHint` é literal `&'static` em ~275 sítios**,
  então acrescentar um campo a ele obriga editar os 275. Registrado por `register_param_hard_max`.
- `ParamGate { param, when, values }` — `ui.rs:215`. Visibilidade condicional.
- **Non-f32** (curva/gradiente/texto/nome/paleta): **NÃO** toque o manifesto — use `Graph::set_text_param`
  (`graph.rs:335`) + `EvalCtx::text_param` (`cook.rs:217`), serializando para String. É o canal
  canônico, com record `x`/header `v2` no formato textual (`format.rs`).

**A regra de ouro do padrão:** para um dado novo (uma unidade, um hard-min), **adicione uma TABELA
side-metadata paralela** (o molde do `ParamHardMax`), não um campo no `ParamUiHint`.

## §2 — Como o painel desenha um param hoje (a costura painel↔shell)

- **A ponte manifesto+hint → linhas** (no shell): `shells/desktop/src/render_loop/motion_bridge_params.rs:260`
  `build_params_snapshot`. Emite `ParamRow`s. TEXT params (Text/Curve/Gradient/Channels/Source)
  saem ANTES do loop do manifesto (`:338`); os `ParamSpec` viram Scalar/Color/Toggle/Enum/Angle/Seed.
- **Os tipos do canal:** `crates/ph2d-panel-motion-params/src/snapshot.rs:23` `enum ParamRow`;
  `:186` `struct ScalarRow { name, label, value, min, max, hard_max, step, integer, driven }` —
  **é aqui que soft (`max`) e hard (`hard_max`) chegam ao painel**; `:245` `enum MotionParamIntent`
  (o retorno de um edit: `SetParam{value:f64}` / `SetTextParam{value:String}`).
- **O desenho por widget:** `crates/ph2d-panel-motion-params/src/rows_paint.rs:36` `paint_rows`
  (um braço por variante). O **slider numérico**: `rows_paint_kinds.rs:60` `paint_scalar_row`
  (`span = max-min`, `paint_slider_with_chip_layout_adaptive`).
- **A fiação dos DOIS ranges** (o seam load-bearing) — `crates/ph2d-panel-motion-params/src/lib.rs`:
  - `:424` `store.set_number_range(chip_id, row.min, row.hard_max, row.step)` — **a caixa clampa em
    `[min, HARD_max]`**.
  - `:433` `store.link_slider_number_mapped(slider_id, chip_id, span, row.min)` — **o slider mapeia
    `0..1` sobre `[min, max]` (SOFT)**.
  - `:315` `normalized_track` / `:322` `row_value` — o afim `0..1 ↔ valor`.
- **`ParamWidget` (todas as variantes)** — `crates/ph2d-node-registry/src/ui.rs:77`:
  `Slider · IntSlider · Angle` (guarda **GRAUS**, chip `deg`) `· Toggle · Seed · Color{channels} ·
  Enum{labels} · Channels{...} · Source · Text · Curve · Gradient`.

## §3 — Os widgets de UI que você reusa (já prontos, editor-core)

- **Slider normalizado** (0..1, sem min/max próprio): `crates/ph2d-editor-core/src/widget/slider.rs:32`.
- **Caixa numérica com range + clamp**: `widget/number_input.rs:19` (`min/max: Option<f64>`, `clamp()`).
- **Composto** (label+slider+chip): `widget/slider_with_chip.rs` (`paint_slider_with_chip*`).
- ⚠️ **O widget de unidade JÁ EXISTE** — `widget/numeric_input_with_unit.rs:20`:
  `enum Unit { Px, Meters, Degrees, Radians, Percent }` com `suffix()` e `parse("90deg") -> (90.0, Some(Degrees))`.
  Construído para o Sprite Inspector; **é o que a Wave A liga aos params de Motion**.
- **Cor:** `BlenderColorPicker` (OKLCH, `crates/ph2d-editor-core/src/widget/blender_color_picker/`),
  `store.register_picker_swatch(id)` (`store_hierarchy.rs:263`), `store.blender_picker(id)` (read-back,
  `blender_ops.rs:103`). A máquina de read-back de Motion: `render_loop/motion_bridge_color.rs`
  (`seed_color_swatches`, `picker_session` de undo, `apply_color_to_node`).
- **Gradiente multi-stop:** `crates/ph2d-panel-motion-params/src/gradient_row.rs` (barra + markers
  arrastáveis + swatch OKLCH por stop + presets), sobre `ph2d_color::ColorRamp` em text param.
- **Curva:** `crates/ph2d-panel-motion-params/src/curve_row.rs` (canvas + handles), sobre
  `ph2d_curve::Curve` em text param. Ambos usam o primitivo foundational `InteractiveState::CurvePoint`.

## §4 — A infra de UNIDADES hoje (e o que falta)

- **`pixels_per_meter`** existe: `crates/ph2d-editor-core/src/project.rs:23` (`DEFAULT=100, MIN=1,
  MAX=4096`), `ProjectSettings.pixels_per_meter` + `enum DisplayUnit { Meters, Pixels }` com
  `from_meters`/`to_meters`/`suffix()` (`:50`). ⚠️ **Os nós de Motion NÃO o leem** — ele vive só em
  editor-core/shell (física, import, grid-snap). **É a ÚNICA constante px↔m; reuse-a (porta única).**
- **Graus↔radianos:** a timeline guarda radianos (`ph2d-timeline PropKind::Rotation`); o app **autora
  em GRAUS** (`ui.rs:82`: *"o param guarda GRAUS"*); a conversão deg→rad mora **DENTRO do nó**, na
  borda do cook (ex.: `crates/ph2d-node-motion-rotate/src/lib.rs:2,57`). **Graus já funciona** via
  `ParamWidget::Angle`.
- ⚠️ **NÃO existe conceito de "unidade" num param hoje** (grep confirmado). `gap_x`/`gap_y` do grid são
  "world units" crus, sem tag px/m. O `enum Unit` mora só na camada de widget, não na metadata do nó.

## §5 — O ESTADO dos dois nós que o Enio chamou de mal-feitos

**LookAt** (`crates/ph2d-node-motion-look-at/src/lib.rs`, id `motion.look_at`):
- Hoje: **um** param `offset` (graus, ±180) + dois inputs numéricos `target_x`/`target_y`. Computa
  `atan2(target - P) * RAD_TO_DEG + offset` → coluna `rot` (kernel CPU+GPU, transcendental-free).
- ⚠️ **Não tem alvo por nome nem por mouse.** O pipe por-nome EXISTE (external channel
  `EvalCtx::external`/`Cook::set_external` em `external.rs`/`cook.rs:123`; `stable_name_id` FNV do
  `Name`; o picker `ParamWidget::Source` do `source.object`), MAS a membrana de objetos publica só a
  **aparência no `[0,0]`** (`render_loop/motion_bridge_objects.rs:152`, decisão #1 do módulo) —
  **nenhum nó puxa a POSIÇÃO de uma entidade nomeada**. Prova de que dá: o `motion.path` já publica
  coords de mundo reais (external `"Track"`, `motion_bridge_shapes.rs:145`). E **nenhum external de
  mouse existe** (só há `live_cursor_in_window` para drops de arquivo, não fiado ao cook).

**Color Array** (`crates/ph2d-node-motion-color-array/src/lib.rs`, id `motion.color_array`):
- Hoje: `colors` (2..4) + **12 canais R/G/B crus como Sliders** (4 slots × RGB). Sem swatch, sem
  picker, sem "adicionar cor". É o "trabalho de merda" que o Enio nomeou.
- O irmão `motion.color_ramp` já migrou o gradiente inteiro para text param + `ParamWidget::Gradient`.

---

## §6 — O DESENHO (o que construir, em ondas)

### ⚠️ Antes: PLANO primeiro, código depois (pd-feature)
Entregue ao Enio, ANTES de escrever código: (1) a curadoria do estado-da-arte a partir das
referências do §Fase-2 (não recomece a pesquisa); (2) o desenho com a **porta ÚNICA** de cada
pergunta; (3) a prova por grep de que não encosta no §6/schema; (4) as 4 condições de UI
(existe · pintado+registrado · clique chega ao barramento · a sequência leva a algum lugar);
(5) os gates red-first; (6) a cena de smoke com números MEDIDOS (sonda headless ANTES da mensagem).

### ⚠️ O Enio pediu FAN-OUT: "levante vários agentes, cada um numa abordagem"
Isto vale para o **DESENHO da Wave A** (unidades + slider dual + toggle de grafo) — é a parte de
espaço-de-solução largo. Use o padrão judge-panel do pd-feature: N abordagens de design
independentes, cada uma um agente, pontuadas por juízes, e sintetize a vencedora enxertando o
melhor das demais. **A varredura por-nó (Wave B+) é mais curadoria que design** — menos fan-out.

---

### WAVE A — A FUNDAÇÃO (aterrissa PRIMEIRO; todo nó depende dela)

**A1 — O slider dual (POPULAR, não construir).** O mecanismo **já existe**: `ParamUiHint.max` =
soft (slider), `ParamHardMax.max` = hard (caixa), o clamp está no `NumberInput`. Trabalho:
1. **Simetrizar o piso:** hoje só há `hard_max`; a caixa não desce abaixo de `row.min` (o min do
   slider). Para "digitar 0.001 onde o slider começa em 0.01" ou um negativo grande, adicione um
   `ParamHardMin` (tabela paralela, molde do `ParamHardMax`) + `ScalarRow.hard_min`, e troque a
   fiação para `store.set_number_range(chip, row.hard_min, row.hard_max, row.step)`.
2. **Popular `ParamHardMin`/`ParamHardMax` por nó** (a varredura). Ex.: grid `rows`/`cols` soft 20 /
   hard 200 (`crates/ph2d-node-motion-grid/src/lib.rs:210`).
   - ⚠️ **§0: o teto HARD é onde o "disfuncional" começa — MEÇA, não chute 200.** A regra-default
     pode ser "hard = soft × 10", MAS onde o número tem recurso (contagem de instâncias que
     congela — a wave "acelera" acabou de medir: LOD tiles acima de 16k, crisp abaixo), o hard é o
     **número medido**, com a tabela ao lado. Um clamp que só diz "por segurança" é palpite.

**A2 — Unidades por param (px / m / deg / unitless / count).**
1. **A tabela `ParamUnit { param, unit }`** (side-metadata, molde do `ParamHardMax` — orthogonal ao
   widget: um Slider pode ser px, m, ou adimensional). Reuse o `enum Unit` de
   `numeric_input_with_unit.rs:20`. **Não** ponha unidade dentro do `ParamUiHint` (275 sítios).
   - Alternativa a pesar no fan-out: uma variante `ParamWidget::Length` em vez da tabela. A tabela é
     a recomendação (a unidade é ortogonal ao tipo de widget).
2. **A conversão display↔store é UMA porta** — [[feedback_derived_coordinate_seed_must_match_sample]]:
   o paint LÊ por ela (mostra "px" ou "m"), o commit PARSEIA por ela (o `parse` de
   `numeric_input_with_unit.rs` já faz o inverso). Duas cópias divergem em silêncio.
3. ⚠️ **O valor STORE é canônico e NÃO muda** — só o display/parse converte. Decisão a NOMEAR: o
   canônico de um param "length" é o número como está hoje (world units crus), e o display
   multiplica/divide por `pixels_per_meter` para mostrar m vs px. **Assim a cor cozida é
   byte-idêntica** (sem churn de schema/fingerprint) — prove por gate.
4. **Graus já está pronto** (`ParamWidget::Angle`); estenda o padrão de "unidade na fronteira" para
   length. Radianos/turns são detalhe de implementação de quem consome.

**A3 — O toggle pixel↔metros no GRAFO** (o botão que o Enio pediu).
1. Um botão no chrome do painel do grafo. Ele flipa o `DisplayUnit` de TODOS os params de length do
   grafo. **Não muda valores armazenados** — só o display/parse (via A2).
2. **Estado:** se deve sobreviver ao save → um record **append-only** novo no `MotionDoc` (o gêmeo
   exato do `z`/`base_z`, um escalar de doc emitido numa seção do `to_text`; header bump; **byte-
   idêntico para docs que nunca o usam**, a política dos records `x`/`y`/`yg` —
   `crates/ph2d-motion-doc/src/lib.rs:71,108`). Se transiente → o `ViewState` do painel
   (`crates/ph2d-panel-motion-graph/src/state.rs:109`). **Recomendação: persistido** (o artista
   escolhe a unidade de trabalho e ela deve sobreviver).
3. ⚠️ **Reuse `ProjectSettings::pixels_per_meter`** — a ÚNICA constante px↔m. Um 2º ppm diverge.

**Gates/smoke da Wave A** (red-first): o grid arrasta o slider a 20 mas a caixa aceita 200 e clampa
acima; um param length mostra "px" e vira "m" com o toggle do grafo, e o **cook é byte-idêntico**
(fingerprint/gate de igualdade); digitar `0.001` funciona onde o slider min é `0.01`. A cena de smoke
imprime o que montou.

---

### WAVE B+ — A VARREDURA por-nó (o conjunto PRO; começa pelos dois exemplos)

**B1 — Color Array → editor de PALETA** (baixo risco, quase toda fiação do que existe):
- Curto prazo (4 slots fixos): trocar os 12 Sliders por grupos `ParamWidget::Color{channels}` (o
  painel já os renderiza como swatch OKLCH; a máquina de read-back `motion_bridge_color.rs` já
  existe).
- Estado-da-arte (N cores + botão "adicionar"): o modelo fixo de 4 slots não escala → **text param**,
  reusando a maquinaria de stops do `gradient_row.rs` **MENOS as posições** (uma paleta é um gradiente
  sem eixo). Swatch OKLCH por cor, `+`/`−` add/remove.

**B2 — LookAt → alvo de verdade** (aqui está o trabalho REAL, os dois canais novos):
- O alvo vira um seletor: **objeto por NOME** (⇒ publicar a posição-de-mundo do objeto nomeado —
  external novo, ex. `pos:<name>`, estendendo a membrana `motion_bridge_objects.rs`; o picker
  `ParamWidget::Source` já existe) · **o MOUSE** (⇒ um external de mouse-world, que **não existe** —
  construa-o, no molde do `motion.path` que já publica coords) · ponto fixo · coordenada fiada.
- Params PRO (do Cavalry Look At + estado-da-arte): eixo, `offset` (já tem), **damping/spring**
  (seguimento suave), up-vector, min/max de ângulo. Curados de `referencia_pesquisa_cavalry.md`.

**B3 — A varredura do catálogo + o passe VISUAL do painel:**
- Nó a nó (ou família a família), curar o conjunto PRO de `referencia_pesquisa_cavalry.md` §B +
  `referencia_catalogo_nodes_minicavalry.md`, **cada família um smoke**
  ([[feedback_final_product_every_node_ships_the_full_pro_param_set]] — o superset, conferido por nó).
- O passe visual: tornar o painel `motion-params` "óbvio, bonito, rápido, intuitivo" (o pedido do
  Enio). Reuse os widgets do §3; zero hex/f32-literal/string hardcoded (HR-15, tokens/i18n).

---

## §7 — Os inegociáveis desta linha (resumo)

- **§6 congelado:** `NodeManifest=8`/`NodeOp=2`/`OpResolver=1`. Nada novo no manifesto — tudo é
  side-metadata do registry (molde `ParamHardMax`/`reduces`/`luts`) ou text param. **Prove por grep**
  antes de fechar.
- **§0 medir antes de limitar:** todo teto (hard_max, faixa de count) traz a MEDIÇÃO e a tabela.
- **Porta única** em cada conversão (display↔store, soft↔hard, nome↔posição).
- **Modo L:** você não integra/pusha; commit local em fast mode escopado (`-- <paths>`, nunca `-A`);
  todo Bash com o `cd` da worktree. Fecha a wave → handoff de integração → PARA.
- **UI canônica (HR-15):** tokens/i18n, zero hex/f32-literal/string hardcoded.
- **As 4 condições de UI** por feature: existe · pintado+registrado · clique no barramento · a
  sequência leva a algum lugar.

## §8 — Índice rápido dos sítios de edição
- Contrato: `ph2d-nodegraph/src/node.rs:52,101,141`.
- Side-metadata: `ph2d-node-registry/src/ui.rs:77,174,200,215`; `lib.rs:31,164-331,336`.
- Text param: `ph2d-nodegraph/src/graph.rs:335`; `cook.rs:217`; `format.rs` (record `x`).
- Painel/costura: `ph2d-panel-motion-params/src/{snapshot.rs:23,186; rows_paint.rs:36;
  rows_paint_kinds.rs:60; lib.rs:315,322,424,433; gradient_row.rs; curve_row.rs}`;
  `shells/desktop/src/render_loop/motion_bridge_params.rs:260,524-589`.
- Widgets: `ph2d-editor-core/src/widget/{slider.rs:32; number_input.rs:19; slider_with_chip.rs;
  numeric_input_with_unit.rs:20; blender_color_picker/}`.
- Unidades: `ph2d-editor-core/src/project.rs:23,50,110`.
- Grid (exemplo de hints): `ph2d-node-motion-grid/src/lib.rs:42,210`.
- LookAt: `ph2d-node-motion-look-at/src/lib.rs:42,200`. Color Array:
  `ph2d-node-motion-color-array/src/lib.rs:30,177`. Membrana de objetos:
  `shells/desktop/src/render_loop/motion_bridge_objects.rs:135,152`; `motion_bridge_color.rs`.
- Doc/view state para o toggle: `ph2d-motion-doc/src/lib.rs:71,108`;
  `ph2d-panel-motion-graph/src/state.rs:109`; `ph2d-nodegraph/src/format.rs` (records append-only).

---

## §9 — B3: O CENSO, e o mapa da varredura (escrito 2026-08-08, com números MEDIDOS)

> A §6 dizia *"nó a nó, ou família a família"*. A §0 do CLAUDE.md manda **medir antes de
> decidir** — então a varredura começa por um censo executável, não por ordem alfabética.

**A sonda:** `cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture`
(roda na crate que registra os 118 nós — o build mais barato que os enxerga; uma sonda na
shell mediria o mesmo e custaria o app inteiro).

**O retrato de 2026-08-08:** **118 nós · 411 params · 395 com hint · 105 com unidade.**
Só **um** nó tem param sem hint nenhum (`value.attribute`, cujo controle real é o picker
de canais — o `f32` ali é o `mode` que o picker escreve). **Cinquenta** nós têm ≤ 2
controles, e é aqui que a curadoria mora: *magro por natureza* e *magro por omissão* são
coisas diferentes, e o censo não sabe distingui-las — quem distingue é a referência.

### O mapa, com o veredito de cada família

| Família | Estado MEDIDO | Veredito |
|---|---|---|
| **TRANSFORM** (`move` · `rotate` · `scale` · `transform` · `mirror` · `orbit`) | `scale` escrevia uma coluna **Vec2** a partir de **um** número; `mirror` pregava a linha no **centroide** | ✅ **FECHADA** (2026-08-08) — o 2º eixo e o `Axis Offset`, smoke `PH2D_TRANSFORM_SMOKE=1` |
| **ECHO** (`motion.trail`) | 3 params (`length`/`fade`/`shrink`) contra os 8 da referência | **ABERTA** — faltam `spacing` (um fantasma a cada N ticks) e `hue_shift` (a cauda colorida que o próprio caso de uso do catálogo nomeia). ⚠️ `spacing` precisa de um contador de ticks, e o nó é sequencial: mede-se primeiro onde o contador mora |
| **DEFORMERS** (`bend` · `twist` · `spherize` · `four_point_warp` · `lattice` · `kaleidoscope` · `slit_scan`) | `spherize` 1 · `slit_scan` 1 · o resto 3-8 | **ABERTA, a medir** — os dois magros podem ser magros por natureza |
| **VALUE** (24 nós) | quase todos 1-2 params | ⛔ **RECUSADA COM MOTIVO** — um `value.unary` é *um verbo sobre um número*; um param a mais ali é cerimônia, e a §12 (domínio de valor) desenhou essa família justamente para ser pequena e componível |
| **ESTRUTURAIS** (`util.reroute*` · `motion.combine` · `integrate` · `output` · `luminance` · `make_point` · `morph` · `sim.zone` · `value.switch` · `pulse.sample_hold`) | 0 params | ⛔ **RECUSADA** — zero params é o contrato deles, não uma lacuna |
| **RIG** (`fk` · `ik_2bone` · `fabrik` · `rubber_hose` · `skin_deformer` · `skeleton`) | magros | ⏸ **ADIADA** — o CLAUDE.md §5 defere rig+skinning "pro FIM de tudo" |

### ⚠️ O `motion.duplicator` tem ZERO params, e isso está quase todo CERTO

A tabela §B da pesquisa do Cavalry é grande (Distribution · Shape Position/Rotation/Scale
· Visibility/Opacity · Auto Id / Shape Id · Shape Time Offset · Use Index Context · Skip
Invisible), e a leitura ingênua é *"faltam sete params"*. **Ela está errada**, e o motivo
é arquitetural: o Duplicator do Cavalry é um **mega-nó** e o nosso é um **grafo**.

- *Distribution* — lá é um dropdown de 21 tipos; aqui são **21 nós** que alimentam a porta
  `points`. Trazê-la para dentro seriam duas portas para a mesma pergunta.
- *Shape Position/Rotation/Scale/Opacity por-cópia* — lá são alvos que Falloff/Stagger
  modulam dentro do nó; aqui são `motion.move`/`rotate`/`scale` **a jusante**, que é o que
  torna o falloff composável.
- *Skip Invisible Duplicates* — é o `motion.cull`.

**O gap REAL é um só:** *que forma vai em que ponto*. Nós fazemos o **produto cartesiano**
(N formas × M pontos = N·M cópias) e a referência tem `Auto Id` (cicla) / `Shape Id`
(fixa). Com 3 formas e 100 pontos, *"cem pontos, cada um recebendo uma das três formas"*
**não é exprimível hoje**. Isso é semântica de contagem, não um knob: ⇒ **wave própria**.

### A lei desta varredura, em uma linha

Um param novo entra quando a coluna que o nó escreve, ou a referência, **nomeia uma
capacidade que hoje é inexprimível no grafo inteiro** — nunca porque o nó parece pequeno.
E todo default novo **reduz LITERALMENTE à expressão anterior**, porque a arte já autorada
é o que uma varredura de parâmetros mais facilmente destrói.
