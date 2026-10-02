# ARQUIVO — CLAUDE.md (história, 71 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-1573,1645-1656` do original.
>
> ⚠️ **A única alteração ao corpo:** 14 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **UI/UX — redesenho plano** (Godot 4.6 «Modern» + modelo de painel do Blender; `PH2D_UI_NEW=0` volta ao clássico):
  quatro temas derivados, e a moldura · o vão · a quina · o **recuo** · o vão **ícone→rótulo** cada um por UMA porta em
  [`ph2d-tokens`](../../../crates/ph2d-tokens/src/spacing.rs) — mais o **cartão** no lugar do risco azul, a **lei do grupo**, os
  **apelidos de cor** que resolvem pelo PAI (⛔ **57 valores saíram do `tokens.json`**: editar um deles funde limpo e a
  edição **evapora**), o **chão da janela** (`WindowGround`, degrau ABSOLUTO de `0,04`) que faz de cada área um cartão, e
  as **abas de encaixe** com largura por conteúdo, ordem, arrasto entre encaixes, ícone e transbordo (a w50 pôs a cache
  de texto a **rodar** em vez de se deitar fora: `182 ms` por quadro com tudo aberto passaram a `7,8`).
  ⛔⛔ **O gesto que FECHAVA uma coluna arrastando a borda SAIU** (ordem do dono, 09/09: *«Deixa o colapsar apenas no
  menu da barra superior»*) — e ⚠️ **a lei ficou sem quem a accione**: a **INVOLUÇÃO** do
  [`hero::dock_columns`](../../../crates/ph2d-editor-core/src/screens/hero/dock_columns.rs) continua certa e gateada (reabrir
  devolve exactamente o que aquele fecho levou; re-derivar do registo abria **22** onde o fecho levara **1** — *uma
  involução é uma MEMÓRIA, nunca uma re-derivação*), e em 10/09 o `dock_columns::close` tinha **ZERO chamadores de
  produto** ⇒ *retirar um gesto deixa a lei dele viva e órfã, e nenhuma sonda deste repo pergunta se uma PORTA tem
  chamador*. ✅ **CURADO em 19/09: a outra metade da ordem foi escrita** — *View ▸ Left Column* / *Right Column*, dois
  ALTERNADORES (`MODULE_TRUTHS`, acesos quando a coluna está aberta) que entram pelo despacho real. ⚠️ Entre 09/09 e
  19/09 **não havia maneira nenhuma de fechar uma coluna**. ⛔ E uma mutação SOBREVIVENTE nomeou a lei que faltava
  gatear: fechar pelo menu passa a **ESCOLHA** de largura (`dock_width_choice`, `None` quando ninguém arrastou) e nunca
  o número — o doc do `close` já o escrevia, e ninguém o media porque ninguém o chamava.
  ⚠️ **`HIER_ROW_H_PX` e `SECTION_GAP_PX` deixaram de existir** — a linha da hierarquia usa o `ROW_H_PX` como todas as
  outras, e o vão de secção é a porta `section_gap_px()`. ⛔ A cura de um uso novo é **converter**, nunca repor a
  constante.
  ⭐⭐⭐ **A LINHA DE PROPRIEDADE E O HR-15 FECHARAM** (2026-09-16, 96 commits — guia do integrador:
  [FECHO](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_FECHO_2026-09-16.md); o mecanismo, por §, no
  [handoff da jornada](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-14_LINHA_DE_PROPRIEDADE.md),
  que tem 176 KB e **se consulta, não se lê**): o formulário do app tem **uma porta**
  (`widget::property_row_columns` — a unidade sai do rótulo, o nome sai de cima do controlo, a caixa de marcar entra, e
  há **manual verificado por teste**), e *zero string hardcoded* deixou de ser intenção — **26 painéis + a moldura + a
  shell + as 8 crates de família** falam por `ph2d-i18n` (**4 044** chaves · **3 027** sítios), com **30 gates** de
  censo a defendê-lo a partir da crate-régua nova [`ph2d-label-census`](../../../crates/ph2d-label-census). ⚠️ **O que sobra é
  de outra natureza e é a fronteira seguinte:** os MOTORES (o catálogo do `ph2d-component-desc`, os manifestos dos nós,
  `painter-effects`/`-brush`, `tool-vector`) escrevem o rótulo **numa tabela de dados**, e ali a chave tem de ser
  derivada do id — mais os `62` literais que só o censo de PORTA vê (`python3 scripts/censo-texto-pintado.py`, de `418`
  em 18 crates). ⛔⛔ **Para quem funde:** os 30 gates são CENSOS — um literal de UI novo de outra linha reprova na
  árvore COMBINADA (cura: migrar o texto, ou isenção NOMEADA com mecanismo), e o CI **não os corre** (o job de teste é
  um `-p` de 25 pacotes; quem os corre é o `ship.sh`).
  ⭐⭐⭐ **A LINHA DE PROPRIEDADE CHEGOU AOS PAINÉIS E O TÍTULO DE SECÇÃO É UM SÓ** (25/09,
  [handoff do integrador](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-25_A_LINHA.md)):
  toda escolha passa por `paint_choice_row`, todo botão de acção por `caixa_do_botao`, todo grupo se
  DECLARA (`composto::grupo`) e a catraca `CARGA_DE_COMANDOS` só desce; o título de secção do app é o
  do Grid (corpo `Md`, sem caixa alta, separador à direita) e a grade `Behind` vai atrás de verdade
  pelo acumulador das faixas. A fronteira dos motores, a escada das elisões e o balão (20/09) estão no
  [handoff de 20/09](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20.md).
  ⭐⭐⭐ **A ROLAGEM É UMA, AS SECÇÕES SÃO CARTÕES E O TEXTO TEM FONTE, PESO E TAMANHO** (01/10,
  [handoff do integrador](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-01_A_LINHA.md)):
  toda lista rola pela porta `panel::scroll_area` (⛔ barra registada à mão fica morta sob o dedo);
  secções em cartões com pega, tema e notas em todo painel; *Edit ▸ Preferences…* escolhe fonte
  (Inter · Noto Sans · Atkinson), peso (Light · Normal · Strong) e tamanho (Small · Normal · Large),
  lidos UMA vez no `layout_inner`. ⛔⛔ **Até 01/10 o app desenhava a Noto do SISTEMA e os testes a
  Inter** (nome de família errado) — a fábrica é a Inter; o piso de um campo é a porta
  `number_input_min_w_px()`, nunca o `const`.
  ⏳ **Aberto:** partir o `DrawMode` nos dois eixos · a pose 2D/3D e os 9 toggles de módulo → Layout (as três
  **decisão do dono**) · as caixas do `painter_layers` registadas como botão (só instrumento) · o painel da
  escultura (território da `line/sculpt3d`) · o **«travou por um minuto»** de 09/09 segue **sem reprodução**.
  **Smokes:** abrir o app (a UI nova é o caminho de omissão) · abrir **todos** os painéis pelo menu *Window* e mexer no
  ecrã (a w50: sem número à vista, o sintoma é o app deixar de engasgar) · *View → Reset Panel Layout*.
  ⚠️ **`PH2D_UI_NEW=0` NÃO é o ecrã de antes do redesenho** — ele devolve **seis pintores de widget**, a família de temas
  da barra do topo e o tema de arranque. A estrutura (colunas, encaixes, faixa de abas, transbordo) é a **mesma** nas
  duas desde 2026-08-30, e há gate a mantê-la assim (`the_look_is_a_widget_skin_never_an_area_model`). ⛔ A cláusula
  *«o clássico tem de ficar byte a byte»* esteve aqui e era **falsa no dia em que foi escrita** — as abas nasceram em
  30/08 sem consultar a aparência, e a cláusula entrou no roteador **oito dias depois**
  ([`medicoes/10`](../../UI_New_and_Simple/medicoes/10_o_classico_nao_e_um_ecra_anterior.md)).
  ⚠️ **A arrumação vive fora do repo** (`~/.ph2d/layout.txt`, XOR contra o `DEFAULT_VISIBLE`) — um ficheiro velho abre o
  app com um painel fechado, e **apagá-lo é o reset**, não sintoma de regressão.
  **Ler:** [`docs/UI_New_and_Simple/`](../../UI_New_and_Simple) ·
  ⭐⭐⭐ **[handoff de 20/09](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20.md)** — a fronteira dos motores, a escada das elisões e o balão; o §4 tem **sete** coisas que uma leitura rápida do diff entende ao contrário (entre elas que a catraca dos cortes **não desce** com a lei de encurtar, e porquê) e o §5-bis os **dois** vermelhos que só a árvore combinada viu ·
  [handoff de 10/09](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-10.md) (⚠️ o §6 é o que a
  fusão parte, com endereço — o `Panel::ICON` obrigatório, o `IconId` cuja **ordem é o índice**, o `slot_tabs` partido em
  quatro e as **sete catracas a zero de folga**; o §11 as leis que a jornada pagou) ·
  [handoff de 07/09](../../UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-07.md) ·
  [handoffs](../../UI_New_and_Simple/handoffs/README.md)

