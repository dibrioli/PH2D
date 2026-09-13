# `line/shell-folhas` — as folhas partilhadas que prendem três famílias

> **Por que esta linha existe, e por que ela não é um capricho de arrumação:** ao fechar a Fase B,
> **três linhas independentes chegaram ao mesmo diagnóstico, cada uma por seu lado** —
>
> | linha | o que ela disse |
> |---|---|
> | `app-motion` | 13 pedaços partilhados prendem os 26 bloqueadores; o maior (`vec_entities`) prende 7 |
> | `app-flip` | *«duas funções de outra família prendem **84 %** da flip»* — `vec_transform` e `name_unique` |
> | `app-vec` | *«o bloqueador desta família **não é a `App`** — é `name_unique`»* |
>
> ⭐ E a `physics`, a batedora, tinha dito o mesmo com outras palavras: *as 5 portas do `AppHost`
> chegam; o que ficou na shell ficou por **PARTILHA**, não por acoplamento* — e nomeou três folhas.
>
> ⛔⛔ **Esta linha NÃO abriu ontem de propósito.** As peças maiores são território da `line/app-vec`,
> que estava a mexer nelas naquele momento (o `vec_glyph` saiu da shell na árvore dela enquanto a
> medição corria). Abrir ali seria a colisão de mesmo-símbolo que a DIRETRIZ §1.5.5 manda evitar.
> **Hoje não está ninguém**, e é por isso que ela abre agora.

---

## §1 — O GRAFO, medido no `main` de 2026-09-12 (depois das três integrações)

⚠️ **Estes números são de hoje.** A lista de ontem já não descreve a árvore — a Fase B da `vec`
mexeu no meio dela. *Uma régua de folhas envelhece a cada integração.*

### Camada 0 — puras, **zero arestas de volta à shell** (saem primeiro, sem cascata)

| ficheiro | LOC | quem consome |
|---|---:|---|
| `render_loop/inspector_ordering.rs` | 595 | physics (+ o que o censo da physics mediu: 11) |
| `audio.rs` | 569 | motion 6 · physics 1 |
| `preview_drive.rs` | 530 | physics 5 · field3d · vec · flip |
| `render_loop/off_canvas.rs` | 322 | consumido por `vec_entities` |
| `name_unique.rs` | 181 | physics 4 · sculpt3d · vec · flip |
| `transport.rs` | 69 | physics 5 · motion 2 |
| `modal.rs` | 24 | vec 5 · physics 2 · field3d |
| **total** | **2 290** | **7 ficheiros** |

⭐ **Nenhum destes sete toca `App` ou `gfx`, e nenhum tem uma única `crate::` para a shell.** Medido
em código (com as linhas de comentário retiradas — a armadilha §2.12). *Eles ficaram na shell por
PARTILHA, e é literalmente só isso.*

### Camada 1 — o **CICLO**, e é a armadilha que decide o escopo

```
vec_entities (269) ──▶ name_unique ✓c0 · off_canvas ✓c0 · morph_set ─┐
                                                                      │
morph_set (489) ──▶ vec_entities · vec_transform ─────────────────────┤
                                                                      │
vec_transform (324) ──▶ vec_entities ◀────────────────────────────────┘
```

⛔⛔ **`vec_entities` → `morph_set` → `vec_entities` é um ciclo.** Os **três** saem juntos ou nenhum
sai — e quem tentar tirar um vai descobrir isso a meio, com o `cargo check` a apontar para o sítio
errado. **1 082 LOC, 3 ficheiros.**

### ⛔ O que fica FORA, e porquê (não é esquecimento)

| ficheiro | LOC | porque fica |
|---|---:|---|
| `undo.rs` | 389 | depende de `crate::flip::entities` — **território da `line/app-flip`** (regra 5 da Fase B: não se edita a árvore de outra linha) |
| `project.rs` | 249 | depende de `undo` (acima) e de quatro `project_*` |
| `project_library.rs` | 132 | depende de `asset_index_build` e `project_catalogs` — cauda do `project` |
| `build_smoke.rs` | 515 | **não é folha**: `gfx` em 41 sítios |
| `init.rs` · `app_state.rs` · `input_dispatch.rs` | — | são a **raiz de composição** e as costuras. Ficam por desenho |

---

## §2 — ⛔⛔ A decisão de desenho: UMA FOLHA POR ASSUNTO, nunca um saco

A tentação é óbvia e está **errada**: uma crate `ph2d-shell-folhas` com os dez ficheiros lá dentro
resolve o `cargo check` e cria um **saco** — uma crate cuja razão de existir é *«o que sobrou»*.
No dia seguinte toda a gente lhe acrescenta coisa, e ela vira a segunda shell.

⇒ **Agrupe por ASSUNTO, e o número de crates sai da medição, não de um palpite.** A pergunta é *«o
que estas peças respondem?»*, e o teste é: **um nome que descreva a crate sem usar a palavra
«diversos», «comum», «shared» ou «utils»**. Se não conseguir o nome, o agrupamento está errado.

⚠️ **E há uma cerca dura:** ⛔ `vec_entities`/`vec_transform`/`morph_set` **NÃO vão para a
`ph2d-app-vec`**. Elas são consumidas pela `flip` e pela `motion`, e pô-las na crate de uma família
faria duas famílias depender de uma terceira — família→família é exactamente o acoplamento que o
ADR-0075 existe para impedir. *Uma peça partilhada por três famílias é uma FOLHA, por definição.*

---

## §3 — O BLOCO (cole isto na janela nova)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA — Modo L · W2 FOLHAS       (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/shell-folhas.  Ela é CURTA e tem um alvo
único: tirar da shell as FOLHAS PARTILHADAS que hoje prendem três
famílias inteiras (~150 mil linhas à espera delas).

⭐ POR QUE ELA EXISTE: três linhas independentes fecharam a Fase B com o
   MESMO diagnóstico, cada uma por seu lado — a motion ("13 pedaços
   partilhados prendem os 26 bloqueadores"), a flip ("duas funções de
   outra família prendem 84% de mim") e a vec ("o meu bloqueador não é
   a App, é o name_unique"). Você tira a pedra do caminho das três.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh          # tem de dizer `workstation`
2. cd /home/enio/Documentos/Projetos/PH2D && git status -sb
      → você está na RAIZ, em main, limpa.
3. mkdir -p Worktrees
   git worktree add -b line/shell-folhas Worktrees/line-shell-folhas main
4. cd Worktrees/line-shell-folhas
   pwd && git branch --show-current    # DEVE dizer line/shell-folhas
5. cargo check -p ph2d-editor-core     # warm-up; o 1º build é frio
6. bash scripts/mergiraf-setup.sh      # idempotente; ✗ não é bloqueio
7. LEIA INTEIRO:
   a) docs/archive/integracao-jornadas/BLOCO_ABERTURA_LINHA_FOLHAS_2026-09-12.md
      — o §1 é o GRAFO MEDIDO (as três camadas, o ciclo, e o que fica
        fora com o motivo); o §2 é a decisão de desenho, que é LEI aqui.
   b) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
      — a §1.2 é literalmente a sua tarefa ("o que é partilhado com
        outra família sai para uma folha"), e a §2 tem 12 armadilhas
        medidas, cinco com modo de falha MUDO.
   c) docs/IntegracaoMultiAgente/DIRETRIZ.md §0, §1.5, §6.7
8. Reporte "linha das folhas pronta" e SIGA (não pare).

A TAREFA — o alvo, medido no main de 12/09:
  CAMADA 0 (puras, ZERO arestas à shell — comece por aqui):
    render_loop/inspector_ordering.rs  595
    audio.rs                           569
    preview_drive.rs                   530
    render_loop/off_canvas.rs          322
    name_unique.rs                     181
    transport.rs                        69
    modal.rs                            24
    ─────────────────────────────────────
                                     2 290 LOC / 7 ficheiros

  CAMADA 1 (⛔ é um CICLO — os três saem JUNTOS ou nenhum sai):
    vec_entities ──▶ morph_set ──▶ vec_entities, e vec_transform ──▶ vec_entities
    vec_entities.rs   269 · morph_set.rs 489 · vec_transform.rs 324
    ─────────────────────────────────────
                                     1 082 LOC / 3 ficheiros

  ⛔ FICAM FORA, com motivo (§1): undo.rs (atravessa para a família
     flip — território de outra linha) · project.rs e project_library.rs
     (dependem do undo) · build_smoke.rs (gfx em 41 sítios, não é folha)
     · init/app_state/input_dispatch (raiz de composição e costuras).

⛔⛔ A DECISÃO DE DESENHO, e ela é LEI (§2):
  UMA FOLHA POR ASSUNTO, nunca um saco. Uma crate `ph2d-shell-folhas`
  com os dez lá dentro resolve o compilador e cria a SEGUNDA SHELL — no
  dia seguinte toda a gente lhe acrescenta coisa. O teste do nome: se
  você não consegue descrever a crate sem dizer «comum», «shared»,
  «diversos» ou «utils», o agrupamento está errado.
  ⛔ E vec_entities/vec_transform/morph_set NÃO vão para a ph2d-app-vec:
  a flip e a motion consomem-nas, e pôr uma peça partilhada na crate de
  UMA família faz duas famílias depender de uma terceira (ADR-0075).

O FIM DA LINHA:
  · as 10 peças fora de `shells/desktop`, em crates-folha nomeadas por
    assunto, com a shell a compilar e os testes todos verdes;
  · a PROVA: `cargo nextest list --workspace --cargo-profile ci-test`
    ANTES (na base) e depois, comparados por
    `python3 scripts/nextest-list-diff.py antes.txt depois.txt` —
    ONLY-A vazio, ou a linha não fechou;
  · no handoff, para o integrador: o LOC medido da shell no fim, e uma
    linha por família a dizer o que ela passa a poder tirar.

REGRAS PERMANENTES:
A. Tudo DENTRO de Worktrees/line-shell-folhas/. O mesmo path relativo
   existe na raiz: editar lá é editar a árvore ERRADA. `pwd` na dúvida.
C. Commits locais frequentes, `git commit --no-verify`. NUNCA push,
   NUNCA --force, NUNCA `git add -A`.
D. `git rebase main` no início. Cargo.lock ou registry-init em
   conflito: NUNCA na mão — regenere.
E. Fechar = gate batched 1× (nextest-impacted + clippy --all-targets +
   auditoria ≥2 lentes) e PARE. Você NÃO integra e NÃO roda o
   foundational-integrate.sh — é do integrador, por ordem do Enio.
   ⚠️ E corra `cargo test -p ph2d-host-desktop --test it` À PARTE: o
   nextest-impacted NÃO apanha `shells/desktop/tests/it/`, e foi
   exactamente aí que a flip reprovou na integração de 12/09.
H. HANDOFF de integração obrigatório (DIRETRIZ §1.5.9).
I. DEIXE O SMOKE COMPILADO na sua worktree, 2 corridas, a 2ª colada no
   handoff (Finished em segundos, ZERO Compiling):
   cargo build -p ph2d-host-desktop --profile smoke
⛔ NÃO toque no TETO_LOC do the_shell_only_shrinks — é um número que
   soma entre linhas, e quem o reconta é o integrador (ele VAI reprovar
   na sua worktree assim que você tirar código; isso é o seu marcador
   de progresso, não um defeito).
═══════════════════════════════════════════════════════════════════
```

---

## §4 — O que esta linha destranca (para o integrador, no dia da fusão)

| família | hoje na shell | o que ela disse que a prende |
|---|---:|---|
| `motion` | 100 451 | 13 folhas; a maior (`vec_entities`) prende 7 dos 26 bloqueadores |
| `vec` | 30 680 | `name_unique` é a **raiz** do grafo dela |
| `flip` | 18 845 | `vec_transform` + `name_unique` prendem **84 %** |

⚠️ **Nenhum destes números é uma promessa de que tudo sai.** São o que as três linhas mediram como
**bloqueado por estas peças** — o que de facto sai mede-se depois, com o fecho outra vez. *A régua
de um bloqueio é o fecho, nunca a contagem de citações* (HOWTO §2.12, a lição que a motion pagou a
30×).
