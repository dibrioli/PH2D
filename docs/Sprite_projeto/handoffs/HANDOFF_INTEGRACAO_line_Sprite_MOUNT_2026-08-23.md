# HANDOFF DE INTEGRAÇÃO — `line/Sprite` · **o consumidor de uma âncora** (DIRETRIZ §1.5.9)

**Data:** 2026-08-23 · **Branch:** `line/Sprite` · **Base:** `main` @ `35f937cb2`
**Wave anterior desta linha:** [`…_2026-08-22.md`](HANDOFF_INTEGRACAO_line_Sprite_2026-08-22.md) (já integrada).
**ADR desta wave:** [ADR-0072-amendment-1](../../architecture/decisions/0072-amendment-1.md).
**Spec normativa:** [`07_named_anchors.md`](../07_named_anchors.md) §7.6-bis e §7.8-bis (novas).

---

## 1. Identidade

O `CLAUDE.md §5` dizia, sobre este módulo: *«⛔ **nada consome uma âncora** — o ADR-0072 §2.6 é
autoria sem consumidor»*. Esta wave fecha isso.

**A tese, numa frase:** *uma âncora é um **QUADRO na hierarquia*** — uma entidade que monta nela já
é **filha** de quem a possui, e o componente novo diz apenas *qual* quadro do pai serve de origem.

| entregue | onde |
|---|---|
| `AnchorMount` + `MountState` + `mount_state`/`mount_frame`/`mount_state_of` | `crates/ph2d-ecs/src/anchor_mount.rs` (novo) |
| a API de runtime do §2.6 em **Rust** | `anchor_world_pose` · `anchor_names` · `anchor_pose_under` (mesmo ficheiro) |
| o quadro injetado nas **duas** travessias de mundo | `transform.rs::propagate_transforms` + `transform_inverse.rs::parent_world_transform_into` |
| persistência (registro + `PROJECT_SCHEMA` 89 → **90**) | `scene/registry.rs` · `project_schema.rs` |
| autoria: a linha **«Rides Parent Anchor»** da §12 | `sections/anchor_mount_row.rs` (novo) + a costura de 7 pontos |
| «quantos montam nesta âncora» na lista da §12 | `InspectorAnchorRow::riders` |
| o smoke | `PH2D_MOUNT_SMOKE=1` (`shells/desktop/src/mount_smoke.rs`, novo) |

**Segunda vaga do mesmo dia** — os quatro pedidos do primeiro smoke do Enio:

| pedido | entregue |
|---|---|
| escolher uma âncora **pousa** o objeto nela | o snap no braço `Mount(Some(_))` |
| a âncora que o filho monta fica **visível** ao mexer nele | passagem 2 do `marks_plan` — **mesmo com a §12 fechada** |
| duas caixas no pai: **«Always show anchors»** e **«Show anchors at runtime»** | `ph2d_ecs::AnchorVisibility` (`PROJECT_SCHEMA` 90 → **91**) |
| botão **«Reset to Anchor»** no filho | `AnchorFieldEdit::SnapToAnchor` — a MESMA operação do primeiro |

⛔ **`at_runtime` grava e não tem quem a leia** — não há modo de jogo (`shells/game`, R1, adiado
pelo Enio). Está marcada como tal no componente, no id, na spec e no smoke.

---

## 2. ⚠️ A lei que a integração NÃO pode partir

> **O quadro de âncora entra nas DUAS travessias de mundo, pela MESMA função.**

Este repositório responde «onde está esta entidade?» por dois caminhos, de propósito:
`propagate_transforms` (DFS, por quadro, renderer) e `world_transform` (subida pela cadeia, sob
demanda, gizmo · pick · física). Um quadro injetado **só numa** faz a espada **desenhar** na mão e
todo gesto agarrá-la na origem do pai — a família que o doc de `transform_inverse` já regista
(`docs/Physics/BUGS_physics.md` #2, medida a um offset de pai inteiro).

⚠️ **A ORDEM na subida é ao contrário de propósito:** empilha-se filho→raiz e a dobra percorre em
`rev()`, por isso o quadro da âncora entra **antes** da pose do pai para sair **depois** dela.
Trocar as duas linhas **compila** e põe a âncora no espaço errado.

**Gate que prende as duas:** `the_two_walks_agree_about_a_mounted_child`
([`anchor_mount_hierarchy.rs`](../../../crates/ph2d-ecs/tests/anchor_mount_hierarchy.rs)). Um merge
que toque `transform.rs` ou `transform_inverse.rs` tem de o correr.

---

## 3. Superfície de colisão — saída do script, colada

```
SUPERFÍCIE DE COLISÃO — line/Sprite contra main
  merge-base 35f937cb2   ·   6 commit(s)   ·   34 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                         90   (base: 89)
  ⚠   └ tripla do gate               (90, 13, 14)   (base: (89, 13, 14))
    VEC_SCENE_SCHEMA 14 · FLIP_SCHEMA 13 · DOC_VERSION 18   (todos = base)
▸ REGISTRO DE COMPONENTES — o contador é TRÊS
  ⚠ ph2d-ecs                               66   (base: 65)
  ⚠ ph2d-render (espelho)                  67   (base: 66)
  ⚠ ph2d-script (espelho)                  67   (base: 66)
▸ CONTRATO CONGELADO (§6)   node.rs intocado · tool.rs intocado
▸ ADR   cria 0072-amendment-1 (não consome número novo: é emenda)
▸ Cargo.lock   nenhum '+name' novo
▸ MARCADORES DE CONFLITO   nenhum
▸ TETOS DE LOC   nenhum ficheiro da linha passa do teto
```

### Leitura, símbolo a símbolo

**`PROJECT_SCHEMA` 89 → 91** (dois degraus: v90 `AnchorMount`, v91 `AnchorVisibility`). ⚠️ **O número se CONTA contra o `main` do dia, nunca se copia daqui.**
Quem obriga o bump é o **REGISTRO**, não um campo: um componente fora do `ComponentRegistry` é
descartado **em silêncio** pelo snapshot. São **três** sítios (a escada em `project_schema.rs`, a
tripla em `project_schema_tests.rs`, e o degrau que **não** pode ir para `project.rs`).

**Os três contadores de registro.** ⚠️ Esta linha já os deixou 4 e depois 2 atrás, **com a nota do
mecanismo escrita ao lado deles**. Desta vez os três subiram **no mesmo commit**, e cada um só é
visto pela suíte da **sua** crate:

| crate | valor | quem o vê |
|---|---|---|
| `ph2d-ecs` | 67 | `cargo test -p ph2d-ecs` |
| `ph2d-render` (espelho) | 68 = ecs + 1 (`Sprite`) | `cargo test -p ph2d-render` |
| `ph2d-script` (espelho) | 68 = ecs + 1 (`LuauScript`) | `cargo test -p ph2d-script` |

⛔ **São grandezas DIFERENTES — não copie o número de um para o outro.**

**Componentes ECS novos** (os nomes canónicos, é por eles que o save indexa):
`ph2d::ecs::AnchorMount` · `ph2d::ecs::AnchorVisibility`.

**Ids novos** (família própria, em `ids/inspector_anchor.rs`): `INSP_MOUNT_PICK` ·
`INSP_MOUNT_NONE_OPT` · `INSP_MOUNT_OPT[64]` · `INSP_MOUNT_SNAP` · `INSP_ANCHOR_VIS_EDITOR` ·
`INSP_ANCHOR_VIS_RUNTIME`. ⚠️ O comprimento do array **é** `ph2d_ecs::ANCHORS_MAX`,
com gate na shell (`the_mount_option_ids_cover_the_model_cap`).

**Ratchets de LOC que esta wave MOVEU** (só descem; um merge que os suba é erro):

| entrada | antes | agora | como |
|---|---|---|---|
| `ph2d-ecs/src/transform.rs` (ficheiro) | 768 | **removida** | `mod tests` cortado para `transform_tests.rs` (idioma do `children_order_tests.rs`); ficou em 621, abaixo do cap default |
| `paint_inspector` (função) | 380 | **348** | os TRÊS popovers diferidos saíram para `paint_frame_shared::paint_deferred_popovers` |
| `ph2d-panel-inspector/src/populate.rs` (ficheiro) | 605 medido | — | o bloco da §12 saiu para `populate_anchor.rs` (idioma do `populate_physics.rs`); **nenhuma entrada foi criada** |

---

## 4. Contratos congelados encostados

**Nenhum.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` estão intocados.

O que **é** foundational e foi tocado, com o desenho de isolamento a favor:

- `ph2d-ecs/src/anchor_mount.rs` — **módulo irmão novo**, append-only. Zero colisão possível.
- `transform.rs` / `transform_inverse.rs` — **um bloco cada**, no ponto onde o filho recebe o
  quadro do pai. Um merge que reescreva esses laços tem de reintroduzir a chamada a `mount_frame`.
- `scene/registry.rs` — uma linha `reg.register`, append no fim do bloco.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- `typos` sobre o corpo em português dos doc-comments novos (`anchor_mount.rs` é denso).
- `cargo machete` — **nenhuma dependência nova foi acrescentada**, então é improvável.
- A matriz 3-OS: o `mount_frame` usa `libm::sincosf` por dentro do `compose` que já existia; o
  hash de determinismo não muda **desde que nenhuma entidade dos fixtures ganhe `AnchorMount`**.

---

## 6. Ordem, dependências e smokes

**Ordem:** os 6 commits são lineares. O rebase sobre `main` foi *fast-forward* limpo no início da
jornada; não há dependência de outra linha.

### O smoke desta wave

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
  && env PH2D_MOUNT_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

Um boneco branco com duas âncoras (`hand_r`, `head`) e **três** filhos: espada vermelha em `hand_r`,
chapéu amarelo em `head`, e **uma mancha cinzenta que não monta em nada**.

⚠️ **A mancha é metade da cena.** Sem ela, uma versão com a montagem ignorada por completo pareceria
igual — três filhos a acompanhar o pai é o que filhos já faziam. Os três nascem com
`Transform::IDENTITY`: o que os separa é **só** a montagem.

O gesto: abrir **Sockets / Anchors** → escolher `hand_r` → arrastar a cruz. A espada vai junto, ao
vivo, num passo de `Ctrl+Z`; a mancha não se mexe. Selecionar a espada mostra
**«Rides Parent Anchor: hand_r»**; pôr «—» larga-a em cima da mancha.

### ⛔ O que NÃO foi smokado pelo Enio ainda

Toda esta wave. O gizmo, a §12 e a persistência das ondas anteriores já foram.

---

## 7. Achados que o integrador deve conhecer

1. **⛔ Duas das três superfícies do ADR-0072 §2.6 estão BLOQUEADAS, e foi medido antes de as
   construir.** O `ScriptHost` do desktop arranca com um script **placeholder**, `provide_read`
   **nunca é chamado** na shell, e não há UI para anexar um script a uma entidade. O `McpHost` é um
   `MemoryHost` de `HashMap<String, Value>`, com *«Real backends (S2/S3) implementarão sobre bevy_ecs
   World direto»* escrito no próprio doc. ⇒ `ph2d.anchor` e `sprite_anchor_get` seriam API sobre
   pontes que não existem — o mesmo defeito que esta wave cura, um nível acima. O gatilho de cada
   uma está na spec §7.8-bis; **não** as marque como pendências desta linha.
2. **Um gate meu era vazio, e foi a mutação que o disse.** `a_mount_whose_host_is_itself_mounted_composes_in_order`
   afirmava apanhar a troca de ordem da dobra e **não apanhava**: a fixtura era só translação, e
   translações comutam. A cura foi a **fixtura** (corpo rodado 90°), não o comentário.
3. **O gate da cena de smoke apanhou um defeito da cena.** A mão estava a `0,46 m` da origem com
   objetos de `0,48 m`, e a espada tapava o controlo. Os dois números de posição passam a vir da
   medição.
4. **O `paint_inspector` levou o corte que a §12 exigia, e a catraca desceu de verdade** (380 → 348).
   Levar só o popover novo devolveria o número a 380 exactos, e *ficar no mesmo sítio não é encolher*.
5. **A cura do teto do `transform.rs` foi cortar o `mod tests` para o irmão**, verbatim, e a
   entrada `768` **saiu** da allowlist em vez de subir.
6. **`anchor_world_point` (o overlay) passou a chamar `ph2d_ecs::anchor_pose_under`.** Há **uma** lei
   de «onde está esta âncora», e a alça, a montagem e a API de runtime leem-na. Antes eram duas
   cópias de uma linha de álgebra — que é exactamente a que se reimplementa sem ninguém reparar.

---

## 8. `CLAUDE.md §5` — a alteração já feita na linha

O bullet **Sprite Inspector** troca `⛔ nada consome uma âncora` por `✅ uma âncora já MOVE coisas`,
com a lei das duas travessias, o bloqueio medido do Luau/MCP e `PH2D_MOUNT_SMOKE` na lista de smokes.
⚠️ Se o `main` do dia tiver mexido nesse bullet, **funda, não escolha um lado**.

---

## 9. Higiene

- `cargo fmt --all` ✔ · `clippy --all-targets` sobre as **6** crates do diff, zero avisos ✔
- Suíte batched `--no-fail-fast` sobre a workspace ✔ (ver §10)
- `bash scripts/doc-index.sh --check` → 14 índices em dia ✔
- `bash scripts/adr-index.sh` regenerado ✔
- `rm -rf target/*/incremental` no fecho ✔
- ⛔ **Nenhum `push`. Nenhuma integração.** A linha fecha aqui e espera ordem do Enio.

## 10. Estado da suíte no fecho

```
cargo nextest run --workspace --no-fail-fast --cargo-profile ci-test
Summary [62,5 s] 17868 tests run: 17867 passed, 1 failed, 1818 skipped
   FAIL  ph2d-tool-painter  a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs
```

⚠️ **O único ✗ é a TERCEIRA flake de relógio já listada no `CLAUDE.md §5.0`**, registada desde
2026-08-16 em [`39_auditoria_solid_e_tracos.md`](../../Painter/39_auditoria_solid_e_tracos.md).
Re-corrida sozinha: **3 de 3 verde** (0,34 s cada). ⚠️ Esta wave **não toca uma linha** do Painter —
o diff são `ph2d-ecs`, `ph2d-editor-core`, `ph2d-panel-inspector`, os dois espelhos de registro e a
shell. *Re-rode sozinho antes de suspeitar da sua mudança.*

Suítes das crates tocadas, isoladas: `ph2d-ecs` **202** · `ph2d-editor-core` **1216** ·
`ph2d-panel-inspector` **180** · `ph2d-render` ✔ · `ph2d-script` ✔ · `ph2d-host-desktop` ✔.

### Provas de mutação desta wave (8, todas apanhadas)

| mutação | gate que a apanhou |
|---|---|
| a subida deixa de injetar a âncora | `the_two_walks_agree_about_a_mounted_child` (+4) |
| a propagação deixa de injetar a âncora | idem (+2) |
| a ORDEM da dobra trocada | idem + `…_composes_in_order` **depois de a fixtura ser corrigida** |
| `mount_choice` sem guarda de alcance | `a_mount_option_past_the_parents_list_picks_nothing` |
| o seletor pinta-se sempre | `the_mount_picker_appears_exactly_when_it_is_useful` |
| «—» só vale com âncoras no pai | `the_dash_option_always_clears_the_mount_even_with_no_parent_anchors` |
| escolhe sempre a primeira âncora | `picking_a_parent_anchor_publishes_the_name` |
| `riders: mounted.len()` | `the_snapshot_counts_the_riders_of_each_anchor_and_finds_the_parent` |
