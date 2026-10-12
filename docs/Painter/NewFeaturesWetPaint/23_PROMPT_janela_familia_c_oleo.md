TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»): a FAMÍLIA C (Oil Bristle · Express Oil · Acrylic Round)
contra o Rebelle — a tarefa 3 do prompt 22, que ficou por começar.

Os defeitos que o dono achou no smoke da família C estão fechados (#48 relevo, #49 verde, #50 orla clara — smoke
do dono OK em 12/10). Agora é a aparência dos três pincéis.

Leia SÓ: o handoff `docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_RELEVO_E_VERDE.md`
(R1–R9 no topo, valem a janela inteira; §5 e §7). Não releia o 05, o 18 nem o histórico da linha.

⚠️ **Ordem do dono (11/10): «faça como achar melhor e pare de me fazer perguntas».** Decida o TÉCNICO sem
perguntar. Pergunte só o que é PRODUTO, e a única pergunta de produto prevista aqui é mudar um valor que ele afinou
(abaixo). Em vez de perguntar, mostre a prancha.

## Fase 0 — onde você está

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
   → `line/PainterWatercolor`. Outra coisa: PARE e reporte.
2. `git log --oneline -3 && git status --short` → a ponta é `4a7193d3a` (Bug #50) ou o commit deste prompt;
   árvore limpa. `git fetch -q origin`; se o `main` andou: `git rebase main` (conflito fora dos seus ficheiros: PARE).
3. `bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10` → os números de ANTES vão para o handoff.
4. Shell `fish`/`zsh`: laços e listas de caminhos num `bash -c '…'`. `export CARGO_INCREMENTAL=0` em todo `ci-test`.
   Comando pesado SEMPRE por `bash scripts/ph2d-run.sh …` (o hook recusa sem ela).

## A tarefa, por ordem

**1. As pranchas de HOJE (antes de mexer em nada).** As curas #48–#50 mudaram cor e relevo; as pranchas de
`comparar_oleo/novos/` são de antes delas.
- `PH2D_SONDA_OUT=<dir na scratchpad> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --test it midias_de_fabrica`
  grava `midia_<preset>.ppm`.
- `bash docs/Painter/NewFeaturesWetPaint/ferramentas/prancha_comparar.sh <dir> <saída> "Oil Bristle|oil|…"
  "Express Oil|express|…" "Acrylic Round|oilround|…"` (fotos do Rebelle em `fixtures/rebelle/`, grelha A–J × 1–6
  igual nos dois). Grave em `comparar_oleo/janela23_antes/`.

**2. Os quatro pontos que o dono viu** (um pincel de cada vez; prancha antes e depois de cada um):
- **Oil Bristle:** está perto. Falta o pincel CHATO ser FINO quando anda de lado: o traço vertical do Rebelle,
  células D4–D6.
- **Express Oil:** mais OPACO e mais LISO que o do Rebelle. O dele é claro, com falhas entre os tufos.
- **Acrylic Round:** RALA (riscos finos). A do Rebelle é corpo cheio.
- **Pista das fotos:** o óleo do Rebelle tem cerdas de DUAS espessuras (finas e grossas); o nosso, uma.
  ⛔ Os `.rebelle-brush.png` do Rebelle trazem os parâmetros legíveis, mas são de outro motor (~180 números de
  «partículas»), sem correspondência com os nossos knobs, e a EULA do teste é uso pessoal: servem de PISTA,
  nunca de receita. As texturas dele não se copiam (`/pd-cleanroom` se for preciso ir além da foto).

Método (§0.0, §0.9, §0.10 do CLAUDE.md): antes de construir, MEÇA se a composição dos knobs que já existem
exprime o alvo (varredura de knobs numa rodada só, num build, por `override` em execução). Construa motor novo só
se a varredura provar que não chega. A régua de cada ponto é uma MEDIDA na foto do Rebelle (largura do traço de
lado, cobertura e falhas entre tufos, corpo), escrita na prancha. ⚠️ A `midias_de_fabrica` perdeu três métricas
que não mediam o que o olho do dono vê (18 §2): não as ressuscite como critério.

**3. Os valores do dono são o ponto de partida** (`tests/it/receitas_de_fabrica.rs`; regerar por
`PH2D_REGERA_PRESETS=1 … --profile smoke … gera_os_presets_de_fabrica -- --ignored`). Mudar um deles é a ÚNICA
pergunta de produto: mostre a prancha antes e depois, e pergunte uma vez, no fim, com todas as propostas juntas.

R1: ~150 chamadas. Se não fechar, feche no passo em que está, com a prancha e os números.

## O que NÃO é desta janela (registe, não toque)

O realce do Emboss por diferença central, que põe metade da encosta FORA de uma aresta grossa (handoff §7: medido,
não mudado; mexer muda o relevo que o dono afinou) · «Keep volumes» (o dono decide depois) · o R9 que sobra
(camadas 44,7 s · largura 24,7 s · mídias 14,8 s) · famílias A e B · tinta/caneta e Staining K–M (dono) · os flakes
de carga da máscara (`the_mask_stroke_cost…`, `the_cost_of_a_gated_stroke…`, já na tabela).

## Fecho

- Bateria pelo `verificador`, em série, `loadavg` ao lado: a cerca · `ph2d-wet-paint` em `dev` E em `ci-test` ·
  `ph2d-tool-painter` INTEIRO (lib + `--test it`) em `dev` E em `ci-test` · censos (`ph2d-panel-registry-init
  --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`, `ph2d-panel-painter-layers`,
  `ph2d-panel-wet-tuning`) · `ph2d-app-painter --features panel-painter-layers` · fmt e clippy `--all-targets
  -D warnings` nas crates tocadas · typos · tecto de 700 linhas · `scripts/nextest-impacted.sh` 1×.
- `bash scripts/agent-loop-profile.sh 10`; `rm -rf target/*/incremental`; apague os `CARGO_TARGET_DIR` da scratchpad.
- Handoff de continuação ≤ 8 KB com as R1–R9 no topo; entrada curta no `05d_estado_mistura_II_e_presets.md`.
  Commit por caminho (`--no-verify`, `git add` explícito para os novos). ⛔ Nunca push.
- Ao dono, curto e sem jargão: as pranchas antes/depois de cada pincel, e SMOKE em passos numerados (o comando com
  o `cd` DESTA worktree e `--profile smoke` · onde clicar, com o nome da tela · o que tem de acontecer · como saber
  que deu errado).
