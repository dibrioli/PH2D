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

### §7.8 — ⏳ O que FICA para a 5.ª fatia

| Alvo | Sítios | Nota |
|---|---:|---|
| opções de `ParamWidget::Enum` **inline** | `400` em 117 sítios | chave `…param.<p>.<i>` |
| opções por **`const` partilhada** | 38 sítios, **27** consts | ⚠️ duas vivem noutra crate (`pivot::LABELS`, `motion_region::SHAPE_LABELS`) e são lidas por 3–4 nós ⇒ a const é a unidade, não o nó |
| `ReadChannel.label` | 25 sítios | família própria, não medida |

⚠️⚠️ **E a const partilhada tem uma armadilha que só a medição mostra: o NOME não é o
vocabulário.** `MODE_LABELS` aparece em **seis** crates de nó com conteúdos **diferentes** —
`2 · 2 · 3 · 2 · 6 · 2` opções —, logo ela é uma convenção de nome por crate e não uma tabela
partilhada. *Uma regra que lesse o nome daria a mesma chave a seis vocabulários distintos.*

Medido, as **genuinamente partilhadas** são exactamente três — as que vivem numa crate e são
lidas por outras:

| Const | Declarada em | Leitores | Opções |
|---|---|---:|---:|
| `ph2d_motion_region::SHAPE_LABELS` | `ph2d-motion-region` | 4 | 3 |
| `ph2d_nodegraph::pivot::LABELS` | `ph2d-nodegraph` | 3 | 3 |
| `BlendMode::LABELS` | `ph2d-nodegraph` | 1 | 3 |

⇒ a regra da 5.ª fatia é **de duas metades**: uma const **cross-crate** tem a chave do próprio
ENDEREÇO (um vocabulário, um conjunto de chaves, N leitores); tudo o resto — array inline e const
local — segue a lei desta fatia, `…param.<p>.<i>`. ⚠️ E as duas crates que declaram as
partilhadas são **foundational**, logo a tradução tem de acontecer na fronteira, como aqui.
