# 12 — O nome de um painel tinha DUAS fontes, e cinco discordavam no ecrã

> **Medido em 2026-09-17, `line/UIUX`.** A 7.ª fatia do HR-15: a **aba de um encaixe** passa a falar
> pela tabela de strings, e com ela o nome de um painel deixa de ter dois donos.
>
> As seis fatias anteriores fecharam a **fronteira dos motores** (`11_a_fronteira_dos_motores.md`):
> 1 942 palavras que viviam em tabelas de DADOS. Esta fecha a fronteira que sobrava do outro lado —
> o **contrato do painel**.

## §1 — O que a nota de dívida dizia, e porque ela estava meio certa

**Vinte** gates de painel carregavam, à letra, a **mesma** isenção nomeada — dezanove
`every_word_this_panel_shows_comes_from_the_string_table` mais o
`the_program_writes_no_word_into_this_panel` do painel autorado, que a diz por outras palavras:

```
"o `Panel::TITLE` e' um `const &'static str` que o registo le para a ABA, e o `tr` nao e'
 `const fn`. Fazer as abas falarem pela tabela e' mudar o contrato do painel nos 26 que o
 implementam -- obra propria, nomeada no handoff."
```

⚠️ **A premissa é VERDADEIRA e a conclusão era FALSA.** O `tr` não é `const fn` — mas a cura nunca
foi chamar o `tr` dentro de um `const`: é declarar a **CHAVE** e deixar quem PINTA traduzir. E o
[`TextKey::new`](../../../crates/ph2d-i18n/src/lib.rs) **é** `const fn`, desde que aquele tipo
existe — escrito, no doc dele, exactamente para tabelas `const` de rótulos.

⇒ `CLAUDE.md` §0.0: *antes de dizer que uma pergunta é inexprimível, procure o irmão do mecanismo
que está a usar.* O irmão estava na mesma crate, com a lei no cabeçalho.

## §2 — O que a migração achou: três superfícies, duas fontes

Medido antes da 1.ª linha de código, sobre o `HEAD` da linha:

| grandeza | número |
|---|---:|
| painéis que declaram `Panel::TITLE` | **28** |
| painéis que pintam o PRÓPRIO cabeçalho com `tr("panel.<…>.title")` | **20** (21 sítios) |
| painéis cujas duas superfícies **DISCORDAM** | **5** |

⛔⛔ **As cinco, com o que o artista lia ao mesmo tempo no ecrã:**

| painel | o CABEÇALHO dizia | a ABA dizia |
|---|---|---|
| `tokens` | Tokens | Design Tokens |
| `wet_tuning` | Wet Tuning | Wet Paint |
| `model3d` | 3D Model | Model 3D |
| `color_equalization` | Color EQ | Color Equalization |
| `bgremoval` | Bg Removal | Background Removal |

⚠️⚠️ **E o gate que existe para impedir exactamente isto não podia vê-lo.** O
[`the_tab_and_the_menu_call_a_panel_the_same_thing`](../../../crates/ph2d-panel-registry-init/tests/it/the_tab_and_the_menu_call_a_panel_the_same_thing.rs)
existe desde 2026-09-08, abre com *«UM PAINEL TEM UM NOME, não dois»*, e compara a **aba** com o
**MENU**. A terceira superfície é o painel a **nomear-se a si próprio**, e ela nunca entrou na conta.

⇒ *um painel tem três sítios onde se apresenta, e um gate que mede dois deles lê-se como se medisse
o assunto.*

## §3 — O que mudou

1. **O contrato:** `Panel::TITLE: &'static str` → `Panel::TITLE: TextKey`
   ([`panel_trait.rs`](../../../crates/ph2d-editor-core/src/panel/panel_trait.rs)), com
   `PanelManifest::title` e `slot_tabs::Occupant::title` a seguirem. Quem PINTA a aba traduz
   (`o.title.tr()`), nos dois sítios que a lêem (largura natural · pintura) mais o fantasma do
   arrasto.
2. **Uma chave por painel, lida pelas DUAS superfícies:** os 21 sítios de cabeçalho passam de
   `tr("panel.x.title")` a `<Painel>::TITLE.tr()` — a divergência deixa de ser medível porque deixa
   de ser **exprimível**.
3. **Os 5 desacordos resolvem-se pela palavra da ABA** — é a que o menu *Window* também diz, e há
   gate vivo a atar as duas. O nome abreviado que a família das ferramentas de imagem de facto
   precisa é o do **chip da barra do topo**, que tem chave própria (`tool.color_equalization.label`
   = `"CEQ"`): *encurtar o título do painel para caber num chip era responder à pergunta do vizinho.*
4. **Oito painéis ganharam a primeira chave que alguma vez tiveram** (`inspector`, `motion_graph`,
   `motion_params`, `painter_layers`, `skeleton`, `authored`, `widget_gallery`, `widget_lab`) — eles
   não pintam o cabeçalho pela tabela, logo nunca precisaram de uma.

⚠️ **`TextKey` e não `&'static str`**, e a razão está escrita no tipo: guardadas como `&str`, uma
chave e um texto são o **mesmo tipo**, e quem se esquece de traduzir **compila**, passa em todo
teste que não leia o pixel, e pinta `panel.tags.title` na aba — com `leak_key`, um vazamento **por
quadro**. Com o tipo, esse esquecimento é erro de compilação; e a troca do tipo é o censo mais forte
que existe: **os 28 implementadores param de compilar até alguém os migrar.**

## §4 — Duas coisas que a migração revelou de graça

### §4.1 — O fantasma do arrasto MENTIA duas vezes

O recurso do `slot_tabs_drag` era `("", IconId::Inspector)` para um `panel_node_id` que nenhum
manifesto reclama — um nome **vazio** e o glifo de **outro painel**. Com o título a virar `TextKey`
o nome vazio deixou de ser exprimível sem custo (`tr("")` faz `leak_key` **por quadro** de arrasto),
e a resposta certa apareceu: *se o painel arrastado não está no registo, não há nada que arrastar.*

### §4.2 — ⛔ A régua NOVA acusou três sítios CORRECTOS na primeira corrida

O `no_panel_paints_its_own_name_beside_the_key` nasceu a perguntar *«a linha tem `tr(` e acaba em
`.title")`?»* e reprovou sobre **produto certo**:

```
crates/ph2d-panel-model3d/src/paint.rs:251        · tr("panel.model3d.select.title")
crates/ph2d-panel-painter-layers/src/paint_impasto.rs:84 · tr("panel.painter_layers.impasto.title")
crates/ph2d-panel-tokens/src/paint.rs:209         · tr("panel.tokens.contrast.title")
```

Os três são o título de uma **SECÇÃO**, não o nome do painel. ⇒ *o discriminador é a FORMA da
chave*: o nome de um painel é `panel.<id>.title`, com **exactamente três** segmentos;
`panel.<id>.<secção>.title` tem quatro e é outra grandeza. **Uma régua que mede o SUFIXO de uma
chave mede a palavra final, não o assunto dela.**

⚠️ E a mesma correcção fechou um ponto cego que ainda não tinha mordido: ela lia **linha a linha**,
logo um `tr(\n    "panel.x.title",\n)` partido pelo `rustfmt` seria invisível — os 21 sítios desta
fatia eram de uma linha **por acaso**. Hoje ela lê o ficheiro inteiro e salta espaço.

### §4.3 — O `widget-lab` já tinha a forma certa

Ele pintava o próprio cabeçalho com `WidgetLabPanel::TITLE` — uma porta só, desde sempre; faltava-lhe
apenas a tradução. *Quando a migração encontra um sítio que já estava no molde, o molde estava certo.*

## §5 — Os gates

| gate | o que afirma |
|---|---|
| `every_tab_speaks_through_the_string_table` | toda chave de aba **resolve** (`tr(k) != k`), e **deriva** do `Panel::ID` |
| `every_named_prefix_exception_still_names_a_live_panel` | a metade justa das 2 excepções de prefixo |
| `no_panel_paints_its_own_name_beside_the_key` | nenhum painel volta a ter uma segunda porta até ao próprio nome |
| `the_tab_and_the_menu_call_a_panel_the_same_thing` | (existente) agora compara **texto contra texto**, com os dois lados a virem da tabela |

**Prova de mutação: 4 de 4 sangram**, com controlo negativo (os quatro verdes sem mutação nenhuma):
a chave que deixa de existir na tabela · a chave que **resolve** mas é a de outro painel · um painel
que volta a ter a segunda porta até ao próprio nome · a aba e o menu a discordarem outra vez.

⛔⛔ **E o ARNÊS apagou trabalho antes de dizer a verdade.** Ele restaurava com
`git checkout -- <ficheiro>`, e numa árvore **SUJA** isso não desfaz a mutação — desfaz a **FATIA**,
devolvendo o ficheiro ao `HEAD`: **três ficheiros desta wave foram apagados** (o `TITLE` tipado do
painel, o pintor do cabeçalho e as nove entradas novas da tabela). ⭐ **Quem o disse foi o CONTROLO
DO FILTRO**: as três mutações seguintes casaram **zero** testes, porque a árvore deixara de compilar,
e o arnês recusou-se a chamar-lhes *«sobreviveu»*. *Sem esse controlo eu teria lido três falsos
sobreviventes e ido curar gates que estavam certos.* ⇒ a cópia de segurança é do ficheiro **como
está** (`cp`/`mv`), nunca do índice — ou a wave comita-se antes de mutar.

⚠️ **Duas excepções de prefixo, com o mecanismo:** `bgremoval → panel.bg_removal.*` (18 chaves
irmãs) e `color_equalization → panel.color_eq.*` (35). Renomear o prefixo inteiro é outra obra;
renomear só o `.title` deixaria a família a falar duas línguas. *Um painel NOVO não tem por onde
escolher* — a chave dele é `panel.<id>.title` ou o gate reprova.

⚠️ **O piso de população é `20` e não `28`**: quatro painéis (`flip`, `flip_frames`,
`painter_layers`, `wet_tuning`) ficam fora das features de omissão, e um registo vazio faria as duas
metades passar por vácuo.

## §6 — ⏳ O que fica

| alvo | nota |
|---|---|
| **o MENU tem chave PRÓPRIA para a mesma palavra** | ⚠️ ver §6.1 |
| `ph2d-tool-vector` | 7 braços de rótulo, fora dos motores de nó |
| o censo de PORTA | `python3 scripts/censo-texto-pintado.py` — ⚠️ o que ele conta são **glifos e letras soltas** (`X`/`W`/`M`/`◀`) e a **bancada de widgets**, que a régua registada não lê como língua; a fracção de LÍNGUA dele fechou aqui |
| `ph2d-panel-tags`, `-widget-lab`, `-widget-gallery`, `-skeleton`, `-model3d`, `-physics`, `-sculpt3d`, `-wet-tuning` | **não têm gate HR-15 por crate** — a régua registada nunca correu sobre elas |

### §6.1 — ⚠️ Duas superfícies fecharam, a terceira fica atada por um GATE e não por construção

O menu *Window* nomeia ~13 painéis, e cada linha dele carrega uma `TextKey` **própria**
(`chrome.menu.authored_ui`, `chrome.menu.assets`, …). ⇒ para esses painéis **há duas chaves com a
mesma palavra**: a do menu e a da aba.

⚠️ **Isto é anterior a esta fatia e não foi criado por ela** — e o que as mantém honestas é o
`the_tab_and_the_menu_call_a_panel_the_same_thing`, que compara os dois **textos traduzidos**: no
dia em que uma segunda língua traduzir uma e não a outra, ele reprova. *Um gate que compara é mais
fraco que uma construção que não deixa divergir*, e a diferença fica aqui nomeada.

⛔ **O bloqueador tem nome:** a camada de chrome **não depende de painel nenhum** — é por isso que o
`MODULE_TRUTHS` guarda o `Panel::ID` como **literal**, com a cerca escrita ao lado de cada entrada.
A ponte que colapsaria as duas chaves numa é o **registo em runtime** (`with_registry_ref` →
`manifest.title`), que é exactamente o que aquele gate já faz — obra própria, de uma wave.

### §6.2 — As crates sem gate nenhum

⚠️ **A última linha da tabela é a medição mais importante que esta fatia deixa aberta.** Oito crates de UI —
entre elas painéis que o artista usa todos os dias — não têm o `every_word_this_panel_shows_comes_
from_the_string_table`. *Um censo que não corre sobre uma crate não afirma nada sobre ela*, e é ali
que o próximo literal nasce calado.
