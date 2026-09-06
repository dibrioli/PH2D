# HANDOFF DE INTEGRAÇÃO — `line/components` · **a F5 FECHA** (2026-09-06)

> **Entregável de fecho** (DIRETRIZ §1.5.9). ⛔ A linha **não integra e não faz ship** — ela fecha,
> entrega isto e espera ordem explícita do Enio (`CLAUDE.md` §0.7).

---

## 1. Identidade

| | |
|---|---|
| branch | `line/components` |
| HEAD | `7fd1d2bc4` |
| merge-base com `main` | `53832c884` |
| commits | **25** |
| ficheiros | **91** (`+10 588` / `−238`) |

⚠️ **O HEAD muda se o Enio pedir mais uma wave antes de mandar integrar.** O integrador re-lê o HEAD
e **re-roda a `collision-surface.sh`** (§1.5.3), sempre.

---

## 2. O que a linha entregou (uma linha por wave)

| wave | o quê | onde ler |
|---|---|---|
| **F5.5** | A **escada do *Aplicar*** — o critério 4: uma excepção pode subir à receita **interna** | §F5.5 do plano |
| **F5.6** | As excepções sem alvo dizem **QUAIS** (critério 3) — o nome da peça morta viaja | §F5.6 |
| **F5.7** | Apagar uma peça de uma cópia deixou de ser o pior dos três resultados (recusa com voz) | §F5.7 |
| **F7.1** | A **biblioteca ganhou porta**: *Edit Prefab* — a receita abre-se seleccionando-a | §F7.1 |
| **F5.8** | **Trocar por um componente SEM PARENTESCO** — os 3 modos + o relatório · **a F5 fecha** | §F5.8 |
| **F5.9** | A lista de órfãos fica **accionável** (um `✕` por linha) + a auditoria do cartão | §F5.9 |
| **F5.10** | **A peça RECUSADA** (*Removed GameObject*) — `PROJECT_SCHEMA` **115 → 116** | §F5.10 |
| **F5.11** | **A peça ACRESCENTADA** (*Added GameObject*) — **derivada**, schema intocado | §F5.11 |
| **F5.12** | **Mover uma peça na receita move-a em TODAS as cópias** — a 3.ª metade da forma | §F5.12 |
| **F5.13/14** | As cenas de smoke corrigidas: **3 passos impossíveis** + 1 que pedia o gesto que a guarda não apanha | §F5.13, §F5.14 |

⇒ **a F5 não tem mais nada aberto.** O que fica da fase F4 e da F8 está na §7.

---

## 3. Foundational / partilhado tocado, e porquê

| onde | o quê | aditivo? |
|---|---|---|
| **`crates/ph2d-ecs/src/instantiate.rs`** | `ObjectInstance.removed: BTreeSet<u64>` (F5.10) · `OrphanOverride.piece_name` (F5.6) | ⛔ **não aditivo** — muda bytes de um componente ⇒ o degrau de schema |
| `crates/ph2d-ecs/src/lib.rs` | re-export do `OrphanOverride` | ✅ aditivo |
| `crates/ph2d-editor-core/src/ids/` | **51 `const` novos** (3 de menu + 3 tabelas de 16) + 3 portas inversas | ✅ aditivo |
| `crates/ph2d-editor-core/src/action_bus{,_kinds}.rs` | **4 variantes** de `EditorAction` + **4** de `AssetCardAction` | ⚠️ enum partilhado — ver §4 |
| `crates/ph2d-editor-core/src/screens/hero/` | `inspector_model_instance` (3 tipos de linha novos) · `menu_rows` · `pre_populate` · `menu_bar` | ✅ aditivo |
| `crates/ph2d-panel-inspector/` | 3 blocos novos do cartão + `event_instance.rs` + `populate_instance.rs` | ✅ aditivo (2 são **cortes de tecto de LOC**) |
| `crates/ph2d-panel-asset-browser/src/event.rs` | o mapeamento dos 4 verbos do cartão | ✅ aditivo |
| `shells/desktop/` | o grosso da linha (36 ficheiros em `src/`, 7 em `tests/`) | ⚠️ |

⛔ **Contratos congelados (§6): INTOCADOS** — a sonda confirma (`node.rs`, `tool.rs`).
⛔ **ADR: esta linha não cria nenhum** ⇒ fora de toda disputa de número.
⛔ **Registo de componentes: NÃO se mexe** — os dois espelhos ficam em **80** (base 80). O `removed`
vive **dentro** do `ObjectInstance`, e a peça acrescentada é a **ausência** de um elo.

### 3.1 ⚠️ Os cortes de tecto de LOC — onde o integrador vai ver «ficheiro novo» e é MOVE

| ficheiro novo | saiu de | motivo |
|---|---|---|
| `crates/ph2d-panel-inspector/src/sections/instance_orphans.rs` | `sections/instance.rs` | fn 244/200 |
| `crates/ph2d-panel-inspector/src/event_instance.rs` | `src/event.rs` | ficheiro 612/600 |
| `shells/desktop/src/render_loop/hierarchy_delete.rs` | `render_loop/hierarchy.rs` | ficheiro 618/600 |
| `shells/desktop/src/instance_refuse_tests.rs` | `instance_structure_tests.rs` | ficheiro 699/600 |

*Um corte por tecto lê-se no diff como código novo + código apagado; ele não é nenhuma das duas
coisas.*

---

## 4. Símbolos que podem COLIDIR — a saída da sonda, colada

⚠️ **Referência, nunca evidência** (§1.5.9 item 3): a tabela mede a linha contra o `main` **de
hoje**. Se outra linha fundir no meio, todo número da coluna «base» muda e **este documento não
reclama**. ⇒ o integrador **re-roda**.

```text
SUPERFÍCIE DE COLISÃO — line/components contra main
  merge-base 53832c884   ·   25 commit(s)   ·   91 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        116   (base: 114)
  ⚠   └ tripla do gate               (116, 13, 18)   (base: (114, 13, 18))
    VEC_SCENE_SCHEMA                       18   (base: 18)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-render (espelho)                  80   (base: 80)
    ph2d-script (espelho)                  80   (base: 80)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0168   próximo livre: 0169
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
```

### 4.1 `PROJECT_SCHEMA` — **os DOIS degraus são desta linha**

`114 → 115` (F5.6, o nome da peça órfã) e `115 → 116` (F5.10, o `removed`). ⚠️ **Se outra linha
tiver escrito um degrau no meio, o valor certo não é 116 nem o dela** — conta-se, nos **três**
sítios: [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) (a constante **e** a
escada) + a tripla em [`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs).

⚠️⚠️ **A tripla é CEGA a mudanças dentro de um `ComponentBlob`** — é a **quinta** vez que esta linha
o regista (99, 100, 114, 115, 116). O postcard é posicional: um campo novo no fim de um componente
registado é quebra dura, e nada mecânico o acusa.

### 4.2 Ids, `const` e variantes novos (o que o integrador grepa)

| família | valores |
|---|---|
| menu do cartão de asset | `CTX_MENU_ASSET_EDIT` · `CTX_MENU_ASSET_REPLACE` · `_BY_NAME` · `_BY_TREE` |
| tabelas de ids do cartão | `INSP_INSTANCE_APPLY_LEVEL[8]` · `INSP_INSTANCE_DROP_ORPHAN[16]` · `INSP_INSTANCE_RESTORE_PIECE[16]` · `INSP_INSTANCE_APPLY_ADDED[16]` |
| tectos | `MAX_INSTANCE_APPLY_LEVELS = 8` · `MAX_INSTANCE_ORPHAN_ROWS = 16` · `MAX_INSTANCE_REMOVED_ROWS = 16` · `MAX_INSTANCE_ADDED_ROWS = 16` |
| `EditorAction` | `InspectorApplyToLevel` · `InspectorDropUnusedOverride` · `InspectorRestoreRemovedPiece` · `InspectorApplyAddedPiece` |
| `AssetCardAction` | `EditPrefab` · `ReplaceSelection` · `ReplaceSelectionByName` · `ReplaceSelectionByTree` |
| tipos novos no modelo do cartão | `OrphanRow{piece_id,type_id}` · `RemovedRow` · `AddedRow` · `ApplyChoice` |
| campo novo | `StructureReport::moved` |

⚠️ **`InspectorInstanceInfo` ganhou 5 campos** (`apply_levels`, `apply_levels_beyond`,
`orphan_rows`, `removed_rows`, `added_rows`). Ele é construído por **literal** em 3 sítios de teste
de outra crate — o compilador aponta-os, que é o que se quer, mas o integrador vê-os como conflito
se outra linha também lá mexer.

---

## 5. O que só o `ship.sh` apanha (o gate de integração NÃO roda)

- **`fmt`/`typos` pré-fork** — a linha corre `cargo fmt --all -- --check` a cada fecho e está limpa
  **na sua árvore**; a soma com outra linha pode não estar.
- **`machete`** — ⛔ **nenhuma dependência nova** nesta linha (o `Cargo.lock` não ganhou pacote
  externo). Nada a declarar.
- **`clippy` latente** — limpo em `ph2d-host-desktop`, `ph2d-editor-core`, `ph2d-panel-inspector`
  (`--all-targets`).
- **RUSTSEC** — sem `cargo audit` nesta linha.

### 5.1 ⚠️ Os TRÊS gates que só a árvore COMBINADA pode reprovar

1. **Tectos de LOC por SOMA** — `event.rs`, `sections/instance.rs`, `render_loop/hierarchy.rs` e
   `instance_structure_tests.rs` foram cortados **até caberem**; uma linha que acrescente ao mesmo
   ficheiro estoura outra vez. ⛔ **Paga-se com CORTE, nunca com allowlist.**
2. **`hit_indexed_ids_are_registered` + `table_driven_chips_are_registered_too`** — as **três**
   tabelas de ids novas estão registadas em `populate_instance.rs`. Uma fusão que perca esse
   ficheiro deixa 48 botões **mortos sob o dedo**, e só o segundo gate o vê.
3. **`the_menu_bar_relocates_the_verbs_it_shows`** e os censos de dreno
   (`the_added_piece_gesture_reaches_the_verb`, `the_unused_override_gestures_reach_the_verb`,
   `moving_a_piece_of_a_copy_goes_through_the_recipe_door`,
   `deleting_a_piece_of_a_copy_goes_through_the_recipe_door`) — todos **textuais**, ancorados em
   nomes de função e de ficheiro. *Um corte por tecto noutra linha move o sujeito e eles reprovam —
   o que é o desenho: a lei mudou de morada e o censo tem de a seguir.*

---

## 6. Ordem, dependências e **o que smoke-testar**

⛔ **Não há ordem entre commits além da cronológica** — cada wave compila e passa sozinha, e todas
partilham `shells/desktop/src/instance_*`.

### 6.1 Smokes que o Enio já correu e aprovou

| cena | o que ela ensina | veredito |
|---|---|---|
| `PH2D_INSTANCE_SMOKE=3` | a receita dentro da receita + a escada do *Aplicar* + os órfãos | ✅ |
| `=4` | trocar por um componente **sem parentesco** (os 3 modos) | ✅ |
| `=5` | **o que é só desta cópia** — arrastar · recusar a peça · *Put back* | ✅ |
| `=6` | **dar uma peça ao componente** — duplicar · aplicar | ✅ |
| `=7` | **mover uma peça no componente** — as três cópias seguem · e a recusa | ✅ |

Comando (⚠️ o `cd` é o da **worktree**):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_INSTANCE_SMOKE=7 cargo run -p ph2d-host-desktop --release
```

### 6.2 ⚠️ O que NÃO foi smokado

- **Abrir um `.ph2dproj` gravado antes desta linha** — ⛔ não é executável: `find /home/enio -name
  '*.ph2dproj'` devolve **zero**. A migração está provada pelo gate `the_frozen_v95_bytes_still_load`,
  que **constrói** os bytes.
- **O `physics_ecs_c9` cross-OS** — só o CI o mede (ele compara os três SO entre si).
- **A troca sem parentesco em `ByHierarchy` sobre uma árvore com mais de 2 níveis** — os gates
  cobrem-na; a mão do Enio só viu a fixtura de 2.

---

## 7. O que fica ABERTO nesta linha (auditado contra o código, 2026-09-06)

| item | estado | o gatilho |
|---|---|---|
| **F4.6c** — matar o `VecInstance` (~2 961 LOC, 24 ficheiros) | ⬜ **bloqueada na PRÁTICA** | a `line/Vector` está **VIVA** (`git worktree list`); apagar 24 ficheiros que ela edita é catástrofe de merge. ⇒ correr **logo depois** de aquela linha integrar, ou pelo integrador. ⚠️ A fatia que a bloqueava (os eixos de propriedade) **deixou de existir** — o Enio revogou-os em 01/09 |
| **F8 — `FlipDoc` partilhado entre passos** | ⬜ | ⛔ **headless**: não há frase *«faça X e veja Y»*, e a regra do Enio é que cada etapa acabe num smoke. A irmã (`VecScene`) fechou em 02/09 e a medição está no §F8.0 |
| A cópia **arrastada para dentro de outra cópia** não entra na lista de acrescentadas | ⏳ **fronteira nomeada** | ela tem elo e é a raiz da cópia dela; promovê-la é criar receita aninhada — território do *Make Component* (§F5.11) |
| O recuo horizontal do cartão sai do orçamento de quebra | ⏳ **dívida declarada** | falta instrumento: o `MockPanelHost` não expõe extensão de glifos (§F5.9) |
| A guarda `if mine == root` do passe é **redundante** | ⏳ **cerca declarada** | a mutação disse-o; fica por ser onde alguém leria a lei (§F5.12) |

---

## 8. ⚠️ Sete coisas que uma leitura rápida do diff entende ao contrário

1. **`instance_added.rs` não guarda nada.** A peça acrescentada **é** a ausência de `InstanceOf`,
   e essa ausência já era load-bearing desde a F4.2 (o passe não lhe toca; o apagar deixa-a morrer).
   Um campo aqui seria uma segunda fonte.
2. **`promote_piece` não é `materialise_piece` ao contrário por acaso** — o elo nasce no
   **original**, e é isso que impede a peça de aparecer **duas vezes** na cópia onde o artista
   trabalhou.
3. **O `remove::<InstanceOf>` no `duplicate_subtree` NÃO é incondicional.** Duplicar a **raiz** de
   uma cópia tem de continuar a dar uma segunda cópia; a porta estreita (`is_a_recipe_given_piece`)
   é o que os separa, e a largura dela é load-bearing.
4. **`ChildOf` não é componente registado**, e é por isso que o bloco novo do `reconcile_one` é
   obrigatório: a árvore de uma cópia **não tinha dono** até a F5.12.
5. **A ordem da travessia no bloco novo NÃO é o que impede um ciclo** — a mutação que a inverte
   sobreviveu. O que compra a segurança é *recolher e depois aplicar*. ⛔ Quem trocar isso por
   *aplicar enquanto percorre* reabre a pergunta.
6. **A guarda do arrasto só recusa quando o PAI muda.** Reordenar entre irmãos é excepção legítima
   da cópia (a ordem viaja no `SiblingOrder`, que **é** registado) — com gate.
7. **`is_unedited_recipe` esconde a receita INTEIRA da Hierarquia**, raiz incluída (o `MasterRoot`
   também é `MasterPiece`). Três cenas de smoke mandavam clicar em linhas que não existem.

---

## 9. Cinco premissas MINHAS que a medição derrubou nesta linha

1. *«a pré-ordem do mestre impede o ciclo»* — **falso** (§F5.12).
2. *«a guarda da raiz é load-bearing»* — **redundante** (§F5.12).
3. *«a nota da altura do cartão vale para a lista nova»* — ela descrevia a população **antiga**
   (nomes de catálogo, ≤ 20 chars); a lista nova embrulha um `Name` do artista (§F5.9).
4. *«a receita é uma linha da Hierarquia»* — **não é**, desde 30/08 (§F5.13).
5. *«o passo 2 da `=7` demonstra a recusa»* — ele pedia o gesto que a guarda **deixa passar**
   (§F5.14).

⚠️ E **duas fixturas** absolviam a linha que deviam medir: a do swap (os dois eram **irmãos**, e um
ciclo precisa de descendência) e a do *Put back* (uma peça só — com duas leis diferentes a dar a
mesma tela).

---

## 10. Portão de fecho — os números

| | |
|---|---|
| `cargo fmt --all -- --check` | **OK** |
| `cargo clippy -p ph2d-host-desktop -p ph2d-editor-core -p ph2d-panel-inspector --all-targets` | **zero avisos** |
| `CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast` | **21 393 testes · 21 392 passaram · 1 flake de carga** |

⚠️ **A flake é da família do `CLAUDE.md §5.0` e tem as TRÊS assinaturas:** o conjunto de reprovadas
**MUDOU** entre as três corridas do mesmo trabalho (`a_wet_move_costs…` → `the_cost_of_sampling…` →
`a_round_live_offset…` → `apply_from_doc_is_zero_alloc_steady_state`), cada uma verde **3 de 3**
sozinha, e o diff da linha tem **zero linhas** nas crates delas (`ph2d-tool-painter`,
`ph2d-timeline`, `ph2d-vec-boolean`).

### O binário do smoke fica COMPILADO (§1.5.9 item 9)

```text
$ cargo build -p ph2d-host-desktop --release
    Finished `release` profile [optimized] target(s) in 2m 30s
$ cargo build -p ph2d-host-desktop --release
    Finished `release` profile [optimized] target(s) in 0.19s
```

O `incremental/` da worktree foi reclamado depois do gate (§1.5.9 item 7).
