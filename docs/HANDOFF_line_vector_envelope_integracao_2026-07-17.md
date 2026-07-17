# HANDOFF de INTEGRAÇÃO — `line/Vector`: Envelope / Warp (ADR-0123, Fatias A + B)

**Para:** o **agente integrador** (e o próximo implementador da linha).
**De:** a sessão de 2026-07-17 (assumiu a linha para construir o envelope/puppet warp).
**Estado:** a fatia está **fechada e smokada pelo Enio**. **NÃO integrei nem fiz ship** (Modo L,
CLAUDE.md §0.7) — este handoff existe para que o integrador o faça **por ordem explícita do Enio**.

> **Leia primeiro:** `CLAUDE.md` + `DIRETIVA_IMPLEMENTACAO.md`. Depois o **ADR-0123**
> ([`docs/architecture/decisions/0123-vector-envelope-warp-one-spine-cage-as-container-entity.md`](architecture/decisions/0123-vector-envelope-warp-one-spine-cage-as-container-entity.md))
> — é a fonte da verdade do desenho. Este handoff é só a **identidade da linha + os riscos de
> INTEGRAÇÃO**. Para o estado da linha ANTES do envelope (o Blend/Morph vivo, ADR-0122), o handoff
> irmão [`HANDOFF_line_vector_continuacao_2026-07-16.md`](HANDOFF_line_vector_continuacao_2026-07-16.md)
> continua sendo a fonte — este NÃO o duplica.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch** | `line/Vector` (worktree `Worktrees/line-Vector/`) |
| **HEAD** | `be6f61f1` (mais os fixes de tofu deste handoff — ver §7) |
| **Base do fork** | `4d203d48` (merge-base com `main`) |
| **`main` desde a base** | **+5 commits, todos `project-memory/`, ZERO código** — **não precisa rebase** (rebasear reescreveria 39+ commits de uma linha smokada por nada) |
| **Contratos congelados encostados** | **NENHUM** (§4) |
| **Smoke** | **aprovado pelo Enio** (`PH2D_BUILD_SMOKE=11` — a elipse deforma; ele confirmou os anchors nas posições de perspectiva) |

**Os commits DESTA sessão** (do mais novo; o resto do `git log 4d203d48..HEAD` é das sessões
anteriores, já cobertas pelos handoffs anteriores):

- `be6f61f1` / `844a61a3` / `0461745c` — investigação do smoke + limpeza dos logs de debug
- `918749ab` — **Fatia B**: o Envelope Object vivo (entidade + host + smoke)
- `1e389055` — o gesto **Quad** como `Warp` (homografia de Heckbert em f64 + gate do horizonte)
- `e257fba9` — `Cargo.lock` (a crate nova entra no lock)
- `9ada9a6f` — **Fatia A**: a espinha do envelope (`sample + fit`) + o gate da armadilha
- `40c06fbd`..`c62138c1` — o **ADR-0123** + a wave de pesquisa (`21_pesquisa_envelope_warp.md`)

---

## §2 — O que a fatia entrega (contexto de 30 s)

**Deformar geometria Bézier por um mapa NÃO-AFIM** (ADR-0123). A espinha é `densificar → deformar →
refitar`; envelope e puppet são **um pipeline, dois gestos** (o que troca é `warp: R2→R2`). Esta
fatia entrega o motor + o **1º gesto (Quad/perspectiva)** + o **Envelope Object vivo** (a forma que
é a fonte autorada deformada por uma gaiola, re-cozida por frame). **Sem UI ainda** — a gaiola é
autorada (a cena de smoke a define); arrastar os cantos no modo Node é a fatia seguinte.

**A armadilha que decide tudo:** `for v in verts { v = warp(v) }` **está errado** (só afim comuta
com Bézier) — e erra acertando os cantos e mentindo no meio do segmento. O gate-mãe é **invariância
à subdivisão** (repro do bug aberto do Inkscape #10547).

---

## §3 — Riscos de INTEGRAÇÃO (DIRETRIZ §1.5.9.2–3)

### 3.1 Foundational tocado, e por quê

| Arquivo | O quê | Forma |
|---|---|---|
| `crates/ph2d-ecs/src/vec_envelope.rs` | **NOVO** — o componente `VecEnvelope` | Arquivo próprio (isolado por construção, §1.5.2.1) |
| `crates/ph2d-ecs/src/lib.rs` | `mod vec_envelope;` + `pub use` | **Aditivo** |
| `crates/ph2d-ecs/src/scene/registry.rs` | `reg.register::<VecEnvelope>(…)` + a contagem | **Aditivo** — mas vide 3.2 |
| `crates/ph2d-render/src/registry.rs`, `crates/ph2d-script/src/registry.rs` | só a **contagem** do teste | vide 3.2 |
| `shells/desktop/src/*` | o host do envelope (5 arquivos) | vide 3.3 |

**Crate NOVA:** `crates/ph2d-vec-envelope/` (o motor). Glob-membership (`crates/*`) — **zero edição
no `Cargo.toml` do workspace**. Deps: `ph2d-vec-scene` + `kurbo = "0.13"`. **A `ph2d-vec-scene`
continua pura** (só `serde` + `postcard`); kurbo é INTERNA à crate nova, mesmo arranjo da
`ph2d-vec-boolean`/`-blend` — sem skew (é a mesma instância `0.13.0` do vello).

### 3.2 ⚠️ NÚMEROS QUE SOMAM — conte, não escolha

Três gates afirmam a CONTAGEM de componentes registrados. **VecEnvelope soma +1 em cada.** Se outra
linha também registrou um componente, **o valor certo não está em nenhum dos dois lados**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]):

| Arquivo | Esta linha diz | Se outra linha somou, RECONTE |
|---|---|---|
| `crates/ph2d-ecs/src/scene/registry.rs` | `reg.len()` **31 → 32** | ✔ |
| `crates/ph2d-render/src/registry.rs` | `reg.len()` **32 → 33** | ✔ |
| `crates/ph2d-script/src/registry.rs` | `reg.len()` **32 → 33** | ✔ |

> Estes 31/32/32 já **incluem** o `VecBlend`+`VecMorph` das sessões anteriores (que somaram 29→31).
> Contra `main` (que está na base, só +docs) não há conflito — mas se o integrador combinar com
> **outra linha** que registrou componente, some tudo.

### 3.3 Shell tocado

| Arquivo | O quê |
|---|---|
| `shells/desktop/src/envelope_live.rs` | **NOVO** — o host (create/attach/upkeep/recook), espelho do `morph_live` |
| `shells/desktop/src/envelope_live_tests.rs` | **NOVO** — 6 gates de host |
| `shells/desktop/src/render_loop/mod.rs` | 2 linhas: `envelope_live::upkeep` (antes do settle) + `recook` (depois do build) |
| `shells/desktop/src/app_state.rs` | campo `vec_envelope_pending` |
| `shells/desktop/src/main.rs` | `mod envelope_live;` + init do campo |
| `shells/desktop/src/vec_transform.rs` | `+ VecEnvelope` no filtro do `settle_origins` |
| `shells/desktop/src/build_smoke.rs` | a cena `PH2D_BUILD_SMOKE=11` |
| `shells/desktop/Cargo.toml` | dep `ph2d-vec-envelope` (`cargo machete` — é usada) |
| `shells/desktop/tests/settle_skips_every_derived_geometry.rs` | `+ "VecEnvelope"` no `DERIVED` |

**O gate `settle_skips_every_derived_geometry` tem DOIS testes que se completam** — o de presença
vê o `envelope_live.rs` forçar identidade e exige `VecEnvelope` na lista. Os dois passam.

**Sem ids de chrome novos** (sem painel ainda) → `node_id_collisions.rs` e o
`architecture_panel_wiring_parity` **não são tocados** por esta fatia. Sem variant de enum novo
(o `DrawMode` não foi tocado). **`.typos.toml` e `CLAUDE.md` NÃO foram tocados** (ímãs de
conflito intactos).

### 3.4 O que SÓ o `ship.sh` pega (DIRETRIZ §1.5.9.5)

- **3 deps novas** (`cargo machete`): a crate `ph2d-vec-envelope` (`ph2d-vec-scene` + `kurbo`) e o
  `ph2d-vec-envelope` no shell. Todas usadas; nenhuma cria ciclo (o motor não depende de nada do
  shell; a `ph2d-vec-scene` não ganhou dep).
- **`no_tofu_glyphs`** — o gate arch de UI. **Eu introduzi um `↔` (U+2194) e um `≠` (U+2260) em
  mensagens de assert e os drenei** (§7). O gate mora em `ph2d-editor-core` e varre `shells/desktop`
  — **rode `cargo nextest run --workspace` antes de declarar verde**, não só as crates tocadas
  ([[feedback_no_tofu_arrows_in_string_literals]]).
- clippy latente / RUSTSEC / fmt pré-fork: nada conhecido.

---

## §4 — Contratos congelados (§1.5.9.4)

**NENHUM.** `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` intactos.
`architecture_vector_contract_surface` escaneia só `ph2d-vector-doc`/`-traits`, que a linha não
toca. O motor novo (`ph2d-vec-envelope`) tem contrato **próprio, ainda não congelado** (re-congelar
é follow-up de toda a família `ph2d-vec-*`).

---

## §5 — Estado dos gates e do SMOKE (§1.5.9.6)

**Workspace: verde** (`cargo nextest run --workspace --features panel-vector`, após os fixes de
tofu do §7). Clippy limpo. LOC folgado em todos os arquivos novos (maior: `build_smoke.rs`
566/700).

Gates da fatia (todos mutation-tested):

| Gate | Onde | Prova |
|---|---|---|
| **invariância à subdivisão** | `ph2d-vec-envelope/tests/split_invariance.rs` (4) | partir uma cúbica e deformar == deformar a inteira; o ingênuo FALHA (prova de mutação); fixture curvo/isoparamétrico |
| **gesto Quad** | `ph2d-vec-envelope/src/quad.rs` (7 unit) | repouso=identidade · cantos mapeiam exato · paralelogramo=afim · **convexo mantém o horizonte fora** (+ irmão presença) · jacobiana=diferença finita |
| **host do envelope** | `shells/desktop/src/envelope_live_tests.rs` (6) | repouso não muda · gaiola puxada deforma **pelo motor** (byte-a-byte) · vive na identidade · **a fonte autorada sobrevive** · degenerada congela · **a sequência REAL do frame deforma** (o gate que reproduziu o smoke) |
| **settle DERIVED** | `shells/desktop/tests/settle_skips_every_derived_geometry.rs` (2) | o `VecEnvelope` está no filtro E na lista |

**Cena pronta (não peça montagem ao Enio) — o `cd` é ABSOLUTO e vai JUNTO** ([[feedback_run_command_include_cd]]):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && PH2D_BUILD_SMOKE=11 cargo run -p ph2d-host-desktop --features panel-vector
```

**Smokado e aprovado** (2026-07-17): a elipse nasce deformada por gaiola de perspectiva; o Enio
confirmou os anchors nas posições certas (laterais sobem, topo pinça). **Lição do smoke:** o
primeiro screenshot parecia não-deformado — era **stale** (frame anterior à criação do envelope);
o efeito numa elipse é um egg **suave** (não um cone dramático), porque a perspectiva de uma curva
fechada é isso. A correção mid-segment está gateada; a olho, numa elipse, é sutil.

---

## §6 — A FILA (a ordem é do Enio)

Dentro do envelope (ADR-0123, as fatias que faltam):

1. **Arrastar os cantos da gaiola** no modo Node (a interação — o gesto vivo). Precisa de alças
   próprias (não o gizmo de sprite — *um gizmo sobre geometria que se move DOBRA*, ADR-0122).
2. **Mover o objeto-envelope inteiro** (a fonte está congelada em MUNDO no componente; mover o
   conjunto re-baka ou aplica um afim aos cantos + fonte).
3. **O container multi-filho** (o ADR §2 quer 1 gaiola para N formas; esta fatia é **1-para-1**, o
   caso primário — o "Make with Warp" de um objeto só).
4. **Release / Expand** (materializar + soltar a gaiola).
5. **O painel** (seção Envelope: Fidelity + presets + Quad/4-curvas/Pinos).
6. Os outros gestos: **presets** (Fatia C, geradores de gaiola), **4 curvas de lado** (Fatia D,
   Coons), **pinos/MLS** (Fatia E — ⚠️ o MLS DOBRA a ~90°, o `break_cusp` do `WarpedCubic` volta
   `None` de propósito e **tem de** ser implementado para ele).

Depois do envelope, a fila herdada (handoff anterior): Replace Spine / Smooth Color do Blend · Live
Path Effects como nós · texto em caminho · trim path · repeater · largura variável.

---

## §7 — Nota do integrador: os fixes de tofu

Este handoff **corrige 2 glyphs tofu** que eu introduzi e que só o gate `no_tofu_glyphs` (workspace)
pegou:

- `envelope_live_tests.rs:176` — `↔` (U+2194) numa mensagem de assert → `<->`
- `quad.rs:351` — `≠` (U+2260) numa mensagem de assert → `!=`

**Os dois são commitados junto com este handoff.** Se o `git log` mostrar o handoff sem esses
fixes, algo se perdeu — recheque o `no_tofu_glyphs` antes de shippar. É a lição do §3.4:
verde-de-crate-tocada ≠ verde-de-workspace.

---

## §8 — Resumo de fechamento (o formato da DIRETRIZ)

- **DoD:** teste de seam comportamental verde (os 6 gates de host, incl. o que reproduz a sequência
  do frame) + **smoke do Enio** ✅.
- **Contratos congelados:** nenhum.
- **Foundational:** `ph2d-ecs` (componente novo em arquivo próprio + registro), projetado para
  isolamento.
- **Números que somam:** 3 contagens de registry (§3.2).
- **Latentes drenados:** 2 glyphs tofu (§7).
- **Aberto:** a interação (arrastar a gaiola), o container multi-filho, Release/Expand, o painel, os
  gestos C/D/E (§6).
- **Fecha e PARA:** não integro nem faço ship. Espero ordem explícita do Enio.
