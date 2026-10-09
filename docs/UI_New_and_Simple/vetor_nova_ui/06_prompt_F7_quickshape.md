═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F7: o QUICKSHAPE (desenhar e SEGURAR = forma perfeita com nós)
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado em 08/10 (fim da janela F6): line/UIUX = HEAD desta janela, base
(merge-base) = 2a925859d = main; F0–F6 NÃO integradas (~50 commits à frente).
⇒ `git cherry main HEAD` dá >0 e os commits são DESTA linha: siga sem reset.
Se o main tiver andado: `git rebase main` (conflito fora dos seus ficheiros =
PARE e reporte ao Enio).

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F6_NAVEGAR_COM_OS_DEDOS.md
  ⭐ docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/00_resultados.md — o ORÁCULO JÁ
    CORREU (o dono autorizou instalar o Xournal++; 41 fixtures em fixtures/; as leis na §3,
    o que o Procreate manda a mais na §4). NÃO reconstrua o roteiro: corra-o
    (`bash corre_oraculo.sh`, ~4 min) só se mudar o corpus de entradas/.
  docs/UI_New_and_Simple/vetor_nova_ui/referencias/R4_apps_de_ipad.md L165–169 (QuickShape do
    Procreate) e L205 (Concepts): o PRODUTO do dono é o Procreate — gatilho = SEGURAR.

O QUE JÁ EXISTE (endereços medidos pelo explorador em 08/10 — confira, não reconstrua):
  • O lápis livre é `ph2d-vec-edit::Pencil` (crates/ph2d-vec-edit/src/pencil.rs:72): `samples:
    Vec<[f64;2]>` (mundo), `dyns: Vec<PenDynamics>` com `t_ns` (relógio da shell, só anda quando o
    ponteiro ANDA — `on_drag` ignora < 1,5 px) ⇒ «parado» = `now − dyns.last().t_ns`. `on_press`
    :130 · `on_drag` :156 · `on_release` :173 · `build()` ~:250 (RDP `decimate` :277 + Hobby
    `fit_hobby_open`, hobby.rs:75) ⇒ `VecPath{closed:false}`.
  • Chamadores na shell: premido despacho_clique_vetor_premido.rs:413 · arrasto
    despacho_metodos_picks_e_arrastos.rs:460 (`vec_pencil_drag_move`, com o estabilizador) ·
    solto despacho_clique_vetor_solto.rs:205 (selecciona com `vec.pen.select`, :211).
  • ⭐ O hook POR QUADRO com o lápis activo: shells/desktop/src/render_loop/
    fase_vector_live_geometry.rs:95 (`if let Some(id) = self.vec.pencil.active_path()`) — é AQUI
    que o «segurar» se mede (LONG_PRESS_NS = 0,5 s, o mesmo número do toque, interaction/touch.rs).
  • Construtores de forma: crates/ph2d-vec-scene/src/shapes.rs — `rectangle(a,b)` :27,
    `ellipse(center,rx,ry)` :45, `regular_polygon` :68, `line(a,b)` :331; `VecVertex::corner`
    (lib.rs:408) em laço com `closed:true` para um polígono de cantos. `ShapeKind` em kind.rs:38.
  • Nós editáveis: o Edit entra por `enter_with` (ph2d-app-vec/src/vector_mode.rs:226) e um
    `VecPath` de cantos JÁ é editável — não há nada a construir aqui.
  • Modificadores do toque: `TouchMods` (+1 dedo = Shift) já chegam à shell como `ModifiersState`
    (touch_input.rs `effective_modifiers`) ⇒ «2.º dedo = regular» é «Shift = regular», de graça.
  • i18n do vetor: crates/ph2d-i18n/src/vector.rs (649 linhas — PERTO DO TECTO de 700: uma chave
    nova cabe; três não). Convenção `panel.vector.pencil.<controlo>`; ids em
    crates/ph2d-tool-vector/src/ids/vector_pencil.rs (44 linhas).
  • Contrato Tool=12 / PanelEvent=4: NÃO precisa de mudar (medido: o hold cabe no Pencil + o hook
    por quadro). Precisou? PARE e reporte.

ESCOPO — produto final, não MVP:
  1. ⭐ A LEI, pura, em crates/ph2d-vec-edit/src/quickshape.rs: `recognise(samples: &[[f64;2]],
     px_to_world: f64, regular: bool) -> Option<QuickShape>` com `Line{a,b}` · `Polygon{corners,
     closed}` (rectângulo = 4 cantos a 90° com a rotação do traço; triângulo = 3 cantos COMO
     DESENHADOS — o oráculo não regulariza; `regular` = regulariza: quadrado, triângulo equilátero,
     polígono regular) · `Ellipse{centre, rx, ry, rot}` (círculo quando rx≈ry; a elipse é do
     Procreate, não do oráculo). Mínimo de tamanho e folga de fecho: os do oráculo (§3 do doc),
     convertidos para px de TELA (o oráculo mede em unidades de página a 177 %: 40 un. ≈ 94 px) —
     MEÇA o que uma mão faz no nosso app antes de cravar o número (uma constante inventada é uma
     medição que ninguém fez).
     GATES: um teste que lê TODAS as fixtures de oraculo_quickshape/fixtures/ (os `in` são a
     entrada, os `out` a resposta): tipo igual e geometria a ≤ 1 un. (cantos, extremos) / ≤ 1 % do
     raio (círculos); os «cru» ⇒ `None` — EXCEPTO elipse e polígonos ≥ 5 (lei do Procreate), com
     gate de auto-consistência e o motivo no nome. Controlo positivo: ≥ 26 fixtures reconhecidas.
     Prova de mutação de cada cláusula (agente `mutacao`).
  2. O GESTO: no hook por quadro, lápis activo + ponteiro parado ≥ 0,5 s + traço ainda não
     «quick» ⇒ `recognise` sobre os samples; se `Some`, a geometria VIVA passa a mostrar a forma
     (o traço cru some), e arrastar sem soltar ESCALA e RODA a forma (Procreate; com Shift a
     rotação anda em degraus de 15°); soltar comete a FORMA (não o Hobby) como `VecPath` com nós
     editáveis e selecciona-a; `Esc`/ou mexer antes dos 0,5 s = traço normal. Som de clique ao
     reconhecer (o que já existe). Undo = um passo (a forma substitui o traço).
  3. A PORTA VISÍVEL (R4: todo gesto com um botão ao lado): um interruptor na fila do Lápis
     «QuickShape» (ligado por omissão) — pelas 7 pontas do seam (DIRETIVA §2), com o id em
     vector_pencil.rs, a chave em vector.rs e os censos verdes; e a dica no rodapé ⑤ quando o
     lápis está na mão: «Segure no fim do traço: forma perfeita».
  4. Cena de smoke `PH2D_OBJECT_MODE_SMOKE=12` (em uso: 1|2|4|6|7|8|9|10|11 — conte nos `match`):
     lápis na mão, três traços SINTÉTICOS (rectângulo tosco, círculo tosco, linha trémula) cada um
     a segurar no fim ⇒ três formas perfeitas com nós; FOTOGRAFE (fotografa_cena.sh). O «segurar»
     sintético mede-se pelo relógio do store (`clock_ns`), como a cena 11.
  FORA: reconhecer setas/arcos (o oráculo não tem e o dono não pediu); customizar a barra.

DECISÕES QUE NÃO SÃO SUAS: qualquer tecla nova (gate `a_tecla_do_gatilho…`); instalar programas
(já está instalado o que faz falta).

LIÇÕES DESTA LINHA (F4–F6 — não repita): FOTOGRAFE antes de declarar · gate que publica uma vez
mede um app que não existe · o `App` só larga campos (`thread_local` na shell ou crate) · a shell
só encolhe (185 603 / 189 041 em 08/10) e `despacho_metodos_picks_e_arrastos.rs` está a 594 de 600
— o que puder ser lei vai para a crate · `vector.rs` (i18n) e `ph2d-vec-scene/lib.rs` a 649 e 663
de 700 · commit antes de toda mutação · o guarda lê o TEXTO do comando (script no scratchpad) ·
`pkill -f <texto>` mata a própria shell.

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 1.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação · handoff NOVO em
docs/UI_New_and_Simple/handoffs/ (o de F6 cita-se) · CLAUDE.md §5.1 «Último:» · rm -rf
target/*/incremental · binário smoke 2× · smoke ao dono em passos numerados, sem jargão.
Você NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
