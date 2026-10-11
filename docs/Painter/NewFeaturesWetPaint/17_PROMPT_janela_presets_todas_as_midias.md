TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), os PRESETS DE TODAS AS MÍDIAS: o mapa das mídias
medido contra o Rebelle, a largura prometida (W2, a base de todo preset) e o 1.º lote de presets novos.

Pedido do dono (11/10): *«é hora de retornar aos presets do wet paint … o objetivo é criar presets de todas as
mídias (tinta a óleo, acrílica, lápis, caneta, aquarela, nanquim, giz de cera, etc, etc). Já temos 4
presets.»* As correcções da mistura fecharam (05 §4b «Mistura II»); o motor está no `main` em `8f20c60de`.

⛔ A DECISÃO QUE ENQUADRA TUDO (dono, 07/10; memória `project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07`):
*Digital, Impasto e Watercolor NÃO mudam; só o Wet Paint evolui e passa a simular TODAS as mídias.* ⇒ todo preset
novo é `media = WetPaint`, pintado pelo motor `ph2d-wet-paint`. O doc 02 §6.1–6.8 escreveu os presets de arranque
para os OUTROS meios (`PaintMedia::Watercolor`, `Impasto`, `Digital`) ANTES dessa decisão: os VALORES e a
gramática servem de ponto de partida, o meio de destino NÃO. A cerca `os_outros_meios_nao_mudam` continua a
exigir os outros três meios ao byte.

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -3 && git status --short → a ponta é este prompt sobre `8f20c60de`; árvore limpa.
   git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
3. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por `bash scripts/ph2d-run.sh`;
   o hook recusa «cargo test» fora da porta: a bateria vai num ficheiro (Write) e corre por
   `bash scripts/ph2d-run.sh bash <ficheiro>` (a shell é fish/zsh: laços e arrays em bash vão no ficheiro).
4. Cercas verdes ANTES de tocar em produto (num só ficheiro de bateria): `ferramentas/cerca.sh` (com
   `cerca_isencoes.tsv`) · a suíte de ph2d-wet-paint (as impressões do port não movem) · `--lib wetpaint` e
   `--test it` do ph2d-tool-painter (antes deles os presets: `PH2D_REGERA_PRESETS=1 … --test it
   gera_os_presets_de_fabrica -- --ignored`, que tem de sair SEM diferenças) · o censo dos controlos
   (`cargo test -p ph2d-panel-registry-init --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`
   e `cargo test -p ph2d-panel-painter-layers`) · a ponte do preview com `--features panel-painter-layers` no ph2d-app-painter.

LEIA (só isto, nesta ordem): DIRETIVA_IMPLEMENTACAO.md (inteira; releia a cada passo) · 02 §3 (as medianas
por mídia e «O que faz cada mídia parecer o que é»), §4 (o que falta ao nosso pincel), §5 (o formato), §6 (os
valores de arranque — ver o ⛔ acima) · 01 §2 (as leis L1–L7 e as explicações REFUTADAS), §4 (as ondas), §5
(recusas) · 03 (a bancada e as 7 métricas; os números de cada mídia do Rebelle) · 05 §3 (as ondas com a barra),
§4 (decisões do dono), §4b «Conferência dos 4 pincéis», «A claridade é o PIGMENTO», «Mistura II» · o handoff
`docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-11.md` §8 (ABERTO). O código, por
agentes `explorador`, não por você.

O QUE JÁ EXISTE (não se reconstrói):
- O formato e a fábrica: crate `ph2d-brush-presets` (`*.ph2d-brush.png`, `SCHEMA=1`, miniatura 256²), a pasta
  `assets/brushes/WetPainter/<Meio>/<Grupo>/<Nome>.ph2d-brush.png`, o gerador
  `crates/ph2d-tool-painter/tests/it/presets_de_fabrica.rs` (`RECEITAS`, `PRESETS_DE_FABRICA = 4` — um número
  que SOMA: conta-se, nunca se escolhe), o navegador no painel (`paint_wetpaint_presets.rs`, a ponte
  `painter_bridge_presets.rs`), a biblioteca do utilizador (`$XDG_DATA_HOME/ph2d/brushes`).
- Os 4 de fábrica, todos aquarela fluida (`Watercolor/Fluid/`): Wet Round · Wet Wash · Wet Dry Brush · Wet
  Drip, calibrados contra o Rebelle (02 §6.9; carga, largura, orla, papel, escorrimento).
- Os campos que o Wet Paint LÊ (`CAMPOS_DO_PINCEL`, `wetpaint_preset.rs`), os volumes (Size px, Opacity =
  quantidade de pigmento, Water), e as opções guardadas no preset (Tilt, Live Water, Heavy Edge, Smooth Edges).
  Campos NAO_LIDO hoje: `flow`, `accumulate`, `airbrush_rate_s`, `space_attenuation`.
- Os portões: `the_factory_presets_are_counted`, `cada_preset_de_fabrica_volta_ao_byte`,
  `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, `o_pincel_guardado_pinta_o_mesmo_traco`,
  `os_outros_meios_nao_mudam`, `com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`.
- O oráculo: Rebelle 8.3.4 sob Wine pela porta WebSocket (`ferramentas/`, memória
  `reference_rebelle_trial_installed_under_wine`); 281 presets em
  `~/Apps/rebelle/wineprefix/drive_c/Program Files/Rebelle 8/Brushes{, Bristle}/` — mídias: Watercolor · Oil
  · ExpressOil · Pencil · Pastel · Ink · Marker · Airbrush (+ utilitários Blend, Smudge, Wet, Dry, Eraser,
  Clone); `ferramentas/extrai_pinceis.py` lê-os, mas a saída NÃO está versionada (o 02 cita uma corrida
  perdida). Fotos de traço por mídia em `fixtures/rebelle/` (oil, oildirty, express, ink, marker, pastel,
  pencil, penciltilt, airbrush, wc*), medidas no 03.

⛔ RECUSAS E LEIS QUE UM PRESET NOVO NÃO PODE VIOLAR: as texturas do Rebelle (forma/grão/papel) são assets do
alvo — nunca se copiam (as nossas são procedurais ou imagens NOSSAS em `assets/brushes/texturas/`) · Mixbox
(CC BY-NC) não é porta; a mistura é a K–M nossa · nenhum preset guarda Gravity (0 de 281 no Rebelle) · a
claridade é o PIGMENTO, nunca a diluição (recusa do dono) · nem escalar o carimbo para clarear (medido) · as
gotas (Create/Easy Drips) foram desfeitas pelo dono · sem rotação da caneta (decisão do dono, Wacom Intuos5
sem barrel) · a tela de fábrica é direita (só o Drip inclina).

PLANO (uma janela; §0.10 — cada fase num bloco, uma rodada de medição; série só onde o resultado de uma fase
muda a seguinte, e diga-o):

A. **O MAPA DAS MÍDIAS, medido antes de construir** (CLAUDE.md §5.0: *meça se a composição já o exprime*).
   1. Corra `extrai_pinceis.py` sobre os 281 presets e versione a SAÍDA (só os parâmetros em texto, com
      cabeçalho: versão, data, comando) como fixture, p.ex. `fixtures/rebelle_presets_281.json`. ⛔ Nunca as
      imagens de forma/grão embutidas.
   2. A tabela das mídias do dono → oráculo: óleo → Oil · acrílica → ExpressOil (diga se o Rebelle tem outro
      mais perto) · lápis/grafite/carvão → Pencil (que presets) · caneta → Ink (bullet, dip pen) e Marker ·
      aquarela → Watercolor (+ o Gouache Round que vive lá) · nanquim → Ink · giz de cera/pastel seco/pastel
      de óleo → Pastel (que presets) · marcador → Marker · aerógrafo → Airbrush · + o que mais o dono disser
      com «etc» que o Rebelle cubra. Para cada uma: os presets-oráculo pelo nome; o que a faz parecer o que é
      (02 §3); os campos do NOSSO preset que a exprimem; o que falta ao motor e que onda o entrega (W2 fios e
      largura · W3 altura/oleosidade · W4 papel × pressão · W5 lei da tinta · W6 dinâmicas · W7 caminho sem
      água · o #47 do Shape Noise para os secos com grão).
   3. A MEDIÇÃO que decide o que é honesto hoje: para cada mídia, a melhor composição dos knobs de hoje (uma
      sonda `#[ignore]` que varre o preset-candidato, sem tocar em produto) contra a foto do Rebelle da mesma
      mídia, nas 7 métricas do 03, à LARGURA MEDIDA igual (05 §4.1). Resultado: a tabela «mídia · melhor
      candidato · métricas · barra · dentro/fora · o que a impede». ⇒ É esta tabela que diz quais presets o
      motor já pinta COMO a mídia e quais seriam um Wet Round com outro nome (⛔ «feature pior que não existir»:
      um preset «Lápis» que pinta aquarela ensina o contrário ao dono).

B. **W2 — a largura prometida** (05 §3, §5; decisão do dono §4.1: o número do painel é a largura medida ± 2 px,
   em TODAS as mídias): o raio do motor sai da `width_table` do preset em `stamp_dabs_wetpaint`; gate
   «largura = 2·raio ± 2 px» de 2 a 300 (hoje 22–30 px a 40). Vem ANTES do lote de presets porque muda a lei
   do motor: calibrar presets antes dela era calibrá-los duas vezes. Regere a fábrica no MESMO commit da lei
   (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` fica vermelho — é o esperado; diga-o) e re-meça os 4
   de fábrica contra o 02 §6.9 (a largura sobe para a do Rebelle; carga, orla e escorrimento não podem sair
   da barra). Os FIOS com física própria (o resto da W2) entram nesta janela SÓ se a Fase A mostrar que
   óleo/acrílica não passam a barra sem eles e couberem na sessão; senão ficam para a janela seguinte com o
   plano escrito.

C. **O 1.º LOTE de presets novos — só os que a Fase A mediu DENTRO da barra** (depois da W2). Esperado (a
   confirmar pelos números): a família aquarela completa pelos valores do 02 §6.1 re-apontados ao Wet Paint
   (Flat, Gouache Round, os que faltam), nanquim/tinta lavada (Ink com água), e o que mais a tabela aprovar.
   Para CADA preset:
   - uma `Receita` no gerador, a pasta `WetPainter/<Meio>/<Grupo>/` (o meio pelo nome do Rebelle em inglês,
     como o `Watercolor` de hoje), a miniatura que o motor pinta, `PRESETS_DE_FABRICA` RECONTADO;
   - medido na bancada (`ferramentas/presets_regua.py` + as 7 métricas) contra o preset-oráculo, a linha na
     tabela do 02 §6.9 (nosso · Rebelle);
   - um portão NOVO por mídia, `cada_preset_parece_a_sua_midia` (ou um por mídia), com a barra tirada da foto
     do Rebelle e o CONTROLO DENTRO: o Wet Round na mesma bancada REPROVA a barra dessa mídia (sem isso, o
     portão aprova um Round renomeado).
   - O nome do preset é conteúdo (lido do ficheiro); se o navegador mostrar o nome do GRUPO ou do MEIO como
     rótulo de interface, isso vai por `ph2d-i18n` (HR-15) — confira em `paint_wetpaint_presets.rs`.

D. **O plano das mídias que ficam de fora**, escrito no 05 §3/§5 e no 00_README: para cada uma, a onda que a
   desbloqueia, a barra (das fotos do Rebelle) e a ordem proposta. Se a ordem depender do que o dono quer
   primeiro (p.ex. óleo/acrílica pelos fios e a altura, ou lápis/carvão pelo papel × pressão, ou os secos —
   giz de cera, pastel, marcador, caneta, aerógrafo — pelo caminho sem água), pergunte-lhe SEM JARGÃO, com o
   que ele ganha em cada opção.

AS DECISÕES QUE NÃO SÃO SUAS: tirar ou renomear os 4 presets de hoje; a ordem das mídias secas se ela mudar o
que o dono vê primeiro; o Staining com a K–M (handoff §8.1). Pergunte; não decida.

PORTÕES E CERCAS NO FECHO (um ficheiro de bateria): as cercas da Fase 0 verdes DEPOIS · os portões novos com o
VERMELHO de antes registado (o gate da largura reprova no motor de hoje; o de cada mídia reprova o Wet Round) ·
provas de mutação (agente `mutacao`, em série, restauro por cópia + `touch`): a largura volta à lei antiga;
um preset de mídia perde o campo que o distingue; o gerador conta errado · `cargo fmt --check` e clippy
`--all-targets -D warnings` nas crates tocadas · o tecto de 700 linhas por ficheiro.

FECHO: 05 §4b entrada nova «Presets de todas as mídias — o mapa, a W2 e o 1.º lote» (o mapa da Fase A com os
números, a W2 antes · depois · Rebelle, cada preset novo com a sua linha, as recusas medidas, os abertos) · o 02
§6.9 com as linhas novas · uma prancha 1:1 (cada preset novo ao lado do preset-oráculo do Rebelle) · commit por
caminho (`--no-verify`; `git add` explícito para os novos) · o binário do smoke compilado (`bash
scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2×, a 2.ª saída no handoff) · nunca push.
AO DONO, curto e sem jargão: que mídias já têm preset, quais ainda não e porquê (o que o motor ainda não sabe
fazer), o que ele tem de decidir, e o smoke em passos numerados (o comando com o `cd`; Painter ▸ Wet Paint ▸ o
navegador de pincéis; cada preset novo com o nome que aparece na tela; o que tem de acontecer — o traço parece
a mídia do nome; como saber que deu errado).
