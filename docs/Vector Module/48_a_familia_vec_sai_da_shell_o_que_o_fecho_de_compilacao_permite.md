# 48 — A família `vec` sai da shell: o que o FECHO DE COMPILAÇÃO permite (W2/L4, 2026-09-11)

> **O que este doc é.** A medição que responde *«quanto da família `vec` pode sair de
> `shells/desktop` hoje?»* — e a resposta não é uma escolha de escopo, é um **fecho** sobre o grafo
> de compilação. Ele existe porque a resposta é **8 ficheiros de 129**, e porque as três leituras
> que dei antes dessa estavam todas erradas *no mesmo sentido*: a favor de mover demasiado.
>
> Contexto: [auditoria de velocidade §4-C2](../DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md)
> · [briefings W2](../IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md) §0–§3 e §5
> · DIRETRIZ §6.7 item 5.

## §1 — O censo de partida

| | ficheiros | LOC |
|---|---:|---:|
| produto (`vec_*.rs` sem `_tests`) | 68 | 17 527 |
| teste (`vec_*_tests.rs`) | 61 | 17 933 |
| **total da família** | **129** | **35 460** |

A definição de «família» que reproduz o censo do briefing é o **basename a começar por `vec_`**,
onde quer que o ficheiro esteja em `src/` — é por isso que `render_loop/vec_snap_labels.rs` conta
(o 68.º ficheiro de produto) e `render_loop/vector_bridge*.rs` **não** (`vector`, não `vec_`).

⚠️ **Esses quatro `vector_bridge*.rs` (1 354 LOC de produto + 861 de teste) são a metade-shell do
módulo vectorial dentro do `render_loop` e não pertencem a família nenhuma do §2 do briefing.** Eles
existem, ninguém os conta, e quem fizer a Fase B tropeça neles.

## §2 — As quatro réguas, e por que só a última vale

O conjunto que pode sair tem de ser **fechado sob toda aresta de compilação** — a crate não pode
referir a shell, porque a shell é um `bin`. Medi isso quatro vezes:

| régua | move | o que ela esquecia |
|---|---:|---|
| «não toca `App` nem `crate::<mod>` de fora da família» | 71 fich. / 18 585 LOC | tudo o que está abaixo |
| **+** fecho sob `crate::vec_x` (quem refere quem fica, fica) | 29 / 7 621 | o hub `vec_entities` |
| **+** um `_tests.rs` não sai sem quem o **declara** | 15 / 3 055 | o sentido filho → pai |
| **+** `#[path]` é aresta DURA nos **DOIS** sentidos | **8 / 1 843** | — |

⛔ **A última mordeu depois de eu já ter movido 15 ficheiros.** `vec_gizmo_view.rs` **declara**
`#[path = "vec_gizmo_pick.rs"] mod …`, e esse toca `App` ⇒ o pai não pode sair. *Um `mod` declarado
por `#[path]` é parte da árvore de módulos do pai, não uma referência que se re-aponta* — e a minha
primeira passagem só tinha modelado a relação filho → pai.

⭐ **A lição generaliza para as outras quatro linhas da W2:** o fecho tem de incluir `#[path]` nos
dois sentidos, e 51 dos 129 ficheiros desta família declaram um.

## §3 — O que saiu, e o que o PAGOU

`crates/ph2d-app-vec` nasceu com os 8 do fecho + o que a A2 (o desacoplamento) libertou a seguir:

| | LOC |
|---|---:|
| os 8 do fecho | 1 843 |
| a lei pura do `vec_snap` + `vec_snap_sprites` (A2) | ~400 |
| `VecState` (19 campos de `App` com os doc-comments deles) | ~260 |

`shells/desktop/src`: **493 265 → 491 019 LOC**. `App`: **298 → 280** campos (`vec_*` de 75 para 56,
mais um `vec_state`).

⚠️ **O «~401 campos de `App`» do censo da auditoria e os meus 298 são contas DIFERENTES** — não
reconcilio as duas aqui, e quem citar um número diga qual regex o produziu. A minha é
`^\s+(pub(\(crate\))?\s+)?<nome>\s*:` dentro do corpo de `pub(crate) struct App`.

## §4 — Os 121 que FICAM, com o motivo contado

Depois da A2 o fecho é **zero**: tudo o que podia sair, saiu. Os 121 que ficam estão bloqueados
assim (um ficheiro pode ter mais de um motivo; a contagem é do motivo que o fecho registou):

| motivo | ficheiros |
|---|---:|
| refere um módulo da família que fica (cascata) | 35 |
| `crate::<mod>` de fora da família | 34 |
| declarado por `#[path]` de quem fica | 23 |
| toca `App` | 22 |
| acoplado por CAMINHO (`include_str!` / `CARGO_MANIFEST_DIR`) | 4 |
| **declara** por `#[path]` alguém que fica | 3 |

**Os módulos de fora que a família puxa são 47 distintos**, e os que mais pesam:

```
  9  crate::render_loop        4  crate::instance_sync     3  crate::input_dispatch
  5  crate::app_state          4  crate::ui_panel_spec     3  crate::profile_live
  4  crate::build_smoke        3  crate::instance_verbs    3  crate::envelope_live
  4  crate::bool_live          3  crate::morph_set         2  crate::undo  · … (+35)
```

⇒ **é esta a lista que o substrato da L0 tem de cobrir para esta família.** ⛔ Ela não se resolve
com «mais um método no trait de host»: `render_loop`, `input_dispatch` e `app_state` são a raiz de
composição, e `bool_live` / `profile_live` / `envelope_live` / `morph_set` / `instance_*` são
**outras famílias** que a 3.ª rodada ainda não abriu.

## §5 — As armadilhas de CAMINHO, que nenhuma leitura de `crate::` vê

Quatro ficheiros da família estão presos por uma **string**, não por um `use`:

| ficheiro | o que ele lê | modo de falha |
|---|---|---|
| `vec_bindings_tests.rs` | `include_str!("render_loop/vector_bridge.rs")` | falha a COMPILAR se mover — alto |
| `vec_morph_edit_tests.rs` | `include_str!` de `render_loop/mod.rs`, `morph_machine_drive.rs`, 2× `input_dispatch/*`, e `../../../crates/ph2d-panel-vector/src/event_clicks.rs` | idem |
| `vec_bucket_repro.rs` | `env!("CARGO_MANIFEST_DIR")/tests/fixtures/*.svg` | lê OUTRA pasta se mover; `panic!` com o caminho |
| `vec_component_general_census_tests.rs` | varre `CARGO_MANIFEST_DIR/src` | ⛔ **compila e PASSA sobre uma população menor** |

⛔⛔ **O último é a espécie perigosa e é a única que não avisa.** Um censo textual que caminha o
`src/` da sua própria crate, movido para outra crate, mede outra árvore e fica **verde**.

⚠️ **E há o mesmo perigo do lado de FORA.** Doze gates fora da família nomeiam um ficheiro dela por
caminho de string — entre eles `the_gesture_reads_what_the_frame_drew.rs` (`src/vec_gizmo_view.rs`),
`the_trim_tool_owns_its_gesture.rs`, `the_bucket_tool_owns_its_gesture.rs`,
`the_marquee_shape_comes_from_one_door.rs`, `the_net_knows_every_derived_writer.rs`
(`src/vec_tree_settle.rs`) e `one_word_for_the_reusable_thing.rs`. **Nenhum deles cobre os 8 que
saíram** (medido, não presumido) — mas qualquer ficheiro que a Fase B mova tem de os re-apontar.

⛔⛔ **A pior é `settle_skips_every_derived_geometry.rs`**: o 2.º teste dele faz `read_dir` do
`src/` **sem recursão**, filtra por `Transform::IDENTITY;`, e a única defesa é `hosts.len() >= 3`. A
população de hoje tem **4** ficheiros e um deles é da família (`vec_text_object.rs`). ⇒ *mover
ficheiros da família para uma SUBPASTA de `src/` tira-os desse censo em silêncio* — a contagem cai
para 3 e o gate continua verde.

## §6 — ⛔ Porque a A3 (agrupar em `src/vec/`) NÃO foi feita, com o preço

O briefing pede `shells/desktop/src/vec_*.rs` → `src/vec/` com «um `mod vec;` no lugar de N linhas».
**Não está feito**, e o motivo é medido, não preguiça:

1. **O benefício é fino.** Ele existe para que «o corte da Fase B seja mover UMA pasta» — mas o
   fecho do §4 diz que a Fase B **não move a pasta**: ela move o que cada cura desbloquear, ficheiro
   a ficheiro.
2. **O custo é máximo, e cai nos ficheiros mais disputados do repo.** `mod vec;` faz
   `crate::vec_entities` virar `crate::vec::vec_entities` — **189 ficheiros** só para esse símbolo,
   131 para `vec_scene`, e as edições caem no `render_loop/mod.rs` (14 058 linhas) e no
   `input_dispatch.rs` (7 180), que as OUTRAS QUATRO linhas da W2 estão a editar hoje. É
   exactamente a colisão que o §0 do briefing diz ser a razão de as seis linhas não cortarem juntas.
3. **Quebra um gate em silêncio** — o `read_dir` plano da §5.

⭐ **A variante de baixa churn existe e também não paga:** `#[path = "vec/vec_x.rs"] mod vec_x;`
move os ficheiros e mantém os caminhos de módulo (zero churn de referência), mas continua a serem N
linhas no `main.rs` (não o `mod vec;` pedido) e continua a quebrar o `read_dir` plano.

⇒ **decisão do integrador / do dono**, não desta linha. O que fica escrito é o preço.

## §7 — O que a A1 (poda de cenas) mediu: ZERO a apagar

Os roteadores desta família são quatro, e são **booleanos** (`is_none()`), não níveis numerados:
`PH2D_VEC_APPEARANCE_SMOKE` · `PH2D_VEC_BONE_SMOKE` · `PH2D_VEC_FADE_SMOKE` ·
`PH2D_VEC_STACK_SMOKE`. **Os quatro são citados em 3 a 7 documentos vivos cada** (fora de
`docs/archive`) ⇒ nenhum se apaga.

⚠️ **Dois achados que o censo por nome de ficheiro não daria:**

- **`PH2D_VEC_SVG_SMOKE` existe e não é desta família** — ele vive em `svg_import_smoke.rs`. *O nome
  da env diz `VEC`; o ficheiro não.* Um censo por basename nunca o encontra, e ele é citado em 3
  docs.
- **`PH2D_BUILD_SMOKE` é partilhado** (com `fx`, `instance`, texto, UI, …) e **nenhum nível dele é
  implementado num ficheiro `vec_*.rs`** — as cenas do módulo vectorial (`bool_smoke.rs`,
  `weld_smoke.rs`, `trim_smoke.rs`, `bucket_smoke.rs`, `pencil_smoke.rs`, `zorder_smoke.rs`,
  `texture_pattern_smoke.rs`, `text_path_smoke.rs`, …) chamam-se pela FEATURE, não por `vec_`.
  ⇒ **~17 ficheiros de cena do módulo vectorial caem fora de todas as seis famílias do §2 do
  briefing**, e a poda deles não tem dono. *Não os toquei: o roteador é partilhado e mexer nele numa
  de cinco linhas paralelas é a colisão que o §0 proíbe.*

## §8 — Como se re-mede (os instrumentos, para a Fase B)

Os quatro scripts desta jornada são de medição, não de produto, e viveram no scratchpad da sessão.
O que importa guardar é **a forma**, e ela está no §2: o fecho é iterativo e tem de modelar, sobre
os ficheiros de basename `vec_*`:

1. `\bApp\b` · `crate::<mod>` onde `<mod>` é um módulo da shell fora da família;
2. `include_(str|bytes)!` para fora da família, e `CARGO_MANIFEST_DIR`;
3. `crate::vec_x` → `x` tem de estar no conjunto;
4. **`#[path = "..."]` nos dois sentidos** (pai→filho e filho→pai).

E a prova de que nada se perde é `python3 scripts/nextest-list-diff.py antes.txt depois.txt`, com as
duas listas na MESMA árvore de dependências. ⚠️ A `--depth 1` (omissão) compara só o NOME da função,
logo tolera módulos renomeados; a `--depth 2` compara `módulo::fn`. **Os módulos da crate mantêm o
nome do ficheiro original de propósito** (`vec_snap`, `vec_snap_sprites`) para a prova passar nas
duas.

## ⛔ Recusas MEDIDAS

| o que | porquê |
|---|---|
| mover 71 ficheiros (a 1.ª leitura) | 63 deles não fecham sob `crate::vec_x`, `#[path]` ou `App` — §2 |
| mover `vec_transform.rs` | declara `#[path]` de um teste que precisa de `crate::profile_live` |
| mover `vec_gizmo_view.rs` | declara `#[path = "vec_gizmo_pick.rs"]`, que toca `App` |
| mover `sprite_snap_points` / `vec_move_sources` para a crate | dependem de `crate::vec_transform`, que não pode sair |
| agrupar os 11 campos de tipo-da-shell num `VecState` | trocaria 11 campos por 1 **sem mover uma linha** — §4 do handoff |
| `VecState` como componente/recurso do ECS | o `WorldSnapshot` do undo cobre todo componente registado ⇒ cada nudge do lápis viraria um passo de undo |
| a A3 (`mod vec;`) nesta linha | 189 + 131 referências nos dois ficheiros que as outras 4 linhas editam hoje — §6 |
