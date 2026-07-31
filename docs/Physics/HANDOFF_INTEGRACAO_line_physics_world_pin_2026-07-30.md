# HANDOFF DE INTEGRAÇÃO — `line/physics`, jornada da POLIA + o PINO DE MUNDO

> **Para o agente INTEGRADOR.** A linha está FECHADA e todos os smokes foram
> aprovados pelo Enio. Este documento é a única coisa que você precisa ler; o
> detalhe por-wave vive no tracker ([`HANDOFF_line_physics.md`](HANDOFF_line_physics.md)),
> o mapa em [`00_plano_waves.md`](00_plano_waves.md), e os desenhos em
> [`03_plano_polia.md`](03_plano_polia.md) e [`02_plano_joints_ui_authoring.md`](02_plano_joints_ui_authoring.md) §9.
>
> ⚠️ **SUPERSEDE [`HANDOFF_INTEGRACAO_line_physics_pulley_2026-07-30.md`](HANDOFF_INTEGRACAO_line_physics_pulley_2026-07-30.md)**,
> escrito hoje mais cedo. Ele está **DUAS vezes desatualizado**: a linha reabriu
> depois dele (a wave do pino de mundo) **e** o `main` andou 186 commits desde
> que ele afirmou *"o `main` NÃO andou"*. Não integre por aquele documento.

## §0 — GO / NO-GO

**GO, aguardando a sua ordem.** O Enio aprovou os smokes `=61`, `=63`, `=64` e
`=65`.

- Branch: **`line/physics`**, worktree em `Worktrees/line-physics`.
- Tip: **`aa1f14d9e`**.
- Base (fork): **`7ec917506`**.
- **70 commits**, **229 arquivos**, **+39.232 / −2.359**.

⚠️ **O `main` ANDOU: 186 commits** (`git rev-list --count $(git merge-base main HEAD)..main`).
**Não é `--ff-only`.** Mas o dano é pequeno e está MEDIDO — leia a §3 antes de
qualquer outra coisa.

## §1 — O que a linha entrega

### (A) A POLIA — plano 03 inteiro, W0..W6 + a WESTON

O 7º tipo de joint, e o primeiro cujo comprimento é um **orçamento**, não uma
distância.

| wave | o quê | cena |
|---|---|---|
| W0 | as quatro correções da foto do smoke (criação pelo canvas nascia na ORIGEM · o anel de comprimento perguntava `length.is_some()` · o readout `0 / 0 N` permanente · a row Ratio morta) | `=58` |
| W1 | ⚠️ **o `ratio` SAIU por ser física errada** (numa corda única a tensão é uniforme ⇒ vantagem 1). No lugar: **uma roldana é uma ENTIDADE com RAIO**, rota de N nós tangenciando a SUPERFÍCIE, arco no comprimento e não no Jacobiano, lado por ponto fixo, `ω = s/r` | `=58` |
| W2 | **o MOTOR** (roldana dirigida = guincho) e a **RUPTURA** (UM limiar; o que difere é o EIXO de cada roldana) | `=59` · `=60` |
| W3 | **a TALHA** — a roldana montada num corpo que se move; a vantagem volta **sem um número** (2 kg equilibram com 1,00 kg, medido) | `=61` |
| W4 | **o TAMBOR DIFERENCIAL** — um eixo é UM nó, logo um tambor é UMA roldana com DOIS raios; vantagem `r_entra/r_sai` | `=62` |
| W5 | **a COMPOSIÇÃO** — tambor e cadernal na MESMA corda: as vantagens MULTIPLICAM (1 kg segura 16 kg sem ninguém digitar um "16") | `=63` |
| W6 | **as ALÇAS** — re-colocar o eixo de uma roldana MONTADA (gesto morto e silencioso) e o **segundo diâmetro** agarrável | `=63` |
| — | o **PISO** · a **rota que não resolve PARA de segurar** · o **§10** · o **ÍMÃ** do eixo montado · a **cadernal dirigida** · o **conta-gotas de corda** | `=61` · `=63` |
| **W-Weston** | **a talha DIFERENCIAL** — o MESMO eixo atravessado DUAS vezes, cadernal abraçada entre os contatos: peso `R/(R−r)`, vantagem `2R/(R−r)` | `=64` |

⚠️ **A Weston achou DOIS bugs PRÉ-EXISTENTES**, e os dois entram corrigidos: o
teto do guincho era **cego ao peso da corda** (taxa içada caía a 38% no peso 8;
latente desde o W4) e **um rewind replayava SEM AS CORDAS** (o
`rebuild_from_rest` trocava o mundo sem reinstalar a tabela; calado porque
`target == 0` replaya zero passos).

### (B) W-JointWorld — o PINO DE MUNDO

Um joint cujo lado B é um **PONTO DO CENÁRIO**, não um corpo. Remove o **corpo
estático inventado** que hoje é obrigatório para pendurar qualquer coisa.

- **Marcador `JointWorldAnchor`** (presença = o booleano) ⇒ **zero bump de
  schema**. ⚠️ **Não é `body_b == 0`**: aquele estado já significa *meio-autorado*.
- **O ponto é o `Transform` da própria entidade-joint** — que já é a âncora
  autorada, já tem o dot arrastável e já viaja no save.
- **`JointRef.entities.1` e `JointView.body_b` viraram `Option<Entity>`** —
  esquecer o caso do mundo fica **impossível por TIPO**.
- §12 diz **World** (não "(missing)"), **não oferece** o conta-gotas do lado B, e
  o par **`Anchor B: [Object | World]`** é quem torna o pino um pino.
- O gesto de canvas vale nas **DUAS direções** (corpo→vazio e vazio→corpo), e o
  **dot âmbar move a âncora** (o corpo vai junto).

Cena **`=65`**: dois pêndulos idênticos; só o da esquerda precisa de um objeto
inventado. Medido: **0,8383 m de percurso nos DOIS**, quatro decimais.

## §2 — Números que se CONTAM (confira, não copie)

| pin | fork | **main HOJE** | linha | o que fazer |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | 37 | **38** | 45 | ⚠️ **CONTE: 46** — ver §3.1 |
| ADR | — | **0148** é o último | 0145 | ⚠️ **RENUMERAR para 0149** — ver §3.2 |
| registro `ph2d-physics-ecs` | 21 | **21** (intocado) | **24** | fica 24 |
| ids de gizmo | ≤968 | **≤968** (intocado) | **969, 970, 971** | ficam; próximo livre **972** |
| `physics_ecs_c9` | — | — | **`7cb7728d…`, 96 corpos** | rodado, debug ≡ release |
| cenas de smoke | ≤53 | **≤53** (intocado) | **54..65** | ficam |

⚠️ **O c9 é o único destes que eu RODEI.** Os outros saem de `grep` na fonte —
que é o certo, e é a lição que esta linha pagou: *número citado de memória é
número velho*.

## §3 — O QUE VAI CONFLITAR (medido, não previsto)

Sondado com `git merge-tree --write-tree main HEAD`, que não toca a árvore.
**Exatamente DOIS arquivos conflitam**, e é o mesmo assunto nos dois:

```
CONFLICT (content): shells/desktop/src/project.rs
CONFLICT (content): shells/desktop/src/project_tests.rs
```

**Todo o resto funde sozinho** — incluindo `render_loop/mod.rs`, `main.rs`,
`input_dispatch.rs`, `node_id_collisions.rs`, `Cargo.lock` e a `MEMORY.md`.

### §3.1 — `PROJECT_SCHEMA`: CONTE, não escolha

O fork estava em **37**. A `line/Vector` bumpou para **38** no `main`; esta linha
bumpou **oito vezes** (37 → 45). ⚠️ **O valor certo não está em nenhum dos dois
lados do conflito:**

```
38 (main hoje) + 8 (bumps desta linha) = 46
```

Reescreva os parágrafos `v3X`/`v4X` do `project.rs` **na ordem em que os bumps de
fato empilharam** (os desta linha vêm DEPOIS do 38 da Vector), e atualize a
tripla-pin do `project_tests.rs`. ⚠️ **Esta é a QUARTA vez que estas duas linhas
disputam este número** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]);
o antídoto é uma linha de shell, não uma lembrança:

```
grep -n "PROJECT_SCHEMA: u32" shells/desktop/src/project.rs
```

### §3.2 — ADR-0145: RENUMERAR o desta linha

⚠️ **Colisão que o git NÃO reporta**, porque os nomes de arquivo diferem e cada
lado só acrescenta o seu:

| lado | arquivo |
|---|---|
| **main** | `0145-wet-paint-solver-row-parallel-passes-rayon-exception.md` (+ 0146, 0147, **0148**) |
| **linha** | `0145-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md` |

**Renumere o DESTA LINHA para `0149`** (o próximo livre depois do 0148 do main) —
quem chegou ao `main` primeiro fica com o número, a regra que o repo já aplicou
três vezes. Renomeie o arquivo e conserte as referências, que são **7 ocorrências
em 5 arquivos** (medido, não estimado):

```
grep -rln "0145-physics-ik\|ADR-0145" docs/ crates/ shells/
#   docs/architecture/decisions/0145-physics-ik-…md   (o próprio, + o título dentro)
#   docs/Physics/00_plano_waves.md                    (2×)
#   docs/Physics/02_plano_joints_ui_authoring.md
#   docs/Physics/03_plano_ik.md
#   docs/Physics/04_plano_fk_e_modos_de_joint.md
#   docs/Physics/HANDOFF_line_physics.md              (2×)
```

⚠️ **Nenhuma delas está em código** — é tudo doc-link. E o
`HANDOFF_INTEGRACAO_line_physics_pulley_2026-07-30.md` também as tem, mas ele
está **superseded** (topo deste documento): não gaste tempo com ele.

O gate `architecture_adr_numbers_are_unique` (sem allowlist) é quem pega se
esquecer.

### §3.3 — O que NÃO conflita, e por quê

⚠️ **O `main` não tocou física nenhuma** (`git diff --name-only <fork>..main --
crates/ph2d-physics crates/ph2d-physics-ecs docs/Physics` sai **vazio**), nem
`gizmo/hit.rs`, nem `physics_smoke.rs`, nem `00_plano_waves.md`. Não há risco
semântico escondido atrás de um merge textual limpo — a sobreposição real entre
as duas linhas é **só o schema e o número de ADR**.

## §4 — O gate de fechamento (rodado 1× sobre o diff acumulado)

Na worktree, ANTES do rebase:

- `cargo test`: **1362** (shell bins) · **284** (`ph2d-physics-ecs`) · **247**
  (`ph2d-physics`) · **983** (`ph2d-panel-inspector` + `ph2d-editor-core`).
- `cargo clippy --all-targets` nas crates tocadas + shell: **0 warnings**.
- `cargo fmt --all`: limpo.
- **`architecture_workspace_file_loc_cap`** (crates, 700) **e**
  `shells/desktop/tests/file_loc_caps.rs` (shell, 600): verdes. ⚠️ **Os DOIS** —
  o segundo não roda num `cargo test -p` filtrado e já ficou vermelho-latente
  nesta linha três vezes.
- `node_id_collisions` · `no_tofu_glyphs` · `arch_safe_clamp_only` ·
  `handle_scenes_start_paused` · `architecture_panel_wiring_parity`: verdes.
- `physics_ecs_c9`: **`7cb7728d…`, 96 corpos**, idêntico em debug e release.

⚠️ **Rode tudo de novo DEPOIS do rebase.** O conflito do schema é textual, mas a
tripla-pin do `project_tests.rs` é uma asserção — ela falha se você contar
errado, e é bom que falhe.

⚠️ **O que o `ship.sh` acrescenta e eu não rodei:** `machete`, `deny`, `audit`,
`typos` e a matriz de 3 OSes.

## §5 — Contabilidade de contratos

- **Nenhum contrato congelado tocado** (CLAUDE.md §6) — conferido por grep.
- **Nenhuma crate nova.** Uma dep nova, já no workspace com o MESMO pin:
  `libm = "=0.2.16"` em `ph2d-physics-ecs`.
- **Um ADR novo** (o da IK), a renumerar — §3.2.
- **Um componente novo** que move schema: nenhum. Os três do registro
  (`PulleyWheel`, `WestonAxle`, `JointWorldAnchor`) cunham blob-key própria.

## §6 — Aberto, nomeado, NÃO bloqueante

- **`axle_pair` recusa três ou mais contatos num eixo** — dois diferenciais em
  série é topologia própria.
- **O eixo composto da Weston é cenário na v1** (montá-lo num corpo que se move
  quer o Jacobiano do 2º contato no ledger).
- **`radius_out` e `WestonAxle` são duas formas de dizer "eixo composto"** e um
  dia querem ser um enum.
- **Um pino de mundo e um pino entre dois corpos leem IGUAL na tela** — a
  geometria está certa e o overlay o desenha de graça; falta o glifo dizer que
  aquela ponta é o cenário. Decisão de desenho.
- **Não há alça para *onde no corpo* o pino de mundo prende** — o dot move a
  ÂNCORA; o `local_a` é semeado na criação.
- **O readout `0 N` de uma corda degenerada** não diz por quê (quer i18n).
- **Os sete itens do horizonte** (plano 02 §8) seguem sem dono; o item 3 dele
  (*Pin-to-world autorável*) é justamente o que esta jornada fechou.

## §7 — Os smokes, para você repetir se quiser

Todos `--release`, e todos **aprovados pelo Enio**:

```
env PH2D_PHYSICS_SMOKE=<61|63|64|65> cargo run -p ph2d-host-desktop --release
```

`=61` a talha · `=63` a composição + as alças · `=64` a Weston · `=65` o pino de
mundo. As cenas `54..60` e `62` rodaram e tiveram seus defeitos fechados, mas
**não têm nota de aprovação final** — a jornada seguiu direto para a wave
seguinte. Elas não bloqueiam a integração; o que as cobre são os gates.
