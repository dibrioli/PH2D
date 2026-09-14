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
