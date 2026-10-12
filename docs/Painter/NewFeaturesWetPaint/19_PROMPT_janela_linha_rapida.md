TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»): a ONDA DA VELOCIDADE. Tornar a linha rápida antes de
continuar a fila (ordem do dono, 11/10: «começando pelos problemas de lentidão»).

O documento-mãe é `/home/enio/Documentos/Projetos/PH2D/docs/archive/integracao-jornadas/PROMPT_LINHAS_RAPIDAS_2026-10-11.md`
(no checkout PRINCIPAL, fora da worktree, por versionar). Leia dele SÓ o §1, o §2 e o §3.Painter. Este prompt
resume-o e acrescenta o que a janela de 11/10 deixou.

## As regras (R1–R9 do documento-mãe; valem a janela inteira)

- R1 janela curta: ~150 chamadas de ferramenta ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (ficheiro novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `bash scripts/cargo-check-narrow.sh <crate>` → depois o teste.
- R4 no laço, teste por NOME: `bash scripts/cargo-test-narrow.sh <crate> <filtro>`; a suíte inteira e o
  `nextest-impacted.sh` correm 1× no fecho, pelo agente `verificador`.
- R5 nada de `until … sleep`: comando > 2 min = `run_in_background` e trabalho independente; o arnês avisa.
- R6 chamadas independentes na MESMA mensagem. R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset.
- R8 escreva menos (comentário = a lei em 1–3 linhas; resposta ao dono curta, CLAUDE.md §0.8).
- R9 nenhum teste da suíte normal passa de 10 s em `dev`. Cura, por ordem: (a) `opt-level = 2` na crate quente;
  (b) fixture menor que ainda morda (prove com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`.
  ⛔ Lei afirmada nunca vai para `#[ignore]` por ser lenta.
- Todo comando pesado por `bash scripts/ph2d-run.sh …` (o hook recusa `cargo test` fora dela; baterias num ficheiro).

## Fase 0 — onde você está

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
   → `line/PainterWatercolor`. Outra coisa: PARE e reporte.
2. `git log --oneline -3 && git status --short` → a ponta é este prompt, sobre `180c7abec`; árvore limpa.
   `git fetch -q origin`; se o `main` andou: `git rebase main` (conflito fora dos seus ficheiros: PARE e reporte).
3. `bash scripts/hw-profile.sh` · `bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10` → guarde os
   números de ANTES (vão para o handoff).
4. `cargo nextest run -p ph2d-tool-painter --status-level pass` (pela porta, em fundo) → os 15 testes mais lentos,
   com o tempo, sobre a SUA base. Guarde a lista.

## A tarefa, por ordem (a 1.ª serve às cinco linhas)

**1. `opt-level = 2` para `ph2d-wet-paint` e `ph2d-pigment`.** Elas compilam sem otimização em `dev` (o
`ph2d-tool-painter` já tem: `Cargo.toml` da raiz, `[profile.dev.package.ph2d-tool-painter]`). A/B do documento-mãe
(só essas duas otimizadas): Heavy Edge 131 → 20 s · tom ao secar 59 → 10 s · achatamento 173 → 26 s; compilar a
frio igual (1 min 16 s × 1 min 17 s). Meça pelo §2.1 do documento-mãe — um `CARGO_TARGET_DIR` PRÓPRIO na sua
scratchpad (⛔ nunca o `target/` de outra worktree: troca os `.rlib`), o0 e o2 intercalados 2×, o MÍNIMO, e o custo
de remontar depois de 1 edição nas duas. Ganhou ≥ 2× e remontar não piorou > 1 s ⇒ acrescente os dois blocos NO FIM
da lista `[profile.dev.package.*]` (só acrescentar — ADR-0107), com a tabela da medição num comentário de 3 linhas
como os vizinhos. `rm -rf` o diretório do A/B no fim.
⚠️ **Depois, corra os portões que fixam pixels ao byte**: `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`,
`wetpaint::opcoes_tests::com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`,
`wetpaint::grid_ratio_tests::smooth_edges_no_zero_e_o_desligado_de_antes_ao_byte`, `os_outros_meios_nao_mudam`,
e a suíte do `ph2d-wet-paint`. A otimização não deve mudar um bit (o Rust não funde operações de vírgula flutuante),
mas são eles que o provam. Se algum mudar: PARE e diga-o no handoff antes de regravar qualquer coisa.

**2. R9 sobre o que ficar > 10 s** (medido sobre a sua base, depois do passo 1). Os conhecidos:
- `mistura_camadas_tests::n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto` (já deu TIMEOUT 180 s no
  portão geral) · `mistura_wet_drip_tests::uma_mistura_nunca_e_mais_clara_que_as_tintas` 65 s ·
  `mistura_wet_drip_tests::sem_celulas_soltas` 40 s · os três da tabela do passo 1 (ficam 10–26 s otimizados).
  O commit `8b84cfdb8` deu-lhes uma cena por thread e um override de prazo em `.config/nextest.toml`: se a cura os
  puser abaixo de 10 s, o override sai no mesmo commit (catraca: um override sem razão medida é licença).
- **Os dois portões de LEI da fábrica criados a 11/10** (sem `#[ignore]`): `presets_de_fabrica::a_largura_e_a_prometida`
  **80–100 s** (12 presets × 8 raios de 2 a 300, 480 quadros de água; o raio 300 pinta numa tela 1000 × 1880) e
  `midias_de_fabrica::cada_preset_parece_a_sua_midia` **~60 s** (8 mídias × 2 cenas de 1 800 quadros: o preset e o
  controlo Wet Round). Ideias a medir: o controlo pintado uma vez por LARGURA e não por mídia; o raio 300 numa faixa
  da altura do traço; menos quadros onde a tinta já secou. ⛔ Nenhum vai para `#[ignore]`.
- **Toda fixture encolhida prova que ainda morde**: o agente `mutacao`, em série, restauro por cópia + `touch`, com as
  mutações já registadas no handoff `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-11_PRESETS_MIDIAS.md`
  §5.1 (M1 o raio da W2 sem a lei · M8 o grão ignorado · M5 a lei crua a vazar · M6 a tinta molhada mais escura).
  Têm de continuar VERMELHAS.

**3. O gerador da fábrica** (`presets_de_fabrica::gera_os_presets_de_fabrica`, `#[ignore]`) leva 12–15 min para os 12
presets (em série; só os 64 raios de cada um em paralelo) e corre a cada mudança de lei do motor. Meça pôr os presets
em paralelo também (o tecto da porta é 50 % da máquina). A grelha `PONTOS_DA_LEI` fica em **64** (128 errou o mesmo:
a lei ondula de raio para raio) — não a mexa sem uma medição.

**4. Os docs gigantes** com `python3 scripts/doc-split.py` (prova por sha256): `NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md`
(136 KB) e `docs/Painter/BUGS_painter.md` (94 KB), pedaços ≤ 40 KB. ⚠️ O `05_plano_da_linha_e_o_antes.md` CONTINUA a
existir como ÍNDICE (títulos + links): o prompt 17, o `18_mapa_das_midias.md`, os handoffs e o CLAUDE.md apontam para
«05 §4b». Confira os links com `grep -rn "05_plano_da_linha" docs CLAUDE.md`.

## O que NÃO é desta janela (a fila espera — registe, não toque)

O relevo que some ao trocar de pincel (o Emboss é knob do render, não guardado com a tinta) · os pincéis de óleo
longe do Rebelle (a régua não mede as estrias com luz) · a decisão do dono sobre a tinta/caneta (a mistura da obra ou
a lei no pincel) · o Staining com K–M. Estão no §8 do handoff acima.

## Fecho

- Bateria num ficheiro, pelo `verificador`: a cerca (`docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh`) · a
  suíte do `ph2d-wet-paint` · `ph2d-tool-painter --lib wetpaint` e `--test it` · a fábrica ao byte depois de
  `PH2D_REGERA_PRESETS=1 … gera_os_presets_de_fabrica -- --ignored` · os censos (`ph2d-panel-registry-init` com
  `--features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`, `ph2d-panel-painter-layers`,
  `ph2d-panel-wet-tuning`) · `ph2d-app-painter --features panel-painter-layers` · `cargo fmt --check` e clippy
  `--all-targets -D warnings` nas crates tocadas · o tecto de 700 linhas · `scripts/nextest-impacted.sh` 1×.
  Falha só sob carga: confirme sozinho com o `loadavg` ao lado (`docs/DevOps/FLAKES_DE_CARGA.md`; dois membros já
  conhecidos: `the_water_runs_at_a_usable_rate_under_a_real_frame_loop`, `a_wet_move_costs_what_the_footprint_costs…`).
- `bash scripts/agent-loop-profile.sh 10` de novo; `rm -rf target/*/incremental`; a máquina limpa
  (`pgrep -af 'ph2d|cargo|rustc'`).
- Handoff de continuação **≤ 8 KB** com as R1–R9 no topo, a tabela ANTES × DEPOIS (cada teste lento, a suíte, o
  portão geral, o perfil do agente) e o que ficou > 10 s com a razão. Entrada curta no 05 §4b (ou no pedaço novo).
  Commit por caminho (`--no-verify`, `git add` explícito para os novos). ⛔ Nunca push.
- Ao dono, curto e sem jargão: quanto mais rápido ficou (minutos antes × depois), se algum teste mudou de resultado,
  e o que fica para a fila. Esta onda não muda o que ele vê no Painter: não há smoke.
