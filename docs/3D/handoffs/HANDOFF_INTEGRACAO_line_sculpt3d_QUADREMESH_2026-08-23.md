# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, jornada do quad remesh (2026-08-23)

> ⚠️ **A LINHA NÃO ESTÁ FECHADA.** Este handoff é um **retrato** pedido pelo Enio a meio
> da jornada; o trabalho continua e o `HEAD` vai andar. ⇒ **o integrador tem de re-rodar
> `collision-surface.sh` e reconferir o `HEAD` antes de fundir** (DIRETRIZ §1.5.9 item 3,
> «a tabela colada é REFERÊNCIA, nunca EVIDÊNCIA»). O item 7 (reclamar o `incremental/`)
> **está por fazer de propósito** — ele é o passo de FECHO, e desligá-lo agora tirava o
> `cargo check -p` de baixo dos pés do resto da jornada.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/sculpt3d` |
| `HEAD` | `de0243d3417a663bcd017b0cdf3504dd6c119a1f` |
| merge-base com `main` | `35f937cb2a42b28aeeaf685afb5ad185df28fd18` |
| commits | **36** |
| ficheiros | **58** (`+12 507 / −859`) |

---

## 2. Foundational / partilhado tocado, e porquê

### ⭐ Crate NOVA (drop-crate, `CLAUDE.md` §0.1) — risco de colisão **nulo**

`crates/ph2d-gridmap/` — **o mapa de grade inteira** (a 6.ª crate da cadeia do quad
remesh). Clean-room de Bommes 2009 (MIQ) + QuadCover 2007; ⛔ nenhuma linha de CoMISo,
libQEx, vcglib ou do traçador do quadwild (GPL). Membro por **glob** (`crates/*`) ⇒
**zero edições no `Cargo.toml` da workspace**.

### ⚠️ Crates da cadeia, com MUDANÇA DE API PÚBLICA

| ficheiro | o que mudou | risco |
|---|---|---|
| `crates/ph2d-crossfield/src/lib.rs` | ⭐ **`pub mod comb`** + `pub use comb::{Holonomy, holonomy}` (novos) e `CrossField::from_directions` (novo) | **aditivo** |
| `crates/ph2d-crossfield/src/comb.rs` | ⛔ **`Holonomy` mudou de campos:** `p50/p95/max` → `rough_p50/rough_p95/rough_max`, mais `cycles`/`defects`/`turn_max` | ⚠️ **quebra quem os lesse** |
| `crates/ph2d-quadfill/src/report.rs` | ⛔ **`FillReport::holonomy` → `rough`**, mais `dirty_patches`/`combed_patches` e outras colunas novas | ⚠️ **quebra quem os lesse** |
| `crates/ph2d-trace/src/{lib,patches}.rs` | `PatchLayout::face_dir` (o campo viaja com o layout) + `prune.rs` (novo, **desligado**) | aditivo |

⭐ **Consumidores conferidos por `grep`: o único que lia estes campos é
`shells/desktop/src/sculpt3d_quad_shape.rs`, e está actualizado.** Nenhuma outra linha
usa `ph2d-crossfield` ou `ph2d-quadfill` — eles são F2/F5 da cadeia do quad remesh.

### ⚠️ Ferramenta PARTILHADA — o único ponto que outra linha pode encostar

`scripts/nextest-impacted.sh` — **duas redes obrigatórias novas**, e as duas nasceram de
defeito medido nesta jornada:

1. ⛔ **uma árvore suja fazia este gate sair VERDE sem medir nada** (`4 testes` sobre um
   diff de 13 ficheiros; depois de commitar, `3 842`). Agora ele **avisa** quando
   `git status --porcelain` não está vazio.
2. ⭐ força `binary(architecture_workspace_file_loc_cap)` e `binary(file_loc_caps)`, que
   varrem a **workspace inteira** e não eram seleccionados pelo impacto.

⭐ *É uma melhoria do gate de toda a gente e é aditiva — mas se outra linha lhe tiver
mexido, este é o ficheiro a olhar primeiro.*

### Shell (`shells/desktop/src/`) — **só sondas de `sculpt3d`**

`sculpt3d_field_follow.rs` · `sculpt3d_holonomy_probe.rs` (novo) ·
`sculpt3d_patch_valence.rs` (novo) · `sculpt3d_quad_shape.rs` ·
`sculpt3d_simplest_case.rs` (novo) · `sculpt3d_undo.rs` (**só** as 3 linhas de
`#[path] mod` que declaram as sondas novas).

⚠️ **`sculpt3d_undo.rs` é o único ponto do shell que outra linha do sculpt3d encostaria**
— e a mudança é um bloco de declarações de módulo no fim, **append-only**.

### Documentação e configuração

| ficheiro | o quê |
|---|---|
| `CLAUDE.md` | ⚠️ **`+218` linhas no §5**, todas no bullet do 3D/Sculpt (ver §7 abaixo) |
| `docs/3D/quad-remesh/PLAN.md` | `+1 650` — o plano vivo da linha |
| `project-memory/` | 9 memórias novas + 7 linhas no `MEMORY.md` |
| `.typos.toml` | ⭐ `decies` e `comercial` — **latentes PRÉ-EXISTENTES** do `PLAN.md` que o `ship.sh` pagaria |
| `Cargo.lock` | `+13` — só a aresta interna da `ph2d-gridmap`, **nenhum pacote externo novo** |

---

## 3. Superfície de colisão

⚠️ **Colada de `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`,
2026-08-23. Referência, não evidência — RE-RODE antes de fundir.**

```
SUPERFÍCIE DE COLISÃO — line/sculpt3d contra main
  merge-base 35f937cb2   ·   36 commit(s)   ·   58 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                         89   (base: 89)
      └ tripla do gate               (89, 13, 14)   (base: (89, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               65   (base: 65)
    ph2d-render (espelho)                  66   (base: 66)
    ph2d-script (espelho)                  66   (base: 66)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0162   próximo livre: 0163
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-gridmap"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3)
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
```

⭐ **Nenhum schema movido · nenhum registo movido · nenhum ADR criado · nenhum pacote
externo novo.** O `ph2d-gridmap` do `Cargo.lock` é aresta **interna** (path dep).

### ⚠️ Símbolos novos que outra linha poderia encostar

⭐ **Todos vivem em crates da cadeia do quad remesh ou na crate nova** — nenhum é id,
token, variante de enum de contrato, ou entrada em lista ordenada partilhada.

| símbolo | onde |
|---|---|
| `Holonomy::{cycles, defects, turn_max, rough_*}` | `ph2d-crossfield` |
| `CrossField::from_directions` | `ph2d-crossfield` |
| `PatchLayout::face_dir` | `ph2d-trace` |
| `ph2d_trace::prune::*` (**desligado**) | `ph2d-trace` |
| `FillReport::{rough, dirty_patches, combed_patches, slid, slid_refused, conformal, regraduated, domain_cells, quad_patches, shape, skew_prov, skew_by_fan}` | `ph2d-quadfill` |
| `ph2d_quadfill::{Interior, SQUARE_ROUNDS, Provenance, QuadShape, detail_lost}` | `ph2d-quadfill` |
| `ph2d_gridmap::*` | crate NOVA |

---

## 4. Contratos congelados encostados

⭐ **NENHUM.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` intocados
(confirmado pelo `collision-surface.sh`). Nenhum ADR criado ⇒ fora da disputa de número.

---

## 5. O que só o `ship.sh` apanha

| | estado |
|---|---|
| **fmt** | ⭐ `cargo fmt` corrido em todas as crates tocadas |
| **typos** | ⭐ `0` nos ficheiros da linha. ⚠️ **Drenei 2 latentes PRÉ-EXISTENTES** (`decies`, `comercial` no `PLAN.md`) para o `.typos.toml` — *se outra linha lhes tiver mexido, é conflito de 2 linhas* |
| **machete** | ⭐ limpo na `ph2d-gridmap`. ⚠️ Ela tem **4 dev-dependencies internas** (`remesh-iso`, `quadfill`, `quantize`, `quadflow`) — as três últimas **só para a sonda do produto**, e são para **jusante** (não há ciclo) |
| **deps novas** | ⭐ **nenhuma externa** |
| **clippy** | ⭐ `--all-targets` limpo nas 4 crates tocadas + shell. ⛔ **NÃO corri `--workspace`** |
| **RUSTSEC / deny** | ⛔ **não corridos** — sem deps externas novas, o risco é o drift do `advisory-db` |
| **matriz 3-OS** | ⛔ não corrida. ⚠️ O `ph2d-gridmap` é `f32` puro sem `unsafe` e sem plataforma |

---

## 6. Ordem, dependências, e o que smokar

### Ordem

⭐ **Nenhuma dependência entre commits fora da ordem cronológica.** Os 36 são
sequenciais numa só linha; um `--ff-only` resolve.

### ⭐⭐⭐ O que smokar: **NADA**

⚠️ **Esta jornada não muda uma linha do comportamento do produto.** Tudo o que foi
construído está **desligado com a tabela da rejeição ao lado**:

| interruptor | estado | onde |
|---|---|---|
| `aligned::INTERIOR` | `FromBoundary` (o antigo) | `ph2d-quadfill` |
| `lscm::LSCM_MAP` | `false` | `ph2d-quadfill` |
| `regraduate::REGRADUATE` | `false` | `ph2d-quadfill` |
| `rectangle::RECTANGLE_MAP` | `false` | `ph2d-quadfill` |
| `PROPORTIONAL_DOMAIN` | `false` | `ph2d-quadfill` |
| `SQUARE_ROUNDS` | `0` | `ph2d-quadfill` |
| `prune::PRUNE_STEMS` | `false` | `ph2d-trace` |
| `ph2d-gridmap` inteira | **nenhum consumidor no produto** | crate nova |

⇒ **a saída do botão `Quad Retopology` é byte-idêntica à do `main`.** *O valor da jornada
são as RÉGUAS, as RECUSAS medidas e as três dívidas do F3 que ficaram visíveis.*

### ⚠️ Vermelhos e flakes que o integrador vai ver

| gate | estado |
|---|---|
| `the_quads_are_as_square_as_the_oracles` | ⛔ **vermelho, `#[ignore]`, PRÉ-EXISTENTE** |
| `a_plain_sphere_is_as_square_as_the_oracles` | ⛔ vermelho, `#[ignore]`, pré-existente |
| `the_tracer_survives_the_aligned_field` | ⛔ vermelho, `#[ignore]`, pré-existente |
| `the_ear_does_not_ship_an_edge_across_the_piece` | ⛔ vermelho, `#[ignore]`, pré-existente |
| flakes de relógio (**5** conhecidas) | `CLAUDE.md` §5.0 — **re-rode sozinhas antes de suspeitar** |

### ⭐ O portão batched CORREU, e o resultado é `3 886 / 3 887`

```
Summary [62.627s] 3887 tests run: 3886 passed (1 slow), 1 failed, 279 skipped
```

⚠️ **A única falha é a QUINTA flake de relógio, e foi registada no §5.0 por esta linha.**
A família `flip_smooth::resample_measurement::precisao::orcamento`
(`shells/desktop/src/flip_fit_budget_tests.rs`) mede razões de tempos **sub-milissegundo**
(`1,36 → 8,77 ms`), e sob o fan-out de `3 887` testes:

| corrida | qual falhou |
|---|---|
| 1.ª | `a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` |
| 2.ª | `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` |
| sozinhos | ⭐ **os três passam, 3 de 3** |

⭐⭐ **A falha MUDA de teste entre corridas** — *uma regressão não muda de sítio; uma
leitura de relógio muda.* E ⛔ **esta linha não toca UM ficheiro de Flip**
(`git diff --name-only main...HEAD | grep -c flip` ⇒ `0`).

⚠️ **`--no-fail-fast` é obrigatório aqui:** sem ele o `nextest` cancelou em `1300/3887` e
a corrida nunca chegou a medir os outros `2 587`.

---

## 7. ⚠️ `CLAUDE.md §5` — `+218` linhas, e isso PRECISA de decisão

⛔ **O §5 diz «uma linha por wave» e esta jornada acrescentou-lhe `218`.** A regra não foi
cumprida, e digo-o em vez de o esconder.

⚠️ **Duas coisas diferentes estão lá dentro:**

1. ⭐ **Correcções de FALSIDADES que o §5 afirmava** — e essas **têm** de ficar, porque o
   §5 é injectado em todo agente antes da primeira palavra: a acusação da holonomia
   (a régua não podia dar aquela resposta), «singularidade SEM CANTO = DENTRO de um
   patch» (inferência refutada), e os números da régua partida.
2. ⛔ **Narrativa da jornada**, que devia estar **aqui** e não lá.

⇒ ⭐ **Recomendação ao integrador:** cortar o bullet do 3D/Sculpt para o **estado + o
aberto + os ponteiros**, e deixar o mecanismo neste handoff e no `PLAN.md`
(§4-quinquagies .. §4-septemetquinquagies). *Não o faço eu porque cortar o §5 durante a
integração é decisão do integrador, que vê as outras linhas.*

---

## 8. O estado técnico, em sete linhas

1. ⭐ **A caça ao enviesamento fechou por eliminação MEDIDA:** campo ilibado · quatro
   achatamentos · forma do domínio · menos patches · subdivisão local · ponto fixo ·
   **e a parametrização global, construída e medida**.
2. ⛔ **A parametrização global entrega a promessa e move `1°`** (`18° → 17°`; o oráculo
   faz `6°`). **A marcação do arco nunca foi o constrangimento.**
3. ⛔ **Curar o leque também não chega:** as faces de patches de quatro lados — sem leque
   nenhum — já medem `15,2°`.
4. ⇒ ⭐⭐⭐ **A distorção nasce entre o DOMÍNIO e a SUPERFÍCIE.** Mesmo com F3, marcação
   e domínio perfeitos, **o preenchimento por patch fica em `15°`**.
5. ⭐ **A obra seguinte é a EXTRACÇÃO** (QEx, Ebke 2013 — o *paper*; a `libQEx` é GPL):
   pôr os pontos da grade nas isolinhas inteiras do mapa global.
6. ⚠️ **O que lhe falta está nomeado e medido:** as translações de ciclo têm de ser
   inteiras e estão a `0,291` de distância ⇒ pede **arredondamento uma-a-uma com
   re-solve** (o *mixed-integer* do nome), não arredondamento em lote.
7. ⭐ **Três dívidas do F3 ficaram visíveis por réguas novas:** `2` patches com
   singularidade dentro no gancho (um com `10` voltas) · um patch-anel no toro sem ponte
   · `3` patches sujos no toro.
