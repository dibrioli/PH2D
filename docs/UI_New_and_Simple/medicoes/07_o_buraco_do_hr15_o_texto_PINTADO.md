# 07 — O buraco do HR-15: o texto **PINTADO**

> Medido em 2026-09-09 pela `line/UIUX`, a partir de um achado lateral da wave das faces vazias.

## §1 — O que o gate vê, e o que ele não vê

O `CLAUDE.md` §0.3 diz **«zero string hardcoded — tudo via tokens / i18n (HR-15)»**, e o gate que
o defende é [`hr15_no_hardcoded_ui_strings.rs`](../../../crates/ph2d-editor-core/tests/hr15_no_hardcoded_ui_strings.rs).
O doc dele nomeia, de si próprio, exactamente **dois** padrões:

| padrão | o que é |
|---|---|
| `.label("…")` | a etiqueta de acessibilidade |
| `.placeholder("…")` | o texto de sugestão de um campo |

⛔⛔ **A maior categoria de texto que o artista lê não é nenhuma das duas: é o texto que um pintor
DESENHA** — `paint_text(.., "Select an entity in the Hierarchy…", ..)`. Ele não passa por
`.label` nem por `.placeholder`, logo o gate **nunca o viu**.

*Um gate que enumera dois padrões afirma sobre esses dois, e a prosa à volta dele afirma sobre a
lei inteira.* É a mesma forma do censo de recuo que esta linha já pagou quatro vezes — só que aqui
o que falta não é uma FORMA do que se lê, é uma **população**.

## §2 — O tamanho, contado

Varredura sobre `crates/ph2d-editor-core/src` + todos os `crates/ph2d-panel-*/src`, contando
chamadas `paint_text*(…)` cujo argumento de texto é um **literal**, com os comentários e o bloco
`#[cfg(test)]` fora:

| crate | literais pintados |
|---|---|
| `ph2d-panel-inspector` | **36** |
| `ph2d-editor-core` | 17 |
| `ph2d-panel-painter-layers` | 15 |
| `ph2d-panel-vector` | 8 |
| `ph2d-panel-motion-params` | 7 |
| `ph2d-panel-audio-editor` | 6 |
| `ph2d-panel-flip` | 5 |
| `ph2d-panel-timeline` | 4 |
| `ph2d-panel-color-equalization` | 3 |
| `ph2d-panel-tokens` | 3 |
| `ph2d-panel-widget-lab` | 2 |
| `ph2d-panel-grid-snap` | 1 |
| `ph2d-panel-asset-browser` | 1 |
| **total** | **108** em 13 crates |

⚠️ **A premissa que o gate declara sobre si mesmo já não é verdade.** Ele diz: *«Until the Fluent
runtime (`ph2d-i18n`) is wired and `t!(...)` exists, this test enforces a frozen baseline»* — e o
`ph2d_i18n::tr` **existe e é usado** (o painel de escultura, o modelador 3D e o vetorial têm tabela
própria em `crates/ph2d-i18n/src/`). *O gate está à espera de uma coisa que chegou.*

## §2-bis — ⛔⛔ O número da §2 estava **4× errado**, e a causa é a de sempre

> Corrigido em **2026-09-10**, ao tentar curar a `ph2d-editor-core` e tropeçar em
> `paint_panel_title(rect, "Widget Gallery", …)` — *uma chamada que a §2 não podia ver, porque ela
> só conhecia `paint_text*`.*

⚠️ **Um censo textual tem de saber TODAS as formas do que lê** — é a lição que este repo já pagou
seis vezes, e aqui ela custou um número publicado. Derivadas do fonte, as **portas de texto** (uma
função que recebe um `&str` e o entrega a um pintor, **ou a outra porta** — a definição é
recursiva) são **`127`**, não uma.

| | portas que o censo conhecia | literais |
|---|---:|---:|
| §2 (2026-09-09) | `1` (`paint_text*`) | `108` |
| ponto fixo (2026-09-10) | **`127`** | **`418`** |

| crate | literais pintados | chaves `panel.<id>.*` |
|---|---:|---:|
| `ph2d-panel-painter-layers` | **166** | **0** |
| `ph2d-panel-inspector` | **99** | **0** |
| `ph2d-editor-core` | 52 | — |
| `ph2d-panel-audio-editor` | 34 | **0** |
| `ph2d-panel-grid-snap` | 22 | **0** |
| `ph2d-panel-audio-mixer` | 17 | **0** |
| `ph2d-panel-flip` · `-motion-params` | 8 · 8 | **0** · **0** |
| `-equalize-sizes` · `-color-equalization` · `-vector` · `-bgremoval` | 7 · 6 · 5 · 4 | 0 · 0 · 276 · 0 |
| `-physics` · `-timeline` · `-upscale` | 2 · 2 · 2 | 60 · 48 · 0 |
| `-asset-browser` · `-hierarchy` · `-padding` · `-widget-lab` | 1 cada | 0 |
| **total** | **418** em **19** crates | |

⭐⭐ **E as duas colunas juntas dizem o que nenhuma diz sozinha:** o `painter-layers` pinta `166`
rótulos e declara **zero**; o `inspector`, `99` e **zero**. *Não é que aqueles painéis falem mal a
tabela — é que eles não a usam de todo* (§3-bis).

⚠️ **Por que a §2 lia `108` e uma recontagem estrita de `paint_text*` lê `79`:** nenhuma das duas
declarava quantos **saltos** seguia. O `ph2d-panel-timeline` passa os rótulos por um `label()`
local, a um salto do pintor — invisível a uma leitura e visível à outra. *Um censo que não declara
o seu alcance não é comparável consigo mesmo.*

⭐ **O instrumento fica no repo e o doc chama-o pelo nome** (`CLAUDE.md` §2 — *ferramenta que nenhum
passo escrito chama pelo nome morre*):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && python3 scripts/censo-texto-pintado.py
```

⛔ Ele **imprime e não escreve** — não há `--write`, e não há catraca global (a razão está na §3).

⛔⛔ **E ELE ERROU A MESMA CLASSE DE ERRO NA PRIMEIRA HORA DE VIDA.** A primeira corrida leu `438`;
o número é `418`. A causa: o leitor não conhecia o **literal de carácter** — `find('"')` é Rust
legítimo, e ele lia aquela aspa como uma string a abrir, passando a ler o resto do ficheiro ao
contrário (comentário como texto, texto como código). ⚠️ **O irmão em Rust apanhou-o sozinho:** a
primeira corrida do gate acusou **um comentário do próprio ficheiro do gate**. ⇒ *ter duas
implementações da mesma régua não é redundância — é a única forma de uma delas acusar a outra.*
⚠️ E **declara o que não vê**: literais que chegam por variável, `const`, tabela de `&str` ou
`format!`. ⇒ *`418` é um PISO.*

## §2-ter — ✅ A crate DESTA linha está curada: `52 → 32`, e o resto é declarado

> 2026-09-10. ⚠️ **Só a `ph2d-editor-core`** — as outras 18 são de outras linhas, e a recusa da §3
> continua de pé para elas.

| | antes | depois |
|---|---:|---:|
| literais pintados na `ph2d-editor-core` | `52` | **`32`** |
| deles, **língua de produto** | `20` | **`0`** |
| deles, declarados como **não-língua** | — | `32` |

⭐⭐ **E a migração não foi mecânica: ela COLAPSOU quatro rótulos escritos em dois sítios.**

| rótulo | estava escrito em | ficou |
|---|---|---|
| *No matches* | `context_menu_overlay.rs` **e** `widget/command_palette.rs` | `chrome.no_matches` |
| *Shape options* | `screens/hero/left_rail.rs` **e** `screens/hero/tool_bar.rs` | `chrome.flyout.shape_options` |
| *Mask options* | os mesmos dois | `chrome.flyout.mask_options` |
| *Hex* | `blender_color_picker/hex_field.rs` **e** `.../paint.rs` | `chrome.color.hex` |

⇒ **8 sítios viraram 4 chaves.** *Uma palavra escrita em dois sítios ainda não é uma palavra do
app — só uma PORTA é*: mudar *No matches* tocava um dos dois e deixava o outro a dizer a antiga, e
nenhum gate o via.

### ⛔ Os `32` que ficam são declarados, com o mecanismo

| o quê | quantos | porquê não é língua |
|---|---:|---|
| `widget/showcase/*` | 30 | a **Galeria de Widgets é uma bancada**, e o subtítulo dela di-lo (*«reference for peripheral agents»*). Os rótulos são os **nomes dos nossos tipos** (`Rect2Editor`, `BitmaskGrid32`, `VariantEditor (recursive, depth ≤4)`) — traduzir o nome de um tipo nosso é ruído. Mesmo estatuto do `widget-lab`, que até a aparência força para o redesenho *«porque é onde ele se estuda»* |
| `widget/command_palette/header.rs` | 2 | **símbolos desenhados como letras**: o `X` do fecho e o `x` do visto. O próprio ficheiro já o dizia — *«o mesmo idioma do X de fechar, que também é uma letra»* |

### ⭐ E o gate é o MOLDE para as outras 18 crates

[`no_label_of_this_crate_is_written_in_the_painter`](../../../crates/ph2d-editor-core/tests/no_label_of_this_crate_is_written_in_the_painter.rs)
— **por crate**, nunca global (a razão está na §3), com o mesmo **ponto fixo** da §2-bis
reimplementado em Rust, o controlo de vacuidade e a metade de obsolescência.

⭐⭐ **Duas implementações independentes do mesmo ponto fixo — o script em Python e o gate em Rust —
dão `32` e `32`.** *Uma régua nova que concorda ao número com outra, escrita noutra linguagem por
outro caminho, é uma régua de que se pode duvidar menos.*

## §3 — Por que esta linha MEDIU e não CUROU

⛔ **Não é preguiça, é o custo de merge.** Curar os `418` (§2-bis; a §2 dizia `108`) toca **19 crates**, e onze delas são de
outras linhas — cinco delas vivas em 2026-09-09. A memória do repo já regista o preço desta forma
exacta (*«apagar ~2 961 LOC do `VecInstance` em 24 ficheiros com a `line/Vector` VIVA é catástrofe
de merge»*), e uma migração mecânica larga é o caso pior: diff enorme, zero conflito semântico,
conflito textual em todo o lado.

⛔ **E a catraca também não se arma sozinha.** Um censo com 108 entradas por crate faria a próxima
linha que pinte um rótulo ficar **vermelha na crate dela por causa de um gate desta**. O
`CLAUDE.md` §0.2 pede que o foundational novo seja *projectado para isolamento*; um censo global
com dívida alheia dentro é o contrário disso.

## §3-bis — ⭐⭐⭐ O buraco não é só de CANON: ele CEGA o censo que cura a foto 3

> Medido em 2026-09-10, ao tentar correr o censo da **D2** nos painéis que faltavam.

A `spec/02 §8` dizia, na linha do degrau `G`: *«⛔ **Nenhum outro painel foi censado** — o `66 de
74` é deste»*. Isso lia-se como uma tarefa por fazer. **Não é: é uma tarefa sem instrumento.**

O censo da D2 classifica as **entradas declaradas** de um painel (o que ele oferece) por âmbito —
*app inteiro* · *só este editor* · *propriedade do objecto*. A lista de entradas de um painel é o
vocabulário dele, e ele só existe onde o painel fala pela tabela do `ph2d-i18n`:

| painéis | vocabulário declarado |
|---|---:|
| `vector` | 276 chaves |
| `model3d` | 131 |
| `sculpt3d` | 113 |
| `physics` | 60 |
| `timeline` | 48 |
| `wet_tuning` | 29 |
| `tokens` | 10 |
| **os outros 19** | **ZERO** |

⛔⛔ **`19` de `26` painéis não têm uma única chave `panel.<id>.*`** — e entre eles estão
exactamente os que o artista tem abertos o dia inteiro: `hierarchy`, `inspector`,
`painter_layers`, `motion_graph`, `asset_browser`, `flip`.

⇒ **os dois itens abertos deste módulo são um só.** Enquanto o rótulo de um controlo é um literal
dentro do pintor, *não há lista para classificar*: censar aqueles painéis obriga a ler pintor a
pintor, à mão, sem catraca e sem forma de saber que a leitura ficou completa. *Um censo cuja
população se conta lendo prosa não é um censo — é uma opinião com tabela.*

⚠️ **Isto reordena a §4 abaixo, e sobe o preço de adiar:** migrar os literais deixa de ser dívida
de canon (*«o app não fala outra língua hoje»*) e passa a ser o **pré-requisito do `G`**, que é a
metade por fazer da cura da foto 3 do dono (*«sem menus na barra superior, os painéis incharam»*).

⛔ **O que NÃO muda:** o custo de merge continua a ser o que a §3 mediu — 13 crates, 11 de outras
linhas. A ordem que isto compra é **por painel, começando pelos que o artista abre**, e não uma
varredura mecânica de uma vez.

## §4 — O que fica recomendado, em ordem

1. ✅ **FEITO, e o desenho MUDOU pelo caminho.** Esta linha pedia *«um gate com a baseline por
   crate congelada, na janela de integração»* — e a §3 do próprio doc explica porque isso não
   serve: **uma baseline com a dívida de 19 crates dentro põe a próxima linha vermelha por causa de
   um gate desta.** ⇒ o que existe é
   [`no_label_of_this_crate_is_written_in_the_painter`](../../../crates/ph2d-editor-core/tests/no_label_of_this_crate_is_written_in_the_painter.rs),
   **por crate, a zero, na crate desta linha**, com as excepções nomeadas e o mecanismo de cada
   uma. ⭐ **Cada linha dona de uma crate copia-o com a lista dela** — e nesse dia nenhuma delas
   tem de editar o teste de outra pessoa. *Uma catraca partilhada e uma catraca replicada medem a
   mesma coisa; só a segunda respeita o isolamento.*
2. **Curar por CRATE, pela linha dona de cada uma** — ✅ a `ph2d-editor-core` fechou (§2-ter);
   ⏳ faltam **18**, e a `ph2d-panel-painter-layers` sozinha (`166`) é `40 %` do que resta. ⭐ **E a ORDEM entre as crates sai da §3-bis, não do tamanho delas:** primeiro os
   painéis que o artista tem abertos o dia inteiro (`hierarchy`, `inspector`, `painter_layers`),
   porque é ali que o censo do degrau `G` fica cego — *curar o maior primeiro é ordenar por
   esforço; curar o mais aberto primeiro é ordenar por resposta.*
3. ✅ **FEITO (2026-09-10) — a prosa do gate foi corrigida.** Ela dizia *«Until the Fluent
   runtime is wired and `t!(...)` exists»*, e a porta **já tinha chegado**. O
   [`hr15_no_hardcoded_ui_strings`](../../../crates/ph2d-editor-core/tests/hr15_no_hardcoded_ui_strings.rs)
   passa a nomear **as duas metades** e a dizer que nenhuma delas sozinha é o HR-15: ele mede o
   rótulo de acessibilidade e o *placeholder* em `src/widget/`; o texto **pintado** tem gate
   próprio. *Um gate à espera de uma coisa que já chegou é a frase que faz a próxima pessoa
   acreditar que a lei está fechada.*

## ⛔ Recusas MEDIDAS

| o que | por que não |
|---|---|
| migrar os 108 nesta janela | 13 crates, 11 de outras linhas, 5 vivas — catástrofe de merge por diff mecânico |
| censar o degrau `G` nos outros 25 painéis **antes** de os literais migrarem | ⛔ **impossível com instrumento**: `19` de `26` não declaram uma única entrada (§3-bis), logo o censo teria de ler pintor a pintor — sem catraca e sem saber quando acabou |
| armar a catraca global agora | reprovaria outras linhas na crate delas por um gate desta; §0.2 pede isolamento |
