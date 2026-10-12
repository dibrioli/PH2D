TROCA DE AGENTE — line/PainterWatercolor: o **Bug #51** — a imagem no Shape com «Use Texture Colors» cala o
Color (não tinge) e o Randomize Color. Achado do dono 12/10, aberto e MEDIDO na janela 23; a cura vem aqui —
junto com o **Randomize Color POR TRAÇO** (pedido do dono 12/10), que mexe no MESMO sorteio (§0.10: um bloco).

Leia SÓ: `docs/Painter/BUGS_painter.md` §Bug #51 (o mecanismo, os portões vermelhos e a cura desenhada) e o
topo (R1–R9) do handoff `docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_FAMILIA_C.md`.
Não releia a família C.

⚠️ **Ordem do dono (11/10): «faça como achar melhor e pare de me fazer perguntas».** Decida o técnico. O que o
dono pediu é produto e já está decidido: (1) o Color age como TINTA da textura; (2) o Randomize Color volta a ter
efeito; (3) ao associar uma imagem ao Shape com «Use Texture Colors» ligado, o Color vai a BRANCO (sem efeito);
(4) com «Per-Layer Color» ligado, o Color fica desativado e inativo; (5) descobrir se algo mais deixa de agir;
(6) o **Randomize Color tem DUAS opções**: «a cada carimbo» (o de hoje, o padrão) e «a cada traço» (um sorteio
por traço: o traço inteiro sai numa cor, o seguinte noutra). A referência do idioma é o Procreate (*Color
dynamics*: o jitter por carimbo e a variação por traço são controlos separados) — corra-o se precisar, não o leia.

## Fase 0 — onde você está

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
   → `line/PainterWatercolor`. Outra coisa: PARE e reporte.
2. `git log --oneline -3 && git status --short` → a ponta é o commit do Bug #51 aberto (ou o deste prompt);
   árvore limpa. `git fetch -q origin`; se o `main` andou: `git rebase main` (conflito fora dos seus ficheiros: PARE).
3. `bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10` → os números de ANTES vão para o handoff.
4. Shell `fish`: laços num `bash -c '…'`. `export CARGO_INCREMENTAL=0` em todo `ci-test`. Comando pesado SEMPRE
   por `bash scripts/ph2d-run.sh …`.

## A tarefa, por ordem

1. **Os portões primeiro:** `cargo test -p ph2d-tool-painter --lib textura_tinge -- --include-ignored` → o
   controlo verde e os três `diag_bug51_*` vermelhos. Só conta o vermelho sobre o que se viu assim.
2. **A cura (BUGS #51 §cura, (a)–(d)):** a lei `textura × tinta` nos três núcleos (`accumulate.rs`,
   `accumulate_batch.rs`, `stamp_color.rs` — e a chave do cache de cor tem de conter a tinta); o `Dab` com o
   desvio SORTEADO (`color_shift`) no lugar do `shift_colors_like`; a tinta por modo (uma camada = Color;
   Per-Layer = branco nas camadas de textura); o Color inativo no painel em Per-Layer; Color → branco ao
   associar a imagem e ao ligar o modo. Os 103 literais do `Dab`: pelos erros do compilador, com contagem.
3. **O painel:** o Color inativo tem de ser visível E não-clicável (o padrão de `paint_shape.rs:40-44`); um
   portão de painel no estilo de `ph2d-panel-painter-layers/tests/it/seam_texture_colors.rs`.
4. **Randomize Color «a cada traço»** (sobre o `color_shift` do passo 2 — por isso vem DEPOIS dele): o
   sorteio do desvio H/S/V passa a ser feito UMA vez por traço, em `Stroke::new` (`ph2d-painter-brush/src/
   stroke.rs:255`, a semente do traço), e todo carimbo herda-o; «a cada carimbo» é o `jitter::per_dab` de hoje.
   - Um campo no `BrushSpec` (enum, padrão = por carimbo ⇒ preset e projecto antigos ficam ao byte). Ele viaja
     onde os três `color_jitter_*` viajam: `wetpaint_preset/json.rs:195/238`, `snapshot.rs`, `jitter_settings.rs`
     (o Reset), e os ids/secção do painel (`ids/painter.rs`, `ids/painter_brush_sections.rs`). Confira se o
     `BrushSpec` entra no `.ph2dproj` — se sim, o degrau do schema conta-se (`python3 scripts/schema-recount.py`).
   - ⚠️ O fluxo do RNG: no modo «a cada traço» o sorteio sai do `per_dab`; os outros jitters (escala, rotação,
     posição) não podem mudar de bytes — portão: um pincel com Randomize «a cada carimbo» é byte-idêntico ao de
     antes, e um com Scale/Rotate jitter dá os mesmos carimbos nos dois modos.
   - O traço que se refaz por inteiro (Curve, Line, os métodos de preenchimento: o traço é re-carimbado a cada
     Move) tem de manter a MESMA cor durante o gesto — a cor é do traço, não do Move.
   - Painel: um seletor de duas opções na secção Randomize Color, texto por `ph2d-i18n` (zero string fixa).
   - Portões: (a) «a cada traço»: todos os carimbos de um traço têm a mesma cor, e dois traços seguidos têm
     cores diferentes; (b) funciona também com a textura do #51 (Color branco); (c) mutação «sorteia por
     carimbo» sangra (a).
5. **Sobrou mais alguma coisa?** Varra o resto do caminho com Shape de imagem (Wet Paint, Watercolor, Impasto,
   Composite: quem lê `rgb_image`/`resolved_colors`) e diga, com a linha, o que mais deixava de agir.
6. **Custo:** o HSV por píxel do Randomize no caminho dinâmico — meça antes/depois no MESMO processo
   (`per_layer_perf_live`, MEDIR_VELOCIDADE.md) e escreva o número.

R1: ~150 chamadas. Se não fechar, feche no passo em que está.

## Fecho

- Bateria pelo `verificador`, em série, `loadavg` ao lado: `ph2d-painter-brush` · `ph2d-tool-painter` INTEIRO
  (lib + `--test it`) em `dev` E `ci-test` · `ph2d-paint-gpu` · `ph2d-app-painter --features panel-painter-layers`
  · censos (`ph2d-panel-registry-init --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`,
  `ph2d-panel-painter-layers`) · fmt + clippy `--all-targets -D warnings` nas crates tocadas · typos · tecto 700 ·
  `scripts/nextest-impacted.sh` 1×.
- Handoff de continuação ≤ 8 KB com as R1–R9 no topo; o #51 vai para o índice dos fechados.
  Commit por caminho (`--no-verify`). ⛔ Nunca push.
- Ao dono, curto e sem jargão, SMOKE em passos numerados (o comando com o `cd` desta worktree e `--profile
  smoke` · onde clicar · o que tem de acontecer · como saber que deu errado): uma imagem colorida no Shape, Color
  vermelho tinge-a, Randomize Color varia-a carimbo a carimbo, e com várias camadas o Color fica cinzento; e o
  Randomize Color «a cada traço»: cada traço sai inteiro numa cor, o seguinte noutra.
