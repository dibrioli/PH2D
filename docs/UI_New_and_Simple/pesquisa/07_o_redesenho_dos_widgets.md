# O redesenho dos widgets — plano e minimalista, e COMPACTO (2026-09-01)

> **Fase de PESQUISA.** Nada aqui está construído. O Enio pediu quatro coisas numa mensagem só, e
> este documento responde às quatro com número ao lado, para a discussão que vem a seguir.
>
> Irmãos: [`01_o_que_a_subida_abriu.md`](01_o_que_a_subida_abriu.md) (o que o stack novo trouxe) ·
> [`03_diagnostico_e_principios.md`](03_diagnostico_e_principios.md) (o diagnóstico das três fotos) ·
> [`medicoes/03_o_censo_de_cor.md`](../medicoes/03_o_censo_de_cor.md) ·
> [`medicoes/06_o_orcamento_de_ecra_em_tablet.md`](../medicoes/06_o_orcamento_de_ecra_em_tablet.md)

## §0 — O pedido, em quatro perguntas

| # | o pedido (palavras dele) | responde-se em |
|---|---|---|
| P1 | *"redesenho de cada Widget do app buscando um padrão mais plano e minimalista como godot e blender"* | §1, §3, §7 |
| P2 | *"como construir os widgets de forma que os painéis tenham boa aparência e bom funcionamento mesmo se forem extremamente estreitos"* | §2, §4, §5, §6 |
| P3 | *"embora planos e simples precisamos estudar a beleza dos widgets. Quero um app bonito"* | §7 |
| P4 | *"Atualizamos o Vello/WGPU. O que há neles de mais moderno que pode nos ajudar?"* | §8 |

⚠️ **O exemplo que ele deu é o eixo de tudo**, e está medido no §2:

```
nosso    [ Opacity ]  [ ═══════════○────── ]  [ 100 ▲▼ ]     ← três colunas
Blender  [ Normal Offset            0 m     ]                 ← duas, o número DENTRO
alvo     [ Opacity                  100     ]                 ← UMA, rótulo E número dentro
```

---

## §1 — ⭐⭐⭐ O diagnóstico: a nossa linguagem é a do **Procreate**, e ele está a pedir a do **Blender**

Isto não é opinião — está escrito no código e nos tokens.

**A prova textual.** O doc-comment do nosso slider abre assim
([`slider.rs:1-8`](../../../crates/ph2d-editor-core/src/widget/slider.rs)):

> *"Procreate uses sliders for brush size, opacity, flow. Same pattern as Button."*

**A prova numérica.** Raio de canto, nós contra a referência que ele nomeou:

| | raio do painel | raio do controlo | fonte |
|---|---:|---:|---|
| **nós** | **16 px** | 4–12 px, e `full: 999` para pílulas | `tokens.json` `chrome.panel-radius`, `radius.*` |
| **Godot** (editor) | **4 px** | 4 px (tema *Classic*: **3**) | `editor_theme_manager.h:89` `default_corner_radius = 4` |

⇒ **os nossos painéis são 4× mais redondos que os do Godot**, e temos uma família inteira de
pílulas a `999` que a linguagem-alvo simplesmente não tem.

**A prova de espaçamento.** O Godot deriva TUDO de um número (`base_spacing`), com três presets:

| preset Godot | `base_spacing` | `extra_spacing` | botão de diálogo |
|---|---:|---:|---|
| Compact | 2 | 2 | 90 × 26 |
| Default | 4 | 0 | 105 × 34 |
| Spacious | 6 | 2 | 112 × 36 |

Nós temos **nove** degraus de espaçamento (`2 4 6 8 12 16 24 32 48`) e **três** de densidade
(`22 / 26 / 32`), e ⚠️ **o default é `Comfortable` (32 px)** — o mais gordo dos três.

⭐ **A conclusão do §1:** o pedido não é «mudar umas cores». É **trocar a família tipológica** — de
uma linguagem *touch-first, redonda, com ar* (Procreate/iOS) para uma *density-first, quadrada,
sem ar* (Blender/Godot). ⚠️ E há uma tensão real, declarada já: **o alvo é tablet**, onde o alvo de
toque quer ser grande. O §6 resolve-a separando **altura de linha** (que é toque) de **cromo
horizontal** (que é desperdício).

---

## §2 — ⭐⭐⭐ O orçamento MEDIDO de uma linha de propriedade

A linha da foto é [`slider_with_chip.rs`](../../../crates/ph2d-editor-core/src/widget/slider_with_chip.rs).
As constantes são dela:

| peça | px | constante |
|---|---:|---|
| coluna do rótulo | **70** | `DEFAULT_LABEL_W` |
| folga | 6 | `Spacing::Sm` |
| **trilho** (o controlo) | *o que sobrar* | — |
| folga | 6 | `Spacing::Sm` |
| caixa numérica | **72** | `DEFAULT_CHIP_W` = `number_input::MIN_W_PX` |
| trilho mínimo antes de partir | 60 | `SLIDER_CHIP_MIN_SLIDER_W` |

⇒ **cromo fixo = 70 + 6 + 6 + 72 = `154 px`**, e o limiar de empilhamento é `154 + 60 = `**`214 px`**.

O corpo do Inspector é `inner_w = largura − 2×10 (BODY_PAD) − 16 (barra de rolagem)`. Logo:

| painel | `inner_w` | trilho | **% da linha que é controlo** | resultado |
|---:|---:|---:|---:|---|
| 308 (Hierarquia) | 272 | 118 | 43,4 % | uma linha |
| **304 (Inspector, o default)** | **268** | **114** | **42,5 %** | uma linha |
| 260 | 224 | 70 | 31,2 % | uma linha |
| **220 (`PANEL_MIN_W`)** | **184** | — | — | ⛔ **EMPILHA: 2 linhas** |
| 180 | 144 | — | — | ⛔ empilha |

### ⛔⛔ Os três achados

1. **No tamanho de omissão, 57,5 % da linha é cromo.** O artista arrasta 114 px de 268.
2. **⛔ Na largura mínima que o próprio app permite (`220`), TODO slider já está em duas linhas.**
   O `PANEL_MIN_W_PX = 220` e o limiar de empilhamento é `214` sobre `inner_w` — que a 220 vale
   `184`. *A cura de estreiteza que temos hoje custa o dobro da altura*, que é o recurso mais
   escasso num painel de tablet.
3. **⛔⛔ O `Vector3Editor` não tem cura nenhuma e fura o próprio mínimo declarado.**
   `field_rects` divide por três com `.max(0.0)` e **sem piso**
   ([`vector3_editor.rs:62`](../../../crates/ph2d-editor-core/src/widget/vector3_editor.rs)):
   precisa de `3×72 + 2×8 = 228 px` e a `220` recebe `184` ⇒ cada campo fica com **56 px**,
   **22 % abaixo** do `MIN_W_PX = 72` que a própria casa declara. O `Rect2Editor` em linha precisa
   de `306` — ele foi salvo por ter ganho um `Grid2x2` próprio.

### ⛔ E o achado estrutural: **não existe LEI do estreito**

Censo das 44 fontes de widget, por menção de vocabulário de adaptação
(`adaptive|is_stacked|narrow|reflow|Grid2x2|demote|overflow`):

| widget | menções | tem mesmo um caminho? |
|---|---:|---|
| `segmented_adaptive` | 35 | ✅ reflui |
| `slider_with_chip` | 20 | ✅ empilha (e é o que custa altura) |
| `rect2_editor` | 13 | ✅ `Grid2x2` |
| `scrollbar` | 11 | ✅ |
| `panel_chrome` · `command_palette` · `status_bar` · `number_input` · `skin` | 2–5 | parcial |
| **os outros ~35** | **0** | ⛔ **nenhum** |

⇒ **quatro widgets inventaram cada um a sua cura, e trinta e cinco não têm nenhuma.** É por isso
que «painel estreito» hoje não é uma propriedade do sistema — é uma lotaria por widget.

---

## §3 — O que as referências FAZEM (medido nas fontes vendorizadas)

### 3.1 — Blender, do manual dele (`referencias/blender-manual/.../buttons/fields.rst`)

> *"**Sliders**, a second type of number field, have a **colored bar in the background** to display
> values over a range."*

> *"The first type of number field shows triangles pointing left (<) and right (>) on the sides of
> the field **when mouse pointer is over the field**."*

⭐⭐ **Duas leis, as duas directamente úteis:**
- **um slider NÃO é um controlo diferente de um campo numérico** — é o mesmo campo com uma barra
  colorida atrás. Uma caixa, não três. *É literalmente o que o Enio desenhou na mensagem.*
- **os steppers (▲▼) só existem no hover.** Nós pintamo-los sempre, e eles são parte dos 72 px.

E o resto do mesmo ficheiro, que é o que torna a caixa única **suficiente**:
- arrastar sobre o campo edita (não é preciso um trilho separado);
- `Ctrl` = passos discretos, `Shift` = precisão;
- clicar/`Return` transforma-o num campo de texto — **o mesmo pixel é o trilho e a caixa**;
- aceita **expressões** (`3*2`, `sqrt(2)`) e **unidades** (`1m 3mm`, `2ft`);
- edição **multi-valor**: premir e arrastar na vertical por vários campos edita-os todos.

⚠️ E o ponto ao lado da caixa na foto dele **tem nome**: é um **decorator**
(`buttons/decorators.rst`) — um botão minúsculo à direita que mostra se a propriedade é animada,
e que grava keyframe ao clique. *Não é decoração: é a coluna de animação.* Nós temos timeline e
não temos essa coluna.

### 3.2 — Godot, do fonte dele (`referencias/godot-editor-src`)

- `default_corner_radius = 4` (`editor_theme_manager.h:89`), *Classic* = 3.
- Todo o tema deriva de `base_spacing` + `extra_spacing` (§1), incluindo
  `separation_margin`, `popup_margin = base×2.4`, `window_border_margin`.
- `forced_even_separation = increased_margin + 6`, **forçado a par** — com o comentário a dizer
  porquê: *"if the vsep is odd … it will be lopsided"*. ⭐ *Um sistema de espaçamento que se
  preocupa com a paridade do pixel é o nível de cuidado que «plano» exige* — sem sombra e sem
  gradiente, o desalinhamento de 1 px passa a ser a única coisa que se vê.
- `border_width` é `CLAMP(0..2)` — a borda é **0, 1 ou 2 px**, nunca 1,5.
  ⚠️ Nós temos `stroke.default = 1.5`.

⏳ **Buraco nomeado:** o `EditorSpinSlider` — o widget do Godot que é exactamente o alvo (rótulo
dentro à esquerda, valor dentro à direita, barra fina de preenchimento) — **não está na árvore
vendorizada** (só `scene/gui/`, não `editor/gui/`). O `fetch-referencias.sh` existe; se quisermos
citar a anatomia dele com números em vez de memória, é um `fetch` de um ficheiro.

---

## §4 — ⭐⭐⭐ A lei da compactação, e o widget unificado

### 4.1 — A lei, numa frase

> **Uma linha de propriedade é UMA caixa. O rótulo, o valor e o controlo vivem DENTRO dela;
> o que não cabe encolhe pela ordem `rótulo → unidade → valor`, e o controlo nunca encolhe,
> porque o controlo É a caixa.**

O que ela troca, medido contra o §2:

| | hoje | com a lei |
|---|---:|---:|
| cromo horizontal fixo | **154 px** | **~16 px** (o padding da caixa) |
| % da linha que é arrastável a 304 | 42,5 % | **100 %** |
| largura mínima antes de partir | 214 px | *não parte* — degrada por conteúdo (§6) |
| altura a `PANEL_MIN_W = 220` | **2 linhas** | **1 linha** |

⇒ num Inspector com 12 sliders a `220 px`, a altura gasta cai de ~24 linhas para **12**.

### 4.2 — A anatomia proposta

```
┌───────────────────────────────────────────┐ ·
│▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│ ·   ← 1. a barra de valor É o fundo
│  Opacity                            100 % │ ·   ← 2. rótulo à esquerda, valor à direita
└───────────────────────────────────────────┘ ·   ← 3. decorator, FORA, 1 coluna de ~14 px
      ↑ arrastar aqui edita          ↑ clicar aqui escreve
```

1. **O preenchimento é o fundo**, não um trilho separado (Blender). Sem `track`, sem `thumb`.
   ⚠️ **O thumb morre** — e é ele que hoje obriga a `SLIDER_CHIP_MIN_SLIDER_W = 60`.
2. **Rótulo e valor são duas âncoras de texto na mesma linha de base**, dentro do mesmo rect.
3. **O decorator é uma coluna do FORMULÁRIO, não do widget** — largura fixa, opcional por painel.
   ⭐ É onde a nossa timeline finalmente encosta na Inspector (hoje não há coluna de animação).

### 4.3 — ⛔ O que se perde, dito antes de alguém descobrir

| perda | é real? | mitigação |
|---|---|---|
| o valor deixa de ser um alvo de clique separado | **sim** | clique curto = editar (Blender); arrastar = escorregar. É um **gesto por duração**, não por região |
| não há mais ▲▼ permanentes | sim | aparecem no hover (Blender) — ⛔ **e num tablet não há hover**: ver §6.4 |
| rótulos longos sobre valores longos colidem | **sim** | §6.2 (a escada de degradação) e §7.3 (o esbatimento) |
| um slider e um campo numérico passam a ser o MESMO widget | é o objectivo | ⭐ mata `slider_with_chip` (497 LOC) e funde-o com `number_input` (459) |

### 4.4 — O raio de explosão, contado

| porta | chamadas | ficheiros |
|---|---:|---:|
| `paint_slider_with_chip*` | **70** | **39** |
| `paint_number_input` / `NumberInput::new` | **92** | — |

⇒ **162 sítios**. ⚠️ **E é por isso que a forma da obra importa mais que o desenho:** se a
assinatura se mantiver, os 162 herdam o desenho novo sem serem tocados. *A porta única já existe
e já foi a cura de um bug desta família* (o doc-comment do `slider_with_chip` diz que ele nasceu
porque *"the slider in panel X looks different from the one in panel Y"*).

---

## §5 — O estudo da compactação, widget a widget

Os **44** ficheiros de `widget/` mais os 10 sub-módulos. Coluna «hoje» = medido; coluna
«compacto» = **proposta**, não medida.

### 5.1 — Numéricos — *onde está quase todo o ganho*

| widget | hoje | compacto | perde |
|---|---|---|---|
| `slider` + `slider_with_chip` + `number_input` | 3 widgets, 3 colunas, 154 px de cromo | ⭐⭐⭐ **um só**: rótulo+valor dentro da barra (§4) | thumb, steppers permanentes |
| `numeric_input_with_unit` | campo + sufixo de unidade | a unidade entra no texto do valor (`100 %`, `1,5 m`) — ⭐ é o que o Blender faz | uma coluna |
| `vector3_editor` | 3 campos em linha, **sem piso** (§2) | 3 caixas unificadas; abaixo de `3×MIN` **empilha em 3 linhas rotuladas `X Y Z`** | largura por campo |
| `rect2_editor` | `Row` \| `Grid2x2` | mesma escada, mas com a caixa unificada o `Grid2x2` chega muito mais tarde | — |
| `bitmask_grid32` | grelha 4×8 fixa | 8×4 quando mais alto que largo | — |

### 5.2 — Escolha — *o segundo maior ganho*

| widget | hoje | compacto | perde |
|---|---|---|---|
| `checkbox` | caixa 18 px **+ rótulo à direita** | ⭐ **a linha inteira é o alvo**, marca à direita (padrão Blender/GNOME) — o rótulo deixa de ter coluna própria | nada |
| `toggle` | interruptor deslizante | ⭐ vira **caixa de verificação**: um interruptor custa ~2× a largura e diz o mesmo | a animação de deslizar |
| `radio_group` | N linhas | **segmentado** quando N≤4 e couber; senão dropdown | — |
| `tabs` / `segmented_adaptive` | já reflui | manter; unificar com `radio_group` (são o mesmo modelo) | — |
| `dropdown` / `combobox` | caixa + seta | seta de 8 px, rótulo esbate no fim (§7.3) | — |
| `pill_group` | pílulas `radius: 999` | ⛔ **`999` sai** — 4 px, e as pílulas viram segmentos encostados | o look «iOS» |

### 5.3 — Contentores e cromo — *onde a linguagem se vê*

| widget | hoje | compacto |
|---|---|---|
| `card` | superfície + cabeçalho + corpo + moldura | ⭐ **a moldura desaparece**: um cartão passa a ser um cabeçalho + um recuo. Blender não desenha caixas dentro de painéis |
| `section_header` | cabeçalho | ⭐ **chevron + maiúsculas + 1 px de linha**, e **recolhível** — a peça que falta para painéis estreitos |
| `divider` | linha 1 px | manter; é a única moldura que sobrevive |
| `panel_chrome` | raio **16** | **4** (§1) |
| `modal` / `popover` / `tooltip` | flutuam | manter; sombra é o único sítio onde ela se justifica (§7.4) |
| `scrollbar` | 10 px sempre | ⭐ **2 px em repouso, 10 no hover/arrasto** — devolve 8 px a cada painel, em TODOS eles |

### 5.4 — Listas, texto, estado

| widget | hoje | compacto |
|---|---|---|
| `list_item` / `tree_view` / `context_menu` | linha com ícone+texto+acessório | ⭐ o **recuo** da árvore cai de nível para `8 px`; ícone só se distinguir |
| `key_value_list` | chave \| valor \| remover | duas caixas unificadas + remover no hover |
| `text_input` / `text_area` | moldura permanente | ⭐ **fundo, sem moldura**; a moldura só no foco |
| `progress_bar` / `level_meter` / `spinner` | — | já são planos; só perdem raio |
| `tag` / `avatar` / `color_swatch` | `radius: 999` / círculo | 4 px — ⚠️ **excepto `avatar`**, onde o círculo é semântico |
| `status_bar` | pílula | barra encostada |
| `radial_menu` · `command_palette` · `variant_editor` · `skin` · `color_picker` | — | ⛔ **fora deste estudo**: não vivem em painel estreito |

### 5.5 — ⭐ O que este estudo APAGA

Se a lei do §4 passar, três widgets deixam de existir por fusão
(`slider` + `slider_with_chip` + `number_input` → um) e dois por absorção
(`toggle` → `checkbox`, `radio_group` → `segmented_adaptive`).
⇒ **de 20 `WidgetKind` para ~16**, com menos código e uma linguagem mais estreita.
⚠️ **Mas `WidgetKind::code()` é um número que viaja no documento** e o doc-comment dele proíbe
reciclar códigos: a fusão é **de pintura**, e os códigos velhos ficam a apontar para o pintor novo.

---

## §6 — A lei do ESTREITO: os degraus

⚠️ Hoje a nossa única resposta a «estreito» é **empilhar**, e empilhar troca a largura (que falta)
pela altura (que falta mais). A escada proposta gasta altura **por último**.

### 6.1 — Os degraus, por ordem de aplicação

| # | degrau | custo | exemplo |
|---|---|---|---|
| 1 | **fundir colunas** | zero | o slider do §4 |
| 2 | **encolher o rótulo** (abreviar, depois esbater) | zero | `Geometry Offset` → `Geom. Offset` → `Geometry Off⋯` |
| 3 | **esconder o acessório** (steppers, unidade, decorator) | zero | §3.1 |
| 4 | **refluir** (N em linha → grelha) | pouca altura | `Rect2Layout::Grid2x2`, que já existe |
| 5 | **empilhar** | **dobra a altura** | o que fazemos hoje, em primeiro |
| 6 | **recolher a secção** | esconde | `section_header` do §5.3 |

### 6.2 — O rótulo é o que cede, e cede por uma ordem

⛔ **O valor NUNCA se trunca** — um número cortado é um número errado. Trunca-se o rótulo, porque
o utilizador sabe em que secção está. *É a inversão exacta do que fazemos hoje*, onde o rótulo tem
uma coluna fixa de 70 px e o trilho é que encolhe até desaparecer.

### 6.3 — ⭐ E daqui sai um número que ainda não temos

`PANEL_MIN_W_PX = 220` foi escolhido **antes** de haver alvo tablet. Com a lei do §4 o mínimo
funcional passa a ser *"cabe o valor mais largo + um rótulo de 3 caracteres"*, que é da ordem de
**~120 px**. ⏳ **Isso é para MEDIR, não para escrever de memória** — e a medição precisa do
`TextSystem`, porque depende da fonte.

### 6.4 — ⛔⛔ A tensão do tablet, declarada

O Blender esconde no **hover**, e **num tablet não há hover**. Três saídas, nenhuma medida:

| saída | preço |
|---|---|
| acessórios sempre visíveis em toque | perde o ganho do degrau 3 |
| acessórios no **toque longo** | um gesto a mais para aprender |
| ⭐ acessórios ligados ao **modo de densidade** (`Comfortable` = sempre; `Compact` = hover) | o `Density` já existe e já é preferência do artista |

⇒ a terceira é a que reaproveita máquina que temos. **É decisão de produto.**

### 6.5 — E a altura NÃO segue a largura

⚠️ `Density` default é `Comfortable = 32 px` **por causa do Pencil**, e isso está certo. A
compactação deste estudo é **horizontal**. ⛔ Não se corta altura de linha para caber num tablet —
corta-se cromo lateral, que não é alvo de toque de ninguém.

---

## §7 — A beleza, quando a caixa desaparece

> *"Embora planos e simples precisamos estudar a beleza dos widgets. Quero um app bonito."*

⭐⭐⭐ **A tese:** num desenho plano não há sombra, gradiente nem moldura para esconder erro.
Sobram **quatro** portadores de beleza, e os quatro são medíveis.

### 7.1 — O alinhamento é a decoração

Sem molduras, a única coisa que dá ordem é **as coisas alinharem**. É por isso que o Godot força a
paridade do espaçamento (§3.2). ⇒ **uma grelha vertical de 4 px** para tudo, e cada linha de
propriedade com a **mesma linha de base de texto** — hoje o rótulo (num rect de altura de linha) e
o chip (centrado no seu próprio rect) são centrados **separadamente**, e basta 1 px de diferença
para a coluna parecer suja.

### 7.2 — A cor faz o trabalho da moldura, e faz-se por passos IGUAIS

Sem moldura, a hierarquia é **luminância**. ⭐ **E nós já temos a máquina certa:**
`ph2d_tokens::{oklch_to_srgb, srgb_to_oklch, ColorValue::from_oklch}` (`color.rs`), com **45
ficheiros** a referi-la. ⇒ os degraus de fundo (`bg` → `bg-elev` → `bg-input`) e os estados
(hover, press, disabled) podem ser **`ΔL` constante em Oklch** em vez de hex escolhidos à mão.

⚠️ **O censo de cor já disse que isto é uma dívida**: 83 slots por tema, **16 são apelidos puros**,
e o tema `workshop` prova que 16 declarações chegam. *Um sistema plano precisa de menos cores e de
passos mais exactos, que é exactamente a mesma obra.*

### 7.3 — ⭐⭐ O esbatimento em vez do `…`

O degrau 2 do §6.1 pede truncar o rótulo. Um `…` é feio e come 3 caracteres.
**O `vello` 0.10 tem a peça exacta e não a usamos: `Scene::push_luminance_mask_layer`**
(`scene.rs:154`) — o texto desaparece num gradiente de alfa nos últimos ~16 px da caixa.
⇒ nada de `…`, nada de recontar caracteres, e lê-se melhor. **Zero consumidores hoje.**

### 7.4 — A sombra passa a ser rara, e por isso tem de ser boa

Num desenho plano só **três** coisas flutuam: `modal`, `popover`, `tooltip`.
⭐ `Scene::draw_blurred_rounded_rect` / `_in` (`scene.rs:256`/`282`) faz sombra desfocada
**nativamente**, sem textura e sem passe extra. **Zero consumidores hoje.**

### 7.5 — E o texto pequeno tem de ser NÍTIDO

A 10–13 px (toda a nossa escala: `xxs 10 … base 13`) a beleza **é** a nitidez. Já temos o eixo
(`TypePreset` com `hint` + snap-X, `paint_text.rs:152`), e ⚠️ **o `hint` do `vello` tem default
`false`** — quem pintar texto por fora daquela porta perde-o em silêncio.

---

## §8 — O que a subida do stack dá a ESTE trabalho

> Complementa o [`01_o_que_a_subida_abriu.md`](01_o_que_a_subida_abriu.md), que mediu a subida em
> geral. Aqui só o que serve **ao redesenho dos widgets**. Consumo verificado por `grep` hoje.

| capacidade | crate | serve para | usamos? |
|---|---|---|:--:|
| `push_luminance_mask_layer` | vello 0.10 | ⭐⭐ o **esbatimento** do rótulo truncado (§7.3) — a peça que a lei do estreito precisa | ⛔ **0** |
| `draw_blurred_rounded_rect(_in)` | vello 0.10 | sombra nativa para as 3 superfícies que flutuam (§7.4) | ⛔ **0** |
| `push_clip_layer` com **`Stroke`** | vello 0.10 | recortar por um **contorno** — anéis de foco e realces sem segunda geometria | ⛔ **0** |
| `OverflowWrap` · `TextWrapMode` · `WordBreak` | parley 0.11 | a escada de degradação do rótulo (§6.2) sem contar caracteres à mão | ⛔ **0** |
| `Cluster::from_point_exact` | parley 0.11 | ⭐ clicar dentro da caixa unificada e cair no **carácter certo** (§4.2 põe o cursor de texto onde antes havia trilho) | ⛔ **0** |
| `RunMetrics::cap_height` / `x_height` | parley 0.11 | ⭐⭐ o **alinhamento óptico** do §7.1 — centrar por altura de maiúscula, não por caixa da fonte | ⛔ **0** |
| `PlainEditor` (`parley::editing`) | parley 0.11 | a metade «editar» da caixa unificada, já paga | ⛔ **0** |
| `HueDirection` · `lerp` em qualquer espaço · `gradient()` | color 0.3 | ⭐⭐ os degraus de estado por `ΔL` em Oklch (§7.2) | ⛔ **0** (mas temos Oklch próprio) |
| `font_embolden` | vello 0.10 | 2.ª alavanca de peso a 11 px (a 1.ª é o eixo `wght`) | ⛔ **0** |
| gradiente em alfa não-pré-multiplicado | vello 0.10 | o esbatimento do §7.3 não escurece a meio | ⭐ de graça |
| binning > 256 bins | vello 0.10 | uma UI densa **é** uma cena grande — era um defeito alcançável | ⭐ de graça |
| `SurfaceConfiguration::color_space` | wgpu 29 | gama larga / HDR | ⛔ **0**, e ⛔ **fora desta etapa** |

⭐⭐⭐ **A resposta ao P4, em uma frase:** *a subida quase não trouxe formas novas de desenhar
caixas — trouxe o motor de TEXTO e o de COR, que são exactamente os dois materiais de que uma UI
plana é feita.* Os três itens marcados ⭐⭐ acima (`cap_height`, `luminance_mask`, Oklch) são as
peças que fazem a diferença entre «plano» e «plano e bonito», e **nenhum está a ser usado**.

---

## §9 — O que está MEDIDO, o que é PROPOSTA, e o que falta medir

⚠️ Separado de propósito: este documento vai ser lido por quem implementa.

**Medido** (número + ficheiro no texto): o cromo de 154 px e o limiar de 214 · a tabela de larguras
de painel · os 42,5 % · o empilhamento a `PANEL_MIN_W` · o `Vector3Editor` sem piso · o censo dos
35 widgets sem caminho estreito · os raios 16 vs 4 · os presets de espaçamento do Godot · as duas
leis do manual do Blender · os 162 sítios de chamada · o consumo zero das 9 capacidades do §8 · o
Oklch que já temos.

**Proposta** (desenho, não medição): a anatomia do §4.2 · a escada do §6.1 · a tabela por widget do
§5 · as fusões do §5.5 · os quatro portadores de beleza do §7.

⏳ **Por medir, e cada um bloqueia uma decisão:**

| o que | porque bloqueia |
|---|---|
| a largura real dos nossos rótulos com o `TextSystem` | o `~120 px` do §6.3 é uma estimativa, e o mínimo do painel sai daí |
| ⛔ o gesto **clique-curto vs arrasto** num tablet com Pencil | é a fundação do §4.2; se falhar, a caixa unificada não pode ser editável por texto |
| o `EditorSpinSlider` do Godot (não vendorizado, §3.2) | citaríamos a anatomia com número em vez de memória |
| o custo de um `push_luminance_mask_layer` por linha de painel | 25 painéis × N linhas é muita camada; pode ser preciso limitá-lo a quem trunca |
| quantas das 70+92 chamadas passam um `label_w`/`chip_w` próprio | decide se a obra é uma porta ou 162 sítios |

---

## §10 — Para a discussão (as perguntas que são dele)

1. **A caixa unificada é o alvo?** (§4.2) — é a mudança de que tudo o resto depende.
2. **Quão longe vai o «plano»?** Raio `16 → 4` é o Godot. ⚠️ As pílulas a `999` e o `toggle`
   deslizante são as duas peças mais «Procreate» que temos: saem?
3. **A coluna de animação** (o *decorator* do Blender, §3.1) — queremo-la? É a ponte entre o
   Inspector e a timeline, e reserva largura em todos os painéis.
4. **O gesto de acessório em tablet** (§6.4): sempre visível, toque longo, ou ligado à densidade?
5. **Ordem da obra.** A recomendação da linha é: **(a)** a caixa unificada (§4) — é onde está o
   ganho e ela sozinha resolve o §2 inteiro; **(b)** a escada do estreito (§6) como **lei com gate**,
   não como cura por widget; **(c)** os tokens (raio, espaçamento, Oklch) — ⚠️ que tocam **tudo** e
   por isso vêm depois de a forma estar decidida, não antes.

⛔ **Nada disto começa antes de ele escolher** — o §5 apaga widgets e o §9 diz o que ainda não foi
medido.

---

## §11 — ✅ AS DECISÕES DO ENIO (2026-09-01) e a BANCADA que elas abriram

As perguntas do §10 foram respondidas no mesmo dia:

| # | pergunta | resposta dele |
|---|---|---|
| 1 | a caixa única é o alvo? | ⭐ **"A caixa única é o alvo!"** |
| 2 | as pílulas e o interruptor deslizante saem? | ⭐ **"podem sair"** |
| 3 | a coluna de animação? | ⭐⭐ **"Sim. Em todas as propriedades que podem ser animadas — e nessa engine vou querer animar tudo."** |

E duas instruções que mudam a **forma** da obra:

> ⛔⛔ *"Fizemos um excelente trabalho com as fontes do nosso app e isso custou muito esforço e
> tempo. Talvez a versão nova do Vello consiga resultados melhores, mas por precaução vamos manter
> o que temos e colocar o que o Vello consegue fazer como **mais uma opção**. Cuidado para não
> destruir o que temos."*

> ⭐ *"Antes de modificar as coisas seria interessante criar um painel de testes como fizemos com
> Widget Gallery. Vamos fazer nossos estudos num painel antes de sair mudando tudo. Precisamos de
> um estudo sobre widgets originais, desenhos diferentes, podemos colocar no painel várias amostras
> e várias cores e comportamentos para testar."*

### 11.1 — ⭐⭐⭐ A bancada existe: `ph2d-panel-widget-lab`

Menu **Window › Widget Lab**. Cinco secções, na ordem em que uma decisão de desenho se toma:

| § | secção | a pergunta |
|---|---|---|
| 1 | os **seis desenhos** | qual é o *look* |
| 2 | ⭐ a **régua de largura** (`268 · 184 · 140 · 110`) | ele aguenta um painel estreito? |
| 3 | os **estados** (normal · hover · drag · off · typing) | vê-se que é interactivo |
| 4 | as **cores** (6 acentos) | o acento funciona em todos |
| 5 | ⭐⭐ o **widget de HOJE**, lado a lado, com o aviso `EMPILHOU` | estamos mesmo a melhorar |

Mais uma **caixa viva** que se arrasta — e que é um `InteractiveState::Slider` a sério, conduzido
pelo despacho de ponteiro do produto. *Uma bancada com arrasto próprio mediria o arrasto da bancada.*

Os seis desenhos, e o que cada um TROCA (a coluna que interessa é a última):

| # | nome | o preenchimento é | perde |
|---|---|---|---|
| 1 | `Bar` | o fundo inteiro (Blender) | o número compete com a barra |
| 2 | `Underline` | uma linha de 2 px em baixo | a fracção fica discreta |
| 3 | `Inset` | uma cápsula num sulco | gasta altura em molduras |
| 4 | `Ghost` | fundo ténue + linha de base | mal se vê que arrasta |
| 5 | `Notch` | fundo + marca vertical no valor | mais um elemento por linha |
| 6 | `Split` | fundo, rótulo **FORA** | ⚠️ **controlo negativo** — volta a ter coluna fixa |

⚠️ **A `Split` é o desenho do Blender** (duas colunas) e está lá **de propósito**: a decisão foi ir
mais longe que ele, e tê-la ao lado torna a decisão **verificável** em vez de lembrada.

### 11.2 — ⛔ O que a bancada NÃO faz, e por que isso é a metade importante

**Nada nela é chamado pelo app.** Ela desenha as próprias amostras com os primitivos e ⛔ **não
reaproveita o `slider_with_chip`** — ele é o que está a ser substituído, e herdar a geometria dele
importaria a decisão que se quer refazer. Os **162 sítios de chamada** do produto estão intactos.

⚠️ **E ela não é a Widget Gallery.** A galeria é a *fonte única de verdade* do que o editor **é** —
agentes periféricos copiam a decoração dela. Se as propostas vivessem lá, a fonte de verdade
passaria a conter candidatos, e a próxima linha copiaria um.

### 11.3 — ⚠️ Três gates de outras linhas apanharam esta obra, e os três estavam certos

*Registar um painel não o põe no ecrã*, e as três provas disso vieram de graça:

| gate | o que apanhou |
|---|---|
| `every_registered_panel_is_reachable_by_the_z_order_walk` | o painel nascia **registado e nunca pintado** — o menu abriria uma janela invisível. **Sétima vez** que aquele ficheiro paga o mesmo defeito, e a primeira em que o gate o apanha antes do smoke |
| `every_painted_menu_row_is_registered_and_therefore_clickable` | a linha do menu existia e o id **não estava no store** ⇒ o `Click` nunca nasceria |
| `no_magic_numeric_in_widget_or_screens` | ⭐ ele varre **as crates-irmãs**, não só a `editor-core` — 9 literais meus |

⭐ E um quarto gate ficou **melhor por ter falhado**: o `every_toggle_row_of_the_bar_is_marked_by_…`
afirmava `covered == 16` à mão, e uma linha de menu nova punha-o vermelho **sem nada estar errado**.
Passou a ser **derivado** (`|Window| + 3`). *Um número literal ali obriga cada painel novo a editar
o gate de outra pessoa, e o sinal que isso produz é ruído com cara de defeito.*

### 11.3-bis — ⛔⛔⛔ E o painel **não abriu**: ele estava fora do BINÁRIO

Report do Enio, no mesmo dia: *«Widget Lab não abriu»*. Com **10 gates verdes**, o painel no
`PANEL_Z_ORDER_FALLBACK`, a linha no menu e o id no `WidgetStore`.

**O mecanismo:** o `shells/desktop` põe `default-features = false` no
`ph2d-panel-registry-init` e **re-declara cada painel como feature própria**. Um painel no
`default` *daquela* crate e sem linha no shell é **compilado para fora** — sem erro, sem aviso.

⚠️ **E o que fica na tela é o pior resultado possível:** a linha do menu continua **pintada e
clicável**, porque quem a pinta é a `ph2d-editor-core`, que não sabe nada de features do shell.
⇒ um **botão morto produzido por um manifesto**, não por código em falta.

**Por que nenhum gate o via** — os três corriam na crate onde o painel *existe*:

| gate | estava | porquê |
|---|---|---|
| `every_window_menu_row_reaches_a_consumer` | ✅ verde | corre em `panel-registry-init`, features `default` da própria crate |
| `every_registered_panel_is_reachable_by_the_z_order_walk` | ✅ verde | idem |
| `build_typed_registry_matches_enabled_features` | ✅ verde | é **consistente** com um painel desligado |

⭐ **A cura durável** lê os **dois manifestos** e exige `default(registry) ∩ panel-* ⊆ default(shell)`:
`shells/desktop/tests/every_panel_the_registry_ships_reaches_the_binary.rs`. Mutação: com as duas
linhas do shell fora — o estado exacto em que o Enio abriu o app — ele fica vermelho e **imprime a
cura**. ⚠️ Ele nasceu com o **próprio parser partido** (dividia por vírgula antes de descascar
comentários, perdia 6 painéis) e só o **controlo positivo do corpus** (`>= 20`) o apanhou.

⛔ **A lição de diagnóstico:** *um smoke que «não abriu» pode não ser código.* Antes de depurar o
painel, pergunte **ele está no binário?** — `strings <bin> | grep <id>` separa as duas hipóteses em
segundos, e aqui teria dado a resposta certa antes da primeira leitura de fonte.

### 11.4 — ⛔⛔ E uma régua desta casa tem um ponto cego NOVO, com nome

O `the_painted_control_reaches_a_consumer` acusou a **caixa viva** de não ter consumidor. Ela tem:
o despacho **genérico** de ponteiro move todo `InteractiveState::Slider` **sem nomear id nenhum** —
é precisamente isso que faz o gesto da bancada ser o gesto do produto.

⇒ *um consumidor genérico lê-se exactamente como consumidor nenhum.* É a **segunda** ocorrência da
família do `HIER_SEARCH` (que é invisível por outro motivo: o efeito não sai do pintor), e entrou na
catraca **com a prova medida ao lado** — `the_live_box_actually_drags` premea 75 % da largura e
verifica `0,75` + `Dragging`, partindo do `populate` do próprio painel.

### 11.5 — ⏳ O que a bancada ainda NÃO estuda (nomeado, não esquecido)

| item | porquê ainda não |
|---|---|
| **o 4.º preset de texto** (o que o Vello 0.10 permite) | ⭐ é a instrução do Enio, e o sistema já tem a porta aditiva documentada (`TextRendering`: *"Adicionar preset = novo variant + caso em `params`/`id`/`display_name`/`next`"*). ⛔ **Os três presets actuais não se tocam** |
| o **esbatimento** do rótulo em vez de `…` | precisa de `push_luminance_mask_layer` e do custo por linha medido (§7.3, §9) |
| checkbox de linha inteira · toggle→checkbox · scrollbar 2 px | §5.2 e §5.3 — a bancada estuda a **caixa** primeiro, que é onde está o ganho |
| a coluna de animação com os **estados** do Blender (losango cheio/vazio, driver) | precisa da timeline, não de desenho |

---

## §12 — ✅ O PADRÃO ESTÁ ESCOLHIDO (2026-09-02)

Depois de ver os seis desenhos lado a lado na bancada, o Enio decidiu:

> *"O padrão do APP deverá ser Sliders tipo **Underline**, **raio 4**, **linha 22**. Como opções de
> customização vamos disponibilizar os **4 primeiros desenhos**. Também deixe como opções de
> customização **raio** e **linha**. Obviamente tudo em **inglês**."*

### 12.1 — O que isso é, em código

| grandeza | valor | e é um TOKEN |
|---|---|---|
| desenho | `SliderDesign::Underline` | — |
| raio | **4 px** | `Radius::Xs` — ⭐ e é o `default_corner_radius = 4` do Godot, a referência que ele nomeou |
| altura de linha | **22 px** | `Density::Compact` |

⭐ **Os três defaults exprimem-se em tokens, com zero literais.** A tabela acima não é uma
coincidência feliz: é o design system a já conter a resposta que o dono escolheu de olho.

⚠️ **`ph2d_tokens::SliderStyle` é irmão exacto do `TextRendering`** — *aparência escolhida pelo
artista, ortogonal ao `Theme`*, publicada uma vez por quadro e lida pelo pintor. Quem acrescentar um
terceiro eixo de aparência segue esta forma em vez de inventar a terceira.

### 12.2 — ⛔ Os dois desenhos que foram construídos e NÃO shipam

| desenho | por que não |
|---|---|
| `Notch` | recupera a precisão que a `Bar` perde, ao preço de mais um elemento por linha — **não escolhido** |
| `Split` | ⚠️ era o **controlo negativo**: o desenho de duas colunas (o do Blender). Mantê-lo na customização deixaria o artista escolher de volta os `154 px` de cromo que este redesenho existe para apagar |

Registados no `slider_style.rs` com o mecanismo de cada um, e com gate a impedir que voltem sem
decisão (`the_customisation_offers_exactly_the_four_chosen_designs`).

### 12.3 — ⭐ Uma PORTA, não duas: o estudo passou a pintar com o pintor do PRODUTO

O `BoxDesign` do laboratório **morreu**; no lugar dele está
`ph2d_editor_core::widget::paint_property_box`, que o **produto** e a **bancada** chamam. ⛔ *Um
segundo pintor «só para a bancada» faria o estudo divergir do que shipa sem ninguém notar* — que é
literalmente o bug que criou o `slider_with_chip` (*"the slider in panel X looks different from the
one in panel Y"*).

E a caixa entrou na **Widget Gallery**, por baixo do widget que substitui: a galeria é a fonte única
de verdade do que o editor **É**, e ela lê a preferência viva — *uma galeria que mostrasse o default
fixo mentiria a quem customizou*.

### 12.4 — ⚠️ Quatro gates apanharam esta obra, e os quatro estavam certos

| gate | o que exigiu | a cura foi |
|---|---|---|
| `widget_mod_block_in_sync_with_folder` | o bloco de `mod` é gerado | correr o `ph2d-widget-sync` |
| `every_widget_is_shown_or_explicitly_opted_out` | ⭐ um widget novo **aparece na galeria** ou declara porquê não | **mostrá-lo** — não um opt-out |
| `every_widget_file_wires_a11y` | um widget tem semântica para quem não vê | ⭐ `PropertyBox::a11y_node` (`Role::Slider` + `numeric_value`) |
| `no_magic_numeric_in_widget_or_screens` | ele varre as crates-irmãs | o marcador na **mesma linha** |

⏳ **E um buraco NOMEADO, não fingido:** o nosso `NodeBuilder` tem `label` e `numeric_value*` e
**nenhum slot para o texto do valor** ⇒ quem não vê ouve *«Speed, 62 %»* e **perde a unidade**.
⛔ Não o dobrei dentro do `label` (misturar duas grandezas num campo com dono); a cura é um campo no
`ph2d-a11y`, foundational de outra gente.

### 12.5 — ⛔ O que este bloco NÃO fez, e é a próxima obra

**As linhas de propriedade do app continuam a usar o widget antigo.** O `paint_slider_with_chip`
(**70 sítios**) ainda pinta as três colunas; o que existe hoje é a decisão, o pintor partilhado, a
customização e a amostra na galeria.

⚠️ **Trocá-lo é uma obra com um problema de contrato por decidir**, e por isso não entrou aqui a
correr: o `slider_with_chip_chip_rect` é uma função **pura** (sem `TextSystem`) que a rachura de
token usa para desenhar por cima do valor, e a caixa única deriva a região do valor **da largura do
texto medido**. ⇒ ou a região do valor passa a ter largura fixa (`chip_w`, e o contrato mantém-se),
ou aquela função ganha o `TextSystem` (e muda de assinatura). *Escolher isso ao fim de um bloco longo
é como nasceram os dois defeitos que o Enio apanhou por foto nesta mesma sessão.*

---

## §13 — ✅ A TROCA: as linhas de propriedade do app SÃO a caixa única (2026-09-02)

O §12.5 dizia que faltava, e nomeava a pergunta de contrato que a bloqueava. Ela foi respondida e a
troca está feita: `paint_slider_with_chip_layout` — **a porta dos 70 sítios** — pinta a caixa.

### 13.1 — A resposta ao contrato: a coluna do valor é FIXA

| opção | e por que a que ficou |
|---|---|
| largura **medida** pelo texto | ⛔ cada linha põe o número num `x` diferente e a coluna sai **esfarrapada**. *Números de um formulário alinham-se, ou o olho não os compara.* |
| largura **fixa** (`chip_w`) | ⭐ mantém o `slider_with_chip_chip_rect` **puro** (é ele que a rachura de token usa para desenhar sobre o valor) e alinha a coluna |

⭐ **E a coluna do valor virou UMA LEI com dois leitores** — `property_box::value_column`. Antes eu
tinha a aritmética escrita duas vezes (no pintor e no `chip_rect`), *idêntica*; hoje o pintor
**pergunta**. ⛔ *Duas contas que hoje concordam são duas contas que amanhã divergem, e o sítio onde
isso apareceria é o pior: o número pintado num `x` e o alvo de clique noutro.*

### 13.2 — ⛔⛔ O achado que mudou o desenho: as SETINHAS ficam

Eu ia apagá-las (Blender mostra-as só no hover). A casa recusou, com razão e com instrumento:

- `store.is_chip_no_stepper` existe **precisamente** para o defeito que eu ia criar, e o
  doc-comment dele descreve-o: *"the dispatch carves the right column out of every NumberInput's hit
  rect by default, producing a phantom continuous-hold"*.
- `architecture_no_chip_without_steppers` **proíbe** suprimir as setas (canon 2026-05-24).

⇒ **apagar o desenho de um alvo que continua clicável é fabricar um controlo invisível** — a mesma
família que este documento inteiro persegue. As setas vivem **dentro** da caixa, e o cromo ainda
colapsa: o que desapareceu foram os `70 px` de rótulo, os `12` de folga e o trilho separado.

### 13.3 — O que mudou para quem chama

| símbolo | antes | agora |
|---|---|---|
| `paint_slider_with_chip*` | 3 colunas | **a caixa** — 70 sítios, **zero assinaturas mexidas** |
| `label_w` | a coluna externa do rótulo | ⚠️ **ignorado, declarado** — é a grandeza que a caixa põe a zero; mudar 50 assinaturas para o apagar seria 50 diffs a exprimir uma decisão já exprimida |
| `slider_with_chip_is_stacked` | `true` num painel estreito | **`false` sempre** — e é a decisão, não um bug: sem coluna externa nada pode deixar de caber |
| `slider_with_chip_height` | `2 × row_h` a `PANEL_MIN_W` | **`row_h`** — a altura que a estreiteza cobrava |
| `slider_with_chip_chip_rect` | conta local + caso empilhado | a **lei** (`value_column`) |
| ⭐ `slider_with_chip_min_w` | — | **novo**: o piso honesto (*cabe o valor + folgas*) |

⭐ **O arrasto passou a pegar no rótulo também** — o modelo do Blender que o Enio escolheu: *o campo
inteiro é o controlo*. Antes a zona de arrasto era só o trilho, e num painel estreito ela chegava a
`30 px`.

### 13.4 — ⚠️ DUAS asserções ficaram vácuas, e as duas trocaram de instrumento

O `debug_assert!` do `chrome::input_map` e o gate `the_zone_numbers_never_stack_at_the_windows_width`
perguntavam *«a janela é larga que chegue?»* **pelo proxy do empilhamento** — e o proxy morreu.

⛔ Mantê-las seria a pior espécie de gate: **verde por construção, com o nome de uma protecção.** As
duas passaram a medir contra o `slider_with_chip_min_w`. *A pergunta continua boa; o instrumento é
que deixou de a medir.*

⭐ **E o mesmo movimento matou um espelho:** o `ZONE_MIN_TRACK = 60.0` do `input_map/layout.rs`
declarava-se *"o espelho do `SLIDER_CHIP_MIN_SLIDER_W` do widget"*, com **dois gates** a provar que
os dois números concordavam. ⛔ *Dois gates a vigiar duas cópias são a admissão de que há duas
cópias.* Hoje quem precisa do piso **pergunta**.

### 13.5 — ⚠️ E três gates de árvore apanharam a obra

| gate | o que apanhou |
|---|---|
| `widget_primitives_under_loc_cap` | `slider_with_chip.rs` a **594** > 500 ⇒ partido por **assunto**: `mod.rs` (*como uma linha se compõe*) + `number_chip.rs` (*como um número se edita*) |
| `paint_number_chip_paints_steppers` | o caminho do ficheiro seguiu a divisão — ⭐ e só porque ele usa `unwrap_or_else(panic!)`: um `unwrap_or_default()` teria ficado **verde sobre um ficheiro inexistente** |
| `every_widget_file_wires_a11y` | o `number_chip.rs` é um **fragmento sem `NodeId`** ⇒ opt-out com o motivo (a semântica é da LINHA, `PropertyBox::a11y_node`) |

⛔ **E eu criei duas coisas mortas no caminho**, apanhadas pelo clippy: o `SLIDER_CHIP_MIN_SLIDER_W`
(o piso do desenho antigo) — apagado — e o `slider_with_chip_min_w` **sem consumidor**. ⭐ O segundo
não se apagou: ele era exactamente o instrumento que as duas asserções vácuas precisavam.
*Uma função nasceu órfã e a cura foi ligá-la ao sítio que a pedia sem saber.*

## §14 — ⛔ O DEFEITO DO SMOKE: *«offset e drift em relação ao cursor»* (2026-09-03)

Report do Enio, uma frase, sobre a troca do §13 já no app inteiro.

### 14.1 — O mecanismo: um rect com DOIS papéis, e só um deles estava a ser visto

A caixa tem duas leis de geometria, escritas em subsistemas diferentes:

| quem | lei | onde |
|---|---|---|
| o **pintor** | `fill_w = superficie.w · t` | `widget/property_box.rs::paint_surface` |
| o **despacho** | `t = (px − rect.x) / rect.w` | `interaction/dispatch/number_input.rs::update_drag_value` |

…e o `rect` do segundo é **o que o chamador registou no `HitIndex`**.

A 1.ª redacção da linha do produto registava *a caixa **menos** a coluna do valor*, por um raciocínio
local e plausível — ***«ali em cima o clique é para escrever, logo não é para arrastar»***. Isso
responde à pergunta **1** do hit-test (*este ponto é meu?*) e ignora que o rect responde também à
**2** (*por quanto dividir?*).

⇒ **Um rect mais estreito do que o que se pinta não RECORTA: ele ESCALA.**

| linha | pintado | registado | factor |
|---|---|---|---|
| produto, painel a `220 px` | `220` | `220 − 12 − 72 = 136` | **1,62×** |
| bancada, *decorator* ligado | `w − 14` | `w` | `1,07×` |

⭐ **A mesma lei partida dos dois lados opostos.** É por isso que o defeito viveu meses na bancada
sem ninguém o ver — `14 px` de discrepância contra `84` — e saltou à cara no primeiro smoke do
produto. O erro é `0` na borda esquerda e **cresce ao longo do curso**: é literalmente *offset e
drift*.

### 14.2 — A cura: uma PORTA, `surface_rect`

`ph2d_editor_core::widget::surface_rect(rect, decorator)` devolve o rectângulo que o preenchimento
atravessa, e tem **dois leitores**: o pintor e quem regista. Os três sítios que pintam a caixa
passaram a chamá-la (a galeria é estática e não regista nada).

⭐ **Excluir a coluna do valor faz-se por ORDEM, não por largura:** o `HitIndex` resolve em `rev()`,
então registar o chip **depois** da trilha dá-lhe a coluna do número sem tocar no denominador. O
clique-para-escrever e as setinhas continuam exactamente onde estavam.
⚠️ **A ordem é agora load-bearing** — trocá-la faria o número deixar de ser editável em **todo** o
app, sem nenhum outro sinal. Tem gate (`the_value_column_still_belongs_to_the_chip`).

⚠️ **O topo da faixa continua alcançável** e não por acaso: durante o arrasto o ponteiro sai pela
direita e o `clamp` entrega `1.0`. É o que o Blender faz. ⛔ Não «corrigir» isso encolhendo o rect —
é o defeito outra vez.

### 14.3 — O gate corre o GESTO, e tem controlo

`the_fill_lands_under_the_cursor.rs` (3 testes):

1. carrega o ponteiro a sério (`dispatch_pointer`) em **três** pontos do curso sobre a linha do
   produto pintada, e exige `|rect.x + rect.w·t − px| ≤ 1 px`;
2. **o controlo** reproduz o registo estreito e exige que a barra de `1 px` o **reprove** — sem
   isto, o gate nº 1 poderia ser duas expressões da mesma conta, verde por construção
   ([§13.4](#134--as-duas-asserções-que-ficaram-vácuas) já tinha pago essa lição);
3. a coluna do valor resolve para o **chip** e o rótulo para a **trilha**.

⚠️ **E a 1.ª redacção do próprio gate mediu a coisa errada:** `store.slider_visual(id)` devolve
`(estado, hover_live)` — o **relógio de hover**, que satura em `1.0` a arrastar. Lia-se como um
valor de `1.0` perfeitamente plausível. Quem devolve o valor é `store.slider(id)`.
*Um acessor cujo segundo campo é uma animação e não o valor é uma armadilha com nome de conveniência.*

## §15 — ⭐⭐⭐ O CENSO, e a ordem do trabalho que falta (2026-09-03)

*«Siga com estudo, planejamento e inteligência»* (Enio). O §5 estudou os **44** ficheiros de
`widget/` por *categoria*; o que faltava era a **população** — e ela reordena tudo.

### 15.1 — Quem o app de facto usa (contado fora de `ph2d-editor-core`)

| widget | sítios | estado |
|---|---:|---|
| `paint_checkbox` | **81** | ⏳ **o maior lever, e por fazer** |
| `paint_slider_with_chip` | 58 | ✅ §13 |
| `paint_section_header` | 39 | ⏳ recolhível (§5.3) |
| `paint_color_swatch` | 37 | ⏳ raio |
| `paint_segmented_adaptive` | 36 | ✅ já reflui |
| `paint_dropdown_chip` | 36 | ⏳ seta 8 px + esbatimento |
| `paint_toggle` | ~~29~~ → **3 ficheiros** | ⏳ decisão do Enio: sai — ⚠️ ver 15.1-bis |
| `paint_number_input` | 29 | ⏳ os chips soltos |
| `paint_text_input` | 23 | ⏳ moldura só no foco |
| `tabs` · `tag` · `rect2` · `radio_group` · `level_meter` | 2–4 | — |
| `combobox` · `vector3_editor` · `tree_view` · `list_item` · `key_value_list` · `progress_bar` | **0** | ⛔ **só existem na galeria** |

⭐⭐ **O slider era o nº 2.** O nº 1 é a caixa de verificação, com **40 %** mais sítios — e o §5
dava-lhe uma linha de tabela.

⭐⭐⭐ **E a `PillGroup` tem ZERO consumidores** — nem painel, nem cromo, nem a pele de canvas; só o
`pub use` e um token. A decisão do Enio (*«as pílulas podem sair»*) **não custa nada**, e ⛔ não há
`WidgetKind::Pill`, logo **nenhum código viaja em documento** (a cerca do `skin.rs` não se aplica).

### 15.1-bis — ⛔ E o próprio censo mordeu: **um `grep` por NOME conta homónimos**

A linha do `paint_toggle` dizia **29 sítios em 11 ficheiros**. Verificado um a um, os consumidores
do widget são **três**: `ph2d-panel-grid-snap`, `-painter-layers` e `-timeline`. Os outros são
**funções locais com o mesmo nome** — o `paint_toggle` do Audio Mixer é um *botão colorido* cujo
`active_bg` é parâmetro, o `paint_toggle_button` do Equalize Sizes e o `paint_toggle_row` do Motion
Params idem. ⛔ **Nenhum deles desenha o interruptor deslizante**, e trocá-los por caixas de
verificação teria apagado três widgets diferentes que ninguém pediu para apagar.

⚠️ *Um censo por nome mede a palavra, não o consumidor.* A coluna que vale é «quem **importa** o
símbolo da crate que o define» — e essa é uma pergunta diferente, com outra resposta.

⛔ **E o `WidgetKind::Toggle` tem `code() == 2`, que VIAJA em documento** (`skin/kind.rs`): o
interruptor **não se apaga**, funde-se em PINTURA — o código velho passa a apontar para o pintor da
caixa de verificação, que é exactamente o que o §5.5 já mandava.

⚠️ **Seis widgets do catálogo não têm um único consumidor no produto.** Eles continuam vivos porque
o gate `every_widget_is_shown_or_explicitly_opted_out` os mostra na galeria — *o que é certo, e não
é o mesmo que serem usados*. ⛔ Compactar um deles é trabalho que ninguém vê: **a ordem segue a
população**.

### 15.2 — A ordem, e porquê

| # | obra | porquê agora | risco |
|---|---|---|---|
| **1** | **checkbox: a linha inteira é o alvo** | 81 sítios, e ⭐ **os chamadores JÁ registam a linha** (`Rect::new(x, y, w, h)` em 17 de 19 amostrados) ⇒ é **só pintura**, zero mudanças de chamador | baixo |
| 2 | `toggle` → `checkbox` | decisão do Enio; 11 ficheiros; apaga um widget | médio (substituição em chamadores) |
| 3 | pílulas fora | decisão do Enio; **custo zero** (15.1) | nenhum |
| 4 | o **ritmo da linha** | ⛔ hoje a linha de propriedade mede `22` (do `SliderStyle`) e a de checkbox `18` (literal em ~19 chamadores): um formulário alterna alturas e **não lê como grelha** | médio: mexe em chamadores |
| 5 | scrollbar `2 px` em repouso | devolve **8 px de largura a TODOS os painéis** de uma vez (§5.3) | baixo |
| 6 | **a coluna de animação** | ⭐ decisão nº 3 do Enio, e a premissa que a bloqueava **DISSOLVEU** — ver 15.3 | alto: precisa de consumidor |
| 7 | o 4.º preset de fonte (Vello 0.10) | instrução dele; isolado; ⛔ **os três presets actuais não se tocam** | baixo |

### 15.3 — ⭐⭐ A premissa que bloqueava a coluna de animação dissolveu-se, e foi ele que a dissolveu

O §13 escreveu, no código: *«esta função não sabe se a propriedade é animável»*. Isso pressupõe que
**a resposta VARIA por propriedade**. A decisão dele é a outra: ***«em todas as propriedades que
podem ser animadas, e nessa engine vou querer animar tudo»*** ⇒ a resposta é **constante**, e uma
constante não precisa de ser consultada.

⚠️ **O que sobra não é o desenho, é o CONSUMIDOR.** Um ponto que se pinta e não põe uma chave é um
controlo morto — a espécie que este repo caça (`CLAUDE.md` §5.0). O trabalho real é:
*que identidade é que a timeline liga* (ela liga por **nome**, ADR-0141..0144), *o que o clique
faz* (criar / remover chave no playhead) e *os três estados* (não animado · animado · com chave
aqui). ⇒ é uma obra com plano próprio, **não um `decorator: true`**.

---

## §16 — ✅ AS OBRAS 1, 2 e 3 DO PLANO (2026-09-03)

### 16.1 — A caixa de verificação: a linha inteira é o alvo, e a marca vai à direita

O widget **nº 1 por população** (81 sítios). ⭐ **Custou zero mudanças de chamador**, e a razão é
medida: **17 de 19** chamadores amostrados já passavam `Rect::new(x, y, w, h)` ao pintor **e**
registavam esse mesmo rect no `HitIndex` ⇒ *a linha inteira já era o alvo de clique há muito tempo;
o que faltava era o desenho dizê-lo*. Os 2 restantes passam meia-linha (dois checkboxes lado a
lado), onde encostar à direita da meia-linha é igualmente correcto.

Três mudanças, todas de pintura:

| o quê | antes | agora |
|---|---|---|
| a marca | `rect.x` (esquerda) | `property_box::value_column(...)` — **a coluna do número** |
| o rótulo | depois da caixa, `TypeToken::Base` (**13 px**) | à esquerda, `Sm` (**12 px**) — o mesmo da linha de propriedade |
| a superfície | nenhuma | **acende** no hover, emergindo do nada (`hover_axis` com repouso `None`) |

⚠️ **O corpo de letra não é cosmética:** a linha de propriedade escreve a `12` e esta escrevia a
`13`. Num formulário as duas alternam, e **1 px de corpo entre linhas vizinhas lê-se como
desalinho**, não como ênfase.

⚠️ **A truncagem do rótulo é a MESMA função** (`property_box::fit_label`, agora `pub(crate)`) — ⛔
não uma cópia: metade das linhas a cortar e a outra metade a transbordar é pior que nenhuma das
duas.

### 16.2 — ⛔ O ponto cego que impede um gate honesto sobre o rótulo

O `TextSystem::without_system_fonts()` — o arnês de **todo** gate de widget desta casa — não tem
glifos, e `paint_text` **não produz tinta nenhuma**. Medido: `"On"` e um rótulo de 40 caracteres
dão `path_data` byte a byte igual. ⇒ um gate de tinta sobre o corte do rótulo ficaria **verde por
vacuidade, com o nome de uma protecção** — a espécie que o §13.4 já pagou.

O gate que existe **regista a cegueira e auto-destrói-se** quando ela acabar
(`the_label_truncation_is_not_measurable_here_and_that_is_recorded`): ele afirma que as duas tintas
são IGUAIS, então no dia em que o arnês ganhar uma fonte vendorizada ele fica vermelho e obriga a
substituí-lo por um que MEÇA.

### 16.3 — As pílulas saíram, e o censo tornou-o grátis

`widget/pill_group.rs` **apagado** (170 LOC). ⭐ Zero consumidores em todo o repo, e ⛔ nenhum
`WidgetKind::Pill` ⇒ nada viaja em documento.

⛔⛔ **E a cerca da galeria era FALSA:** o opt-out dizia *«compound: covered by the topbar Image
Tools pill cluster on every paint»*, e o topbar **nunca chamou** `paint_pill_group` — ele pinta as
próprias pílulas e só usa o **token** `PILL_PADDING_PX`, que viajava por um `pub use` deste widget.
*Uma isenção que nomeia um consumidor inexistente é uma cerca a proteger o nada* — e ela sobreviveu
porque ninguém verifica o TEXTO de um opt-out, só a presença dele.
⇒ o token passou a vir do dono (`ph2d_tokens`), e o `let _ = PILL_PADDING_PX; // keep import alive`
do `cluster_painter` — um cadáver de import — morreu com ele.

### 16.4 — O interruptor deslizante: fusão de PINTURA, nunca de modelo

`paint_toggle` passou a chamar `checkbox::paint_boolean_mark`, o **único** pintor de um booleano no
app. A tinta é **byte a byte igual** à da caixa, com gate a exigi-lo em 8 combinações de
estado × valor.

⛔ **O `Toggle` NÃO se apaga**, e o motivo tem endereço: `WidgetKind::Toggle` tem `code() == 2` e o
código **viaja em documento** (`skin/kind.rs` proíbe reciclar códigos) ⇒ um painel autorado gravado
ontem partiria. ⚠️ E o `Role::Switch` fica: *fundir a tinta de dois controlos não os torna o mesmo
controlo*, e um leitor de ecrã a anunciar «caixa de verificação» onde o documento diz «interruptor»
mentiria sobre o modelo.

⭐ **O `thumb_circle` e os seus dois gates saíram com ele.** O que eles defendiam era real (numa
moldura quadrada o curso era zero, numa moldura em pé era `−30` com a semântica invertida e o disco
24 px fora do corpo) — e nada disso pode voltar, porque uma caixa quadrada não tem curso.
*Um gate que defende geometria que já não se desenha não é uma protecção — é uma âncora.*

### 16.5 — ⛔⛔ E a varredura da workspace apanhou um defeito MEU que três portões não viam

`every_scrollable_panel_intercepts_the_wheel`: **a bancada publicava um polegar de rolagem e a roda
do rato sobre ela dava ZOOM na câmera.** Isto é a segunda metade do report do Enio (*«o painel não
tem scroll»*) — eu tinha posto a barra, que **arrasta**, e a roda continuava a atravessar o painel.

⚠️ **Nenhum portão desta linha o via:** o gate vive em `shells/desktop/tests/`, onde nem a suíte da
crate do painel nem `cargo test --bins` chegam. Só a varredura da workspace inteira o alcança — é a
mesma família que a `line/quadextract` pagou em 29/08.

*Publicar um polegar de rolagem e não interceptar a roda é meia-cura, e meia-cura passa no smoke
porque o artista arrasta a barra a primeira vez.*

⚠️ A varredura acusou mais dois, e nenhum era real: `the_cost_of_depth_is_linear_not_explosive` (a
flake de carga que o `CLAUDE.md` §5.0 nomeia — verde sozinha) e um erro de doctest sobre uma
**árvore a meio de uma edição minha**.

## §17 — ⛔ A OBRA 5 FOI RECUSADA POR MEDIÇÃO, e o buraco que ela escondia (2026-09-03)

### 17.1 — A recusa: a barra fina paga-se com a rolagem

O §5.3 propunha **`scrollbar` a `2 px` em repouso, `10` no hover** — *"devolve 8 px a cada painel,
em TODOS eles"*. ⛔ **Não shipa**, e a razão é uma cerca que a proposta não leu: o
`SCROLLBAR_W = 10` tem no doc-comment as palavras do próprio dono —

> *"Wide enough to be a comfortable drag target on iPad/tablet (the user's «Fundamental para
> ipad/tablet» requirement)"*

⇒ e **num tablet não existe hover**. A barra ficaria a `2 px` para o dedo, permanentemente.

⚠️ A saída óbvia — *«fino no rato, gordo no toque»* — **também não existe**: censo completo,
`PointerSource::Touch` tem **ZERO** usos fora do `ph2d-host`. O app não consegue distinguir um dedo
de um rato, então não há como escolher.

### 17.2 — ⛔⛔ E ao verificar a cerca apareceu o buraco a sério

Censo completo de quem escreve `panel_scroll` (sem `head`, todos os ficheiros): **a roda** e **o
polegar da barra**. Mais nada — `kinetic`/`fling`/`drag_to_scroll` dão 23 acertos no repo e **todos
são outra palavra** (*reshuffling*, *fling* de animação).

⇒ **Um painel deste app só rola se houver roda de rato, ou se o dedo acertar exactamente nos
`10 px` da barra.** O tablet é o pré-requisito declarado deste redesenho inteiro, e nele os painéis
eram, na prática, **não-roláveis**.

*Oito pixels de largura era a pergunta errada.*

### 17.3 — ✅ O que se construiu: arrastar o CORPO rola o painel

`BodyScrollAnchor` + três braços de despacho (`Down` arma · `Move` rola · `Up` larga).

⭐ **O gatilho é `hit.is_none()`**, e é ele que torna isto seguro: **nenhum painel regista o próprio
fundo** no `HitIndex`, então *«não acertou em nada»* é exactamente *«espaço vazio do painel»*.
⛔ Um widget que reclame a pressão fica com ela — este gesto **nunca** rouba outro, porque só existe
onde não havia gesto nenhum. Há gate a afirmá-lo, e a **mutação que apaga a guarda mata-o**
(verificado: 3 verdes, o controlo vermelho).

⚠️ **A lei é `1:1` e INVERTIDA, e ⛔ não a da barra.** A diferença é o que cada mão agarra: na barra
o dedo percorre uma **trilha que representa o conteúdo inteiro** (delta proporcional,
`content_h / track_h`); aqui ele agarra o **próprio conteúdo**, e arrastar `40 px` tem de mostrar
`40 px`. Reciclar a conta proporcional faria o conteúdo fugir do dedo a `2,5×` num painel comprido.

⚠️ **O tecto lê-se do store a cada `Move`**, não se fotografa no `Down` como o da barra: uma secção
que se fecha a meio do arrasto encolhe o conteúdo, e um tecto velho deixaria rolar para além do fim.

⭐ **O painel elegível é DERIVADO** (`WidgetStore::scrollable_panel_at`), das tabelas que o pintor
já publica — ⛔ nunca de uma lista escrita à mão. O shell tem uma dessas para a roda
(`cursor_over_hero_panel`) e ela **já custou um painel mudo** nesta mesma jornada (§16.5): quem
acrescenta um painel novo não sabe que a lista existe. *Aqui, publicar `content_h`/`visible_h`
ganha o gesto sem escrever uma linha.*

⏳ **O que fica nomeado:** não há **inércia** (o conteúdo pára com o dedo) nem limiar de movimento —
uma pressão parada em espaço vazio já arma o arrasto, e larga sem rolar nada. As duas são afinação
de sensação, e a sensação mede-se com o dedo, não aqui.

## §18 — ⛔⛔ A OBRA 7 FOI CONSTRUÍDA, SMOKADA E **REVERTIDA** (2026-09-03)

Instrução do Enio, 2026-09-01: *«por precaução vamos manter o que temos e colocar o que o vello
consegue fazer como mais uma opção de aparência das fonts.»* ⇒ construiu-se o 4.º preset,
`CrispEmbolden`, sobre a `Scene::font_embolden` do Vello 0.10 (que tinha **zero consumidores**).

### 18.1 — O veredito do dono, em duas frases

> *«logo ao selecionar a nova opção o cursor ficou lento e não se pode abrir nada mais. não vi
> diferença na font»*

⇒ **caro e invisível.** As duas metades juntas fecham a questão: se fosse só invisível, era afinar o
número; se fosse só caro, era pesar o ganho. **Sem ganho e com custo, não há o que pesar.**

### 18.2 — O mecanismo do custo, e porque não é o meu número

`font_embolden` não é uma bandeira de rasterizador: com `amount != 0` o Vello **desvia o glifo para
outro caminho** — desenha a outline para um buffer, corre `kurbo::expand_path` (offset de contorno,
com junção e limite de miter) e **re-codifica o resultado**
(`vello_encoding/src/glyph_cache.rs`). Um contorno deslocado tem **muito mais segmentos** que o
original, e esses segmentos são re-percorridos **a cada quadro**, para cada glifo no ecrã. ⇒ o custo
escala com *glifos × segmentos*, e uma UI desenha milhares de glifos por quadro.

⚠️ **Nenhum cache o remove:** o cache guarda a *codificação* do glifo, não o trabalho de a percorrer
por quadro.

### 18.3 — E a hipótese sobre a invisibilidade, nomeada e NÃO verificada

O `amount` é em **pixels** — a cadeia é `Size::new(font_size)` → outline já escalada →
`expand_path` (lido no fonte, `glyph_cache.rs:56`). A `0,2 px` por lado, uma haste de ~1,2 px a
12 px devia engrossar **~33 %** — muito visível. Não foi.

⇒ **hipótese com endereço:** `Diagonal2::new(0.2, 0.0)` é um deslocamento **anisotrópico com uma
componente a ZERO**, e um offset de contorno com componente nula é mal condicionado — a normal que
aponta em `y` recebe deslocamento `0` e a curva degenera. Isso produziria exactamente o par
observado: **muitos segmentos** (lento) e **forma quase igual** (invisível).

⛔ **Não foi verificada, e não vale a corrida.** O teste desta casa **não carrega fontes** (§16.2),
então isto só se mede no ecrã — e a única corrida disponível é a do dono. E mesmo que um `amount`
**uniforme** (`(a, a)`) fosse bem condicionado e visível, ele deixaria de ser *«engrossar só o X»*,
que era **a razão inteira de o preset existir**: um synthetic bold uniforme é o que o eixo `wght` já
faz melhor, com espaçamento desenhado pelo tipógrafo.

### 18.4 — O que fica

- ⛔ **O preset saiu do produto**, com o wiring todo (id, linha de menu, `pre_populate`, handler,
  o braço do overlay). *Uma entrada de menu que congela o app é pior que uma ausente.*
- ⚠️ **O modo NÃO era persistido** em `~/.ph2d/prefs.txt` — reiniciar limpava-o. Foi a **primeira**
  coisa verificada ao receber o report, e é a diferença entre um smoke mau e um smoke que prende o
  dono no estado partido.
- ⭐ **O `TextRendering::ALL` FICA.** Ele nasceu porque este preset partiu o
  `text_rendering_cycles_three_states` — um gate com a contagem **no nome** — e a lição sobrevive à
  reversão: *uma contagem literal num gate faz cada feature nova editar o teste de alguém.*
- ⇒ **A resposta à pergunta do Enio** (*«o que há de mais moderno no Vello que pode ajudar?»*),
  para este item: **`font_embolden` foi medido e não paga.** Os outros achados da §8 (o esbatimento
  do rótulo por `push_luminance_mask_layer`, o `draw_blurred_rounded_rect`) continuam por medir, e
  ⚠️ este resultado ensina como: **perguntar o custo POR QUADRO antes de desenhar o que quer que
  seja**, porque num editor tudo se repinta.
