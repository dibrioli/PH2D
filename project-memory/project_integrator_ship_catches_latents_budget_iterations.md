---
name: project-integrator-ship-catches-latents-budget-iterations
description: Modo L — o gate per-linha e o foundational-integrate NÃO rodam fmt/clippy-all-targets/machete/deny; o integrador orça 2-4 iterações de ship.sh pra drenar latentes das linhas
metadata: 
  node_type: memory
  type: project
  originSessionId: ac6fba2f-c694-4c47-a142-9e06671dae88
---

**Contexto:** integração de 5 linhas (MotionNodes/Painter/anim/Vector/audio),
2026-07-11, todas fechadas "verde" pelo próprio gate batched. O integrador
ainda drenou **~8 falhas latentes** antes do ship 100% verde.

**Confirmado de novo (6 linhas + FLIP, 2026-07-12): 4 iterações de ship**, e o
padrão se repetiu inteiro — typos (a linha FLIP declarou "risco baixo, o ship
confirma" e NÃO rodou o gate: 3 typos reais + 6 palavras pt-BR) · LOC/fn de
painel (`paint_brush_body` 222 > 200, estourando até uma **dispensa de 215** que
já existia em `FN_OVERAGE_OK`) · e os 2 gates colaterais que o próprio split
disparou (LOC de arquivo + HR-12 a11y do arquivo novo). **O split arrasta gates:
orce isso.** Um latente que o `foundational-integrate` PEGOU: `assert_eq!(reg.len())`
do `ComponentRegistry` em **3 sites** (ph2d-ecs / ph2d-render / ph2d-script) —
duas linhas registraram um componente cada e uma delas só bumpou o primeiro.

## Por que verde-de-linha ≠ verde-de-ship

Duas camadas de gate rodam ANTES do ship, e nenhuma cobre tudo:

1. **Gate batched da linha** (`nextest-impacted` + clippy `-p` + fmt manual): a
   linha escolhe o que roda. Erra por omissão — clippy `-p <crate>` não pega um
   lint num crate-irmão; fmt "à mão" não roda o pin; a suíte completa do shell
   (`file_loc_caps`, `no_tofu_glyphs`) não roda se a linha não a invocou.
2. **`foundational-integrate.sh`** (rebase → sync → `cargo check --workspace` →
   `nextest-impacted`): pega LOC/tofu/count-mirrors **se** o gate vive num teste
   dentro do impacted-set (pegou o HR-18 do Painter, o LOC do vec-edit, os 2
   count-mirrors do Vector). Mas **NÃO roda fmt, clippy `--all-targets`, machete,
   deny, typos** — esses só existem no `ship.sh`.

⇒ Tudo que é **fmt-skew** (a linha não rodou `cargo fmt` no pin) e **lint de
clippy** (a linha não rodou `--all-targets` no crate certo) **atravessa as duas
camadas** e só vermelha no ship final. Nesta rodada: watercolor fmt-skew (4
arquivos), `HashMap` banido (`clippy.toml` ADR-0022) em ph2d-audio-encode,
`then_some` (unnecessary_lazy_evaluations) em audio_overlay.

## A pegadinha nova: o fmt do integrador quebra o LOC cap

`watercolor_render.rs` estava a **699** (não-canônico — a linha Painter não rodou
fmt, handoff §5). O `cargo fmt --all` do integrador canonicalizou e **re-expandiu
pra 701** (multi-arg fold), estourando o cap de 700 → nextest do ship vermelho.
É o [[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]] no contexto de
integração: **rode fmt ANTES de medir LOC, e depois de fmt re-cheque o LOC gate.**
Fix = split (mover `watercolor_render_active` pro sibling), nunca allowlist.

## How to apply (integrador)

1. **Orce 2-4 iterações de `ship.sh`.** Cada uma descasca a próxima camada
   (fmt → LOC-pós-fmt → clippy-disallowed-type → clippy-lazy-eval). Não é
   retrabalho; é o desenho (as linhas delegam isso, §5 dos handoffs).
2. **Atalho pra clippy:** quando o ship vermelha em 1 lint, rode
   `rustup run <pin> cargo clippy --workspace --all-targets --features
   ph2d-spike/bevy_ecs -- -D warnings` DIRETO — pega TODOS os lints de uma vez,
   em vez de um-por-ship (cada ship é ~10min).
3. **Todo fix de latente é commit no main** (a linha já integrou) com msg
   `fix(ship): …` nomeando qual gate pegou.
4. Ver [[feedback_ship_parity_gaps_ci_only]] (ship↔CI), [[feedback_ci_direct_lint_gates_and_fmt_skew]],
   [[project_integration_prefork_lines_ship_drift]], [[feedback_ship_prep_no_fail_fast]].

## Os dois critérios que o ship de 2026-07-31 acrescentou (jornada de 7 linhas)

### Advisory: TENTE o upgrade antes de escrever o `ignore`

Dois RUSTSEC chegaram juntos, e a resposta certa foi **oposta** para cada um:

- **wasmtime (RUSTSEC-2026-0222, publicado NAQUELE dia) — CONSERTADO.** ⚠️ A
  armadilha: `tests/spike` pinava `"44"` e **a série 44 inteira está fora de toda
  faixa corrigida**, então `cargo update -p wasmtime` "atualizava" (44.0.1→44.0.3)
  **sem alcançar conserto nenhum** — parecia que não havia fix. O fix era o bump
  de major (`"47"` → 47.0.3). *Leia as faixas do advisory antes de concluir que
  não há saída.*
- **tract (RUSTSEC-2026-0217) — ignorado, com a justificativa MEDIDA.** O upgrade
  foi TENTADO: a faixa começa em 0.21.16 e bumpar o `deep_filter` vendorizado
  falha a resolução da workspace (`failed to select a version for half`). Isso deu
  **motivo medido** ao pin que o ADR-0123 até então só AFIRMAVA.

⇒ Um `ignore` escrito sem tentar o upgrade é um palpite; escrito depois, é um
fato com mecanismo. E os dois configs são **paralelos e independentes**:
`deny.toml [advisories].ignore` **e** `.cargo/audit.toml` — mexer num só deixa o
outro ✗.

### Typos: a pergunta é *"allowlistar isto pode esconder um typo REAL?"*

33 ocorrências / 16 palavras acumuladas por 7 linhas (o gate só roda no ship).
A tentação é allowlistar tudo; o critério que funciona é por-palavra:

- **Pode esconder → REESCREVA o texto.** `grep -rin` virou `grep -r -i -n`,
  porque `rin` colide com **`ring`** — e este repo diz "ring" o tempo todo
  (CheckpointRing, return ring, os rings do nodegraph). Allowlistar teria cegado
  o gate para eles.
- **Não pode → entra no config.** Prosa pt-BR correta não se reescreve para
  agradar um dicionário de inglês; isso é o gate mandando no texto.

⚠️ **`flase` NÃO era typo — era o DADO DO TESTE** (`["", "flase", "no", "2",
"OFF"]`, provando que uma escape mal digitada cai no default). "Corrigi-lo"
deletaria o caso que o gate existe para cobrir. *Leia o sítio antes de consertar
a grafia.*

⚠️ **`extend-ignore-identifiers-re` casa o identificador INTEIRO.** `fing` e
`pont` vivem dentro de `k_fing`/`pont_marcado`, então `^fing$` **não pega** —
vão para `[default.extend-words]`, que é o token que o typos de fato compara.
(`inh` e `flase` são identificadores inteiros e funcionam na lista âncorada.)

### E um `✗` de nextest que era FLAKE, não regressão

`ph2d-timeline::no_alloc_bridge` falhou **uma vez** e passou 2/2 na workspace
inteira depois, além de 4/4 isolado no mesmo perfil `ci-test`. Mecanismo: ele
mede uma propriedade **local** (`apply_from_doc` não aloca) com
`dhat::HeapStats`, um contador **global do processo** — qualquer inicialização
preguiçosa que caia na janela medida o vermelha
([[feedback_zero_alloc_gate_capacity_not_global_counter]]). *Antes de caçar a
regressão, meça se REPRODUZ.*

## Infra: `/dev/shm/ph2d-target` some entre sessões

O `target/` do primário é symlink pro tmpfs (`workstation`). O
`/dev/shm/ph2d-target` **evapora no reboot** (**3ª vez em 2026-07-12**) → `cargo`
do primário falha com "failed to create directory target" / "Not a directory".
As integrações não pegam isso (cada worktree tem `target/` real próprio); só o
`ship.sh` (roda no primário) trava. Fix: `mkdir -p /dev/shm/ph2d-target`.

**Cuidado — o modo como isso FALHA engana:** o `ship.sh` marca `✗ clippy` e
`✗ nextest` como se fossem falhas de CÓDIGO, mas os dois **nem chegaram a
rodar** (abortam ao criar o `target/`). Se você "corrigir" código com base
nesse ✗, está caçando fantasma. Cheque a mensagem real no log antes.
Ver [[feedback_pipe_masks_script_exit_code]].

## A terceira espécie de `✗`: o gate de RAZÃO que o RUNNER mata de fome (2026-08-08, 6 linhas)

Um `✗` de nextest tem três causas, não duas — e a terceira parece as outras duas:

1. **regressão** (o código quebrou),
2. **flake de azar** (não reproduz),
3. **fome de núcleo**: o gate mede uma RAZÃO entre um caminho serial e um
   paralelo, e o nextest roda ~32 processos de teste ao mesmo tempo. O `rayon`
   reporta os 32 threads e **não tem um núcleo livre**. Medido:
   `serial 15,243 ms · paralelo 15,441 · ganho 0.99x`.

**O par de medições que separa (3) de (1):** *reprova sob a suíte cheia* **E**
*passa isolado*. Sem as duas metades você está a silenciar regressões.

**A cura é `threads-required = 'num-cpus'`** em `.config/nextest.toml` (o mesmo
mecanismo de override que o repo já usa para serializar o ISPC). ⚠️ Ela **não
afrouxa barra nenhuma** — a asserção fica onde estava, feita com os núcleos de
que ela precisa para ser verdadeira. Quatro gates entraram nesta rodada
(`measure_normals_parallel_speedup`, `the_cost_of_depth_is_linear_not_explosive`,
`the_cost_of_sampling_a_path_is_flat_in_its_anchors`,
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`).

⚠️ **A mensagem do próprio gate não pode nomear a causa**, e isso engana: ela
oferece *"PAR_MIN alto demais? limiar invertido?"* — hipóteses sobre o código —
porque o gate **não enxerga o runner que o está a matar de fome**. Ler a
mensagem e ir consertar `PAR_MIN` é caçar fantasma.

⚠️ **Enumere com `--no-fail-fast`** ([[feedback_ship_prep_no_fail_fast]]): o
fail-fast revela UMA por corrida, então descobri-las pelo ship custa **uma
corrida de ship por gate** — e cada corrida conserta o anterior e revela o
seguinte, o que se lê como progresso e é uma fila.

## E o `cargo fmt` do integrador pode carimbar um comentário no módulo ERRADO

O rustfmt **reordena `mod` alfabeticamente** (`reorder_modules`), e um comentário
de fim-de-linha fica preso à **POSIÇÃO**, não ao item. Dois casos na mesma rodada:
*"o que um Down SIGNIFICA com várias figuras"* foi parar no `taper_tail_tests` (a
ponta de um traço à mão livre) quando o dono é o `stroke_router`; e a *ponte de
TOKENS* ficou no `sculpt3d_panel_bridge`, deixando `tokens_bridge` nu. **Depois de
`cargo fmt --all`, leia o diff dos `mod`** — um comentário que descreve o módulo
errado é pior que comentário nenhum, e o fmt não o inventa: ele o herda de um
merge e o carimba. O dono verdadeiro costuma estar escrito no `//!` do próprio
módulo.
