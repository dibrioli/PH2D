# 11 — A fronteira dos MOTORES: 74 % dela é uma DERIVAÇÃO, não mil chaves à mão

> **Medido em 2026-09-17**, na árvore da `line/UIUX` já rebaseada no `main` da rodada de seis
> (`git merge --ff-only main` → `3090cac3f`, `0` à frente / `0` atrás).
> Instrumento: `git grep` + varredura por contagem de chavetas (scratchpad, com `assert` de
> população) e `python3 scripts/censo-texto-pintado.py`.

## §1 — Porque esta página existe

O fecho de 2026-09-16 deixou o HR-15 fechado **fora dos motores** e escreveu a fronteira seguinte
em prosa: *«os MOTORES escrevem o rótulo numa tabela de dados, e ali a chave tem de ser derivada do
id»*. **Prosa não é um plano.** Esta página põe o número em cada célula dela, e a primeira medição
já **mudou o tamanho da wave**.

⚠️ **E ela corrige uma premissa minha por escrito:** eu supunha que o rótulo de um parâmetro de nó
vivia no `ParamSpec` do manifesto. **Não vive** — o `ParamSpec` é `{ name, default }`, e o rótulo
está no `ParamUiHint`, que é *side-metadata no registry* (a lei que o `CLAUDE.md` §5 já declara
para este módulo). Quem apanhou o erro foi o `assert` de população da própria sonda, que leu `0`
onde esperava `~819`. *Uma varredura sem piso de população teria impresso «0 % deriváveis» e eu
teria acreditado.*

## §2 — A população, por motor

| Motor | população MEDIDA | forma em que o rótulo nasce |
|---|---:|---|
| manifestos de nó (135 crates `ph2d-node-*`) | **801** `ParamUiHint.label` | `ParamUiHint { param: "channel", label: "Channel", … }` |
| … mais as opções de enum dos mesmos | **398** | `ParamWidget::Enum { labels: &["X", "Y", …] }` (151 widgets) |
| `ph2d-component-desc` | **86** `display_name` + **103** `FieldDesc.name` | literal **POSICIONAL** num `const` (`D::authored("ph2d::ecs::AudioSource2D", "Audio Source 2D", …)`) |
| `ph2d-painter-brush` | **37** | braços de `pub const fn name(self) -> &'static str` |
| `ph2d-painter-effects` | **24** | braços de `pub const fn display_name` |
| `ph2d-tool-vector` | **7** | braços de `pub fn label` |
| **o censo de PORTA** (o que só o ponto-fixo vê) | **63** em **10** crates | literal que nenhum censo léxico alcança |
| **TOTAL** | **≈ 1 519** | |

⚠️ **O `63` subiu de `62` em `9` crates** desde o fecho: a `line/components` trouxe um literal novo
no `ph2d-panel-tags`. *O censo de porta não é um gate — ninguém o corre num portão —, e por isso
ele deriva sozinho.* Fechar a fronteira dos motores sem o transformar em gate deixa a mesma porta
aberta.

## §3 — O achado que muda o tamanho: a lei da derivação JÁ SHIPA

⭐⭐⭐ Os **nomes de PORTA** de um nó nunca foram texto autorado: o cartão pinta-os por
`PortLabel::of(p.name)` ([`paint_port_label.rs`](../../../crates/ph2d-panel-motion-graph/src/paint_port_label.rs)),
que **deriva** `target_x` → `Target X`, `in0` → `In 0`. Uma porta, um consumidor, **521**
`PortSpec` servidos sem um único literal de interface.

⇒ a pergunta certa deixa de ser *«quantas chaves tenho de escrever?»* e passa a ser
***«quantos rótulos JÁ SÃO a derivação do id?»***. Medido com a **mesma régua** do `PortLabel::of`:

| | contagem | fracção |
|---|---:|---:|
| `ParamUiHint` com `param` **e** `label` literais | 578 | — |
| **`label` == `derive(param)`** (a tabela não acrescenta nada) | **428** | **74,0 %** |
| divergentes (texto autorado a sério) | 150 | 26,0 % |
| blocos cujo `param` é uma `const` (`param::COUNT`) ⇒ a sonda não os lê | 223 | — |

**Amostra do que diverge de verdade** — e é boa, não é ruído:

```
soft         → "Softness"     (derivado: "Soft")
key          → "Order By"     (derivado: "Key")
mode         → "Path Mode"    (derivado: "Mode")
target_mode  → "Target"       (derivado: "Target Mode")
type         → "Noise Type"   (derivado: "Type")
gust_freq    → "Gust Frequency"
```

⇒ **três quartos da maior população da fronteira desaparecem** se a lei for *«o rótulo é derivado do
id; a tabela guarda só o que DIFERE da derivação»* — que é exactamente a forma que o `PortLabel::of`
já prova em produção, um nível acima.

## §4 — O que isto prescreve para a wave (e o que ela NÃO pode fazer)

1. **A chave sai do id, nunca de um segundo literal.** `node.<nó>.param.<param>` para o hint,
   `component.<canonical_name>.name` / `.field.<n>` para o catálogo. Um rótulo novo que ninguém
   traduziu cai na **derivação**, e a derivação é legível em inglês — *o modo de falha é um rótulo
   sem tradução, nunca um `???` no ecrã*.
2. ⚠️ **A chave derivada do `canonical_name` prende a tradução ao nome do TIPO.** Renomear
   `ph2d::ecs::AudioSource2D` muda a chave **em silêncio** ⇒ o gate obrigatório é o de **dois
   sentidos** que a crate-régua já sabe fazer (`gate::chaves`): toda chave derivada existe na tabela,
   e toda chave da tabela é derivada por alguém.
3. ⛔ **`tr()` não é `const`** e estas tabelas são `static`. O que viaja na tabela é `TextKey`
   (const-construtível) e quem chama `.tr()` é o **pintor** — o padrão já pago 10 vezes no fecho de
   16/09. Um `ComponentDesc::display_name: &'static str` que passasse a `String` reescrevia 86
   construtores `const`.
4. ⛔ **Os 63 literais do censo de porta NÃO são cobertos por nada disto** — eles vivem em sítios
   que a régua léxica já isenta, e o único instrumento que os vê é um script que **nenhum portão
   corre**. Ou ele vira gate na mesma wave, ou a fronteira fecha com um buraco que se enche sozinho.

## §5 — Como reproduzir

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX
grep -rhoE 'ParamUiHint *\{' crates/ph2d-node-*/src/ | wc -l     # 801
grep -rhoE 'D::[a-z_]+\(' crates/ph2d-component-desc/src/catalog/*.rs | wc -l   # 86
python3 scripts/censo-texto-pintado.py                            # 63 em 10 crates
```

A sonda da derivação vive no scratchpad de propósito (ela responde UMA pergunta e o `assert` dela é
o que a torna honesta); o que fica versionado é o **número**, nesta página.

---

# §6 — ⛔ RECUSA MEDIDA: a coluna do rótulo NÃO era a wave (2026-09-17)

Antes de escolher a fronteira dos motores eu preferi uma candidata **visível**: generalizar a
`every_label_this_panel_paints_fits_its_column` (que existe em **3** de **29** painéis) aos outros
26, porque havia um report do dono por trás — *«3 pontos (…) sendo usados antes de ficar estreito»*,
com foto, em 2026-09-14.

A crate-régua foi construída (`ph2d-label-fit`, quarta irmã da família `ph2d-label-*`), correu sobre
**as tabelas de texto dos 24 painéis** — possível *só porque esta linha as construiu* — e o
resultado **refutou a hipótese**:

| largura do painel | strings acima da coluna do rótulo |
|---:|---:|
| `220` (mínimo do dock) | **510** |
| `245` | 219 |
| `304` (omissão) | 59 |
| `333,8` · `369,7` (**as que o dono tem hoje**, lidas do `layout.txt`) | **37** · **24** |

⛔⛔ **E a população estava ERRADA, que é o defeito que este repo já pagou quatro vezes.** Os mais
largos não são rótulos de coluna nenhuns — são **frases de estado**:

```
physics    300,4 px   "Paused: drag carries the whole rig, anchors included"
sculpt3d   298,8 px   "Only used by Twist, and only with Segments above 1"
```

…e elas **DOBRAM**, não cortam: o `paint_hint` da física passa por `paint_text_block` com a largura
inteira da linha, e o doc dele já o diz (*«só o excedente da quebra é acrescentado»*). *Uma régua
que mede uma frase que dobra contra uma coluna que ela não usa fabrica dívida* — exactamente o que a
1.ª redacção do censo do ritmo fez ao ler `48 de 81` ficheiros.

⇒ **o que sobra à largura REAL do dono é quase nada, e nada do que sobra é um rótulo cortado.** A
wave era **insurance**, não um defeito visível — e a ordem em vigor do dono é *«se temos o manual,
precisamos converter o APP todo a ele»*, que é a §2 desta página.

⚠️ **A crate foi APAGADA no mesmo dia.** Ela compilava e fazia a coisa certa, e **não tinha
consumidor** — a lei desta casa é que uma ferramenta que nenhum passo escrito chama pelo nome morre.
*O que se guarda de uma experiência revertida é a MEDIÇÃO, e ela está nesta tabela.*

⭐ **O que ela deixa para quem voltar aqui:** a régua honesta desta pergunta **não é a tabela de
i18n de um painel** — é a tabela de LINHAS que o pintor dele lê (`IROWS` na física, `PLAYER_CARDS` no
Inspector). Medido: **11** dos 29 painéis têm uma, e **3** já têm o gate. *Quem quiser a insurance
começa nos 8 que têm tabela e nenhuma prova; os outros 18 pagam primeiro a tabela.*

---

## §7 — A 4.ª fatia: os rótulos de PARÂMETRO dos nós (medida, e o que ela custou)

A maior das quatro. **766 sítios de `ParamUiHint.label`** em **124 crates** de nó, mais **63** que
nenhuma varredura de texto podia alcançar — `828` chaves no total, em
[`node_params.rs`](../../../crates/ph2d-i18n/src/node_params.rs).

### §7.1 — A população, e as três formas que uma varredura não vê

Medido antes de escrever uma linha (`ParamUiHint` tem `param`, `label` e `widget`):

| Campo `param` | Sítios | Resolve-se |
|---|---:|---|
| literal (`param: "count"`) | **578** | directo |
| caminho de `const` (`param: param::KIND`) | **203** | lendo `const NOME: &str = "…"` na crate |
| campo de tuplo · índice de array · `$p` de macro | **9** | à mão |

E o `label` tem quatro formas: literal · `const` partilhada (`MODE_LABEL`, duplicada em **duas**
crates de propósito) · argumento de construtor auxiliar (`hint(param, label)`) · e **o próprio
`param`** (`label: param`, nos coeficientes do `motion.expression`, onde o rótulo *era* o id).

### §7.2 — ⛔⛔ O defeito do gerador, e porque ele quase passou em silêncio

A 1.ª redacção delimitava o bloco de cada hint por uma **janela de 900 caracteres**. Num hint cujo
`label` **não** é literal, ela lia o literal do hint **SEGUINTE** e reescrevia-o com a chave deste:

```
ParamUiHint { param: MODE,        label: MODE_LABEL,        … }   ← o label nao e' literal
ParamUiHint { param: AIR_RESIST,  label: "Air Resistance",  … }   ← ESTE foi reescrito com a chave `…param.mode`
```

⭐ **Ali o splice deu sintaxe inválida e o compilador viu.** Com os offsets alinhados teria sido
uma **chave errada em silêncio** — a linha do painel a mostrar a palavra do param vizinho. ⇒ a v4
delimita o bloco por **contagem de chavetas**, saltando strings com escapes.

### §7.3 — ⚠️⚠️ `(tipo, param)` NÃO é único, e o ficheiro que o causa já escrevia a razão

O gerador acusou **uma** colisão em 766: o `motion.spline_wrap` declara **duas rows** sobre o param
`path` — *«Shape»* (escolhe a forma) e *«Use Selected Path»* (um botão) —, e o comentário ao lado
delas chama-lhes *«dois GESTOS para o mesmo param, como arrastar um slider e digitar o número»*.

⇒ a chave de uma ROW leva o **widget** quando, e só quando, o param declara mais do que uma:
`…param.path.source` e `…param.path.pick_selection`. A regra depende do **CONJUNTO** de rows e não
da ordem delas, logo uma terceira não renomeia as duas que já lá estão; ⛔ e não existe
`…param.path` sem sufixo, porque um dos dois gestos a herdar a chave curta seria a ambiguidade a
ficar escrita.

### §7.4 — ⭐ O gate é o ORÁCULO da cauda

Uma varredura vê a **FORMA do fonte**; o
[`every_param_label_is_a_key_derived_from_its_type_and_param`](../../../crates/ph2d-node-registry-init/tests/it/every_param_label_is_a_key_derived_from_its_type_and_param.rs)
lê os hints **REGISTADOS**, logo vê o que um construtor auxiliar de facto produziu. Ele nomeou as
**63** da cauda, cada uma com `(tipo, param, texto, chave esperada)` — e a cura correu a partir da
saída dele, nunca de eu re-derivar a regra (*um erro meu na regra apareceria dos dois lados e
cancelava-se*). **59** curaram-se por substituição do literal no sítio onde ele de facto vive; **4**
à mão (os coeficientes, onde o construtor passou a receber a chave).

### §7.5 — ⛔⛔ O que a migração PARTIU, e o que era mudo

Quatro gates de **vocabulário** afirmavam que N nós usam a **mesma palavra** para a mesma pergunta.
Com chaves derivadas do tipo, duas chaves são diferentes **por construção** — e as falhas foram de
três espécies:

| Forma | O que acontecia | Mudo? |
|---|---|---|
| `assert_eq!(a.label, b.label)` | reprova em voz alta | não |
| `assert_eq!(h.label, "Distance")` | reprova em voz alta | não |
| `hs.iter().find(\|h\| h.label == nosso)` | **não acha ninguém**, a lista de acusados fica vazia | ⛔ **sim** |
| `tabela.entry(h.label)` (censo agrupado por palavra) | cada grupo passa a ter um elemento | ⛔ **sim** |

⭐ E os quatro ficaram **mais fortes**: deixaram de afirmar que dois literais estão escritos igual e
passaram a afirmar que o artista **lê** a mesma palavra nos dois cartões, que é a propriedade que
eles sempre quiseram.

### §7.6 — ⚠️⚠️ Traduzir DUAS vezes é um vazamento, não um no-op

`ph2d_i18n::tr` de algo que não é chave faz `leak_key` (`Box::leak`) e devolve a entrada. ⇒ num
painel repintado por quadro, **um segundo `tr` no mesmo caminho é um vazamento por quadro e por
linha**. A lei que fica: **uma tradução por caminho, na FRONTEIRA**.

- **painel lateral** → `motion_bridge_params*.rs` (13 sítios), onde a row é montada;
- **cartão** → `stamp_card_params` (**um** sítio), e os três `tr` que a 1.ª redacção pôs no pintor
  do `ph2d-panel-motion-graph` foram **retirados**. ⭐ A lei já estava escrita no doc do próprio
  snapshot: *«toda row é de primitivos RESOLVIDOS»*.

### §7.7 — ⭐⭐ E o gate que nenhum dos outros dois podia fazer

Os dois gates do registo afirmam que a chave é bem derivada e que ela tem palavra na tabela.
**Nenhum pergunta se a palavra chega ao artista** — um consumidor sem `tr` pinta
`node.motion.wiggle.param.amount` na linha e os dois ficam verdes. É o ponto cego que o `CLAUDE.md`
§5.0 nomeia sobre si mesmo. ⇒
[`motion_bridge_param_label_reaches_the_panel_tests.rs`](../../../crates/ph2d-app-motion/src/motion_bridge_param_label_reaches_the_panel_tests.rs),
que constrói o painel **e** o cartão para cada um dos 137 tipos pela porta do produto e exige que
nenhum rótulo comece por `node.`. ⭐ **Ele reprovou à primeira corrida, com `682` params do cartão a
pintar o identificador** — o segundo consumidor que a wave não tinha coberto.

Prova de mutação: **5 de 5 sangram**, com controlo sobre o próprio filtro.

### §7.8 — O que a 4.ª fatia deixou para a 5.ª (fechada — ver §8)

| Alvo | Sítios | Nota |
|---|---:|---|
| opções de `ParamWidget::Enum` **inline** | `398` em 114 sítios | chave `…param.<p>.<i>` |
| opções por **`const`** | 37 sítios, **32** consts | ⚠️ três vivem noutra crate e são lidas por 3–4 nós |
| `ReadChannel.label` | 25 sítios | família própria, **não medida** |

## §8 — A 5.ª fatia: as OPÇÕES de cada selector (fechada)

`562` chaves em [`node_options.rs`](../../../crates/ph2d-i18n/src/node_options.rs): **398** opções
em `114` arrays inline e o resto em **32 `const`**.

### §8.1 — ⚠️⚠️ A chave deriva do sítio de DECLARAÇÃO, e são dois porque são duas identidades

Um array escrito **inline** dentro de um `ParamUiHint` pertence a um par `(tipo, param)` e a mais
ninguém ⇒ `node.<tipo>.param.<param>.<i>`. Uma **`const`** é um VOCABULÁRIO com N leitores ⇒
`node.opts.<crate>.<CONST>.<i>`, uma vez para todos. ⛔ A chave derivada do nó daria N cópias do
mesmo texto, e mudar uma não mudava as outras.

⚠️ **E o NOME da const não é a identidade dela:** `MODE_LABELS` existe em **seis** crates de nó
com conteúdos diferentes (`2 · 2 · 3 · 2 · 6 · 2` opções). *Uma regra que lesse o nome daria a
mesma chave a seis vocabulários distintos* ⇒ a resolução é feita dentro da crate do **uso**, e só
um caminho qualificado (`ph2d_motion_region::`, `ph2d_nodegraph::`) sai dela.

### §8.2 — ⛔⛔ E aqui quem traduz é o PINTOR, ao contrário do rótulo — o motivo é o TIPO

O `label` de uma row é um `&'static str` e pode ser trocado por outro, logo a 4.ª fatia resolve na
**fronteira**. As opções são um `&'static [&'static str]` **dentro de um `ParamUiHint` que é
`Copy`** e existe em ~800 sítios de struct-literal: reconstruí-lo na ponte obrigava a alocar por
quadro no caminho de **OMISSÃO** (o cartão — o painel lateral está desligado desde 07/09).

⇒ o array carrega as chaves até ao consumidor, e há **três** consumidores, cada um com o seu gate:

| Superfície | Onde resolve | Gate |
|---|---|---|
| painel lateral | `rows_paint_kinds` (só as que desenha) | `the_option_captions_reach_ink_as_words_not_as_keys` |
| ESTADO do cartão (a row fechada) | `paint_card_params::shown` | `an_enum_row_shows_the_word_and_never_the_key` |
| LISTA do cartão (ao abrir) | `CardChoices::labels` | `a_static_choice_list_opens_with_words` |

⭐ **A 3.ª é de graça e isso está escrito no doc dela:** aquele método corre **ao abrir a lista**,
não por quadro.

### §8.3 — ⭐ A régua do painel é DIFERENCIAL, porque o arnês conta glifos

O `MockPanelHost` conta **glifos**, não texto ⇒ pinta-se a mesma row duas vezes — com as chaves e
com as palavras que elas resolvem — e exige-se a **mesma contagem**. Isso afirma a propriedade
inteira sem escrever um número: *se o pintor resolver, as duas pinturas são a mesma; se não, a das
chaves emite muito mais glifos, porque uma chave é `~4×` mais longa.* ⚠️ Um `assert` contra um
número mediria a fonte e o tema.

⭐⭐ **E o controlo da fixtura apanhou a MINHA fixtura errada à primeira corrida:** escolhi
`motion.mirror::keep`, que é servido por uma `const` — logo a chave dele é `node.opts.…` e não a
forma inline, e *sem o controlo o gate teria medido dois lados iguais por serem os dois
identificadores*.

### §8.4 — ⛔⛔ O que a migração partiu, e a metade MUDA

Seis gates de `ph2d-app-motion` liam o texto das opções. Cinco reprovaram em voz alta (comparam
listas contra palavras). O sexto é a família muda outra vez:

```rust
if labels.first() != Some(&"Sink") { continue; }   // ⟵ deixa de casar; a lista fica VAZIA
```

⭐ **Quem o tornou barulhento foi o piso de população** (`vistos.len() >= 3`) que já lá estava.
*Um censo que filtra por texto passa a medir nada, e só um piso o diz.*

⚠️ E o **lookup por texto** do `sim_demo::indice_de` — que procura a opção pelo nome e devolve
`None` em silêncio — está coberto: a mutação que lhe tira o `tr` faz reprovar **seis** gates de
cena que já existiam.

### §8.4-bis — ⭐⭐⭐ O achado: as chaves tornaram VISÍVEL uma duplicação que os gates não viam

Cinco gates de **vocabulário** comparam os arrays de dois ou mais nós para afirmar que eles usam
as mesmas palavras. Antes da migração todos passavam porque `&[&str] == &[&str]` compara
**conteúdo** — e com chaves derivadas do sítio de declaração, dois arrays que *dizem* o mesmo
deixam de ser iguais. Isso partiu os cinco, e a leitura de cada um é diferente:

| Gate | O que a reprovação revelou | Cura |
|---|---|---|
| `the_pivot_question_has_one_vocabulary` | ⭐ o `motion.transform` tinha uma **cópia inline** do vocabulário do pivô, ao lado da porta `ph2d_nodegraph::pivot::LABELS` que os outros três lêem | **o nó passa a ler a porta** |
| `wind_vocabulary` | duas `const` gémeas em crates irmãs **sem dependência entre si** | comparar o texto |
| `metric_vocabulary` | um nó declara inline, o outro numa `const` | comparar o texto |
| `pulse_edge_vocabulary` | a lista canónica do gate são PALAVRAS | comparar o texto |
| `presets_frame_themselves` | o `label` do molde é texto e o do selector é chave | comparar o texto |

⭐⭐ **O primeiro é o valioso:** a mensagem daquele gate já dizia *«os rótulos são os da PORTA»*, e
o código tinha duas listas que coincidiam por acaso. *Uma lei escrita em dois sítios ainda não é
uma lei — só uma PORTA é*, e foi preciso a chave, que carrega o endereço, para a duplicação
aparecer. ⇒ os outros quatro são divergências legítimas de **declaração**, e nesses o gate fica
mais forte por medir o que o artista lê.

### §8.5 — ⛔ O defeito do gerador, outra vez uma leitura curta demais

A 1.ª redacção lia o valor de `labels:` com `[^,\n]+` — que **corta no primeiro vírgula** —, logo
`&["a", "b"]` lia-se `&["a"` e **nenhum** array inline foi reconhecido; em vez disso 117 nomes
falsos foram tratados como consts e **30 foram reescritas**. ⇒ revertido antes de compilar, e o
valor passa a ser lido como *array (por chavetas) ou caminho (até à vírgula de topo)*.

Prova de mutação: **7 de 7 sangram**. Tecto de LOC curado por **corte**
(`ph2d-node-field-radial-sweep`: as quatro tabelas de UI saem para `params_ui.rs`, o molde que os
irmãos já têm) — as chaves são mais longas que as palavras e o `rustfmt` partiu os arrays.

### §8.5-bis — ⛔⛔ E o meu PORTÃO estava a ser lido por uma JANELA

Duas corridas seguidas do `nextest-impacted.sh` devolveram **sete** reprovadas cada uma, em
conjuntos **disjuntos** de crates — o que eu li como *«o conjunto impactado cresceu com o diff»*.

⛔ **Era mais simples e pior: eu canalizava a saída por `tail -8`.** Os ficheiros das duas
corridas têm `10` linhas e **nenhuma linha de `Summary`** — nunca vi a lista completa de
nenhuma delas, e as «sete» eram só as que cabiam na janela. *A memória desta casa já regista
isto por escrito: «um `tail` é uma JANELA, não um veredito».*

⭐ **A cura tem duas metades.** A primeira é não truncar o portão — o arnês já guarda a saída
inteira num ficheiro, e o `tail` só a destrói antes de lá chegar. A segunda é **parar de
descobrir uma família corrida a corrida**: uma varredura estática que cruza os textos migrados
com as linhas que comparam `labels` responde a lista toda de uma vez, e foi ela que provou que
não sobrava nenhum caso — três dos quatro «candidatos» que ela acusou eram **doc-comments**, e
o quarto usa o rótulo só na mensagem de erro.

### §8.6 — ⏳ O que FICA

| Alvo | Sítios | Nota |
|---|---:|---|
| `ReadChannel.label` | 25 | os canais nomeados de um `ParamWidget::Channels` — família própria |
| os `62` literais do censo de PORTA | — | `python3 scripts/censo-texto-pintado.py` |
| `ph2d-tool-vector` | 7 braços de rótulo | fora dos motores de nó |
