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
