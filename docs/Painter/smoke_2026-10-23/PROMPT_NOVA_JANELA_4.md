# Prompt da janela seguinte — line/PainterWatercolor, a 2.ª onda das ferramentas (24/10)

> Estado: a coluna do Painter (a do Vector), a tabela ferramenta × linha (`NAO_LE`), a ponta inteira (Dual, Shape
> Tone, píxel, durezas) nas ferramentas que mexem na tinta, a Máscara e a Seleção em dois botões e o preview só no
> Pincel e na Borracha (fundo cheio + o S apagado) estão na linha (`24eb33e379` · `bcc7472cd8` · `31d8510dc1`),
> à espera do smoke. Handoff: `docs/Painter/handoffs/HANDOFF_line_PainterWatercolor_2026-10-23_FERRAMENTAS.md`
> (§4 tem a FILA ANTES DA INTEGRAÇÃO: os três testes lentos da fábrica de pincéis).
> Sondas por commitar viraram commit com este prompt: `tool/paint/sonda_opacidade_tests.rs` (`#[ignore]`: o centro
> do traço por pincel, por pressão, com o rato; os fatores um a um).

```
Linha line/PainterWatercolor (Modo L), continuação de 24/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, o handoff
docs/Painter/handoffs/HANDOFF_line_PainterWatercolor_2026-10-23_FERRAMENTAS.md (todo: é curto) e `git log --oneline -6`.
Delegue leitura e verificação a agentes (explorador, verificador, mutacao); a janela principal decide e desenha.
⛔ Suíte completa só antes da integração; na linha testes por NOME, mutação, clippy das crates tocadas 1× no fim.
`git worktree list` antes de tocar a fundação (ph2d-editor-core): a line/UIUX pode estar aberta — append-only.

ORDENS DO DONO (24/10, depois do smoke), uma família (a ferramenta na mão decide o que o painel mostra e o que o
motor lê) — um plano, um lote de portões (§0.10):

1. ESTUDO DAS PROPRIEDADES DO PAINEL POR FERRAMENTA (dono: «é preciso um estudo das propriedades do painel para
   filtrar quais aparecem para cada tool»). As capturas do dono mostram, com Eraser/Blur/Smear/Clone na mão, as linhas
   «Blend + Color», «Composite Brush» e «Paint Mode» — «esses campos não deveriam aparecer». ⚠️ A `NAO_LE` já esconde
   Blend/Color/Composite na Borracha no Digital: MEÇA em que estado o dono os viu (suspeita: a Borracha no Wet Paint
   fica em `PaintMode::WetPaint` e `BrushSettings::ferramenta()` responde `Pincel`; e o «Paint Mode» não está na
   tabela). Refaça a sonda `censo_ferramenta_x_linha` (`CENSO_ARMAR=ponta`) para CADA ferramenta × CADA meio que ela
   aceita (ver 2) e estenda a tabela/gate `o_painel_mostra_o_que_a_ferramenta_le` a esses meios.
2. OS MEIOS E OS PRESETS POR FERRAMENTA (padrão Procreate: cada ferramenta tem a sua biblioteca):
   - Blur, Smear e Clone só atuam no modo DIGITAL: o «Paint Mode» não aparece para elas (o motor fixa o Digital) e cada
     uma tem a SUA lista de presets (criar, guardar, a biblioteca do 23/10 — ordem, apagar, exportar — vale para elas).
   - A Borracha é especial: a SUA lista de presets e os meios Digital, Watercolor e Wet Paint (o Impasto fica de fora).
   - Hoje a Borracha usa o slot do Pincel (`mode_switch.rs`, `paint_mode.rs` — medido 24/10: ela herda a pressão→
     opacidade do pincel na mão). Dê-lhe o slot próprio. Desenho: o «slot por ferramenta» que já existe
     (`switch_brush_slot`) + a biblioteca de presets por ferramenta; os presets de fábrica de cada uma (Borracha dura,
     macia, de textura; Blur/Smear/Clone básicos) — meça cada um no traço (como `sonda_opacidade_com_o_rato_por_pincel`).
3. O FLOW À VISTA, COMO OS MELHORES APPS (dono: «Glaze Round, com o mouse, Strength máximo, ainda é transparente e não
   tem como configurar isso … colocar no painel os parâmetros», SÓ o modo Digital). Medido 24/10: o Glaze Round tem
   Flow 0,03 (escondido: o painel do Digital não tem Flow) e o tecto 0,37 do Glaze (`traco.tecto`) segura o centro em
   94/255 mesmo com Flow 1; o Burn 0,021, Color Glaze/Dodge ~0,01–0,02. Lei a fazer (Photoshop/Krita): Strength = o
   tecto do traço (Opacity), Flow = a taxa por dab que SOMA até ao tecto; hoje com Accumulate OFF o Flow vira um 2.º
   tecto (`ph2d-painter-brush/src/stroke_cover.rs:51`, `stamp_route.rs:295`). Flow no painel ao lado do Strength
   (Digital), e o Glaze (o `tecto`) visível e explicado onde já mora (Rendering). Gate: Glaze Round com Strength 1 e
   Flow 1 e tecto 1 pinta opaco num traço; o Flow baixo soma até ao Strength ao insistir; o preset de fábrica continua
   a dar a mesma veladura (miniatura igual).
4. A BORRACHA COM A CANETA (medido 24/10: com o rato apaga 255 numa passada; com a caneta o Hard Eraser tem pressão→
   opacidade 1,0 e o tecto por traço ⇒ 2–4 passadas na mão do dono 0,24–0,86). Com o slot próprio (2), o preset de
   fábrica «dura» apaga a fundo e a pressão mexe no TAMANHO. ⚠️ As três perguntas ao dono (24/10) ficaram sem resposta
   explícita; a ordem dele «corrigir conforme fazem os melhores apps» é a resposta: siga Procreate/Krita e diga-o.
5. O CONTA-GOTAS: sem preview no painel, e um CURSOR próprio com uma caixa de cor flutuante ao lado do cursor com a
   amostra que está a apanhar, em tempo real (padrão Procreate/Photoshop: o anel/caixa com a cor nova e a anterior).
   Pela porta do cursor da shell; a cor lida é a do píxel composto sob o cursor (o mesmo leitor do pick).
6. O SELETOR DE COR: abre AO LADO do Inspector (hoje abre por cima dele); se o artista o mover, abre sempre na última
   posição (gravada nas preferências de layout, como as colunas); um PIN no seletor para ficar sempre visível (não fecha
   sozinho). O seletor é partilhado (`picker_target`): o pin vale para todos os donos.

Portões red-first + mutação por lei; fotos antes/depois pelo teste e pela app (`fotografa_cena.sh`, já corrigido para
a tela virtual), olhadas. Fecho: handoff (técnico) + resposta ao Enio curta, sem jargão, smoke em passos numerados
com o cd (binário: `bash scripts/ph2d-run.sh cargo build --profile smoke -p ph2d-host-desktop`).
```
