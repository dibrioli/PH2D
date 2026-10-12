═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F12, A ÚLTIMA JANELA: A ESPIRAL (E), AS CENAS E AS FOTOS (G) E O FECHO
        DA LINHA (F9–F12 JUNTAS) — ATÉ O SMOKE DO DONO
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie). Modo L.

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: FASE 0 → 1 → 2 do
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com MODULO=UIUX.
Estado em 12/10: F0–F8 no `main`; F9 + F10 + F11 fechadas na linha e NÃO integradas; F12: A (o
traço), B, C, D e F FEITAS (`222f4630b1`, `d4fa902bf6`, `1d301208c0`); E, G e o fecho por fazer.
Siga SEM reset; se o `main` andou: `git rebase main` (a linha foi rebaseada sobre `9ea782138`).

LEIA, NESTA ORDEM (e só isto antes de agir):
  1. docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-12_F12c.md — «O que
     FALTA» (itens 1–5) é o seu ponto de partida exacto; «Perguntas ao dono» vão no relatório final.
  2. docs/UI_New_and_Simple/vetor_nova_ui/18_prompt_F12_sem_modais_e_o_traco.md — o pedido original
     do dono palavra por palavra, as DECISÕES QUE NÃO SÃO SUAS e as JÁ DECIDIDAS (continua a valer).
  3. As regras R1–R9 do topo de docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md
     (R1 = ~150 chamadas ⇒ commit + handoff de continuação ≤ 8 KB + PARE).

O PEDIDO DO DONO (o que esta janela fecha — palavra por palavra):
 *«O gzimo do espiral não funciona. Para expansion e Inner crie setas e tire o modal.»* (11/10)
 *«eu havia pedido que os ajustes que atualmente estão nos modais passassem para setas lado a lado
  (setas da imagem — a mesma que ajusta as pontas das estrelas) e que os modais fossem retirados de
  todas as formas. Então antes de integrar as linhas vector vamos fazer isso.»* (12/10)
 ⇒ As setas e a saída do cartão estão FEITAS. Falta a espiral, mostrar tudo em cenas fotografadas, e
  fechar a linha para o smoke do dono (F11 + F12 juntas). Depois do smoke dele vem a integração das
  DUAS linhas do Vetor — NÃO é sua.

O QUE JÁ EXISTE (confira no código antes de acreditar):
  • A fila de ⇕: `ph2d-tool-vector/src/shape_handles.rs` (`Spot::Rail`, `rail_slot`, `RAIL_PITCH`,
    `Law::Steps` com o ramo contínuo/`step_world`), a tabela em `shape_handles_table.rs` (`arrow`),
    os gates em `shape_handles_f12_tests.rs`; o censo `the_fields_without_a_gesture_are_counted_not_listed`
    está em ZERO para todas as formas.
  • O *Smoothing* do rectângulo fora do Vetor: `shapes_units::shown_fields`/`NOT_SHOWN` (o slot 4
    fica no documento — o Motion escreve-o; nenhuma migração, `PROJECT_SCHEMA` = 188).
  • Círculo/pizza: o furo de volta e o campo novo `Corner round` (`ph2d_vec_scene::round_slice_corners`).
  • O cartão SAIU (`d4fa902bf6`); `selection_gap()` mora em `editor-core/.../float_bar.rs`.
  • A etiqueta da alça sob o cursor (`ph2d-app-vec/src/shape_handles_live.rs::readout`).
  • As fotos do dono: /var/tmp/ph2d_F12_dono/ — a 7 é a SETA, o idioma de todos os ajustes.

ESCOPO — produto final, por esta ordem:
  E. ESPIRAL. Reproduza o «não funciona» pelo press/arrasto REAL da shell (como as cenas 15–17 — um
     estágio de cena que agarra a ponta de fora e arrasta), NÃO só pelo teste da lei. Hipótese do
     F12c item 2 (meça antes de acreditar): `Law::Winding` arredonda `wound/360` com *Turns* inteiro
     (1..8) — um arrasto curto não muda nada. O ORÁCULO (Inkscape, `shape_handles_f9_oracle_tests`,
     fixtures em `oraculo_alcas/`) dá `revolution` FRACCIONÁRIA (3,000 → 3,250 a +90°). Se a cura
     for tornar *Turns* contínuo: confirme que o `cook` da espiral aceita fracção, que um save antigo
     (inteiro) cozinha igual, e reescreva os gates da lei e do oráculo para a fracção (sem `round`).
     Os ⇕ de *Expansion* e *Inner* já existem — verifique na foto que funcionam.
  G. CENAS. (1) A cena 16 (`ph2d-app-vec/src/vector_mode_card.rs`, estágios 80–86) ensina o CARTÃO
     que já não existe — refaça-a para a fila de ⇕ (e renomeie o ficheiro; «cena que ensina o
     contrário do que acontece é pior que cena ausente», CLAUDE.md §5.0). (2) Refaça as 15–17 se
     mudaram. (3) Cena NOVA `=19` (a contagem vive em `vector_mode.rs`): várias formas com as setas
     lado a lado — polígono (ponto da envergadura + 4 ⇕), estrela (5 ⇕), pizza (furo + ⊙ das
     quinas a arredondar), espiral (a ponta a seguir a mão + 2 ⇕); um ⇕ arrastado com a etiqueta à
     vista. (4) FOTOGRAFE TODAS (`docs/Components/ferramentas/fotografa_cena.sh`; ⛔ nunca
     `spectacle` em `kwin --virtual` — `import -window`), compare com a foto 7 do dono e confirme
     POR FOTO que nenhum cartão aparece. Nenhuma seta por cima de uma alça de escala nem da ④ —
     se a 1.ª seta (na borda, a ¼ do topo) tocar a alça de canto numa caixa baixa, meça e diga-o.
  FECHO (DIRETRIZ §1.5.9, nesta ordem):
     1. Mutação de cada cláusula nova (`mutacao`, ANTES do portão — nunca ao mesmo tempo): as do F12b
        item 5 e as do F12c item 5 (`RAIL_PITCH`/`rail_slot`, o ramo contínuo do `Steps`,
        `NOT_SHOWN`, `round_slice_corners`, o `corner_at` que esconde a quina, o `hot` dos pontos
        desenhados, o ramo `hover` do `readout`) e as da cura da espiral. Cláusula que sobrevive
        ganha teste.
     2. Portão batched 1× sobre o diff acumulado (`verificador`): `scripts/nextest-impacted.sh` +
        clippy `--all-targets` do `ship.sh` + fmt + `the_shell_only_shrinks` + LOC.
     3. Handoff de INTEGRAÇÃO novo em docs/UI_New_and_Simple/handoffs/ (F9–F12 juntas; cita o da
        F11 — `HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F11_GIZMOS.md`): o que mudou, os números do
        portão, as mutações, e o que fica aberto. Avisos ao integrador: o campo novo apenso
        `Corner round` (Ellipse/Pie, slot 3), o slot 4 do RoundRect escondido mas vivo, a remoção de
        `ToolBars::card` (editor-core), e que `PROJECT_SCHEMA` NÃO mudou.
     4. CLAUDE.md §5.1 da UI/UX: tirar «o cartão lateral (D10)», pôr a fila de ⇕, smoke `6..19`,
        «Último:» → o handoff novo; ≤ 700 bytes (gate). `bash scripts/doc-index.sh`.
     5. `bash scripts/agent-loop-profile.sh` · `rm -rf target/*/incremental`.
     6. Binário smoke 2× (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile
        smoke`), a 2.ª saída no handoff («Finished» em segundos, zero «Compiling»).
     7. O SMOKE ao dono — curto, SEM jargão, em PASSOS NUMERADOS (CLAUDE.md §0.8): (1) o comando
        completo com o `cd` · (2) onde clicar, com o nome que aparece na tela · (3) o que tem de
        acontecer · (4) como saber que deu errado. Cubra as cenas 15–17 (F11) e 16 refeita, 18 e 19
        (F12), e desenhar à mão: polígono, estrela, pizza, espiral, retângulo arredondado.
  FORA: a estrela (aprovada — só herdou setas); integrar; enviar; a placa GPU do Motion
  (`contorno_cpu.rs`, só depois de o dono aprovar o bico cortado).

AINDA SEM RESPOSTA DO DONO (pergunte de novo no relatório final, com a sua recomendação; NÃO decida,
NÃO bloqueie):
  • O bico cortado (doc 19 §5, opções a/b/c; fotos em /var/tmp/ph2d_F12_cena18/).
  • A F11: a *Resize Box* mantém a espessura do traço (*Scale Strokes & Effects*)?
  • O ⇕ do *Viewed* dos isométricos: subir dá «De baixo» — recomendação: inverter só ele.
  • O retângulo gravado antes com *Smoothing* continua suave sem controlo — aceitar (recomendado)
    ou zerar ao abrir.

DECISÕES QUE NÃO SÃO SUAS: tirar qualquer outro parâmetro; mudar o limite do miter de todo traço sem o
antes × depois ao dono; a cor das alças fora dos tokens; instalar programas; integrar; pushar.
JÁ DECIDIDAS (não re-pergunte): alças só no modo Forma; o press nas alças ganha a desenhar; a fila ②
fica com todos os números; as setas ⇕ no idioma da estrela, lado a lado na lateral; o cartão saiu.

COMECE AUTOMATICAMENTE: FASE 0–2, reporte ao Enio em 3 linhas (sem jargão) e SIGA com a E.
Se a R1 chegar antes do fim: commit + handoff de continuação ≤ 8 KB (o que falta, por ordem) + PARE.
═══════════════════════════════════════════════════════════════════
