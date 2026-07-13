# HANDOFF de integração — `line/Vector` (2026-07-13)

> Para o **agente integrador**, sob ordem explícita do Enio. A linha está **fechada**: não
> integrei, não pushei, não fiz ship (DIRETRIZ §1.5.9 / CLAUDE.md §0.7).
>
> **§8 é a seção que você mais precisa ler** — o que eu NÃO provei.

---

## §1 — O estado, verificado

| | |
|---|---|
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| Branch | `line/Vector` |
| Base | rebaseada sobre o `main` de hoje (24 commits do main absorvidos no início) |
| Commits novos | **3** (+ este handoff) |
| Trabalho não-commitado | nenhum |

```
73125773 fix(vector): uma FORMA VIVA nao tem alca de raio (o bug do "funciona e depois esquece")
81711b86 feat(vector): a ALCA de raio na quina — o gizmo (o pedido do Enio)
f712c93e feat(vector): raio de quina VIVO por-vertice (Live Corners) — o motor + o funil
```

**Gate de fechamento, saída real:**

```
cargo nextest run --workspace --no-fail-fast   → 6612 tests run: 6612 passed, 93 skipped
cargo clippy --workspace --all-targets         → 0 warnings, 0 errors
rustup run 1.95 cargo fmt --all                → aplicado
typos                                          → limpo
```

---

## §2 — O que a wave entregou

**O pedido do Enio: a alça de raio na quina** (Live Corners do Illustrator / Corner Tool do
Affinity). Modo **Node**, uma bolinha por quina do path selecionado, correndo pela
bissetriz. Arrastar para dentro arredonda; arrastar de volta até a âncora afia. Cheia = já
tem raio; vazada = afiada, só estacionada.

Mas o pedido exigiu **a espinha por baixo dele**, e é ela que sobrevive à wave:

**O documento guarda a quina AFIADA + um raio; o mundo consome a COZIDA** ([ADR-0119](architecture/decisions/0119-vector-live-corners-authored-source-cooked-geometry.md)).
É o `inkscape:original-d` + `d`. `VecVertex.corner_radius` é a fonte; `VecPath::cooked()` é
o que renderiza, aponta, enquadra e corta.

**Sem raio nenhum, `cooked()` é `Cow::Borrowed`** — mesmo ponteiro, zero alocação, zero
aritmética. É o que permitiu ligar o cozimento em TODO consumidor de geometria do módulo
(render, hit-test, bbox, booleana, gradiente) **sem mudar uma vírgula do comportamento de
hoje**. Um gate prova a identidade por PONTEIRO, não por igualdade.

**O motor** (`ph2d-vec-scene/src/corner_live.rs`) fecha o gap que o handoff anterior marcava
como aberto: **arredondar quina entre CURVAS**. A construção é a MESMA do `corners.rs`, com
duas generalizações — a direção da aresta vira a tangente da curva; o recuo vira distância ao
longo da curva. Na quina reta ela **reduz exatamente** à fórmula de sempre, e um gate compara
os dois motores byte a byte.

---

## §3 — O que mudou fora do meu módulo (o que você precisa saber para fundir)

### Contratos congelados: **nenhum tocado.** `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` intactos.

### Foundational tocado (ADR-0107 — permitido, projetado para isolamento)

| Crate / arquivo | O quê | Risco de colisão |
|---|---|---|
| `ph2d-vec-scene/src/lib.rs` | campo novo em `VecVertex` (apendado por **último**); `VEC_SCENE_SCHEMA_VERSION` **7→8** | **Baixo**, mas ver abaixo |
| `ph2d-vec-render/Cargo.toml` | dep nova `ph2d-tokens` | Baixo |
| `shells/desktop/src/project.rs` | `PROJECT_SCHEMA` **7→8** | ⚠ **ALTO** — ver §4 |
| `shells/desktop/src/main.rs` | `mod corner_handles;` | Baixo (linha nova) |
| `shells/desktop/src/render_loop/mod.rs` | 1 bloco novo no passe de overlay | Médio (arquivo quente) |
| `shells/desktop/src/input_dispatch.rs` | `on_press_node` ganhou 1 arg | Médio |

### Módulos NOVOS (isolados de propósito — não colidem com ninguém)

`ph2d-vec-scene/src/corner_live.rs` + `corner_live_tests.rs` ·
`ph2d-vec-edit/src/corner_handle.rs` + `corner_handle_tests.rs` + `pen_drag.rs` ·
`ph2d-vec-render/src/corner.rs` · `shells/desktop/src/corner_handles.rs` +
`corner_handles_tests.rs`

---

## §4 — ⚠ A colisão que eu ESPERO que você encontre: `PROJECT_SCHEMA`

**Eu subi o `PROJECT_SCHEMA` de 7 para 8.** Se qualquer outra linha desta jornada também
subiu um schema embutido no projeto (Flip, Painter, Motion, Timeline), vocês dois escreveram
`8` por motivos diferentes, e o merge textual vai parecer limpo.

**O valor certo não existe em nenhum dos dois lados** — é a memória
[`feedback_numbers_that_sum_across_lines_count_dont_pick`](../project-memory/feedback_numbers_that_sum_across_lines_count_dont_pick.md).
Se duas linhas bumparam, o número é **9**, não `8`. **Conte, não escolha.**

E o gate que trava isso está em `shells/desktop/src/project.rs`:

```rust
fn a_flip_or_vec_schema_bump_must_bump_the_project_schema() {
    assert_eq!(
        (PROJECT_SCHEMA, ph2d_flip::FLIP_SCHEMA_VERSION, ph2d_vec_scene::VEC_SCENE_SCHEMA_VERSION),
        (8, 3, 8),   // ← se outra linha bumpou, ESTA TRIPLA muda junto
    );
}
```

**Eu ampliei esse gate**: ele pinava só o Flip, e a `VecScene` — que também vai embutida no
arquivo de projeto — passava por fora. Um campo novo em `VecVertex` teria bumpado um contador,
deixado o outro para trás, e **o teste teria passado**. Agora ele cobre os dois. Se uma linha
futura embutir um terceiro schema, ele precisa entrar na tripla.

---

## §5 — Como smokar (o comando, pronto)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo run --release --bin ph2d-host-desktop
```

**O roteiro, exatamente:**

1. Pill **Vector** na barra de ferramentas.
2. Modo **Pen** (a caneta) → clique 4 ou 5 pontos e feche a forma. *(Desenhe, não use a
   Shape tool — ver a nota abaixo.)*
3. Modo **Node** (a seta branca). Cada quina ganha uma **bolinha vazada** na bissetriz,
   ligada à âncora por um fio.
4. **Arraste uma bolinha para dentro** → a quina arredonda ao vivo. A bolinha fica **cheia**.
5. **Arraste de volta até a âncora** → a quina volta a ser afiada.
6. `Ctrl+Z` desfaz (a fila global de undo cobre — a geometria está na captura).

**Funciona numa quina entre CURVAS também** — desenhe com a caneta arrastando (para puxar os
handles) e faça um bico; a alça aparece lá e o filete sai tangente aos dois lados.

> **Nota deliberada (não é bug):** uma forma da **Shape tool** (retângulo, polígono, estrela)
> é uma **Live Shape** e **não** ganha alça de raio — a geometria dela é re-cozida dos
> parâmetros, e um raio por-vértice seria varrido no próximo arrasto de slider. O raio dela é
> o campo **Radius** do painel. Para ter Live Corners numa forma paramétrica, use **Convert to
> Curves** primeiro. É a divisão do Illustrator, e é o Bug #11 do `BUGS_vector.md`.

---

## §6 — Os 4 bugs que os gates pegaram (detalhe em `BUGS_vector.md` #8–#11)

1. **Sinal de Cramer trocado** na interseção das tangentes → **todo** filete caía num fallback
   frouxo. Achado pelo gate que compara o motor novo com o velho na mesma quina reta.
2. **De Casteljau numa aresta "reta"** devolve controles no meio dela (uma reta aqui é um
   *smoothstep*, não uma reta uniforme). A curva ficava idêntica, a **representação** não — e
   é ela que o `to_bez` da booleana lê para emitir `line_to`. Eu ia recriar, pela porta dos
   fundos, o bug que aquele código já contorna.
3. **A alça escorregava do dedo** (o `park` implementado como piso no desenho e como
   subtração no arrasto — os dois não se cancelam).
4. **A alça funcionava e depois esquecia** numa Live Shape. Achado **simulando o smoke no
   papel**, não por teste.

---

## §7 — Os gates, e o que cada um prova (provados por **mutação**, não por alegação)

| Gate | Morde o quê (confirmado quebrando de propósito) |
|---|---|
| `the_straight_case_agrees_with_the_live_shape_engine_that_already_existed` | pegou os bugs #8 e #9 |
| `the_two_trims_of_a_segment_never_cross_they_collapse` | removi o guard de ordem → **vermelho** |
| `the_handle_never_lies_the_cook_trims_exactly_where_the_handle_says` | cozimento clampando diferente da alça → **vermelho** |
| `the_handle_stays_under_the_cursor_all_the_way_through_the_drag` | pegou o bug #10 |
| `a_live_shape_has_no_radius_handles_because_the_recook_would_erase_them` | removi o guard → **vermelho** |
| `pressing_the_handle_and_dragging_inward_actually_rounds_the_corner` | o seam vivo (press→drag→forma redonda) |

**E o gate que eu escrevi ERRADO, registrado de propósito:** a varredura combinada de quinas
vizinhas (`two_neighbouring_corners_devouring_the_same_edge…`) **não morde** o guard de ordem
nem o clamp de meia-corda — confirmei removendo os dois, um de cada vez, e ela seguiu verde.
Ela prova finitude e contenção (a lição do `NaN`), e **só isso**. O doc-comment dela agora diz
exatamente isso, em vez de alegar o que ela não faz. Um gate que não pode ficar vermelho é um
placebo, e eu tinha escrito um.

---

## §8 — **O que eu NÃO provei** (leia isto antes de confiar no resto)

1. **Não rodei o app.** Nenhuma destas linhas foi vista na tela. O smoke do §5 é do Enio, e o
   que pode estar errado ali é o que nenhum gate meu alcança: a alça pode estar **grande
   demais / pequena demais** (`CORNER_HANDLE_R_PX = 6`), o estacionamento pode estar
   **perto demais da âncora** (`CORNER_HANDLE_PARK_PX = 14`), e as cores dos tokens
   (`Accent`/`BorderEmph`/`AccentSoft`) podem **sumir contra o preenchimento da forma**. São
   três números e três tokens que só o olho decide.

2. **O `Cow::Owned` aloca por chamada, e eu não medi no app.** Num path COM raio, cada
   `cooked()` reconstrói o `VecPath`. `path_world_curve_bbox` tem 11 chamadores e alguns são
   por-frame. Eu **não otimizei de propósito** (a lição do roteador: 1–6 µs medidos,
   otimização desnecessária) — mas também **não medi**. Se o Enio arredondar 50 quinas de uma
   forma grande e o editor engasgar, é aqui. O caminho é um cache invalidado na edição, não um
   redesenho.

3. **Não testei o raio sob escala NÃO-uniforme.** O raio escala pelo fator **médio** dos eixos
   (a mesma aproximação que o raio do gradiente radial já fazia). Sob `scale(3, 1)` a quina
   deveria virar elíptica e não vira — ela fica circular com um raio de compromisso. É uma
   aproximação **consciente**, não um descuido, mas ninguém olhou para ela numa tela.

4. **A dívida de HR-15 do `ph2d-vec-render` continua lá.** A alça nova é tokenizada; o overlay
   de âncoras e as alças de conector seguem em `Color::from_rgba8(...)` literal (dívida da Fase
   1, registrada no doc-comment delas). **Não migrei de propósito**: mudar a cor delas muda a
   APARÊNCIA do que já existe, e isso é decisão do Enio, não refactor de quem passava por ali.

5. **Um teste de perf do Flip é FLAKY sob carga.** `ph2d-flip-render::pack_perf
   packing_a_dense_scribble_is_bounded` falhou uma vez na minha suíte cheia e passou isolado em
   72 ms. **Não é meu** (não toquei o Flip), mas você vai encontrá-lo, e o CI também pode. Vale
   um handoff pro dono do Flip.

6. **`cooked()` não cozinha `subpaths` de forma testada.** O código cozinha todos os contornos,
   mas os meus gates só exercitam o contorno primário. Um compound path (saída de booleana) com
   raio num vértice de subpath é caminho não-coberto — e como a booleana **assa** os raios, ele
   é hoje inalcançável pela UI. Fica registrado porque um dia deixará de ser.

---

## §9 — O que fica aberto (para o próximo da linha)

- **Tipos de quina** (Affinity: Round / **Chamfer** / Concave / Cutout). O chanfro é quase de
  graça — é uma **reta** entre os dois pontos de recuo, em vez do arco. A costura toda já
  está de pé; falta um enum no vértice.
- **Live Path Effects como NÓS** — o item #1 da pesquisa, o **multiplicador**. A distinção
  fonte ≠ cozido que esta wave introduziu é **o pré-requisito dele**, e agora existe. Era isso
  que a pesquisa não tinha visto: o item #2 (a "alça pequena") continha o item #1 dentro.
- Os itens 3–7 da pesquisa seguem intactos: texto em caminho, trim path, repeater, blend,
  largura variável.
- **Corner radius numa Live Shape** (hoje: não tem, por projeto — Bug #11). Se o Enio quiser,
  o caminho não é preservar o raio no recook (impossível: a contagem de vértices é função dos
  parâmetros) — é dar ao catálogo um campo `Radius` **por-canto**, como o round-rect já tem.

---

**A linha está fechada. Não integre, não pushe, não faça ship sem ordem explícita do Enio.**
