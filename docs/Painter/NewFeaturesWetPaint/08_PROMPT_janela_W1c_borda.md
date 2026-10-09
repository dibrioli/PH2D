# 08 — PROMPT da janela W1c: a borda e as gotas pela auditoria de 08/10

> Cole o bloco abaixo numa janela NOVA. Ele é auto-contido: a janela não tem a conversa de 08/10, só
> isto, o código e os docs. Ordem do dono (08/10): *«ficam no Paper! … Cuidado para não piorar nossa
> engine.»*

```
TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), onda W1c: a borda inferior e as gotas pela
auditoria de 08/10 (2 lentes + o oráculo do S)

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro; o mesmo caminho relativo existe no main):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../Worktrees/line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -3 && git status --short   → a ponta é «docs(Painter 08/10): o prompt da janela W1c…»;
   árvore limpa.
3. git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
   ⚠️ O degrau do schema da linha é 187 → 188 com migração (ProjectFileV187, braço `187 =>` em
   shells/desktop/src/project_load_read.rs): se o main passou de 187, renumere À MÃO os dois (o
   schema-recount.py só reconta a escada e a tripla).
4. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh
   · bash scripts/ph2d-run.sh bash scripts/cargo-check-narrow.sh ph2d-wet-paint
5. AS QUATRO CERCAS têm de estar verdes ANTES de tocar em produto, e ficam verdes até ao fim:
   a) bash docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh   (Digital/Watercolor/Impasto: ficheiros)
   b) cargo test -p ph2d-tool-painter --test it os_outros_meios_nao_mudam   (os outros meios: pixels)
   c) cargo test -p ph2d-tool-painter --test it sem_create_drips_e_o_escorrimento_natural_de_antes_ao_byte
      (⛔ Create Drips DESLIGADO = o motor de ANTES da W1b, ao byte — o escorrimento natural que o dono quer
      intacto; fnv 0x7cc9…6d0e a meio, 0x0747…3d4e no máximo)
   d) cargo test -p ph2d-tool-painter --test it com_a_tela_direita_o_traco_e_o_de_antes_ao_byte
   + a suíte do motor: cargo test -p ph2d-wet-paint (fingerprints do port 1:1, Sim::gotas = None).
   (cargo test sempre num .sh por bash scripts/ph2d-run.sh, nunca em heredoc.)

«NÃO PIORAR A ENGINE» = estas cercas verdes do princípio ao fim, e toda mudança de lei SÓ no caminho com
Create Drips LIGADO (Sim::gotas com create). Se uma mudança precisar de tocar no caminho sem a lei, PARE e
reporte antes.

LEIA (nesta ordem; só isto, depois o código)
1. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md (a cada passo).
2. docs/Painter/NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md §4b — as cinco rondas da W1b de 08/10
   (o que foi feito, o que foi MEDIDO e RECUSADO: a faixa de transição que inunda — 106–127 mil px; a
   difusão do pigmento, inerte sobre o assente; não reconstrua).
3. docs/Painter/NewFeaturesWetPaint/06_gotas_rebelle_x_ph2d.md e fixtures/rebelle_gotas/README.md — a
   tabela NOVA do S (o oráculo a 1:1: s_inclinado.png, s_inclinado_1.png, s_sem_gotas.png, s_plano.png).
   ⚠️ As fotos antigas (plano/inclinado/meio/sem_gotas/gotas_max) estão REDUZIDAS a 565×380: não servem
   de oráculo para a borda; só as s_*.png servem.
4. Código: crates/ph2d-wet-paint/src/drips.rs (inteiro), solver/advect_jacobi.rs (advect_jacobi_core:
   o pin), sim.rs (Sim::gotas, gravity × DRIP_LAW_GRAVITY, o estágio final), painter/stage.rs;
   crates/ph2d-tool-painter/src/tool/paint/{inercia.rs, wetpaint_settings.rs, wetpaint/session.rs,
   wetpaint/grid_map.rs, wetpaint/composite.rs, papel.rs, watercolor_settings.rs}, tool/{trait_impls_raster.rs,
   documents.rs, persist.rs}; shells/desktop/src/undo.rs; crates/ph2d-panel-painter-layers/src/
   {paint_watercolor_paper.rs, populate_sections.rs, number_field.rs}; os gates
   crates/ph2d-tool-painter/tests/it/{gotas_pela_lei_do_rebelle.rs, sonda_gotas_do_dono.rs}.

DECISÕES DO DONO JÁ TOMADAS (não re-perguntar)
- Só o Wet Paint evolui; Digital/Watercolor/Impasto não mudam um byte.
- A tela nasce direita; o dial Tilt é a porta única da gravidade (sem fileira Gravity, sem gravity nos presets).
- Create Drips DESLIGADO = o escorrimento natural de antes, ao byte (08/10: «vc aniquilou aquele escorrimento
  bonito e natural…» — foi devolvido; NÃO o toque). Create Drips LIGADO = a lei do Rebelle.
- Easy Drips (o papel que segura menos) fica.
- 08/10: os controlos das gotas FICAM NA SECÇÃO PAPER (como no Rebelle) — não os mova para o cartão Tilt.
- Qual nasce ligado (Create Drips on/off de fábrica) — o dono decide DEPOIS desta janela, no smoke. Não mude.
- Decisões técnicas são suas (padrão-ouro); produto e envio são do dono.

O QUE O ORÁCULO MOSTROU (o S, tela (0,1), Create Drips 5/5, água 100, 1:1 — fixtures/rebelle_gotas/s_*.png)
- O corpo do traço CLAREIA: escuro 15–38 contra ~101 com a tela direita (o pigmento sai do corpo).
- O escuro sobe em RAMPA suave (90 % → 10 % em 21–41 px) até uma linha fina de 1–3 px NA borda; a borda é
  tão lisa como sem inclinação (saltos 0/1/2 px, máx 4, 1,1 px rms); o contorno não se mexe (mediana 0 px).
- No vale empoça mais (33→127, 20→146).
- As gotas saem de TODA a orla de baixo (10–11 no S: encostas, arco, vale), 3 px a meia altura (3–7),
  228–430 px; escuro ao longo do fio 27–187 contra o corpo 17–29; um pequeno «funil» escuro onde nascem.
- Create Drips off (no Rebelle): 0 gotas, a orla escurece em pontos.

OS ACHADOS DA AUDITORIA (08/10) — corrija TODOS, num bloco (§0.10), cada um com o gate que faltava, visto
VERMELHO antes:

L1-1 ⭐ A ESCADA. A linha de contacto (drips.rs pin_downhill, chamada em advect_jacobi.rs) congela o contorno
     de 1 px do pincel e a força da lei (DRIP_LAW_GRAVITY = 4, sim.rs Sim::gravity) empoça o pigmento até à
     COR PURA (232) numa faixa chapada de ~9 px; o TOPO dessa faixa é a escada que o dono vê. Medido (sonda
     do dono, encosta descendente do S, scratchpad/borda.py da auditoria): colunas de borda dura 79,6 % contra
     31,2 % com a tela direita; a dureza segue a força (×1 36,7 % · ×2 70,6 % · ×4 79,6 %), NÃO o NOTCH_W nem
     o FRONT_DRY. A força 4× foi calibrada contra fotos REDUZIDAS (inválidas para a borda) e pela régua
     orlas() que premia uma orla escura — dois objectivos nunca medidos juntos.
     CURA (direcção): a borda e o corpo como o oráculo do S — corpo que clareia, rampa suave, linha fina na
     borda, sem faixa chapada; recalibre DRIP_LAW_GRAVITY (e o que dele depende: SLIDE, HOLD, HOLD_EASY,
     RUN_5, TRAIL_ALPHA) CONTRA AS s_*.png a 1:1, numa rodada só, com a tabela no doc. A linha fina na borda
     é o edge darkening clássico — confira se o motor já o tem (Knob::EdgeDarkening) antes de inventar.
     GATE: na cena do S (sonda_gotas_do_dono ou um S na bancada 1000×640 igual ao do oráculo), o perfil
     vertical da borda em 3–4 colunas: escuro do corpo inclinado / direito, altura da rampa (90 → 10 %),
     espessura do que é ≥ 100, fração de colunas de borda dura, saltos/rms da borda — cada barra no VALE
     entre o nosso de hoje e o Rebelle (os números acima). Mutação: DRIP_LAW_GRAVITY de volta a 4 ✗.
L1-2 O NOTCH_W age nas encostas, ao contrário do comentário (drips.rs, o bloco «Uma BRECHA estreita…» e o
     05 §4b 5.ª ronda): enche 44 de 221 colunas da borda e REGULARIZA a escada (patamares ≥ 6 px: 2 → 9). Ou
     ele passa a respeitar a encosta, ou sai (meça; o gate das brechas a_orla_de_baixo_e_continua_sem_brechas
     só usa traços horizontais — fixture sem o fenómeno: estenda-o a uma encosta).
L1-3 ⭐ AS GOTAS NASCEM DE TODA A ORLA no Rebelle; a regra «só o PONTO BAIXO» (spawn: `baixo > LOW_SLACK`)
     foi inventada contra o pente e contradiz o oráculo. Reponha o nascer ao longo da orla com o espaçamento
     da bacia (CATCH_PER_SIZE), sem voltar ao pente de dezenas (a cena do dono deu 20 fios antes; o Rebelle
     10–11 num S de ~1000 px). GATE: na cena do S, fios em encostas > 0 e total na faixa do Rebelle
     (calibre com o S do oráculo); fios 3 px a meia altura; o «funil» escuro no nascimento.
L1-4 is_front e o pin discordam por 1 célula sobre a orla (drips.rs is_front conta FRONT_DRY a partir de baixo
     da célula molhada; o pin conta a partir de baixo da seca ⇒ corrida de 9; na borda da folha um diz
     «não é orla» e o outro «está fora»). Teste-propriedade da auditoria: corrida de 8 ⇒ is_front true mas a
     de baixo não é segura (a água passa por baixo de uma orla); orla a 2 células do fundo ⇒ segura mas nunca
     pinga. CURA: UM predicado partilhado; GATE: a propriedade «is_front ⇒ a célula de baixo é segura».
     Corrija também o comentário de sim.rs (Sim::gotas) que diz que só a rota de Jacobi aplica a lei: a força
     ×4 e o drip_pass correm também na Gauss-Seidel (rota de testes).
L1-5 Com grid_ratio > 1 as constantes das gotas (CATCH_PER_SIZE, RUN_5, SPEED, MEANDER_PX, NECK_PX…) estão
     documentadas em px e o motor usa-as em CÉLULAS: a ratio 2 uma gota corre ~566 px contra ~266. Converta
     pela razão (o tool conhece-a; o motor recebe-a ou o DripPaper leva a escala). GATE: a cena do S a
     ratio 2 contra a sua própria tela direita, e o comprimento da gota em px a ratio 2 ≈ ratio 1.
L2-1 ⭐ BUG: as gotas de uma sprite passam para outra ao SAIR do Painter. set_source
     (tool/trait_impls_raster.rs) repõe todo campo por-documento (papel, heights, covers, mats, vidros)
     MENOS `gotas`; ao reentrar, bind_document não arruma o documento desmontado e abre_a_sprite → set_source.
     Repro: bind_document(1) → set_gotas{create:false,size 9,length 2,easy:true} → on_deactivate() →
     bind_document(2) ⇒ a 2 fica com as gotas da 1; voltar à 1 ⇒ DEFAULT. GATE: o teste da troca
     (the_paper_drips_travel_with_their_document_across_a_switch) com on_deactivate() entre os binds, e/ou um
     censo de que set_source repõe todo campo por-documento.
L2-2 ⭐ O Wet Drip de FÁBRICA nunca pinga em estado nenhum que o painel alcance (tela off, anéis 4/6/8, Easy):
     0 px abaixo da orla. GATE: o pincel cujo nome promete gotas pinga na tela no máximo (barra pelo
     oráculo: o «Dip Pen»/«Round» da tabela pinceis do 06 §2) — e regere a fábrica se a água dele mudar.
L2-3 A dica (ph2d-i18n painter_layers.rs, inerte.create_drips_on «…tilted past halfway») discorda da lei:
     com a gravidade 0,005 × anel/4 × 4 contra SLIDE 0,02828, só os anéis 6–8 soltam gotas; o anel 5 já
     passou de metade e esmaece. GATE: a dica deriva do primeiro anel em que wet_tilt_lets_drips_slide()
     acende (ou o texto diz o mesmo número) — depois da recalibração do L1-1, recalcule.
L2-4 Anel 0 com a inclinação «ligada» (SetValue(TILT_RING, 0) liga tilt_on; um arrasto até ao centro dá anel
     0): gravidade 0, Create Drips não age, mas NÃO esmaece (inercia.rs GotasDaTela usa só !tilt_on). CURA: a
     lei de inércia pergunta pela gravidade efectiva. GATE: o censo (censo_dos_controlos.rs pre_condicoes)
     ganha estados do Wet Paint COM a inclinação (anéis 0, 4, 5, 8) e janela de água mais longa — hoje só mede
     a fábrica, onde as quatro rows esmaecem e são inertes (verde trivial).
L2-5 Sequência de fábrica sem gotas e a explicação escondida: Paper nasce recolhido (populate_sections.rs) e
     o cartão Tilt não diz que as gotas pedem inclinação forte. Os controlos FICAM no Paper (dono). CURA dentro
     disso: a dica de Create Drips nomeia o «Tilt» e o anel mínimo; e decida (técnico) se o cartão Tilt ganha
     uma linha de texto/dica que aponte o Paper — sem mover controlos. GATE: de fábrica, «porque não há
     gotas» está visível na dobra por omissão (ou a dica do Tilt o diz).
L2-6 Nenhum Reset chega às gotas (watercolor_settings.rs: o Reset do Paper só repõe o slot da textura; o Reset
     do Wet Paint endireita a tela e deixa as gotas). CURA: o Reset do Paper repõe as gotas (DripPaper::DEFAULT).
     GATE: Reset depois de mexer em cada uma das quatro rows ⇒ DEFAULT (na_fabrica_nenhum_reset_muda_o_pincel
     só corre na fábrica: verde trivial).
L2-7 Ctrl+Z não desfaz as gotas (shells/desktop/src/undo.rs: só a cor do papel é passo de undo, papel.rs).
     CURA: as gotas entram no mesmo passo/diff do papel. GATE: desmarcar Create Drips → undo ⇒ volta.
L2-8 Drip Size/Length mostram decimais (5.036) ENQUANTO se arrasta (number_field.rs mirror_value salta a caixa
     focada; util.rs format_number 3 casas). CURA: campos inteiros arredondam no arrasto. GATE: o valor
     ENCAMINHADO pelo painel (não um SetValue fabricado: seam_wetpaint.rs as_gotas_do_papel… alimenta o tool
     com 9.0/12.0 inventados) é inteiro.
L2-9 Caminho morto: wet_tuning_id_map (wetpaint_settings.rs) ainda encaminha os ids slider/chip/reset da
     Gravity, e o reset do grupo Physics escreve a Gravity — nada os pinta hoje. CURA: KNOBS_DA_OBRA também
     filtra o mapa e o reset de grupo. GATE: nenhum id da Gravity é encaminhado.

GATES (red-first, cada um visto VERMELHO antes; barras do VALE entre o nosso de hoje e o oráculo a 1:1)
- Os de cada achado acima + mutações das leis novas no fecho (mutação que SOBREVIVE = equivalente provado
  ou a lei sai).
- As quatro cercas da Fase 0 verdes no fim. Os gates de hoje da bancada (gotas_pela_lei_do_rebelle: 8)
  continuam verdes OU são reescritos DE PROPÓSITO com a razão (a força recalibrada muda números): diga
  quais e porquê no commit e no 05.
- Pior tick das gotas ≤ 2 ms a 4096² (ph2d-wet-paint tests/it/drips.rs measure_drip_pass_cost, --profile
  smoke, com a load ao lado; acima de load ~5 o relógio não vale — repita). Kill-criterion: > 2 ms depois da
  2.ª tentativa ⇒ PARE e reporte antes da 3.ª.
- Determinismo ao bit (os testes do motor drips::). Inerte sem gravidade ao bit.

NÃO RECONSTRUIR / NÃO REABRIR (medido e recusado; ver 05 §4b)
- A faixa de transição na borda (o papel seco de fora a receber uma fração do fluxo, em 4 formas): inunda.
- A difusão do pigmento (ExtDiffusion) para suavizar a orla: inerte sobre o pigmento assente.
- A 1.ª linha de contacto (cortar só a componente de DESCIDA): faz os degraus e as bases planas
  (26 433 px fora da mancha). Fechar também os buracos de DENTRO: prende a água.
- A Gauss-Seidel do arrasto (doc 44 §1), as recusas do 01 §5, extFingering/extRivulets como solução.
- O oráculo só se reabre por gotas_inicia.sh / gotas_tela.sh (agora com --forma s / --size_px / --agua) /
  gotas_fecha.sh. ⛔ NUNCA o fecha.sh do oráculo, nunca o arranque.sh inteiro, nunca DISPLAY=:0 nem
  spectacle; um pgrep -f apanha o próprio shell.

REGRAS
- CLAUDE.md §0 inteiro. Inner loop = bash scripts/cargo-check-narrow.sh <crate>; testes/clippy 1× no fecho;
  comando pesado por bash scripts/ph2d-run.sh; cargo test num .sh, nunca em heredoc.
- Commit por caminho, --no-verify; ficheiro novo: git add antes. NUNCA push, --force, add -A. Tecto de 700
  linhas por ficheiro (drips.rs já tem ~770: parta-o — ex. drips/edge.rs, drips/trail.rs — ANTES de crescer).
- Sondas temporárias saem antes do commit. Medição com a TABELA ao lado (CLAUDE.md §0.0).
- Fecho: nextest dos crates tocados (ph2d-wet-paint, ph2d-tool-painter, ph2d-panel-painter-layers,
  ph2d-panel-wet-tuning, ph2d-i18n, ph2d-host-desktop) + clippy -D warnings + fmt + as quatro cercas;
  actualize o 05 (§4b a W1c, a tabela da recalibração, as recusas novas) e commite.
- A linha só fecha (/pd-linha-fechar) quando o dono mandar.

AO DONO: respostas curtas, sem jargão, o que ele consegue FAZER agora; o teste em passos numerados (o comando
com o cd, onde clicar com o nome do ecrã, o que tem de acontecer, como saber que deu errado) e uma imagem
ampliada antes/depois da borda (o nosso S contra o S do Rebelle, mesma ampliação).

Primeiro passo depois da Fase 0: diga ao dono, em poucas linhas e sem jargão, o plano (a borda como no
Rebelle, as gotas de toda a orla, os bugs) e comece — a ordem de corrigir já foi dada.
```
