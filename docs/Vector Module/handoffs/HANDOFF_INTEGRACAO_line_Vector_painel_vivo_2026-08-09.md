# HANDOFF DE INTEGRAÇÃO — `line/Vector`: o painel autorado fica VIVO, e o sistema de widgets é AUDITADO (2026-08-09)

**Status:** FECHADO 2026-08-09 · no `main` em `7a998424e` (o commit que trouxe este arquivo).

> **A LINHA ESTÁ FECHADA e pede ordem de integração.**
> **47 commits · 100 arquivos · +9.420/−917**, sobre `17a0f6d6d`.
> Supersede nada: é a continuação da jornada de UI/UX que integrou em 08/08.
>
> ⚠️ **A jornada tem DUAS metades, e elas se lêem de formas diferentes:**
>
> | | commits | o que é | como se julga |
> |---|---|---|---|
> | **A — a FEATURE** | 15 (até `f3ba220c8`) | o painel autorado fica vivo | **smokada** — os oito passos da cena `=62`, aprovados pelo Enio |
> | **B — a AUDITORIA** | 32 (de `228a0d6e9`) | dezoito achados no sistema de widgets | **gates** — não há smoke, e o §4b diz por quê |
>
> A metade B **não tem cena própria de propósito**: catorze das correções são de geometria e
> de gate, e as que o artista veria são *bugs a desaparecer*, não features a demonstrar. O que
> ela pede do integrador é a **varredura**, não o olho. O inventário fechado vive ao lado, em
> [`AUDITORIA_widgets_achados_2026-08-09.md`](../Estudos/AUDITORIA_widgets_achados_2026-08-09.md).
>
> ⚠️ **O plano que esta linha executava está FECHADO** — `WidgetKind::ALL` mede **20**, e as
> quatro waves do levantamento de 08/08 saíram todas (cobertura 67,1% → ~100%). A linha está
> num limite natural: não há trabalho cortado ao meio aqui dentro.

## §1 — O que esta linha entrega

A W8b tinha entregue *a árvore autorada vira painel*. Faltavam quatro coisas, e as quatro
fecharam:

1. **Os tipos que faltavam vestem** — `NumberInput`, `LevelMeter`, `ColorSwatch` e
   `IconButton` (catálogo **12 → 16**).
2. **O parâmetro por-tipo** — a `SkinParam`, o canal que leva à pele o que nem o
   retângulo, nem o rótulo, nem os tokens determinam: a **cor** de uma swatch e o
   **glifo** de um botão de ícone.
3. **O painel deixa de ser um SNAPSHOT** — ele segue o documento aberto, quadro a quadro.
4. **A família de LISTA** — `Tabs`, `RadioGroup`, `SegmentedAdaptive` e `Dropdown`
   (catálogo **16 → 20**), com as opções a virem dos **filhos que o artista desenhou** e
   **clicáveis uma a uma**. Era o maior buraco estrutural do levantamento de 08/08:
   **14,6% da UI real**.

## §1b — E a metade B: a AUDITORIA (32 commits)

Dezoito achados, **um mecanismo só**: *um fato com duas cópias que discordam.* O índice da
opção contra a contagem de opções · a régua do pintor contra a cópia que o dispatch faz dos
números dela · a moldura do cartão contra a caixa que lhe deram · **o NOME de um gate contra
o que o corpo dele mede**.

**O que muda o PRODUTO** (o artista vê a diferença):

| | o defeito, com o número que o mediu |
|---|---|
| `41a95074e` | a opção marcada podia **não existir** — índice fora da contagem, e a família reagia de **três** maneiras diferentes |
| `82966f7c3` | a lista longa **saía da tela** em vez de rolar |
| `101c98b2a` | a `TextArea` tinha **DUAS réguas** — o dispatch copiava os números do pintor, e o caret caía na linha errada |
| `042b6cb95` | o texto de uma row saía dela: **337 px à esquerda** no `list_item`, **três linhas por cima das seguintes** no `key_value_list` |
| `db5cd9615` · `b4e7c2764` | cinco slots de largura **FIXA** dentro de hosts **VARIÁVEIS** (o chip de unidade · o `X` da tag · a ORDEM do clamp do popover · o cartão) |
| `8770f7cae` | **a faixa de arrasto do título comia 2 px do botão de FECHAR** — e num dos dezasseis painéis o X **arrastava em vez de fechar** |
| `9ba26966f` · `aa68663b9` · `6900a8283` · `df43a5c08` · `85f0c2f02` | o thumb saía do corpo · o load esquecia a posição autorada · uma row morta legava o valor · abrir o picker **escrevia o documento** · a borda da 1ª row era da faixa de arraste |

**O que muda a SUÍTE** (o produto não muda; o que muda é quanto ela vale):

- **Seis gates não podiam falhar pelo motivo que alegavam** — o do topo do Chroma media
  `M/M` (empurrava e lia pela **mesma função**, inversas exatas ⇒ `1.0` para qualquer
  máximo, *incluindo o `0.001` que era o bug reportado*) · quatro gates de roteamento
  passavam **VAZIOS** · dois mediam ordem e largura positiva onde a pergunta era a **borda**.
- **O gate de staleness que o cabeçalho do painel gerado PROMETIA não existia** — o arquivo
  abria com *"NÃO EDITE À MÃO — editar a saída de um gerador é o que deixa o gate de
  staleness vermelho"* e nada chamava `emit()` fora dos testes do próprio gerador. Agora
  existe, e **disparou no primeiro uso real**.
- **Cinco doc-comments descreviam um produto que não existe**, e quatro deles nomeavam uma
  **derivação de token errada** — `SECTION_LABEL_TO_CONTROL_PX = 4.0` dizia `Xxs` (que vale
  **2**) ⇒ seguir o comentário encolheria o gap de toda seção do app.

⚠️ **E TRÊS itens do inventário DISSOLVERAM na medição** — dois porque eu classifiquei pela
FORMA em vez do mecanismo (um gate que *parece* auto-referente e é um **pin** legítimo; uma
"comparação golden" que **não existe**), e o bloco inteiro do `Xl3`, cujos treze sítios medem
não-defeito. **As duas varreduras falsas foram as que mais renderam**: uma achou o gate de
staleness ausente, a outra achou os 2 px do botão de fechar. *O valor de varrer um item falso
é o que se tropeça no caminho.*

## §2 — As leis, e onde cada uma mora

### O parâmetro por-tipo é um CAMPO com neutro, nunca um argumento por tipo

`SkinParam { rgba, icon }` — o molde do `KernelResolver` dos Motion Nodes. Um tipo que
não consome um campo não sabe que ele existe, e `SkinParam::default()` é o mundo
pré-wave **byte a byte**.

⚠️ **A premissa que estava errada e foi corrigida por escrito:** nada nesse molde exige
que o campo seja *pequeno*. Foi essa suposição que me fez escrever, no levantamento, que
o ícone *"não entra pelo mesmo canal"* — e o `IconGlyph` já codificava exatamente as duas
respostas possíveis para *qual ícone?*.

### A precedência do ícone é UMA porta

`ph2d_editor_core::widget::icon_glyph(chosen, drawn)` — **a escolha vence, e a ausência
dela É o desenho**. Não há terceiro estado, então as duas rotas nunca podem discordar; um
slug que este build não conhece degrada para o desenho. **Três** consumidores a
perguntam (a ponte do canvas, o plano do painel gerado, o painel compilado), e três
cópias divergiriam com modos de falha diferentes.

### O SLUG é a chave durável, nunca o discriminante

O discriminante de `IconId` é a posição **alfabética do arquivo SVG** (`enum_order_matches_svgs`
o pina), então acrescentar um ícone desloca todos os posteriores. Um documento que
guardasse o número passaria a desenhar outro glifo, em silêncio, num dia em que ninguém
tocou no documento.

⚠️ **E isto achou um defeito VIVO no `main`:** `IconId::Color` desenhava os sliders e
`IconId::ColorEqualization` a paleta — a ordem de declaração estava trocada contra a
alfabética, e o `enum_order_matches_svgs` só afirmava `*id as usize == i`, trivialmente
verdadeiro para qualquer lista na ordem em que foi escrita. Consequência de produto: o
botão *"Add adjustment"* do painel de camadas do Painter mostrava a paleta. Corrigido, com
gate novo que compara o `Debug` contra o PascalCase do slug.

### O glifo INVERTE Y, porque os dois espaços discordam

O documento é **Y para cima** (a câmera inverte: `scale_non_uniform(k, -k)`); a caixa de
24×24 do ícone é **Y para baixo**, porque é a viewbox do SVG. `icon_face` é o único ponto
em que uma geometria de documento entra naquela caixa ⇒ a conversão pertence ali, e é por
isso que **uma correção consertou as duas metades**.

⚠️ **Por que shipou:** os cinco gates existentes mediam a **caixa** (extensão, centragem,
razão de aspecto, degeneração), e **uma caixa é simétrica sob inversão**. O gate novo
pergunta qual **vértice** está no topo de cada lado.

### O documento aberto vence o código colado

`rows::with_rows` — a precedência num lugar só, perguntada por quatro consumidores.
⚠️ **Isto NÃO remove o recompilar para SHIPAR:** o gerado continua sendo o artefato
compilado, que é o que passa pelos gates de parity/a11y/LOC. O que saiu foi o ciclo de
compilação de dentro do laço de **autoria**. Vivo para autorar, compilado para entregar.

- **Uma varredura, duas representações** (`ui_panel_spec::Authored`): o vivo quer a
  **curva**, o gerado quer **texto** (um `const` não constrói um `BezPath`). Só o lado do
  código faz `to_svg`.
- **`thread_local`, não um lock:** publicar e pintar são os dois na thread da UI, e estado
  global entre testes paralelos é a flake que a `ph2d-painter-brush` pagou.
- **`Option` e não `Vec` vazio:** *não há documento* e *documento sem controles* são
  coisas diferentes.
- Publica **só com o painel visível** — a lei do ADR-0125.

### O botão autorado vira SINAL

O painel autorado era **o único do app sem ponte**. O vazamento (fila só empurrada,
crescendo sem teto) era a metade menor; a maior é que **um aperto não tem valor no
store**, então o intent era o único canal que o carregava — e **todo botão autorado era um
controle morto**.

`SignalOrigin::Control` é a terceira origem da saída do R0, onde markers da timeline e
contatos da física já publicam. ⚠️ **Só o `Fired` vira sinal** — um slider já diz o que
vale pelo store, e publicá-lo aqui também poria o mesmo fato em dois fios.

⚠️ **A POSIÇÃO é load-bearing:** a ponte publica **depois** de o quadro virar e **antes**
do dreno. Fora dessa janela o aperto chega um quadro atrasado — invisível num toast e
visível no dia em que o consumidor for som.

### Um controle que toma opções POSSUI os seus filhos

`WidgetKind::takes_options()`. Um controle de lista não tem *um valor*: tem N rótulos e um
índice marcado, e o lugar nativo desses rótulos num editor vetorial são os **filhos**. A
árvore já exprime contenção e o artista já os nomeia na Hierarquia ⇒ **zero campo novo,
zero schema**.

A consequência é uma lei de posse que muda a leitura da **ÁRVORE**, não só a da pele: a
varredura **não desce** os filhos de um controle de lista como rows. Sem ela, desenhar três
abas dentro de uma faixa daria uma faixa **e mais três linhas soltas** no painel.

⚠️ **E ela derruba uma premissa que o levantamento de 08/08 deixou escrita.** Ele chamou a
família de *estrutural*, por oposição a *omissão de fiação*, e a leitura implícita era que o
canal de side-metadata não a alcançava. **Nada no molde do canal exige que o campo seja
pequeno** — `options: &[String]` é um campo com neutro (a lista vazia) exactamente como
`rgba: Option<_>` é.

### O dropdown é o único que não cabe num passe de pintura só

`WidgetKind::defers_a_popover()`, e a divisão é de **LAYOUT**, não de gosto. Os três irmãos
desenham as N opções DENTRO do retângulo que lhes foi dado — quem os pinta acabou quando a
função retorna. O dropdown desenha **um chip** e guarda a lista para uma superfície que tem
de aparecer **por cima do que for pintado depois dela**.

⇒ O painel colecta os abertos durante o corpo e pinta-os num **passe diferido**, cuja ordem
decide três coisas: **depois do `pop_layer`** (senão a lista da última row é cortada pelo
recorte do corpo) · **depois das rows** (senão elas pintam por cima dela) · **depois dos
punhos de chrome** (o hit é *último-registado-ganha*, e uma lista sobre o canto de
redimensionar tem de tomar o clique).

⚠️ **A pele NUNCA o pinta aberto** — nem no painel nem na prévia do canvas. Ela desenha o
chip; a lista é do passe de quem tem `hit_index` para registar as opções, que a prévia não
tem e (§2 do plano) não deve ter.

### A escolha diz QUAL opção, e fecha a lista

`AuthoredIntent::Choice { key, index }` — e ele fecha um vão que a própria linha shipou três
commits antes: as três primeiras da família caíam em `Fired`, que diz *"alguém mexeu neste
controle"* e **não diz QUAL opção**.

⚠️ **Fechar é a metade que o despacho genérico não faz.** Ele alterna o `open` de quem foi
CLICADO, e quem foi clicado é a **opção**, não o chip. Sem essa linha a lista fica aberta
depois de escolher, e o clique seguinte — que o artista dá para a fechar — escolhe outra
coisa.

⚠️ **`rows::selected_of` devolve `Option<usize>`, e é isso que a mantém uma porta ÚNICA em
vez de duas.** Ela responde duas perguntas de uma vez (*este controle É de escolha?* e
*qual?*), e os dois consumidores querem metades diferentes: o `paint` cai em `0` porque um
controle de opções tem de desenhar alguma marcada, e o `event` precisa do `None` para saber
que aquela row não emite escolha.

### Clicar numa opção SELECIONA aquela opção — e a geometria sai de UMA porta

⚠️ **Defeito reportado pelo Enio no smoke, e a hipótese dele estava errada de um jeito útil**
(*"Tabs não seleciona as tabs no painel vivo, talvez porque não tenha nada dentro"*): as três
opções **estavam lá** e a faixa desenhava-as. Faltavam duas coisas, e cada uma sozinha já
bastava para o defeito.

1. **A row publicava UM retângulo de hit** para a faixa inteira — o clique chegava sem poder
   dizer *qual* aba.
2. **Nada neste repo escreve `InteractiveState::Tabs` nem `::Radio`** — nem o despacho
   genérico, nem pintor nenhum. Medido por grep, não suposto.

⚠️ **E o defeito é da própria linha, um commit antes:** o dropdown ganhou ids por opção e os
três irmãos em linha ficaram na lei antiga. O doc-comment do `skin.rs` já dizia *"quem precisa
de id por opção é o painel COMPILADO, que os deriva da chave"* — a regra estava escrita e foi
aplicada a um membro só da família.

**`skin::inline_option_rect(kind, host, i, n)` é a porta única da geometria**, e mora ao lado
da pele porque é ela que decide *como um tipo é desenhado*, logo é ela que pode responder
*onde a opção `i` caiu*. Uma cópia no painel acenderia a aba num lugar e a faria responder
noutro. Ela devolve `None` para quem não toma opções **e para o Dropdown** (a lista dele não
está na row), então o painel não precisa de um `if` por tipo.

**`rows::select_in` é a IRMÃ EXATA do `selected_of`** — se as duas discordarem sobre qual campo
da variante guarda a seleção, o controle desenha uma opção e devolve outra ao ser clicado.

⚠️ **Os dois gates não são redundantes, e a mutação prova:** *"toda opção ocupa a row inteira"*
**não é vista pelo seam** (com todas sobrepostas o hit é *último-registado-ganha* e o clique
ainda acerta alguma), e só o gate de geometria a pega.

## §3 — Colisão: o que esta linha move

| Eixo | Estado |
|---|---|
| `PROJECT_SCHEMA` | **69, INTOCADO** (`git diff main -- project.rs` vazio) |
| `VEC_SCENE_SCHEMA_VERSION` | **14, intocado** |
| Contrato congelado (§6) | **intacto** (`ph2d-nodegraph`, `ph2d-core/src/tool.rs` com diff vazio) |
| Registro do `ph2d-ecs` | **54 → 55** (`VecWidgetIcon`), e os **DOIS espelhos 55 → 56** |
| ADR | **nenhum** ⇒ fora de toda disputa de número |
| `Cargo.toml` | **1**, e é aresta interna (`ph2d-panel-authored`) |
| Dep externa nova | **nenhuma** (`Cargo.lock` sem `+name`) |
| `WidgetKind` | 12 → **20** (variantes apendadas; o `code()` é estável) |
| `ph2d-i18n` | `lib.rs` **partido** — as 186 chaves `panel.vector.*` para o irmão `vector.rs` (701 → 520 LOC) |

⚠️ **Esta linha do registro estava ERRADA neste handoff até 09/08** — ela dizia *"intocado,
`VecWidgetIcon` já existia"*, e medido contra a `main` (54 / 55 / 55) contra o tip (55 / 56 / 56)
o componente é **novo**. É a linha mais cara da tabela para se errar: **o contador é TRÊS**
(`ph2d-ecs` conta só os próprios; `ph2d-render` soma o `Sprite`; `ph2d-script` soma o
`LuauScript`), cada um roda **só na suíte da própria crate**, e este repo já pagou o vermelho-
latente dessa família três vezes. *Um handoff que erra aqui manda o integrador procurar o
conflito no lugar errado.*

⚠️ **O split do `ph2d-i18n` é o único ponto de merge sensível desta linha.** Ele foi por
**assunto** (o teto de LOC pegou nas 3 chaves novas), e uma linha que acrescente uma chave
`panel.vector.*` ao `lib.rs` **funde limpa contra um arquivo de onde a tabela saiu** — o
mesmo modo de falha que o corte do `project.rs` produziu na integração de 04/08. Confira que
o `tr` de toda chave `panel.vector.*` continua a resolver.

## §4 — Smokes

**`env PH2D_BUILD_SMOKE=62 cargo run -p ph2d-host-desktop --release`** — a cena imprime
o que montou; ⚠️ **se a linha `[ui-panel] … 9 row(s)` não aparecer, PARE.**

Os oito passos estão **aprovados** — os 1-6 na primeira rodada, os 7-8 depois da família de
LISTA e do dropdown.

1. A swatch **Tint** mostra o preenchimento *dela*, não o dos irmãos.
2. **Play** desenha a ESTRELA que o artista fez — **em pé**, e igual no canvas e no painel.
3. **Trash** mostra o lixo do catálogo, porque foi ESCOLHIDO; o picker troca os dois juntos.
4. O cabeçalho **Appearance** dobra e o painel **encolhe**.
5. **Renomeie um filho ou edite os nós da estrela: o painel muda enquanto você desenha.**
6. Com `PH2D_SIGNAL_LOG=1`, clicar um botão dá toast + `[signal] … <- controle autorado`.
7. ⚠️ **A row `View` é uma faixa de ABAS**, e as opções dela são os três filhos `Design` /
   `Preview` / `Code`. Renomeie um na Hierarquia: a aba muda de nome. **Se em vez disso
   aparecer uma linha nova solta no painel, a lei de posse quebrou.** E **clique na aba
   `Code`: ela acende** — é o defeito reportado no smoke de 09/08, e ele tinha DUAS causas
   (um retângulo de hit para a faixa inteira · ninguém a escrever a seleção).
8. ⚠️ **A row `Blend` ESCONDE as opções.** Clique no chip: a lista abre **por cima de tudo**,
   inclusive do canto de redimensionar. Escolha `Screen`: ela **fecha** e o chip passa a
   dizer `Screen`. **Se a lista ficar aberta depois da escolha, PARE.** E a pergunta de olho:
   arraste o painel para baixo até o chip ficar perto do fundo, abra outra vez — a lista tem
   de virar para **CIMA**, e as opções têm de responder ao clique **onde estão desenhadas**.

## §4b — A metade B não tem smoke, e isto é o argumento

⚠️ **Não escrevi cena para a auditoria de propósito, e vale ler o porquê antes de pedir uma.**
Das catorze correções, seis mudam **gates** (o produto não se move) e as oito que mudam o
produto são **bugs a desaparecer** — um caret que passa a cair onde o dedo clicou, um texto
que deixa de sair da row, 2 px de botão que voltam a fechar em vez de arrastar. Uma cena que
os *demonstrasse* teria de encenar cada defeito, e o oráculo de cada um já é um gate com
mutação provada — que é uma pergunta mais afiada do que a que o olho faz aqui.

**O que a metade B pede do integrador é a VARREDURA**, e ela está no §7. As duas coisas que
o olho ainda decide — e que aparecem na cena `=62` que já existe — são: **a lista longa ROLA**
(passo 8) e **o painel não deforma** depois das cinco correções de geometria.

⚠️ **UMA mudança de comportamento é do app inteiro, não desta cena:** a faixa de arrasto do
título encolheu **2 px à direita em TODOS os painéis**. Se algo se arrastar diferente, é aqui.

## §5 — Aberto, com o preço ao lado

- ⚠️ **A guarda do popover VAZIO não tem gate, e está medido:** a mutação que a remove
  **sobreviveu** à suíte inteira. Um popover sem opções não regista opção nenhuma, logo
  **não come cliques** — a primeira versão do doc-comment afirmava que sim, e foi a mutação
  que a desmentiu. O efeito é **só visual** (um painel flutuante vazio), e o harness de
  painel deste repo lê retângulos de hit e nunca a cena. Fica declarada no `paint.rs`, no
  molde das defesas em camada que o ADR-0145 documentou em vez de gatear.
- **Duas molduras autoradas:** o painel vivo mostra a **primeira**. Escolher pela SELEÇÃO
  é decisão de produto — inventar um desempate que o artista não vê seria pior.
- **Mutar um GRUPO** de opções (esconder uma aba) não existe: um filho de controle de lista
  é um rótulo, não um controle. Se vier, é campo autorado — decisão de produto.
- **`AuthoredIntent::Value`/`Flag`/`Text`** são drenados e descartados (o store é a
  autoridade). Se um dia quiserem consumidor, ele entra na mesma ponte.

## §6 — Para o integrador

- ⚠️ **Rode `--test the_two_halves_read_the_glyph_through_one_door` e CONTE 3 testes.**
  Este arquivo já perdeu gates por edição em massa nesta casa.
- ⚠️ **O golden do painel** (`generated/panel.rs`) é derivado da cena do smoke. Se um
  merge mover `icon_face` ou a varredura, **re-gere** com
  `cargo test -p ph2d-host-desktop --bins print_the_generated_panel -- --ignored --nocapture`
  e confira que o diff é vazio — foi assim que o glifo invertido foi pego.
- O arch-gate da porta única ancora no **nome** `icon_face`, não em `icon_face(`: ele já
  ficou vermelho sobre produto correto quando a metade do spec passou a **passar** a função
  em vez de a chamar.
- ⚠️ **O `PARE` da cena e o gate de contagem são DERIVADOS**, não literais
  (`ui_panel_smoke::expected_rows`). Um número escrito à mão só sabe dizer *"mudou"*, e a
  pergunta é outra: *a lei de posse continua de pé?* — ele tem de disparar quando uma opção
  escapa para a moldura e ficar quieto quando alguém acrescenta um controle.
- ⚠️ **`skin/geometry.rs` e `skin/geometry_tests.rs` estão no `A11Y_OPT_OUT`** do
  `hr12_widgets_a11y`, com o motivo escrito ao lado: o primeiro é **layout puro** (não pinta
  nada) e o segundo é módulo de teste. Um arquivo novo em `src/widget/**` que PINTE não pode
  entrar nessa lista.
- ⚠️ **DUAS corridas de mutação saíram CONTAMINADAS e a lição vale mais que o conserto:** a
  porta nova cruzou o teto de LOC do `skin.rs` e depois o arquivo do split tropeçou no HR-12
  — **os dois falhavam na BASELINE**, então todo veredito *"SANGRA"* das duas primeiras
  rodadas era do gate errado. Só a terceira, com a árvore verde, mede o que alega.
  *Confira a baseline VERDE antes de acreditar num `SANGRA`.*
- ⚠️ **O parentesco da cena é por NOME, numa porta única** (`authored_parent`), e isso é
  cicatriz: a versão por índice contou a tabela à mão, errou por um, pendurou as opções na
  entidade errada — e **a contagem de rows deu certa por acidente** (a entidade adotada não
  vestia widget), então o `PARE` passou sobre um painel com a faixa **sem opção nenhuma**.

## §6b — Para o integrador, o que a AUDITORIA acrescenta

- ⚠️ **A `PANEL_HEADER_CLOSE_RESERVE` mudou de 40 para 42, e o `panel_drag_handle_rect`
  passou a CLAMPAR.** Isso toca a faixa de arrasto de **todos** os painéis (encolhe 2 px à
  direita). A lei mora na porta, não nos 21 chamadores, e o `min` **só encolhe** — nenhum
  painel pode passar a sombrear o que não sombreava. Se um merge trouxer um painel novo, ele
  nasce coberto.
- ⚠️ **Quatro doc-comments tiveram a derivação corrigida** (`SECTION_LABEL_TO_CONTROL_PX`
  dizia `Xxs`, que vale 2; `SECTION_INNER_ROW_GAP_PX` dizia `Sm`, que vale 6). **Os NÚMEROS
  não mudaram** — se um merge "canonizar" um deles para o token nomeado, o gap de toda seção
  do app muda.
- ⚠️ **O `generated/panel.rs` ganhou um gate de staleness de verdade** e uma porta
  `regenerate` (`#[ignore]`). Se o emissor mudar de formato, o gate fica vermelho e a
  resposta é a porta — **nunca** editar o gerado à mão, que é o que o cabeçalho dele proíbe.
- ⚠️ **`crates/ph2d-panel-authored/Cargo.toml` ganhou o `ph2d-ui-codegen` em
  `[dev-dependencies]`** — em `[dependencies]` seria o painel compilado a depender do próprio
  gerador. É a única mudança de `Cargo.toml` da linha; **nenhum pacote externo novo**.
- ⚠️ **`ph2d-runtime` recebeu um variant `SignalOrigin::Control` + `Signal::from_control`,
  ADITIVOS** — a linha CONSUMIU a crate-folha que o `line/runtime` R0 acabou de integrar, em
  vez de disputar o nome dela. **Nenhuma dependência entrou**, então o gate estrutural
  `the_event_core_is_a_leaf` continua de pé. A adjacência que o handoff do R0 nomeia (*o
  runtime de UI querer este nome*) **não aconteceu aqui**.

## §7 — Gate de fechamento, e o que o integrador re-roda

**Medido no tip da linha, `516532271`:** `scripts/nextest-impacted.sh` **8560/8560 verdes,
834 skipped** · `cargo clippy` limpo nas crates tocadas · `cargo fmt --all` aplicado · golden
do painel **em dia**.

Na árvore COMBINADA, além do gate padrão, estes quatro são os que esta linha pode derrubar —
cada um por um motivo diferente, e nenhum deles é alcançado por um `cargo test -p` filtrado:

| rode | porque ESTA linha o move |
|---|---|
| `cargo test -p ph2d-ecs -p ph2d-render -p ph2d-script registry` | o registro foi **54→55** e os **DOIS** espelhos **55→56** — o contador é TRÊS e cada um roda só na suíte da própria crate |
| `cargo test -p ph2d-i18n` + um `tr` de `panel.vector.*` | o `lib.rs` foi **PARTIDO**; uma chave nova fundida no arquivo de onde a tabela saiu compila e **não resolve** |
| `cargo test -p ph2d-editor-core --test architecture_widget_loc_cap --test hr12_widgets_a11y` | quatro splits de LOC e duas entradas novas no `A11Y_OPT_OUT` |
| `cargo test -p ph2d-panel-authored` | o gate de staleness do gerado; se um merge mover o emissor, a resposta é a porta `regenerate`, **nunca** editar o gerado |

⚠️ **DUAS flakes ALHEIAS, medidas e exoneradas** — as duas de CARGA, e as duas em crates que
esta linha **não toca** (`git diff main` vazio nas duas):

- `ph2d-host-desktop::flip_smooth::…::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
  — falhou sob a suíte cheia com **`load average 40,21`** em 32 núcleos e passou **11/11
  isolada**. É gate de RAZÃO morto de fome pelo runner.
- `ph2d-timeline::the_cost_of_depth_is_linear_not_explosive` — já registada no `CLAUDE.md §5`
  como flake pré-existente da mesma família.

**A regra do repo:** *nenhum smoke desta máquina significa nada com o load acima de ~5.*
Re-rode sozinho antes de suspeitar de um merge.

## §8 — Ordem, e o que NÃO fazer

- ⚠️ **Nenhuma ordem interna é load-bearing** — os 47 commits podem ser rebaseados na ordem
  em que estão; não há par de commits cuja troca quebre uma premissa (ao contrário da jornada
  de física de 08/08, que tinha um).
- ⚠️ **O `PROJECT_SCHEMA` fica em 69 e o `project.rs` tem diff VAZIO** ⇒ esta linha está
  **fora de toda disputa de número** desta janela. Se um conflito aparecer ali, ele **não é
  desta linha**.
- ⚠️ **Nenhum ADR** ⇒ fora da disputa de número de ADR também.
- ⛔ **Não "canonize" os quatro literais cuja derivação foi corrigida** — os NÚMEROS estão
  certos e os comentários é que estavam errados. Trocar `SECTION_LABEL_TO_CONTROL_PX` pelo
  `Spacing::Xxs` que o comentário antigo nomeava **encolhe o gap de toda seção do app**.
- ⛔ **Não reverta a `PANEL_HEADER_CLOSE_RESERVE` para 40** — ela era 2 px curta na escala de
  **fábrica**, e o pin `the_reserve_is_the_pad_plus_the_icon_at_factory_scale` existe para
  fazer essa reversão custar duas edições.
