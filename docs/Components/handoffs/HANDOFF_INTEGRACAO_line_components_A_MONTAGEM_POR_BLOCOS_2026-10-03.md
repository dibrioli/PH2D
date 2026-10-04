# HANDOFF DE INTEGRAÇÃO — `line/components`: A MONTAGEM POR BLOCOS (navegação W10) — 2026-10-03

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §18. Este documento é o que o
> integrador precisa.
>
> ⚠️⚠️ **A linha leva SEIS waves por integrar**, juntas e sem reverter nada umas das outras: W5
> ([`DESVIO_2026-10-02`](HANDOFF_INTEGRACAO_line_components_DESVIO_2026-10-02.md)), W6
> ([`O_MUNDO_QUE_MUDA_2026-10-02`](HANDOFF_INTEGRACAO_line_components_O_MUNDO_QUE_MUDA_2026-10-02.md)), W7
> ([`O_CUSTO_E_OS_ATALHOS_2026-10-03`](HANDOFF_INTEGRACAO_line_components_O_CUSTO_E_OS_ATALHOS_2026-10-03.md) — a
> superfície foundational e o §3 «o que um merge pode partir»), W8
> ([`A_ARENA_E_O_TUTORIAL_2026-10-03`](HANDOFF_INTEGRACAO_line_components_A_ARENA_E_O_TUTORIAL_2026-10-03.md)), W9
> ([`O_CUSTO_A_ESCALA_2026-10-03`](HANDOFF_INTEGRACAO_line_components_O_CUSTO_A_ESCALA_2026-10-03.md) — ⚠️ a API do
> `Poly` como VISTA) e esta W10. Smoke do dono APROVADO na W5–W9; **o da W10 está por fazer** (§5).

## §0 — O `--ff-only` deve passar limpo

- base `1ad60a1ce` (o `main`, que não andou durante a linha) · `git merge-base --is-ancestor main HEAD`.
- Os commits da W10, por cima de `de2d23cae` (o fecho da W9): `11de0849c` a montagem por blocos ·
  `d4fc2e8ee` a prova (mutação 21/21, as fixturas, o plano §18) · `9e78132c0` clippy + fmt · e o deste
  handoff.

## §1 — Superfície de colisão da W10

**Zero número que soma entre linhas** (`PROJECT_SCHEMA` `180`, registos e `LIVE_SECTIONS` iguais), zero crate
nova, zero pacote externo, **zero linha em `shells/desktop`**, zero i18n.

| ficheiro | o quê |
|---|---|
| `crates/ph2d-nav/src/blocos.rs` · `blocos_junta.rs` · `blocos_tests.rs` **NOVOS** | `pub struct MalhaPorBlocos` (`new` · `poe(chave, Peca)` · `tira(chave)` · `monta() -> Result<NavMesh, MeshError>`) e `pub struct Peca { lo, hi, verts, ring_off, ring, area }`, reexportados na raiz; `pub type blocos::Chave` |
| `crates/ph2d-nav/src/grelha.rs` **NOVO** (privado) | a grelha de localização saiu do `mesh.rs`; `Localizador` = uma grelha, ou uma por bloco |
| `crates/ph2d-nav/src/mesh.rs` | os campos da `NavMesh` passam a `pub(crate)`; `polys_de_cada_vertice` partilhado; `NavMesh::diferenca(&self, &NavMesh) -> Option<&'static str>` só com `test-support`/`test` (a comparação campo a campo dos gates). ⚠️ **Nenhuma API pública da `NavMesh` mudou** |
| `crates/ph2d-navmesh/src/tiles.rs` | a `TiledMesh` monta pela `MalhaPorBlocos` (`+blocos`, `fn peca`); o `monta` inteiro saiu para `tiles_tests.rs` como ORÁCULO |
| gates | `ph2d-navmesh/src/tiles_tests.rs` (+1, o oráculo), `ph2d-nav/src/blocos_tests.rs` (8) |
| docs | plano 30 §18; o arnês `ferramentas/mutacao_navegacao_w10_2026-10-03.py` |

### ⚠️ O que um merge pode partir

1. **Uma `Peca` fora do seu rectângulo é um `panic`** (`assert!`, não `debug_assert!`): é o que torna impossível
   a mesma aresta orientada em dois blocos (plano §18.5). Quem chama hoje é só a `TiledMesh`, que corta cada
   mosaico pelo rectângulo dele.
2. **Quem construa uma `NavMesh` por literal** dentro da `ph2d-nav`: os campos são `pub(crate)` agora e o
   campo `grid` é um `Localizador`.
3. Tectos: `mesh.rs` `539 / 700` (desceu: a grelha mudou-se), `blocos.rs` `~590`, `tiles.rs` `~517`,
   `tiles_tests.rs` `~280`.

## §2 — O que a wave traz

A malha montada é **a MESMA, ao bit** (vértices, anéis cosidos, vizinhos, gémeos, cantos,
vértice→polígonos, ilhas, áreas, paredes, caixa — e a mesma resposta de `locate_all`); muda QUEM a calcula. Na
cena de `100 × 100 m` com `1 000` obstáculos (`--release`, antes/depois alternados, load `5–9`):

| | antes (W9) | W10 |
|---|---|---|
| uma porta (sem · com 100 lamas) | `2,66 · 3,77 ms` | **`1,14 · 1,82 ms`** |
| uma lama a mexer | `5,23 ms` | **`3,62 ms`** |
| a montagem (por fase, load `1,7`) | `1,95 ms` | **`0,41 ms`** |
| o pior tique depois de uma porta, 10 · 50 · 200 agentes (fila a `20 000`) | `5,2 · 5,9 · 8,4 ms` | **`3,6 · 4,4 · 7,0 ms`** — com as mesmas procuras e a mesma fila |

## §3 — ⏳ O que fica ABERTO

- **A grelha das paredes do desvio** (`ph2d_orca::Walls::from_walkable_walls`) refaz-se inteira a cada mudança:
  `0,53 ms` — agora um terço de uma porta na ponte. A ORDEM das paredes desempata distâncias iguais e entra no
  hash: a versão incremental tem de dar os mesmos índices (o molde: blocos + oráculo).
- O mosaico refeito (`0,70 ms`) é o maior pedaço de uma porta.
- O que resta O(malha) na junção (`0,37 ms` de cópia e tradução) só desce com ids fixos — recusados enquanto
  nenhum consumidor os usar (plano §18.6).
- Os abertos da W9 (§3 dela) e da W5–W8 que não eram de custo.

## §4 — A prova de fecho

Tudo 1× sobre o diff acumulado, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha:

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | **`19 530 / 19 530`**, `125 s` (load `3,1`) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` (nav, navmesh, orca, physics-ecs, app-components, host-desktop) | zero — a 1.ª acusou dois `type_complexity` (curados com `type Aresta` e `type Caso`) |
| `cargo fmt --all --check` | verde (a 1.ª acusou 5 ficheiros da W10; formatados) |
| `typos` · `cargo machete` | zero · zero |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `censos-da-arvore-combinada.sh` | **`127 / 127`**, `12` de `12` censos |
| `doc-index.sh --check` | `20` índices em dia |
| `git grep '\.polys()\['` no código | zero |
| a máquina no fim | nada desta linha a correr; `/dev/dri` só Xwayland/plasmashell |

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote novo).

**Mutação 21 / 21** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w10_2026-10-03.py`](../ferramentas/mutacao_navegacao_w10_2026-10-03.py), quatro controlos).
⛔ A 1.ª corrida deu **17/22**: quatro ramos sem fixtura (tirar um bloco; o vizinho com os mesmos pontos de lado
que deixa de ligar; dois pontos cosidos numa aresta que DESCE; o ponto a EPS da fronteira dos blocos), curados
com quatro fixturas; e uma EQUIVALENTE (a parede que só marca o canto `u`), que saiu com o porquê. E o
CONTROLO de população do oráculo acusou **zero junções em T** na 1.ª fixtura: o corte canónico das costuras dá
os mesmos pontos aos dois lados, e uma junção só nasce de um obstáculo que ENCOSTA a uma costura de um lado só
— a fixtura põe-nos agora de propósito (`115` vértices cosidos).

**Auditoria (2 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **correcção** | a malha por blocos é a montagem inteira, ao bit | `TiledMesh::update_with_areas` → `MalhaPorBlocos::poe` (L1 + quem se recose) → `monta` (L2 `cose` · L3 `liga`) → `junta` | `a_montagem_por_blocos_e_a_montagem_inteira_ao_bit` (o `monta` antigo como oráculo, campo a campo + `locate_all`) · M1–M22 |
| **determinismo** | o replay e o hash não mudam | a malha é a mesma ⇒ a procura, a fila e o desvio lêem os mesmos números | os gates da ponte (hash c9, scrub) verdes; `medir_replaneio` com as MESMAS procuras e a mesma fila antes/depois |
| **não-checado pela compilação** | os números de tempo | as sondas `medir_custo` (`SO_GRANDE=1`) e `medir_replaneio`, alternadas, com o load ao lado | — |

## §5 — O smoke (o comando inteiro)

O que se vê não muda — o smoke é ver que NADA partiu: as cenas da navegação andam como antes.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

Errado = o guarda da `=3` não passa a porta quando ela abre, ou fica parado depois de ela fechar; na `=4` um
inimigo entra na lava ou não usa o portal; na arena um morcego preso no muro.

Fotografadas no ecrã virtual antes de ir ao dono (`target/prova/w10/nav3.png`, `nav4.png`, `vida4.png`, 59–60
fps): na `=3` o guarda patrulha (*«Moving · 2.14 m to go»*, o mesmo da foto da W9); na `=4` o vermelho deu a
volta à lava e *«Arrived»* ao herói; na arena os morcegos e o herói andam.
