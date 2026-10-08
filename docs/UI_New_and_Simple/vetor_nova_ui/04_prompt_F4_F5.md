# 04 — Prompt da próxima janela: F4 + F5 da nova UI do Vector

> Escrito em 2026-10-08, no fecho de F0–F3 (dono, depois do smoke: *«ótimo!»*). O dono cola o bloco
> abaixo numa janela nova. ⚠️ O estado da linha descrito aqui é o de `a38b14fa8`; a FASE 0 confere-o.

```
═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F4 (radial + Ctrl+K que ensina) e F5 (iPad)
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado medido em 08/10: line/UIUX = HEAD a38b14fa8, base (merge-base) =
2a925859d = main; F0–F3 NÃO integradas ainda (9 commits à frente do main).
⇒ `git cherry main HEAD` dá >0 e os commits são DESTA linha: siga sem
reset. Se o main tiver andado: `git rebase main` (conflito fora dos seus
ficheiros = PARE e reporte ao Enio).

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md
    (o que F0–F3 entregaram, as decisões técnicas, a superfície, o ABERTO)
  docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md §4 (F4, F5) e §5 (recusas)
  docs/UI_New_and_Simple/vetor_nova_ui/02_estudo_os_apps_amados.md §3 (iPad)
  docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md D1, D2, D10
  As fontes em vetor_nova_ui/referencias/R4_apps_de_ipad.md quando precisar.

O QUE JÁ EXISTE (não reconstrua — generalize):
  • O recurso por quadro `interaction::ToolBars` (① coluna, flyout, ② row_lead/
    row_tail, ④ actions + selection, ⑤ hint/readout), publicado pela crate do
    vetor (`ph2d_app_vec::bars_publish`) a partir da TABELA do painel
    (`ph2d_panel_vector::bars` + `bars_row`). O chrome lê; o módulo escreve.
  • Os ids são os do PAINEL e o clique vai pela porta dele
    (`HeroScreen::apply_event` entrega a todo painel). Uma tecla é um CLIQUE no
    mesmo id (`ph2d_app_vec::tool_keys`, tabela `ph2d_panel_vector::tool_keys`).
  • Widget genérico `screens/hero/float_bar.rs` (+ `tool_bars.rs`); a barra do
    Quadro já o usa. Ícones do módulo a `MODULE_FACE_GROWTH = 1,725` (dono, 08/10).
  • O radial: crates/ph2d-editor-core/src/screens/hero/radial.rs (projecta o
    trilho, 8 sectores, o 8.º é o Ctrl+K). A paleta: global_palette.rs (projecta
    `rail_entries`, NÃO as ToolBars — por isso nenhum comando do vetor está lá).

ESCOPO (03_plano.md §4) — produto final, não MVP:
  F4  1. O RADIAL projecta a ① do vetor quando ele está na mão (17 ferramentas
         em 7 sectores ⇒ MEÇA e DECIDA: famílias como sectores com sub-anel, ou
         as 7 mais usadas + «mais»; o 8.º continua o Ctrl+K). Escreva a decisão
         e a medição no handoff.
      2. A paleta Ctrl+K projecta TAMBÉM as ToolBars (①, flyout, ②, ④, os
         pulldowns Snap/Path) — o mesmo id, a mesma porta — e, ao escolher, FAZ
         PISCAR o botão onde o comando mora (o *Feature Finder* do Affinity),
         ensinando a UI nova. ⛔ Campos numéricos e amostras de cor não são
         comandos (o clique sem Down não os opera): fora da paleta, ou abrem o
         seletor/foco pela porta certa (`store.open_picker_on`).
  F5  3. Modificadores na tela: primeiro o DEDO EXTRA (+1 = Shift, +2 =
         Shift+Alt, sem chrome); depois o widget segurar/travar, SEMPRE visível
         quando travado. Todo gesto com uma porta VISÍVEL ao lado (estudo §3).
      4. 2 dedos = desfazer, 3 = refazer, 4 = foco; tocar e segurar = menu de
         contexto.
      5. QuickShape (desenhar e segurar = forma perfeita, com nós editáveis) —
         ⛔ é uma LEI (reconhecimento de forma) ⇒ ORÁCULO CORRIDO (CLAUDE.md §0.9),
         passo 1 = triagem de licença por artefacto instalado
         (docs/_ComoInvestigarApps/). Sem oráculo permissivo que se corra, PARE
         esta peça e reporte ao Enio com o que mediu — não invente a lei.
      6. ③ recolhida em faixa de ícones no iPad 11" e no mini; barras laterais
         espelháveis para CANHOTOS (com esse nome); a ② EMBAIXO como opção de
         layout para tablet (D10 decisão 2).
  FORA: customizar a barra por arrasto (§5 do plano — 2.ª obra, só se pedido).

DECISÕES QUE NÃO SÃO SUAS (pergunte ao Enio se a obra as pedir):
  • tirar os chips MOVE/ROT/SCALE do Edit (decisão dele de 05/10: ficam);
  • mudar teclas já decididas (padrão Illustrator; L/K/W cedem à timeline);
  • aumentar o BOTÃO das barras (os ícones já estão no máximo dentro dele).

LIÇÕES DE F0–F3 (custaram tempo — não repita):
  • NUNCA apelide os ids (`use … ids as t;`): os portões que lêem o fonte
    procuram `ids::X` e ficam cegos (o `VECTOR_SYM_FUSE_ON` leu-se morto).
  • Faça COMMIT antes de qualquer prova de mutação: o `git checkout` da
    restauração apaga a edição por gravar (aconteceu em 08/10).
  • Teste de um número do dono afirma o NÚMERO, não a constante (a mutação
    sobrevivia ao comparar a constante consigo mesma).
  • Largura útil da fila = `tool_bar::content_rect` (respiro 2 × Spacing::Xs =
    8), nunca `aw − 16` copiado.
  • A crate do vetor é o ÚNICO escritor de `set_area_commands`; ele limpa a
    sua parte no quadro em que larga a mão — preserve isso.
  • Não edite enquanto o `verificador` corre o gate (ele compila o que está
    no disco).

LEIS DA CASA QUE ESTA OBRA TOCA:
  • A shell só encolhe: código em crates/ph2d-app-vec, ph2d-panel-vector,
    ph2d-editor-core (folga medida a 07/10: 3 367 linhas; a linha já gastou +46
    em src). Tectos: 700 LOC por ficheiro, 600 painel/shell, 500 widget.
  • HR-15 (zero hex/f32/texto cru; `ph2d-i18n` — as chaves do vetor nas barras
    moram em crates/ph2d-i18n/src/vector_bars.rs, que o censo já lê).
  • Contrato Tool=12 / PanelEvent=4 CONGELADO. Precisou mexer? PARE e reporte.
  • Gates no fecho: cada gate novo com prova de mutação (agente `mutacao`).

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas
(pwd + branch + HEAD · o que não vai reconstruir · a ordem de ataque) e SIGA
sem esperar confirmação — salvo os PARE acima (contrato, QuickShape sem
oráculo, decisões que são do dono).

COMO TERMINA (DIRETRIZ §1.5.9):
  Gate batched 1× sobre o diff acumulado (via `verificador`) · mutação dos
  gates novos · handoff de integração NOVO em docs/UI_New_and_Simple/handoffs/
  (o de 07/10 cita-se, não se reescreve) · CLAUDE.md §5.1 «Último:» → o novo ·
  rm -rf target/*/incremental · binário smoke compilado 2× (2.ª saída no
  handoff). SMOKE do Enio: cena nova `PH2D_OBJECT_MODE_SMOKE=10` (em uso:
  1|2|4|6|7|8|9 — conte nos `match` de vector_mode.rs, paint_mode.rs e
  flip_mode.rs), com FORMAS e uma situação real (o radial aberto sobre a
  coluna do vetor; a paleta a fazer piscar um botão), FOTOGRAFADA com
  docs/Components/ferramentas/fotografa_cena.sh. Passos numerados, sem
  jargão: (1) comando com o cd · (2) onde clicar, com o nome na tela ·
  (3) o que tem de acontecer · (4) como saber que deu errado.
  Você NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
```
