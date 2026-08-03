# HANDOFF DE INTEGRAÇÃO **MESTRE** — `line/Painter` (2026-08-02)

> Para o **agente integrador**. A linha **NÃO integrou e NÃO fez push** — ela fecha, entrega isto e
> para (CLAUDE.md §0.7).
>
> **Este documento SUPERSEDE os dois handoffs por-metade** e é o único que você precisa ler inteiro:
> - [`HANDOFF_INTEGRACAO_line_Painter_S3_2026-08-01.md`](HANDOFF_INTEGRACAO_line_Painter_S3_2026-08-01.md) — a metade **A** (o motor de undo/journal)
> - [`HANDOFF_INTEGRACAO_line_Painter_watercolor_cadence_2026-08-02.md`](HANDOFF_INTEGRACAO_line_Painter_watercolor_cadence_2026-08-02.md) — a metade **B** (a aquarela)
>
> Eles seguem válidos no DETALHE de cada wave; aqui está o que a integração precisa.

---

## 1 — Identificação

| | |
|---|---|
| Branch | `line/Painter` |
| Worktree | `Worktrees/line-Painter` |
| HEAD | `bc0817ce3` **ou o commit desta correção** — confira com `git log -1` (o handoff é ele próprio um commit) |
| Base | `main` a **`a9f5977e9`** |
| **Rebase** | ✅ **`main` é ancestral de `HEAD` — o rebase é NO-OP hoje.** Re-confira antes de integrar (`git merge-base --is-ancestor main HEAD`) |
| Commits | **76** |
| Diff | **97 arquivos, +13.313 / −734** |
| Crates tocadas | `ph2d-tool-painter` · `ph2d-painter-brush` · `ph2d-wet-paint` · `shells/desktop` |

---

## 2 — O CHECKLIST de colisão (leia isto primeiro)

| pergunta | resposta | como conferi |
|---|---|---|
| `Cargo.toml` tocado? | **NENHUM** | `git diff main..HEAD --name-only \| grep Cargo.toml` → vazio |
| Dependência nova? | **NENHUMA** (o `rayon` já era do `ph2d-tool-painter` desde o ADR-0109) | idem |
| Crate nova? | **NENHUMA** | idem |
| `PROJECT_SCHEMA`? | **48 nos dois lados — a linha não toca `project.rs`** | `git show main:…/project.rs` vs local |
| Contrato congelado (§6)? | **INTACTO, 4/4 verde** — rodado, não auto-relatado | `cargo test -p ph2d-editor-core --test architecture_tool_contract_surface` |
| **ADR novo?** | **NENHUM** — os dois ADRs tocados são **EMENDAS** (appends) | ver §4 |
| Id / token / variant novo? | **NENHUM** | — |

⚠️ **A ausência de ADR novo é o item mais importante desta tabela.** O repo já teve **sete** colisões
de número de ADR entre linhas paralelas, sempre porque duas linhas escolheram o mesmo número. Esta
linha **não escolhe número nenhum**: ela apenda a ADRs existentes (0109 e 0146), e um append textual
não colide com um append de outra linha no mesmo arquivo a não ser no MESMO ponto — improvável, e
resolvido mantendo os dois.

---

## 3 — ⚠️ Os DOIS arquivos de alta colisão, e um defeito herdado

### 3.1 `CLAUDE.md` (+4 / −1)

A linha risca a frase *"Aberto: o passo segue work-limited … a próxima alavanca é a GPU"* da entrada
do Wet Paint e acrescenta a entrada **"O ITEM 3 — A GPU DO SOLVER — FECHOU POR MEDIÇÃO"**.

**Toda linha edita o `CLAUDE.md`.** Se houver conflito, ele é **textual e semanticamente disjunto**:
mantenha as duas metades. A frase riscada é da própria linha (o Wet Paint é dela).

### 3.2 `project-memory/MEMORY.md` (+1) — ⚠️ **PONTEIRO PENDURADO**

A linha acrescenta ao índice:

```
- [Largar a POSSE cega todo comparador daquele lado](feedback_dropping_ownership_blinds_every_comparator_that_reads_that_side.md) — …
```

⚠️ **O arquivo apontado NÃO está na branch e NÃO está no disco desta worktree.** Ele existe
**untracked na árvore PRIMÁRIA** (o `git status` da primária o mostra como `??`), deixado por uma
sessão anterior desta linha que escreveu o ponteiro mas nunca commitou o arquivo — a memória é
versionada no repo (CLAUDE.md §4) e o symlink do Claude Code resolve para a primária.

**Por que eu não "consertei" escrevendo o arquivo aqui:** ele existe untracked na primária, então uma
segunda versão minha faria o checkout da integração **recusar** (*"untracked working tree file would
be overwritten"*), e as duas versões seriam textos diferentes da mesma memória.

**O que fazer:** na árvore primária, `git add project-memory/feedback_dropping_ownership_*.md` junto
com a integração — o arquivo já está lá, escrito. (Há mais cinco memórias `??` na primária, da mesma
família; elas não têm ponteiro nesta branch, então são decisão do Enio, não da integração.)

---

## 4 — Os dois ADRs, e por que são emendas e não ADRs novos

### `ADR-0109` (rayon no `ph2d-tool-painter`) — **+56 linhas**

Emenda de 2026-08-02: **o passe de SECAGEM e o DESPEJO do mapa de umidade**, row-parallel.

Por que cabe sob ele e não pede ADR novo — os três invariantes do §2 do ADR valem **verbatim**: cada
linha de saída é função pura de um **snapshot imutável** (que já existia, porque a lei do decaimento é
independente de ordem *por desenho*), cada task escreve **só a própria fatia**, e não há RNG nem
transcendental. ⚠️ **E a redução, que é o que a cerca de contenção do §2 nomeia, é exatamente o caso
que ela ISENTA:** `max` sobre `u8` (o `wettest`) e `min`/`max` sobre índices (a bbox) na secagem, e
**nenhuma redução** no despejo — inteiros, associativos e comutativos, sem soma em ponto flutuante
cuja ordem entre threads pudesse mudar um byte.

A emenda traz também as **três curas que mediram ~1,00×** e foram descartadas, para ninguém as
reconstruir.

### `ADR-0146` (a GPU do solver do Wet Paint) — **+84 linhas**

Emenda 5: **os dois gatilhos MENSURÁVEIS do ADR fecharam por medição** (o último regime
work-limited, o K–M, foi re-medido de 4,75× para 1,1-1,4×). A recomendação não é revogada; ela é
re-precificada. Sobrevive só o gatilho hipotético.

---

## 5 — O que entra, em duas metades

### Metade A — o motor de UNDO / o journal (≈53 commits, sessões anteriores)

Detalhe em [`HANDOFF_INTEGRACAO_line_Painter_S3_2026-08-01.md`](HANDOFF_INTEGRACAO_line_Painter_S3_2026-08-01.md)
e no [doc 28](Painter/28_otimizacoes_o_que_funcionou.md) §5.60-§5.70. Módulos novos:
`undo_confine.rs` · `undo_delta_confine.rs` · `undo_delta_journal.rs` · `undo_elide.rs` ·
`undo_inspect.rs` · `undo_shape_state.rs` (+ os testes irmãos).

### Metade B — a AQUARELA (22 commits, 2026-08-02)

O retrato completo é o **[doc 32](Painter/32_aquarela_o_que_custa_hoje.md)** (o MAPA); o log
cronológico é o doc 28 §5.71-§5.77; os bugs cuja causa enganava são
[`BUGS_painter.md`](Painter/BUGS_painter.md) **#18 · #19 · #20 · #21**.

| o que | antes | agora |
|---|---|---|
| `CHROME wet` — o véu de umidade, **no shell** | 42,64 ms/quadro | **~6** |
| `secagem` (`dry_canvas_wet`) | 28,50 ms/quadro | **2,93** |
| `pour` (`pour_canvas_wet`) | 12,46 ms | **0,63** |
| `carimbo` com Smudge | 49,60 ms | **5,06** |
| a lavagem por **evento** de ponteiro | 2,56× num mouse de 960 Hz | 1 por QUADRO |
| pen-down | 268 MB alocados | curado |

**Veredito do Enio (2026-08-02):** *"pela primeira vez consegui pintar uma imagem de 4096 com fluidez
nos parâmetros padrão da aquarela"*.

**Módulos novos da metade B:** `wash_diag.rs` (o instrumento, irmão do `wet_diag`) ·
`tool/paint/watercolor_dry.rs` (split de LOC por assunto: *o que molha o papel* × *o que o seca*) ·
`watercolor_render/diag.rs` (o envelope) · os testes irmãos · a sonda
`measure_watercolor_pour.rs`.

---

## 6 — Superfície nova (para você detectar colisão)

Tudo abaixo é **privado ao `tool::paint`** ou interno ao shell — **nada é público fora da crate**:

- `PaintState` ganhou `canvas_wet_snapshot: Vec<u8>` (scratch do decaimento).
- `watercolor_backdrop` exporta `pub(super) WET_PAR_MIN` (o piso do pool, compartilhado pelos dois
  passes porque os dois caminham o mesmo mapa).
- `pub(super) pour_hardening_lut()` / `pour_hardening()` (a tabela de dureza do despejo).
- `crate::wash_diag` — `note_composite/stamp/pour/dry/pendown` + `take() -> WashRead`.
- Shell: `paint_perf.rs` ganhou a linha `AQUARELA`; `painter_bridge_wetness.rs` ganhou
  `clip_to_viewport` / `veil_downscale` / `build_veil(.., step)`.

⚠️ **Um leitor só do `wash_diag`.** `take()` **zera** os contadores — dois leitores publicariam
pedaços do mesmo quadro como se fossem quadros. Hoje o leitor é o `[paint-perf]`; a sonda
`measure_the_window_the_composite` também drena, e por isso é `#[ignore]` e roda com
`--test-threads=1`.

---

## 7 — Verde local (medido agora, `load average 0,22`)

| suíte | resultado |
|---|---|
| `ph2d-tool-painter` (release) | **966 passed, 0 failed**, 170 ignored |
| `ph2d-wet-paint` | 29 binários `ok` |
| `ph2d-painter-brush` | 2 binários `ok` |
| `shells/desktop` (release) | **79 binários `ok`, 0 FAILED** |
| clippy `--all-targets` | limpo |
| `architecture_panel_loc_cap` (LOC de `crates/`) | 3/3 |
| `file_loc_caps` (LOC da shell, 600) | 2/2 |
| `architecture_tool_contract_surface` | 4/4 |

⚠️ **O que só o `ship.sh` pega:** `machete` / `deny` / `audit` / `typos` / `fmt` na versão pinada. A
linha não acrescenta dep nem crate, então o risco ali é baixo — mas **rode o `ship.sh` inteiro**, é a
paridade com o CI.

⚠️ **Rode a suíte do painter em DEBUG também.** A linha tem um precedente registrado (o
`ph2d-flip-colorize` panicava só em debug, e a nota sobreviveu ao fato por três integrações); e um
gate desta família já reprovou só em debug por medir wall-clock em vez de razão.

---

## 8 — O que SMOKE-TESTAR (já aprovado pelo Enio, mas re-confirme pós-merge)

```
cd /home/enio/Documentos/Projetos/PH2D
env PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, pincel **250**, modo **Watercolor**, ~3 s de traço. No log `[paint-perf]`:

1. **`CHROME wet`** perto de **~6 ms** (era 42,6) — é o véu de umidade.
2. **`AQUARELA … pour`** e **`secagem`** perto de **~1 ms** cada (eram 9,7 e 12,8).
3. **`carimbo`** não deve subir com a tela (limitado pela PEGADA).
4. Com **Rewet 0,400** o `composite` fica em ~18 ms — **isso é esperado e está documentado** (doc 32
   §4); com Rewet 0 ele fica em ~7,7.

⚠️ **Nenhum número deste log significa nada com `load average` acima de ~5** — a linha `poca:` /
`ns/celula` é o detector (um dígito = máquina sã).

---

## 9 — Aberto, com número (nada disto bloqueia a integração)

1. **`build_rewet_fields` = 8,74 ms, 51% do composite** — dez box blurs em resolução cheia. O blur já
   é O(n) por prefix sums **e já é paralelo**: ele está no **piso de largura de banda** (2,1 ns/texel).
   As duas alavancas restantes são **decisão de produto**, com o preço de cada uma no
   [doc 32 §4](Painter/32_aquarela_o_que_custa_hoje.md):
   - **A — baixar `REWET_DS_SPREAD`**: o custo cai por `ds²` (8,74 → ~2,2 em `ds=2`), o mecanismo já
     existe, e o preço é o **LOOK**. O juiz é o **olho**, não um gate.
   - **B — cachear os campos derivados da base congelada**: `pres`/`wr`/`wg`/`wb` dependem só de
     `base` e `ground`, **os dois congelados pela sessão**, e são recomputados todo quadro. O preço é
     **4 planos canvas-sized: 268 MB a 4096²** — a classe que o ADR-0117 existe para não deixar passar.
     Se **A** entrar primeiro, **B** fica **16× mais barata em memória**.
2. **`composite max` de 163 ms** num quadro isolado do log do Enio (contra p50 de 10,6) — **sem causa
   atribuída**. O candidato é o composite de **commit** do pen-up; falta um log com histograma.
3. O **laço paralelo** do composite (5,45 ms) nunca foi decomposto por termo.

---

## 10 — Ordem de trabalho sugerida

1. `cd Worktrees/line-Painter && git rebase main` (hoje é no-op — confirme).
2. Integre com `--ff-only` se possível; senão, `scripts/foundational-integrate.sh` (o gate da árvore
   combinada).
3. ⚠️ **Na árvore combinada, rode os gates de `shells/desktop/tests/`** — eles **só correm na
   varredura impactada**, e um fechamento por `cargo test -p` por crate **não os alcança**. É a causa
   estrutural que já deixou duas linhas com arch-gate vermelho no próprio tip.
4. Confira `CLAUDE.md` e `project-memory/MEMORY.md` (§3) — e **commite a memória pendurada**.
5. `./scripts/ship.sh` inteiro; corrija todo `✗` antes de qualquer push.
