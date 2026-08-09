# HANDOFF DE INTEGRAÇÃO — `line/Vector`, o painel autorado fica VIVO (2026-08-09)

> **15 commits · 68 arquivos · +5.762/−778.**
> **Todos os smokes aprovados pelo Enio** — os oito passos da cena, incluindo a família de
> LISTA, o dropdown e a seleção de aba.
> Supersede nada: é a continuação da jornada de UI/UX que integrou em 08/08.

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
| Registro do `ph2d-ecs` | **intocado** — `VecWidgetIcon` já existia |
| ADR | **nenhum** ⇒ fora de toda disputa de número |
| `Cargo.toml` | **1**, e é aresta interna (`ph2d-panel-authored`) |
| Dep externa nova | **nenhuma** (`Cargo.lock` sem `+name`) |
| `WidgetKind` | 12 → **20** (variantes apendadas; o `code()` é estável) |
| `ph2d-i18n` | `lib.rs` **partido** — as 186 chaves `panel.vector.*` para o irmão `vector.rs` (701 → 520 LOC) |

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

## §7 — Gate de fechamento

`scripts/nextest-impacted.sh` **8515/8515 verdes** · `cargo clippy --workspace
--all-targets` **limpo** · `cargo fmt --all` aplicado · golden do painel **em dia**.

⚠️ **Uma flake ALHEIA, medida e exonerada:** `ph2d-flip::flip_smooth::…::the_fit_rebuilds_the_
neighbourhood_not_the_whole_stroke` falhou duas vezes sob a suíte cheia e passou isolada e na
terceira corrida cheia. É gate de RAZÃO morto de fome pelo runner, e a linha toca **zero
arquivos** daquela crate (`git diff main -- crates/ph2d-flip` vazio). Re-rode sozinho antes
de suspeitar de um merge.
