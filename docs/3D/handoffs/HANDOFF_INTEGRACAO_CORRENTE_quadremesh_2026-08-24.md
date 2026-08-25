# HANDOFF DA CORRENTE — as três linhas do QUAD REMESH ao `main` (2026-08-24)

> **Por que este documento existe:** o Enio perguntou se cada linha aberta desde o
> pivô da extracção tinha handoff. ⭐ **Tem — todas.** O que faltava era **um mapa da
> corrente**: quem descende de quem, o que já está no `main`, e o que o integrador
> encontra quando funde. Este documento é esse mapa, e **todo número aqui foi medido
> hoje contra o `main` de hoje** (`e69226157`), não contra o ponto de fork.
>
> ⛔ Ele **não substitui** os handoffs de linha — aponta-os. O mecanismo de cada obra
> vive no handoff dela.

---

## 1 — A CORRENTE, medida (⚠️ e ela não é o que se supunha)

```
main  e69226157  (hoje)
  │
  └── 5038249c6   ← ponto de fork COMUM das três linhas (o main de 24/08)
        │
        └── line/sculpt3d ······················ 15 commits ······ e9c9ec8db
              ├── e207f91d4   ⚠️ 1 commit SÓ da sculpt3d (toca APENAS o LEDGER)
              │
              └── line/quadextract ············· 12 commits ······ d9e2d204c
                    │
                    └── line/seamelim ·········· 11 commits ······ 997b55750
```

| linha | HEAD | commits sobre o `main` | worktree |
|---|---|---|---|
| `line/sculpt3d` | `e207f91d4` | 16 | `Worktrees/line-sculpt3d` |
| `line/quadextract` | `d9e2d204c` | 27 | `Worktrees/line-quadextract` |
| `line/seamelim` | `997b55750` | **38** | `Worktrees/line-seamelim` |

⭐⭐ **`line/seamelim` contém 38 dos 39 commits da corrente.** Só `e207f91d4` fica de fora.

⛔⛔ **CORRECÇÃO A UMA AFIRMAÇÃO QUE JÁ CIRCULOU:** o handoff da `line/seamelim` dizia,
na 1ª redacção, que a ordem era *«sculpt3d → quadextract → seamelim»*, como se fosse
uma corrente de três elos. **Não é.** A `quadextract` bifurcou da `sculpt3d` em
`e9c9ec8db`, e a `sculpt3d` ganhou **um** commit **depois** disso. ⇒ `sculpt3d` **não é
ancestral** de `quadextract` (`git merge-base --is-ancestor` di-lo, e foi assim que o
erro apareceu). *Uma linhagem escrita de memória é uma linhagem que envelhece.*

---

## 2 — Os HANDOFFS: existem todos, e três ainda não entraram

| obra | handoff | onde está |
|---|---|---|
| sculpt3d — quad remesh, 22/08 | `HANDOFF_INTEGRACAO_line_sculpt3d_QUADREMESH_2026-08-22.md` | ✅ **já no `main`** |
| sculpt3d — quad remesh, 23/08 | `HANDOFF_INTEGRACAO_line_sculpt3d_QUADREMESH_2026-08-23.md` | ✅ **já no `main`** |
| sculpt3d — a EXTRACÇÃO, 24/08 | [`HANDOFF_INTEGRACAO_line_sculpt3d_EXTRACAO_2026-08-24.md`](HANDOFF_INTEGRACAO_line_sculpt3d_EXTRACAO_2026-08-24.md) | ⏳ vem com a `seamelim` |
| quadextract — a extracção existe, 24/08 | [`HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md`](HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md) | ⏳ vem com a `seamelim` |
| seamelim — a costura por eliminação, 24/08 | [`HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md`](HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md) + [auditoria](AUDITORIA_line_seamelim_2026-08-24.md) | ⏳ é o desta linha |

⭐ **Os três pendentes já estão TODOS na árvore da `line/seamelim`** — integrá-la traz
os três de uma vez.

---

## 3 — A ORDEM, e ela reduz-se a dois passos

1. ⭐ **`line/seamelim` primeiro, sozinha.** Ela subsume os 15 da `sculpt3d` e os 12 da
   `quadextract`. Integrar qualquer uma das outras antes só multiplica rebases.
2. **Depois, `e207f91d4`** — o único commit que sobra, e ele toca **apenas**
   `docs/3D/cleanroom/LEDGER_quadwild.md` (`+15/−2`). Ver o §5, que é onde ele fica
   delicado.

⛔ **Não integre `line/sculpt3d` ou `line/quadextract` como linhas.** Elas não têm nada
de código que a `seamelim` não traga; integrá-las faria o integrador ver a mesma obra
duas ou três vezes — que é exactamente o modo de falha que a regra da base não-`main`
existe para evitar.

---

## 4 — ⛔⛔ A COLISÃO REAL: o ADR-0164 está tomado DUAS vezes

| | ficheiro |
|---|---|
| **no `main` de hoje** | `0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md` |
| **nas três linhas** | `0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md` |

O `main` ganhou **0164, 0165 e 0166** enquanto estas linhas corriam (vieram da
`line/components`). ⇒ ⭐ **o ADR da extracção passa a ser o `0167`**, que é o próximo
livre contado contra o `main` de hoje.

⚠️ **O rename alcança 9 ficheiros** (contagem medida):

| ficheiro | referências |
|---|---|
| `docs/3D/cleanroom/LEDGER_quadwild.md` | 7 |
| `CLAUDE.md` | 2 |
| `docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_EXTRACAO_2026-08-24.md` | 2 |
| `docs/3D/00-INDEX.md` · `docs/3D/cleanroom/README.md` · `docs/3D/cleanroom/TRIAGEM_quad_remesh.md` · `docs/3D/cleanroom/fixtures/README.md` · `docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md` · `docs/architecture/decisions/README.md` | 1 cada |

⇒ `git mv` do ficheiro + varredura das 9 referências + **regenerar o índice**
(`bash scripts/adr-index.sh`, que é derivado — ⛔ nunca à mão).

⚠️ *Isto é a lei do §5.0 do `CLAUDE.md` a acontecer: «número que soma entre linhas se
CONTA, nunca se escolhe» — e a colisão passou **muda** porque as duas linhas escreveram
o mesmo literal em ficheiros de nome diferente.*

---

## 5 — ⚠️ O LEDGER da sala limpa é o ponto delicado do passo 2

Medido (só por metadados — o LEDGER está fora do alcance de quem escreve isto):

| comparação | diferença |
|---|---|
| `line/seamelim` **contra** `line/quadextract` | **idênticos** (zero linhas) |
| `line/seamelim` **contra** `line/sculpt3d` | `+610 / −14` |
| o que `e207f91d4` escreveu | `+15 / −2` |

⇒ ⛔ **Fundir a `seamelim` e depois tomar o lado dela no LEDGER APAGA o que
`e207f91d4` escreveu.** O ficheiro é o registo legal da sala limpa; a reconciliação
não é um detalhe de merge.

⭐ **Quem a deve fazer é a janela R-pós** (ela pode ler os dois lados; quem escreveu
este documento não pode, por desenho). O bloco dela está em
[`NEXT_R-POS_eliminacao.md`](../cleanroom/NEXT_R-POS_eliminacao.md), e este item
pertence ao passo 4 dela (incidentes/registo).

---

## 6 — O que o merge encontra, medido a seco (`git merge-tree`, sem tocar em nada)

**`main` + `line/seamelim`** — três conflitos, e nenhum é de código:

| ficheiro | espécie | ⭐ como se resolve |
|---|---|---|
| `docs/architecture/decisions/README.md` | conteúdo | ⛔ **DERIVADO** — nunca à mão: `bash scripts/adr-index.sh` (depois do rename do §4) |
| `project-memory/MEMORY.md` | conteúdo | união trivial: o `main` acrescentou **15** linhas de índice, esta corrente acrescentou **1** |
| `scripts/cleanroom-sweep.sh` | **add/add** | ⭐ **os dois lados são o MESMO ficheiro** (blob `49cb96e40…`, 102 linhas, diff vazio) — o git só o marca porque o caminho não tem ancestral comum. Tome qualquer lado |

✅ **`CLAUDE.md` funde AUTOMÁTICO** — os dois lados editaram linhas diferentes do §5.
✅ **`Cargo.lock` e `shells/desktop/Cargo.toml` fundem automático.**

**`main` + `line/sculpt3d`** — os mesmos dois primeiros, sem o `cleanroom-sweep.sh`.

---

## 7 — Superfície de colisão contra o `main` DE HOJE

⚠️⚠️ **O `collision-surface.sh` mede contra o PONTO DE FORK, não contra o `main` de
agora** — ele imprime `merge-base 5038249c6` nas três linhas, e a coluna «base» é a do
`main` de **24/08**. *É a armadilha que o próprio `main` acabou de documentar*
(`77ff95113`). ⇒ os números abaixo foram **re-medidos à mão** contra `e69226157`:

| grandeza | `main` de hoje | o que a corrente tem | veredito |
|---|---|---|---|
| `PROJECT_SCHEMA` (3 sítios) | **97** · tripla `(97, 13, 14)` | **não tocado** por nenhuma das três | ✅ o rebase adopta o 97 sem conflito |
| `VEC_SCENE` · `FLIP` · `DOC_VERSION` | 14 · 13 · 18 | não tocados | ✅ |
| registo de componentes (3 contadores) | 69 · 70 · 70 | não tocados | ✅ |
| contrato congelado (`node.rs`, `tool.rs`) | — | **intocados** nas três | ✅ **nenhum ADR de contrato é preciso** |
| ADR | último **0166** | as três criam `0164` | ⛔ **renumerar para `0167`** (§4) |
| `Cargo.lock` | — | 1 pacote novo, `ph2d-quadextract` — **interno** | ✅ aresta interna, não dependência externa |
| tectos de LOC | 700 / 600 / 500 / 650 | nenhum ficheiro passa | ✅ |
| marcadores de conflito | — | nenhum | ✅ |

---

## 8 — O que SÓ o `ship.sh` apanha

- `cargo fmt --all` — a `seamelim` formatou **ficheiro a ficheiro** (só os dela); as
  outras duas linhas não foram re-verificadas hoje.
- `typos` — os docs das três são densos em português e **nunca** correram por ele.
- `machete` / `RUSTSEC` — ⭐ **nenhuma dependência externa nova** em nenhuma das três.
- `clippy --all-targets` — ✅ zero avisos nas 3 crates que a `seamelim` tocou; ⏳ as
  crates que só a `quadextract` tocou não foram re-verificadas depois de o `main` andar.

---

## 9 — O que SMOKAR depois de integrar

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_SCULPT3D_SMOKE=35 PH2D_RETOPO_EXTRACT=1 cargo run -p ph2d-host-desktop --release
```

Botão **`Quad Retopology`**. ⭐ A malha tem de cobrir a peça **inteira, sem buracos**
(era esse o report do Enio). `PH2D_GRIDMAP_WELD=0` no mesmo comando volta ao
comportamento antigo, e os buracos devem reaparecer — é o controlo.

⚠️ **Tudo o que shipa aqui está atrás de `PH2D_RETOPO_EXTRACT`, que nasce desligado.**
Sem a env, o botão é byte-idêntico ao de antes, e há gate a contá-lo.

---

## 10 — Onde ler o mecanismo de cada obra

| obra | documento |
|---|---|
| a extracção existe (ADR + espec + fixtures) | [`..._line_sculpt3d_EXTRACAO_2026-08-24.md`](HANDOFF_INTEGRACAO_line_sculpt3d_EXTRACAO_2026-08-24.md) |
| a extracção + o arredondamento inteiro (G5) | [`..._line_quadextract_2026-08-24.md`](HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md) |
| a costura por eliminação (a casca fecha) | [`..._line_seamelim_2026-08-24.md`](HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md) · [auditoria](AUDITORIA_line_seamelim_2026-08-24.md) |
| a sala limpa (espec, atestados, fixtures) | [`docs/3D/cleanroom/`](../cleanroom/) |
