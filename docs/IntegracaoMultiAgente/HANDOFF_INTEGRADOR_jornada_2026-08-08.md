# HANDOFF DO INTEGRADOR — jornada de 2026-08-08 (4 de 6 integradas)

> Para o **próximo agente integrador**. Você assume no meio de uma jornada Modo L
> que o Enio abriu com seis linhas e a ordem *"vá por partes para não encher o
> contexto"*. Quatro já estão no `main`; **duas faltam**, e depois delas vem o
> gate completo + a §5 do CLAUDE.md.
>
> ⚠️ **Integração e ship só por ordem EXPLÍCITA do Enio** (CLAUDE.md §0.7). A ordem
> desta jornada JÁ FOI DADA e cobre as seis linhas listadas abaixo — ela **não**
> se estende a nenhuma linha fora desta lista, e o **ship/push** ao fim da última
> integração (DIRETRIZ §1.5.4) precisa de confirmação própria.

---

## §1 — Onde a jornada está

`main` = **`9723ba9bb`** · `PROJECT_SCHEMA` = **69** · próximo scrollbar livre = **842**
· próximo ADR livre = **0157**.

| # | Linha | Estado | Números que ela contou |
|---|---|---|---|
| 1 | `line/Vector` | ✅ integrada | schema **62** · scrollbar **839** (`AUTHORED`) · `ph2d-ecs` 52→**54** + os 2 espelhos · +1 painel |
| 2 | `line/sculpt3d` | ✅ integrada | schema **63** · scrollbar **840** (`SCULPT3D`) · **ADR-0156** · +1 painel |
| 3 | `line/physics` | ✅ integrada | schema **68** (v64..v68) · `ph2d-physics-ecs` **29** · `c9` re-hasheado |
| 4 | `line/motion-value` | ✅ integrada | schema **69** · scrollbar **841** (`MOTION_PARAMS`) |
| 5 | **`line/Painter`** | ⬜ **PRÓXIMA** | sem schema · **ADR-0156 → 0157** (renumerar) |
| 6 | **`line/runtime`** | ⬜ falta | **zero números** |

Depois das duas: **gate COMPLETO da árvore combinada** e **CLAUDE.md §5**.

Os handoffs das linhas (a munição) são os que o Enio listou:

- `docs/HANDOFF_INTEGRACAO_line_Painter_MESTRE_2026-08-08.md`
- `Worktrees/line-runtime/docs/Runtime/HANDOFF_INTEGRACAO_line_runtime_R0_2026-08-08.md`

---

## §2 — A ordem foi MEDIDA e as duas que restam são as de menor sobreposição

A ordem saiu da sobreposição par-a-par de arquivos tocados (o §1 da skill: *a ordem
se mede, não se escolhe*): as linhas que mexem em **números que somam** foram
primeiro, para que cada uma contasse contra um `main` já estável; `line/Painter` e
`line/runtime` ficaram por último porque **nenhuma das duas bumpa schema**.

**Não reordene as duas restantes sem re-medir** — mas se você as integrar em
qualquer ordem o resultado é o mesmo: elas não disputam nenhum contador.

---

## §3 — 5/6 `line/Painter` (68 commits) — o que já está medido

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
```

**⚠️ O único número em disputa é o ADR.** A linha criou
`0156-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md`
e o `main` **já tem** um 0156 (`0156-sculpt3d-ao-trace-…`, da `line/sculpt3d`, que
chegou primeiro). Como os NOMES de arquivo diferem, **o git funde os dois limpos** e
quem pega é o gate `architecture_adr_numbers_are_unique` — é a **8ª vez** que isto
acontece no repo.

⇒ **Renumere para `ADR-0157`.**

**O rewrite do token é ESCOPADO** ([[feedback_a_token_rewrite_scopes_to_the_changed_files_not_the_whole_tree]]):

- Reescreva o **token `ADR-0156`** e o **stem do arquivo**, nunca o número nu.
  O `Cargo.lock` contém `0156` **dentro de checksums**.
- Escopo = **os arquivos que a LINHA mudou**. Medido: **18 arquivos** citam
  `ADR-0156`/`0156-liquify` na linha (16 de código/doc + o ADR + o handoff dela).
  Lista exata:

```
git diff --name-only main...HEAD | tr '\n' '\0' \
  | xargs -0 -I{} sh -c 'grep -l "ADR-0156\|0156-liquify" "{}" 2>/dev/null'
```

- ⚠️ **Confira se algum arquivo ALHEIO cita `ADR-0156`** (o do sculpt3d) antes de
  qualquer `sed` — se citar, o escopo é a **interseção** *linha ∩ citações*, e um
  arquivo MISTO (como o `CLAUDE.md` já foi em 31/07) exige edição cirúrgica.

**Schema:** o `project.rs` da worktree ainda lê `55` porque a linha **não foi
rebaseada** (o `foundational-integrate.sh` faz o rebase). Confirme com

```
git diff --name-only main...HEAD -- shells/desktop/src/project.rs
```

Se vier vazio, ela não toca o schema e não há número a contar. **Se vier
preenchido, PARE e conte** — o literal pode ser o mesmo dos dois lados e o git
funde sem conflito, exatamente como a `line/FLIP` quase passou muda em 01/08.

---

## §4 — 6/6 `line/runtime` (5 commits) — zero números, um hazard nomeado

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-runtime
```

Medido: **não toca `project.rs`**, **nenhum ADR novo**, dois `Cargo.toml`
(`ph2d-runtime` + shell).

⚠️ **O hazard que o handoff dela nomeia** é o par de laços de toast em
`render_loop/mod.rs`: a resolução é **MANTER o `publish`**. Leia o §
correspondente do `HANDOFF_INTEGRACAO_line_runtime_R0_2026-08-08.md` antes de
resolver o conflito — pelos **ESTÁGIOS do índice** (`:1` base · `:2` ours ·
`:3` theirs), nunca pelos marcadores.

---

## §5 — O que esta jornada JÁ ensinou (não redescubra)

### 5.1 — A família do ENDEREÇO: três arch-gates corretos sobre um produto que se mudou

O `src/project.rs` do shell bateu **623 > 600 LOC** na árvore combinada — **cinco
linhas apendaram degraus de schema na mesma janela e nenhuma cruzava sozinha**.
Cortado por responsabilidade: a metade da ESCRITA saiu para o irmão
**`project_save.rs`** (o `project_load.rs` já existia desde 05/08).

Isso quebrou **três** gates que estavam **certos sobre o produto** e erravam o
**endereço**:

| Gate | Onde | Cura |
|---|---|---|
| `the_module_that_holds_the_channels_is_unconditional` | `tests/a_baked_object_outlives_the_3d_module.rs` | varre a **FAMÍLIA** `src/project*.rs` pelo helper novo `sculpt_source::project_family_fn` |
| `the_save_prefers_the_live_scene_and_falls_back_to_the_bytes` | `tests/the_sculpt_document_is_wired.rs` | idem |
| `the_save_writes_the_live_tape` | `src/project_tape_tests.rs` | vive em `src/`, **não alcança** o helper de `tests/` ⇒ `concat!(include_str!("project.rs"), include_str!("project_save.rs"))` |

**A lei:** *afirme a PROPRIEDADE, nunca o ENDEREÇO*; e toda varredura de família
leva **controle positivo** — o `panic!` final do `project_family_fn` existe porque
um `unwrap_or_default()` deixaria toda asserção passar **por vácuo**.

⚠️ **Por que apareceram um de cada vez:** o `nextest` do gate **CANCELA na primeira
falha**. Orce isso — cada rodada do `foundational-integrate.sh` revela **um**
vermelho da mesma família.

### 5.2 — Vermelho LATENTE não é colisão de integração

`an_undeclared_row_is_untouched_by_the_display_boundary` afirmava `row.max ==
12_000`; um commit **três dias mais novo da mesma linha** baixou o teto de arrasto
do `rate` para **1_200** de propósito e não reconferiu o gate.

*Quem move o número que tornava uma afirmação verdadeira TEM de reconferi-la*
(CLAUDE.md §0). Antes de culpar o merge: **`git log` no arquivo e `git blame` na
linha** — se os dois commits são da linha, o vermelho é dela.

### 5.3 — `cargo check` NÃO compila alvos de teste

Uma iteração inteira foi perdida com `cargo check -p` verde sobre um `tests/`
quebrado. Use **`--all-targets`**.

### 5.4 — Uma fixture de uma linha pode não conhecer um campo de outra

`project_tape_tests.rs` (physics) construía `ProjectFile` sem o campo `settings`
(motion-value) ⇒ `E0063` **só na árvore combinada**. Nenhuma das duas compilava
quebrada sozinha.

### 5.5 — Higiene de git desta jornada

- `git add -- <paths>` · `git commit --no-verify -F <arquivo> -- <paths>`.
  **NUNCA** `-A` / `-a` / `git add .` / `git stash`.
  ⚠️ Registro honesto: **um `git add -A -- shells/desktop/src` escapou** numa
  resolução de conflito desta jornada. Foi escopado por pathspec e não causou
  dano, mas é desvio — não repita.
- Mensagem de commit **por arquivo** (`-F`), nunca `-m` com crase: crase em
  mensagem é substituição de comando.
- Todo comando começa com o `cd` da worktree ([[feedback_bash_cwd_resets_and_slips_to_the_primary]]).

---

## §6 — O gate COMPLETO da árvore combinada (depois das duas)

O `foundational-integrate.sh` **não é** o gate completo — ele roda a varredura
IMPACTADA. Antes do CLAUDE.md, rode na árvore primária:

1. **Workspace em DEBUG e em RELEASE.** Precedente registrado: o
   `ph2d-flip-colorize` panicava só em debug, e um gate da `line/FLIP` reprovava
   só em debug (kill de wall-clock mede o PERFIL).
2. **Os arch-gates de shell**, que só correm na varredura impactada e já chegaram
   vermelhos ao tip de uma linha:
   - `architecture_workspace_file_loc_cap`
   - `shells/desktop/tests/file_loc_caps.rs`
   - `no_tofu_glyphs`
   - `arch_safe_clamp_only`
   - `no_two_smoke_scenes_claim_the_same_level`
   - `every_panel_the_shell_drives_is_in_its_registry`
   - `architecture_panel_wiring_parity`
   - `architecture_adr_numbers_are_unique`
   - `node_id_collisions`
3. **Contrato congelado por GREP**, não por auto-relato: `NodeOp=2` ·
   `OpResolver=1` · `NodeManifest=8` · `Tool=12` · `RasterEditTool=5` ·
   `CanvasPaintTool=1` · `PanelEvent=4`.
4. **Suítes de GPU `#[ignore]`** na RTX (Flip, painter preview, mesh-render):
   sem adapter elas fazem *skip gracioso*, **que não é verde**.
5. **`physics_ecs_c9`** em debug E release — o hash tem de bater entre os dois.

**Orce 2-4 iterações**: o ship do integrador drena os latentes (§6 da skill).

---

## §7 — CLAUDE.md §5 (o último passo antes do ship)

Uma entrada por linha integrada, no molde das que já estão lá. O que **não** pode
faltar, porque é o que a próxima LLM lê antes de agir:

- Os **números CONTADOS** e o motivo (schema, scrollbar, registro de componentes,
  ADR) — e a **fonte** ao lado (`project.rs` é a fonte; a §5 é o espelho, e
  **espelho enverga**: este número já esteve falso quatro vezes).
- O que a integração **ACHOU** que nenhuma linha tinha visto (a família do
  endereço da §5.1 deste handoff é o item mais importante da jornada).
- Os **smokes** de cada linha, com a env var exata.
- O que fica **ABERTO**, com o número ao lado.

---

## §8 — Ship

Quem fecha a **última** integração da jornada faz `./scripts/ship.sh` → `git push
origin main` → babysit (DIRETRIZ §1.5.4 e §8) — **e isso exige confirmação
explícita do Enio**, que não está coberta pela ordem de integrar.
