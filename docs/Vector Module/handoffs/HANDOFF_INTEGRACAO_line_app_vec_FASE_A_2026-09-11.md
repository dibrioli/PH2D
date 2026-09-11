# HANDOFF DE INTEGRAÇÃO — `line/app-vec` (W2/L4, **FASE A**) — 2026-09-11

> **Estado: FASE A fechada. A FASE B NÃO começou** e depende de a `line/app-host` (L0) integrar
> primeiro (briefings W2 §1). Esta linha **não integra e não faz ship** — entrega isto e para
> (CLAUDE.md §0.7).

## §1 — Identidade

| | |
|---|---|
| branch | `line/app-vec` |
| HEAD | `06619e417a2735977a2aab7ebf9bea645d07e07d` |
| merge-base com `main` | `8fa4f115bbdedb7635581af8c528a8aedd5c8d74` |
| commits | **5** |
| diffstat | 40 ficheiros, +1 158 / −617 |

## §2 — ⭐ O ACHADO que muda o plano da W2 para esta família

**A família `vec` não sai da shell por incrementos: 8 ficheiros de 129.** E isso não é uma escolha
de escopo — é um **fecho** sobre o grafo de compilação, que medi quatro vezes, cada resultado menor
que o anterior e **as três primeiras todas a favor de mover demasiado**:

| régua | move | o que ela esquecia |
|---|---:|---|
| «não toca `App` nem `crate::<mod>` de fora» | 71 fich. / 18 585 LOC | tudo abaixo |
| **+** fecho sob `crate::vec_x` | 29 / 7 621 | o hub `vec_entities` |
| **+** um `_tests.rs` não sai sem quem o **declara** | 15 / 3 055 | o sentido filho → pai |
| **+** **`#[path]` é aresta DURA nos DOIS sentidos** | **8 / 1 843** | — |

⛔ **A última mordeu depois de eu já ter movido 15 ficheiros** (revertidos): `vec_gizmo_view.rs`
**declara** `#[path = "vec_gizmo_pick.rs"]`, e esse toca `App`. *Um `mod` declarado por `#[path]` é
parte da árvore de módulos do pai, não uma referência que se re-aponta.* **51 dos 129** ficheiros
desta família declaram um `#[path]` ⇒ **as outras quatro linhas da W2 têm de modelar essa aresta
nos dois sentidos**, e nenhuma sabe disso ainda.

A medição completa, com os instrumentos e as recusas, está em
[`docs/Vector Module/48_…`](../48_a_familia_vec_sai_da_shell_o_que_o_fecho_de_compilacao_permite.md).

## §3 — O que a linha entregou

1. **`crates/ph2d-app-vec`** nasceu com os 8 do fecho + o que a A2 libertou. `crates/*` é **glob** no
   workspace ⇒ **zero edição central**.
2. **A2, `impl App`:** a **lei** do snap saiu para `ph2d_app_vec::vec_snap` / `::vec_snap_sprites`.
   Os métodos de `App` da família vão de **14 para 11**; os 8 que ficam estão nomeados no §5.
3. **A2, campos:** **19** campos `vec_*` de `App` viraram `ph2d_app_vec::state::VecState`. `App`:
   **298 → 280** campos (`vec_*` de **75 → 56**, mais um `vec_state`).
4. **A1, poda de cenas: ZERO a apagar** — os 4 roteadores desta família são citados em 3–7 docs cada.
5. `shells/desktop/src`: **493 265 → 491 019 LOC**.

⭐ **E o desacoplamento comprou TRÊS gates que não podiam existir antes:** `snap_cfg`, `wants_curves`
e `ids_of_bits` eram métodos de `App` ⇒ exigiam uma janela ⇒ **não tinham um único teste**. Agora
medem: o Alt desliga o encaixe **inteiro** sem reescrever os interruptores do painel · o limiar é
linear no world-por-pixel · a porta única responde a **qualquer** das duas reivindicações de posição
(um `&&` ali mataria o encaixe sobre a curva — o modo de falha que o doc-comment dela nomeia).

## §4 — Foundational / partilhado tocado, e por quê

| ficheiro | o quê | risco de merge |
|---|---|---|
| `shells/desktop/src/main.rs` | 5 `mod vec_x;` → `pub(crate) use ph2d_app_vec::vec_x;` · 19 inicializadores → 1 | ADIÇÃO/REMOÇÃO em região da família |
| `shells/desktop/src/app_state.rs` | −19 campos, +1 (`vec_state`) | idem |
| `shells/desktop/src/render_loop/mod.rs` | 1 `mod` → `use`; ~49 sítios `self.vec_x` → `self.vec_state.x` | **MODIFICAÇÃO** de linhas vec-específicas |
| `shells/desktop/src/input_dispatch.rs` | ~39 sítios idem | idem |
| `shells/desktop/Cargo.toml` | +1 dep; `panel-vector` encadeia `ph2d-app-vec/panel-vector` | 1 linha cada |
| `scripts/nextest-list-diff.py` | **bug corrigido** — ver §7 | — |
| `crates/ph2d-panel-vector/{src/section_scope.rs, tests/it/…}` | 2 doc-comments com o nome antigo do campo | 1 linha cada |
| `shells/desktop/tests/it/` (6 ficheiros) | strings de gates textuais — ver §6 | 1–9 linhas cada |

⚠️ **As ~88 modificações no `render_loop/mod.rs` e no `input_dispatch.rs` são o item a conferir com
olho**, porque as outras quatro linhas da W2 estão a editar os mesmos dois ficheiros hoje. Cada
linha alterada é **vec-específica** (`self.vec_*`), logo o merge de linha resolve; mas é
modificação, não adição.

## §5 — ⛔ O que NÃO saiu, e é a lista que a L0 tem de cobrir

**Os 121 ficheiros que ficam, por motivo** (o que o fecho registou):

| motivo | ficheiros |
|---|---:|
| refere um módulo da família que fica (cascata) | 35 |
| `crate::<mod>` de fora da família | 34 |
| declarado por `#[path]` de quem fica | 23 |
| toca `App` | 22 |
| acoplado por CAMINHO (`include_str!` / `CARGO_MANIFEST_DIR`) | 4 |
| **declara** por `#[path]` alguém que fica | 3 |

**Os 47 módulos de fora que a família puxa**, os que pesam: `render_loop` (9) · `app_state` (5) ·
`build_smoke` (4) · `bool_live` (4) · `instance_sync` (4) · `ui_panel_spec` (4) · `instance_verbs`
(3) · `morph_set` (3) · `input_dispatch` (3) · `profile_live` (3) · `envelope_live` (3) · `undo` (2).
⛔ **Isto não se resolve com «mais um método no trait de host»:** `bool_live`, `profile_live`,
`envelope_live`, `morph_set` e `instance_*` são **outras famílias** que a 3.ª rodada não abriu.

**Os 8 métodos de `App` que ficam, com o motivo:**

| método | precisa de |
|---|---|
| `vec_px_to_world` | `gfx.surface` + `gfx.camera` — ⚠️ e **não é do vector**: 11 ficheiros de outras famílias o chamam |
| `vec_snap_point` · `vec_snap_move` | `gfx.hero_screen.grid` — estado de PAINEL |
| `vec_rebuild_snap_targets` | `gfx.guides` + a orquestração do gesto |
| `dragged_entity_bits` · `dragged_vec_path_ids` · `snap_dragged_vec_during_drag` | `gfx.hero_screen.gizmo` |
| `sprite_snap_points` · `vec_move_sources` | `crate::vec_transform`, que **não pôde sair** |
| `settle_tree_before_capture` | a **captura do undo** e TRÊS famílias (vec + flip + hierarquia) |

**Os 11 campos que ficam por o TIPO morar na shell** (`blend_spines`, `bucket_cache`, `bucket_face`,
`connect_sides`, `marquee`, `morph_plans`, `morph_set_pending`, `patternpath_handle`, `pencil_hand`,
`trim_hit`, `width_grab`): ⛔ **agrupá-los num `VecState` da shell trocaria 11 campos por 1 sem mover
uma linha de lugar.** Cada um espera que o TIPO dele saia.

## §6 — ⚠️ O que só o portão de fecho pegou (e o que o `ship.sh` ainda pode pegar)

**O portão apanhou 10 vermelhos**, todos gates que leem o FONTE como texto e procuram um nome de
campo. ⭐ **Eles funcionaram como desenhados** — e o primeiro a falar foi um **controlo positivo**
(`the_scanner_finds_what_it_scans_for`): *«`self.vec_pencil.on_press(` sumiu do dispatch — as
asserções de ORDEM abaixo passariam sem examinar nada»*. Sem ele, cinco gates de ordem do mesmo
ficheiro ficariam **verdes sobre um texto sem o sujeito**.

⚠️⚠️ **E o `assert` de contagem impediu uma 11.ª quebra, essa silenciosa:** `self.vec_pencil` conta
**12** em bruto e **9** com fronteira de palavra — as outras três são `self.vec_pencil_hand.begin(`,
e o `vec_pencil_hand` **não se moveu**. Um `str.replace()` cru teria apontado um gate a um campo
inexistente, e ele passaria a medir nada.

**O `ship.sh` (não corri):** `cargo deny` · `cargo audit` · `cargo machete` (⚠️ a lista de deps da
crate nova é medida ficheiro a ficheiro, mas o machete é quem o confirma) · o `--all-features` do
workspace inteiro · e o `stack-audit.sh --tetos`. **Zero pacote externo novo** ⇒ `deny`/`audit`
devem ser inertes.

## §7 — ⚠️ Um bug de FERRAMENTA que atinge as outras quatro linhas

`scripts/nextest-list-diff.py` — **a prova comum da W2** — devolvia a **AJUDA** em vez de comparar
quando se passava `--depth 2`, porque o valor do flag ficava nos posicionais. **A forma documentada
no cabeçalho dele nunca tinha sido corrida.** ⚠️ O modo de falha é o caro: um flag que imprime a
ajuda lê-se como *«usei-o mal»*, não como *«ele está quebrado»* — e o `--depth 2` é precisamente o
que **aperta** a prova de uma extracção (`módulo::fn` em vez de só `fn`). **Corrigido** (aceita
`--depth 2` e `--depth=2`; o controlo de 3 posicionais continua a devolver a ajuda).

## §8 — A PROVA (§3 dos briefings)

| item | resultado |
|---|---|
| **(a) nenhum teste se perde** | **`ONLY-A = 0`** nas duas profundidades. `22 635 → 22 638`; **MOVED 34** (os que foram para a crate); **ONLY-B 3** = exactamente os 3 gates novos (`o_bypass_desliga_o_encaixe_e_preserva_os_interruptores`, `o_limiar_segue_o_zoom`, `qualquer_reivindicacao_de_posicao_pede_a_geometria`) |
| **(b) roteadores iguais** | **106 antes, 106 depois, `diff` vazio.** Os 5 gates `no_two_*_scenes_claim_the_same_level` vivem em ficheiros que a linha não tocou ⇒ intocados |
| **(c) a shell encolheu** | **493 265 → 491 019 LOC** (−2 246, **−0,46 %**) |
| **(d) gate de fecho** | `fmt --check` ✓ · `clippy --all-targets --all-features` nas 2 crates ✓ · `typos` ✓ · `doc-index --check` ✓ (19 índices) · **9 gates de teto de LOC** ✓ · **`nextest-impacted.sh`: 13 324 testes, 0 falhas** (a `load 70`, com os gates de razão da família de flakes incluídos) |
| **(e) smoke do dono** | binário **compilado e quente** — `cargo build -p ph2d-host-desktop --profile smoke`, 1.ª corrida `1m 03s`, **2.ª corrida: `Finished` em `0,21 s`, ZERO linhas `Compiling`** (`target/smoke/ph2d-host-desktop`, 77,8 MB). ⚠️ Comportamento **idêntico** por construção: a extracção não muda produto |

⛔⛔ **O item (c) do briefing pede TAMBÉM a unidade `bin (check-test)` num `--timings` a frio, e eu
NÃO a entrego — de propósito, e o motivo é aritmética, não carga:** a extracção tirou **0,46 %** da
shell. Sobre os `16,8 s` de front-end a frio que a auditoria mediu, isso é **~0,08 s** — uma ordem de
grandeza abaixo da variância entre corridas. ⚠️ E hoje a máquina está a `load 35–70` com cinco linhas
irmãs a construir, o que pela regra do CLAUDE.md §5.0 **anula** qualquer leitura de relógio.
*Publicar um número aqui seria publicar ruído com duas casas decimais.* ⇒ **o que esta linha move na
unidade de compilação é a LOC, e ela está medida; o relógio só terá sinal depois de a Fase B mover
os 121.**

## §9 — ⛔ A A3 (agrupar em `src/vec/`) NÃO foi feita — é decisão do integrador

O briefing pede `src/vec_*.rs` → `src/vec/` com «um `mod vec;` no lugar de N linhas». **Não fiz, e o
preço está medido** (doc 48 §6):

1. **O benefício é fino** — ele existe para que «o corte da Fase B seja mover UMA pasta», e o fecho
   do §5 diz que a Fase B **não move a pasta**: move o que cada cura desbloquear, ficheiro a ficheiro.
2. **O custo cai nos dois ficheiros mais disputados do repo** — `mod vec;` faz `crate::vec_entities`
   virar `crate::vec::vec_entities`: **189 ficheiros** só para esse símbolo, **131** para
   `vec_scene`, e as edições aterram no `render_loop/mod.rs` e no `input_dispatch.rs`, que as outras
   quatro linhas da W2 estão a editar **hoje**. É a colisão que o §0 do briefing dá como razão de as
   seis linhas não cortarem juntas.
3. ⛔⛔ **Quebra um gate em SILÊNCIO.** `settle_skips_every_derived_geometry.rs` faz `read_dir` do
   `src/` **sem recursão** e a única defesa é `hosts.len() >= 3`. A população é **4** e um deles é da
   família (`vec_text_object.rs`) ⇒ mover ficheiros para uma subpasta tira-os do censo, a contagem cai
   para 3, e **o gate continua verde**.

⭐ A variante de baixa churn (`#[path = "vec/vec_x.rs"] mod vec_x;`) move os ficheiros com **zero**
churn de referência — mas continua a serem N linhas no `main.rs` (não o `mod vec;` pedido) e
continua a quebrar o `read_dir` plano. ⇒ **decisão de quem integra**, com o preço escrito.

## §10 — Superfície de colisão (`collision-surface.sh`, corrido AGORA nesta worktree)

```
SUPERFÍCIE DE COLISÃO — line/app-vec contra main
  merge-base 8fa4f115b   ·   5 commit(s)   ·   40 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA        128   (base: 128)        └ tripla  (128, 13, 22)  (base: idem)
    VEC_SCENE_SCHEMA      —     FLIP_SCHEMA  13    DOC_VERSION  18   (todos = base)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho) 86 (base: 86)   ·   ph2d-script (espelho) 86 (base: 86)
▸ CONTRATO CONGELADO (§6)  — os dois INTOCADOS
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 1 pacote novo: "ph2d-app-vec"  (aresta INTERNA, não dependência externa)
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum ficheiro da linha passa do teto
```

⭐ **NENHUM contador partilhado se move.** Mover código não muda serialização, e nada aqui o fez:
zero schema, zero registo, zero contrato, zero ADR, zero pacote externo.
⚠️ **Símbolos novos que outra linha poderia colidir:** a crate `ph2d-app-vec` (nome), a feature
`panel-vector` dela, `ph2d_app_vec::state::VecState`, e o campo `App::vec_state`. Nenhum deles é um
número numa escada.

## §11 — O smoke, e o que NÃO foi smokado

⚠️ **A extracção não muda produto** — nenhuma das cinco cenas desta família mudou de comportamento;
o que mudou é onde o código vive. O smoke serve para **confirmar a inércia**.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

As outras quatro da família, o mesmo comando trocando a variável:
`PH2D_VEC_STACK_SMOKE=1` · `PH2D_VEC_APPEARANCE_SMOKE=1` · `PH2D_VEC_FADE_SMOKE=1` ·
`PH2D_VEC_SVG_SMOKE=1` (⚠️ esta última **não é desta família por nome de ficheiro** — vive em
`svg_import_smoke.rs`; ver doc 48 §7).

⏳ **NÃO smokado:** o gesto de **snap** no canvas (é o que a A2 mais mexeu — a lei mudou de crate,
byte a byte) e o **lápis**, cujo `self.vec_pencil` foi renomeado em 9 sítios de gates de ORDEM de
dispatch. Os dois têm gates verdes e nenhum passou por mão humana nesta jornada.

## §12 — Para quem pegar a FASE B

1. `git rebase main` (depois de a L0 integrar) e **ler o `HOWTO_partir_uma_familia_da_shell.md`
   inteiro**.
2. ⛔ **Antes de mover um ficheiro, refazer o fecho** — a forma está no doc 48 §8, e as quatro
   arestas são `App` · `crate::<mod>` de fora · `include_str!`/`CARGO_MANIFEST_DIR` · **`#[path]` nos
   dois sentidos**.
3. **Re-apontar os 12 gates de fora que nomeiam um ficheiro da família por caminho de string**
   (lista no doc 48 §5). ⛔ Nenhum deles cobria os 8 que saíram — isso foi **medido**, não presumido
   —, mas qualquer ficheiro novo que se mova cai lá.
4. ⚠️ A `ph2d-app-vec` tem de continuar a **não depender da shell**. Se a família precisar de algo
   que só a shell tem, isso é o trait de host da L0 ou um recurso do ECS — **e não um `pub` a mais na
   shell**.
