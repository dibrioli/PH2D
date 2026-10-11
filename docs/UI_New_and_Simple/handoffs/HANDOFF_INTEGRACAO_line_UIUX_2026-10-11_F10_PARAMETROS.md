# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-11 — F10: MAIS PARÂMETROS, O CARTÃO LATERAL, AS ALÇAS NO MODO FORMA

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F9, 11/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F9_ALCAS_DAS_FORMAS.md) — também
> NÃO integrada: esta linha leva a F9 + a F10 (F0–F8 já estão no `main`). Lei, oráculos, decisões e
> recusas: [`15_mais_parametros_das_formas.md`](../vetor_nova_ui/15_mais_parametros_das_formas.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `8f20c60de` (o `main` não andou — sem rebase).
- Commits desta jornada: `68929b338..HEAD` (7 de código/teste + docs). A F9 é `b541e1b0a..68929b338`.
- **O pedido (dono, 11/10):** *«Não podemos reduzir parâmetros… precisamos de mais. Se algum
  parâmetro não pode ser colocado no gizmo… um modal que aparece na lateral.»* · *«os controles devem
  aparecer imediatamente… ainda no modo shape; no modo select não»* · *«clicar nos controles ou gizmo
  tem a preferência em relação a criar nova forma.»*

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro; «…enquadrar e as alças das formas (D10)» → «…enquadrar, as alças
das formas no modo Forma e o cartão lateral (D10)»; o smoke `…|6..15` → `…|6..16`; para caber, o
parêntese do spec/06 encurtou (`seletor`, `em mode_drive`).

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **o oráculo** | o Inkscape 1.4.4 (GPL: caixa preta) pela LINHA DE COMANDO (`object-to-path` + `export-plain-svg`) sobre 31 formas nossas: o *Rounded*, a torção (`arg2`), a divergência e o `t0` da espiral | `vetor_nova_ui/oraculo_parametros/` (`corre_inkscape_params.py`, 4 fixtures) |
| ⭐ **os parâmetros** | tipo do canto (Rounded · Straight · Concave · Cutout) no rect arredondado, polígono e estrela; envergadura (*Curve*) e liso (*Smooth*) no polígono e na estrela; torção (*Twist*) na estrela; *Expansion* e *Inner* na espiral; *Tooth* e *Curvature* na engrenagem; *Right* no trapézio; *Mid* no losango — todos APENSOS e NEUTROS em zero | `ph2d-vec-scene/src/{polystar.rs, corner_style.rs}` (novos), `symbols.rs` (`gear_ex`), `flow.rs` (`trapezoid_asym`, `diamond_mid`), `space.rs` (`bend_edges`), `kind.rs` (o `cook`); os campos em `ph2d-tool-vector/src/{shapes.rs, shapes_fields.rs}` |
| **as alças novas** | `Spot::EdgeMid` (a envergadura no meio de uma aresta), `Law::Polar` (o vale da estrela: raio + torção, Ctrl = só o raio), o raio interno da espiral, a largura do dente, o meio do losango (`inset`: desenhado para dentro), o Alt no trapézio; o `Spot::Rounding` sobre uma aresta CURVA | `ph2d-tool-vector/src/{shape_handles.rs, shape_handles_table.rs}` |
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
- **O vale torce com a mão**: os 12 arrastos do vale da F9 dão a torção `= Δarg2` a `< 0,3°`.
- **Tecto de 8 campos MANTIDO**: o mais cheio é a estrela (8). O tipo de canto POR canto (9 no rect) é
  recusa medida — o tecto é do documento (`VecShape::Param.values: [f64; 8]`).
- **A caixa no modo Forma** revê o ADR-0112 por ordem do dono (o clique que ela «rouba» é o que ele
  quer que ela ganhe), sem o miolo.

## §3 — Foundational tocado e contratos

- **`ph2d-vec-scene`**: módulos novos `polystar` (`PolyStar`, `polystar`, `spiral_ex`,
  `round_rect_styled`) e `corner_style` (`CornerStyle`); `corner_live::round_authored_corners_styled`
  (a antiga delega, byte a byte); `gear_ex`, `trapezoid_asym`, `diamond_mid` (as antigas ficam, com a
  mesma assinatura — `gear` tem consumidores no Motion); `space::bend_edges` (crate). O `cook` de 7
  formas passa pelas portas novas (com zero = as antigas: gate).
- **`ph2d-tool-vector`**: `Spot::EdgeMid`, `Law::Polar`, **`HandleDesc::inset` (campo novo — struct
  literal fora da tabela não compila)**; o módulo `shapes_fields` (os blocos `FieldDesc` saíram do
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

- A shell (F10): `git diff --shortstat 68929b338 HEAD -- shells/` = **+63 −41** (o `src` +5 líquidas;
  os testes de texto +17). Desde o merge-base (F9 + F10): **+211 −300**. O tecto recontado é do
  integrador.
- Os gates de TEXTO da shell que liam o cluster do Select foram REESCRITOS para a regra nova
  (`the_shape_handles_belong_to_shape_mode_alone`, `the_shape_hint_teaches_…`,
  `the_pointer_and_the_keyboard_wire_the_shape_handle_gesture`, `joint_draw_gesture`) — se outra linha
  mexeu no `despacho_clique_select.rs`, o conflito resolve-se a favor do press da alça FORA do portão do
  Select.
- **Gate batched**: _por preencher (§8)._

## §5 — Provas de mutação

_Por preencher (§8)._ O agente B (o cartão) correu 6/6 (a M6 sobrevivia → gate
`the_card_never_trades_an_obstacle_for_the_selection`); o A, 1 (as alças de volta ao Select ⇒ os 4
`shape_mode_tests` falham).

## §6 — Smoke (do dono)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=16 ./target/smoke/ph2d-host-desktop`
2. A cena desenha uma estrela grande com a ferramenta de forma na mão e, sozinha, puxa as bolinhas
   verdes: as pontas arredondam, o vale torce, as arestas envergam; depois clica duas vezes no chip
   *Corner* do cartão ao lado da estrela.
3. Tem de acontecer: uma estrela torcida, de lados curvos e pontas mordidas para dentro; bolinhas verdes
   sobre ela; à direita, um cartão com *Corner: Concave* e *Smooth*; em cima os oito números.
4. Errado: sem bolinhas verdes; pontinhos laranja/azuis por cima da estrela; o cartão ausente ou a
   tapar a estrela.
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

## §8 — Portão, binário e `agent-loop-profile.sh`

_Por preencher no fecho._
