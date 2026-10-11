═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F9: OS PARÂMETROS DE CADA FORMA SAEM DO INSPECTOR PARA O CANVAS
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado em 11/10: F0–F8 INTEGRADAS e enviadas (o `main` de 11/10, `8f20c60de`, já as tem). A linha foi
reposta no `main` (`git reset --keep main`) e o único commit à frente é ESTE prompt ⇒ `git cherry main
HEAD` dá 1 (este doc): siga sem reset. Se o `main` tiver andado: `git rebase main`.

O PEDIDO DO DONO (11/10): *«tirar do inspector as opções de customização de cada shape e colocar nos
gizmos, como apps de alta qualidade como o Affinity fazem.»* — os parâmetros de uma forma (o raio dos
cantos, as pontas e o raio interior da estrela, os lados do polígono, o ângulo do arco e da pizza, as
voltas da espiral, a largura da cabeça de uma seta…) passam a ser ALÇAS sobre a forma, agarráveis no
canvas, em vez de campos no painel lateral.

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F8_ENQUADRAR.md (§1, §7, §10)
  docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md §2 (a tabela de destino das 41 secções — quem vai
    para o canvas, quem vai para a fila ②) e §6 (as decisões do dono: Illustrator nas teclas, a ④
    ligada)
  docs/UI_New_and_Simple/vetor_nova_ui/referencias/R1_affinity.md (o Affinity: as alças laranja das
    formas, a barra de contexto) · R3 (Illustrator *Live Shapes*: os widgets de canto e de lados)
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7e_EDIT_SHAPE.md (os nós
    próprios do QuickShape: a gramática visual e o ciclo de arrasto que JÁ existem)

O QUE JÁ EXISTE (confira no código antes de acreditar numa ausência — CLAUDE.md §5.0):
  • O catálogo paramétrico: `ph2d-vec-scene/src/kind.rs` (`ShapeKind`, ~27 tipos, append-only,
    `ShapeValues = [f64; MAX_SHAPE_FIELDS]`) e o descritor de campos por tipo; a edição viva da forma
    seleccionada é `ph2d-app-vec/src/shape_params.rs` (`shape_field_target`, `apply_shape_field`,
    `edit_selected_shape` — «tudo genérico sobre o catálogo»: uma alça nova deve entrar pela MESMA
    porta, nunca uma 2.ª mutação da forma).
  • Alças no canvas que já agarram: os nós do Edit Shape (`ph2d-vec-render/src/shape_nodes.rs`,
    `ph2d-vec-edit/src/quickshape_edit.rs`, `pencil_edit.rs`), os pinos do Envelope, a caixa de
    redimensionar e o gizmo de objecto (`ph2d-editor-core/src/gizmo/`). Reuse a gramática e o ciclo
    (agarrar → arrastar → soltar = UM passo de undo; `Esc` repõe).
  • Onde os campos aparecem hoje: o painel Vector (`ph2d-panel-vector`) — secção da forma.

ESCOPO — produto final:
  0. ⭐ O ORÁCULO PRIMEIRO (CLAUDE.md §0.9). O Affinity não tem build para Linux: a LEI do produto do
     dono é o MANUAL dele (R1, e o help oficial — que alças cada forma tem, o que arrastar cada uma
     faz, onde fica o número exacto). O oráculo que CORRE: o **Inkscape 1.4.4 instalado** (GPL —
     corre-se, nunca se lê): as ferramentas Rectângulo (alças de arredondar), Estrela/Polígono (raio
     interior, arredondamento), Espiral (voltas, raio interior) e Elipse (arco/pizza) têm alças no
     canvas. Corra-o por script numa tela invisível (o molde é `oraculo_quickshape/corre_oraculo.sh`:
     Xvfb + xdotool + gravar o `.svg`) sobre formas NOSSAS, e grave fixtures com cabeçalho: onde cada
     alça nasce, o que o arrasto faz ao parâmetro (a lei, com e sem Shift/Ctrl), os limites. Triagem
     de licença por artefacto primeiro (`pacman -Qi inkscape`).
  1. ⭐ A LEI, forma a forma: que parâmetro vira alça, onde ela nasce (no espaço da forma, segue a
     rotação/escala), o que o arrasto mede (raio = distância ao canto; lados/pontas = ângulo à volta
     do centro, em degraus; ângulos de arco = o ângulo do ponteiro), o que o Shift faz, os limites (os
     do descritor — perguntados, nunca copiados). Um parâmetro sem gesto natural (ex.: um booleano)
     NÃO vira alça: diga qual e para onde vai.
  2. AS PORTAS: as alças aparecem com a forma seleccionada e a ferramenta de seleção na mão (e não
     brigam com os nós do Edit Shape nem com as alças de escala da caixa — meça a prioridade do clique
     e prove-a com gates de costura: o evento real → o parâmetro). Undo: um passo por arrasto. Toque:
     a alça agarra-se com o dedo (o reconhecedor da F6 já entrega o `Press`/`Drag`).
  3. O INSPECTOR: a secção da forma SAI do painel (o pedido). ⚠️ O número EXACTO não pode desaparecer
     do app (um artista que quer «8 pontas» ou «raio 12 px» digita) — proponha onde ele fica (o
     Affinity põe-no na barra de contexto, que aqui é a fila ② de opções da ferramenta) e PERGUNTE ao
     dono no relatório; até ele decidir, a fila ② é o sítio.
  4. Uma cena NOVA (conte no roteador `ph2d-app-vec/src/vector_mode.rs`; a próxima livre deve ser
     15): um rectângulo arredondado, uma estrela e um arco seleccionados um a um, cada alça arrastada
     pelo press real; FOTOGRAFE (`docs/Components/ferramentas/fotografa_cena.sh`).
  5. As sondas e os gates; prova de mutação de cada cláusula nova (agente `mutacao`), gate batched
     (`verificador`).
  FORA: as formas do Motion (`ph2d-node-motion-shape` — outro catálogo, outro módulo); customizar a
  barra; integrar.

DECISÕES QUE NÃO SÃO SUAS: onde fica o número exacto (proponha a fila ② e pergunte); tirar QUALQUER
outra secção do Inspector além da da forma; um botão novo em barra/painel; instalar programas.

DECISÕES DO DONO JÁ TOMADAS (não re-pergunte): teclas do Illustrator; a ④ ligada; o Edit Shape fica
junto da forma; o contorno no Edit Shape AUMENTA a forma; a seta por omissão é o V aberto; `Ctrl+0`
enquadra tudo e `F` enquadra a seleção com zoom.

LIÇÕES DESTA LINHA (F7–F8 — não repita):
  · ⛔ A premissa do prompt mente: a F8 achou o «enquadrar» que o prompt dizia não existir (outro
    nome). Procure pelo COMPORTAMENTO, não só pelo nome — `git grep` do verbo e do tipo.
  · ⛔ Os oráculos são a mesma lei com PARÂMETROS diferentes: grave-os inteiros e faça o gate
    reproduzir os números deles com os parâmetros de cada um (F8: ≥ 200 corridas ao centésimo de px).
  · ⛔ Um gesto novo herda os inimigos do antigo: uma alça nova compete com a caixa, os nós e o gizmo
    pelo MESMO clique — e o gesto mais NOVO ganha (F8: a roda perdia o 1.º entalhe a meio de um voo).
  · ⛔ O que só a árvore combinada mostra: na F8 o `main` tinha escrito a MESMA função que a linha
    (`GizmoView::world_aabb`). Antes de escrever uma porta, `git grep` o nome e o comportamento.
  · ⛔ COMMIT os teus docs ANTES de lançar o `mutacao` (árvore limpa a cada passo); confira o FILTRO
    do teste que cada mutação corre (F8: um filtro desactualizado fez uma mutação «sobreviver»).
  · O guarda de recursos recusa um comando cujo TEXTO cita `cargo`/`nextest` (até um heredoc): escreva
    texto por `Edit`/`Write`. O `cargo fmt` parte linhas: âncoras por script são o texto FORMATADO.
    A shell só encolhe; `*_tests.rs` irmão não é isento do tecto de 700; a `App` não ganha campos.

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 0.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação · handoff NOVO em
docs/UI_New_and_Simple/handoffs/ (o da F8 cita-se) · CLAUDE.md §5.1 «Último:» (troque o link; a
entrada UI/UX tem 693 de 700 bytes — corte texto para caber; o smoke `…|6..14` → `…|6..15`) ·
rm -rf target/*/incremental · binário smoke 2× · smoke ao dono em passos numerados, sem jargão. Você
NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
