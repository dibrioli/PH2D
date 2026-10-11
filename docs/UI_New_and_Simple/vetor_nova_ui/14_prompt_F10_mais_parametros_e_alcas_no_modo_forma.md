═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F10: MAIS PARÂMETROS, O CARTÃO LATERAL, E AS ALÇAS NO MODO FORMA
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado em 11/10: F0–F8 no `main`; a F9 (as alças das formas) FECHADA na linha e NÃO integrada —
`git cherry main HEAD` dá os commits da F9 + este prompt: siga SEM reset. Se o `main` andou:
`git rebase main` (os conflitos prováveis estão no §4 do handoff da F9).

O PEDIDO DO DONO (11/10, depois do relatório da F9) — palavra por palavra:
 *«Não podemos reduzir parâmetros de configuração das formas. Pelo contrário. Precisamos de mais
 parâmetros. Se algum parâmetro não pode ser colocado no gizmo então precisamos criar um modal que
 aparece na lateral.»*
 *«Ao criar uma forma, os controles devem aparecer imediatamente mesmo fora do modo select, ainda no
 modo shape. No modo select os controles não devem aparecer.»*
 *«No modo shape ao clicar nos controles ou gizmo esses têm a preferência em relação a criar nova
 forma. A nova forma só é criada se o usuário tocar no canvas vazio e não nos controles ou gizmo.»*

⚠️ Isto INVERTE uma decisão da F9: as alças nasceram só no Select (`VecOverlayPlan::shape_handles`,
o press no cluster de alças do Select). Agora: modo FORMA sim, Select NÃO.

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F9_ALCAS_DAS_FORMAS.md
    (§1–§3, §7 — o que ficou sem alça)
  docs/UI_New_and_Simple/vetor_nova_ui/13_alcas_das_formas.md (a lei, os oráculos, as recusas)
  docs/UI_New_and_Simple/vetor_nova_ui/referencias/R1_affinity.md §1.3 (a Context Toolbar e as
    opções de cada ferramenta de forma) · R3 §A (Illustrator: Live Shapes, Transform ▸ Shape
    Properties)

O QUE JÁ EXISTE (confira no código antes de acreditar numa ausência — CLAUDE.md §5.0):
  • A lei das alças: `ph2d-tool-vector/src/shape_handles.rs` (+ `shape_handles_table.rs`, 42 formas)
    e o mundo `ph2d-app-vec/src/shape_handles_live.rs` (press/drag/Esc/duplo-clique, `on::*`,
    `draw`). A escrita é UMA porta: `shape_params::edit_selected_shape`.
  • Os campos: o descritor `ph2d-tool-vector/src/shapes.rs` (SHAPES, `FieldDesc`); o catálogo de
    geometria `ph2d-vec-scene/src/kind.rs` (append-only, `ShapeValues = [f64; MAX_SHAPE_FIELDS = 8]`,
    espelhado em `ph2d-ecs::MAX_SHAPE_VALUES` — o documento GUARDA o array); os números na fila ②
    (`ph2d-panel-vector/src/bars_row.rs::shape_chips`); a semente `shape_params::frame_fields`
    (o latch `shape_armed`: no modo Shape os campos falam da forma ARMADA, não da selecionada).
  • O modo Shape: o press do canvas cai em `shape.on_press` (`despacho_clique_vetor_premido.rs`,
    o ramo da caneta); o nascimento vivo é `shape_live::make_committed_shape_live`.
  • A pergunta da F9 (onde fica o número exacto) — o dono respondeu com o pedido acima: o que não
    cabe numa alça vai a um CARTÃO LATERAL.

ESCOPO — produto final:
  0. ⭐ O ORÁCULO PRIMEIRO (CLAUDE.md §0.9). «Mais parâmetros» = os que os apps de referência têm e
     nós não: o Affinity pelo MANUAL (por forma: Corner Rounded/Straight/Concave/Cutout + Single
     radius + Absolute sizes; Polygon Curve/Smooth points; Star Curved edges, Inner/Outer circle;
     Pie Invert/Close; Arrow Ends/Style/Proportional; Cog Curvature; Spiral Turns/Decay/Arc angle;
     Callout…) e o Inkscape CORRIDO (`oraculo_alcas/corre_inkscape.py`: Rounded e Randomized da
     estrela, expansão e t0 da espiral, os tipos de arco). Faça a TABELA forma × parâmetro (temos /
     Affinity / Illustrator / Inkscape) no doc 15 e proponha a lista; cada parâmetro novo entra
     APENSO, NEUTRO em zero (um save antigo cozinha igual — o gate do `kind.rs` cobra).
     ⚠️ Se uma forma passar de 8 campos, o tecto `MAX_SHAPE_FIELDS`/`MAX_SHAPE_VALUES` é FOUNDATIONAL
     e é do documento (serde do `VecShape`): meça o que quebra (saves antigos, o undo por snapshot,
     o `PROJECT_SCHEMA` — conte com `python3 scripts/schema-recount.py`) ANTES de subir o número.
  1. As ALÇAS dos parâmetros novos que têm gesto (a mesma lei: `Spot` × `Law`, gates
     `every_handle_sits_on_its_outline` e `the_hand_takes_the_handle_where_it_is_dropped`
     automáticos sobre a tabela).
  2. ⭐ O CARTÃO LATERAL: os campos SEM alça (os novos e os da F9: suavização do canto, nº de bolhas,
     jitter da explosão/nuvem, o «Viewed» das isométricas, os tipos de canto…) num cartão flutuante
     que aparece JUNTO da forma enquanto as alças estão à vista e some com elas. Procure a porta que
     já existe para um cartão flutuante (a barra de ações ④, o popover das amostras, os cartões do
     Motion) — `git grep` do comportamento — antes de escrever uma nova. Mesmos ids
     (`vector_shape_field_id/_choice_id`): um comando, um id. Decida (e diga) se a fila ② continua
     com os números ou se eles passam ao cartão — o dono não mandou tirá-los da ②.
  3. ⭐ As alças no MODO FORMA, não no Select: (a) a forma acabada de desenhar mostra as alças
     IMEDIATAMENTE, ainda com a ferramenta de forma na mão; (b) no Select (seta preta) as alças NÃO
     aparecem (só a caixa); (c) no modo Forma, o press sobre uma alça — ou sobre o gizmo da forma —
     ganha ao «desenhar outra forma»; só o press no canvas VAZIO desenha. ⚠️ Confira se o gizmo da
     caixa existe hoje no modo Forma (a F9 mediu que no vetor o gizmo de sprite só entra no Select);
     se não existe, a «preferência do gizmo» pede-o lá — meça a prioridade do clique e prove-a com
     gates de costura (o evento real → a alça / o gizmo / a forma nova). ⚠️ O latch `shape_armed`
     (F9) decide de quem são os campos no modo Forma: depois de desenhar, os campos e as alças têm de
     falar da forma NOVA; re-armar no catálogo volta à forma armada — releia o doc do `frame_fields`.
     A dica ⑤ muda junto (hoje `panel.vector.hint.select_shape` vive no Select).
  4. A cena 15 muda (as alças agora no modo Forma, o cartão lateral aberto); conte no roteador
     `ph2d-app-vec/src/vector_mode.rs` se é a 15 refeita ou a 16 nova; FOTOGRAFE
     (`docs/Components/ferramentas/fotografa_cena.sh`).
  5. Gates; prova de mutação de cada cláusula nova (agente `mutacao`, depois do commit); gate batched
     (`verificador`).
  FORA: as formas do Motion; integrar; enviar.

DECISÕES QUE NÃO SÃO SUAS: tirar QUALQUER parâmetro (o dono: «não podemos reduzir»); tirar os números
da fila ② (pergunte); a posição/encaixe do cartão além de «na lateral» (proponha, pergunte); um botão
novo em barra/painel; instalar programas.

DECISÕES DO DONO JÁ TOMADAS (não re-pergunte): teclas do Illustrator; a ④ ligada; o Edit Shape fica
junto da forma; a seta por omissão é o V aberto; `Ctrl+0`/`F` enquadram; F10: alças no modo Forma e
NÃO no Select; o que não cabe numa alça vai a um cartão lateral; o press nas alças/gizmo ganha ao
desenhar.

LIÇÕES DA F9 (não repita):
  · ⛔ O que só a FOTO mostra: as alças de ângulo no aro caíam debaixo das alças de escala da caixa
    (cardeais da elipse = meio das arestas) — fotografe ANTES do relatório, e com a forma que o dono
    vê (a estrela a 0,8 lia-se pentágono).
  · ⛔ Meça a lei antes de a «melhorar»: a pega do Inkscape é TRANSLAÇÃO (20/20 a 0,000); a rotação
    que parecia certa errava 0,6°. E uma métrica sobre um vértice que o `cook` cola à aresta não
    converge (o bico → `BoxPoint`).
  · ⛔ Gates de TEXTO de outras famílias leem títulos de comentário da shell (o `joint_draw_gesture`
    usava «A alça do TEXTO EM CAMINHO» como fim de bloco): ao fundir/renomear blocos do despacho,
    `git grep` o título nos `tests/it/`.
  · ⛔ Uma cor nova é uma fileira nova no painel de tokens (+25 px na catraca de altura) e uma entrada
    nos censos de apelidos (`FROZEN`, `FAMILIAS`, `N × 8`).
  · ⛔ O §5.1 do CLAUDE.md que cita o handoff ANTES de ele existir reprova o gate dos caminhos —
    escreva o handoff antes do último gate.
  · O clippy do `ship.sh` (`--features ph2d-spike/bevy_ecs -D warnings`) apanha o que o `check` não
    vê (8 argumentos, listas em doc, `field_reassign_with_default`): corra-o no gate.
  · COMMIT antes do `mutacao`; controle de população em cada filtro; a mutação que sobrevive pede o
    TESTE que falta (a F9: o `Esc` pela porta de entrada).
  · O guarda de recursos recusa um comando cujo TEXTO cita `cargo`/`nextest` (até um heredoc): texto
    por `Edit`/`Write`. A shell só encolhe (a F9 desceu 112); `*_tests.rs` irmão não é isento do
    tecto de 700; a `App` não ganha campos (o estado vai ao `VecState`).

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 0.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação · handoff NOVO em
docs/UI_New_and_Simple/handoffs/ (o da F9 cita-se) · CLAUDE.md §5.1 «Último:» (troque o link; a
entrada UI/UX tem 694 de 700 bytes — corte texto para caber) · `bash scripts/doc-index.sh` ·
rm -rf target/*/incremental · binário smoke 2× · smoke ao dono em passos numerados, sem jargão. Você
NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
