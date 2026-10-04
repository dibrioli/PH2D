# HANDOFF DE INTEGRAÇÃO — `line/components`: AS PAREDES DO DESVIO POR MOSAICOS (navegação W11) — 2026-10-04

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §19. Este documento é o que o
> integrador precisa.
>
> ⚠️⚠️ **A linha leva SETE waves por integrar**, juntas e sem reverter nada umas das outras: W5
> ([`DESVIO_2026-10-02`](HANDOFF_INTEGRACAO_line_components_DESVIO_2026-10-02.md)), W6
> ([`O_MUNDO_QUE_MUDA_2026-10-02`](HANDOFF_INTEGRACAO_line_components_O_MUNDO_QUE_MUDA_2026-10-02.md)), W7
> ([`O_CUSTO_E_OS_ATALHOS_2026-10-03`](HANDOFF_INTEGRACAO_line_components_O_CUSTO_E_OS_ATALHOS_2026-10-03.md) — a
> superfície foundational e o §3 «o que um merge pode partir»), W8
> ([`A_ARENA_E_O_TUTORIAL_2026-10-03`](HANDOFF_INTEGRACAO_line_components_A_ARENA_E_O_TUTORIAL_2026-10-03.md)), W9
> ([`O_CUSTO_A_ESCALA_2026-10-03`](HANDOFF_INTEGRACAO_line_components_O_CUSTO_A_ESCALA_2026-10-03.md)), W10
> ([`A_MONTAGEM_POR_BLOCOS_2026-10-03`](HANDOFF_INTEGRACAO_line_components_A_MONTAGEM_POR_BLOCOS_2026-10-03.md)) e
> esta W11. Smoke do dono APROVADO na W5–W10; **o da W11 está por fazer** (§5).

## §0 — O `--ff-only` deve passar limpo

- base `1ad60a1ce` (o `main`, que não andou durante a linha) · `git merge-base --is-ancestor main HEAD`.
- Os commits da W11, por cima de `cd7a87e94` (o fecho da W10): `fbe0c81d8` as paredes por mosaicos ·
  `e4bc775cf` a frescura pela versão da malha + o arnês · `37fa60885` o plano §19 e a memória · e o deste
  handoff.

## §1 — Superfície de colisão da W11

**Zero número que soma entre linhas** (`PROJECT_SCHEMA`, registos e `LIVE_SECTIONS` iguais), zero crate nova,
zero pacote externo, **zero linha em `shells/desktop`**, zero i18n. Um `dev-dependency` novo **entre crates do
repo**: `ph2d-navmesh` → `ph2d-orca` (só para o oráculo dos testes; a `ph2d-orca` continua com ZERO
dependências) — o `Cargo.lock` ganha essa aresta.

| ficheiro | o quê |
|---|---|
| `crates/ph2d-orca/src/blocos.rs` · `blocos_tests.rs` **NOVOS** | `pub struct ParedesPorBlocos` (`new` · `poe(chave, lo, hi, paredes)` · `tira` · `retem` · `monta() -> Walls` · `contas() -> Contas`), `pub struct Contas { l1, l2 }`, `pub type blocos::Chave`; reexportada na raiz `ph2d_orca::ParedesPorBlocos` |
| `crates/ph2d-orca/src/walls.rs` | os campos da `Walls` passam a `pub(crate)`; a grelha de `near` passa a uma `Busca` (uma `Grade` por bloco numa grelha regular de blocos; a construção inteira = um bloco). ⚠️ **Nenhuma API pública da `Walls` mudou** |
| `crates/ph2d-nav/src/blocos_junta.rs` · `blocos.rs` · `lib.rs` | `pub struct FaixaDeParedes { chave, lo, hi, paredes: Range<usize> }` (reexportada); `MalhaPorBlocos::faixas_de_paredes()` (vazia se a última `monta` foi recusada); `junta` devolve `(NavMesh, Vec<FaixaDeParedes>)` |
| `crates/ph2d-navmesh/src/tiles.rs` | `TiledMesh::paredes_por_mosaico()` · `TiledMesh::versao()` (sobe a cada mudança da malha) |
| `crates/ph2d-physics-ecs/src/bridge/nav_desvio.rs` · `nav.rs` · `nav_malha.rs` | `NavWorld::walls` passa a `BTreeMap<ChaveMalha, ParedesDaMalha>` (as paredes por mosaicos + as montadas + a versão); `nav_malha.rs` deixa de invalidar (a frescura vive na `ParedesDaMalha::paredes`) |
| gates | `ph2d-orca`: 4 novos em `blocos_tests.rs` · `ph2d-navmesh/src/tiles_tests.rs`: o oráculo da W10 compara também as paredes (+ uma fixtura: dois quadrados encostados por uma quina) · `ph2d-physics-ecs/src/bridge/nav_tests.rs`: +1 · `ph2d-nav/src/blocos_tests.rs`: as faixas numa malha recusada |
| docs | plano 30 §19; o arnês `ferramentas/mutacao_navegacao_w11_2026-10-04.py`; a memória `feedback_a_cache_invalidated_in_another_file_is_a_branch_no_oracle_sees.md` (+1 na família «Ofício de gate», `129`) |

### ⚠️ O que um merge pode partir

1. **Uma parede fora do rectângulo do bloco, ou blocos que não formam uma grelha, são `panic`** (`assert!`) na
   `ParedesPorBlocos` — é o que faz de um ponto de dentro um vértice só daquele bloco. Quem chama hoje é só a
   ponte, com as faixas da `TiledMesh` (as peças já são `assert!` dentro do rectângulo desde a W10).
2. **Quem construa uma `Walls` por literal** dentro da `ph2d-orca`: o campo `grade` é agora `busca`.
3. **Quem descarte `NavWorld::walls` à mão** (uma linha que tenha mexido em `nav_malha.rs` na mesma zona): a
   W11 tirou a invalidação; a frescura é a versão da malha. Um `remove(&chave)` que volte não parte nada (só
   desperdiça a cache).
4. Tectos: `nav/blocos.rs` `629 / 700`, `navmesh/tiles.rs` `534`, `orca/blocos.rs` `422`, `tiles_tests.rs` `403`,
   `bridge/nav.rs` `622` (igual).

## §2 — O que a wave traz

As paredes que o desvio lê são **as MESMAS, ao bit** (`point`, `next`, `prev`, `dir`, `convex`, e a mesma
resposta de `near`); muda QUEM as calcula. Na cena de `100 × 100 m` com `1 000` círculos (`--release`):

| | antes (W10) | W11 |
|---|---|---|
| as paredes depois de uma porta | `0,557 ms` | **`0,149 ms`** (2 blocos refeitos, 12 recosidos) |
| nada mudou (o piso O(malha)) | — | `0,056 ms` |
| a frio (a 1.ª vez de uma malha) | `0,557` | `1,67 ms` (só quando a malha nasce, `~57 ms`) |
| `near` × 20 000 | `26,3–27,7 ms` | `26,9–28,5 ms` (`+2–3 %`) |
| `medir_replaneio` (200 agentes, fila a `20 000`) | pior tique `7,23 · 7,73 ms` | `7,01 · 7,04 ms` — as procuras e a fila IGUAIS nas 12 linhas |

## §3 — ⏳ O que fica ABERTO

- **O mosaico refeito** (`0,70 ms`) é o maior pedaço de uma porta — a alavanca seguinte medida.
- O que resta O(malha) a cada mudança: a junção da malha (`0,37 ms`, plano §18.8) e, nas paredes, comparar +
  concatenar (`0,056 ms`).
- A construção a frio das paredes ficou `3×` mais cara (`1,67 ms`); só paga quando a malha inteira nasce.
- Os abertos da W5–W10 que não eram de custo.

## §4 — A prova de fecho

Tudo 1× sobre o diff acumulado, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha (load `9–29`, outras
sessões):

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | **`19 536 / 19 536`**, `175 s` |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` (orca, nav, navmesh, physics-ecs, app-components, host-desktop) | zero |
| `cargo fmt --all --check` · `cargo machete` | verde · zero |
| `typos` (os ficheiros do diff) | zero da W11 (um nome de grupo do arnês acusado e renomeado `BORDA`). ⚠️ Pré-existente, da W5: uma citação do `desvio.gd` em `HANDOFF_…_DESVIO_2026-10-02.md:171` (cita o `desvio.gd`), e o PDF do tutorial 03 dá falsos positivos |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `censos-da-arvore-combinada.sh` | **`127 / 127`**, `12` de `12` censos |
| `doc-index.sh --check` | `20` índices em dia |
| `git grep from_walkable_walls` | só a definição, testes e doc-comments — o produto já não a chama |

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote novo).

**Mutação 31 / 31** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w11_2026-10-04.py`](../ferramentas/mutacao_navegacao_w11_2026-10-04.py), quatro controlos,
cinco conjuntos de observadores: orca · nav · navmesh · a ponte · os 17 da ponte em ponta). ⛔ A 1.ª corrida deu
**29/30**: apagar a invalidação das paredes montadas em `nav_malha.rs` SOBREVIVIA. Curado no DESENHO (a versão
da malha dentro da `ParedesDaMalha`), não com uma fixtura — e o arnês ganhou P1 (nunca refazer) e P3 (a versão
não sobe), os dois a sangrar. CONTROLOS de população: a fixtura das malhas reais tinha **zero** pontos onde a
fronteira se toca até ganhar os dois quadrados encostados (`24`).

**Auditoria (2 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **correcção** | as paredes por mosaicos são a construção inteira, ao bit | `TiledMesh::update_with_areas` → `MalhaPorBlocos::monta` (faixas) → `ParedesDaMalha::paredes` (versão) → `ParedesPorBlocos::poe` (compara ao bit; L1) → `monta` (L2 das pendentes, concatenar, `Busca`) → `Walls::near` (blocos que tocam o alcance) | `as_paredes_por_blocos_sao_a_construcao_inteira_ao_bit` · o oráculo da W10 com as paredes · `as_paredes_da_ponte_por_mosaicos_sao_as_da_malha_inteira` · B1–B23, W1–W2, N1–N3, P1–P3 |
| **determinismo** | o replay e o hash não mudam | as paredes são as mesmas ⇒ o desvio lê os mesmos números; só `+ − × ÷ sqrt` na `ph2d-orca`, nada novo na ponte | os gates da ponte (hash, scrub, replay) verdes; `medir_replaneio` com as MESMAS procuras e fila antes/depois |
| **não-checado pela compilação** | os números de tempo | a sonda `sonda_paredes_w11` (temporária, apagada; a cena no plano §19.1) e `medir_replaneio`, alternadas | — |

## §5 — O smoke (o comando inteiro)

O que se vê não muda — o smoke é ver que NADA partiu: os personagens desviam-se das paredes como antes.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

Errado = na `=3` o guarda PASSA a porta depois de ela fechar (ela fecha-se na cara dele quando o herói foge
para a zona verde — é aí que ele desiste e volta à ronda), ou não volta à ronda; na `=4` um inimigo entra na
lava ou não usa o portal; na arena um morcego preso no muro.

Fotografadas no ecrã virtual antes de ir ao dono (`target/prova/w11/ph2dnavsmoke3.png`, `ph2dnavsmoke4.png`,
`ph2dvidasmoke4.png`, 59–60 fps; duas esperaram a vez da placa, que a `line_sculpt3d` segurava): na `=3` o guarda
patrulha (*«Moving · 2.24 m to go»*, frame 269); na `=4` o vermelho deu a volta à lava e *«Arrived»* ao herói; na
arena os morcegos e o herói andam (*«Signal: morcego»*).

### O smoke compilado (a 2.ª corrida, colada)

Depois de `rm -rf target/*/incremental` (`5,9 G` do `debug` + `753 M` do `smoke`), a 2.ª corrida de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                186   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                475 : 254   alvo: <= 1,0  razao 1.9x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (1560 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         491 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
