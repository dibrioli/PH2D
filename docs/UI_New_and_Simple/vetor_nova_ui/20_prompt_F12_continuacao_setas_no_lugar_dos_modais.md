═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F12, CONTINUAÇÃO (sub-ondas B → G): NENHUMA FORMA ABRE O CARTÃO
        (O «MODAL»); CADA AJUSTE VIRA A SETA DAS PONTAS DA ESTRELA, LADO A LADO NA LATERAL
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie). Modo L.

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: FASE 0 → 1 → 2 do
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com MODULO=UIUX.
Estado em 12/10: F0–F8 no `main`; F9 + F10 + F11 FECHADAS na linha e NÃO integradas; F12: a sub-onda
A (o traço) e dois pedidos do dono FEITOS (`24c7f2ea7`, `fe21fef57e`, a cena 18, `90eebf7b38`,
`2bb2954b82`). Siga SEM reset; se o `main` andou: `git rebase main`.

LEIA, NESTA ORDEM (e só isto antes de agir):
  1. docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F12b.md — «O que FALTA»
     (itens 1–5) é o seu ponto de partida exacto.
  2. docs/UI_New_and_Simple/vetor_nova_ui/18_prompt_F12_sem_modais_e_o_traco.md — o pedido
     original, palavra por palavra, as sub-ondas B–G, as DECISÕES QUE NÃO SÃO SUAS e as JÁ DECIDIDAS.
     Continua a valer inteiro; este prompt só o reafirma e o ordena.
  3. As regras R1–R9 do topo de HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md (R1 = ~150 chamadas ⇒
     commit + handoff de continuação ≤ 8 KB + PARE).

O PEDIDO DO DONO, REAFIRMADO (12/10, ao rever a F11 — palavra por palavra):
 *«eu havia pedido que os ajustes que atualmente estão nos modais passassem para setas lado a lado
  (setas da imagem — a mesma que ajusta as pontas das estrelas) e que os modais fossem retirados de
  todas as formas. Então antes de integrar as linhas vector vamos fazer isso.»*
 A IMAGEM que ele mandou (a seta, o idioma de TODOS os ajustes):
  /var/tmp/ph2d_F12_dono/7_a_seta_das_pontas_da_estrela_o_idioma_de_todos_os_ajustes.png
  — o anel verde com as duas setas: é o ⇕ das pontas da estrela (`Spot::Box` + `Law::Steps`,
  `shape_handles_face::glyph_of`). Abra-a e compare com a cena 16 antes de desenhar qualquer seta.
 ⇒ É ISTO que fecha a F12 e destrava a integração das DUAS linhas do Vetor (a `line/Vector`, doc 50,
  está fechada com o smoke do dono OK e espera por esta). O dono ainda NÃO aprovou a F11 no smoke:
  o smoke do fim desta onda cobre F11 + F12 juntas.

ESCOPO — produto final, por esta ordem (cada sub-onda fotografada antes de passar):
  B. POLÍGONO — o ponto da curva da aresta de volta (`Spot::EdgeMid`, a linha que a F11 tirou em
     `21226cb088`); *Curve*, *Smooth* e o tipo do canto como ⇕ ao lado das Pontas (`Law::Steps` também
     para campos CONTÍNUOS: a altura da caixa = a faixa do campo). Detalhe no F12b item 3.
  C. RECTÂNGULO — o *Smoothing* SAI do produto (campo, fila ②, cartão; um campo que sai de um
     `ShapeValues` é foundational: confira o `PROJECT_SCHEMA`/migração ANTES); o *Corner* como ⇕.
  D. CÍRCULO e PIZZA — o furo central de volta no canvas; um ponto novo que arredonda as QUINAS
     quando o arco não é inteiro (parâmetro novo apenso e neutro em zero, se o `cook` não o tiver).
  E. ESPIRAL — reproduzir porque o gizmo «não funciona» pelo press/arrasto REAL da shell (como as
     cenas 15–17), curar; *Expansion* e *Inner* como ⇕.
  F. ⭐ TODAS as outras formas — CENSO primeiro: a lista de cada forma × cada campo que hoje só o
     cartão alcança (`shape_handles::fields_without_handle`) e o gate
     `the_fields_without_a_gesture_are_counted_not_listed` — esse número tem de chegar a ZERO. Cada
     campo vira um ⇕; os ⇕ ficam na LATERAL da caixa, UM AO LADO DO OUTRO (o dono), nunca por cima de
     uma alça de escala nem da ④; cada um mostra a ficha do nome durante o arrasto (`handle_readout`,
     como o «Points: 7»). No fim NENHUMA forma abre o cartão — confirme que o cartão SOME (não que
     fica vazio), por gate e por foto. Um campo que NÃO couber como seta (um enum sem ordem, uma
     cor) não se decide sozinho: liste-o e pergunte ao dono no relatório, com a sua recomendação.
  G. CENAS — refaça as que mudarem (a contagem vive em `vector_mode.rs`; a próxima nova é a `=19`),
     uma cena que mostre VÁRIAS formas com as setas lado a lado, fotografe TODAS
     (`docs/Components/ferramentas/fotografa_cena.sh`) e compare com as fotos do dono antes do relatório.
  FORA: a estrela (aprovada — só herda curas do traço); integrar; enviar.

AINDA SEM RESPOSTA DO DONO (pergunte de novo no relatório; NÃO decida, NÃO bloqueie as sub-ondas):
  • O bico cortado (doc 19 §5, opções a/b/c, fotos da cena 18 em /var/tmp/ph2d_F12_cena18/).
  • A F11: a *Resize Box* mantém a espessura do traço ao redimensionar (*Scale Strokes & Effects*)?

COMECE AUTOMATICAMENTE: FASE 0–2, reporte ao Enio em 3 linhas (sem jargão) e SIGA com a sub-onda B.

COMO TERMINA (DIRETRIZ §1.5.9): mutação de cada cláusula nova (`mutacao`, ANTES do portão — nunca ao
mesmo tempo) · gate batched 1× (`verificador`) · handoff de integração NOVO (cita o da F11; F9–F12
juntas) · CLAUDE.md §5.1 «Último:» ≤ 700 bytes · `bash scripts/doc-index.sh` ·
`rm -rf target/*/incremental` · binário smoke 2× · smoke ao dono em passos numerados, sem jargão,
com as cenas 15–17 (F11) e as novas (F12). NÃO integra, NÃO pusha — a integração das duas linhas do
Vetor é a ordem SEGUINTE do dono, depois deste smoke.
═══════════════════════════════════════════════════════════════════
