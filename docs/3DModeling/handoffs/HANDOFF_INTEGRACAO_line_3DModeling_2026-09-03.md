# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, 2026-09-03

> ⚠️ **Este handoff cobre TUDO desde o `merge-base`.** Os handoffs de
> [30/08](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-30.md) e
> [29/08](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-29.md) foram escritos e a linha **nunca
> integrou** — os commits deles estão dentro deste range. Leia-os para o **mecanismo** das W106–W107;
> este documento é o que o integrador precisa.

## 1 — Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| HEAD | `6353bf401` |
| merge-base com `main` | `066b4f92e` |
| commits | **55** |
| arquivos | **189** (`+26 633 / −2 617`) |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling` |

## 2 — Foundational / compartilhado tocado, e por quê

**26 arquivos fora das crates do módulo.** Todos **aditivos** salvo onde dito.

| arquivo | o quê | aditivo? |
|---|---|---|
| `shells/desktop/src/project_schema.rs` + `_tests.rs` | **`PROJECT_SCHEMA` 103 → 106** (três degraus) — ver §3 | ⚠️ **NÃO**: número que soma |
| `crates/ph2d-field/src/lib.rs` | `FIELD_DOC_VERSION` → **16** (sobe por arrasto com os degraus) | ⚠️ **NÃO** |
| `crates/ph2d-editor-core/src/ids/chrome/model3d.rs` | família de ids `model3d_select_button` (W112) | ✅ |
| `crates/ph2d-editor-core/src/screens/hero/state.rs` | `GizmoStateGroup::remove_from_selection` — **extraída** do `toggle_in_selection` | ✅ (o `toggle` passa a chamá-la) |
| `crates/ph2d-editor-core/src/interaction/state/store_core.rs` | — (W106, paleta) | ✅ |
| `crates/ph2d-i18n/src/model3d.rs` | 3 chaves novas (`panel.model3d.select.*`) | ✅ |
| `crates/ph2d-panel-model3d/*` | fileira `selects` + intent `SetLassoMode` + `CHIP_FAMILY_COUNT` | ✅ |
| `crates/ph2d-vector/*` | `StableImage` do atlas da `vello` 0.10 — **dep directa nova** `vello_encoding = "0.10.0"` | ⚠️ ver §5 |
| `shells/desktop/src/undo.rs` | lê `field3d_smoke::take_authored_change()` e guarda a seleção 3D | ✅ |
| `shells/desktop/src/undo_selection.rs` | **arquivo NOVO** — as 3 leis da seleção que sobrevive (split de LOC) | ✅ |
| `shells/desktop/src/input_dispatch/keyboard_hierarchy*.rs` | `Delete` e `Ctrl/Cmd+D` na Hierarquia | ✅ |
| `shells/desktop/src/main.rs`, `input_dispatch.rs` | roteamento das teclas novas | ✅ |
| `shells/desktop/tests/the_undo_preserves_the_vector_selection.rs` | o arch-gate ganha a metade 3D | ✅ |
| `project-memory/*` | uma memória nova + índice | ✅ |

## 3 — Símbolos que podem COLIDIR (saída do `collision-surface.sh`, não de memória)

```text
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 066b4f92e   ·   54 commit(s)   ·   189 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        106   (base: 103)
  ⚠   └ tripla do gate               (106, 13, 17)   (base: (103, 13, 17))
    VEC_SCENE_SCHEMA                       17   (base: 17)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)  79   (base: 79)     ph2d-script (espelho)  79   (base: 79)
▸ CONTRATO CONGELADO (§6) — intocado nos dois arquivos
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️ **A tabela acima é REFERÊNCIA, não evidência** — ela mede contra o `main` de 03/09. **RE-RODE
`collision-surface.sh` nesta worktree imediatamente antes de fundir** (DIRETRIZ §1.5.3).

### ⚠️ Os três degraus do `PROJECT_SCHEMA`, e o que fazer se outra linha também subiu

| degrau | o quê | commit |
|---|---|---|
| **104** | a JUNTA entre as cópias de uma repetição | `afc1a296b` |
| **105** | o CHANFRO em toda forma com aresta (`FIELD_DOC_VERSION` 14 → 15) | `4e4537599` |
| **106** | o EIXO de cada modificador com direcção (`FIELD_DOC_VERSION` → 16) | `fc034a25f` |

⛔ **O número certo se CONTA, nunca se escolhe.** Se outra linha subiu o schema no meio, os três
degraus **renumeram-se** e a **tripla** do gate (`project_schema_tests.rs`) tem de os seguir — são
**três** sítios (`project_schema.rs`, a escada ao lado, a tripla no irmão), e um degrau escrito no
arquivo errado **funde limpo e evapora**.

## 4 — Contratos congelados (§6)

**NENHUM encostado.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` intocados —
confirmado pela sonda.

## 5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

| item | estado |
|---|---|
| `cargo fmt --all -- --check` | ✅ limpo |
| `typos` | ✅ limpo — ⚠️ **estava vermelho e foi curado em `6353bf401`**: o `bounds_clip.rs` é arquivo NOVO desta linha, logo o `alo`/`ahi` nunca passara pelo ship |
| `clippy --all-targets` | ✅ **0** nas crates tocadas |
| **`machete` / `deny` / `audit`** | ⚠️ **UMA dep directa nova**: `vello_encoding = "0.10.0"` em `crates/ph2d-vector/Cargo.toml`. Ela **já estava no `Cargo.lock`** como transitiva da `vello` (por isso a sonda diz «nenhum `+name` novo`»), mas é a primeira vez que é **directa** — o `machete` e o `deny` só a vêem no ship |

## 6 — Ordem, dependências e o que smokar

**Ordem:** os 55 commits são lineares e dependentes; funda a linha inteira. ⛔ Não escolha commits.

### O que foi smokado e APROVADO pelo Enio

| wave | smoke | veredito |
|---|---|---|
| W104 filete honesto | `PH2D_FIELD_SMOKE=11` + paleta | «smoke ok» |
| W109 cabeçalho clicável | canvas de 4 vistas | «smoke ok» |
| W111 chanfro honesto | estrela com Chamfer/Fillet | «smoke OK» (+ pergunta sobre a complexidade, respondida no §112.6) |
| W112 laço que subtrai | 3 formas + `Shift`+arrasto | «smoke OK» |

### ⛔⛔⛔ O que NÃO foi smokado com sucesso — **o undo continua quebrado**

**Enio, 03/09, depois de três tentativas: *«não funciona. não melhorou em nada.»***

Ver [§7](#7--o-que-fica-aberto) — é o item nº 1 e o integrador tem de o carregar como **conhecido e
não resolvido**.

### Cenas de smoke desta linha

```bash
cargo run -p ph2d-host-desktop --release
```
Pill **MODEL** · `PH2D_FIELD_SMOKE=<n>` (roteador: `field3d_smoke_scenes.rs`) ·
diagnóstico `PH2D_UNDO_LOG=1` (novo — ver §7).

## 7 — O que fica ABERTO

### ⛔⛔⛔ 1. O UNDO/REDO PULA ETAPAS — três tentativas, NÃO curado

**O report vivo.** O que **já está eliminado por medição** (não repita):

| suspeita | veredito | onde |
|---|---|---|
| um passo por quadro no arrasto | curado e gateado desde a W6 (`gesture_in_progress`) | — |
| `pointer_up` não marcar o quadro | **`on_mouse_input` marca na PRIMEIRA linha** — press e release já contam | `input_dispatch.rs:3264` |
| o clique num chip do painel não marcar | mesma linha (o clique nasce no release) | idem |
| `FieldPose` fora da fotografia | **está registado** (`register_field_components`) | `ph2d-field-ecs/src/lib.rs:203` |
| a intenção do painel chegar um quadro atrasada | os gates de alcance provam a mesma chamada | `field3d_reach_tests.rs` |
| as três saídas de um gesto de gizmo | **eram** um defeito real (W113) — curado e gateado, **e não era o caso dele** | doc 06 §114 |
| a seleção 3D morrer em todo `Ctrl+Z` | **era** um defeito real (W113) — curado por `StableId`, **e não era o caso dele** | doc 06 §114 |
| a forma da paleta nascer sem passo | **era** um defeito real (W115) — curado, **e não era o caso dele** | doc 06 §116 |

⭐ **O instrumento existe e responde em uma corrida:**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_UNDO_LOG=1 cargo run -p ph2d-host-desktop --release
```
Toda supressão **sobre um documento que mudou** imprime o motivo, de cinco possíveis. ⚠️ **As linhas
repetidas são a MESMA mudança pendente** — contar linhas conta quadros, não defeitos.

⚠️ **A LEI a ter na cabeça:** uma mudança suprimida **não se perde — funde-se no próximo passo**,
porque o `undo_baseline` só é substituído quando um passo é registado. *Um passo suprimido e um passo
ausente leem-se iguais de fora, e as causas são opostas.*

⛔ **E o relatório da captura incremental MENTE** (evidência no log dele, doc 06 §116.4):
`nascidas=0` com `linhas` a crescer, e `delta=0 B` num passo com `world=true`. Foi por acreditar nele
que a leitura demorou — **meça `undo_capture_cache::last_report()` contra o diff verdadeiro antes de
o citar**.

### ⏳ 2. Os outros abertos (nenhum é regressão desta linha)

| item | estado |
|---|---|
| o teto de `round` da **estrela** é `12,3 %` do bordo contra `43–60 %` das outras | doc 06 §104.1 |
| o quadro de MOVIMENTO custa `26,7 ms` contra `16,7` — a marcha é 80 % dele | doc 06 §13.0; sobre-relaxação e montagem **recusadas por medição** |
| pela mistura **n-ária** o chanfro ainda desce `c/sin 2α` (`1,15×` no prisma, `1,06×` no octaedro) | bloqueio nomeado: matriz de Gram sem forma fechada em `N ≥ 3` — doc 06 §112.4 |
| o vértice do **vale** da estrela mede `63,7°` e o modelo de duas facetas prevê `47,7°` | modelo incompleto; ⛔ **não** está provado que seja defeito — doc 06 §112.6 |
| o panic do `ph2d-gridmap` | **dono: `line/quadextract`** — inalterado |

## 8 — A UMA LINHA para o `CLAUDE.md §5` (o integrador escreve; não acrescente parágrafo)

> ⛔⛔⛔ **O UNDO/REDO DESTE MÓDULO PULA ETAPAS e NÃO está curado** (Enio, 03/09, após três
> tentativas): ⭐ o diagnóstico é `PH2D_UNDO_LOG=1`, que **nomeia** qual das cinco guardas do
> `post_frame_undo` comeu o passo, e ⚠️ **uma mudança suprimida não se perde — funde-se no passo
> seguinte**, então *«pulou»* e *«não registou»* leem-se iguais · ⛔ **oito suspeitas já estão
> eliminadas por medição** (handoff de 03/09 §7.1) e três eram defeitos REAIS já curados (as três
> saídas de um gesto · a seleção 3D morta em todo `Ctrl+Z` · a forma da paleta sem passo) — **nenhuma
> era o caso dele** · ⛔ e o relatório do `undo_capture_cache` **mente** (`nascidas=0` com `linhas` a
> crescer). Mais: ⭐ o **chanfro** recua o que o slider diz em qualquer quina (doc 06 §112) · o
> **laço** subtrai (§113) · a paleta tem **33** formas (§106).

## 9 — Itens 7 e 9 do protocolo

- `rm -rf target/*/incremental` — **feito** (ver o relatório na resposta de fecho).
- **Binário de release quente** — construído com a linha de comando exacta que este handoff entrega;
  a 2ª corrida está colada na resposta de fecho.
