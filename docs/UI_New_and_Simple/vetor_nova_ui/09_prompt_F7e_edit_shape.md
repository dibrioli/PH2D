═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F7e: «EDIT SHAPE» (afinar a forma perfeita logo depois do QuickShape)
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado em 10/10 (fim da janela F7d): line/UIUX = HEAD desta janela, base
(merge-base) = 2a925859d = main; F0–F7d NÃO integradas (98 commits à frente;
o dono disse «ainda não vamos integrar»). ⇒ `git cherry main HEAD` dá >0 e os
commits são DESTA linha: siga sem reset. Se o main tiver andado: `git rebase main`
(conflito fora dos seus ficheiros = PARE e reporte ao Enio).

O PEDIDO DO DONO (10/10, depois do smoke OK das setas): escolheu, de três opções, «ajustar a forma
depois» — o «Edit Shape» do Procreate: a forma perfeita acabou de nascer e o artista AFINA-A com nós
próprios dela (os cantos do rectângulo, o raio do círculo, a abertura do arco, o bico da seta), sem ir
ao modo de nós genérico.

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7d_QUICKSHAPE_SETAS.md
    (§1, §2, §3, §7 — a seta, as pontas ARMADAS × MOSTRADAS, o que ficou aberto)
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F7_QUICKSHAPE.md §1 e §7 (o
    gatilho, o gesto, «os nós já são editáveis no Node; não há botão próprio»)
  docs/UI_New_and_Simple/vetor_nova_ui/referencias/R4_apps_de_ipad.md L165–169 (o QuickShape do Procreate:
    «O botão "Edit Shape" mostra nós editáveis») e docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md §4–§5
  docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/00_resultados.md §3h (a lei da seta; não
    reconstrua o que lá está recusado)

O QUE JÁ EXISTE (endereços de 10/10 — confira, não reconstrua):
  • A forma reconhecida é um VALOR com tipo: `QuickShape` (crates/ph2d-vec-edit/src/quickshape.rs) —
    Line, Polygon {corners, closed}, Ellipse {centre, rx, ry, rot}, Arc(ArcShape), Biarc([ArcShape;2]),
    Arrow(ArrowShape {shaft, heads}) — com `pivot`/`transformed`/`to_path`/`heads`. Ao SOLTAR, o lápis
    comete só o caminho (`VecPath` com nós) e o valor tipado morre (`quickshape_hold::Hold::reset`): o
    Edit Shape precisa que ele SOBREVIVA ao release (ou que se reconstrua do caminho — decida e meça).
  • O gesto: crates/ph2d-vec-edit/src/pencil.rs (`hold_tick`, `on_drag` escala/roda, `on_release`,
    `quick_dismiss`), quickshape_hold.rs; a shell solta em shells/desktop/src/input_dispatch/
    despacho_clique_vetor_solto.rs (seleciona o caminho cometido).
  • Os nós genéricos (modo de edição de nós do Vector) já editam qualquer caminho — o Edit Shape é a
    versão TIPADA: arrastar um canto do rectângulo mantém-no rectângulo? o nó da elipse mantém-na elipse?
    o do meio do arco mantém-no arco? O Procreate é a referência do dono
    (memória feedback_the_owners_reference_product_decides_the_law).
  • As pontas da seta são `Marker` no traço (`StrokeSpec.marker_*`); a tool separa pontas MOSTRADAS
    (as da seleção) das ARMADAS (as do próximo caminho, `VectorTool::armed_markers`) — não mexa nessa lei.
  • Cena 12: crates/ph2d-app-vec/src/vector_mode_quick.rs — 8 traços (7 seguros + o gelado livre),
    `PX_TO_WORLD = 0,0103`. Fotos em docs/UI_New_and_Simple/vetor_nova_ui/fotos/ (f7d_01 a última).

ESCOPO — produto final:
  0. ⭐ O ORÁCULO PRIMEIRO (CLAUDE.md §0.9): o Procreate não corre aqui (iPad) — o manual (R4) é a
     referência do produto. Procure um app INSTALADO que edite a forma reconhecida com nós próprios
     dela e CORRA-O: o Excalidraw `next` (~/Apps/excalidraw-oracle-next, MIT — a forma do «draw shape»
     vira elemento nativo com alças; o corredor dele em docs/MiroClone/ferramentas/excalidraw_oracle/
     e o nosso em oraculo_quickshape/corre_excalidraw.mjs) e o tldraw (~/Apps/tldraw-oracle). Grave
     as fixtures com cabeçalho: que alças cada forma tem, e o que arrastar cada uma faz à forma.
  1. ⭐ A LEI, por tipo: que nós cada forma mostra e que invariante cada um guarda (o rectângulo
     continua rectângulo? o círculo pode virar elipse? o arco passa por três pontos? a seta move o cabo
     e o bico fica a ponta?). Decida e MEÇA o que não for óbvio; cada constante com a FAIXA, no MEIO.
  2. O GESTO e a ENTRADA: como o artista chega ao Edit Shape logo depois do QuickShape (o Procreate
     mostra um botão no topo depois de soltar) e como sai (um toque fora; trocar de ferramenta; o
     próximo traço). Undo: UM passo por afinação. Gates de costura (o evento real → o efeito).
  3. Cena 12 (ou uma cena NOVA — conte o número no roteador da família): a forma segura, o Edit Shape
     aberto e um nó arrastado; FOTOGRAFE (docs/Components/ferramentas/fotografa_cena.sh).
  4. As sondas e os gates; prova de mutação de cada cláusula nova (agente `mutacao`), gate batched
     (`verificador`).
  FORA: formas novas no QuickShape; setas em dois traços; integrar.

DECISÕES QUE NÃO SÃO SUAS: tecla nova; um BOTÃO novo em barra/painel (proponha onde, com foto, e
pergunte no relatório se não houver lugar óbvio); instalar programas.

LIÇÕES DESTA LINHA (F7–F7d — não repita):
  · ⛔ COMMIT os teus docs ANTES de lançar o agente `mutacao`, e diga-lhe para NÃO restaurar nada fora
    das mutações. ⭐ Sobrevivente sem testemunha óbvia: BUSQUE-a — base × mutante sobre milhares de traços
    (`measure_arrow_witness` é o molde); cláusula que nenhuma busca distingue SAI do código (F7d tirou 2).
  · ⛔ Uma excepção que depende de IGUALDADE de números entre dois sítios (o tamanho da ponta da seta
    contra o da tool) cai no primeiro arredondamento — o dono apanhou-a no smoke. Cure com uma LEI
    (separar o que se mostra do que se arma), não com uma comparação.
  · ⛔ A FOTO apanha o que nenhum gate vê (o gelado nascia com seta): fotografe ANTES de declarar, e
    olhe ampliado o traço que vem DEPOIS do que mudou.
  · ⛔ O instrumento mente antes da lei: a mão sintética aplicava o repouso DEPOIS do estabilizador (75
    traços «mudavam»; eram 3). O lápis filtra na ENTRADA. Confira o modelo da mão antes de afinar números.
  · Uma família nova nas sondas muda as faixas de TODAS as constantes: acrescente as famílias ANTES de
    escolher os meios, e meça as constantes JUNTAS (as grelhas `measure_arrow_angles`/`_joint`).
  · O `cargo fmt` parte linhas: âncoras por script têm de ser o texto FORMATADO (`grep -F` antes). O
    `Write` recusa um ficheiro que mudou desde a leitura: leia-o antes. `*_tests.rs` irmão não é isento do
    tecto de 700. A shell só encolhe. `pkill -f` mata a própria shell. Heredoc com `<<'EOF'`.

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 0.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação · handoff NOVO em
docs/UI_New_and_Simple/handoffs/ (o de F7d cita-se) · CLAUDE.md §5.1 «Último:» (troque só o link;
a entrada UI/UX tem ~690 de 700 bytes) · rm -rf target/*/incremental · binário smoke 2× · smoke ao
dono em passos numerados, sem jargão. Você NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
