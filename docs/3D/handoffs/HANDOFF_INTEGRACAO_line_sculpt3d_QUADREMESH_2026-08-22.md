# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, o **QUAD REMESH GLOBAL** (F1–F5)

> **Para o agente INTEGRADOR.** Escrito segundo a [DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md).
> A linha **não integra e não pusha** ([CLAUDE.md §0.7](../../../CLAUDE.md)) — ela fecha, entrega isto e para.
>
> **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d`
> **Branch:** `line/sculpt3d` · **merge-base:** `ee1432203`
> **78 commits · 162 arquivos.** Tamanho: `git diff --shortstat ee1432203..line/sculpt3d`
> (~`+32.100 / −369`; a contagem de linhas inclui este próprio documento).
>
> ⚠️ **Use o TIP da branch, não um sha colado** (`git rev-parse line/sculpt3d`) — este
> documento é o último commit dela, e um sha escrito aqui dentro nomearia sempre o
> commit *anterior* a si próprio.

---

## ⚠️ §0 — LEIA ISTO PRIMEIRO

**Três coisas, e a primeira é a única que exige acção sua fora do merge.**

1. ⭐⭐⭐ **Uma colisão de ADR foi ACHADA e JÁ RESOLVIDA por esta linha** — ver §4.
   `line/3DModeling` também escrevia `0161`. ⚠️ **A colisão passaria MUDA** (dois
   ficheiros com nomes diferentes fundem limpo). Esta linha moveu-se para `0162`
   porque era **2,4× mais barata de mover**. *Não há nada a fazer; está aqui para
   você não a redescobrir e para o precedente ficar registado.*
2. ⛔ **O produto entra com um defeito ABERTO e fotografado** — os quads não são
   quadrados (§7). Ele tem gate vermelho com endereço e **não é regressão**: é o
   estado em que a feature nasce, e o Enio sabe. ⚠️ **Integrar não é aprovar.**
3. ⭐ **A feature é opt-in por um botão que não existia** — sem clicar em *Quad
   Retopology* nada no app muda. O risco de regressão para os outros módulos é
   estruturalmente baixo (§2).

---

## §1 — O que a linha entregou

A cadeia **global** de retopologia por campo cruzado (ADR-0160 → **ADR-0162**),
clean-room a partir dos papers, com o oráculo GPL **fora da árvore**. Cinco fases,
cada uma numa crate-folha nova:

| fase | crate | o quê |
|---|---|---|
| **F1** | `ph2d-remesh-iso` | remalha isotrópica — a densidade da saída deixa de depender da entrada |
| **F2** | `ph2d-crossfield` | campo cruzado MIQ (Bommes 2009) + termo de **alinhamento ao relevo** (`ALIGN_WEIGHT = 0,03`) |
| **F3** | `ph2d-trace` | traçado dos patches, com a **ponte** que abre o patch-anel e a cerca `GenusLost` |
| **F4** | `ph2d-quantize` | quantização Bi-MDF (libSatsuma/LEMON — **MIT/Boost**, permissivas) |
| **F5** | `ph2d-quadfill` | quadrangulação por patch — leque, grade, parametrização, alisamento |

Mais o backend **local** (`ph2d-quadflow`, Instant Meshes referenciado) promovido de
variável de ambiente a **escolha do painel**, e a bancada GPL-isolada em
`/home/enio/Documentos/Projetos/ph2d-quadbench` (fora do repo, ADR-0162 §Trilha B).

⚠️ **A narrativa completa — 35 secções, com cada hipótese medida e cada recusa — vive
em [`docs/3D/quad-remesh/PLAN.md`](../quad-remesh/PLAN.md).** Este documento é só o que
o merge precisa.

---

## §2 — Foundational / compartilhado tocado, e por que é ADITIVO

**Dez ficheiros fora das crates do módulo.** Nenhum remove nada; nenhum toca lista
partilhada não-namespaced.

| ficheiro | o quê | risco |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/sculpt3d.rs` | **+23 linhas**, ids do card de retopologia | ⭐ **baixo — os ids são `hash_node_id("sculpt3d.…")`**, não `NodeId(<número>)`. Não há literal numérico para colidir, e o namespace é do módulo |
| `crates/ph2d-i18n/src/sculpt3d.rs` | **+6** chaves `panel.sculpt3d.quad_*` | baixo — mesmo namespace |
| `shells/desktop/Cargo.toml` | **+6** deps de path para as crates novas | textual; região contígua no fim do bloco |
| `shells/desktop/src/sculpt3d_*.rs` | 28 ficheiros, **todos** com prefixo `sculpt3d_` | ⭐ pasta do módulo por convenção de nome |
| `shells/desktop/tests/the_sculpt_mesh_edits_are_wired.rs` | +74/−14, o seam do painel | do módulo |
| ⚠️ `scripts/nextest-impacted.sh` | **CURA DE INFRA — leia o §5** | ⭐⭐ afecta **todas** as linhas |
| `CLAUDE.md` | §5, bloco do módulo 3D | ⚠️ **o único ficheiro que a linha e o `main` tocaram os dois** — ver §3 |
| `docs/architecture/decisions/{0160,0162}*.md` + `README.md` | os dois ADRs + índice derivado | ver §4 |

⭐ **A raiz `Cargo.toml` NÃO foi tocada** — a workspace usa `members = ["crates/*"]`,
então seis crates novas custam **zero** edições centrais. É o ponto de colisão que a
auditoria de 2026-05-22 removeu, a funcionar.

⭐ **Contratos congelados (§6): INTOCADOS**, confirmado por `collision-surface.sh`
(`ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs`). ⇒ **nenhum ADR de
contrato é exigido.**

---

## §3 — Superfície de colisão (saída de `collision-surface.sh`, colada)

⚠️ **REFERÊNCIA, nunca EVIDÊNCIA** (§1.5.9): esta tabela mede a linha contra o `main`
de **2026-08-22**. **Re-rode o script em cada worktree imediatamente antes de fundir**
— a divergência entre as duas leituras é ela própria um achado.

```text
SUPERFÍCIE DE COLISÃO — line/sculpt3d contra main
  merge-base ee1432203   ·   77 commit(s)   ·   161 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                         84   (base: 84)
      └ tripla do gate               (84, 13, 14)   (base: (84, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                               57   (base: 57)
    ph2d-render (espelho)                  58   (base: 58)
    ph2d-script (espelho)                  58   (base: 58)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR
    último no disco: 0162   próximo livre: 0163
  ⚠ esta linha cria ADR: 0160 0162   — reconte contra o main do dia
▸ Cargo.lock — 6 pacote(s) '+name' novo(s), TODOS internos:
      ph2d-crossfield · ph2d-quadfill · ph2d-quadflow
      ph2d-quantize · ph2d-remesh-iso · ph2d-trace
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⭐⭐ **Nenhum número que soma entre linhas se mexeu.** Zero schemas, zero registos de
componente, zero contratos. ⇒ a classe de colisão do [CLAUDE.md §5.0](../../../CLAUDE.md)
não se aplica a esta linha, **excepto no ADR** (§4).

⭐⭐ **`Cargo.lock`: nenhuma dependência EXTERNA nova — conferido crate a crate.** Os
seis `+name` são as crates internas desta linha, e a totalidade das dependências
delas é:

```text
ph2d-crossfield  →  ph2d-mesh
ph2d-quadflow    →  ph2d-mesh
ph2d-remesh-iso  →  ph2d-mesh
ph2d-trace       →  ph2d-mesh · ph2d-crossfield · ph2d-quantize
ph2d-quadfill    →  ph2d-mesh · ph2d-quantize · ph2d-trace · ph2d-remesh-iso
⭐ ph2d-quantize →  NENHUMA
```

⚠️ **`libSatsuma` (MIT) e `LEMON` (Boost) foram REFERENCIADOS, não linkados** — o
Bi-MDF do F4 é escrito do paper, e `ph2d-quantize` não tem uma única dependência,
nem interna. ⇒ **nada novo para `cargo-machete`, `cargo-deny` ou `cargo-audit`.**

### O único ficheiro que a linha e o `main` tocaram os dois

`CLAUDE.md`. ⭐ **E as duas edições estão em secções diferentes:**

| quem | onde |
|---|---|
| `main` (`899a8c18e`) | **§1**, a linha da tabela «Fim de dia · o disco encheu» (`btrfs-health.sh`) |
| esta linha | **§5**, o bloco do módulo **3D / Sculpt** |

⇒ merge textual limpo esperado. ⚠️ **Se conflitar, resolva pelos ESTÁGIOS do índice
(`:1` base · `:2` ours · `:3` theirs) e MANTENHA AS DUAS** — são adições
independentes, e a regra da lista partilhada é *só ADICIONE*.

---

## §4 — ⚠️ A colisão de ADR, e o precedente

**Achado:** `line/sculpt3d` e `line/3DModeling` escreviam **ambas** `0161`, com
slugs diferentes:

```text
line/sculpt3d    0161-quad-remesh-pivots-to-the-global-family-...md
line/3DModeling  0161-3d-modeling-is-an-implicit-field-tree-...md
```

⛔ **Nomes de ficheiro diferentes ⇒ o git vê dois ficheiros NOVOS e funde os dois
LIMPO.** O repo ficaria com dois ADR-0161 e **nada acusaria** — é literalmente a lei
do [CLAUDE.md §5.0](../../../CLAUDE.md) («a colisão passa MUDA quando duas linhas
escrevem o MESMO literal»), com o agravante de que aqui nem o mesmo *ficheiro* é.

**Resolvido nesta linha** (commit `ca2e11b6d`), e quem se move saiu da **contagem**:

| linha | ocorrências de `ADR-0161` / slug | ficheiros |
|---|---|---|
| ⭐ `line/sculpt3d` → moveu para **0162** | **49** | 30 |
| `line/3DModeling` → fica com **0161** | 118 | 54 |

⭐ **Mover agora torna a ordem de integração irrelevante:** seja quem for a fundir
primeiro, os dois números já são distintos. `0162` foi contado contra o `main` de hoje
**e** contra as seis worktrees vivas (nenhuma outra reclama `0160` ou `0162`).

⚠️ **O índice `decisions/README.md` é DERIVADO** (`bash scripts/adr-index.sh`, com
`--check` no `ship.sh`). ⛔ **Não resolva um conflito nele à mão — regenere.**

⚠️ **Para o integrador, ao fundir 3DModeling:** re-conte. Se outra linha tiver entrado
no meio a reclamar `0161`, o mesmo trabalho repete-se — e nenhum gate o apanha.

---

## §5 — ⚠️⚠️ A cura de infra que afecta TODAS as linhas

`scripts/nextest-impacted.sh` derivava o pacote **do prefixo do caminho**
(`sed 's#^crates/\([^/]*\)/.*#\1#p'`).

⛔ **Consequência, medida em 2026-08-19: um diff inteiramente em `shells/desktop/src/`
produzia `CHANGED` vazio, o script caía no ramo «no crate changes» e rodava QUATRO
testes — saindo VERDE.** `shells/desktop` é o shell inteiro (sculpt3d, undo,
persistência, `input_dispatch`), e **todo fechamento de linha cujo diff fosse só de
shell correu com essa cobertura**. O mesmo valia para `tools/` e `tests/`.

⚠️ E o directório **não é** o nome do pacote fora de `crates/` (`shells/desktop` é o
`ph2d-host-desktop`), então a convenção antiga não podia ser estendida a mais um
prefixo: a fonte tem de ser o **manifesto**. Passou a derivar de `cargo metadata`.

⇒ **Depois de integrar esta linha, o gate de fechamento das OUTRAS linhas passa a
correr mais testes.** Isso é a cura, não uma regressão — mas **espere latentes**
([[project_integrator_ship_catches_latents_budget_iterations]]), e é bom motivo para
integrar esta linha **cedo** na jornada.

---

## §6 — O que a linha CORREU, e o que só o `ship.sh` pega

**Correu, nesta worktree, sobre o diff acumulado:**

| gate | resultado |
|---|---|
| `cargo check --workspace --all-targets` | ⭐ verde |
| `cargo fmt` nas crates tocadas | ⭐ verde |
| `cargo clippy --all-targets` (quadfill + shell) | ⭐ **0 avisos** |
| `scripts/nextest-impacted.sh --no-fail-fast` | ⭐ **10.534 / 10.534**, 1.481 skipped |
| `scripts/adr-index.sh --check` | ⭐ verde (162 ADRs) |
| prova de mutação | ⭐ 4 mutações, 4 vermelhos (ver §7) |

⚠️ **`--no-fail-fast` NÃO é opcional aqui.** A primeira corrida parou em
`ph2d-sculpt3d::measure_brush_kernel` (gate de RAZÃO, `10× a malha custou 9,34×`) e
**cancelou 1.898 testes**. Ele passa sozinho e passou na re-corrida completa — é a
flake de carga que o [CLAUDE.md §5](../../../CLAUDE.md) já legisla («nenhuma leitura
de relógio desta workstation vale nada acima de `load ~5`»). *Não suspeite do seu
merge antes de o re-correr sozinho.*

⚠️ **Segunda flake, também pré-existente:**
`ph2d-host-desktop::only_the_lower_row_breathes_and_it_moves_with_the_playhead` —
reprovou uma vez em 2.699, verde sozinha e na segunda corrida da suíte. Já nomeada no
[handoff de 16/08](HANDOFF_INTEGRACAO_line_sculpt3d_LAYER_2026-08-16.md).

**O que esta linha NÃO correu e só o `ship.sh` pega**
([[feedback_ship_parity_gaps_ci_only]]):

- `typos` sobre a árvore inteira (o diff tem **31.765 linhas** de texto denso em PT,
  com muito termo técnico — ⚠️ é o gate mais provável de acusar);
- `cargo fmt --check` sobre crates que a linha **não** tocou (fmt-skew de toolchain);
- `cargo-machete` / `cargo-deny` / `cargo-audit` — ⭐ **risco baixo por construção:
  zero dependências externas novas** (§3);
- `doc-index.sh --check` — ⚠️ **este handoff acrescenta um ficheiro a
  `docs/3D/handoffs/`, e o índice é DERIVADO.** Regenere (`bash scripts/doc-index.sh`)
  antes do ship;
- a matriz 3-OS (macOS/Windows) e o `replay-hash`.

---

## §7 — ⛔ O que fica ABERTO, e o que smokar

### ⛔ O defeito fotografado que NÃO está resolvido

**Os quads não são quadrados.** Medido contra o oráculo, com o **mesmo código** nos
dois lados (`ph2d_quadfill::quad_shape`):

| `d = 1,0` | aspecto p50 | enviesamento p50 | faces com canto pior que 60° |
|---|---|---|---|
| ⭐ oráculo, orelha | **`1,08`** | **`6°`** | **`0`** de 4.658 |
| ⛔ nós, orelha | `1,98` | `27°` | **9.159** de 78.403 |

- **Gate vermelho com endereço:** `the_quads_are_as_square_as_the_oracles`
  (`#[ignore]`, `shells/desktop/src/sculpt3d_quad_shape.rs`).
- **Causa nomeada e medida:** a 2.ª família de linhas da grade não fica ortogonal à
  1.ª — assinatura da interpolação transfinita, que casa com a **fronteira** do patch
  e enviesa no **meio**. ⇒ *o interior de um patch tem de nascer de parametrização
  alinhada ao campo*; hoje `fill_with` nem **recebe** o campo.
- ⛔ **Uma cura foi construída, medida e REJEITADA** — `SQUARE_ROUNDS = 0`, com a
  tabela em `crates/ph2d-quadfill/src/relax.rs`. **Não a ligue sem uma tabela nova.**

### Os outros vermelhos que a linha entrega, todos com endereço

| gate | o quê |
|---|---|
| `the_tracer_survives_the_aligned_field` | o toro 32×16 dá fronteira malformada com o campo alinhado. ⭐ **O produto está protegido**: a cerca `GenusLost` recusa e a porta cai para o campo só-suavidade, com o log a dizer qual correu |
| `the_ear_does_not_ship_an_edge_across_the_piece` | ⭐ **VERDE desde 22/08** (de 57 % da peça para 5,5 %) — listado aqui porque o `#[ignore]` saiu e o gate agora corre |

### ⭐ As provas de mutação desta jornada (para você não as repetir)

| mutação | matou |
|---|---|
| trocar um sinal em `nearest_square` | ⭐ `the_closed_form_finds_the_nearest_square` (oráculo por força bruta). ⛔ **`a_rhombus_becomes_a_square` SOBREVIVEU** — é tautologia: `h·iᵏ` é quadrado para qualquer `h` |
| apagar `shape: r.shape` na porta | `the_button_delivers_the_global_chain` |
| tirar «enviesamento» da linha do log | `the_report_carries_the_shape_of_every_quad` |

### O que smokar (o que NÃO foi smokado pelo Enio)

```bash
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
```

1. Abrir uma escultura → card **TOOL** → **Quad Retopology**.
2. O chip **Engine**: *Even Grid* (a cadeia global, default) e *Fast* (Instant Meshes).
3. Os sliders **Detail** e **Follow Curvature**.
4. **Ctrl+Z** devolve a malha anterior.

⚠️ **O que o Enio JÁ viu e recusou** («péssimo», 22/08): a malha sai com os quads
enviesados. ⇒ *não peça a ele um smoke desta feature esperando aprovação* — o valor
de integrar agora é **a cadeia entrar no `main`** e as outras linhas passarem a
compilar contra ela, não a feature estar pronta.

⚠️ **Cenas de smoke do módulo:** `PH2D_SCULPT3D_SMOKE=<n>` continua a valer, e a
contagem **conta-se lendo o roteador**, nunca uma nota.

---

## §8 — Ordem, dependências e a UMA LINHA do `CLAUDE.md §5`

- **Ordem interna:** os 78 commits são **lineares e dependentes** (F1 → F2 → F3 → F4
  → F5, depois as curas). ⛔ **Não cherry-pick.** `--ff-only` sobre o tip rebaseado.
- **Ordem entre linhas:** ⭐ **integre esta cedo**, pela §5 (a cura do
  `nextest-impacted.sh` aumenta a cobertura do gate das outras) e porque as seis
  crates novas são **drop-crates puros** — ninguém depende delas ainda.
- ⚠️ **`line/3DModeling` tem de ser fundida DEPOIS de reconferir o ADR** (§4).
- **`CLAUDE.md §5`:** o bloco do módulo **3D / Sculpt** já traz a linha de estado e a
  de **Aberto** actualizadas neste diff. ⛔ **Não acrescente parágrafo de jornada** —
  a narrativa é este ficheiro e o `PLAN.md`.

---

## §9 — Higiene: o `incremental/` foi reclamado

Feito depois do gate batched, antes de parar (§1.5.9 item 7):

```bash
rm -rf /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d/target/*/incremental
```

---

## §10 — Resumo colável para o Enio

> Linha `sculpt3d` pronta (tip de `line/sculpt3d`, 78 commits, 162 ficheiros).
> Foundational tocado: `ids/chrome/sculpt3d.rs` e `i18n/sculpt3d.rs` (aditivos,
> namespaced), `shells/desktop/Cargo.toml` (+6 deps de path) e
> `scripts/nextest-impacted.sh` (**cura de infra que afecta todas as linhas**).
> **Zero** schemas, **zero** registos de componente, **zero** contratos congelados,
> **zero** dependências externas novas. Uma colisão de ADR com `line/3DModeling` foi
> achada e **já resolvida** (movi-me para `0162`). Gate: **10.534/10.534**.
> ⛔ O defeito que o Enio fotografou **continua aberto**, com gate vermelho e causa
> medida. Aguardo ordem de integração.
