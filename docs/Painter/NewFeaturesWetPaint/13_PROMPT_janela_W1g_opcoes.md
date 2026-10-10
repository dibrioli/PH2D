TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), W1g: OPÇÕES NOVAS para a água — nada muda com elas
desligadas. Ordem do dono (10/10, depois da W1f): *«com ajustes corretos entre agua, pigmento e tilt temos resultados
muito bons. Se vc sabe como melhorar sem danificar e sem provocar limitações e sim criando novas opções então vamos
tentar»*. A captura dele (quatro S vermelhos, água/pigmento a descer, tela inclinada): o traço de HOJE é a referência
— ⛔ não pode mudar um byte com as opções no neutro (cerca: o S e a bancada da W1f/W1e ao byte, as miniaturas de fábrica).

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -4 && git status --short → a ponta é este prompt («docs(Painter 10/10): o prompt da W1g …»),
   com bc4da8742 (docs W1f) · fcc685e43 (fix W1f) logo abaixo; árvore limpa.
3. git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
4. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por bash scripts/ph2d-run.sh; o
   hook recusa «cargo test» fora da porta: o script vai num ficheiro (Write) e corre `bash scripts/ph2d-run.sh bash
   <ficheiro>`. A shell é fish/zsh: laços e arrays em bash vão no ficheiro.
5. Cercas verdes ANTES e DEPOIS: ferramentas/cerca.sh · os_outros_meios_nao_mudam · a suíte de ph2d-wet-paint (as
   impressões digitais do port NÃO movem) · o censo dos controlos (`cargo test -p ph2d-panel-registry-init --features
   panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`) · os portões da W1f
   (`o_escorrimento_do_s_nao_se_corta_em_rectangulos`, `a_meia_inclinacao_nao_escorre`,
   `azul_sobre_vermelho_vela_no_seco_e_sangra_no_molhado_como_no_rebelle`).

LEIA (só isto): DIRETIVA_IMPLEMENTACAO.md (§2 fiar as pontas de UI juntas, §4 olhar a 8×, §5) · 05 §4b a entrada
«W1f» inteira (a causa dos rectângulos, as ~40 recusas, o que se perdeu, a medição a 4K) · o precedente de uma
opção do Wet Paint ponta a ponta: o checkbox **Smooth Edges** (`PAINTER_WETPAINT_SMOOTH_EDGES` em
`ids/painter_wetpaint.rs`, `wetpaint_settings.rs`, o painel `ph2d-panel-painter-layers/src/paint_wetpaint.rs`, o seam
`tests/it/seam_wetpaint.rs`, o Reset, o preset) e `project-memory/reference_topic_panel_registration.md`.

⛔ O QUE NÃO SE RECONSTRÓI (dono e medições): a LINHA DE CONTACTO da W1b/W1c e o sistema de gotas (Create/Easy Drips,
DripLaw, o canal) — vetados; e QUALQUER trava célula a célula na entrada do papel seco (a lei da gota da W1e e as ~40
variantes dela, 05 §4b W1f: imprimem a grelha — pontas quadradas, escadas, riscos brancos). Uma opção nova que
precise disso é recusada antes de começar.

AS OPÇÕES (cada uma: desligada de fábrica = o produto de hoje ao byte; ligada = medida contra o Rebelle; o painel
com i18n e tokens; o preset guarda-a; o Reset volta-a ao neutro; seam test a dirigir o clique real):

O1 **Água ao vivo** (checkbox; o motor já tem `Sim::sim_while_painting`). O dono prefere tudo em tempo real mas
   pinta telas de até 4K: a 4096² o pior quadro a pintar ia de 2–4 ms (água parada) a 14–45 ms (a correr), por isso
   a W1f desligou-a. A opção devolve-lhe a escolha. ⚠️ Com ela ligada o render do filme molhado tem de continuar na
   barra da mistura (`VELATURA_DO_FILME` foi medida com a água parada — re-meça γ durante/depois com a água ao vivo;
   se divergir, a constante depende do modo, com a tabela). Mostre o custo a quem liga (a tooltip diz o porquê, sem
   números inventados: os medidos). Melhoria do custo (simular só a janela perto do pincel durante o traço) é um
   ITEM À PARTE — meça antes de prometer.
O2 **Borda que pesa** (slider 0–1, 0 de fábrica): com a tela inclinada o pigmento desce e escurece a orla de BAIXO,
   como no Rebelle a meia (orla de baixo ÷ de cima 1,6; hoje 0,77). ⛔ NÃO pela trava da água (é o que fazia os
   rectângulos): pelo PIGMENTO — o pigmento em suspensão desce mais que a água no filme (sedimentação no sentido de
   ĝ) e/ou assenta mais onde a água fica mais funda na orla. Contínuo, sem decisão por célula. Barra: a meia, o
   valor que dê 1,3–1,6 na bancada; a régua do recto continua verde (plano ≤ 22, pontas quadradas ≤ 15 %); direita
   ao byte (sem ĝ não há efeito).
O3 **Fios finos e compridos no máximo** (o Rebelle: ~10 de 3–7 px e 250–460 px; nós ~20 grossos e curtos). Dono
   10/10: *«apenas depois e se for seguro»* ⇒ SÓ depois de O1 e O2 fechadas e verdes, como opção desligada de fábrica,
   e «seguro» quer dizer, medido: (a) desligada = o traço de hoje ao byte; (b) ligada, a régua do recto verde (nada de
   pontas quadradas nem escadas) e a tela direita e a meia sem um fio a mais; (c) nenhum mecanismo vetado (canal,
   linha de contacto, Create/Easy Drips, DripLaw, trava célula a célula na entrada do seco). Se nenhuma lei passar
   (a)–(c) numa rodada de variantes, NÃO entregue: registe as recusas no 05 §4b e diga-o ao dono.

PROTOCOLO: §0.10 — as opções medem-se NUMA rodada (variantes no mesmo processo, intercaladas, o mínimo), antes de
escolher valores; red-first: o gate de cada opção nasce vermelho na opção desligada→ligada e o controlo do neutro
ao byte passa. Comentário de código curto (o porquê vai no 05 §4b).

FECHO: 05 §4b «W1g» (antes · depois · Rebelle por opção, recusas medidas); prancha 1:1 e 8× (cada opção desligada ·
ligada · Rebelle); miniaturas de fábrica NÃO mudam (as opções nascem desligadas — se mudarem, é defeito); commit por
caminho (--no-verify; `git add` explícito para ficheiros novos); nunca push. AO DONO, curto e sem jargão: o nome de
cada opção como aparece na tela, o que faz, e o smoke em passos numerados (o S vermelho, Wet Round, Tilt meio e máximo,
cada opção desligada e ligada).
