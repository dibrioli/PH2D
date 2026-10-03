# HANDOFF DE INTEGRAÇÃO — `line/components`: O CUSTO À ESCALA (navegação W9) — 2026-10-03

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §17. Este documento é o que o
> integrador precisa.
>
> ⚠️⚠️ **A linha leva CINCO waves por integrar**, todas juntas e sem reverter nada umas das outras: a W5
> ([`DESVIO_2026-10-02`](HANDOFF_INTEGRACAO_line_components_DESVIO_2026-10-02.md)), a W6
> ([`O_MUNDO_QUE_MUDA_2026-10-02`](HANDOFF_INTEGRACAO_line_components_O_MUNDO_QUE_MUDA_2026-10-02.md)), a W7
> ([`O_CUSTO_E_OS_ATALHOS_2026-10-03`](HANDOFF_INTEGRACAO_line_components_O_CUSTO_E_OS_ATALHOS_2026-10-03.md) —
> a superfície foundational inteira e o §3 «o que um merge pode partir»), a W8
> ([`A_ARENA_E_O_TUTORIAL_2026-10-03`](HANDOFF_INTEGRACAO_line_components_A_ARENA_E_O_TUTORIAL_2026-10-03.md),
> com os cinco defeitos do §2-bis) e esta W9. ✅ **Smoke do dono APROVADO na W5, W6, W7, W8 e W9 (03/10)** — as
> cenas `PH2D_NAV_SMOKE=3`, `=4` e `PH2D_VIDA_SMOKE=4` (§5).

## §0 — O `--ff-only` deve passar limpo

- base `1ad60a1ce` (o `main`, que não andou durante a linha) · `git merge-base --is-ancestor main HEAD` no
  fecho.
- Os commits da W9 (por cima de `f13d3d375`, o fecho dos defeitos da W8): `af60965d7` a dominância entre
  frentes · `a0c21d06f` a malha contígua · `38aeb4852` o ORCA entalado · `98054121e` a fila do
  replaneio · `0790c19f9` a prova + o plano §17 · e os de fecho.

## §1 — Superfície de colisão da W9

**Zero número que soma entre linhas**: `PROJECT_SCHEMA` continua `180`, registos e `LIVE_SECTIONS`
iguais, zero crate nova, zero pacote externo, **zero linha em `shells/desktop`**, zero i18n.

| ficheiro | o quê |
|---|---|
| `crates/ph2d-nav/src/mesh.rs` | ⚠️ **API pública mudou**: `Poly` é uma VISTA (`Poly<'a>`, `Copy`, campos `&[u32]` / `&[Option<u32>]` / `&[u32]`); `NavMesh::poly(p)`, `poly_count()`, `polys()` passa a ITERADOR. Nova porta `NavMesh::from_rings(verts, ring_off, ring, area)`; `from_polygons(_with_areas)` achatam e chamam-na. Os erros (`MeshError`) e a ordem em que são reportados iguais |
| `crates/ph2d-nav/src/agent.rs` | `AgentRuntime` `+owed: u32` `+broken: bool` `+last_nodes: u64` (pub; o tipo tem campos privados — ninguém o constrói por literal) |
| `crates/ph2d-nav/src/{refresh.rs, polyanya_dominancia.rs}` **NOVOS** | a lei da fila; a dominância |
| `crates/ph2d-nav/src/{polyanya.rs, polyanya_custo.rs, cost.rs, oracle.rs, lib.rs}` | `Stats` `+dominated` `+trimmed`; `Polyanya::set_front_dominance` (o CONTROLO); a gama de deslize de uma raiz de refracção é a aresta inteira |
| `crates/ph2d-navmesh/src/tiles.rs` | a montagem escreve os anéis contíguos e chama `from_rings` |
| `crates/ph2d-orca/src/lp.rs` | `lines[n_walls.min(i)..i]` (a cura do panic) |
| `crates/ph2d-orca/src/walls.rs` | (depois do smoke) a GRELHA das arestas: `Walls::near` lê só as células ao alcance — a mesma resposta, ao bit; a varredura inteira fica como oráculo `#[cfg(test)]` |
| `crates/ph2d-navmesh/src/tiles.rs` | (depois do smoke) `TiledMesh::changed_area() -> Option<Vec<(V2, V2)>>` (os mosaicos refeitos na última actualização) |
| `crates/ph2d-nav/src/refresh.rs` | (depois do smoke) `path_still_walkable(mesh, rt, pos, onde: Option<&[(V2, V2)]>)` — só percorre os troços que tocam `onde` |
| `crates/ph2d-physics-ecs/src/bridge/{nav.rs, nav_fila.rs NOVO}` | a fila na ponte; `ORCAMENTO_DE_NOS_POR_TIQUE = 20 000`; `PhysicsBridge::set_nav_replan_budget` (sonda e gates) |
| exemplos | `ph2d-navmesh/examples/medir_custo.rs` (`SEM_DOMINANCIA`, `SO_GRANDE`, `PIOR`, a linha «uma porta»), `ph2d-physics-ecs/examples/medir_replaneio.rs` **NOVO** |
| gates | `ph2d-navmesh/tests/it/dominancia.rs` **NOVO**, `ph2d-physics-ecs/tests/it/nav_mundo.rs` (+5), `ph2d-orca/src/tests.rs` (+1), `ph2d-nav/src/refresh.rs` (+1) |
| docs | plano 30 §17; o arnês `ferramentas/mutacao_navegacao_w9_2026-10-03.py` |

### ⚠️ O que um merge pode partir

1. **Quem leia `mesh.polys()[i]`, `.polys().len()` ou guarde um `&Poly`** noutra linha: não compila (é o
   bom caso). A troca é mecânica — `mesh.poly(i)`, `poly_count()`, `polys()` itera. ⛔ **Não a faça por
   `sed`/regex sem reler**: a minha troca de `!m.polys().is_empty()` deu `!m.poly_count() == 0` (um NÃO
   bit-a-bit sobre o número — toda malha parecia vazia), compilou, e só os gates do agente o apanharam.
   `git grep -n '\.polys()\['` no fecho: zero.
2. **Quem espere que TODO agente da malha que mudou replaneie no tique da mudança**: agora um caminho que
   ainda se anda continua e entra na fila. Numa cena pequena o orçamento cobre todos no mesmo tique (o
   comportamento de antes); numa grande, não. O gate antigo `so_os_agentes_da_malha_que_mudou_refazem_o_caminho`
   continua verde.
3. `nav.rs` da ponte a `622 / 700`; `polyanya.rs` a `676`; `polyanya_custo.rs` a `615`; `mesh.rs` a
   `610`; `nav_mundo.rs` (gates) a `616`.

## §2 — O que a wave traz

O plano §17 tem tudo (as tabelas, as decisões, as recusas). Em resumo, numa cena de `100 × 100 m` com
`1 000` obstáculos (`--release`, load `~2–3`):

| | antes (fecho da W8) | W9 |
|---|---|---|
| procura com 100 lamas: nós · mediana · p95 · máx | `64 508 · 4,7 · 38 · 66 ms` | **`24 661 · 3,5 · 21 · 30 ms`** |
| uma porta que pára (sem · com lamas) | `3,96 · 5,66 ms` | **`2,65 · 3,76 ms`** |
| uma lama a mexer | `7,13 ms` | **`5,23 ms`** |
| o pior tique depois de uma porta, 10 · 50 · 200 agentes | `10,4 · 34,3 · 124,3 ms` | **`5,7 · 6,5 · 10,1 ms`** (load `5–8`) |
| o tique SEM nada a mudar, 10 · 50 · 200 agentes (depois do smoke) | `2,1 · 10,5 · 37,6 ms` | **`0,38 · 0,82 · 2,29 ms`** |

A precisão da procura ponderada ficou **ao dígito** (a tabela §3 da sonda e a lista dos pares acima de
`1,0001` iguais com e sem a dominância). E um **crash do ORCA** que só aparece à escala: um agente entalado
entre duas paredes (as paredes sozinhas sem velocidade comum) fazia o programa 3D varrer
`lines[n_walls..i]` com `i < n_walls` — `panic`. Achado pela sonda nova com 200 agentes.

## §3 — ⏳ O que fica ABERTO

- **Uma malha com ids FIXOS por mosaico** (polígono = mosaico + índice local): a montagem passaria a ser
  proporcional ao mosaico tocado. Mexe no núcleo da procura — wave própria (plano §17.6).
- ✅ ~~O tique a 200 agentes SEM mudança nenhuma era `37,6 ms`~~ — curado depois do smoke (plano §17.7):
  era o desvio a varrer TODAS as arestas de parede da malha por agente (`97 %` do tique). E no tique em
  que a malha muda, a fila percorria o caminho inteiro de cada agente (`12,4 ms` a 200): agora só os
  troços que tocam os mosaicos refeitos. O que sobra no tique da porta é a malha refeita DUAS vezes
  (`~3 ms` cada: quando a porta começa a andar e quando pára).
- A procura ponderada continua `~4×` os nós da uniforme (a grelha das fronteiras — legítimo, §17.3); o
  orçamento da fila em nós não é tempo uniforme entre as duas (`~110` contra `~330 ns` por nó).
- A 1.ª procura de agentes que nascem juntos não passa pela fila.
- Os abertos da W5–W8 que não eram de custo (handoff da W8 §3).

## §4 — A prova de fecho

Tudo 1× sobre o diff acumulado, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha:

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | **`19 518 / 19 518`**, `122,8 s` (load `1,4 → 21` durante a corrida) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| clippy `--all-targets --all-features -D warnings` (nav, navmesh, orca, physics-ecs, app-components, panel-inspector, host-desktop) | zero — a 1.ª corrida acusou `needless_range_loop` no `mesh.rs` (o laço da vizinhança passa a `ring_off.windows(2)`) e `manual_is_multiple_of` num gate; curados, e os testes das três crates de novo verdes (`19` · `39` · `810`) |
| `cargo fmt --all --check` | verde |
| `typos` · `cargo machete` | zero · zero |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `censos-da-arvore-combinada.sh` | **`127 / 127`**, `12` de `12` censos |
| `doc-index.sh --check` | `20` índices em dia |
| `#[cfg(target_os` escrito/movido | nenhum |
| a máquina no fim | nada desta linha a correr; `/dev/dri` só Xwayland/plasmashell/code |
| **depois do smoke** (a grelha das paredes, os troços): `nextest-impacted` · `check --workspace` deny · clippy (6 crates) · fmt · censos · typos | **`19 521 / 19 521`** · verde · zero (a 1.ª acusou um `!(range > 0.0)`) · verde · verdes · zero |

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote novo).

**Mutação 19 / 19** a sangrar (15 no fecho + M17–M20 da grelha das paredes e dos troços, depois do smoke),
zero defeitos de arnês
([`mutacao_navegacao_w9_2026-10-03.py`](../ferramentas/mutacao_navegacao_w9_2026-10-03.py), quatro
controlos, grupos NAV · NAVMESH · ORCA · PONTE). ⛔ **A 1.ª corrida deu 13/16**, e cada sobrevivente
mudou alguma coisa: o passo «tirar os pontos a mais e polir de novo» era REDUNDANTE com a gama da aresta
inteira (saiu — mesmos custos, mais depressa); o scrub do gate ia para um tique sem âncora antes (o anel
guarda de 10 em 10 — refazia tudo do zero); e a ordem da fila pela consulta ao mundo coincidia, na
fixtura, com a das entidades (metade dos guardas leva agora um componente de teste).

**Auditoria (2 lentes)**

| lente | claim | traço | asserção-vermelha |
|---|---|---|---|
| **precisão** | a dominância nunca encarece um caminho | `Polyanya::expand` → (custo igual na aresta) `domina` → `corte` (o prefixo onde `D ≤ 0`, bissecção conservadora) → o intervalo cortado expande | `a_dominancia_corta_nos_e_nunca_encarece_um_caminho` (96 pares, CONTROLO sem ela) · M2 · a sonda `PIOR=1` igual nos dois |
| **determinismo** | o replay serve a mesma fila | `nav_fila::fila_do_replaneio` lê só `AgentRuntime.{owed,broken,last_nodes}` + o `Ord` das entidades → `serve` → `forget_path` → `step_with` salda a dívida; o `AgentRuntime` vai no `ControllerMemory` do anel | `um_scrub_para_o_meio_da_fila_devolve_a_mesma_corrida` (semeia do âncora 10 a meio da fila) · M12 · M14 · M16 |
| **não-checado pela compilação** | os números de tempo | as sondas `medir_custo` e `medir_replaneio`, com o load ao lado; os nós e as procuras não dependem da carga | — |

## §5 — O smoke (o comando inteiro)

O que se vê não muda numa cena pequena — o smoke é ver que NADA partiu: as cenas da navegação continuam a
andar como antes.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

Errado = o guarda da `=3` não passa a porta quando ela abre, ou fica parado depois de ela fechar; na `=4`
um inimigo entra na lava ou não usa o portal; na arena um morcego preso no muro (e o tutorial 03 da W8
continua a valer).

Fotografadas no ecrã virtual antes de ir ao dono (`target/prova/w9/nav3.png`, `nav4.png`, 59 fps): na `=3`
o guarda patrulha (*«Moving · 2.13 m to go»*); na `=4` o vermelho deu a volta à lava e *«Arrived»* ao
herói.

### O smoke compilado (a 2.ª corrida, colada)

Depois de `rm -rf target/*/incremental` (`7,1 G` do `debug` + `743 M` do `smoke`), a 2.ª corrida de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — zero linhas `Compiling`:

```
▸ linha line_components · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

O perfil do laço do agente (`bash scripts/agent-loop-profile.sh`), no fecho:

```
  ✗ cargo test : cargo check                474 : 229   alvo: <= 1,0  razao 2.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  39%   alvo: >= 80%  (712 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         372 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
──────────────────────────────────────────────────────────────────────────────
  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETIVA_IMPLEMENTACAO aponta pra la'.
  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.
```
