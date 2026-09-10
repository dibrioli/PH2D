# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, 2026-09-10

> **DIRETRIZ §1.5.9.** A linha fecha aqui e **PARA**. Não integra, não pusha, não faz ship —
> isso é ordem explícita do Enio, por um agente integrador dedicado (§0.7).

## 1. Identidade

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling` |
| ramo | `line/3DModeling` |
| commits à frente do `main` | **23** (`517a47b4b` … `9f9b561f8`) |
| base | `39d48cd76` — **igual ao `main` de hoje**, logo um `--ff-only` aplica-se sem rebase |
| handoff anterior | [2026-09-07](HANDOFF_INTEGRACAO_line_3DModeling_2026-09-07.md) — ⚠️ **as W136–W146 nunca tiveram handoff**; este cobre-as |

## 2. ⚠️ OS NÚMEROS QUE SE CONTAM — não os copie, re-conte contra o `main` do dia

⛔ **Conte o DELTA, nunca o literal.** Duas linhas que escrevam o mesmo número fundem **mudas** — o
git não sabe o que o número significa (§5.0).

| contador | `main` | esta linha | delta | onde se conta |
|---|---:|---:|---:|---|
| `FIELD_DOC_VERSION` | `18` | `22` | **`+4`** | [`ph2d-field/src/lib.rs`](../../../crates/ph2d-field/src/lib.rs) |
| `PrimitiveKind::ALL` | `58` | `62` | **`+4`** | [`primitive_kind.rs`](../../../crates/ph2d-field/src/primitive_kind.rs) |
| entradas do catálogo | `68` | `73` | **`+5`** | `grep -c 'key: "panel.model3d.add'` |
| `CENAS` do smoke | `29` | `32` | **`+3`** | [`field3d_smoke_scene_tests.rs`](../../../shells/desktop/src/field3d_smoke_scene_tests.rs) |
| `PROJECT_SCHEMA` | `123` | `123` | **`0`** | ⭐ esta linha **não lhe toca** |

⚠️⚠️ **O `collision-surface.sh` NÃO vê o `FIELD_DOC_VERSION`** — ele conta `PROJECT_SCHEMA`,
`VEC_SCENE_SCHEMA`, `FLIP_SCHEMA` e o `DOC_VERSION` da timeline, e este fica de fora. Duas linhas que
o subam em paralelo fundem **sem conflito e erradas**.

⚠️ **O catálogo mudou de FICHEIRO:** no `main` ele vive em `field3d_shapes.rs`; aqui saiu para
[`field3d_shapes_table.rs`](../../../shells/desktop/src/field3d_shapes_table.rs) (corte por tecto de
LOC). Uma entrada acrescentada no ficheiro antigo por outra linha funde **limpo e evapora**.

## 3. ⛔⛔ FOUNDATIONAL / PARTILHADO TOCADO — leia isto antes de fundir

**O `Cargo.toml` da RAIZ.** Quatro entradas no fim do bloco `[profile.dev.package.*]`:

```toml
[profile.dev.package.ph2d-field-eval]   opt-level = 2
[profile.dev.package.ph2d-field]        opt-level = 2
[profile.dev.package.fidget]            opt-level = 2
[profile.dev.package.fidget-core]       opt-level = 2
```

⭐⭐ **E esta configuração é o ÓPTIMO MEDIDO, não a primeira que funcionou** (a pergunta do dono foi
*«qual o melhor possível?»*). Duas afinações acima dela foram medidas e **recusadas** — detalhe e
tabelas no [doc 11 §11.9](../11_a_avaliacao_ponto_a_ponto.md):

| tentativa | veredito |
|---|---|
| `opt-level = 3` em vez de `2` | ⛔ **sem diferença real** — o nível `2` REPETIDO no fim saiu melhor que o `3`; a diferença era a carga a cair |
| juntar `ph2d-field-render` · `-mesh` · `-profile` | ⛔ **parte um teste** (`quadro MORNO … pagou 4`), sem mecanismo que o explique |
| tirar a `fidget` (só as 2 crates próprias) | ⛔ custa **`21 %`** (`4,46` → `5,41 s`), medido com 3 corridas de cada lado |

É **apêndice** (nenhuma linha existente mexida), mas é um ficheiro que toda linha toca. ⭐ E o efeito
não é local: a `fidget` é dependência partilhada, então **toda crate que a use passa a correr os
testes optimizados**. Medido nesta linha: a suíte das 3 crates do campo vai de `372,3 s` para
`57,1 s`. ⚠️ O preço é compilar essas quatro uma vez a `opt-2`.

## 3-bis. ⭐ A SUPERFÍCIE DE COLISÃO, MEDIDA contra as outras SEIS linhas vivas

Medido em 2026-09-10 (`git worktree list` + diff de cada linha **a partir da base DELA**). Esta linha
toca `104` ficheiros, e o que ela partilha com cada uma das outras é:

| linha | ficheiros dela | partilha comigo | o quê |
|---|---:|---:|---|
| `line/components` | 227 | **1** | `shells/desktop/src/main.rs` |
| `line/motion-value` | 167 | **2** | `main.rs` · `project-memory/MEMORY.md` |
| `line/quadextract` | 46 | **1** | `main.rs` (⚠️ **ATRASADA** — ver abaixo) |
| `line/sculpt3d` | 276 | **1** | `main.rs` |
| `line/UIUX` | 138 | **1** | `project-memory/MEMORY.md` |
| `line/Vector` | 122 | **1** | `main.rs` |

⭐ **Os dois conflitos possíveis são triviais e conhecidos:**
- **`main.rs`** — a minha edição é **UMA linha de comentário** (`PH2D_FIELD_SMOKE=1..29` → `1..32`).
- **`MEMORY.md`** — **duas linhas** acrescentadas ao índice. Resolve-se ficando com as de todos.

⛔ **E o `Cargo.toml` da RAIZ, que a §3 declara como o meu risco, NÃO é tocado por mais nenhuma
linha** — medido, `0` em seis. O risco declarado é real como categoria e **nulo nesta rodada**.

### ⚠️⚠️ A régua que quase deu um alarme falso de `50` ficheiros

A 1.ª medição usou `git diff main..HEAD --name-only` em cada worktree e acusou a `line/quadextract`
de partilhar **`50`** ficheiros comigo, **21 deles em `crates/ph2d-field/src`** — o meu módulo.

**Era artefacto.** A `quadextract` bifurcou de `53832c884`, que é **anterior** ao `main` de hoje: num
ramo atrasado, `diff main..HEAD` mostra **ao contrário** tudo o que o `main` ganhou entretanto. O
teste que o desmente é directo — `git log main..HEAD -- crates/ph2d-field/src` naquela worktree
devolve **zero commits**.

⇒ **a diferença de uma linha mede-se a partir da BASE DELA** (`merge-base`), nunca contra o `main`;
e uma linha atrasada precisa de **rebase antes de entrar**, o que é assunto dela e não meu.

## 4. Contratos congelados encostados

**Nenhum.** O `Tool=12`, o `NodeOp=2` e a superfície do vector-doc (§6) ficam intocados. O
`FIELD_DOC_VERSION` **não é contrato congelado** — é o formato do documento de campo, e sobe por
apêndice de variantes (o postcard é posicional: acrescentar no FIM não move índice nenhum).

## 5. Gate batched do fecho — o que correu, e com que carga

| | |
|---|---|
| `cargo fmt --check` | **0** |
| `cargo clippy --all-targets` (crates editadas) | **0** |
| suíte das 3 crates do campo | **407 / 407**, `57,1 s` |
| `field3d` do shell | **359 / 359**, `4,1 s` |
| workspace (`nextest --workspace --no-fail-fast`) | **22 234 / 22 235**, `1 225 s` — a única ✗ era o tecto de LOC, **curada** |
| re-corrida do raio do corte (shell + gates de arquitectura) | **7 053 / 7 053**, `75 s` |
| `scripts/doc-index.sh --check` | ✓ 18 índices em dia |

⚠️ **A carga foi impressa ao lado de cada medição de relógio** (§5.0). As leituras de custo por ponto
correram a `load 2,92` (calma) e a `load 11,35` (mínimo-de-9); as duas concordam a **~5 %**, e é esse
o controlo da técnica.

### 5-bis. ⛔⛔ O que só a corrida da WORKSPACE apanhou

**Três ficheiros acima do tecto de `700` LOC**, todos crescidos **nesta linha** (no `main` estão os
três abaixo), nas W145/W146:

| ficheiro | antes | depois | corte |
|---|---:|---:|---|
| `ph2d-field-eval/src/hybrid.rs` | `730` | `634` | a **lei numérica** saiu para `hybrid_law.rs` |
| `ph2d-field-ecs/src/verb_tests.rs` | `716` | `208` | os testes de **junção** saíram para `verb_joint_tests.rs` |
| `ph2d-field-ecs/src/edit_params.rs` | `716` | `492` | **escrever** saiu para `edit_params_write.rs` |

⚠️⚠️ **Eles passaram por TRÊS fechos desta linha** porque um fecho que só corre as crates **editadas**
é cego aos gates de arquitectura — eles vivem no `ph2d-editor-core`. *É a quinta ocorrência registada
desta família no repo.*

⚠️ E **`*_tests.rs` NÃO é excluído do tecto**: o gate isenta `**/tests/**` e `**/tests.rs`, e mais
nada.

⭐ **A cura foi SPLIT, nunca allowlist** — e num dos três a medição escolheu o corte: as chaves i18n
ficaram com quem **LÊ**, porque o escritor não lhes toca (casa no enum `Param`, não na string).

## 6. ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **A fita `f64` nova (`point_tape.rs`) NÃO é uma aproximação.** Ela é **bit-a-bit** o que o
   `Context::eval_xyz` devolvia, por construção — mesmo grafo, mesmas funções da `fidget`, só a
   **ordem de visita** diferente. Há gate sobre o valor **e** sobre o gradiente.
2. **Ela NÃO substitui o caminho do traçado.** Quem desenha continua a ser o JIT `f32` em lote do
   `hybrid`. Esta serve quem pergunta **um ponto de cada vez** (as sondas, os gates, o *pick*).
3. **O `at_many` não é «mais rápido em geral»** — ele é `2,6×` numa thread e deu **`5 %`** no censo
   a `opt-0`. Só depois do `opt-2` é que passou a valer `14 %`. *A mesma cura mede-se cinco vezes
   menor no perfil errado.*
4. **O ganho de `372 s → 57 s` NÃO é do algoritmo** — é das quatro linhas de `Cargo.toml`. Quem ler
   o diff de código e atribuir o número a ele vai tirar a conclusão errada sobre onde o tecto está.
5. **A `sd_helix` já NÃO engorda o tubo `1/c`** — a W140 curou-a, com tabela no código. A nota do
   `CLAUDE.md §5` que a dá como aberta está **obsoleta** (ver §9).
6. **O doc 06 não perdeu uma linha.** Ele é um roteador de `91 KB` e a história está **verbatim** em
   `docs/archive/3dmodeling-06-2026-09-10/`, com `sha256` a prová-lo e as 144 secções **indexadas**
   por § no topo do vivo.
7. **O teto de `round` da estrela (`12,3 %`) não é um defeito por corrigir** — é uma lei analítica
   exacta (`star_round_limit`): num vale de estrela a faixa de mistura é estreita por geometria.

## 7. As premissas que a MEDIÇÃO derrubou nesta linha

1. ⛔ *«a cura publicada é o gradiente ANALÍTICO da `fidget`»* — **RECUSADA com número**: sobre pontos
   postos numa aresta ela discorda `1,876e-1` da diferença central, contra uma folga de módulo de
   `2,0e-2` (**`9,4×`**). Não é precisão: num vinco a derivada não existe e as duas são **grandezas
   diferentes**.
2. ⛔ *«o custo por amostra é a alocação `O(contexto)` da `fidget`»* — **REFUTADA**: o caminho novo
   não aloca e continua a crescer com o documento. O que se paga é **percorrer** a fita.
3. ⛔ *«uma vizinha 32-way paralela está a esfomear o teste»* — **REFUTADA**: sozinho, a razão é a
   mesma (`5,4 %` contra `4,8 %`).
4. ⛔ *«os 14 ciclos em série do censo são a alavanca»* — **REFUTADA**: o corredor de testes já satura
   os 32 núcleos com outros testes, então paralelizar por dentro **redistribui** trabalho, não o
   reduz.
5. ⛔ **E a minha 1.ª leitura do item 1 dizia o CONTRÁRIO** — amostrada por grelha, a população do
   vinco lia `2,174e-6` e o veredito escrito era *«o bloqueio dissolve-se»*. Um vinco é uma
   **superfície** (medida nula): a grelha reportou-o **~60 000× mais limpo** do que é.

## 8. ⏳ O que fica ABERTO (auditado contra o CÓDIGO em 2026-09-10)

A lista viva é o **§13.0** do [doc 06](../06_resultados_cena_e_gizmo.md). Auditada hoje: de 48
entradas, **32 fechadas**, **6 recusadas com mecanismo**, **4 decisões do dono**, **5 a sério
abertas** — e nenhuma das cinco é resolúvel sem ser wave nova ou veredito dele:

| item | o que falta | § |
|---|---|---|
| a MARCHA (`26,7 ms` contra `16,7`) | as rotas óbvias estão **medidas e recusadas** | §82 |
| a cache contra o **casco** e não a caixa | `1,11×` na mesa; pede teste em dois níveis | §83.9 |
| mistura **N-ÁRIA** (`c/sin 2α`) | **bloqueio nomeado**: pede a matriz de `intersection_round_n` | §102 |
| teto de `round` da estrela (`12,3 %`) | ⭐ **não é defeito** — lei exacta; é pergunta de produto | §104.1 |
| `SLABS` | decidido: fica em `4` | §82 |

⏳ **Fora do §13.0:** o `docs/Render3d/` tem **8 waves e nada construído** — é módulo novo, e a
recomendação desta linha é **linha própria** (*uma feature = UMA linha*).

## 9. ⚠️ DUAS notas do `CLAUDE.md §5` que esta linha encontrou OBSOLETAS

Elas mandam reconstruir trabalho já pago, que é o defeito que o §5 avisa contra sobre si mesmo:

1. *«a `sd_helix` engorda o tubo `1/c` (a ferramenta da cura já está escrita duas vezes)»* — **a W140
   curou-o**, e a tabela medida está no doc-comment de `ops_spiral.rs`.
2. *«o [doc 06] está em ~850 KB … e o `doc-split.py` é devido»* — **feito**, `901 KB → 91 KB`.

E a nota de `tests/common/mod.rs` que dizia *«a causa de FUNDO continua por curar»* foi corrigida no
mesmo commit em que deixou de ser verdade — ela **prescrevia** a cura que a §7.1 recusa.

## 10. O que SMOKAR depois de integrar

⛔ **Esta jornada não tem smoke visível, de propósito** — tudo o que ela mudou é bit-a-bit idêntico
na imagem. O que se confere é que **nada regrediu**:

1. `cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release`
2. Abrir o pill **MODEL**, criar uma forma pela paleta (`A` ou *+ Add shape…*), pôr-lhe filete.
3. A peça tem de sair **igual à de antes**, e o modelador tem de continuar a responder à mão.
4. Se algo mudar de FORMA, é regressão — os gates de identidade dizem que não pode.

As waves com smoke próprio (W136–W146) estão nas cenas `PH2D_FIELD_SMOKE=1..32`.

## 11. Higiene do fecho

- [x] handoff escrito (este ficheiro)
- [x] `doc-index.sh --check` verde
- [x] `rm -rf target/*/incremental`
- [ ] **UMA LINHA** no `CLAUDE.md §5` — ⚠️ escrita **na integração**, não aqui

## 12. A UMA LINHA para o `CLAUDE.md §5`

> ⭐⭐⭐ **E a AVALIAÇÃO PONTO A PONTO deixou de ser o tecto (W147, 10/09):** o `Field::at` passou de
> interpretador da `fidget` a **fita `f64` achatada** com o gradiente a mandar as seis amostras numa
> passagem só — **bit-a-bit a mesma resposta** (gate sobre o valor e sobre o gradiente), `~3,6×` no
> valor e `~6,3×` no gradiente. ⛔⛔ **E a cura que o §5 prescrevia — o gradiente ANALÍTICO da
> `fidget` — está RECUSADA com número:** sobre pontos POSTOS numa aresta ela discorda `1,876e-1`
> contra uma folga de `2,0e-2` (**`9,4×`**), porque num vinco a derivada não existe e a diferença
> central e a analítica são **grandezas diferentes** — passar a `f64` não cura. ⚠️⚠️ **A 1.ª medição
> disse o CONTRÁRIO** (`2,174e-6`): um vinco é uma **superfície**, e uma grelha nunca lá cai — *os
> pontos do vinco PÕEM-SE, não se procuram*. ⭐⭐⭐ **E o tecto de verdade não era o algoritmo, era o
> PERFIL DE BUILD:** as crates do campo nunca tinham entrado na lista `[profile.dev.package.*]`
> `opt-level = 2` do `Cargo.toml` da raiz — a mesma lista, com a mesma justificação escrita, que já
> existia para o Painter e o áudio. Quatro linhas: a suíte das 3 crates do campo **`372,3 s →
> 57,1 s`** (`6,5×`), o teste mais longo `307 → 43 s`, o `field3d` do shell `22,8 → 4,1 s`. ⚠️ E a
> varredura em lote que eu quase deitei fora por dar `5 %` passou a valer `14 %` medida no perfil
> certo — *a mesma cura mede-se cinco vezes menor no perfil errado*. ⛔ **Os «14 ciclos em série» do
> censo são RECUSA MEDIDA**: o corredor já satura os núcleos, logo paralelizar por dentro
> redistribui e não reduz. ⚠️ **O `Cargo.toml` da RAIZ é tocado** (4 entradas de apêndice, e a
> `fidget` é partilhada). ⭐⭐ **E o [doc 06](docs/3DModeling/06_resultados_cena_e_gizmo.md) virou
> ROTEADOR** — `901 KB → 91 KB`, história **verbatim** em
> [`docs/archive/3dmodeling-06-2026-09-10/`](docs/archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md)
> com `sha256` e as 144 secções indexadas por §. ⚠️ **DUAS notas deste §5 estavam obsoletas** (a
> `sd_helix` do `1/c`, curada na W140; e o corte do doc 06) — *audite a lista contra o CÓDIGO antes
> de pegar um item dela*.
