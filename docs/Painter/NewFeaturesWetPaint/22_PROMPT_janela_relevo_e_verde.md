TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»): o RELEVO que some (#48), o VERDE no soltar (#49) e a
família C (óleo · acrílica) a seguir.

O dono escolheu a família C (óleo, acrílica) para começar (11/10) e afinou a olho os três presets dela contra o
Rebelle — os valores dele já estão nos ficheiros de fábrica e mandam («vamos deixar esses valores e seguir»).
No mesmo smoke ele achou dois defeitos, que vêm PRIMEIRO.

Leia SÓ: o handoff `docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_COMPILADOR.md`
(R1–R9 no topo, valem a janela inteira; §7 = o que mudou depois dele) e os Bugs **#48** e **#49** em
`docs/Painter/BUGS_painter.md`. Não releia o 05 nem o histórico da linha.

## Fase 0 — onde você está

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
   → `line/PainterWatercolor`. Outra coisa: PARE e reporte.
2. `git log --oneline -3 && git status --short` → a ponta é o commit dos presets do dono (sobre `d3e9d8f0d`);
   árvore limpa. `git fetch -q origin`; se o `main` andou: `git rebase main` (conflito fora dos seus ficheiros: PARE).
3. `bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10` → os números de ANTES vão para o handoff.
4. Shell `fish`/`zsh`: laços e listas de caminhos num `bash -c '…'`. `export CARGO_INCREMENTAL=0` em todo `ci-test`.

## O que mudou depois do handoff (não refaça)

- **Spacing do Tuning OCULTO** (`KnobGroup::Hidden`): no pincel do produto a distância entre marcas é o Spacing
  do STROKE (`BrushSpec::spacing`, fracção do diâmetro); o do Tuning só rola a janela do rasto — medido: 8 contra
  2 = imagem idêntica, 0,5 = 0,1 % dos pixels. O motor guarda-o (os presets antigos abrem iguais).
- **Presets do dono** (`tests/it/receitas_de_fabrica.rs`, regenerados por `PH2D_REGERA_PRESETS=1 …
  gera_os_presets_de_fabrica`; os outros nove ficaram ao byte): Oil Bristle water 0,1 · emboss 0,7 · cerdas 2000 ·
  força 2 · tamanho 1 · pigmento 2000 · spacing 0,01 · Acrylic Round opacity 0,6 · pigmento 500 · força 2 ·
  cerdas 4000 · tamanho 0,5 · spacing 0,01 · Express Oil water 0,09 · opacity 0,6 · pigmento 750 · força 2 ·
  tamanho 2 · cerdas 900 · emboss 0,2 · spacing 0,01. A `Receita` ganhou `spacing: Option<f64>`.
- **`midias_de_fabrica`:** saíram, por decisão do dono, a Textura do Oil Bristle (8,46 contra 5,86) e da Acrylic
  (30,4 contra 3,13) e a Borda do Express Oil (0,66 contra 0,80) — a régua não mede o que o olho dele vê (18 §2).
- **⚠️ O que confundiu o dono:** com «On switch: Keep volumes…» o painel guarda Size/Water/Opacity do pincel
  ANTERIOR ao trocar de preset — ele leu 0,048/1 no Express Oil e achou que os valores não tinham entrado. E o
  app lê os presets da pasta onde foi compilado (`CARGO_MANIFEST_DIR/../../assets/brushes`): o smoke corre DESTA
  worktree. Pergunte ao dono se o padrão «Keep volumes» é o que ele quer (não mude sem ele).
- **Pranchas** PH2D × Rebelle com grelha A–J × 1–6: `docs/Painter/NewFeaturesWetPaint/comparar_oleo/` (antes) e
  `…/novos/` (valores do dono); o script está na §7 do handoff.

## A tarefa, por ordem

**1. Bug #48 — o relevo de TODA a tinta some ao trocar para um pincel sem relevo** (o dono viu-o duas vezes).
Causa lida (por medir): o Emboss é um knob do RENDER aplicado com o valor do pincel de AGORA. Primeiro o portão
vermelho: pinta com Oil Bristle, troca para Wet Round, pinta ao lado → o 1.º traço tem de guardar o relevo ao
byte. Cura no mecanismo: o relevo viaja com a tinta (uma altura por célula escrita no depósito, como o `impasto`
do Impasto); o render lê a altura. Prove que um pincel SEM relevo não muda um byte do que já estava na tela, e
que os portões ao byte (`a_miniatura…`, `com_as_opcoes_desligadas…`, `smooth_edges_no_zero…`,
`os_outros_meios_nao_mudam`) mudam só onde há relevo — se mudarem, mostre ao dono antes de regenerar.

**2. Bug #49 — azul escuro sobre azul claro fica VERDE no soltar do rato, com *Pigment mixing* desligado.**
Durante o traço o azul está certo; só no mouse up vira verde; «depois do Undo o problema sumiu». Pergunte ao dono,
na 1.ª mensagem, se os traços SEGUINTES deixaram de ficar verdes depois do Undo (a resposta separa estado da
sessão de lei do soltar). Reproduza pela porta do produto: traço escuro sobre claro, foto antes e depois do
`pointer_up`, caixa ligada e desligada. Suspeita (por medir): o caminho do soltar (commit à camada / Glaze /
Staining do secar) mistura por K–M sem ler a caixa. `/pd-bug-causa`: mecanismo, medição, e o portão que estava
verde.

**3. Família C — o que o dono vê nas pranchas novas** (só depois de 1 e 2; mostre a prancha antes e depois):
- Oil Bristle está perto; falta o pincel CHATO ser fino quando anda de lado (o traço vertical do Rebelle, D4–D6).
- Express Oil está mais OPACO e mais LISO que o do Rebelle (o dele é claro, com falhas entre tufos).
- Acrylic Round está RALA (riscos finos); a do Rebelle é corpo cheio.
- Pista vista nas fotos: o óleo do Rebelle tem cerdas de DUAS espessuras (finas e grossas); o nosso, uma.
  ⛔ Os ficheiros `.rebelle-brush.png` do Rebelle trazem os parâmetros legíveis, mas são de outro motor (~180
  números de «partículas»), sem correspondência com os nossos knobs, e a EULA do teste é uso pessoal: servem de
  PISTA, nunca de receita; as texturas dele não se copiam (`/pd-cleanroom` se for preciso ir além da foto).
- Os valores do dono são o ponto de partida: mudar um deles é decisão dele (mostre a prancha e pergunte).

R1: ~150 chamadas. Se não fechar, feche no passo em que está, com o portão e os números.

## O que NÃO é desta janela (registe, não toque)

O R9 que sobra (camadas 44,7 s · largura 24,7 s · mídias 14,8 s) · as famílias A e B · a tinta/caneta (dono) ·
o Staining com K–M (dono) · a falha de carga da Luz 2D no `nextest-impacted` (`o_rebote_nao_depende_da_oitava`,
outra linha).

## Fecho

- Bateria pelo `verificador`, em série, `loadavg` ao lado: a cerca · `ph2d-wet-paint` em `dev` E em `ci-test` ·
  `ph2d-tool-painter` INTEIRO (lib + `--test it`) em `dev` E em `ci-test` · censos (`ph2d-panel-registry-init
  --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`, `ph2d-panel-painter-layers`,
  `ph2d-panel-wet-tuning`) · `ph2d-app-painter --features panel-painter-layers` · fmt e clippy `--all-targets
  -D warnings` nas crates tocadas · typos · tecto de 700 linhas · `scripts/nextest-impacted.sh` 1×.
- `bash scripts/agent-loop-profile.sh 10`; `rm -rf target/*/incremental`; apague os `CARGO_TARGET_DIR` da scratchpad.
- Handoff de continuação ≤ 8 KB com as R1–R9 no topo; entrada curta no `05d_estado_mistura_II_e_presets.md`; os
  Bugs #48/#49 fechados vão para o índice (`BUGS_painter_fechados_41_a_46.md`, uma linha com o MECANISMO). Commit
  por caminho (`--no-verify`, `git add` explícito para os novos). ⛔ Nunca push.
- Ao dono, curto e sem jargão, com SMOKE em passos numerados (o comando com o `cd` DESTA worktree e
  `--profile smoke` · onde clicar, com o nome da tela · o que tem de acontecer · como saber que deu errado),
  porque os dois defeitos são coisas que ele vê.
