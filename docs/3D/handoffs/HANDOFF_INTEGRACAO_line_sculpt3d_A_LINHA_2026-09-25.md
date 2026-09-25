# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, a LINHA inteira (2026-09-25)

> **Para o agente INTEGRADOR.** Ordem do dono, 2026-09-25: *«smoke OK. Antes de
> seguir vamos escrever handoff para outro agente integrar a linha ao main»*.
> Este documento é a superfície de colisão **medida**, não um resumo do que a
> linha fez — o mecanismo de cada wave vive nos handoffs da jornada (índice no
> §5) e a entrada do roteador está no `CLAUDE.md` §5.
>
> ⛔ **A linha NÃO integrou nem pushou nada.** Ela está fechada, rebaseada sobre o
> `main` de hoje, commitada e parada. O HEAD do CÓDIGO é `179cd1521`; o commit
> deste handoff (e da linha do roteador) vem por cima e não toca em código.

---

## §1 — O que é, em números

| grandeza | valor |
|---|---|
| ramo | `line/sculpt3d`, worktree `Worktrees/line-sculpt3d` |
| HEAD do código | `179cd1521` (+ o commit de docs deste handoff) |
| merge-base com o `main` | `20a630f1b` — **é o `main` de hoje**: a linha foi rebaseada em 2026-09-25, sem conflito |
| commits da linha | **74** de código/docs + **1** deste handoff |
| ficheiros tocados | **269** (`+40 641` / `−798`) |
| desde a integração anterior | a de 2026-09-20 ([`A_LINHA_2026-09-20`](HANDOFF_INTEGRACAO_line_sculpt3d_A_LINHA_2026-09-20.md)) |

⇒ **o `--ff-only` é possível agora**, desde que o `main` não ande até lá — e há
**uma** condição de ambiente que o impede, no §3.1. Leia-o antes do merge.

---

## §2 — A superfície de colisão, MEDIDA

Corrida com o caminho **absoluto do primário**
(`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`), depois
do rebase.

### §2.1 — Contadores partilhados: **ZERO se mexem**

| contador | a linha | o `main` de hoje (lido no FICHEIRO) |
|---|---|---|
| `PROJECT_SCHEMA` | `160` | `160` (`project_schema.rs:408`) |
| tripla do gate | `(160, 13, 22)` | `(160, 13, 22)` |
| `VEC_SCENE_SCHEMA` | `22` | `22` |
| `FLIP_SCHEMA` | `13` | `13` |
| `DOC_VERSION` (timeline) | `18` | `18` |
| `FIELD_DOC_VERSION` | `23` | `23` |
| registo `ph2d-ecs` | `103` | `103` |
| espelho `ph2d-render` | `104` | `104` |
| espelho `ph2d-script` | `104` | `104` |

⚠️ **O `SCULPT_DOC_VERSION` muda (`1 → 3`) e NÃO é um contador partilhado:** ele
vive **dentro** do blob da escultura (`crates/ph2d-app-sculpt3d/src/doc.rs`), é
escrito só por esta família, e cada degrau tem **migração** (`V_ANTES_DA_TINTA = 1`
· `V_ANTES_DA_GRADUACAO = 2`) — um `.ph2dproj` antigo abre. O `PROJECT_SCHEMA`
não sobe por causa dele, e é a mesma lei que deixa o `TimelineDoc` evoluir
sozinho. Nenhuma outra linha escreve naquele ficheiro.

⚠️ O script avisa *«esta linha TOCA project\*.rs»* — é
`shells/desktop/src/project_sculpt_tests.rs`, um **teste** (o golden do
documento: `1538 → 1539` bytes, a conta fecha à mão no próprio gate). Nenhum
degrau da escada nem da tripla foi escrito.

### §2.2 — Contrato congelado (§6): **intocado**

```
crates/ph2d-nodegraph/src/node.rs      intocado
crates/ph2d-editor-core/src/tool.rs    intocado
```

O `PainterTool` ganhou métodos **inerentes** (a tela da vista); `Tool=12` e o
`CanvasPaintTool` não se mexem.

### §2.3 — ADR: **nenhum**

Último no disco `0170`, próximo livre `0171` ⇒ fora de toda disputa de número.

### §2.4 — `Cargo.lock`: **duas crates novas, as duas INTERNAS; zero pacote externo novo**

- `ph2d-mesh-colors` — a lei da **tinta fina** (clean-room de dois papers
  públicos, Yuksel *Mesh Colors*), **zero dependências**.
- `ph2d-uv-atlas` — o atlas (depende só de `ph2d-mesh` e `ph2d-gridmap`).

A família `ph2d-app-sculpt3d` passa a depender de `image` (só `png`,
`default-features = false`) — **o pacote já estava no lock** (a shell usa-o), logo
`cargo deny`/`audit` não ganham sujeito. ⚠️ **Aresta interna nova no grafo:**
`ph2d-app-sculpt3d → ph2d-tool-painter` (família → ferramenta, que a régua das
camadas permite; o `architecture_no_dependency_climbs_a_layer` passou).

### §2.5 — Tectos de LOC

Nenhum ficheiro que a linha tocou passa do tecto. ⚠️ **Isto não fecha a
ACUMULAÇÃO** — ver §4.2.

---

## §3 — Onde a fusão pode doer

### §3.1 — ⛔⛔ O PRIMÁRIO está SUJO, e três dos ficheiros sujos são desta linha

Os commits do `main` e os da linha **não se sobrepõem** (o único commit novo do
`main`, `20a630f1b`, é `memoria(cascadeur)` e toca só
`project_teste_cascadeur_2d_bones_testbed.md`). **Mas a ÁRVORE DE TRABALHO do
primário não está limpa** — medido em 2026-09-25:

```
 M project-memory/MEMORY.md                                   ← a linha também muda
 M project-memory/reference_topic_measurement_discipline.md   ← a linha também muda
 M project-memory/reference_topic_mutation_proofs.md          ← a linha também muda
 M project-memory/feedback_a_pgrep_watcher_catches_its_own_shell.md
 M project-memory/feedback_communication_style.md
 M project-memory/reference_manual_apps_in_home_apps_are_invisible_to_cachy_update.md
 M project-memory/reference_topic_architecture_north_perf_lessons.md
 M project-memory/reference_topic_control_design_hazards.md
 M project-memory/reference_topic_gate_discipline.md
?? project-memory/feedback_a_per_texel_albedo_is_the_base_color_never_a_factor_at_the_end.md
?? project-memory/feedback_a_picker_swatch_emits_no_event_and_a_click_arm_for_it_is_dead_code.md
?? project-memory/feedback_every_guard_written_with_lt_or_gt_is_blind_to_nan.md
?? project-memory/feedback_forcing_a_property_after_the_optimum_is_what_ripples.md
?? project-memory/reference_magicacsg_installed_under_wine.md
?? project-memory/reference_vector_bones_research_corpus_2026_09_23.md
```

⚠️ **A causa é estrutural e não é desta linha:** o symlink
`~/.claude/projects/<key>/memory` aponta para o `project-memory/` do **primário**,
logo toda memória que QUALQUER sessão escreve pela ferramenta aterra lá, por
commitar — de várias frentes (o índice tem entradas do Cascadeur, do MagicaCSG,
dos ossos do vetor…). ⇒ **o `git merge --ff-only line/sculpt3d` RECUSA** («your
local changes would be overwritten») nos três ficheiros marcados.

⛔ **NÃO resolva com `git stash` nu nem com `checkout --`** — o conteúdo sujo é
trabalho de outras sessões, e o stash é partilhado por todas as worktrees. O
caminho que preserva as duas metades:

```bash
cd /home/enio/Documentos/Projetos/PH2D
F="project-memory/MEMORY.md project-memory/reference_topic_measurement_discipline.md project-memory/reference_topic_mutation_proofs.md"
git stash push -m "integ-sculpt3d-2026-09-25-memoria" -- $F
SHA=$(git stash list --format='%H %gs' | grep integ-sculpt3d-2026-09-25-memoria | cut -d' ' -f1)
git merge --ff-only line/sculpt3d
git stash apply "$SHA"      # conflito textual esperado nos três — ver abaixo
```

Os conflitos são **de adição dos dois lados** (medido com `diff`: a linha
acrescentou entradas e secções no fim; o primário acrescentou outras nos mesmos
sítios). A cura é **manter as duas metades, nunca escolher uma**; depois de
resolvido, **dropar a entrada do stash pelo tag** e deixar o resultado **SUJO,
como estava** — commitar a memória das outras sessões não é trabalho desta
integração. ⚠️ O `MEMORY.md` tem um tecto em BYTES declarado no próprio
cabeçalho (`≤ 22 KB`, o carregador corta em `~26 KB`): a união das duas metades
pode passá-lo — **não compacte na integração** (é trabalho do dono da memória e
faria um diff de fusão impossível de auditar); anote-o.

### §3.2 — Mudanças de PRODUTO em crates partilhadas (todas aditivas)

| ficheiro | o quê | porquê é seguro |
|---|---|---|
| `ph2d-gpu/src/context.rs` | pede `wgpu::Features::PRIMITIVE_INDEX` | entra pela **mesma intersecção** com o que o adaptador anuncia das outras features ⇒ o `request_device` **não pode falhar** por ela; onde falta, a fonte do shader sai sem o bloco que a usa e a peça desenha com a cor por vértice |
| `ph2d-app-host/src/modal.rs` | porta nova `pick_files` (plural) | append-only; irmã do `pick_file` que já declarava a paragem do laço |
| `ph2d-editor-core/src/toast.rs` + `progress.rs` | portas `text_budget_px()` e `toast_column_w()` | append-only; um `debug_assert` ata o pintor do balão à porta |
| `ph2d-mesh` (`export.rs`, `read.rs`, `mesh.rs`, `mesh_area.rs`, `mesh_indices.rs`, `collapse.rs`) | a 4.ª cláusula do `lost_by` (tinta fina), `MeshFormat::keeps_fine_paint`, o `.obj` com `vt` + `.mtl`, a área e os índices como portas | o caminho **sem** textura escreve o `.obj` **byte a byte** como antes (o escritor antigo DELEGA no irmão com uv) |
| `ph2d-tool-painter` (`screen_canvas.rs` novo + 9 ficheiros) | o Painter pinta uma **tela do tamanho da vista** (o Painter na peça 3D) e diz se a água ainda escorre | métodos **inerentes** — `Tool=12`/`CanvasPaintTool=1` intocados; a sprite 2D nunca arma a tela da vista |
| `ph2d-app-field3d/src/export.rs` | a exportação da modelação passa a perguntar a mesma lei partilhada | passa `false` à cláusula nova, por medição (a modelação não tem tinta fina) |
| `ph2d-i18n` (`app_sculpt3d.rs`, `sculpt3d.rs`) | chaves novas | só acréscimos nas tabelas da família |

### §3.3 — ⚠️ A shell: desta vez há **PRODUTO** (pouco)

```
shells/desktop/src/input_dispatch.rs                        +1
shells/desktop/src/input_dispatch/painter_canvas_input.rs  +14 −34
shells/desktop/src/input_dispatch/painter_canvas_keys.rs   +47
shells/desktop/src/render_loop/fase_painter_dispatch.rs    +10
shells/desktop/src/sculpt3d_host.rs                         +9
(+ testes: project_sculpt_tests.rs, tests/it/the_dynamic_topology_is_wired.rs,
   tests/it/the_sculpt_gesture_is_wired.rs)
```

São os **três elos** do Painter na peça (o ponteiro · o teclado · o quadro), e o
botão esquerdo deixa de ir para a navegação quando o Painter está sobre a peça.
Saldo **+52 linhas** de produto — ⚠️ **a catraca `the_shell_only_shrinks` passou
no portão desta árvore**, mas a folga dela é propriedade da SOMA: se outra linha
fundir antes e somar na shell, ela pode acender aqui. A cura então é **mover para
a crate da família**, nunca subir o número (`CLAUDE.md` §2).

### §3.4 — O resto do diff, por área

`ph2d-app-sculpt3d` (92) · `docs/3D` (26) · `ph2d-mesh-render` (20) ·
`ph2d-sculpt3d` (19) · `ph2d-panel-sculpt3d` (18) · `ph2d-mesh-colors` (17, nova) ·
`ph2d-uv-atlas` (15, nova) · `ph2d-tool-painter` (12) · `ph2d-mesh` (12) ·
`shells/desktop` (8) · `ph2d-quadchain` (4, só um `example` e dev-deps) ·
`ph2d-editor-core` (3) · `ph2d-i18n` (2) · `ph2d-gpu` (2) · `ph2d-app-field3d` (2) ·
`ph2d-app-host` (1) · `project-memory` (14) · `CLAUDE.md` · `Cargo.lock`.

---

## §4 — A prova de fecho (sobre o HEAD do código `179cd1521`, depois do rebase)

| portão | resultado |
|---|---|
| `cargo fmt --all -- --check` | limpo |
| `typos` | limpo |
| `cargo machete` | limpo |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --features bevy_ecs --locked` (o passo do CI) | verde |
| `scripts/check-standalone-optional.sh` · `scripts/check-workflow-packages.sh` | verdes |
| `cargo clippy --workspace --all-targets -- -D warnings` | **zero** |
| `bash scripts/nextest-impacted.sh` | **18 691 / 18 691** |
| `bash scripts/censos-da-arvore-combinada.sh` | **127 / 127** (controlo do filtro: 12 de 12 censos correram) |
| GPU com adaptador (`--ignored`) | **80/80** + **33/33** (§4.3) |
| pré-voo das **13** arneses de mutação da linha | **243 / 243** âncoras casam |

⚠️ **O portão apanhou, e a linha curou, o que só o `ship.sh` pegaria:** o
`rustfmt` de dois ficheiros da `ph2d-mesh-colors` e quatro palavras portuguesas
que o `typos` lê como inglês (commit `179cd1521`, zero linhas de produto).
⚠️ **Nenhum `#[cfg(target_os` foi escrito nem movido** por esta linha ⇒ a classe
que só o CI de macOS/Windows vê (`DIRETRIZ` §1.5.9 item 5) não tem sujeito aqui.

### §4.1 — As vassouras da parede clean-room

A linha não ganhou alvo restrito novo: o Painter é código **desta casa**, a tinta
fina é clean-room de **papers públicos** (a triagem e a medição estão em
[`27_o_estado_da_arte_de_onde_a_tinta_mora.md`](../27_o_estado_da_arte_de_onde_a_tinta_mora.md)),
e o Blender foi **corrido** sobre um `.obj` NOSSO (a saída é livre, §0.9), nunca
lido. Os achados das vassouras sobre as crates da família continuam a ser **do
`R`**, como nos fechos anteriores.

### §4.2 — O que SÓ a árvore combinada pode reprovar

1. **Censos de texto (HR-15) e tectos de LOC** são propriedades da SOMA: os
   `127/127` medem esta linha contra o `main` de HOJE. Se o integrador fundir
   **outra** linha antes, corra o `censos-da-arvore-combinada.sh` outra vez.
2. **A catraca da shell** (§3.3).
3. **Os arch-gates em `tests/it/` de outras crates** — alcançados pelo
   `nextest-impacted` por `rdeps`, e é essa a corrida da tabela.

### §4.3 — GPU

As suítes `#[ignore]` que só uma placa corre (o CI **não** as corre):
`ph2d-mesh-render` inteira e os gates de produto da tinta e do Painter na peça
(`ph2d-app-sculpt3d … tinta`). Corrida de fecho, **com adaptador**, sobre o HEAD
do código:

| suíte | resultado |
|---|---|
| `cargo test -p ph2d-mesh-render -- --ignored --test-threads=1` | **80 / 80** |
| `cargo test -p ph2d-app-sculpt3d --lib tinta -- --ignored --test-threads=1` (a tinta fina + o Painter na peça, incluída a água que escorre) | **33 / 33** |

⚠️ A placa estava com outra linha quando a corrida foi pedida; o arnês esperou pela
porta com prazo — ⛔ ela nunca se força.

---

## §5 — O que a linha entrega (índice, para o commit de fusão)

Por ordem de chegada, com o handoff da jornada que tem o mecanismo:

| quando | o que fechou | onde ler |
|---|---|---|
| 20/09 | as **manchas pretas** eram `NaN` (a guarda existia numa só das duas curvas) | [`MANCHAS_PRETAS`](HANDOFF_INTEGRACAO_line_sculpt3d_MANCHAS_PRETAS_2026-09-20.md) |
| 20/09 | a cor do pincel é uma **caixa** (o selector da casa já existia) | [`CAIXA_DE_COR`](HANDOFF_INTEGRACAO_line_sculpt3d_CAIXA_DE_COR_2026-09-20.md) |
| 20/09 | a avaliação de pintar com o Painter na malha, o atlas (W0–W5) e a pesquisa de onde a tinta mora | [doc 25](../25_avaliacao_o_painter_na_malha.md) · [26](../26_a_parametrizacao_como_atlas.md) · [27](../27_o_estado_da_arte_de_onde_a_tinta_mora.md) |
| 20/09 | a **tinta fina** (P1–P2c): a tinta deixa de ter a resolução da malha — lei, gémeo WGSL, fileira `Paint Detail`, cena `=52` | [`A_TINTA_FINA_VISIVEL`](HANDOFF_line_sculpt3d_A_TINTA_FINA_VISIVEL_2026-09-20.md) |
| 21–23/09 | a tinta fina **sobrevive** (ao traço seguinte, ao clique fora, ao `Ctrl+Z`, ao pânico do registo), **viaja no `.ph2dproj`**, **sai no `.obj`** (um ladrilho por face), e o **plano graduado** (o nível é da face) | [`A_TINTA_FINA_SOBREVIVE`](HANDOFF_line_sculpt3d_A_TINTA_FINA_SOBREVIVE_2026-09-21.md) §1–§33 |
| 23/09 | o **`Fill Piece`** — a peça inteira com a cor do pincel, num `Ctrl+Z` | [`O_FILL`](HANDOFF_line_sculpt3d_O_FILL_2026-09-24.md) |
| 24/09 | **o Painter pinta a peça 3D**, etapa 1 (o `Digital`) | [`O_PAINTER_NA_PECA`](HANDOFF_line_sculpt3d_O_PAINTER_NA_PECA_2026-09-24.md) |
| 24/09 | etapa 2 (os modos que lêem a peça: borrar, esfumar, clonar, liquify, balde, inpaint, aquarela) + os degraus `32x`…`256x` | [`ETAPA_2`](HANDOFF_line_sculpt3d_O_PAINTER_NA_PECA_ETAPA_2_2026-09-24.md) §1–§8 |
| 24–25/09 | etapa 3a: a **tinta molhada escorre** depois de largar, num passo só de desfazer; trocar a cor e dar o 2.º traço **não a secam** | [`ETAPA_2`](HANDOFF_line_sculpt3d_O_PAINTER_NA_PECA_ETAPA_2_2026-09-24.md) §9–§9.6 |

---

## §6 — Os smokes, para o dono correr DEPOIS da fusão

⚠️ Depois de integrar, o caminho é o do **primário**, não o da worktree:

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_SCULPT3D_SMOKE=52 cargo run -p ph2d-host-desktop --profile smoke
```

- **`=52`** — a tinta fina (a fileira `Paint Detail`, gravar e reabrir, exportar
  o `.obj`) **e** o Painter na peça (`IMG` na barra de cima → `PNTR` no rail;
  `Wet Paint` escorre depois de largar). ✅ **Aprovado pelo dono** — o último smoke
  foi o do 2.º traço, em 25/09.
- **`=51`** — os três pincéis de cor da escultura e a caixa de cor. ✅ Aprovado.

---

## §7 — ⏳ ABERTO

- **Etapa 3b — o relevo do IMPASTO na peça.** Decisão do dono já tomada
  (AskUserQuestion de 24/09): *«Relevo de LUZ»* — a tinta ganha espessura que pega
  luz e sombra, **a forma da peça não muda**, desfaz-se com `Ctrl+Z` e não entra na
  silhueta nem no ficheiro exportado — e *«a luz usada é a luz do cenário 3d»*.
  Só investigado: o composto do Painter sai **aceso** pela luz 2D dele
  (`impasto_visible()`), logo na tela da vista a drenagem tem de sair **sem** luz e
  a altura viajar à parte. É o próximo passo desta linha.
- **Etapa 4 — camadas e efeitos** do Painter na peça. Espera o dono.
- Limites **declarados** do Painter na peça (ETAPA_2 §6 e §9.4): os modos agem à
  resolução do ECRÃ; outras peças não tapam; a borracha não faz nada na peça (o
  que «apagar» quer dizer é pergunta do dono); rodar a vista seca a aquarela e a
  tinta molhada; uma pincelada que escorre fecha em qualquer GESTO da escultura
  (tecla viva, gesto do painel, clique, desfazer).
- `Fill Piece`: a força do pincel não entra (decisão do dono) e o relógio a `16x`
  não foi varrido.

---

## §8 — Seis coisas que uma leitura rápida do diff entende ao contrário

1. **O `SCULPT_DOC_VERSION 1 → 3` não é um contador partilhado** e não mexe no
   `PROJECT_SCHEMA` (§2.1) — e tem migração nos dois degraus.
2. **O `PRIMITIVE_INDEX` do `ph2d-gpu` não pode derrubar o `request_device`**:
   ele entra pela intersecção com o que a placa anuncia (§3.2).
3. **O Painter não passou a conhecer malhas.** Ele pinta uma imagem do tamanho da
   vista, e é a escultura que a pousa na peça (`ph2d_sculpt3d::tela_na_malha`) —
   por isso o contrato `Tool` fica intocado.
4. **As três linhas sujas de `project-memory/` no primário não são conflito de
   git entre ramos**: são memória por commitar de várias sessões (§3.1), e a
   integração preserva-as.
5. **Os `+52` da shell são costura, não família**: o ponteiro, o teclado e o
   quadro do Painter sobre a peça — a lei vive na `ph2d-app-sculpt3d`.
6. **`ph2d-quadchain` só ganhou um `example`** (`atlas_probe`) e dev-deps: a lib
   da cadeia não conhece o atlas.

---

## §9 — O que este handoff NÃO autoriza

⛔ **Ship.** O `git push` é do dono (`CLAUDE.md` §0.7), e integrar não é aprovar.
⛔ **Commitar a memória suja do primário** (§3.1) — ela é de outras sessões.
