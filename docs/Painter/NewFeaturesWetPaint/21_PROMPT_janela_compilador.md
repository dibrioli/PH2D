TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»): o `paper_rows` e o CÓDIGO MAU no perfil `ci-test`.
A 2.ª onda da velocidade (prompt 20) fechou o R9 em dez testes e descobriu que a falha do `paper_rows` não é
carga: é o compilador a gerar código errado para o bake paralelo numa combinação de perfil que o `ship.sh` usa.
Enquanto isto estiver aberto, a linha não pode ir para o `main` (o `ship.sh` reprova).

Leia SÓ o handoff `docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_VELOCIDADE_2.md`
(8 KB; as regras R1–R9 estão no topo e valem a janela inteira; o §2 é esta tarefa). Não releia o 05 nem o
histórico da linha.

## Fase 0 — onde você está

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
   → `line/PainterWatercolor`. Outra coisa: PARE e reporte.
2. `git log --oneline -3 && git status --short` → a ponta é este prompt, sobre `3dde3f541`; árvore limpa.
   `git fetch -q origin`; se o `main` andou: `git rebase main` (conflito fora dos seus ficheiros: PARE e reporte).
3. `bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10` → os números de ANTES vão para o handoff.
4. O shell é `fish`/`zsh`: listas de caminhos e laços dentro de `bash -c '…'`. `export CARGO_INCREMENTAL=0` em
   todo build `ci-test` (o perfil já o pede; o `nextest-impacted.sh` exporta-o).

## O que já está medido (não refaça)

- Reproduz em 0,03 s, 100/100:
  `cargo test -p ph2d-wet-paint --test it --profile ci-test -- --exact paper_rows::the_paper_bake_by_rows_is_the_serial_bake`.
  A rota PARALELA (`Rows::Parallel`) põe o texel errado em 7 403 de 373 464 células, em todas as 530 linhas
  (~14 por linha); nenhuma fica por escrever. A série bate com o oráculo.
- A condição é **`ph2d-wet-paint` ≥ o2 + `lto = "off"` + `overflow-checks`** (a matriz de 15 variantes está no §2).
  Com `lto = false` ou sem `overflow-checks` passa. `rayon`/`rayon-core` a o0 não mudam nada: o genérico
  `par_chunks_mut().enumerate()` é monomorfizado dentro desta crate. rustc `1.98.0 (88d9e12ae 2026-08-18)`.
- `release` e `smoke` não têm `overflow-checks`, logo o produto e o smoke do dono não caem. A exposição veio do
  `[profile.dev.package.ph2d-wet-paint] opt-level = 2` do prompt 19, que o `ci-test` (`inherits = "dev"`) herda.

## A tarefa, por ordem

**1. Reduzir até ao mínimo que ainda erra.** Num teste novo, temporário (na scratchpad ou num `tests/it/` que
depois apaga), copie `bake_paper_rows` (`crates/ph2d-wet-paint/src/paper.rs:~321`) e `walk_rows`
(`src/par.rs:~140`) e corte, um passo de cada vez, enquanto o `ci-test` ainda der diferente: o `Engine` (só um
`Vec<f32>` de `(H+2)·(W+2)`), o tile (um padrão determinístico em vez do gerador), `wrap_mask`, o `enumerate`, o
tamanho (`W = 700`, `H = 530`: experimente potências de dois e larguras < 512). Cada corte: as duas rotas e o
oráculo, contados. Guarde a menor forma que erra e a menor alteração que a faz acertar.

**2. Ler o código gerado.** Com a forma mínima, `--emit=llvm-ir` e `--emit=asm` nas duas configurações (a que
erra e a mesma com `overflow-checks = false`), só do fecho paralelo. Diga o que difere (o índice da linha? a
dobra `& 511`? a vectorização da cópia?). Procure no rust-lang/rust e no LLVM um issue com a mesma forma; se
existir, o número vai para o handoff. Se não, deixe o reprodutor mínimo pronto para o dono decidir se reporta
(⛔ nada sai desta máquina sem ordem dele).

**3. A cura no MECANISMO.** Por ordem de preferência, com a medida ao lado de cada uma:
- uma forma do laço do bake que não gere o código mau e seja a MESMA lei (prove no `ci-test`, no `dev` e no
  `release`: o portão verde nos três, e os portões ao byte da linha — `a_miniatura…`, `com_as_opcoes_desligadas…`,
  `smooth_edges_no_zero…`, `os_outros_meios_nao_mudam` — sem um bit mudado);
- um toolchain onde o código gerado é correcto (versão medida, `bash scripts/stack-audit.sh --tetos` antes; é
  decisão do dono se mudar a versão do repo — reporte, não troque);
- ⛔ `[profile.ci-test.package.ph2d-wet-paint] opt-level = 1` sozinho é silenciar o portão. Só entra se o dono o
  escolher, com o comentário a dizer que é contorno de compilador e o número do issue.

**4. A mesma forma noutros sítios.** O motor tem outros `walk_rows`/`walk_rows2`/`walk_rows_reduce`
(`grep -n "walk_rows" crates/ph2d-wet-paint/src`). Os `parallel_rows::*` passaram no `ci-test`, mas cada um testa
o SEU fecho. Para cada uso: o portão serial × paralelo existe? corre e passa no `ci-test`? Se faltar um, escreva-o.
Corra `cargo nextest run -p ph2d-wet-paint --cargo-profile ci-test` e o mesmo no `ph2d-tool-painter` (1×, pelo
`verificador`) — a linha só fica pronta para o `main` com tudo verde no `ci-test`.

R1: ~150 chamadas. Se a cura não fechar, feche no passo em que está, com o reprodutor mínimo e os números.

## O que NÃO é desta janela (registe, não toque)

O R9 que sobra (camadas 44,7 s · largura 24,7 s · mídias 14,8 s; razões no §4 do handoff) · o relevo que some ao
trocar de pincel · os pincéis de óleo longe do Rebelle · a decisão do dono sobre a tinta/caneta · o Staining com K–M.

## Fecho

- Bateria num ficheiro, pelo `verificador`, em série, `loadavg` ao lado: a cerca
  (`docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh`) · `ph2d-wet-paint` em `dev` E em `ci-test` ·
  `ph2d-tool-painter` INTEIRO (lib + `--test it`) em `dev` E em `ci-test` · os censos
  (`ph2d-panel-registry-init --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`,
  `ph2d-panel-painter-layers`, `ph2d-panel-wet-tuning`) · `ph2d-app-painter --features panel-painter-layers` · fmt e
  clippy `--all-targets -D warnings` nas crates tocadas · tecto de 700 linhas · `scripts/nextest-impacted.sh` 1×.
  Falha só sob carga: sozinha com o `loadavg` ao lado (`docs/DevOps/FLAKES_DE_CARGA.md`).
- `bash scripts/agent-loop-profile.sh 10`; `rm -rf target/*/incremental`; apague os `CARGO_TARGET_DIR` da scratchpad.
- Handoff de continuação ≤ 8 KB com as R1–R9 no topo: o reprodutor mínimo, o que o código gerado mostra, a cura
  e a prova nos três perfis, as recusas medidas. Entrada curta no `05d_estado_mistura_II_e_presets.md`. Uma lição
  durável em `project-memory/` se a houver (ex.: «um portão serial × paralelo tem de correr no perfil do ship»).
  Commit por caminho (`--no-verify`, `git add` explícito para os novos). ⛔ Nunca push.
- Ao dono, curto e sem jargão: se o problema do compilador ficou resolvido e como, se algo do que ele vê mudou
  (não deve), e se a linha já pode ir para o `main`. Sem smoke, salvo se a cura mudar a pintura — nesse caso,
  smoke em passos numerados.
