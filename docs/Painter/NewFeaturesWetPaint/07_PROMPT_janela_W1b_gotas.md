# 07 — PROMPT da próxima janela: W1b, as gotas pela lei do Rebelle

> Cole tudo o que está abaixo da linha numa janela NOVA. Escrito em 08/10, no fecho da janela da W1 e do
> preview vivo, depois da bancada das gotas ([06](06_gotas_rebelle_x_ph2d.md)) e da decisão do dono.

---

TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), onda W1b: as gotas pela lei do Rebelle

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro; a janela abre na raiz = main, e o mesmo
caminho relativo existe nas duas árvores — editar o errado compila e commita sem erro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → tem de dar .../Worktrees/line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -3 && git status --short
   → a ponta é o commit «docs(Painter 08/10): o prompt da janela W1b…» sobre 2a925859d; árvore limpa.
3. git fetch -q origin && git cherry main HEAD | grep -c '^+'   → 12 (os commits da linha).
   Se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
4. bash scripts/hw-profile.sh (workstation) · bash scripts/btrfs-health.sh
   · bash scripts/ph2d-run.sh bash scripts/cargo-check-narrow.sh ph2d-tool-painter
5. A CERCA tem de estar verde ANTES de tocar em produto:
   bash docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh
   bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --test it os_outros_meios_nao_mudam

LEIA (nesta ordem; só isto, depois o código)
1. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md (a cada passo).
2. docs/Painter/NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md — O TRACKER: §3 (a linha W1b e a barra),
   §4 (as três decisões do dono), §4b (o que a W1 e o preview vivo deixaram, e onde vive cada coisa).
3. docs/Painter/NewFeaturesWetPaint/06_gotas_rebelle_x_ph2d.md — INTEIRO: a lei do Rebelle (dados + bancada),
   a nossa ao lado, a proposta aprovada (§5), como repetir (§6). Fixtures: fixtures/rebelle_gotas/README.md.
4. 02_presets_para_wet_painter.md §6.9 (a nota: a gravidade sai do preset, a água desce).
5. Código:
   - crates/ph2d-tool-painter/src/tool/paint/{wetpaint.rs (WetPaintState: tilt_on/ring/spoke — «Boot = …
     ON, straight down, half radius (gate-pinned no-op reconcile)»), wetpaint_settings.rs (WetKnobs,
     reset_brush_wetpaint, as rotas do tilt e do Gravity), wetpaint_preset.rs (CAMPOS_*, o mapa engine na
     captura/aplicação), wetpaint_preset/preview.rs (CenaDoPreview / ChaveDoPreview)}
   - crates/ph2d-wet-paint/src/{solver.rs (553 l.), flow.rs, trail.rs (598 l.), tuning/defs.rs (Gravity,
     extFingering, extRivulets)}
   - crates/ph2d-panel-painter-layers/src/{paint_wetpaint.rs (a fileira Gravity), paint_wetpaint_tilt.rs,
     paint_watercolor_paper.rs (a secção Paper, em todos os meios)}
   - crates/ph2d-tool-painter/tests/it/{sonda_gotas.rs (a nossa metade da bancada), presets_de_fabrica.rs
     (RECEITAS e o gerador)}

DECISÕES DO DONO JÁ TOMADAS (não re-perguntar)
- Só o Wet Paint evolui; Digital/Watercolor/Impasto não mudam um byte (a cerca guarda-o).
- «O tamanho é a largura que se vê» (o gate entra com a W2). Sem rotação da caneta (CanvasPointer congelado).
- 08/10 — **as gotas seguem a lei do Rebelle, as quatro partes do 06 §5**, com a tela **DIREITA de fábrica**
  (não escorre até o artista ligar a inclinação). É a onda W1b, ANTES da W2.
- Decisões técnicas são suas (padrão-ouro); produto e envio são do dono.

OBJECTIVO DESTA JANELA — W1b (05 §3, 06 §5), as quatro partes NUM bloco (§0.10: mesma família, uma bancada)
1. **A gravidade sai do pincel e vai para a TELA.** O dial de inclinação (tilt_on/ring/spoke, a obra) passa a
   ser o ÚNICO dono de «corre ou não, para onde e quanto», e nasce DESLIGADO (o default e o Reset). Nenhum
   preset leva `gravity` (exclua-o do mapa engine na captura e na aplicação — uma lista de knobs «da obra»
   ao lado de CAMPOS_FORA, com o porquê; gate de que nenhum ficheiro de fábrica o tem). A fileira Gravity da
   secção deixa de ser uma segunda porta para a mesma pergunta (decida: sai, ou vira a magnitude do dial —
   uma porta só). ⚠️ O boot do motor tem a inclinação LIGADA: o «no-op reconcile» que um gate prega vai
   mudar — reescreva esse gate de propósito, com a razão, nunca o deixe verde por acaso.
   Calibre a escala com a bancada: Rebelle `SET_CANVAS_TILT (0,1)` = o máximo do nosso dial; `(0, 0,5)` = metade.
2. **As gotas são do PAPEL (a obra)**: Create Drips (on/off), Drip Size e Drip Length (1–10, 5, ligado por
   omissão — os do Rebelle), no mesmo dono do papel do documento e na secção Paper do painel quando o meio é
   o Wet Paint. Persistem onde o papel persiste (confira no código; se o ProjectFile ou o PaintedDocument
   ganharem campo: degrau NOVO no fim, recontado por `python3 scripts/schema-recount.py`; main em 187).
   Cada controlo: o painel escreve onde · quem lê · o leitor DECIDE? · seam test ph2d-ui-testkit (evento real
   → efeito) · i18n · tokens. O preview vivo segue-os (CenaDoPreview e ChaveDoPreview ganham os campos; o
   gate `a_chave_muda_com_cada_parametro…` ganha os casos).
3. **A água de fábrica desce para a do Rebelle** (06 §1.4, dados): Wet Round 0,29 · Wet Wash 0,24 ·
   Wet Dry Brush 0,09 · Wet Drip 0,49 («muita água», não «muita gravidade»); tire `gravity` das RECEITAS e
   regere a fábrica (`PH2D_REGERA_PRESETS=1`): o gate da miniatura fica vermelho e volta verde NO MESMO
   commit, dito no handoff.
4. **O mecanismo da gota no motor** (ficheiro irmão em ph2d-wet-paint, ex. `drips.rs`; solver/trail no
   tecto de 700): numa área molhada inclinada, a água que passa o limiar forma POUCAS gotas (1–2 por traço)
   FINAS e LONGAS a partir da orla de baixo; Size/Length altos ⇒ funil largo até ao fundo; Create Drips off ⇒
   sem fios, mas o pigmento desce para a orla de baixo do traço (a orla escurece). Determinismo ao bit
   (ADR-0134/0147); pior tick ≤ 2 ms (ADR-0134) MEDIDO a 4096² com a tabela ao lado. Kill-criterion: se o
   pior tick passar de 2 ms depois da 2.ª tentativa, PARE e reporte antes da 3.ª. ⛔ Os knobs escondidos
   `extFingering`/`extRivulets` já foram medidos INERTES para isto (06 §3) — não são a solução.

GATES (red-first, cada um visto VERMELHO antes; a barra é a do 05 §3 W1b, tirada do vale do 06)
- A bancada das gotas vira gate (uma versão rápida e não ignorada de `sonda_gotas`, numa tela menor se a
  régua o permitir — meça): tela direita e inclinação a meio ⇒ 0 · inclinação máxima ⇒ gota ≥ 250 px nos
  traços de água ≥ 0,7, 0 nos de 0,4 e 0,2, 1–2 gotas por traço, ≤ 4 px de largura · Create Drips off ⇒ 0
  com a orla de baixo escura · Size/Length 10/10 ⇒ ≥ 450 px · controlo: as gotas no neutro e a inclinação
  desligada ⇒ o traço de hoje ao byte. HOJE estes números dão 0 e 20 px: o vermelho é o ponto de partida.
- Nenhum preset de fábrica nem capturado leva `gravity`; o dial nasce desligado; o Reset desliga-o.
- Seams dos controlos novos; preview vivo segue cada um; a cerca verde; mutações das leis novas no fecho.

NÃO RECONSTRUIR / NÃO REABRIR
- A transferência Gauss-Seidel dependente do sentido (doc 44 §1, dono 29/09).
- As recusas do 01 §5 (Mixbox, texturas do Rebelle, ler o executável, injecção de rato para pintar, SAVE por
  WebSocket) e as do 06 (extFingering/extRivulets como solução).
- O oráculo: as fixtures das gotas JÁ ESTÃO gravadas. Só se reabre se uma medição NOVA for precisa, e então
  pelos `ferramentas/gotas_inicia.sh` / `gotas_tela.sh` / `gotas_fecha.sh`. ⛔ NUNCA o `fecha.sh` do oráculo
  (mata TODA tela virtual, as das outras linhas também), nunca o `arranque.sh` inteiro (DISPLAY=:1 à mão),
  nunca DISPLAY=:0 nem spectacle; e um `pgrep -f <padrão>` apanha o próprio shell — feche pelo nome exacto.

REGRAS
- CLAUDE.md §0 inteiro. Inner loop = bash scripts/cargo-check-narrow.sh <crate>; testes/clippy 1× no fecho;
  comando pesado por bash scripts/ph2d-run.sh; cargo test nunca em heredoc — num ficheiro .sh.
- Commit por caminho, --no-verify, com array dentro de `bash -c`; ficheiro novo: git add antes. NUNCA push,
  --force, add -A. Tecto de 700 linhas por ficheiro.
- clippy `-D warnings` e os censos do painel (`cada_nome_deste_painel_cabe_na_coluna…`, HR-15) apanharam
  coisas na W1: corra-os no fecho, não os deixe para o ship.
- No fim da janela: actualize o 05 (§3 W1b ✅, §4b estado, §5 a W2) e commite. A linha só fecha
  (/pd-linha-fechar, handoff com o smoke do dono em passos numerados, a cena fotografada) quando o dono mandar.

NOTA PARA O INTEGRADOR (já no 05; repita no handoff final)
- O checkout principal tem cópias por commitar de docs/Painter/NewFeaturesWetPaint/ e de
  crates/ph2d-tool-painter/tests/sonda_tracos_por_meio.rs (a linha editou os docs e mudou a sonda para
  tests/it/). Remover as cópias do principal ANTES do --ff-only.

AO DONO: respostas curtas, sem jargão, o que ele consegue FAZER agora.
Primeiro passo depois da Fase 0: diga-me o desenho da W1b (onde vive cada parte, a calibração da inclinação,
o mecanismo da gota em uma frase, os gates) e espere o «pode começar».
