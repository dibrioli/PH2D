═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F7c: QUICKSHAPE COM ARCOS (desenhar um arco e SEGURAR = arco perfeito)
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado em 10/10 (fim da janela F7/F7b): line/UIUX = HEAD desta janela, base
(merge-base) = 2a925859d = main; F0–F7b NÃO integradas (~70 commits à frente).
⇒ `git cherry main HEAD` dá >0 e os commits são DESTA linha: siga sem reset.
Se o main tiver andado: `git rebase main` (conflito fora dos seus ficheiros =
PARE e reporte ao Enio).

O PEDIDO DO DONO (10/10, depois do smoke OK da F7b): «QuickShape com arcos» — hoje
um arco desenhado à mão e SEGURO fica traço livre; no Procreate (o produto de
referência dele, R4 L165–169) vira um ARCO perfeito.

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-09_F7b_QUICKSHAPE_SENSIBILIDADE.md
    (§1–§1d: formas pequenas, sensibilidade, o lápis com quinas, a estrela, a linha aberta)
  docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/00_resultados.md §3c–§3f (a lei, as
    sondas, as FAIXAS de cada constante — ⛔ não reconstrua o que lá está RECUSADO e medido)
  O handoff de F7 (08/10) só se precisar da origem: HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F7_QUICKSHAPE.md

O QUE JÁ EXISTE (endereços de 10/10 — confira, não reconstrua):
  • A lei: crates/ph2d-vec-edit/src/quickshape.rs (614 linhas, tecto 700 ⇒ o arco vai para um
    IRMÃO, ex. `quickshape_arc.rs`). `enum QuickShape` (:106) = Line · Polygon{corners,closed} ·
    Ellipse{centre,rx,ry,rot}; `pivot()` (início da linha/polilinha, centro do fechado),
    `transformed()`, `to_path()`. As constantes num `Law` (`Law::MEASURED`) que as sondas VARREM no
    mesmo processo; `recognise_with(law, …)`. O fluxo: reamostra → RDP → funde viragens → absorve
    lados curtos em QUINAS-FAIXA → se todos os lados são rectos, polígono/linha/polilinha; senão, SE
    FECHADO, elipse dos momentos de área; senão `None` ← é aqui que o arco entra (aberto + curvo).
  • Geometria pura: quickshape_geom.rs (tls, intersect, resample, rdp, solve<N>, snap…).
  • O construtor de arco JÁ existe: `ph2d_vec_scene::arc(center, rx, ry, degrees)`
    (crates/ph2d-vec-scene/src/shapes.rs:351 — segmentos ≤ 90° com handle (4/3)·tan(α/4)·r). Ele
    começa num ângulo fixo: o arco reconhecido precisa de início e sentido próprios ⇒ rode/espelhe
    os vértices como o `to_path` da elipse já faz, ou escreva o caminho com a mesma lei.
  • Gates: quickshape_tests.rs (as 65 fixtures do Xournal++; `own()` = as excepções do DONO, hoje 25,
    contadas em `every_own_law_exception_names_a_fixture_that_exists`), quickshape_hand_tests.rs,
    quickshape_pencil_tests.rs (o gatilho). Sondas: quickshape_measure_tests.rs (418) e
    quickshape_sensitivity_tests.rs (380) — `cases()` tem «meio arco (não é forma)» ⇒ PASSA a ser
    «arco»; `measure_open_gaps` tem «C ¾» como forma ABERTA ⇒ passa a dever virar ARCO (aberto),
    nunca círculo fechado. A mão sintética partilhada: src/test_hand.rs (`hand`, `stabilise`).
  • O oráculo: o Xournal++ deixa os arcos CRUS (fixtures `arco_meio`, `arco_quarto`) ⇒ como as
    elipses, é LEI DO DONO: excepção em `own()` com a geometria do gerador (gera_entradas.py:138–139:
    centro (800,500), raio 200, de 180° a 360° e de 180° a 270°) e o motivo no nome (25 → 27).
  • Cena 12: crates/ph2d-app-vec/src/vector_mode_quick.rs — 4 traços SEGUROS + 1 livre (o gelado);
    `PX_TO_WORLD = 0,0103` (MEDIDO na foto). Fotos: docs/…/vetor_nova_ui/fotos/f7b_0{1,2}.

ESCOPO — produto final:
  1. ⭐ A LEI do arco: `QuickShape::Arc { centre, radius, start, sweep }` (circular; ângulos em
     radianos, `sweep` com SINAL = o sentido do desenho). Reconhece um traço ABERTO, CURVO (os lados
     não passam a régua do recto), de curvatura de um só sinal, que um círculo ajusta bem (ajuste de
     círculo por mínimos quadrados — Kåsa/Pratt, algoritmo publicado: PORTE-O). Nada de inventar
     limites: varra cada constante nova com as mãos sintéticas (tremor ±0,5–2 px, o estabilizador)
     e escreva a FAIXA no doc-comment, como as outras. Fronteiras que a sonda tem de mostrar:
       – um arco muito raso continua LINHA (o oráculo endireita a flecha de 3,3 %, `linha_arco_leve`);
       – um S (inflexão), um zigue-zague e um rabisco continuam NÃO-forma;
       – um arco que quase fecha (o C a que falta pouco) — com a folga de 13 % ele FECHA em círculo;
         acima dela é arco; uma volta e meia (`circulo_volta_e_meia`) continua crua;
       – arcos de 30° a 330°, raios 20–400 px.
     `regular` (Shift / 2.º dedo): DECIDA tecnicamente e justifique no handoff (ex.: o ângulo varrido
     em múltiplos de 15°, ou meia-volta/quarto) — o Procreate faz alguma coisa aqui; se não a puder
     medir, diga-o.
  2. O GESTO: já funciona para qualquer `QuickShape` (o pivô de uma forma aberta é o INÍCIO; arrastar
     escala e roda à volta dele; soltar comete a forma com nós). Confirme por gate que o arco segue o
     mesmo caminho (to_path com nós `Smooth` no meio, `Corner` nas pontas, `closed: false`).
  3. Cena 12: acrescente um arco SEGURO (antes do gelado livre) e FOTOGRAFE (fotografa_cena.sh).
  4. As sondas e os gates actualizados (S, rabisco, meio arco, C ¾); prova de mutação de cada
     cláusula nova (agente `mutacao`), gate batched (`verificador`).
  FORA: setas; elipses ABERTAS (arcos de elipse — só se a medição mostrar que o círculo não chega);
  «Edit Shape».

DECISÕES QUE NÃO SÃO SUAS: tecla nova; instalar programas.

LIÇÕES DESTA LINHA (F7/F7b — não repita):
  · Uma constante na BORDA da sua faixa é uma queixa à espera: ponha-a no MEIO (a «pouca
    sensibilidade» do dono era a elipse a 0,04 numa faixa 0,02–0,11).
  · Sonda de mãos SINTÉTICAS (40–80 por célula) mede o que as fixtures não perguntam — achou o S que
    virava zigue-zague e a estrela que perdia uma ponta. Com 24 mãos a diferença de 1 mão é ruído.
  · Uma comparação de RECTAS (mod π) onde importa o SENTIDO é um defeito à espera (a estrela).
  · Um filtro de região corre sobre os VENCEDORES do máximo local, nunca antes (o gancho do lápis).
  · `*_tests.rs` irmão NÃO é isento do tecto de 700 LOC (já mordeu DUAS vezes nesta linha).
  · A régua do gate (`worst`) não pode dar `panic`: a sonda que a usa morre.
  · ⛔ zsh: um heredoc SEM aspas (`<<EOF`) executa as crases do texto — use `<<'EOF'`.
  · FOTOGRAFE antes de declarar · commit antes de toda mutação · a shell só encolhe · `pkill -f`
    mata a própria shell · o guarda lê o TEXTO do comando (scripts no scratchpad).

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 1.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação · handoff NOVO em
docs/UI_New_and_Simple/handoffs/ (o de F7b cita-se) · CLAUDE.md §5.1 «Último:» (a entrada UI/UX está
a 698 de 700 bytes — troque só o link) · rm -rf target/*/incremental · binário smoke 2× · smoke ao
dono em passos numerados, sem jargão. Você NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
