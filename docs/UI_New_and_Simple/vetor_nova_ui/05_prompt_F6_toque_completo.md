═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F6: o TOQUE COMPLETO (navegar com os dedos) e o QuickShape
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie).
Máquina = workstation ⇒ Modo L (ADR-0107).

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: execute o BLOCO de
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
MODULO=UIUX — FASE 0 → 1 → 2, na ordem.
Estado medido em 08/10: line/UIUX = HEAD (o commit deste prompt), base
(merge-base) = 2a925859d = main; F0–F5 NÃO integradas (36 commits à frente).
⇒ `git cherry main HEAD` dá >0 e os commits são DESTA linha: siga sem reset.
Se o main tiver andado: `git rebase main` (conflito fora dos seus ficheiros =
PARE e reporte ao Enio).

LEIA, dentro da worktree, nesta ordem:
  docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F4_F5_ALCANCE_E_TOQUE.md
    (o que F4–F5 entregaram, o que a FOTO e a MUTAÇÃO apanharam, o ABERTO)
  docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md D10, D10.1, ⛔ D10.2 (a roda SAIU)
  docs/UI_New_and_Simple/vetor_nova_ui/02_estudo_os_apps_amados.md §3 (iPad)
  docs/UI_New_and_Simple/vetor_nova_ui/referencias/R4_apps_de_ipad.md — os
    «Padrões transferíveis» #10, #11, #13 e o Procreate (QuickShape, L165–169).

O QUE JÁ EXISTE (não reconstrua — estenda):
  • O reconhecedor de toque PURO: crates/ph2d-editor-core/src/interaction/touch.rs
    (`TouchGestures::feed/tick/modifiers`, `TouchOut`), números do UIKit
    (`LONG_PRESS_NS` 0,5 s, `SLOP_PT` 10 pt). O 1.º dedo só vira `Press` ao andar;
    um 2.º dedo ANTES disso faz do contacto um GESTO (hoje: toques de 2/3/4 dedos;
    se os dedos ANDAM, o gesto é cancelado e nada acontece — é o buraco abaixo).
  • A ponte da shell: shells/desktop/src/touch_input.rs (`on_touch`, `touch_tick`,
    `apply_touch`, modificadores efectivos = teclado OU toque, `toggle_focus`).
  • O widget Shift·Alt·Ctrl (screens/hero/touch_mods.rs), Ver ▸ Focus mode,
    Left-handed (prefs.txt), a fila ② embaixo (layout.txt), a faixa do ③.
  • O pan da câmara é o botão do MEIO (idioma do Blender) e o zoom é a roda —
    descubra as portas deles na shell ANTES de escrever (o `explorador` acha-as).

ESCOPO — produto final, não MVP:
  1. ⭐ NAVEGAR COM OS DEDOS (o buraco que um iPad sente primeiro — R4 #11:
     universal entre os apps medidos): 2 dedos que ANDAM = pan; pinça = zoom em
     volta do ponto médio; (rotação da vista só se a câmara 2D já tiver rotação —
     MEÇA, não invente). «Quick-pinch encaixa no ecrã» (R4 #11) = o enquadrar que
     já existir (ache-o; se não existir, é pergunta ao Enio, não invenção).
     ⚠️ O toque de 2 dedos (desfazer) e o pan de 2 dedos partilham o início: a
     fronteira entre eles é a FOLGA do UIKit que já está no reconhecedor — um
     toque que andou não desfaz, um pan que não andou não move. Gate com os dois
     lados da fronteira e o CONTROLO.
     Pela MESMA porta do rato (o braço do botão do meio / da roda), nunca uma
     segunda câmara.
  2. QuickShape (desenhar e segurar = forma perfeita com nós editáveis):
     ⛔ é uma LEI ⇒ ORÁCULO CORRIDO (CLAUDE.md §0.9). Medido em 08/10: NENHUM app
     instalado reconhece formas (Xournal++ e Rnote não estão instalados;
     OpenToonz BSD-3 só desenha por arrasto). ⇒ o 1.º passo é PERGUNTAR ao Enio
     se instala o Xournal++ (GPL-2 — corre-se como caixa preta, só a saída é
     gravada; `docs/_ComoInvestigarApps/`). Sem «sim», esta peça fica PARADA e o
     resto da janela segue. Com «sim»: triagem por artefacto (`pacman -Qi`),
     fixtures de traços NOSSOS, a saída dele como gate, e só então o código.
  3. A ponte do toque na shell (`touch_input.rs`, ~150 linhas) — mova para a
     crate tudo o que não for porta da shell (HOWTO_partir_uma_familia_da_shell
     §2 antes do primeiro ficheiro); a shell só encolhe.
  FORA: o menu de roda (D10.2 — o dono mandou retirar; não reconstruir);
  customizar a barra por arrasto (2.ª obra, só se pedido).

DECISÕES QUE NÃO SÃO SUAS (pergunte ao Enio se a obra as pedir):
  • instalar qualquer programa (o oráculo do QuickShape);
  • qualquer tecla nova ou mudada (padrão Illustrator; o `Q` (tiro) e o `J` (veneno) são das
    cenas de jogo — gate `a_tecla_do_gatilho_nao_e_reclamada_pelo_editor`);
  • voltar a ter roda ou faíscas (D10.2).

LIÇÕES DE F4–F5 (custaram tempo — não repita):
  • FOTOGRAFE antes de declarar: dois defeitos só a foto viu (o interruptor que
    troca de id ao ser executado; a porta `stroke_frame` que no tema moderno
    ignora cor e alfa). Gate de ESTADO verde ≠ tinta na tela.
  • Gate que publica as barras UMA vez mede um app que não existe: republique
    (e aplique a ponte do módulo) depois de cada acção, como o quadro faz.
  • Antes de reservar área: o piso de 88,7 % do ecrã com as colunas fechadas é lei
    (`collapsing_both_columns_still_gives_the_tablet_back`) — o que é do tablet
    FLUTUA. E a coluna aberta no iPad 11 é lei (`a_coluna_pintada_encolhe…`).
  • O `App` só larga campos (`the_app_only_sheds_fields`) e `main.rs` está no
    tecto: estado novo da shell vai para `thread_local` no ficheiro dela ou para a
    crate; uma remoção obriga a BAIXAR a catraca no mesmo commit.
  • A shell pergunta ao chrome pelas portas FÍSICAS (`hero.chrome_hit`), nunca
    `hit_index.hit` com um ponto da janela.
  • Commit antes de toda prova de mutação; não edite enquanto o `verificador` corre.
  • O guarda de recursos lê o TEXTO do comando: um heredoc com «cargo/clippy»
    dentro é recusado — ponha o script num ficheiro do scratchpad.

LEIS DA CASA QUE ESTA OBRA TOCA:
  • A shell só encolhe (crates/ph2d-editor-core, ph2d-app-*); tectos 700 LOC por
    ficheiro (o `hero.rs`, o `left_rail.rs` e o `state/mod.rs` estão a ≤ 3 linhas).
  • HR-15 (zero hex/f32/texto cru; chaves novas em crates/ph2d-i18n/src/chrome_reach.rs,
    que o censo `every_chrome_key_exists_on_both_sides` já lê).
  • Contrato Tool=12 / PanelEvent=4 CONGELADO. Precisou mexer? PARE e reporte.
  • Gates no fecho: cada gate novo com prova de mutação (agente `mutacao`).

COMECE AUTOMATICAMENTE: depois da FASE 0–2, reporte ao Enio em 3 linhas
(pwd + branch + HEAD · o que não vai reconstruir · a ordem de ataque), faça a
PERGUNTA do QuickShape (item 2) e SIGA com o item 1 sem esperar a resposta.

COMO TERMINA (DIRETRIZ §1.5.9):
  Gate batched 1× sobre o diff acumulado (via `verificador`) · mutação dos gates
  novos · handoff de integração NOVO em docs/UI_New_and_Simple/handoffs/ (o de
  08/10 cita-se, não se reescreve) · CLAUDE.md §5.1 «Último:» → o novo ·
  rm -rf target/*/incremental · binário smoke compilado 2× (2.ª saída no
  handoff). SMOKE do Enio: o toque não se testa sem ecrã de toque — o smoke é uma
  cena `PH2D_OBJECT_MODE_SMOKE=11` (em uso: 1|2|4|6|7|8|9|10 — conte nos `match`
  de vector_mode.rs, paint_mode.rs e flip_mode.rs) que ALIMENTA o reconhecedor
  com toques sintéticos (pan e pinça sobre FORMAS reais) e mostra o resultado,
  FOTOGRAFADA com docs/Components/ferramentas/fotografa_cena.sh. Passos
  numerados, sem jargão: (1) comando com o cd · (2) onde olhar, com o nome na
  tela · (3) o que tem de acontecer · (4) como saber que deu errado.
  Você NÃO integra e NÃO pusha: entrega o handoff e PARA.
═══════════════════════════════════════════════════════════════════
