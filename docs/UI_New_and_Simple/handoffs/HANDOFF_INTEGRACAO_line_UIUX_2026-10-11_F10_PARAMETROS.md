# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-11 — F10: MAIS PARÂMETROS, O CARTÃO LATERAL, AS ALÇAS NO MODO FORMA

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F9, 11/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F9_ALCAS_DAS_FORMAS.md) — também
> NÃO integrada: esta linha leva a F9 + a F10 (F0–F8 já estão no `main`). Lei, oráculos, decisões e
> recusas: [`15_mais_parametros_das_formas.md`](../vetor_nova_ui/15_mais_parametros_das_formas.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `8f20c60de` (o `main` não andou — sem rebase).
- Commits desta jornada: `68929b338..HEAD` (12 de código/teste + docs). A F9 é `b541e1b0a..68929b338`.
- **O pedido (dono, 11/10):** *«Não podemos reduzir parâmetros… precisamos de mais. Se algum
  parâmetro não pode ser colocado no gizmo… um modal que aparece na lateral.»* · *«os controles devem
  aparecer imediatamente… ainda no modo shape; no modo select não»* · *«clicar nos controles ou gizmo
  tem a preferência em relação a criar nova forma.»*

## §00 — A 2.ª redacção da ESTRELA (o dono: «a estrela ficou horrível»)

Depois do 1.º fecho o dono reprovou a estrela: *«nem tem como criar uma estrela com isso. vá
descobrir como fazem Affinity e Illustrator. veja por fotos como eles posicionam o gizmo»*. Lidas as
figuras do manual do Affinity e os GIFs do Illustrator (doc 15 §1b): os pontos passaram para JUNTO da
ponta de cima (a ponta arredonda pelo eixo dela — `Spot::Apex`, desenhado para dentro do topo; o vale
da direita é o raio interior SÓ radial; o vale da esquerda arredonda os vales; a envergadura a meio do
lado esquerdo da ponta), as pontas/lados num ⇕ na borda direita da caixa visível (`Spot::Box` +
`Law::Steps`), a torção pela mão (a `Law::Polar` do Inkscape) SAIU, e a estrela nova nasce regular
(0,382). De caminho: envergadura + raio de canto engordava a forma (0,067 acima do vértice) —
curado; a régua `dist_to_outline` media só às amostras — curada. Commits `d731f7422`, `413592ee0`.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro; «…enquadrar e as alças das formas (D10)» → «…enquadrar, as alças
das formas no modo Forma e o cartão lateral (D10)»; o smoke `…|6..15` → `…|6..16`; para caber, o
parêntese do spec/06 encurtou (`seletor`, `em mode_drive`).

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **o oráculo** | o Inkscape 1.4.4 (GPL: caixa preta) pela LINHA DE COMANDO (`object-to-path` + `export-plain-svg`) sobre 31 formas nossas: o *Rounded*, a torção (`arg2`), a divergência e o `t0` da espiral | `vetor_nova_ui/oraculo_parametros/` (`corre_inkscape_params.py`, 4 fixtures) |
| ⭐ **os parâmetros** | tipo do canto (Rounded · Straight · Concave · Cutout) no rect arredondado, polígono e estrela; envergadura (*Curve*) e liso (*Smooth*) no polígono e na estrela; torção (*Twist*) na estrela; *Expansion* e *Inner* na espiral; *Tooth* e *Curvature* na engrenagem; *Right* no trapézio; *Mid* no losango — todos APENSOS e NEUTROS em zero | `ph2d-vec-scene/src/{polystar.rs, corner_style.rs}` (novos), `symbols.rs` (`gear_ex`), `flow.rs` (`trapezoid_asym`, `diamond_mid`), `space.rs` (`bend_edges`), `kind.rs` (o `cook`); os campos em `ph2d-tool-vector/src/{shapes.rs, shapes_fields.rs}` |
| **as alças novas** | a estrela e o polígono do Affinity (§00): `Spot::Apex` (o eixo da quina), `Spot::Box` + `Law::Steps` (o ⇕), `Spot::EdgeMid` (a envergadura); o raio interno da espiral, a largura do dente, o meio do losango (`inset`), o Alt no trapézio | `ph2d-tool-vector/src/{shape_handles.rs, shape_handles_spots.rs (novo), shape_handles_table.rs}` |
| ⭐ **o cartão lateral** | os campos que nenhuma alça move, num cartão flutuante à direita da forma (espelha à esquerda; desvia da ④/coluna/flyout; nunca cobre a forma) enquanto as alças estão à vista; os MESMOS ids da fila ② (que fica com todos os números) | `ph2d-editor-core/src/screens/hero/tool_bars_card.rs` (novo), `interaction/tool_bars.rs` (`ToolBars::card`), `ph2d-panel-vector/src/bars_row.rs` (`card_chips`, `shape_field_chip`, `shape_handles_shown`) |
| ⭐ **as alças no modo Forma** | o plano do overlay passa as alças ao **Shape** (nunca no Select); o alvo é a MESMA porta dos campos (`Gate { mode, armed }`: armado no catálogo ⇒ sem alças); a forma acabada de desenhar tem alças no mesmo quadro; a caixa existe no Shape **só com as alças** (`ObjectBox::HandlesOnly`: sem miolo nem pivô — desenhar por cima de uma forma continua); a alça da forma ganha à da caixa; os nós da forma somem com as alças à vista | `ph2d-app-vec/src/{overlay.rs, shape_handles_live.rs, shape_live.rs}`; `ph2d-editor-core` (`paint_sprite_gizmo_with`, `GizmoStateGroup::handles_only`); a shell (o press, a guarda do arrasto da caixa, a publicação da caixa, o overlay de nós) |
| **a dica ⑤** | `panel.vector.hint.shape_handles` («…Empty canvas: draw another»); a `select_shape` saiu | `ph2d-i18n`, `ph2d-panel-vector/src/bars.rs` |
| **cenas** | a 15 refeita (o botão Forma); a **16** nova: uma estrela, as pontas arredondam, o vale TORCE, a aresta ENVERGA e o *Corner* do cartão vai a *Concave* — fotos `f10_01`/`f10_02` | `ph2d-app-vec/src/{vector_mode_card.rs (novo), vector_mode_handles.rs, vector_mode.rs}` |

## §2 — Medido e decidido (técnico, delegado)

Tabelas no doc 15 §1–§3 e §6.

- **O *Rounded* do Inkscape** = vértice liso com alças de `rounded × corda`, na direcção vizinho
  ANTERIOR → SEGUINTE (a estrela torcida mediu 15,015°; a perpendicular ao raio — a 1.ª redacção —
  dava 0°). 12 formas a `< 1e-6`; torcida com liso a `1,8e-5` (a alça dele fica 0,015° ao lado).
- **A espiral** `r = t^divergência`, `t ∈ [t0, 1]`: as ÂNCORAS do Inkscape caem na lei a `≤ 9e-7`; as
  cúbicas dele (4 por volta) afastam-se `0,006` — a régua é a lei. A nossa (16 por volta; com
  divergência < 1 parametrizada pelo RAIO, porque a derivada no centro é infinita): `< 1e-3`.
- **O vale torce com a mão** no Inkscape (12 arrastos a `< 0,3°`) — **recusado**: o produto do dono
  (Affinity, Illustrator) não torce pelo vale; gate novo `the_star_valley_never_twists`.
- **Tecto de 8 campos MANTIDO**: o mais cheio é a estrela (8). O tipo de canto POR canto (9 no rect) é
  recusa medida — o tecto é do documento (`VecShape::Param.values: [f64; 8]`).
- **A caixa no modo Forma** revê o ADR-0112 por ordem do dono (o clique que ela «rouba» é o que ele
  quer que ela ganhe), sem o miolo.

## §3 — Foundational tocado e contratos

- **`ph2d-vec-scene`**: **`DEFAULT_STAR_INNER` 0,5 → 0,382** (só a estrela NOVA: o save guarda o
  valor); módulos novos `polystar` (`PolyStar`, `polystar`, `spiral_ex`,
  `round_rect_styled`) e `corner_style` (`CornerStyle`); `corner_live::round_authored_corners_styled`
  (a antiga delega, byte a byte); `gear_ex`, `trapezoid_asym`, `diamond_mid` (as antigas ficam, com a
  mesma assinatura — `gear` tem consumidores no Motion); `space::bend_edges` (crate). O `cook` de 7
  formas passa pelas portas novas (com zero = as antigas: gate).
- **`ph2d-tool-vector`**: `Spot::{EdgeMid, Apex, Box}`, `Law::Steps` (a `Law::Polar` da 1.ª
  redacção saiu), **`HandleDesc::inset` (campo novo — struct literal fora da tabela não compila)**;
  `DEFAULT_STAR_INNER` (a cópia da ferramenta) passa a apontar para a do catálogo; o módulo `shapes_fields` (os blocos `FieldDesc` saíram do
  `shapes.rs`, sem mudar valores); o descritor ganhou campos em 8 formas (o losango deixou de ser uma
  forma sem campos).
- **`ph2d-editor-core`**: `ToolBars::card` (**campo novo**), o módulo `screens/hero/tool_bars_card`,
  `GizmoStateGroup::handles_only` (**campo novo**), `gizmo::paint_sprite_gizmo_with`.
- **`ph2d-i18n`**: 10 chaves `panel.vector.engine.campo.{rounded,straight,concave,cutout,twist,
  expansion,tooth,curvature,right,mid}`; `panel.vector.hint.shape_handles` nova; `panel.vector.hint.
  select_shape` APAGADA (o único leitor era a dica da F9).
- **`ph2d-panel-vector`**: a ponte `nomes_do_motor::chave_do_campo` (+10); `bars::shape_handles_shown`.
- **`ph2d-app-vec`**: `overlay::{ObjectBox, object_box}`, `shape_handles_live::{Gate, hides_the_nodes}`,
  `shape_live::select_the_drawn_shape` (saiu da shell).
- `PROJECT_SCHEMA`: NÃO mexe (o `VecShape` não mudou). Contratos congelados (§6): nenhum.

## §4 — O que só a árvore combinada / o `ship.sh` pega

- A shell (F10): `git diff --shortstat 68929b338 HEAD -- shells/desktop/src` = **+32 −34** (−2
  líquidas); com os testes de texto, +17. O tecto recontado é do integrador.
- ⛔ **Um gate lê a condição da caixa NA SHELL pelo caminho** (`the_object_box_condition_names_every_
  authoring_tool` exige `ToolId::new("vector")` em `fase_snapshots_publish.rs`): mover essa conta
  para a app-vec partiu-o — a ferramenta continua nomeada lá (`b8ece64df`).
- Os gates de TEXTO da shell que liam o cluster do Select foram REESCRITOS para a regra nova
  (`the_shape_handles_belong_to_shape_mode_alone`, `the_shape_hint_teaches_…`,
  `the_pointer_and_the_keyboard_wire_the_shape_handle_gesture`, `joint_draw_gesture`) — se outra linha
  mexeu no `despacho_clique_select.rs`, o conflito resolve-se a favor do press da alça FORA do portão do
  Select.
- **Gate batched (1×, HEAD `af76cb758`, `verificador`)**: `nextest-impacted` (`BASE=8f20c60de`)
  **19 554 / 19 560** · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ · `machete` ✓ ·
  `check-standalone-optional` ✓ · `check-workflow-packages` ✓ · clippy do `ship.sh` ✗ · fmt ✗. Os
  vermelhos e a cura (`e551c461a`, `b8ece64df`):
  - `every_declared_field_of_every_shape_moves_its_geometry` — o *Corner* do polígono só age com raio:
    a conta procura um outro campo no máximo que o acorde (sem lista à mão);
  - `fn_loc_caps` — `fase_snapshots_publish` 204 > 200: paga com um comentário que repetia o HR-8;
  - `joint_anchor_gizmo` e `the_node_overlay_hangs_off_the_envelope_gate` — agulhas de TEXTO (a porta
    nova da caixa; o guard dos nós com o `hides_the_nodes`, que o gate passa a aceitar e só a ele);
  - `the_palette_reaches_the_vector_bars` — o 1.º chip de ESCOLHA nos modos que ele percorre: cicla e
    sai como `SetValue` no id gémeo do campo (a porta certa), o teste aceita-a;
  - `g4_o_custo_do_rc_nao_depende_do_numero_de_luzes` (Luz 2D) — flake de carga: sozinho 1/1 a load 25;
  - clippy `redundant_guards` (`CornerStyle::of`); fmt (a ordem dos `mod`, dois `main.rs`).
  **2.ª varredura inteira** (`e551c461a`): 19 558 / 19 560 — `the_object_box_condition_names_every_
  authoring_tool` (curado em `b8ece64df`, verde) e `motion_path_perf::the_cost_of_sampling_a_path_is_
  flat_in_its_anchors` (a timeline, intocada: sozinho verde a load 40 — flake). clippy do `ship.sh` e
  fmt ✓ depois.

## §5 — Provas de mutação

Agente `mutacao`, HEAD `b4e74c6e2`, controlos verdes com população em cada filtro, cada âncora 1×,
árvore limpa. **17/20 sangram** à 1.ª; das 3 sobreviventes, 2 pediram teste (vistos VERMELHOS sobre a
mutação, `af76cb758`) e 1 é equivalente:

| # | mutação | sangra em |
|---|---|---|
| M1 | o côncavo vira recta | **sobreviveu** ⇒ `concave_bites_inside_the_straight_chamfer` |
| M2 | o entalhe vira recta | `concave_bites_in_round_stays_out_and_the_cutout_is_a_notch` |
| M3 | os valores 2 e 3 do tipo trocados | **sobreviveu** ⇒ `the_field_value_picks_the_corner_style_in_the_descriptor_order` |
| M4 | o liso perpendicular ao raio | `twist_draws_the_inkscape_arg2_star` |
| M5 | o liso a metade | `smooth_draws_the_inkscape_rounded_star_and_polygon` |
| M6 | a envergadura para dentro | `the_curve_pushes_the_edge_middle_by_half_a_chord` |
| M7 | sem torção | `twist_draws_the_inkscape_arg2_star` |
| M8–M10 | a espiral sem expoente / sem `t0` / sem a reparametrização | `spiral_ex_matches_the_inkscape_divergence_and_inner_radius` |
| M11 | sem o retorno antecipado da identidade | **equivalente**: o caminho genérico dá o MESMO `VecPath` nas entradas medidas — o atalho é a garantia por construção, o gate `the_new_fields_at_zero_are_the_old_shapes_byte_for_byte` a observável |
| M12–M13 | (a `Law::Polar` — saiu na 2.ª redacção da estrela) | — |
| M14 | o ângulo polar não conta como alça | `the_fields_without_a_gesture_are_counted_not_listed` |
| M15 | o losango sem `inset` | `the_diamond_middle_handle_is_drawn_inside_the_box_edge` |
| M16 | o canto só na corda | `the_corner_handle_still_rides_a_curved_edge` |
| M17 | o Alt do trapézio sem `keep` | `alt_moves_only_the_grabbed_side_of_the_trapezoid` |
| M18–M19 | o dente / o lado direito ignorados | `the_f10_fields_are_neutral_at_zero_and_each_one_moves_the_shape` |
| M20 | os nós nunca somem | `the_nodes_hide_only_while_the_shape_handles_show` |

O agente B (o cartão) correu 6/6 (a M6 sobrevivia → gate
`the_card_never_trades_an_obstacle_for_the_selection`); o A, 1 (as alças de volta ao Select ⇒ os 4
`shape_mode_tests` falham).

**A 2.ª redacção da estrela** (HEAD `413592ee0`, agente `mutacao`): **8/10 sangram** à 1.ª — o ⇕
(sinal, degrau, caixa visível: `the_points_widget_steps_the_count_up_and_down`), o ponto da ponta
desenhado para dentro (`the_tip_handle_slides_down_…`), o vale só radial (`the_star_valley_never_twists`
+ o do Inkscape), a estrela regular (`a_new_star_is_born_regular`), arredondar que não engorda e a junção
lisa (`the_rounding_never_grows_past_the_sharp_corner`). As 2 sobreviventes pediram teste, vistos
VERMELHOS sobre a mutação: N1 (o cruzamento do eixo mais LONGE) ⇒ `the_apex_is_the_crossing_nearest_its_corner`;
N10 (a folga do cartão sem a alça) ⇒ `the_card_leaves_a_whole_handle_free_at_the_selection_edge`.

## §6 — Smoke (do dono)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=16 ./target/smoke/ph2d-host-desktop`
2. A cena desenha uma estrela (nasce regular) com a ferramenta de forma na mão e, sozinha, puxa os
   pontos verdes junto da ponta de cima: as pontas arredondam, os vales arredondam, e o ⇕ da borda
   direita da caixa sobe — 7 pontas.
3. Tem de acontecer: uma estrela de 7 pontas macias; 4 pontos verdes junto da ponta de cima e 1 na
   borda direita da caixa; à direita, o cartão (*Corner* · *Smooth* · *Twist*).
4. Errado: pontos espalhados pela estrela ou escondidos debaixo dos quadradinhos azuis; o cartão por
   cima do ⇕; a estrela torta.
5. Na mesma janela: com a ferramenta de forma, desenhe outra forma num canto vazio — as bolinhas dela
   aparecem logo; clique numa bolinha e arraste (ela ganha a desenhar); clique no vazio para desenhar
   mais uma; troque para a seta preta (Seleção) — as bolinhas somem e fica só a caixa.

## §7 — O que fica ABERTO

- ⚠️ **PERGUNTAS ao dono:** a posição do cartão (à direita da forma, espelha à esquerda); os botões
  *Inverter*/*Fechar* da pizza; o tipo de canto diferente em cada canto (pede subir o tecto de 8, que é
  do ficheiro do projecto).
- O despacho real da shell não tem harness de janela: a ordem do press no modo Forma (alça da forma ›
  alça da caixa › desenhar) e o Move/Up de um arrasto da caixa estão provados por gates de texto e pelas
  leis do motor (`shape_mode_tests`), não por um rato real.
- Uma alça da caixa por cima do TRAÇO de uma forma aberta (linha, arco) cai no pick (herdado do Select).
- O arrasto e a edição de texto de um campo numérico DENTRO do cartão não foram exercitados por evento
  (o estado do store é o da fila ②, que é exercitado).
- Do F9 (continua): o botão direito a meio do arrasto não cancela; um dedo real ainda não agarrou uma
  alça.

## §7b — A 2.ª redacção: portão

`nextest-impacted` (`BASE=8f20c60de`, HEAD `413592ee0`): **19 564 / 19 564** (load 118) · clippy do
`ship.sh` ✓ · fmt ✓. Tecto de 700 LOC: a lei das alças (743) e os testes dela (723) partidos em
irmãos (`shape_handles_spots.rs`, `shape_handles_f9_oracle_tests.rs`).

## §8 — Portão, binário e `agent-loop-profile.sh`

Depois do `rm -rf target/*/incremental` (29 GB debug + 4,1 GB smoke) — `bash scripts/ph2d-run.sh
cargo build -p ph2d-host-desktop --profile smoke` — refeito depois da 2.ª redacção da estrela (HEAD
`cad9b8cd6`): 1.ª corrida 48,6 s; a 2.ª:

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.32s
```

O commit deste handoff é só documentação: o binário continua o do código.

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                180   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                568 : 187   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  30%   alvo: >= 80%  (890 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         396 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
(Edições em lote por `python3` com `assert` de contagem; três agentes de implementação em paralelo —
a parte A (portas), a B (cartão) e esta janela (parâmetros) —, o `mutacao` e o `verificador`.)
