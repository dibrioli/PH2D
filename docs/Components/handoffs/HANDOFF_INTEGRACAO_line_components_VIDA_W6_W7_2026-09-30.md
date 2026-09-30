# HANDOFF DE INTEGRAÇÃO — `line/components`: VIDA E DANO W6 + W7 (+ a bala da W5) — 2026-09-30

> ⛔ **Este documento SUPERSEDE o [`…_VIDA_2026-09-25.md`](HANDOFF_INTEGRACAO_line_components_VIDA_2026-09-25.md)
> como documento de integração da linha.** O bloco daquele (os ABERTOS + a PARALAXE + a VIDA W0–W5)
> **já está no `main`** (merge-base `912a9652e`); este cobre só os **8 commits** que vieram depois.
> O §3 daquele (o que um merge textual pode partir no bloco da vida) continua válido e não se repete.

## §0 — O `--ff-only` deve passar limpo (medido)

- O `main` (`912a9652e`) **é ancestral** do HEAD da linha (`git merge-base --is-ancestor main HEAD`
  ⇒ verdade): **zero** commits do `main` por trazer, **nenhum rebase foi preciso**.
- A árvore do **primário** estava **limpa** no fecho (`git status --short` vazio), ao contrário de
  25/09 (§0 do handoff anterior). ⚠️ Se entretanto outra sessão gravar memória lá, a receita é a
  mesma daquele §0: comitar primeiro a memória do primário, rebasear a linha, unir.
- A linha traz **quatro** ficheiros de `project-memory/` (§1) — se o primário tiver mexido nos
  mesmos, a cura é a **UNIÃO** (memória é append), com o `MEMORY.md` sob o tecto do gate
  (`o_indice_cabe_no_orcamento_do_carregador`; a linha deixa-o em `21 917` bytes).

---

## §1 — Identidade e superfície de colisão, MEDIDA

- ramo `line/components` · **9 commits** sobre `912a9652e` · `68` ficheiros — os `66` do colado abaixo
  (`+5 544` / `−131`, medidos antes do commit do fecho) mais este handoff e o `.typos.toml`

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, colado:

```
  merge-base 912a9652e   ·   8 commit(s)   ·   66 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        177   (base: 176)
  ⚠   └ tripla do gate               (177, 13, 22)   (base: (176, 13, 22))
    VEC_SCENE_SCHEMA 22 · FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23   (intactos)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs 108 · ph2d-render (espelho) 109 · ph2d-script (espelho) 109   (intactos)
▸ CONTRATO CONGELADO (§6)       node.rs intocado · tool.rs intocado
▸ ADR                           esta linha não cria ADR
▸ Cargo.lock                    nenhum '+name' novo
▸ MARCADORES DE CONFLITO        nenhum
▸ TETOS DE LOC                  nenhum arquivo da linha passa do teto
```

| grandeza | aqui | `main` | delta | de onde |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | `177` | `176` | **+1** | W6: `Damage` ganha `kind` + as três grandezas do dano que dura; `Health` ganha `resistances`. **Campos** de dois componentes já registados, sem migração (o default é o golpe de antes ao bit) |
| registos `ph2d-ecs` / `-render` / `-script` | `108`/`109`/`109` | igual | **0** | |
| ⚠️ registo da FÍSICA (`register_physics_components`) | igual | igual | **0** | ⛔ o `collision-surface.sh` não o mostra — conferido à mão: zero componentes novos |
| `vida_smoke::CENAS` | `4` | `2` | **+2** | a `=3` (W6, tipos de dano) e a `=4` (W7, a arena) |
| catraca de **comandos** do Inspector | `85` | `81` | **+4** | a lista `Resistances` (`+ Add` / `x Remove` e as duas linhas da fixtura), contada como as seis listas irmãs |
| catraca de **cortes no degrau estreito** | `89` | `87` | **+2** | o par `+ Add Resistance` / `x Remove`, como os nove irmãos |
| catraca de **letras perdidas** | `85` | `83` | **+2** | idem |

⚠️⚠️ **CONTE O DELTA, nunca o literal.** O degrau `176 → 177` mora na escada do
[`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) e a tripla no
[`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs). As três catracas
do Inspector SOMAM entre linhas — uma linha que também acrescente uma lista ao Inspector escreve
os mesmos literais; recontar é somar os dois deltas ao valor do `main` do dia.

**Os ficheiros FORA da família** (o resto é `ph2d-app-components`, `ph2d-health`,
`ph2d-panel-inspector`, `ph2d-physics-ecs`, `ph2d-i18n/inspector_vida.rs` e `docs/Components/`):

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-editor-core/src/vida_edits.rs` | `VidaFieldEdit` ganha as variantes dos tipos/resistências; porta nova `InspectorDamageInfo::fere` | sim — variantes **apendadas**; um `match` exaustivo noutra linha deixa de compilar e o compilador diz onde |
| `crates/ph2d-physics-ecs/Cargo.toml` + `Cargo.lock` | dependência **interna** `ph2d-label-fold` | sim — já estava no fecho pela `ph2d-tags`; zero pacote externo |
| `shells/desktop/src/project_schema*.rs` | o degrau `177` | ver acima |
| `shells/desktop/src/components_scenes_suplentes.rs` | a lista de acções das cenas da vida passa a vir de `vida_smoke::accoes(nivel)` | sim (1 linha) |
| `shells/desktop/tests/it/o_painel_pinta_todo_o_modelo_aceita.rs` | ata `RESISTANCES_MAX = 8` aos ids que a lista pinta | sim |
| `.typos.toml` | `resistencias` em `extend-words` · a saída do oráculo Godot em `extend-exclude` | sim — duas entradas apendadas; outra linha que apende ao lado funde limpo (ou conflitua trivialmente no fim da lista: cura = as duas) |
| `scripts/ph2d-run.sh` | o servidor do `sccache` arranca **antes** da trava da placa, e a recusa nomeia quem a segura sem `.dono` | sim — infra partilhada: ver §2 |
| `project-memory/` (4 ficheiros) | `MEMORY.md` · `reference_topic_mutation_proofs.md` · `reference_topic_ship_ci_integration_lessons.md` · `feedback_a_daemon_launched_under_the_gpu_lock_holds_it_forever.md` (novo) | união, se colidir (§0) |

---

## §2 — O que os 8 commits trazem

Pesquisa · plano · resultados: [`28_plano_vida_e_dano.md`](../28_plano_vida_e_dano.md) §13 (a bala) ·
§14 (W6) · §15 (W7).

| commit(s) | o que traz |
|---|---|
| `08ee6fa1b` | ⚠️ **infra partilhada:** `ph2d-run.sh` — o servidor do `sccache` é um DAEMON e herdava o fd 9 da trava da placa quando lançado por um comando `PH2D_GPU=1`, segurando a placa **para sempre** (três linhas presas 21 min com o `.dono` vazio). Hoje arranca antes do `flock`, e a recusa corre `fuser` quando não há `.dono` |
| `aee323d0b` | *«Não vejo a bala»* (smoke da W5): a simulação estava certa, a CENA pousava a linha de tiro debaixo da coluna de avisos `Signal: …`. As cenas `=1`/`=2` saem da coluna (`COLUNA_DOS_AVISOS_X`, medida na foto, guardada por erro de compilação) |
| `6b1b769c9` | **W6 — a lei** (`ph2d-health/src/tipos.rs`): `Taxa { mult, absorve }` corrida contra o addon MIT *Health, HitBoxes, HurtBoxes* do Godot (21 casos, [`godot_health_tipos/`](../ferramentas/godot_health_tipos/)); a taxa entra depois da armadura e antes do escudo; `golpe()` delega em `golpe_tipado(Taxa::NEUTRA)` byte a byte. As **aflições** (o dano que dura) são lei nossa, com gate por cláusula. ⛔ Divergência declarada: o alvo arredonda cada golpe; a casa é `f64` |
| `2d2f3dba6` | **W6 — a ponte** (`ph2d-physics-ecs`): o tipo do `Damage` encontra a resistência do `Health` pela **dobra** da casa (`kind_key`); a aflição entra com o golpe; os pulsos correm depois do `pre_quadro` e antes dos golpes; a morte cura; as `Aflicoes` vivem no `HealthState` (no anel) |
| `880807421` | **W6 — o Inspector** (`Type`, o dano que dura, a lista `Resistances` no idioma da máquina de estados) e a cena `=3` |
| `f0e734d96`, `9d08db2d3` | o que a prova de mutação da W6 achou: a lava **renova** a queimadura a cada tique de contacto; uma guarda redundante saiu; dois gates novos — **37 de 37** sangram |
| `377933503` | **W7:** a arena `=4` (um jogo pequeno com vida do princípio ao fim, sem peça nova de motor), o **tutorial** [`tutoriais/02_vida_e_dano.pdf`](../tutoriais/02_vida_e_dano.pdf) com gate dos rótulos, e a porta `InspectorDamageInfo::fere` (a queixa *«it hurts nobody»* mentia sobre um dano que só DURA) — **21 de 21** sangram |

---

## §3 — ⚠️ O que um merge textual pode partir

1. **Variantes novas em `VidaFieldEdit`** (tipo, dano que dura, resistências — **apendadas**).
2. **Campos novos em struct literais** (a cura é o neutro de cada um): `Damage { kind: String::new(),
   over_time_per_s: 0.0, over_time_s: 0.0, over_time_every_s: 1.0 }` · `Health { resistances: vec![] }`
   · `InspectorDamageInfo`/`InspectorVidaInfo` ganham os campos espelho — a fixtura de
   `o_inspector_armado.rs` escreve-os (conferido: é o único ficheiro de fixturas que a linha tocou).
3. **`statemachine::lista` / `botoes` passaram a `pub(super)`** (reaproveitadas pela lista das
   resistências em vez de uma 5.ª cópia) — uma linha que as reescreva funde ao lado.
4. **As três catracas do Inspector** (§1) e o `PROJECT_SCHEMA` — SOMAM.
5. **O gate do tutorial 01** (`o_tutorial_nomeia_rotulos_que_existem.rs`) exporta agora as portas do
   parser (`chaves_e_textos_de` · `rotulos_citados_de` · `desescapa` · `sem_prosa`) e **lê os
   pintores SEM a prosa** — uma linha que acrescente um rótulo só num comentário deixa de o satisfazer.

---

## §4 — ⏳ O que fica ABERTO

- **Decisão do dono (dita a ele no smoke da W7):** *«um dano por segundo RENOVA a aflição a cada
  tique de contacto»* — a lava queima enquanto se pisa e ainda `dur_s` depois de sair. Lida da frase
  da própria lei; a alternativa (renovar só ao entrar) é uma linha na ponte.
- **O defeito G** (a bala só-sensor, aberto desde a W2) — **não** era o *«não vejo a bala»* (§13 do
  plano) e continua por investigar.
- **A coluna de avisos por cima do jogo é uma FAMÍLIA** (o placar do #20, as cenas da vida): toda cena
  de jogo a evita à mão; a cura de produto é da linha do runtime/UI e decisão do dono (plano §13).
- Se a dobra com **oito** resistências abertas cabe no encaixe do painel — **não medido** (plano §14).
- As fronteiras declaradas de cada wave: plano 28 §8.4 · §10.7 · §11.6 · §12.7.

---

## §5 — A prova de fecho

Tudo corrido **1× sobre o diff acumulado**, com a régua no merge-base (`BASE=912a9652e`), dentro da
fatia da linha (`ph2d-run.sh`):

| portão | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **`18 669 / 18 669`** verdes (`11 673` saltados) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde, **zero** avisos |
| `bash scripts/check-standalone-optional.sh` · `bash scripts/check-workflow-packages.sh` | verdes (10 crates com dependência opcional · 32 nomes de workflow contra 394 membros) |
| `bash scripts/censos-da-arvore-combinada.sh` (§1.5.9 5-bis — a árvore combinada **é** esta: `main` é ancestral) | **`127 / 127`**, controlo do filtro `12 de 12` |
| `cargo clippy --all-targets -D warnings` nas três crates da W7 (`ph2d-app-components` · `ph2d-editor-core` · `ph2d-panel-inspector`) | **zero** (a W6 correu o seu no fecho dela) |
| registo em âmbito workspace (`-E 'package(ph2d-panel-registry-init)'`) | **`133 / 133`** |
| `cargo fmt --all --check` | limpo |
| `cargo machete` | **zero** dependências por usar |
| `bash scripts/doc-index.sh --check` | `20` índices em dia |
| `#[cfg(target_os` escrito/movido | **nenhum** ⇒ o cruzamento para macOS não se aplica |

`loadavg` no fim: `16,32 14,15 10,81` — nenhum gate de razão reprovou, logo a carga não precisou de
ser desmentida.

**Mutação:** W6 **37 / 37** ([`mutacao_vida_w6_2026-09-29.sh`](../ferramentas/mutacao_vida_w6_2026-09-29.sh))
· W7 **21 / 21** ([`mutacao_vida_w7_2026-09-30.sh`](../ferramentas/mutacao_vida_w7_2026-09-30.sh)),
os dois com pré-voo das âncoras (`MUTA_SO_ANCORAS=1`) e o da W7 com o **controlo da árvore** (somas
dos ficheiros mutados antes e depois, `exit 2` se diferirem — nasceu de a 1.ª corrida ter deixado a
árvore mutada, plano §15).

**Fotos** pelo [`fotografa_cena.sh`](../ferramentas/fotografa_cena.sh) (ecrã virtual): `=3` e `=4`
com nada debaixo da coluna dos avisos.

| `typos` **project-wide** (como o `ship.sh`) | limpo — ⚠️ **depois de duas entradas no `.typos.toml`**: a 1.ª corrida acusou `resistencias` (pt, 20 sítios — isentada como palavra, no idioma do ficheiro) e `conver` na saída verbatim do oráculo Godot (`conver_type` é a grafia do addon — isentado o **ficheiro**, nunca a palavra) |

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `cargo deny`, `cargo audit` (a linha
não acrescenta pacote externo nenhum).

---

## §6 — Os smokes (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=3 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=4 cargo run -p ph2d-host-desktop --profile smoke
```

A `=3` é a dos tipos de dano (tecla **Q** fogo com queimadura, **J** gelo; Salamandra imune ao fogo e
fraca ao gelo, o CONTROLO sem resistências, o Elemental que absorve o fogo). A `=4` é a arena
(Salamandra, morcegos que perseguem, lava, coração, e o recomeço quando o herói cai). As `=1`/`=2`
continuam (§6 do handoff anterior), agora com a bala visível. ✅ **Aprovados pelo dono:** W6 (`=3`) e
W7 (`=4`, 2026-09-30).

**Linha proposta para o `CLAUDE.md §5`** (Componentes, a seguir à entrada da VIDA E DANO — quem a
aplica é o integrador):

> ⭐⭐⭐ **E A VIDA E DANO FECHOU (30/09, plano 28 W6–W7 — [handoff](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_VIDA_W6_W7_2026-09-30.md)):** TIPOS de dano com resistências (imune · fraco · absorve, contra o addon MIT do Godot) e o dano que DURA; a arena `=4` junta tudo num jogo pequeno e o tutorial [`02_vida_e_dano.pdf`](docs/Components/tutoriais/02_vida_e_dano.pdf) ensina-o; ⚠️ **conte o DELTA:** `PROJECT_SCHEMA` **+1**, registos **0**, catracas do Inspector `+4/+2/+2`; ⏳ a lava renovar a queimadura a cada tique é decisão do dono. **Smokes:** `PH2D_VIDA_SMOKE=3|4`.
