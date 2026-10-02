# HANDOFF DE INTEGRAÇÃO — `line/components`: o DESVIO entre agentes (navegação W5) — 2026-10-02

> Plano e resultados: [`30_plano_navegacao.md`](../30_plano_navegacao.md) §13 (as decisões, o que a
> medição derrubou, as recusas medidas). Este documento é o que o integrador precisa; o porquê vive lá.

## §0 — O `--ff-only` deve passar limpo (medido)

- A linha nasceu de `1ad60a1ce` (o `main` no dia, depois de `git reset --keep main` — todo commit da
  rodada anterior já estava no `main`). `git merge-base --is-ancestor main HEAD` ⇒ verdade no fecho.
- ⚠️ **A linha muda o `CLAUDE.md` da worktree** (a entrada dos Componentes no §5.1: o smoke `=1|2` e o
  link deste handoff). Se o primário entretanto mexer no §5, a fusão é de UMA linha.

---

## §1 — Identidade e superfície de colisão, MEDIDA

- ramo `line/components` · **4 commits** sobre `1ad60a1ce` (os três da wave + o fecho) · 45 ficheiros
- `d97a1fd99` a lei + o oráculo · `bc80e3e17` a ponte, o componente, o Inspector · `7701b1268` a cena
  `=2`, as réguas que a mutação pediu, a prova · o fecho (corte de LOC, `fmt`, `typos`, este handoff)

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, colado no fecho:

```
COLISAO_AQUI
```

| grandeza | aqui | `main` | delta | de onde |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | `179` | `178` | **+1** | o campo `NavAgent::avoidance` (último campo; postcard posicional). Sem migração — um v178 é recusado em voz alta, a decisão da casa |
| tripla do gate | `(179, 13, 22)` | `(178, 13, 22)` | só o 1.º | a forma do `FlipDoc`/`VecScene` não muda |
| registos `ph2d-ecs` / `-render` / `-script` | `108`/`109`/`109` | igual | **0** | |
| registo da FÍSICA (`register_physics_components`) | `42` | `42` | **0** | é um CAMPO de um componente que já viaja |
| `LIVE_SECTIONS` | igual | | **0** | a linha nova vive na secção NAV AGENT |
| `NavFieldEdit` | `+1` | | **+1** | `Avoidance(bool)`, **apendado** no fim |
| ids do painel | `+1` | | | `INSP_NAV_AVOIDANCE = hash_node_id("insp_nav_avoidance")` (`ids/inspector_nav.rs`) |
| i18n | `+1` chave | | | `panel.inspector.nav.avoidance` = `"Avoid Others"` |
| `nav_smoke::CENAS` | `2` | `1` | **+1** | a cena `=2` (o roteador conta-a do `montar`) |
| pacotes **externos** novos | `0` | | **0** | |
| crates novas | `ph2d-orca` | | **+1** | folha, **zero** dependências; o `Cargo.lock` ganha a linha dela |

⚠️⚠️ **CONTE O DELTA, nunca o literal** — o degrau `178 → 179` mora na escada de
[`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) e a tripla em
[`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs). Outra linha que suba o
schema escreve o mesmo `179`: `python3 scripts/schema-recount.py`.

### §1.1 — Os ficheiros FORA da família, e porque são aditivos

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-orca/**` | a lei do desvio (ORCA), o oráculo do Godot (`tests/fixtures/godot/desvio_f4.txt`, 656 KB), o banco de cenários, a sonda de custo | crate NOVA |
| `crates/ph2d-physics-ecs/src/bridge/nav.rs` + **`nav_desvio.rs`** (filho, novo) | a condução de todos os agentes recolhe as `Pedida`s e o desvio escreve a intenção; o `NavWorld` ganha a cache `walls` (limpa com as malhas) | sim — a escrita da intenção MUDOU de sítio dentro da mesma ponte; ⚠️ uma linha que tenha editado o fim de `drive_nav_agents` conflitua aqui |
| `crates/ph2d-physics-ecs/src/components/nav.rs` | `NavAgent::avoidance` (último campo, `true` de fábrica) | sim — `..NavAgent::default()` em todos os construtores do repo (conferido por `git grep`) |
| `crates/ph2d-physics-ecs/Cargo.toml` · `tests/it/{nav_desvio,main,no_std_transcendental_on_the_hash_path}.rs` | a dependência; 8 gates de costura; o gate do hash passa a cobrir `ph2d-orca` | sim |
| `crates/ph2d-editor-core/src/nav_edits.rs` | `InspectorNavAgent::avoidance`; `NavFieldEdit::Avoidance` apendado | sim |
| `crates/ph2d-panel-inspector/src/{ids/inspector_nav,populate_nav,event_nav,sections/nav}.rs` | as pontas da caixa *Avoid Others* | sim — uma linha a mais na secção |
| `crates/ph2d-i18n/src/inspector_nav.rs` | a chave | sim |
| `crates/ph2d-panel-registry-init/tests/it/o_inspector_armado.rs` | `avoidance: true` na fixtura | sim |
| `shells/desktop/src/project_schema{,_tests}.rs` | o degrau `179` | ⚠️ SOMA (§1) |
| `shells/desktop/src/components_scenes_suplentes.rs` | a cena selecciona `montada.escolhido` (era `montada.roxo`) | uma linha — `Montada` mudou de forma (`escolhido` + `labirinto`/`porta`) |
| `docs/Components/ferramentas/godot_nav_oraculo/{desvio.gd,desvio_projeto/project.godot}` · `mutacao_navegacao_w5_2026-10-02.py` | o oráculo e o arnês | sim |
| `CLAUDE.md` | a entrada dos Componentes (§0) | uma linha |

### §1.2 — Contratos congelados

**Nenhum** — `node.rs` e `tool.rs` intocados (a superfície acima o diz); nenhum ADR.

---

## §2 — O que a wave traz (o resumo; o detalhe é o §13 do plano)

- **A lei** (`ph2d-orca`): ORCA (van den Berg et al. 2011) escrito do artigo, só `+ − × ÷ sqrt` em `f64`.
  **Paridade com o Godot 4.7.2 PASSO A PASSO** sobre `5 670` passos em seis cenas: `0,00068 px/s` longe do
  toque, `0,046 px/s` na faixa de toque (`0,5 %` de `R`). ⚠️ Medido no oráculo: o desvio do Godot com as
  linhas de execução de fábrica tem uma **corrida de dados** (não determinístico) — o projecto ao lado do
  script liga-o numa linha só; e ele resolve **em sequência** pela ordem de criação.
- **Em sequência pela ordem das ENTIDADES** + **preferência de lado** (`SIDE_BIAS = 0,25`, planalto medido
  `0,05–0,5`) + **10 vizinhos** (`MAX_NEIGHBORS`, tempo de quadro: 1 000 agentes densos `9,5 → 2,44 ms`).
  ⛔ Recusada, medida: a fotografia comum (Jacobi) prende o círculo de oito com qualquer peso.
- **A parede do desvio é a da MALHA do raio do agente** (ele é um ponto contra ela) — a cura da queixa Q1
  (o Godot desvia ignorando a área andável).
- **A ponte**: ninguém se desvia do PRÓPRIO alvo; todo corpo sólido que anda e não é agente é um obstáculo
  que não desvia; um agente com o desvio desligado vai a direito e os outros desviam-se dele por inteiro.
  Nenhum estado novo no anel (o ORCA não tem memória; a velocidade do mover já vai lá).
- **O Inspector**: *Avoid Others* na secção NAV AGENT, com o clique REAL até ao barramento.
- **A cena `=2`**: a porta de dois sentidos — vermelhos com o desvio chegam todos (tique `500`), os
  cinzentos sem ele entalam-se (nenhum chega). Fotografada aos `5,2 s` e `11,2 s` antes de ir ao dono.

---

## §3 — ⚠️ O que um merge textual pode partir

1. **O fim de `drive_nav_agents`** (`bridge/nav.rs`): a intenção já não é escrita no laço — vai para
   `self.desvia(pedidas, dt)` (`bridge/nav_desvio.rs`). Uma linha que acrescente algo por agente no fim do
   laço tem de o pôr ANTES do `pedidas.push`.
2. **`nav.rs` está em `688 / 700`** depois do corte (estava `788` com a wave inteira). Outra linha que
   escreva lá passa o tecto por ACUMULAÇÃO — a cura é cortar por responsabilidade (o `forma`/`Fnv`/
   `raio_que_envolve` são candidatos), nunca subir o número.
3. **`Montada` do roteador da navegação mudou de forma** (`escolhido` + `Option<Labirinto>`/`Option<Porta>`):
   quem leia `montada.roxo`/`.vermelho` deixa de compilar e o compilador diz onde.
4. **`PROJECT_SCHEMA`** — SOMA (§1).

---

## §4 — ⏳ O que fica ABERTO

- ⏳ **Decisão do dono (plano §11.2):** o desvio nasce LIGADO (a recomendação do plano, agora com o custo
  medido). Dito no relatório ao dono.
- **A leitura viva não diz «a dar passagem»** — um agente travado pela multidão lê-se `Moving`. Candidato a
  estado com voz (S6).
- **O custo que sobra a 1 000 agentes** (`2,44 ms`) é a VARRIDA dos candidatos (a célula da grelha é o
  alcance sem perda): uma procura por anéis numa grelha fina tirá-lo-ia.
- **O empurrão de um golpe** (`TopDownState::knockback`) não entra na velocidade que o desvio vê; um corpo
  COMPOSTO conta só com o colisor principal (as peças não).
- Os abertos de 01/10 continuam: a cena `=1` apertada (o dono notou), o defeito G (a bala só-sensor), o
  flake de carga `text_path_smoke::perf::riding_the_path_costs_about_twice_the_straight_layout` por
  promover. As waves seguintes: W6 (o mundo que muda, portas, `Start/Stop Navigation`, patrulha) · W7 (custo
  por área, atalhos) · W8 (a arena, o tutorial `03_navegacao.pdf`).

---

## §5 — A prova de fecho

Tudo corrido **1× sobre o diff acumulado**, régua no merge-base (`1ad60a1ce`), dentro da fatia da linha:

| portão | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | NEXTEST_AQUI |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `cargo clippy --all-targets --all-features -D warnings` nas 8 crates tocadas (`ph2d-orca` · `ph2d-physics-ecs` · `ph2d-editor-core` · `ph2d-app-components` · `ph2d-panel-inspector` · `ph2d-i18n` · `ph2d-panel-registry-init` · `ph2d-host-desktop`) | **zero** |
| `file_loc_caps` (shell) · `arch_safe_clamp_only` · `the_shell_only_shrinks` | verdes |
| `architecture_workspace_file_loc_cap` | ✗ no 1.º fecho (`nav.rs` `788/700`) ⇒ **cortado** para o filho `nav_desvio.rs` (`688` + `120`) |
| `cargo fmt --all --check` | ✗ no 1.º fecho (14 ficheiros da linha) ⇒ formatado |
| `typos` project-wide | ✗ no 1.º fecho (`tha` no `desvio.gd`) ⇒ renomeado; o oráculo re-corrido dá a fixtura **byte a byte** |
| `cargo machete` | **zero** dependências por usar |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `censos-da-arvore-combinada.sh` (a árvore combinada É esta: o `main` é ancestral) | **`127 / 127`**, `12` de `12` censos |
| `doc-index.sh --check` | DOCINDEX_AQUI |
| `#[cfg(target_os` escrito/movido | **nenhum** ⇒ o cruzamento para macOS não se aplica |

**Mutação 30 / 30** a sangrar, zero defeitos de arnês
([`mutacao_navegacao_w5_2026-10-02.py`](../ferramentas/mutacao_navegacao_w5_2026-10-02.py), os quatro
controlos): 17 na lei, 7 na ponte, 3 na família, 3 no painel. A prova achou **duas leis sem régua** (as
paredes da malha fora do desvio; quem não desvia contado como metade) — as réguas foram escritas e
MEDIDAS com a mutação ao lado — e **um mutante equivalente** (trocado). Depois do corte de LOC o grupo da
ponte foi re-provado nos ficheiros novos: MUTA_PONTE_AQUI.

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (zero pacote externo).

---

## §6 — Os smokes (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_NAV_SMOKE=2 cargo run -p ph2d-host-desktop --profile smoke
```

A porta de dois sentidos: em cima os VERMELHOS (com *Avoid Others*) cruzam-se na porta e chegam ao outro
lado; em baixo os CINZENTOS (o controlo, sem o desvio) entalam-se. O `Red 1` vem escolhido: tirar-lhe o
visto de *Avoid Others* no Inspector faz os outros desviarem-se dele. ⏳ **Por smokar pelo dono.** A cena
`=1` (o labirinto) continua igual, agora com o desvio ligado nos perseguidores.

**O binário do smoke está COMPILADO na worktree** — a 2.ª corrida, colada no fim deste documento.

**A frase do módulo no `CLAUDE.md §5.1`** não muda; a entrada troca o smoke (`=1|2`) e o link (já feito
nesta linha).

---

### O smoke compilado (a 2.ª corrida, colada)

```
SMOKE_AQUI
```
