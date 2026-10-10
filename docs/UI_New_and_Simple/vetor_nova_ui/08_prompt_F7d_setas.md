═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F7d: QUICKSHAPE COM SETAS (desenhar a seta e SEGURAR = seta perfeita)
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado em 10/10 (fim da janela F7c): line/UIUX = HEAD desta janela, base
(merge-base) = 2a925859d = main; F0–F7c NÃO integradas (~95 commits à frente;
o dono disse «ainda não vamos integrar»). ⇒ `git cherry main HEAD` dá >0 e os
commits são DESTA linha: siga sem reset. Se o main tiver andado: `git rebase main`
(conflito fora dos seus ficheiros = PARE e reporte ao Enio).

O PEDIDO DO DONO (10/10, depois do smoke OK dos arcos — simétricos e assimétricos): escolheu, de
quatro opções, «SETAS»: desenhar uma linha ou um arco com um bico no fim e SEGURAR ⇒ uma seta
perfeita.

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7c_QUICKSHAPE_ARCOS.md
    (§1, §1b, §2, §7 — o arco, o biarco, a escolha «círculo primeiro», o que ficou aberto)
  docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/00_resultados.md §3c–§3g.2 (a lei, as
    FAIXAS de cada constante e as ⛔ recusas medidas — não reconstrua o que lá está recusado)

O QUE JÁ EXISTE (endereços de 10/10 — confira, não reconstrua):
  • A PONTA DE SETA já existe no motor e segue a TANGENTE das cúbicas: `enum Marker`
    (crates/ph2d-vec-scene/src/marker.rs:63 — None, Triangle, Open, Diamond, DiamondOpen, Circle,
    CircleOpen, Bar), pendurada no traço: `StrokeSpec.marker_start/marker_end/marker_scale/
    marker_round` (stroke_style.rs:238–273), `VecPath.stroke`. A geometria: `stroke_head`
    (marker.rs:543) com `end_tangent` (:504); gates em marker_tests.rs (ex.
    `the_head_follows_the_curves_tangent_not_the_chord`). O `PenStyle` do lápis tem os mesmos campos
    (ph2d-vec-edit/src/pen_support.rs:23–71). O painel Stroke já os mostra (Start/End/Head Size/Head
    Round/Both Ends). ⇒ a seta perfeita é o CABO do QuickShape (linha, arco, biarco, polilinha) com
    `marker_end` (e `marker_start` se tiver duas pontas) — nunca uma geometria de ponta nova.
  • A lei: crates/ph2d-vec-edit/src/quickshape.rs (678 linhas — tecto 700 ⇒ a seta vai para um
    IRMÃO, ex. `quickshape_arrow.rs`); `recognise_with(law, …)`, `Law::MEASURED` (as constantes que as
    sondas VARREM no mesmo processo). Arco: quickshape_arc.rs (`ArcShape`, `fit`, `polyline_rms`),
    quickshape_biarc.rs, quickshape_conic.rs. Gates: quickshape_tests.rs (65 fixtures do Xournal++;
    `own()` = 27 excepções do dono, contadas), quickshape_arc_tests.rs, quickshape_pencil_tests.rs
    (o gatilho; `draw` partilhado). Sondas: quickshape_measure_tests.rs (`cases()`),
    quickshape_sensitivity_tests.rs, quickshape_arc_probe_tests.rs. Mão sintética: src/test_hand.rs.
  • O gatilho: crates/ph2d-vec-edit/src/pencil.rs — `rebuild` (:294–311) reescreve SÓ `verts` e
    `closed` do caminho vivo; o estilo entra em `build` (`self.style.stroke_spec(…)`). ⚠️ A seta
    precisa de pôr a ponta no traço vivo E de a TIRAR no `Esc` (o traço livre volta sem ponta), no
    `regular` que muda a meio e quando a forma deixa de ser seta — gate de cada um.
  • Cena 12: crates/ph2d-app-vec/src/vector_mode_quick.rs — 6 traços SEGUROS + o gelado livre;
    `PX_TO_WORLD = 0,0103` (medido na foto). O gancho do dono vive em `quickshape::OWNERS_HOOK_PX`.

ESCOPO — produto final:
  0. ⭐ O ORÁCULO PRIMEIRO (CLAUDE.md §0.9): o Xournal++ está instalado e o corredor existe
     (oraculo_quickshape/corre_oraculo.sh, gera_entradas.py — acrescente ao FIM, sem `rng`, como a 2.ª
     leva). Corra-o sobre setas nossas (cabo recto e curvo; ponta em V aberto, em V fechado, só uma
     farpa; duas pontas) e grave as fixtures com cabeçalho. Ele reconhece setas? Se não, diga-o com a
     fixture, e a lei é decisão técnica documentada (o Procreate não corre aqui; o produto de
     referência do dono manda — memória `feedback_the_owners_reference_product_decides_the_law`).
     Procure também outro app INSTALADO que reconheça setas (triagem de licença por artefacto;
     `docs/_ComoInvestigarApps/`) antes de inventar a lei.
  1. ⭐ A LEI da seta, num traço SÓ (o lápis é um gesto: a mão não levanta). Decida e MEÇA que gestos
     contam: o cabo e depois o V (vai a uma farpa, volta à ponta, vai à outra), o V de ida-e-volta, e
     ponta(s) nas duas extremidades. A ponta sai de cima do traço (o cabo é o resto, reconhecido pela
     lei que já existe — linha, arco, biarco, polilinha — e a forma do cabo nunca piora por haver
     ponta); o tamanho da ponta vem das farpas desenhadas (`marker_scale`). Fronteiras que a sonda
     tem de mostrar: um gancho de mão no fim de uma linha (o «gancho» que a lei já absorve como quina)
     NÃO é seta; um zigue-zague, um L, um V e o «taco» continuam o que eram; setas de 20–400 px, farpas
     de 20–60°, tremor ±0,5–2 px, o estabilizador. Cada constante nova com a FAIXA no doc-comment,
     e no MEIO dela. `regular`: o cabo regulariza como hoje e a ponta fica (decida; justifique).
  2. O GESTO: arrastar escala e roda a seta inteira (cabo + ponta); soltar comete com nós; `Esc`
     devolve o traço livre SEM ponta. Gates.
  3. Cena 12: uma seta SEGURA (cabo curvo, de preferência) antes do gelado livre; FOTOGRAFE
     (docs/Components/ferramentas/fotografa_cena.sh).
  4. As sondas e os gates; prova de mutação de cada cláusula nova (agente `mutacao`), gate batched
     (`verificador`).
  FORA: «Edit Shape»; formatos de ponta novos; setas desenhadas em DOIS traços (a mão levanta).

DECISÕES QUE NÃO SÃO SUAS: tecla nova; instalar programas; o formato da ponta por omissão SE o
oráculo não o disser (proponha um — `Triangle` é o «Arrow» do painel — e pergunte no relatório).

LIÇÕES DESTA LINHA (F7–F7c — não repita):
  · ⛔ COMMIT os teus docs ANTES de lançar o agente `mutacao`: ele viu um doc modificado, achou que um
    teste o tinha escrito e fez `git checkout` — a §3g.2 perdeu-se e foi reescrita (10/10). E diga-lhe
    no prompt para NÃO restaurar nada fora das mutações.
  · Um modelo com mais parâmetros SEGUE o tremor lento da mão: o biarco fazia biarco do arco simétrico
    que «respira» (o da própria cena — o gate da cena apanhou-o). Teste sempre a família «a mão que o
    dono aprovou» (o que ele já testou e disse OK) contra a lei nova, e meça as constantes JUNTAS.
  · Sobrevivente de mutação sem testemunha óbvia: BUSQUE-a — base × mutação sobre algumas centenas de
    traços candidatos, compare as saídas (achou 6 de 7 em 10/10). Cláusula que nenhuma busca distingue
    é redundante: tire-a do código, não a teste.
  · O `cargo fmt` parte linhas: âncoras de mutação/edição por script têm de ser o texto FORMATADO
    (`grep -F` antes). No zsh, uma lista de ficheiros numa variável expande-se com `${=F}`.
  · Uma constante na BORDA da faixa é uma queixa à espera — no MEIO. `*_tests.rs` irmão não é isento do
    tecto de 700. A régua do gate não pode dar `panic`. FOTOGRAFE antes de declarar. Commit antes de
    toda mutação. A shell só encolhe. `pkill -f` mata a própria shell. Heredoc com `<<'EOF'`.

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 0.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação · handoff NOVO em
docs/UI_New_and_Simple/handoffs/ (o de F7c cita-se) · CLAUDE.md §5.1 «Último:» (troque só o link;
a entrada UI/UX tem ~690 de 700 bytes) · rm -rf target/*/incremental · binário smoke 2× · smoke ao
dono em passos numerados, sem jargão. Você NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
