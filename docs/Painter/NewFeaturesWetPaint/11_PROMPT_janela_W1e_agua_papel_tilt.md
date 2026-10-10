# 11 — PROMPT da janela W1e: escorrimento realista, um só papel, e o Tilt no preset

> Cole o bloco abaixo numa janela NOVA. É auto-contido. Origem: a conversa de 09/10 depois de as gotas
> serem DESFEITAS (05 §4b «⏪ DESFEITO»). Ordens do dono, verbatim: *«o escorrimento será tratado com o que
> já temos: quantidade de água, pigmento e círculo do tilt. O preset do brush configura também o tilt»* ·
> *«Tudo [on, anel, direção], e tudo continua podendo ser modificado pelo usuário»* · *«não sou eu quem
> escolhe. Vc investigando o rebelle que descobre os melhores valores»* · *«temos duas fontes de papel: o
> Paper do painel (compartilhado pelos outros modos) e Paper dentro de Tuning. O correto é obedecer o paper
> do painel»* · *«pincel molhado a tinta acumula mais nos vales do papel (isso está OK). Mas o pincel seco a
> tinta não deve ficar mais dura nos picos do papel? Acho que isso não está funcionando»* · *«Precisamos de
> um comportamento realista de verdade»*.

```
TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), onda W1e: o escorrimento REALISTA só com água,
pigmento e o círculo do Tilt; o Wet Paint obedece ao Paper do PAINEL; o pincel seco pega nos picos; o preset
guarda o Tilt. Um bloco (§0.10): é tudo a mesma família — a água sobre o papel.

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -5 && git status --short → a ponta é «docs(Painter 09/10): o prompt da W1e aponta a ponta…» (logo abaixo «audit(Painter 09/10): a mistura do Wet Paint…»); árvore limpa.
3. git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
4. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por bash scripts/ph2d-run.sh;
   um hook recusa todo texto de Bash com «cargo test» fora da porta (inclusive heredoc): escreva o script num
   ficheiro (Write) e corra `bash scripts/ph2d-run.sh bash <ficheiro>`.
5. Cercas verdes ANTES e DEPOIS: bash docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh ·
   os_outros_meios_nao_mudam (Digital/Watercolor/Impasto ao byte) · a suíte de ph2d-wet-paint.

⛔ O QUE O DONO VETOU (não reconstrua): a LINHA DE CONTACTO (a barreira que prende a água no papel seco) e o
sistema de gotas da W1b/W1c (Create/Easy Drips, Drip Size/Length, DripLaw, a força ×4, o canal que escoa a
orla). Julgou que PIORARAM O TRAÇO. Estão no histórico (8e9278993..8d3e8f68a) e as recusas medidas no 05 §4b.
A W1e trabalha no motor de 08/10 (69b4f1a75) + o Smooth Edges (opção, desligado de fábrica).

LEIA (só isto, depois o código)
1. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md (a cada passo; §4: efeito perceptual não fecha sem OLHAR).
2. docs/Painter/NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md §4b — de «⏪ DESFEITO» até ao fim, e a tabela
   de recusas da W1c. 06_gotas_rebelle_x_ph2d.md §1–§2 (o Rebelle: o escorrimento é da TELA, desligada por
   omissão; 0 de 281 presets têm gravidade; manual: «the speed of the color runoff depends on the tilt of the
   canvas, amount of water and color»). fixtures/rebelle_gotas/README.md.
3. 02_presets_para_wet_painter.md §5 (formato) e §6.9 (a tabela de fábrica — a nota de 08/10 em cima dela está
   OBSOLETA: foi a W1b, desfeita).
4. Código: crates/ph2d-wet-paint/src/{sim.rs (gravity = dir × Knob::Gravity × ring/4), solver*, paper.rs (dente,
   o tile procedural), painter.rs (paper_tooth, rebake_paper), trail/deposit.rs (DepositLaw::cell: o gate do
   papel), tuning/defs.rs}; crates/ph2d-tool-painter/src/tool/paint/{wetpaint.rs (tilt_on/ring/spoke),
   wetpaint/{session.rs (reconcile, PaperKey), dab_route.rs (o papel do painel → o motor)}, wetpaint_preset.rs
   (capturar/aplicar), wetpaint_preset/json.rs}; crates/ph2d-brush-presets/src/modelo.rs (Parametros, extra);
   crates/ph2d-panel-wet-tuning (o grupo PAPER); crates/ph2d-panel-painter-layers/src/{paint_watercolor_paper.rs,
   paint_wetpaint.rs (o checkbox «Paper»), paint_wetpaint_tilt.rs}.
5. As sondas que já existem (ignoradas; corra-as primeiro — são o «antes»):
   - tests/it/sonda_tilt_do_preset.rs (PH2D_SONDA_OUT): a bancada do Rebelle (4 traços de 150 px a 40 px, água
     1 · 0,7 · 0,4 · 0,2, 30 s) com o Wet Round e a gravidade 0,005/0,01, tela off/anel 4/anel 8; e os 4 pincéis
     com a água DELES a anel 4 e 8. Régua: docs/.../ferramentas/tilt_rebelle.py (acha os traços sozinha; corre
     sobre as fotos reduzidas do Rebelle). ⚠️ A régua `orla` lê a cauda pálida de um escorrimento como «orla de
     baixo» (dá < 1): para a orla use só cenas sem escorrimento, ou corrija-a primeiro.
   - src/tool/paint/wetpaint/sonda_papel_seco.rs (--lib): por pincel de fábrica, papel do motor × do painel,
     a correlação escuro × altura do papel e o escuro picos ÷ vales.

OS ACHADOS (medidos a 09/10, load 3–10)

F1 ⭐ O ESCORRIMENTO NÃO É REALISTA: o motor faz um PENTE. Com a água certa, a anel 4 o Wet Wash e o Wet Drip já
   descem em 7–10 dedos curtos e grossos por traço de 150 px; a anel 8 o Wet Round também. O Rebelle na mesma
   bancada: tela a meio ⇒ NADA escorre, só a orla de baixo escurece; no máximo ⇒ UM fio fino e longo por traço
   molhado (283 · 249 px para água 100 · 70; 0 para 40 e 20); com a água de cada preset: Round 302, Wash Round 0,
   Mop 0, Dip Pen 0. (Prancha: corra a sonda e componha pinceis_meio/pinceis_max.) A regra do dono e a do manual
   do Rebelle são a mesma: velocidade = inclinação × água × cor. O defeito é do MOTOR (o comprimento de onda e o
   limiar da instabilidade de dedos: muitos dedos curtos e cedo demais), NÃO dos pincéis — e tem de sair sem a
   barreira vetada. Investigue a física (rivulet/fingering de um filme num plano inclinado: o limiar de
   escorrer cresce com o ângulo e com o volume da conta de baixo; o espaçamento dos dedos é da ordem do
   comprimento capilar — poucos fios) e meça antes de propor; os knobs escondidos extFingering/extRivulets já
   foram medidos INERTES para fazer gotas (06 §3) — confira se mexem no PENTE.
   BARRA (red-first, a bancada como gate): tela off ⇒ 0 px; anel 4 ⇒ 0 px de escorrimento, orla de baixo mais
   escura que a de cima (Rebelle 1,6 na água 100); anel 8 ⇒ 1–2 fios por traço de água ≥ 0,7, ≥ 200 px, finos;
   0 nos de água ≤ 0,4. E o TRAÇO com a tela desligada não muda (fnv da bancada antes da mudança).

F5 A GRAVIDADE POR PINCEL (Knob::Gravity no preset: Round 0,005 · Wash 0,003 · Dry 0,002 · Drip 0,01) não existe
   no Rebelle e não está na regra do dono (água, pigmento, Tilt). Torne-a a MESMA em todos (uma constante do
   motor, calibrada para que o anel 8 dê o Rebelle no máximo e o anel 4 o Rebelle a meio — a orla e os fios
   da F1), e tire-a da diferença entre pincéis (fica no ficheiro, igual em todos, ou sai do preset — decida
   pelo formato: campo desconhecido vai para `extra`).

F2 ⭐ DOIS PAPÉIS. Quando o Paper do painel tem um Kind armado, o motor usa-o (dab_route.rs: PaperKey,
   seed_paper_with); sem Kind, o motor usa o SEU papel procedural (paper.rs, rebake_paper), cujos knobs
   paperContrast/Fibres/Grooves vivem no grupo PAPER do Tuning (e ficam escondidos/inertes com o painel armado).
   Relief/Roughness do substrato (painel) NÃO são lidos pelo Wet Paint. O checkbox «Paper» do Wet Paint
   (paper_visual: visualGrain/emboss/paperVisibility) é uma camada visual própria, com o mesmo nome da secção do
   painel. ORDEM DO DONO: o Wet Paint obedece ao Paper do PAINEL — uma fonte. Desenho: o papel do motor sai como
   fonte do artista (sem Kind = o papel de fábrica do painel, o mesmo que os outros meios usam); o grupo PAPER do
   Tuning sai; Relief/Roughness passam a agir como nos outros meios; o checkbox «Paper» do Wet Paint ou passa a
   ler o painel ou sai (meça o que ele acrescenta que o painel não faça). Gate: cada controlo da secção Paper do
   painel muda o traço do Wet Paint (censo «o painel escreve onde · quem lê · o leitor decide»), e nenhum
   controlo de papel vive fora dela.

F3 O PINCEL SECO NÃO PEGA NOS PICOS com o papel do motor: Wet Dry Brush (água 0,2) r(escuro, altura) +0,05,
   picos ÷ vales 1,06; com o papel do painel armado +0,54 e 1,81. Os molhados acumulam nos vales (r −0,14 a
   −0,84, picos ÷ vales 0,94–0,98 — fraco). A F2 deve curar a F3; meça o pincel seco do Rebelle (Flat/Bristle,
   água do preset ~9, papel Rough e Cold) com a mesma régua para a BARRA (picos ÷ vales e a fração de papel que
   fica branca — o pincel seco real SALTA os vales) e para a dos molhados.

F4 O PRESET NÃO GUARDA O TILT (wet_preset_capturar/aplicar ignoram tilt_on/ring/spoke; Parametros não tem o campo).
   Ordem do dono: guarda os três; o artista continua a mudá-los. Campo novo no Parametros (o formato não sobe de
   versão: um preset antigo sem tilt NÃO mexe na tela), capturar/aplicar, o preview (já leva o tilt na cena;
   a MINIATURA lê o tilt vivo do tool — passe a ler o do preset), «lembrar» da política se fizer sentido, Save/
   Reset. Gate: o_pincel_guardado_pinta_o_mesmo_traco com o tilt mexido; um preset antigo não mexe no tilt.
   VALORES DE FÁBRICA pela evidência do Rebelle (os pincéis dele não inclinam; a tela nasce direita): Round, Wash
   e Dry Brush com o Tilt DESLIGADO (o anel e a direção guardados no meio/para baixo, para quando o artista o
   ligar); Wet Drip — o único que promete escorrer — LIGADO, no anel que a F1 calibrar para «escorre como o
   Rebelle no máximo», para baixo. Regere a fábrica (PH2D_REGERA_PRESETS=1) no commit da lei.

F6 ⭐ A MISTURA DE PIGMENTO não é realista (dono: «não acontece em tempo real mas apenas no mouse up… acontece da
   mesma maneira sobre tinta seca e tinta úmida. Isso precisa ser auditado»). Sonda
   src/tool/paint/wetpaint/sonda_mistura.rs (--lib): vermelho horizontal, azul vertical por cima, com o vermelho
   MOLHADO (0,5 s) ou SECO (Dry canvas); lido DURANTE o azul e depois do Up:
   | | cruzamento durante | depois do Up | rasto do azul após o cruzamento | azul limpo |
   |---|---|---|---|---|
   | vermelho molhado | (57, 56, 183) | (66, 65, 185) | (56, 78, 209) | (51, 78, 214) |
   | vermelho seco | (53, 57, 186) | (43, 58, 196) | (57, 78, 208) | (51, 78, 214) |
   ⇒ o azul COBRE o vermelho como tinta opaca nos dois casos (nem velatura sobre o seco — na aquarela real azul
   sobre vermelho seco escurece para violeta —, nem mistura sobre o molhado); molhado ≈ seco; o pincel quase não
   arrasta tinta (+5 de vermelho no rasto). Mecanismo (leitura, file:line):
   - tempo real: o depósito entra a cada ~2 dabs durante o arrasto (painter/doors.rs:114-154, trail.rs:339), mas a
     SIM pára com o traço aberto (painter.rs:455 `sim_should_run = !stroke_down`; offthread.rs:255/418): a água
     só corre e mistura depois do Up. Os métodos não incrementais (Line, Arc, Ellipse, …) depositam tudo no Up
     (wetpaint_commit.rs:58-88).
   - molhado × seco: nem o depósito (trail/deposit.rs:88-127, só lê `film` no ExtWetSoften) nem a pickup
     (trail/transfer.rs:41-50: assentado com metade do peso, sem porta de `film`/`wet`) olham a humidade; Pickup
     de fábrica 0,005 (quase nada); a mistura é `ColorMix::Plain` por omissão (sim.rs:199-205; o K–M é a caixa
     experimental km_mixing/km_glaze, desligada).
   BARRA: meça o MESMO cruzamento no Rebelle (molhado e seco, a porta WebSocket, ferramentas/gotas_*.sh) e use-o:
   sobre o seco uma VELATURA (o cruzamento mais escuro que as duas cores, o vermelho não se move, o pincel não o
   apanha); sobre o molhado MISTURA (o cruzamento entre as duas, o pincel arrasta cor, a mancha mexe-se) — e a
   mistura VISÍVEL enquanto se pinta (o cruzamento durante ≈ o de depois). Cuidado: deixar a sim correr com o
   traço aberto muda o custo por quadro (meça, MEDIR_VELOCIDADE.md) e as cercas.

JÁ FEITO NESTA LINHA (não refaça): o menu de presets lista cada pincel UMA vez (3d8e40be1); o Smooth Edges do
Wet Paint (opção, desligado).

ORDEM: F6 (a mistura e o tempo real: muda o que todo traço faz sobre outro) → F2+F3 (o papel: muda o traço de todos os pincéis — meça antes/depois a olho a 8×) → F1+F5 (a física do
escorrimento e a constante) → F4 (o Tilt no preset e os valores de fábrica). Medições de cada passo numa rodada
(variantes no MESMO processo); série só porque o resultado de F1 decide os valores de F4.

FECHO: tabela no 05 §4b (antes · depois · Rebelle) para F1–F5; prancha a 1:1 e a 8× (o nosso × o Rebelle);
commit por caminho (--no-verify); nunca push. AO DONO, curto e sem jargão, com o smoke em passos numerados.
```
