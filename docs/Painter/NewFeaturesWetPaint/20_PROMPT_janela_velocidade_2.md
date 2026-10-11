TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»): a 2.ª onda da VELOCIDADE. A 1.ª (prompt 19) ligou a
otimização do motor e deixou a suíte do Painter em 174 s, toda verde; falta o R9 (nenhum teste > 10 s) e um
portão de bits que falhou sob carga. A fila de produto continua à espera.

Leia SÓ o handoff `docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_VELOCIDADE.md`
(8 KB; as regras R1–R9 estão no topo dele e valem a janela inteira). Não releia o 05 nem o histórico da linha.

## Fase 0 — onde você está

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
   → `line/PainterWatercolor`. Outra coisa: PARE e reporte.
2. `git log --oneline -3 && git status --short` → a ponta é este prompt, sobre `140955656`; árvore limpa.
   `git fetch -q origin`; se o `main` andou: `git rebase main` (conflito fora dos seus ficheiros: PARE e reporte).
3. `bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10` → os números de ANTES vão para o handoff.
4. O shell é `fish`/`zsh`: listas de caminhos (`git add`, `commit -- …`) dentro de `bash -c '…'`.
   `cargo-check-narrow.sh` é cego a `tests/it`: para lá, `bash scripts/ph2d-run.sh cargo check -p ph2d-tool-painter --test it`.

## A tarefa, por ordem

**1. `paper_rows` — a corrida no bake paralelo (PRIMEIRO; é correção, não velocidade).**
`ph2d-wet-paint::it paper_rows::the_paper_bake_by_rows_is_the_serial_bake` (`crates/ph2d-wet-paint/tests/it/paper_rows.rs:31`,
compara serial × `Rows::Parallel` ao bit) falhou 1× no `nextest-impacted` a load ~90 e passou 3/3 sozinha. Um
portão de BITS que falha sob carga é corrida, não relógio.
- Reproduza ANTES de ler: o teste em laço (≥ 200×) com a máquina carregada (`stress-ng --cpu 16` em fundo, com
  prazo), a `opt-level` 2 (a de hoje) e a 0 (`--config 'profile.dev.package.ph2d-wet-paint.opt-level=0'`, num
  `CARGO_TARGET_DIR` próprio na scratchpad — ⛔ nunca o `target/` de outra worktree). Conte as falhas por variante.
- Depois leia `bake_paper_rows` (`crates/ph2d-wet-paint/src/paper.rs:321`) contra as três condições do `par.rs`
  (cada linha escreve só a sua; o que lê, ninguém escreve no passe; nenhuma redução cuja ordem importe). Procure
  também estado partilhado fora da grelha (memo, `static`, cache do papel: `paper_memo.rs`).
- Cura no MECANISMO, não no teste; o portão fica como está e passa ≥ 200× sob carga. Se não reproduzir em 200×
  sob carga, diga-o com o número e passe ao 2 (não invente uma causa).

**2. R9 — os testes > 10 s, um a um, pela cura (b): fixture menor que ainda MORDA.**
Lista de partida (sob a suíte, depois da o2; refaça-a com `cargo nextest run -p ph2d-tool-painter --status-level pass`):
camadas `n_camadas_de_amarelo…` 159 · poças da aquarela `a_separacao_das_pocas_e_exacta_na_fronteira` 151 (25 s
sozinha) · achatamento do S 140 · `a_chave_muda_com_cada_parametro…` 113 · `a_largura_e_a_prometida` 105 ·
`cada_preset_parece_a_sua_midia` 98 · heavy edge 93 · opções ao byte 91 · `sem_celulas_soltas` 67 · e ~20 entre 10 e
41 s. Comece pelos de maior tempo. Para cada um:
- Meça ONDE vai o tempo antes de cortar (quadros depois de a tinta assentar? tamanho da tela? número de cenas?).
  Já medido nas mídias: os 1 320 quadros depois dos 8 s da cena são ~87 % do custo; ⛔ trocá-los pelo botão Dry
  canvas MUDA a régua (o Express Oil reprova) — recusado. Ideia por medir: o quadro a partir do qual a tela já não
  muda um byte (ponto fixo) e parar aí, provando que a imagem final é a MESMA ao byte.
- Na largura, a parede é o raio 300 (tela 1000 × 1880, 480 quadros): meça a faixa da altura do traço e menos
  quadros depois de a água assentar.
- Uma fixture que dá a MESMA imagem ao byte não precisa de mutação nova. Uma que muda a imagem: o agente
  `mutacao` prova que ainda morde — as mutações registadas são M1 (largura, `dab_spec.rs:~57`), M8 (grão,
  `dab_route.rs:~185`), M2 (`pigmentPerDab` ao aplicar o preset), MT (`paper.rs:295`, o `dente`), e as do §5.1 do
  handoff `HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-11_PRESETS_MIDIAS.md`. Têm de continuar VERMELHAS.
- Ficou < 10 s: retire o teste do override de 6 × 60 s em `.config/nextest.toml` no MESMO commit (catraca: um
  override sem razão medida é licença).
- ⛔ Lei nunca vai para `#[ignore]` por ser lenta. ⛔ Não mexa no gerador da fábrica nem na grelha `PONTOS_DA_LEI`
  (64) — a fila paralela foi medida e recusada (545 → 577 s).

R1: ~150 chamadas. Se a lista não acabar, feche no teste em que está e passe o resto à janela seguinte, com os
números.

## O que NÃO é desta janela (registe, não toque)

O relevo que some ao trocar de pincel · os pincéis de óleo longe do Rebelle · a decisão do dono sobre a
tinta/caneta · o Staining com K–M (§8 do handoff PRESETS_MIDIAS).

## Fecho

- Bateria num ficheiro, pelo `verificador`, em série, `loadavg` ao lado: a cerca
  (`docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh`) · `ph2d-wet-paint` · `ph2d-tool-painter` INTEIRO (lib
  toda + `--test it`: a bateria `--lib wetpaint` não via o Tooth) · `paper_rows` em laço sob carga · os censos
  (`ph2d-panel-registry-init --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`,
  `ph2d-panel-painter-layers`, `ph2d-panel-wet-tuning`) · `ph2d-app-painter --features panel-painter-layers` · fmt e
  clippy `--all-targets -D warnings` nas crates tocadas · tecto de 700 linhas · `scripts/nextest-impacted.sh` 1×.
  Falha só sob carga: sozinha com o `loadavg` ao lado (`docs/DevOps/FLAKES_DE_CARGA.md`; membros conhecidos:
  `a_wet_move_costs…`, `the_water_runs_at_a_usable_rate…`).
- `bash scripts/agent-loop-profile.sh 10`; `rm -rf target/*/incremental`; a máquina limpa.
- Handoff de continuação ≤ 8 KB com as R1–R9 no topo, a tabela ANTES × DEPOIS por teste, as recusas medidas e o
  que ficou > 10 s com a razão. Entrada curta no `05d_estado_mistura_II_e_presets.md`. Commit por caminho
  (`--no-verify`, `git add` explícito para os novos). ⛔ Nunca push.
- Ao dono, curto e sem jargão: se a falha do papel era real e o que se corrigiu, quanto mais rápido ficou
  (minutos antes × depois), se algum teste mudou de resultado, o que fica para a fila. Sem smoke (não muda o que
  ele vê), salvo se a cura do papel mudar a pintura — nesse caso, smoke em passos numerados.
