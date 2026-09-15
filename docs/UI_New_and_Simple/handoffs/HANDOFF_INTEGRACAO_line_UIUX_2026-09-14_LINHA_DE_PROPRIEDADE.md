# HANDOFF — `line/UIUX` · 2026-09-14 · **A LINHA DE PROPRIEDADE TEM PORTA**

> Report do dono, com duas fotos: *«Label acima do campo numérico! Muito ruim!»* (secção LEG do
> Platform Player) · *«número na frente da label»* (Sprite Sheet) · *«falta para nós um modelo
> pronto e bem estabelecido com todas as regras para todos os widgets … o inspector está assustador
> de horrível»*.

## 1 — O censo: a coisa que o app mais repete tinha ONZE respostas

**Cinco pintores** de «rótulo + campo numérico» só no Inspector — **dois empilhavam** o rótulo
(`sections/rows::num_row`, `sections/visibility::number_row`, a foto 1) e três punham-no à esquerda.

E a **largura da coluna do rótulo**, medida por régua sobre todas as fontes de UI:

| valor | onde |
|---|---|
| `96` | `inspector/ordering` (×2) · `inspector/anchor_mount_row` |
| `78` | `inspector/sprite_sheet` · `inspector/transform` · `panel-physics` · `panel-wet-tuning` |
| `64` | `editor-core/panel/rows` · `panel-flip` · `panel-padding` · `panel-upscale` |
| `84` | `panel-color-equalization` · `panel-sculpt3d` |
| `76` · `72` · `150` · `176` | `panel-bgremoval` · `panel-equalize-sizes` · `panel-grid-snap` · `panel-timeline` |

⚠️⚠️ **E a primeira contagem — a minha, à mão — dizia SEIS.** Ela procurou nos painéis que a foto
do dono me pôs à frente; a régua varreu as fontes todas e devolveu **onze**. *Um censo escrito à mão
mede os sítios de que já se suspeita.*

## 2 — ⛔ Uma largura FIXA está errada por CONSTRUÇÃO, e a prova não é de gosto

A coluna docada é **arrastável** (`WidgetStore::DOCK_W_MIN`..`720`). Um rótulo de `96 px` numa
coluna aberta a `720` deixa o controlo com `600`; na largura mínima ele come a linha inteira.
⇒ ***os onze literais não são onze gostos: são onze leituras da MESMA coluna à largura de omissão.***

## 3 — A porta: `widget::property_row_columns`

Devolve `PropertyRow { label, control, dot }`.

- **A coluna do rótulo é uma FRACÇÃO da linha.** `LABEL_COL_FRAC = 0,348` é **medido**: `96 / 276`,
  onde `96` é a resposta que 3 dos 6 sítios do Inspector escreviam e `276` é a largura real de uma
  linha dele (`304` do token `inspector-w`, menos `2×8` de recuo do painel e `2×6` do cartão).
- **O tecto tem RECURSO:** o controlo nunca fica abaixo de `ICON_BTN_SIZE_PX + Spacing::Lg` — a
  coluna do *stepper* de um `NumberInput` mais um dígito (§0.0: *um limite legítimo diz de que
  recurso ele é*).
- **Ela COMPÕE com a [`form_row_columns`]**, não a duplica: o ponto da coluna de animação continua
  a sair de lá. *Duas portas a responder ao mesmo `x` é a forma que esta casa acabou de pagar onze
  vezes.*
- ⏳ **O vão horizontal `rótulo → controlo` é `Spacing::Md` (8) porque é o que a casa já responde**
  em 3 dos sítios — e **não tem derivação**. Fica NOMEADO como dívida em vez de fingir-se lei.

## 4 — Convertido: o Inspector inteiro

`sections/rows::num_row` · `sections/rows::seg_row` · `sections/visibility::number_row` ·
`sections/ordering::number_row` (×2) · `sections/sprite_sheet` · `sections/transform` ·
`sections/anchor_mount_row`. **Zero** literais de coluna ficaram no Inspector.

⭐ **A altura de uma row caiu de `rótulo + caixa + respiro` para uma LINHA** (~16 px por row), e a
`num_row_h`/`card_h` encolheram com ela — as duas têm de ser a mesma resposta, senão a moldura do
cartão nasce por cima das caixas.

⚠️ **A `seg_row` foi convertida no MESMO commit, e não por zelo:** ela partilha as secções com a
`num_row`, e deixar uma à esquerda e a outra por cima **é** a queixa do dono uma linha mais abaixo.
O grupo segmentado **reflui** dentro da coluna dele (três opções compridas quebram em duas
fileiras); o que mantém a secção legível é a coluna da esquerda estar sempre no mesmo `x`.

⚠️ **O `transform` teve só o NÚMERO trocado.** A lei adaptativa dele (inline × empilhado, decidida
por o que dois chips precisam) tem feedback do dono de 2026-05-24 por trás e fica intacta — o que
muda é ela passar a calcular-se com a coluna que a secção de facto desenha.

## 5 — ⭐⭐ A prova de que o painel encolheu não foi uma medição minha

O `seam_player::every_player_control_carries_a_hover_hint` reprovou com **«isenção STALE»**: a
entrada isentava *«o scrollbar do Inspector (ele nasce porque a §14 estica o painel)»*, e com as
rows mais curtas **o scrollbar deixou de nascer**. ⇒ *a metade de obsolescência de uma catraca
mediu o efeito da conversão antes de eu o medir.*

E o `every_form_row_reserves_the_animation_column` reprovou por ser uma régua **textual** que
procurava o nome da porta antiga: ensinada às **duas** (a nova compõe com a velha e devolve o mesmo
`dot`).

## 6 — O gate: `the_label_column_is_one_answer`

Duas metades — *ninguém escolhe a largura* + *a tolerância ainda descreve alguma coisa*. A catraca
nasce com **12** entradas (os painéis fora do Inspector) e ⛔ **só encolhe**.

⚠️ **A porta é isenta de si própria** — ela é onde a resposta VIVE, não um sítio de pintura.
⚠️ **O da timeline (`176`) responde a OUTRA pergunta** (a coluna de nome de uma FAIXA) e está na
lista por a régua ser textual; quem o converter decide primeiro se a pergunta é a mesma.

## 7 — ⏳ ABERTO

- **Os 12 painéis da catraca** — é a continuação directa desta wave, e agora é mecânica: a porta
  existe e o gate nomeia os sítios.
- **O vão horizontal** (`Spacing::Md`) sem derivação.
- **O rótulo com UNIDADE no texto** (`Float Height (m)`, `Cling Distance (m)`): numa coluna de
  ~96 px eles elidem. A saída não é alargar a coluna — é a unidade sair do rótulo e ir para o campo,
  que é o que as ferramentas de referência fazem. ⛔ Mexe em strings de i18n e é wave própria.
- **A página escrita** do sistema, derivada destas portas — o passo 4 do plano que o dono aprovou.
  ⚠️ Ela vem **depois** das portas, de propósito: um documento escrito primeiro envelhece contra o
  código, que é exactamente o que a `docs/design/` fez (ela não diz **nada** sobre a linha de
  propriedade, a coisa que o app mais repete).

## 8 — ADENDO: a LINHA PARTILHADA do núcleo também pergunta a porta

> *«não vejo melhorias. Mas siga com as melhorias»* — o dono, depois do commit acima.

⚠️ **A primeira coisa foi verificar se eu tinha mexido no pintor errado.** Sonda sobre a pintura
real da §14 (`MockPanelHost`, os rects que saem do `hit_index`): o campo *Float Height* cai em
`x = 166,6` (antes começava na margem do cartão, com o rótulo numa faixa por cima) e as rows distam
**25 px** em vez de 41. ⇒ *o pintor era o certo e a conversão está no produto*; o que ele viu foi um
binário anterior.

⛔ **Mas a leitura dele continua a valer para o APP**, e por uma razão medível: a conversão anterior
tocou o **Inspector**, e o resto da casa passa por outra linha — o `RowCtx` de
[`ph2d-editor-core/src/panel/rows.rs`](../../../crates/ph2d-editor-core/src/panel/rows.rs), *«uma
lei, N hospedeiros»*, que tinha a coluna do rótulo escrita como `LABEL_COL_W = 64.0`.

**Convertido:** a constante virou a função `panel::label_col_w(inner_x, inner_w, y, row_h)`, que
delega na porta; os três sítios internos (`labeled_action_button`, `labeled_number_field`,
`label_cell`) passam por ela, e o rótulo passou a ser **elidido**. ⭐ *É uma mudança num ficheiro que
alinha todos os painéis que compõem por `RowCtx`* — o do esqueleto incluído, convertido no mesmo
commit (dois sítios).

⏳ **E o painel do VETOR fica NOMEADO, com o valor de ontem.** Ele re-exportava a constante do
núcleo e tem **33 sítios** a usá-la como constante livre, muitos sem `y`/`row_h` em alcance ⇒ a
conversão é wave própria. Ele recebe um `const LABEL_COL_W = 64.0` local, com o doc a dizer porquê,
e entra na catraca do `the_label_column_is_one_answer` **no lugar do núcleo, que saiu**.
*Uma dívida nomeada com o valor de ontem é honesta; um `sed` em 33 sítios é como se parte um painel.*

**Portão:** `nextest-impacted` **13 882/13 882** · `clippy --workspace -D warnings` exit 0 · fmt.

---

## 9 — ⭐⭐⭐ ADENDO: os DOZE painéis saíram da catraca num dia, e a dívida estava precificada na moeda errada

Commit `defbac096` · 32 ficheiros · +228/−150.

### 9.1 — O erro que a §8 escreveu, com o nome dele

A §8 fecha com a dívida nomeada do painel de vetor:

> *«tem **33 sítios** a usá-la como constante livre, **muitos sem `y`/`row_h` em alcance** ⇒ a
> conversão é wave própria»*

⛔⛔ **Falso, e a refutação é de uma linha:** a largura da coluna **não depende do vertical**. A
[`property_row_columns`](../../../crates/ph2d-editor-core/src/widget/property_box/mod.rs) usa
`row_y`/`row_h` **só** para montar os `Rect` que devolve; o número que sai de `label.w` é função de
`x` e `w` e de mais nada. E os 33 sítios do vetor vivem **todos** num método do `BodyCtx`, que tem
`inner_x`/`inner_w` como campos.

⇒ ***um bloqueio afirmado sobre um argumento que o resultado não lê é um palpite com cara de
medição*** — e ele custou os outros **onze** painéis, adiados pela mesma frase.

⚠️ A forma de o ter apanhado no dia era **ler a porta antes de escrever o preço**: a função tem
oito linhas e a dependência lê-se nelas. Eu escrevi o preço a partir da *assinatura*.

### 9.2 — A porta ganhou a metade horizontal

```rust
pub fn property_label_col_w(x: f32, w: f32) -> f32   // a lei, sem o vertical que ela não lê
pub fn property_row_columns(x, w, row_y, row_h) -> PropertyRow  // CHAMA a de cima
```

E a [`panel::label_col_w`](../../../crates/ph2d-editor-core/src/panel/rows.rs) perdeu os dois
parâmetros que ignorava (5 chamadores actualizados). *Um parâmetro que o resultado ignora é um
convite a supor que ele importa* — foi essa suposição que escreveu a §8.

### 9.3 — O que foi convertido

| painel | sítios | literal que morreu |
|---|---|---|
| `ph2d-panel-vector` | 33 | `64` |
| `ph2d-panel-flip` | 8 | `64` |
| `ph2d-panel-model3d` | 8 | `72` |
| `ph2d-panel-bgremoval` | 4 | `76` |
| `ph2d-panel-grid-snap` | 2 | `150` |
| `ph2d-panel-physics` | 2 | `78` |
| `ph2d-panel-equalize-sizes` | 1 | `72` |
| `ph2d-panel-color-equalization` | 1 | `84` |
| `ph2d-panel-padding` | 1 | `64` |
| `ph2d-panel-sculpt3d` | 1 | `84` |
| `ph2d-panel-upscale` | 1 | `64` |
| `ph2d-panel-wet-tuning` | 1 | `78` |

⚠️ **Onde a constante era ÚNICA, ela foi APAGADA e o sítio chama a porta directamente** — o painel
deixa de ter uma resposta própria para esconder. Só o vetor ficou com uma função-ponte
(`paint_sections::label_col_w`), porque são 33 chamadores em 11 ficheiros.

⚠️ **O da timeline FICA na catraca**, e a distinção está escrita nela: a
`tracks.rs::LABEL_COL_W = 176` é a coluna de nome de uma **FAIXA**, não a de uma linha de
propriedade. *A régua é textual e não sabe separar as duas — quem a converter decide primeiro se a
pergunta é a mesma.*

### 9.4 — ⛔ Um gate MEU foi construído, medido e deitado fora no mesmo dia

A 1.ª redacção da 3.ª metade do `the_label_column_is_one_answer` comparava as **duas portas**:
`property_row_columns(x,w,y,h).label.w` contra `property_label_col_w(x,w)`, em 108 células.

**A prova de mutação disse-o inútil: ela SOBREVIVEU.** Apagar o tecto (`.min(usable − gap −
control_min)`) muda **as duas** ao mesmo tempo, porque uma chama a outra.
⇒ ***um gate que compara duas construções é cego a uma mutação partilhada*** — aqui ele comparava
a função consigo própria.

**O que ficou** é `a_property_row_never_starves_its_control`, com o oráculo **fora** da lei:

- a linha reparte-se (`rótulo + vão + controlo ≤ w`) sempre que `w` chega para uma linha;
- quando um controlo utilizável **cabe**, ele **é** utilizável: `control ≥ ICON_BTN_SIZE_PX +
  Spacing::Lg`;
- numa linha larga (`w ≥ 200`) o rótulo é a **menor** das duas colunas;
- piso de população nas duas famílias (≥ 100 células cada), senão uma varredura colapsada passa
  trivialmente.

**Duas mutações, as duas mortas:**

| mutação | efeito medido | veredito |
|---|---|---|
| apagar o `.min(…)` da coluna | a `w = 100` o campo fica com `48` px, piso `52` | ✗ vermelho |
| `LABEL_COL_FRAC 0,348 → 0,748` | o rótulo passa o controlo na linha larga | ✗ vermelho |

E a catraca textual **também** foi provada com a lista já a UMA entrada: um `const … = 96.0` novo
num painel convertido ⇒ `the_label_column_is_never_chosen_at_the_painting_site` vermelho.

⛔ **A cerca de degenerado é nomeada:** abaixo de `gap + piso` a porta não reparte nada — entrega os
mínimos (`control = 1 px`, o vão inteiro) e a soma passa a linha por construção. *Uma linha que já
não cabe não é uma linha de propriedade*, e afirmar a partição ali seria medir o degenerado.

### 9.5 — ⚠️ A reescrita por NOME destruiu prosa, pela 5.ª vez neste repo

O `paint_sections.rs` do vetor tinha, num doc-comment, a frase *«Era `const LABEL_COL_W: f32 =
64.0`»* — e a substituição textual transformou-a em `const label_col_w(self.inner_x,
self.inner_w): f32 = 64.0`. Foi apanhada por **ler cada linha reescrita** (`git diff --unified=0 |
grep '^+'`), não por um gate. *Num repo onde o porquê vive em doc-comments, a memória histórica é a
vítima mais comum de um `sed`.*

O mesmo mecanismo mordeu uma segunda vez no `equalize-sizes`, onde a própria **declaração** da
constante entrou na substituição — e ali a cura foi barata porque o compilador fala alto.

### 9.6 — ⏳ ABERTO depois disto

1. **A unidade dentro do rótulo** (`"Float Height (m)"`), que é o que faz um rótulo elidir num
   painel estreito. Toca strings de i18n ⇒ wave própria.
2. **A coluna da timeline**, com a pergunta por decidir (§9.3).
3. **A página escrita** do modelo, derivada das portas — o passo 4 do plano aprovado pelo dono.
4. As dívidas nomeadas da §7 continuam: o vão horizontal `rótulo → controlo` (`Spacing::Md`, sem
   derivação), o INSET sem porta, os 74 sítios horizontais de `Spacing::` e as 356 constantes locais.

**Portão:** `nextest-impacted` (BASE `1d43da737`) **13 883/13 883**, exit 0, `load 49,7` ·
`clippy --workspace --all-targets -D warnings` exit 0 · `fmt --all --check` exit 0 · árvore limpa ·
binário de smoke reconstruído (exit 0, 79 582 488 bytes).

---

## 10 — ⭐⭐⭐ O report seguinte: *«caixas de input numérico sem cor de fundo»*

Commit `43ac9d0da` · 9 ficheiros · +300/−18.

### 10.1 — A causa não era a paleta: era QUEM a escolhia

O `number_input` e a `text_area` enchiam com `resolve(fill_token(state))` = **`ColorToken::Bg1`**
— que é **exactamente** o token de um cartão de secção
([`section_cards::CardDepth::Section`](../../../crates/ph2d-editor-core/src/widget/section_cards/mod.rs)),
e é sobre cartões que o Inspector põe as linhas dele.

**Medido nos oito temas: `0/255`.** E num tema moderno a moldura de repouso é ZERO — é a lei do
`LineEdit` do Godot que esta casa adoptou. ⇒ *não havia caixa nenhuma*, só o número pousado no
cartão.

⚠️⚠️ **E o `text_input` já perguntava ao tema** (`visuals::Chrome::field_fill`). A resposta certa
existia, tinha **um** chamador, e dois pintores irmãos usavam uma terceira.
***Uma lei com uma porta e dois consumidores fora dela não é uma lei; é uma coincidência que ainda
não divergiu.***

⚠️ E há uma **quarta**: o chip numérico ao lado de um slider (`number_chip`) pinta `Bg3` em repouso
e `Bg2` focado. *«Que cor tem um campo?»* tinha quatro respostas neste app.

### 10.2 — A segunda metade: a cor que o tema dava também estava errada, e por uma premissa

O `Chrome::field_fill` moderno era `dark_1.lerp(BLACK, max(c,0) * 0.5)`, com o comentário
*«o `LineEdit` do Godot assenta num degrau abaixo do painel»*. A frase está certa **sobre o
Godot** e errada **sobre nós**: ele põe o campo sobre o PAINEL; aqui as linhas assentam num
**CARTÃO**, que está um degrau ACIMA do painel (a wave de 05/09 desceu o painel, precisamente para
o cartão se ler). Resultado medido: `4/255` do painel e `8` do cartão.

⇒ a lei passa a ser **um degrau abaixo da superfície mais funda em que um campo pode assentar**
— as três são nomeadas (painel · cartão de secção · cartão de subsecção) —, com o degrau a ser o
[`SURFACE_STEP`](../../../crates/ph2d-tokens/src/derive.rs) que a casa já mediu para separar o
CHÃO do painel.

⛔ **Isso NÃO é herdar a resposta de outra pergunta.** É a mesma pergunta — *duas superfícies que
se tocam lêem-se como duas?* — nos mesmos quatro temas. A cerca que o
`a_card_stands_off_its_panel` planta por escrito é contra herdar os `12/255` do par
*cartão-contra-painel*, que é um par diferente.

⚠️ **Qual das três é a mais funda MUDA com o tema:** no `Dark` e no `Gray` é o painel, no `Light`
é a subsecção (ali a escada sobe). *Escrever «abaixo do painel» seria escrever a lei na polaridade
de um tema* — o defeito que aquele mesmo gate já pagou.

| tema | vs painel | vs cartão | vs subcartão | moldura de repouso |
|---|---|---|---|---|
| Dark **antes** | 12 | **0** | 10 | não |
| Dark **hoje** | 10 | 22 | 32 | não |
| Gray **antes** | 19 | **0** | 14 | não |
| Gray **hoje** | 10 | 29 | 43 | não |
| Light **antes** | 13 | **0** | 11 | não |
| Light **hoje** | 35 | 22 | 11 | não |
| Forge/Workshop/Sunstone/Blueprint | — | **0** | — | **sim** |
| Oled | 0 | 0 | 0 | **sim** (*Draw Extra Borders*) |

⛔ **A família clássica e o OLED ficam byte-idênticos** — ali a moldura existe e é ela que separa.

### 10.3 — Três gates, porque são três defeitos

| gate | crate | o que defende | mutação |
|---|---|---|---|
| `a_field_is_never_the_colour_of_what_it_sits_on` | tokens | a COR, contra as três superfícies nomeadas, com piso de população e a cláusula da moldura | repor a derivação de ontem ⇒ ✗ |
| `the_text_in_a_field_still_reads` | tokens | afundar o fundo não pode comer o texto (WCAG AA 4,5:1) | — |
| `every_field_painter_asks_the_theme_for_its_fill` | editor-core | o CAMINHO: o `fill_token` tem UM leitor, a porta | o `number_input` volta a escolher por token ⇒ ✗ |

⚠️ **A segunda existe por um ponto cego real:** o `field_fill` é um valor **derivado, não um
token** — logo o censo de contraste do design system (`contrast_tests`, que varre
`CONTRAST_PAIRS`) **não o vê**. *Um valor derivado fora da tabela de tokens é invisível às réguas
da tabela.*

### 10.4 — ⏳ NOMEADO e não tocado

O **chip numérico** ao lado de um slider (`slider_with_chip::number_chip`) é a quarta resposta:
`Bg3` em repouso — `42/255` do cartão no `Dark`, logo bem visível — e `Bg2` focado, que sobre um
cartão de subsecção é `0` mas traz o anel de foco de 2 px. **Ele tem fundo**, que era o report;
unificá-lo com os campos é decisão de APARÊNCIA (chip levantado contra campo afundado), não a cura
de um defeito. ⛔ Não o converta sem o veredito do dono.

E o **estado DESACTIVADO não tem tinta própria num tema moderno** — a distinção viaja no texto
(`TextDisabled`). É o que o `text_input` já shipava; fica nomeado na porta em vez de se inventar
um segundo tom.

**Portão:** `nextest-impacted` (BASE `1d43da737`) **14 269/14 269**, exit 0, `load 36,9` ·
`clippy --workspace --all-targets -D warnings` exit 0 · `fmt --all --check` exit 0 · binário de
smoke reconstruído (exit 0).

---

## 11 — ⭐⭐⭐ *«Checkbox invisível»* — o §10 tinha curado DOIS de SEIS

Commit `c5b646052` · 7 ficheiros · +232/−25.

### 11.1 — A foto

Secção **Sprite Sheet**: a caixa *Centered*, **marcada**, lê-se (é `Accent`); as de *Flip H* e
*Flip V*, **desmarcadas**, não existem. ⇒ metade das caixas do painel eram invisíveis, e a metade
que se via era a que estava ligada.

Mesma causa do §10, um widget ao lado: enchia `ColorToken::Bg1` — o token do **cartão** em que ela
assenta — e num tema moderno a moldura de repouso é ZERO.

⚠️⚠️ **E o comentário ao lado do código já escrevia a lei CERTA:**

> *«num tema moderno a caixa é plana (marcada = acento cheio, desmarcada = **um degrau abaixo do
> painel**)»*

…sobre um código que pintava o cartão. ***Um doc que descreve a lei que o código não implementa faz
o defeito parecer auditado*** — e foi por isso que o §10 não o apanhou: eu li aquele comentário e
acreditei nele.

### 11.2 — O censo que o §10 devia ter corrido

| pintor | tinta de repouso | curado em |
|---|---|---|
| `number_input` | `Bg1` | §10 |
| `text_area` | `Bg1` | §10 |
| `checkbox` (desmarcada) | `Bg1` | **§11** |
| `combobox` | `Bg1` | **§11** |
| `dropdown` | `Bg1` | **§11** |
| `radio_group` (não seleccionado) | `Bg1` | **§11** |

*Seis respostas à mesma pergunta divergem no dia em que uma superfície se mexe* — e uma mexeu-se
em 2026-09-05, quando o painel desceu um degrau **para o cartão se ler**. A wave que curou uma
leitura do dono abriu esta.

⚠️ **A lição de processo:** o §10 curou os dois pintores que o report nomeava e escreveu um gate
para a COR e outro para o CAMINHO — mas o do caminho media só `fill_token`, que é o vocabulário da
família dos campos. *Um censo escrito à volta do sintoma mede a família do sintoma.*

### 11.3 — A porta

[`crate::paint::body_fill(theme, feel, classico)`](../../../crates/ph2d-editor-core/src/paint.rs) —
a irmã do `stroke_frame` para o PREENCHIMENTO.

- **Clássica:** o token de sempre, **byte-idêntico**.
- **Moderna:** `Chrome::field_fill` em repouso · o corpo **quente** da tabela de estados
  (`Widgets::of(theme).hovered.bg_fill`) em `Hovered`/`Active`.

⚠️ **A segunda metade não é decoração:** afundar só o repouso deixaria as duas pontas do eixo do
hover iguais, e o rato deixaria de dizer alguma coisa numa caixa desmarcada. *Curar a cor de
repouso sem olhar para o eixo apaga a resposta ao rato.*

⛔ **Fora da porta por desenho:** a caixa MARCADA e o rádio SELECCIONADO são acento cheio em
qualquer aparência — não são um corpo afundado, são a resposta.

Medido no tema por omissão: uma caixa desmarcada vai de **`0/255`** para **`22/255`** do cartão.

### 11.4 — Gates

| gate | o que defende | mutação |
|---|---|---|
| `no_widget_paints_a_control_body_with_the_card_token` | nenhum dos 60+ ficheiros de widget resolve `Bg1` para pintar um corpo | o `combobox` volta a resolvê-lo ⇒ ✗ |
| `the_card_token_tolerance_still_describes_something` | a metade de obsolescência da catraca (§5.0) | — |

⚠️ **A varredura PÁRA no primeiro `#[cfg(test)]`** — um gate que afirma a lei cita o token dos dois
lados, e contá-lo acusaria o próprio gate (o defeito que o
`a_census_gate_that_scans_its_own_tree_counts_itself` já registou). ⛔ A cerca presume o que o
`rustfmt` desta casa faz — o `mod tests` no fim do ficheiro.

### 11.5 — ⏳ NOMEADO na catraca, com o motivo

O miolo do **menu radial** continua a encher `Bg1`, e ali **não é o mesmo caso**: ele flutua sobre
o **CANVAS**, e o `Bg1` responde *também* a *«de que cor é o canvas»* (o doc do `derive::Roles::panel`
escreve-o). A porta afunda um degrau abaixo da pilha *painel/cartão*, que não é a superfície
debaixo dele. ⛔ Converter às cegas trocaria um defeito por outro.

### 11.6 — ⏸️ A wave das UNIDADES ficou MEDIDA e por fazer

Antes deste report eu tinha começado a wave seguinte (tirar a unidade de dentro do rótulo). O que
ficou medido, e que é o ponto de partida de quem a pegar:

- **73 strings de i18n** carregam uma unidade entre parênteses (`31` em `inspector_player`, `38` em
  `inspector`, `3` no `lib`, `1` em `painter_layers`).
- ⚠️ **Uma delas é FALSO POSITIVO e mostra a forma da armadilha:** `"Clear {n} unused override(s)"`
  — o `(s)` é o plural, não uma unidade.
- ⭐ **Medido com o sistema de texto real**, à largura de omissão do Inspector (coluna do rótulo
  `91,2 px`, fonte `12`): **20 de 39** rótulos cortam hoje; **sem a unidade, 1**. O sobrevivente é
  *«Non-Spatialized Radius»* (`131,8 px`), que é um nome genuinamente longo — outra pergunta.
- ⭐ O widget existe (`NumericInputWithUnit`, com `px`/`m`/`deg`/`rad`/`%`) e tem **um** consumidor
  de produto (`panel-motion-params`). Faltam `s` e `m/s` ao enum — ⚠️ e o `parse_suffix` casa por
  **sufixo mais longo primeiro**, logo `m/s` tem de vir antes de `s` **e** de `m`, senão `5m/s` lê
  como segundos.
- ⭐⭐ **E há uma descoberta que muda a forma da wave:** o app já tem `DisplayUnit` (m ⇄ px) e
  `DisplayAngle` (° ⇄ rad) nas definições do projecto, com `DisplayAngle::widget_unit()` a mapear
  para o widget. ⇒ um `(m)` escrito no rótulo **não é só comprido: ele pode estar ERRADO** quando o
  artista escolhe pixels. *A unidade tem de vir da definição, não da string.*
- As rows do player são **table-driven** (`PlayerRow`, um pintor só), logo aquele bloco de 31 é
  barato; os outros `25` sítios de `num_row` são individuais.

**Portão:** `nextest-impacted` (BASE `1d43da737`) **14 271/14 271**, exit 0, `load 83,5` ·
`clippy --workspace --all-targets -D warnings` exit 0 · `fmt --all --check` exit 0 · binário de
smoke reconstruído (exit 0).

---

## 12 — ⭐⭐⭐ A UNIDADE sai do rótulo: **32 rótulos cortados passam a 12**

Commit `aa1df2465` · 11 ficheiros · +416/−59.

### 12.1 — A medição, com o texto real

⚠️ **A §11.6 estimou a coluna em `91,2 px` e ela é `84,2`** — eu tinha derivado a largura da linha
da largura do PAINEL, e as rows da §14 vivem dentro de um **card**, que recua `Spacing::Sm` de cada
lado. *Uma régua derivada da superfície errada erra no sentido optimista.* O gate calcula-a agora
dos tokens:

```
inspector-w − 2 × panel-head-pad − 2 × Spacing::Sm   ⇒  linha de 256 px
property_label_col_w(0, 256)                          ⇒  coluna de 84,2 px
```

| | rótulos elididos (de 52) |
|---|---|
| com a unidade no rótulo | **32** |
| com a unidade no CAMPO | **12** |

E os `12` que sobram **não são unidades: são nomes compridos** (§12.5).

### 12.2 — O que mudou

- **`Unit` ganha três**: `Seconds` · `MetersPerSecond` · `MetersPerSecondSquared`, as três
  **contadas** do censo (8 · 11 · 1 ocorrências).
- ⚠️⚠️ **A ordem do `parse_suffix` é load-bearing e o modo de falha é MUDO:** `"5m/s"` termina em
  `"s"`, logo um `Seconds` testado primeiro devolve o **número certo e a unidade errada**, sem erro
  nenhum. A lista passou a ser `Unit::ALL`, pública, e o gate **deriva** a ordem dela — ⛔ não de
  uma cópia no teste, que provaria que a cópia está ordenada.
- **A tabela carrega a unidade:** `PlayerRow` de `(rótulo, id, dica)` para
  `(rótulo, id, dica, Option<Unit>)`. ⭐ Os **três** consumidores que o doc da tabela promete
  (pintor · `populate` · varredura de seam) falharam a compilar — *uma tabela com a arity mudada é
  o gate mais barato que existe*.
- **`num_row_unit`** pinta o sufixo dentro do campo, e a `num_row` de sempre delega nela com
  `None` ⇒ **zero churn** nos outros 25 sítios de `num_row`.
- ⚠️ **A chave de i18n MANTÉM o sufixo** (`…float_height_m`): ela é um ENDEREÇO, não o texto.
  *Um `rename` não distingue um endereço de uma memória* — e renomear 32 chaves mudaria 64 sítios
  para dizer o mesmo.
- ⚠️ **`m/s²` ship-a como `m/s2`**, ASCII: o widget usa a MESMA string para mostrar e para ler, e um
  expoente que o artista não tem no teclado seria um campo que ele não consegue escrever. A dívida
  de separar as duas strings já estava nomeada no `DisplayAngle::suffix`.

### 12.3 — O tecto de LOC, curado por CORTE

O `player.rs` chegou a `602/600` com o doc da tabela. O `PlayerRow` mudou-se para o ficheiro da
**tabela** — *o pai responde «como a secção se desenha», o filho «o que ela oferece»*, e a forma de
uma linha é do segundo. ⛔ Subir o número seria a cura errada (`CLAUDE.md` §2).

### 12.4 — Gates

| gate | mutação |
|---|---|
| `every_label_this_panel_paints_fits_its_column` + a metade de obsolescência | a unidade volta ao rótulo ⇒ ✗ |
| `a_longer_suffix_is_never_shadowed_by_a_shorter_one` | o `s` passa à frente do `m/s` ⇒ ✗ |
| `what_the_artist_types_comes_back_as_what_they_typed` | a mesma ⇒ ✗ |

### 12.5 — ⏳ DECISÃO DO DONO, com o número ao lado

A catraca `AINDA_CORTAM` tem **12** entradas, cada uma com a largura medida contra a coluna de
`84,2`:

`Corner Look-ahead` 109,8 · `Swim Line (weights)` 113,9 · `Weight on Ground` 105,2 ·
`Push on Ground` 92,7 · `Foot Ray Spread` 92,7 · `Wall Ray Spread` 91,5 · `Air Jump Height` 90,9 ·
`Air Acceleration` 90,3 · `Dash Cooldown` 89,4 · `Lift Momentum` 89,3 · `Push on Bodies` 87,9 ·
`Takeoff Gravity` 86,2.

**Duas saídas, e nenhuma é minha:**

1. **Encurtar os nomes** (*«Corner Look-ahead»* → *«Corner Ahead»*) — barato, e é só texto.
2. **Dar ao rótulo uma fatia maior da linha** — a fracção da porta é `0,348` (medida do literal que
   a casa mais escrevia) e **`0,47` faria caber todos**; ⛔ mudaria **todos** os painéis do app, dias
   depois de ele aprovar o desenho actual.

⚠️ E a elisão em si **não é o defeito** — a coluna docada é arrastável e um rótulo tem sempre de
poder cortar. O que o gate afirma é *à largura de OMISSÃO, o artista não devia ver nenhum*.

### 12.6 — ⏳ A dívida que fica NOMEADA na porta

Quando a unidade é um **COMPRIMENTO**, o app já tem uma definição de projecto (`DisplayUnit`,
metros ⇄ pixels; e `DisplayAngle` para ângulos, com `widget_unit()` a mapear para este mesmo
widget) — e a linha ainda mostra a unidade **fixa** que a tabela declara.

⛔ **Ligar as duas exige converter também o VALOR.** Mostrar `px` sobre um número guardado em metros
trocaria um rótulo comprido por um rótulo **mentiroso**, que é estritamente pior. Wave própria, e o
molde já existe: o `event_transform.rs` da §Transform converte pelos dois settings.

### 12.7 — ⚠️ Uma reprovação da varredura foi FLAKE DE CARGA

`a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` — **membro listado** no
`CLAUDE.md` §5.0. As três assinaturas: **zero linhas** do diff naquela crate · `3 de 3` verde
sozinha a `load 48` · verde na re-corrida cheia. A reprovação original foi a `load 43` no pico do
fan-out.

**Portão:** `nextest-impacted` (BASE `1d43da737`) **14 275/14 275**, exit 0, `load 51,5` ·
`clippy --workspace --all-targets -D warnings` exit 0 · `fmt --all --check` exit 0 · binário de
smoke reconstruído (exit 0).

---

## 13 — ⭐⭐ *«Melhor junto ao número dentro da caixa»* — o chip sai

Commit `8b88c46f1` · 6 ficheiros · +323/−230.

### 13.1 — O veredito, e a lei que o explica

A §12 entregou a unidade num **chip** com fundo próprio (`Bg2`), encostado à borda direita do
campo — o desenho que o `NumericInputWithUnit` já tinha. Veredito do dono: *«não ficou legal.
Melhor junto ao número dentro da caixa»*.

⭐ **E ele tem razão pela lei que a caixa tinha ganho DOIS COMMITS antes:** o §10/§11 fizeram do
campo uma **superfície afundada**, e um segundo rectângulo com outro fundo lá dentro lê-se como
*duas* caixas. *O chip era coerente com o campo de ontem — o de fundo transparente com borda — e
deixou de o ser no dia em que o campo ganhou corpo.*

⚠️ **A lição:** uma wave que muda uma SUPERFÍCIE invalida os desenhos que assentavam nela. O chip
não mudou; mudou o que está por baixo dele. (É a mesma forma do §11.2: o painel desceu em 05/09 e
abriu o defeito dos seis pintores.)

### 13.2 — O desenho que fica

[`NumberInput::suffix`](../../../crates/ph2d-editor-core/src/widget/number_input/mod.rs) — a
unidade é **tinta dentro do mesmo recorte do valor**, colada ao número, em `Text2`.

- ⚠️ **Cor de rótulo e não de valor:** ela diz o que o número SIGNIFICA e não é parte dele. Um
  `1.20 m` todo na mesma cor lê-se como um campo de texto.
- ⚠️ **O `x` é MEDIDO, não reservado** (`prefix_width` do valor): um número curto não deixa buraco,
  e um comprido empurra a unidade para fora do recorte — que é o certo, porque *o valor é o que não
  pode desaparecer*.
- ⚠️ **Só em REPOUSO.** A escrever, o campo mostra o que o artista escreveu: o parser aceita o
  sufixo digitado (`"5m/s"`), logo pintá-lo por cima do buffer faria o texto discordar do que vai
  ser lido.

⭐⭐ **E a geometria melhorou com a aparência:** o `input_rect` do `NumericInputWithUnit` é agora o
host INTEIRO, logo *um clique onde a unidade está põe o cursor no número* — antes aqueles 36 px não
eram de ninguém.

### 13.3 — O gate lê a CENA, e exige as duas metades

`the_unit_is_ink_inside_the_box_and_never_a_second_box`: mais **glifos** (a unidade foi pintada)
com o **mesmo** número de caminhos (não nasceu um segundo rectângulo).

⚠️ ***Uma das metades sozinha aprova o defeito*** — só glifos aprovaria o chip de volta, só
caminhos aprovaria uma unidade que não é pintada de todo.

| mutação | veredito |
|---|---|
| a unidade ganha fundo próprio (o chip de volta) | ✗ |
| a unidade deixa de ser pintada | ✗ |
| a unidade é pintada TAMBÉM a escrever | ✗ (`while_typing_…`) |

⭐ **E os dois gates do chip foram SUBSTITUÍDOS, não apagados.** A partição campo/chip e a contenção
do chip num host estreito deixaram de ter **sujeito**: *a cura de aparência dissolveu a classe
inteira daquele defeito*, porque não há um segundo rectângulo para conter. No lugar ficam *«o campo
é o host inteiro»* e *«a unidade nunca cai à esquerda do número»*.

### 13.4 — ⛔ Dois tectos de LOC, os dois curados por CORTE

| ficheiro | | corte |
|---|---|---|
| `panel-inspector/sections/player.rs` | `602/600` | o `PlayerRow` mudou-se para o ficheiro da TABELA |
| `editor-core/widget/number_input.rs` | `594/500` | virou `number_input/{mod,tests}.rs` |

O segundo é o corte que a `section_cards` já tinha feito: *o que o widget FAZ e o que prova que ele
o faz crescem por motivos diferentes.*

⚠️ **E apagar o re-export do `PlayerRow` deixou um doc-comment ÓRFÃO** colado ao item seguinte — o
clippy apanhou-o (`empty line after doc comment`). A prosa mudou-se com o tipo, que é o sítio dela.
*É a segunda vez que esta linha paga esta forma em duas semanas.*

**Portão:** `nextest-impacted` (BASE `1d43da737`) **14 277/14 277**, exit 0, `load 39,7` ·
`clippy --workspace --all-targets -D warnings` exit 0 · `fmt --all --check` exit 0 · binário de
smoke reconstruído (exit 0).

---

## 14 — ⭐⭐⭐ *«Alinhar no meio do painel, as labels à direita»* — e isso fechou os 12

Commit `cdf3c0da2` · 24 ficheiros · +471/−294.

### 14.1 — O veredito

Foto dos cartões LEG/WALK/JUMP: *«as caixas numéricas são muito grandes. Maiores que as labels.
Melhor alinhar no meio do painel e as labels alinhadas todas à direita (no centro do painel)»*.

### 14.2 — A partição ao meio

`LABEL_COL_FRAC` **`0,348 → 0,5`**, com o rótulo a acabar **um vão antes** do meio para o controlo
começar exactamente nele. Numa linha de card do Inspector (`256 px`): rótulo `120`, controlo `114`.

⚠️ **A fracção é da LINHA, não do utilizável.** A coluna de animação sai do lado do CONTROLO, e
medir a metade sobre o utilizável poria a fronteira `7 px` à esquerda do meio — *«o meio do painel»
é o meio do que o artista vê*, não o meio do que sobra depois de uma reserva que ele não conhece.

⚠️⚠️ **E a fracção deixou de ser MEDIDA — a troca é honesta e vale registá-la.** Ela nasceu a
reproduzir o literal que a casa mais escrevia (`96/276 = 0,348`), que era arqueologia correcta do
produto de então. ***Uma medição diz o que o produto FAZ; ela nunca disse o que ele DEVIA fazer*** —
e a segunda pergunta é do dono (`CLAUDE.md` §0.8).

⭐⭐⭐ **E a decisão de aparência dele fechou, de graça, o item que eu lhe tinha devolvido como
escolha.** A catraca `AINDA_CORTAM` (§12.5) tinha **12** nomes elididos numa coluna de `84,2`; a
`120` o mais comprido do app (*«Swim Line (weights)»*, `113,9`) **cabe**, e a lista está **VAZIA**.

### 14.3 — O alinhamento à direita

[`widget::paint_property_label`](../../../crates/ph2d-editor-core/src/widget/property_box/row.rs) —
elidido, **medido no peso em que pinta**, encostado à direita da coluna. Com o controlo a começar no
meio, um rótulo à esquerda deixa um rio de espaço variável entre o nome e o campo dele: *quanto mais
curto o nome, mais longe do valor que ele nomeia*.

⚠️ **A assinatura é a do `paint_text_elided`, argumento a argumento** — converter um sítio é trocar
o nome da função. ***Uma conversão que muda a forma da chamada em 20 sítios é uma que alguém faz
pela metade.***

| onde | sítios |
|---|---|
| Inspector (rows · ordering · visibility · anchor · sprite sheet) | 7 |
| o `RowCtx` do núcleo | 1 |
| `panel-vector` | 8 |
| `panel-flip` | 4 |
| `panel-model3d` | 2 |
| `panel-grid-snap` | 1 |

⚠️ **A DECISÃO saiu para uma porta pura** (`property_label_origin`): dentro do pintor ela só é
observável por quem sabe ler glifos de uma cena, e *uma decisão que só o pintor conhece é uma
decisão que nenhuma mutação mata*.

⛔ **O que NÃO se converteu, com o motivo:** o valor do `paint_fact` do model3d (não é um rótulo, é o
conteúdo da coluna do controlo) e o rótulo do *toggle* do grid-snap (a linha dele é
`nome | interruptor`, outra forma).

### 14.4 — Gates

| gate | o que defende | mutação |
|---|---|---|
| `a_property_label_is_flush_against_its_control` | o fim do texto é o fim da coluna · e a metade que **degrada para a esquerda** quando nem a reticência cabe | o rótulo volta a alinhar à esquerda ⇒ ✗ |
| `a_property_row_never_starves_its_control` | **o controlo começa no meio da linha** | a partição volta a `0,348` ⇒ ✗ |

⚠️⚠️ **A metade *«o rótulo é a MENOR das duas colunas»* FOI SUBSTITUÍDA, não relaxada.** Ela era a
lei da fracção `0,348` e **reprovaria sobre o desenho certo** (a `120 > 114`, porque a coluna de
animação sai do lado do controlo). ***Um gate escrito sobre um NÚMERO reprova quando o dono muda o
número; um escrito sobre a LEI sobrevive.***

⚠️ E a fixtura da segunda metade precisou de uma **coluna mínima**: o `text_elide::fit` devolve
sempre texto que cabe, salvo quando nem a reticência cabe — *uma fixtura sem o fenómeno mede
silêncio*.

### 14.5 — ⛔ Um tecto de LOC, curado por CORTE

`property_box/mod.rs` `508/500` ⇒ a geometria da LINHA saiu para `property_box/row.rs`: *a CAIXA é
um widget — desenha-se, tem estados e um nó de acessibilidade; a LINHA é a repartição de uma faixa,
e serve painéis que nunca pintam uma caixa.*

⚠️ **O corte partiu DOIS gates, os dois ALTO** (a espécie barata — `CLAUDE.md` §5.0): a isenção do
censo da coluna nomeava o `mod.rs` (e a `LABEL_COL_FRAC` mudou-se), e o `hr12_widgets_a11y` viu um
ficheiro de widget novo. O segundo entra no `A11Y_OPT_OUT` com o motivo: *o nó é do CONTROLO, não
da faixa — um nó aqui poria um alvo focável por baixo de cada linha do app.*

### 14.6 — ⏳ NOMEADO

O rótulo interno do `paint_slider_with_chip_layout_adaptive` continua **à esquerda**: aquela linha é
`rótulo | trilho | chip`, uma forma diferente da que o dono viu na foto, e mudá-la toca **toda**
linha de slider do app. ⛔ Decisão dele.

**Portão:** `nextest-impacted` (BASE `1d43da737`) **14 278/14 278**, exit 0 · `clippy --workspace
--all-targets -D warnings` exit 0 · `fmt --all --check` exit 0 · binário de smoke reconstruído.

---

## 15 — ⛔⛔⛔ O gate media uma largura que o dono NÃO USA

Commit `7f39c34ba` · 8 ficheiros · +258/−10.

### 15.1 — O achado

Report: *«3 pontos (…) sendo usados antes de ficar estreito»* — **com o
`every_label_this_panel_paints_fits_its_column` verde**. Fui ao ficheiro de arrumação dele:

```
~/.ph2d/layout.txt → [drawing_2d] dock_w_right = 220.9
```

…contra os `304` do token `inspector-w` que o meu gate media.

| painel | coluna do rótulo | rótulos elididos (de 52) |
|---|---|---|
| `304` (omissão — **o que o gate media**) | `120,0` | **0** |
| `220,9` (**o que ele tem**) | `78,4` | **16** |

⇒ ***um gate calibrado na largura de OMISSÃO é cego à largura que o artista de facto tem.*** E a
cura não é medir melhor um ponto: é medir a **ESCADA** (`ELIDEM_POR_LARGURA`, quatro larguras que
cercam a dele, com o número medido em cada e a metade de obsolescência ao lado).

⚠️ **A largura `220,9` não é inventada** — é lida do ficheiro do dono, e está escrita no gate com
essa proveniência. *Um número de fixtura sem proveniência é um palpite com cara de medição.*

### 15.2 — O rótulo pede emprestado ao campo

Na foto dele os campos mostravam `2`, `65`, `0.100 s` com espaço de sobra enquanto o nome ao lado
truncava. ⇒ [`property_label_col_w_for`](../../../crates/ph2d-editor-core/src/widget/property_box/row.rs):
a coluna é `clamp(desired, metade, o que o controlo pode ceder)`.

- **piso = a METADE** — a ordem dele sobre o alinhamento continua a valer, e a `304` nada muda;
- **tecto = o CONTROLO**, que nunca desce do piso nomeado (o *stepper* mais um dígito);
- `None` = a metade, para quem não sabe o que vai pintar.

⚠️ **O `desired` é da SECÇÃO, não da linha** — uma coluna por linha seria uma coluna diferente por
linha, e ele pediu *«as labels alinhadas todas à direita»*. A §14 mede o rótulo mais largo dos doze
cards, uma vez por quadro.

| painel | antes | depois |
|---|---|---|
| `200` | 32 | **13** |
| `220,9` | 16 | **3** |
| `245` | 7 | **0** |
| `304` | 0 | 0 (coluna `120` = a metade) |

### 15.3 — As reticências não ficam penduradas num espaço

*«quando fica estreito as palavras com 3 pontos não se alinham perfeitamente à direita»*. Medido:
*«Air Jump Height»* a `62 px` saía **`"Air Jump …"`** — o corte caiu logo a seguir a um espaço e o
espaço ficou. Com o rótulo à direita, isso abre um **buraco entre o texto e os pontos** que as
vizinhas não têm.

⚠️ **É o mesmo defeito de sempre — o AVANÇO e a TINTA não são a mesma grandeza** —, só que aqui a
diferença é um caractere inteiro em vez de um *side bearing*.

⛔ **O que eu achei primeiro e a medição REFUTOU:** supus uma **dupla elisão** (o pintor a cortar
outra vez o texto já cortado, por comparação de floats na fronteira). Medido em 16 células: o
re-corte **nunca** muda a string, e o fim do avanço cai **exactamente** no bordo da coluna nas três
larguras. *A hipótese era plausível, tinha mecanismo, e estava errada.*

### 15.4 — ⛔ E um gate MEU sobreviveu a uma mutação, outra vez

A 1.ª redacção da escada **calculava a coluna ela própria** (chamando a porta com o rótulo mais
largo). Apagar o pedido no PINTOR deixava-a **verde**: ela provava a porta, não a fiação.
***Um gate que refaz a conta do produto mede a conta, não o produto.***

⇒ `the_painter_borrows_the_slack_the_control_does_not_need` lê o **rect que o painel REGISTOU** para
o campo *Float Height*, pintado com um `HeroLayout` na largura do dono e outro na de omissão.

| gate | mutação | veredito |
|---|---|---|
| `the_elision_ladder_only_shrinks` | a coluna volta à metade seca | ✗ |
| `the_ellipsis_never_hangs_off_a_space` | o espaço volta | ✗ |
| `the_painter_borrows_the_slack_the_control_does_not_need` | a secção deixa de medir o rótulo mais largo | ✗ |

### 15.5 — ⏳ NOMEADO e não curado

- Os **3** que sobram a `220,9` (`Corner Look-ahead` `109,8` · `Weight on Ground` `105,2` ·
  `Swim Line (weights)` `113,9`) são maiores do que a linha aguenta com um campo utilizável. ⛔ Ali
  só encurtar o nome resolve — decisão do dono.
- A **tinta** de um `…` acaba um *side bearing* antes da de uma letra: as reticências alinham no
  **avanço** (medido: exacto ao centésimo), e a olho ficam ~2 px recuadas face a um rótulo que não
  corta. Medi-lo pede métricas de glifo (skrifa), que esta wave não abriu.

**Portão:** `nextest-impacted` (BASE `1d43da737`) **14 281/14 281**, exit 0 · `clippy --workspace
--all-targets -D warnings` exit 0 · `fmt --all --check` exit 0 · binário de smoke reconstruído.
