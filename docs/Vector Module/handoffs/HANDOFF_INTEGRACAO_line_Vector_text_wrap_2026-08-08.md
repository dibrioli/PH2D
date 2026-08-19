# Handoff de integração — `line/Vector` · W2a, **o texto REFLUI**

**Status:** FECHADO 2026-08-08 · no `main` em `9ff1f294e` (o commit que trouxe este arquivo).

> **Data:** 2026-08-08 · **Branch:** `line/Vector` · **Wave:** W2a (a fila dos Estudos)
> **Estado:** fechada, gates verdes, **SMOKE APROVADO pelo Enio (2026-08-08)** — aguarda
> ordem de integração.
>
> ⚠️ O smoke reprovou uma vez: os chips `Auto|Fixed` shiparam **mortos** (§4.1). Aprovado na 2ª.

---

## 1. O que esta wave entrega, numa frase

Um bloco de texto passa a **caber numa caixa que o artista autora**: `wrap_width` é um
`Option<f64>` no `VecShape::Text`, o quebrador mede com a **mesma régua do cozedor**, e o
**cursor lê a linha DESENHADA** em vez da linha digitada.

O plano 25 §418 nomeava a lacuna assim: *"o texto tem de saber medir-se, e hoje não sabe"*.
Um texto dentro de um fluxo de auto layout media a frase inteira como se ela nunca quebrasse,
porque as únicas quebras eram as que o artista escrevia à mão.

---

## 2. As decisões que decidem o resto

### 2.1 A caixa mora no componente, e o bump é de CAMPO

`wrap_width` fica em `VecShape::Text`, **ao lado** de `align` / `tracking` / `line_height`.

Um **componente ECS novo custaria zero de schema** (`stable_type_id = blake3(NOME)[..8]`, o
precedente do `VecStrokeProfile`/ADR-0148 e dos overrides da física) — e foi **recusado com
motivo**: ele partiria a resposta a *"como este bloco de texto se dispõe?"* em dois lugares, e
todo consumidor teria de saber juntar as metades.

⚠️ **`PROJECT_SCHEMA` 60 → 61, e a classe importa:** os degraus v57-v60 foram **variantes
apendadas** (o índice 0 não se move ⇒ o bump serve só ao caminho inverso). Este é um **CAMPO
apendado** ao blob de um componente, e postcard é **posicional** ⇒ **todo arquivo já salvo bate
no fim dos bytes**. É quebra dura nos dois sentidos, e é isso que o `project_schema_tests.rs`
narra na escada.

⚠️ **O valor é PROVISÓRIO.** Ele se **CONTA** contra o `main` do dia, não se escolhe
[[feedback_numbers_that_sum_across_lines_count_dont_pick]]. Tripla desta linha: **`(61, 13, 14)`**
(`PROJECT_SCHEMA`, `FLIP_SCHEMA_VERSION`, `VEC_SCENE_SCHEMA_VERSION` — os dois últimos
**intocados**).

### 2.2 A porta é UMA, e o cursor passa por ela

`vec_glyph::wrapped_lines(font, text, layout, axes, placement) -> Vec<&str>` decide onde uma
linha acaba. Três consumidores: **o cozedor** (`text_to_vec_paths`), **o compound**
(`text_to_compound_path`, que delega ao cozedor) e **o CURSOR**.

⚠️ **O cursor é a metade que o smoke julga.** Ele fazia `rsplit('\n')` — a última linha
**DIGITADA** — e com o refluxo isso passaria a piscar **fora do bloco**, no fim de uma linha
que ninguém desenhou. É [[feedback_derived_coordinate_seed_must_match_sample]] aplicado ao
caret: *quem deriva uma coordenada tem de a derivar da mesma função que a desenha*.

⚠️ **A régua é `line_advance`, não o shaper.** Um quebrador que medisse com uma segunda régua
concordaria com o cozedor quase sempre — e discordaria num tracking qualquer, o que aparece como
uma linha que passa da caixa em vez de como um erro.

### 2.3 Um texto EM CAMINHO não reflui, e a recusa mora na PORTA

`wrapped_lines` recebe o `placement` e devolve o texto inteiro quando ele é `OnPath`. A
alternativa — pôr `wrap_width: None` em cada sítio que monta um `TextLayout` para um caminho —
é uma **enumeração**, e ela apodrece no dia em que nascer o construtor N+1.

O porquê é geometria: a curva **já** diz por onde os glifos correm; refluir ali quebraria o
texto em linhas que o mapeamento por arco depois poria todas por cima umas das outras.

### 2.4 A UI vem na mesma wave, e só UMA row vive de cada vez

**`Width: Auto | Fixed`** + o slider **`Wrap width`**, na seção Text.

⚠️ A grandeza tem **presença** (reflui?) **E valor** (a que largura), então um slider só, com
`0` a significar *"sem caixa"*, seria um número a querer dizer duas coisas. Dois chips + um
slider que **só existe em Fixed** é o par `Mass: Auto | Manual` do editor de áudio: em Auto não
há largura a editar, e um slider ali seria um controle que não faz nada.

`TEXT_WRAP_MIN = 1.0` / `MAX = 40.0` são o **ALCANCE do slider**, não um cap do modelo (§0: um
limite legítimo nomeia o recurso — aqui não há recurso, há a faixa em que o gesto é útil).

### 2.5 O que o auto layout ganha é CONSEQUÊNCIA, não código

O `layout_live` mede um filho pela bbox dos `VecPath` dele, e o texto vivo entra na cena como
**um compound produzido pela mesma porta** ⇒ a bbox já é a refluída. **Nenhuma linha de layout
nesta wave** — e a consequência é **afirmada num gate** em vez de assumida
(`a_boxed_text_measures_narrower_and_taller_than_a_loose_one`, que exige as DUAS metades:
estreitar E crescer em altura, senão ele passaria sobre um texto TRUNCADO).

---

## 3. A tabela de colisão

| Eixo | Valor | Nota |
|---|---|---|
| `PROJECT_SCHEMA` | **60 → 61** | ⚠️ **PROVISÓRIO** — campo apendado, quebra dura; re-conte contra o `main` do dia |
| `VEC_SCENE_SCHEMA_VERSION` | **14**, intocado | |
| `FLIP_SCHEMA_VERSION` | **13**, intocado | |
| Registro do `ph2d-ecs` | **intocado** | nenhum componente novo (o campo entra no que já existe) |
| Contrato congelado | **intacto** | `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` — gates verdes, rodados |
| ADR | **nenhum** | ⇒ esta wave fica **fora** de toda disputa de número |
| `Cargo.toml` | **zero** | nenhuma dep, nenhuma crate nova |
| Ids novos | 4 | `VECTOR_TEXT_WRAP_AUTO` · `_FIXED` · `_W` · `_W_NUM` (hash de string ⇒ cobertos pelo `node_id_collisions`) |
| Cena de smoke | **`=63`** | próximo livre: **64** |

### 3.1 Os pontos de merge sensíveis

1. **`shells/desktop/src/project.rs`** — o literal do `PROJECT_SCHEMA`. ⚠️ **Duas linhas que
   escrevam o mesmo número NÃO conflitam** (o git não sabe o que ele significa), e o bump da
   segunda **evapora com a suíte verde**; o sinal costuma ser o conflito no
   `project_schema_tests.rs` ao lado. Confira os DOIS.
2. **`crates/ph2d-editor-core/src/ids/chrome/vector.rs`** — o arquivo cruzou 700 LOC e foi
   **PARTIDO**: os ids da seção Text saíram para o irmão **`vector_text.rs`** (`mod` + `pub use`
   no `chrome/mod.rs`). Uma linha que acrescente um id de texto ao arquivo antigo **funde limpa
   para o lado errado do corte** — o mesmo mecanismo que a `line/sculpt3d` produziu no
   `project.rs` em 04/08 [[feedback_clean_text_merge_can_be_semantically_broken]].
   ⚠️ **`VECTOR_MODE_TEXT` e `VECTOR_CONVERT_TO_CURVES` ficaram no PAI**, de propósito: o
   primeiro é um MODO (mora com os outros três) e o segundo assa **qualquer** forma viva.
3. **`shells/desktop/src/vec_text.rs`** — cruzou 600 LOC; o `mod tests` saiu para o irmão
   `vec_text_tests.rs` por `#[path]`, e ele **segue FILHO** (o `use super::*` alcança os
   privados). E `layout_of`/`axes_of` passaram a `pub(crate)` (a cena de smoke pergunta *"com
   que layout esta sessão coze?"*, que é exatamente o que elas respondem).
4. **`shells/desktop/src/main.rs`** — a lista de `mod`. ⚠️ A minha primeira inserção **orfanou o
   doc-comment do `tokens_smoke`**; o `mod text_wrap_smoke;` está agora em ordem alfabética,
   com doc-comment próprio.
5. **`build_smoke_router.rs`** — o roteador é uma lista de `if level == N` e **o primeiro
   vence**; o gate `no_two_smoke_scenes_claim_the_same_level` é quem pega uma colisão (verde,
   rodado).
6. **13 fixtures** ganharam `wrap_width: None` (o construtor de `VecTextEdit` / `VecTextParams`).
   São o **CONTROLE da wave**: com todas em `None` a suíte inteira fica verde, o que prova que o
   caminho sem caixa é byte-idêntico ao mundo que já shipava.

---

## 4. Gates e mutações

**21 gates novos.** O central mede a **TINTA** (a largura dos glifos produzidos), não chama
`line_advance` dos dois lados: duas cópias da mesma régua a concordar são **verdes por
construção**, e cegas a uma régua errada nas duas pontas.

| # | Mutação | Sangra |
|---|---|---|
| M1 | o cursor volta a `rsplit('\n')` | `the_caret_sits_on_the_last_drawn_line_not_the_last_typed_one` (e **só** ele — 2144 outros verdes) |
| M2 | tirar o `matches!(placement, At(_))` da porta | `a_text_riding_a_path_never_reflows` |
| M3 | o quebrador mede com outra régua (`track_px = 0`) | o gate central da tinta |
| M4 | `text_params` larga o `wrap_width` | o round-trip pelo componente |
| M5 | tirar os dois chips do `populate` | `both_width_chips_reach_the_bus` |
| M6 | pintar o slider incondicionalmente | `the_width_slider_lives_only_in_fixed_mode` |
| M7 | pôr a escrita de volta dentro do `debug_assert` (§6) | `no_write_hides_inside_a_debug_assert` **e** a suíte de release |
| M8 | o reconhecedor devolve sempre `false` | `the_recogniser_recognises` (o controle positivo) |

⚠️ **A M3 SOBREVIVEU na primeira rodada, e o defeito era da FIXTURE:** ela usava
`tracking: 0.0`, onde um quebrador que ignora o tracking mede **exatamente o mesmo** que o
cozedor — as duas réguas concordavam por acidente. Com `tracking: 0.25` ela morde
[[reference_topic_fixture_discipline]].

⚠️ **E dois defeitos foram MEUS, os dois no oráculo do cursor:** ele esquecia que `caret_of`
devolve a BASE do cursor (`pen_y − 0.2·size`), não a caneta; e a primeira `CARET_BOX = 6.0` só
produzia 2 linhas, quando o gate exige ≥ 3 para o *"não é a última digitada"* significar algo.

**Seam:** os três gates de painel dirigem um par **Down+Up REAL**. ⚠️ Um `WidgetEvent::Click`
sintético **pula a checagem de focabilidade do store**, então um chip tirado do `populate`
continuaria a "passar": pintado, com área de hit, e **morto sob o mouse**.

### 4.1 ⚠️ E o seam não bastou — os chips shiparam MORTOS

**Reportado pelo Enio no smoke:** *"os botões Auto e Fixed não aceitam ser checados
(provavelmente sem link)"*. Estava certo, e o diagnóstico dele também.

O `render_loop` citava **só** o slider `VECTOR_TEXT_WRAP_W`. Os dois chips eram pintados,
registados, o ponteiro sobre eles virava `Click`, o `Click` chegava ao barramento — **e do outro
lado não havia braço nenhum**. O seam prova **painel → bus** e é **estruturalmente cego** ao
passo seguinte.

⚠️ **O número que fecha o caso:** com o defeito reinstalado, o seam do painel fica **3/3 VERDE** e
só o gate novo sangra. *Dois gates verdes compostos não provam a corrente inteira*
[[feedback_green_composed_gates_can_hide_an_unproven_connector]].

O gate novo é **`shells/desktop/tests/the_width_chips_are_wired.rs`** (3 asserções): cada chip é
citado por um braço que **escreve o pedido** · Auto pede `Some(None)` e Fixed **semeia** (sem a
2ª metade, dois botões que fazem a mesma coisa passariam) · e o dreno chama a porta E escreve o
`wrap_width` dos textos **selecionados**. ⚠️ Ele afirma a **relação**, nunca uma distância em
bytes — o proxy que já expirou duas vezes nesta linha em 23/07.

⚠️ **E o conserto trouxe uma decisão de produto:** **Fixed semeia com a largura que o texto JÁ
mede** (`vec_glyph::unwrapped_block_width`, o **terceiro** consumidor da mesma `line_advance`),
não com um número de fábrica — clicar Fixed **não move um glifo**, ele só torna o número
editável. É o `Auto → Manual` da massa no editor de áudio, que semeia o campo com a massa que o
corpo já tinha. Sem sessão viva não há texto a medir, e aí cai no default do slider.

| # | Mutação | Sangra |
|---|---|---|
| M9 | tirar os dois braços do `render_loop` (**o defeito que shipou**) | `both_width_chips_are_consumed_by_the_shell` + `auto_asks_for_no_box…` — ⚠️ **o seam fica 3/3 verde** |
| M10 | o Fixed semeia com `DEFAULT_TEXT_WRAP` | `auto_asks_for_no_box_and_fixed_asks_for_one` |

---

## 5. A bateria de fechamento (rodada, não auto-relatada)

- `cargo fmt --all -- --check` — limpo
- `cargo clippy --workspace --all-targets` — limpo
- `file_loc_caps` (shell) · `architecture_workspace_file_loc_cap` — verdes **depois dos dois
  splits**
- `arch_safe_clamp_only` · `node_id_collisions` · `architecture_panel_wiring_parity` ·
  `no_tofu_glyphs` — verdes
- `architecture_tool_contract_surface` — verde
- `no_two_smoke_scenes_claim_the_same_level` — verde
- `no_effect_inside_debug_assert` (**novo**, §6) — verde
- `the_width_chips_are_wired` (**novo**, §4.1) — verde
- suíte do shell **debug**: 2149 passed, 0 failed · crates tocadas: verdes
- suíte **release**: 2149 passed, 0 failed — ⚠️ **e ela nasceu VERMELHA**, ver §6

---

## 6. ⚠️ A suíte de RELEASE achou um bug de PRODUTO que a de debug não podia ver

**Pré-existente**, medido no commit pai (`HEAD~1`, mesma falha) ⇒ **não é da W2a** — veio com a
wave dos tokens (W4c.1). Mas está na branch, então fecha aqui.

`shells/desktop/src/render_loop/tokens_bridge.rs` fazia:

```rust
debug_assert_eq!(set_color_overrides(keep), 0);   // ⚠️
```

**`debug_assert!` apaga o argumento inteiro em release.** Com a escrita lá dentro, o botão
**"Reset This Mode" não fazia NADA num build de release** — sem erro, sem aviso, sem um pixel
diferente, e com a suíte de debug **verde**. É a forma mais cara de efeito colateral: ele não
falha, ele **desaparece**.

A cura é mecânica — **o valor primeiro, a asserção depois** (ela só precisa do NÚMERO):

```rust
let dropped = set_color_overrides(keep);
debug_assert_eq!(dropped, 0);
```

E o gate novo **`no_effect_inside_debug_assert`** (`ph2d-editor-core/tests/`) varre `crates` +
`shells` recusando a FORMA, com **controle positivo** (o reconhecedor tem de reconhecer, senão
um scanner que devolvesse `false` deixaria o gate verde para sempre
[[feedback_a_negative_search_needs_a_positive_control]]) e com os falsos-positivos óbvios
pinados (`debug_assert_eq!(dropped, 0)` passa; `offset_of(x)` não casa `set_`). A varredura do
repo inteiro achou **estes dois sítios e mais nenhum**.

⚠️ **A lição operacional, e ela é a razão de a bateria ter as duas metades:** *uma suíte que só
corre em debug é cega a toda uma classe de defeito* — a mesma nota que a `line/FLIP` deixou em
30/07 pelo motivo oposto (um kill de wall-clock que só reprovava em debug). **Rode as duas.**

---

## 7. O smoke

```
env PH2D_BUILD_SMOKE=63 cargo run -p ph2d-host-desktop --release
```

Dois textos com **a MESMA frase**: o de cima com caixa (várias linhas, **já selecionado**), o de
baixo sem nenhuma (o controle, numa linha só, a sair do quadro).

⚠️ **A cena imprime o número que a torna válida** — em quantas linhas cada um coze. Se o de cima
disser `1 linha`, ou se aparecer a linha `!! a cena NAO contem o fenomeno`, **PARE**: o resto do
roteiro não diz nada.

O roteiro impresso tem 7 passos. Os que decidem:

- **A fileira Width** — clicar Auto tem de fazer o slider **SUMIR** (e a frase voltar a uma linha).
- **A caixa é do TEXTO** — dar zoom e pan **não** pode mudar a quebra.
- **O cursor** — duplo-clique e escrever: ele pisca no fim da **última linha desenhada**, e desce
  junto quando a linha enche.
- **Salvar e abrir** — a caixa volta com o texto.
- **O controle** — o texto de baixo abre em Auto, sem slider, exatamente como estava.

---

## 8. Aberto, nomeado

- **Sem hífen.** Uma palavra maior que a caixa **transborda inteira**. Partir uma palavra é
  decisão de produto (e um dicionário), não uma melhoria mecânica — há gate a pinar o
  comportamento de hoje para ninguém o "consertar" sem passar por essa decisão.
- **Sem justificação.** `TextAlign` segue Left/Center/Right; *justified* precisa de uma segunda
  pergunta (como distribuir a folga) e é wave própria.
- **A caixa não tem alça no canvas.** Ela é autorada pelo slider; um gizmo de largura é o gesto
  natural seguinte e não foi construído.
- **A altura não é autorada.** O bloco cresce para baixo sem limite — não há *overflow* nem
  encadeamento de caixas.
