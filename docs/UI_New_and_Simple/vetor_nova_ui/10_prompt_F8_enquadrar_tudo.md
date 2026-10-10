═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F8: «ENQUADRAR TUDO» (o desenho inteiro cabe na tela, num gesto)
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado em 10/10 (fim da janela F7e): line/UIUX = HEAD desta janela, base (merge-base) = 2a925859d =
main; F0–F7e NÃO integradas (~110 commits à frente; o dono ainda não pediu integração) ⇒
`git cherry main HEAD` dá >0 e os commits são DESTA linha: siga sem reset. Se o main tiver andado:
`git rebase main` (conflito fora dos seus ficheiros = PARE e reporte ao Enio).

O PEDIDO DO DONO (10/10, depois do smoke OK do Edit Shape): de quatro opções escolheu «enquadrar
tudo» — um gesto/botão que encaixa o desenho inteiro na tela (no iPad, o beliscão rápido do
Procreate). Hoje não existe em lado nenhum (procurado na F6: `fit`, `frame_all`, `enquadr`,
`zoom_to_fit`, `reset_view`, `home_view`, `center_view`, `NamedKey::Home` — e de novo a 10/10).

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7e_EDIT_SHAPE.md (§1, §7)
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F6_NAVEGAR_COM_OS_DEDOS.md
    (§1, §2 e §8 — a câmara, a roda que zooma ao CENTRO, a pinça aos dedos, a mola do zoom
    `canvas_zoom`, o reconhecedor de toque e a cena 11; «quick-pinch encaixa no ecrã» ficou como
    pergunta: é ESTA tarefa)
  docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md §4 (F6) e §6 (decisão 3: o padrão de teclas é o
    do ILLUSTRATOR) · referencias/R4_apps_de_ipad.md (o Procreate: o beliscão rápido)

O QUE JÁ EXISTE (endereços de 10/10 — confira, não reconstrua):
  • A câmara do editor: `Camera2d` (`center`, `height_world`, sem ângulo); o zoom anima por mola
    (`canvas_zoom`, `Surface`) — enquadrar deve ANDAR pela mesma mola, não saltar.
  • ⭐ O precedente de enquadrar: abrir um componente para editar ENQUADRA a câmara na raiz dele
    (`crates/ph2d-app-components/src/master_editing.rs` ~L100–135 e `prefab_framing`). Leia-o antes
    de escrever uma 2.ª conta de «caixa → câmara» (uma lei, uma porta).
  • A pinça e os dedos: o reconhecedor de toque da F6 (`feed_synthetic` entra pela MESMA porta de um
    dedo real) — o beliscão rápido é um gesto NOVO sobre ele (um gesto novo herda os inimigos do
    antigo: a pinça lenta não pode passar a enquadrar sem querer).
  • O menu *Ver* (Window/View) e os perfis de teclas: o dono escolheu o do ILLUSTRATOR.

ESCOPO — produto final:
  0. ⭐ O ORÁCULO PRIMEIRO (CLAUDE.md §0.9). Corridos por script, sem janela, com o harness que a F7e
     deixou (Playwright): o tldraw 5.5.2 (`~/Apps/tldraw-oracle`, «tldraw license»: corre-se, não se
     porta — `editor.zoomToFit()`, `zoomToSelection()`, e o atalho dele) e o Excalidraw `next`
     (`~/Apps/excalidraw-oracle-next`, MIT — o «zoom to fit» dele, `scrollToContent` com ajuste).
     Grave fixtures com cabeçalho sobre cenas NOSSAS (um objecto pequeno sozinho; vários espalhados;
     um muito comprido; a cena vazia; com seleção): a MARGEM em volta, o zoom MÁXIMO (um ponto
     sozinho não pode encher a tela), o que fazem sem nada, se animam e quanto tempo. O Procreate e o
     Illustrator pelo manual (o beliscão rápido; *Fit Artboard* / *Fit All*). Os corredores da F7e
     são o molde: `oraculo_quickshape/corre_excalidraw_edita.mjs`, `corre_tldraw_edita.mjs`.
  1. ⭐ A LEI: o que é «tudo» (os objectos visíveis da cena? escondidos e trancados contam? o
     documento/folha?); e «a seleção» à parte (o *Frame Selected*). A margem, o zoom máximo e o
     mínimo, a cena vazia — cada constante com a FAIXA medida, no MEIO; nenhum `MAX_*` sem a tabela.
  2. AS PORTAS: menu *Ver* ▸ «Enquadrar tudo» / «Enquadrar seleção» (a porta visível); as teclas do
     perfil ILLUSTRATOR (confira no perfil o que o Illustrator usa — proponha, ver abaixo); no toque,
     o beliscão RÁPIDO (meça a fronteira rápido × lento contra a pinça da F6, com a mão sintética,
     e prove que a pinça lenta continua a zoomar). Undo: enquadrar NÃO é um passo do documento.
     Gates de costura (o evento real → a câmara).
  3. Uma cena NOVA (conte o número no roteador `vector_mode.rs`; a próxima livre deve ser 14 — os
     estágios internos 13–16 da cena 7 NÃO são valores da variável): objectos espalhados e fora da
     tela, o «Enquadrar tudo» pelo menu → tudo à vista com a margem; FOTOGRAFE
     (`docs/Components/ferramentas/fotografa_cena.sh`) — e o primeiro quadro DEPOIS da mola parar.
  4. As sondas e os gates; prova de mutação de cada cláusula nova (agente `mutacao`), gate batched
     (`verificador`).
  FORA: rodar a vista; customizar a barra; integrar.

DECISÕES QUE NÃO SÃO SUAS: tecla nova (proponha a do perfil Illustrator e pergunte no relatório);
um BOTÃO novo em barra/painel (o menu *Ver* é o lugar óbvio; outra casa — proponha com foto);
instalar programas.

DECISÕES DO DONO JÁ TOMADAS (10/10, não re-pergunte): o botão Edit Shape fica junto da forma; arrastar
o contorno no Edit Shape AUMENTA a forma; a ponta de seta por omissão é o V aberto.

LIÇÕES DESTA LINHA (F7–F7e — não repita):
  · ⛔ Um gate que LÊ A SHELL pelo texto prende o código ao sítio: na F7e o press do lápis mudou-se
    para a crate e `the_pencil_owns_its_whole_gesture` ficou vermelho sobre código certo. Ao mover,
    `git grep -n 'shells/desktop/src' -- crates tools` e faça o gate SEGUIR o código.
  · ⛔ COMMIT os teus docs ANTES de lançar o `mutacao` (ele confere `git status` vazio a cada passo:
    rascunhos vivem no scratchpad). Sobrevivente sem testemunha: BUSQUE-a; cláusula que nenhuma busca
    distingue SAI do código (F7e tirou 2).
  · ⛔ O instrumento mente antes da lei: o corredor do tldraw arredondava a rotação a 1e-3 rad e um
    gate de 0,01 px ficou vermelho. Grave os números do oráculo INTEIROS.
  · A ordem do quadro decide o que se lê (assenta → afins → caneta → quadro do lápis): confira-a antes
    de ler um estado que outra fase escreve.
  · O guarda de recursos recusa um comando cujo TEXTO cita `cargo`/`nextest` (até um heredoc de
    docs): escreva o texto por `Edit`/`Write`, não por `python3 - <<EOF` na linha de comando.
  · O `cargo fmt` parte linhas: âncoras por script são o texto FORMATADO (`grep -F` antes). A shell só
    encolhe. `*_tests.rs` irmão não é isento do tecto de 700. `ph2d-vec-edit/src/lib.rs` está a 699.

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 0.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação · handoff NOVO em
docs/UI_New_and_Simple/handoffs/ (o da F7e cita-se) · CLAUDE.md §5.1 «Último:» (troque o link; a
entrada UI/UX tem 686 de 700 bytes; o smoke é `…|6..13` → `…|6..14`) · rm -rf target/*/incremental ·
binário smoke 2× · smoke ao dono em passos numerados, sem jargão. Você NÃO integra e NÃO pusha:
entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
