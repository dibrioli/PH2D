# 37 — **GRUPOS: o verbo existia e era inalcançável** (2026-08-30)

> Pedido do Enio: *"criar a feature de Grupos para vector e para todos os objetos que poderiam ser
> combinados um grupo. Na hierarquia deve aparecer como um objeto só e no menu do botão direito da
> hierarquia 2 novas opções: agrupar e desagrupar. Assim o grupo poderia ser usado como pattern."*

## §1 — ⭐⭐⭐ A medição que reenquadrou o pedido

**Grupos já existiam, inteiros.** Antes de escrever uma linha, duas varreduras responderam:

| o que o pedido supõe que falta | estado medido |
|---|---|
| um modelo de grupo | ✅ existe — [`vec_entities_group.rs`](../../shells/desktop/src/vec_entities_group.rs), 102 LOC, gateado |
| **para todos os objetos** | ✅ existe — *"o grupo é uma entidade comum, então ele aceita sprite e path vetorial no mesmo saco"* |
| o gesto | ✅ existe — `Ctrl+G` / `Ctrl+Shift+G`, ligado e vivo |
| selecção que trata o grupo como um objeto | ✅ existe — *"um grupo entra e sai da seleção INTEIRO"* |
| gizmo de canvas para um objeto sem geometria | ✅ existe — [`group_gizmo_view.rs`](../../shells/desktop/src/group_gizmo_view.rs) |
| árvore recolhível na Hierarquia | ✅ existe — chevron, `is_hierarchy_collapsed` |
| persistência | ✅ existe |
| **um menu, botão ou rótulo que diga a palavra `Group`** | ⛔ **NENHUM em todo o app** |

⇒ *o que faltava não era a feature: era o ALCANCE dela.* É a lei deste repo aplicada à UI — **uma
ferramenta que nenhum passo escrito chama pelo nome morre** —, e é por isso que o dono do produto
pediu para criar o que já tinha.

⚠️ **E o atalho tinha três cercas invisíveis**, que explicam porque ele nunca foi encontrado:
1. só responde com a ferramenta **Vector** em mãos (duas sprites escolhidas com o Move: `Ctrl+G` morto);
2. lê **apenas** a selecção de caminhos da caneta — uma sprite escolhida na Hierarquia é invisível
   para ele, embora o `group_entities` a aceite;
3. falha para o **`stderr`**: o artista recebe silêncio.

O verbo da Hierarquia entra por outra porta e **não herda nenhuma das três**.

## §2 — O que shipa

- **Duas linhas no menu de contexto da Hierarquia**, `Group` e `Ungroup`, **juntas** e imediatamente
  antes do bloco que já junta a selecção (`Merge Sprites` → `Merge to Layers` → `Pack into Sheet`).
  Lidos em sequência, os quatro respondem *"quão junto?"* em ordem crescente de dano — o grupo é a
  forma **suave**, e o merge destrói os originais. ⚠️ O par fica junto porque *um verbo cujo inverso
  não se vê não se usa*.
- **`EditorAction::HierGroup` / `HierUngroup`**, duas acções e não uma com interruptor: *agrupar e
  desagrupar não são o mesmo gesto com um sinal trocado*, e uma única acção obrigaria a shell a
  adivinhar o sentido a partir do estado da selecção.
- **[`hier_group.rs`](../../shells/desktop/src/hier_group.rs)** — a **decisão**, pura e gateada: sobre
  quem o verbo age, e o que ele diz. A mutação vai pelas portas que já existem.
- **Feedback**: cinco desfechos, cinco frases distintas, nenhuma muda. As recusas são `warning` (o
  app está correcto; o que falhou foi a pré-condição) e dizem **o que fazer**.
- **O grupo nasce SELECCIONADO e RECOLHIDO** — é a metade *"na hierarquia deve aparecer como um
  objeto só"*.

## §3 — A lei do SUJEITO, e porque ela não é nova

O menu é por LINHA; agrupar é sobre um CONJUNTO. O *Merge Sprites* já tinha respondido:

- a linha clicada **está** na selecção ⇒ o sujeito é a **selecção**;
- está **fora** e há selecção múltipla ⇒ o verbo **não age, ORIENTA** (*"right-click on one of the
  selected objects"*) — agir sobre a união traria para o grupo um objecto que o artista não
  escolheu; agir só sobre a linha faria o verbo falhar por ter um sujeito só;
- há **um ou nenhum** seleccionado ⇒ a **linha clicada**, sozinha.

⛔ Inventar aqui uma terceira lei para a mesma pergunta seria a divergência que este repo paga
sempre. A frase de orientação é **a mesma** do Merge, com o sujeito trocado: duas redacções para a
mesma situação ensinariam que são situações diferentes.

## §4 — ⚠️ O nome conta MEMBROS, não sujeitos

`Group 2` sobre uma coisa só seria mentira no primeiro sítio que o artista lê. Dois caminhos do
mesmo grupo são **um** membro — a normalização para ancestrais de topo que o `group_entities` já
fazia internamente saiu para [`top_members`](../../shells/desktop/src/vec_entities_group.rs), porque
passou a ter dois leitores: o verbo e quem lhe dá o nome.

## §5 — ⚠️ Recolher é DIFERIDO, e a razão é uma armadilha

O grupo nasce **durante** o dreno; a ponte `node_for(bits)` da Hierarquia só o conhece depois da
publicação seguinte. Recolher no mesmo quadro seria recolher um id que ainda não existe — **em
silêncio**. Guardam-se os bits (`App::pending_group_collapse`) e tenta-se a cada quadro até a linha
aparecer, **uma vez** (`take`): sem isso, o artista abria o grupo e o quadro seguinte fechava-o
outra vez, e *um controlo que se desfaz sozinho lê-se como avaria*.

## §6 — ⛔⛔ Um gate que já tinha DOIS buracos, e a cura é a derivação

O `simple_row_context_menu_items_are_populate_registered` guardava a lista de linhas do menu
**escrita à mão**, e o comentário dele contava a história certa — *"Use as Brush Shape shipped dead
because it was hit-painted but OMITTED here"* (Enio, 2026-06-25). Depois disso o menu ganhou
`Merge to Layers` e `Export Image…`, e **nenhum dos dois entrou na lista**: estão vivos por sorte,
não por gate.

⇒ a lista passa a ser **derivada do próprio menu** (`menu_rows(HierarchyRow)`). O que o artista vê é
exactamente o que o gate exige, e uma linha nova entra sozinha. *Uma lista escrita à mão ao lado de
uma tabela é duas respostas à mesma pergunta, e a que envelhece é sempre a escrita à mão.*

## §7 — Os gates, e a mutação que SOBREVIVEU

- `the_group_pair_raises_its_own_action_with_the_clicked_row` — o roteamento.
- ⛔⛔ **`the_menu_offers_group_and_ungroup_by_name` existe porque uma mutação sobreviveu:** apagar a
  linha `Ungroup` da tabela deixava o gate acima **verde**, com o verbo perfeitamente ligado a um
  item que ninguém vê. *Um gate que injecta o evento nunca mede se o artista tem por onde o
  produzir.* Ele afirma presença, rótulo e **adjacência** do par.
- `hier_group_tests.rs` (6, puros) — a lei do sujeito nos quatro casos, e que os cinco desfechos
  dizem coisas **distintas** e nenhuma vazia.
- **5 mutações, 5 mortas**: o sujeito sempre-a-selecção · o clique fora que deixa de orientar · a
  linha não registada (shipa morta sob o dedo) · a linha fora do menu · o par na ordem trocada.

## §8 — ✅ **O GRUPO É A ARTE DE UMA ESTAMPA** (report do Enio no smoke, mesmo dia)

> *"Selecionar o grupo como shape de pattern não funcionou."*

### §8.1 — ⭐ A saída que não mexe no schema

`PatternSource::Shape(VecPathId)` endereça **geometria**, e um grupo não tem `VecPathId` — ele nasce
`(Transform, Name, RootOrder)` e mais nada. ⛔ As saídas óbvias eram todas caras: uma variante nova
(schema), um id de entidade (que o undo respawna com bits novos), um nome (heurística).

⭐⭐ A que shipa **não toca no formato**: o id continua a ser o de um CAMINHO, e o que muda é a
**resolução** — ele passa a nomear o **OBJECTO** a que aquele caminho pertence, pela porta
`object_selection_for`, que é a mesma que decide o que um clique no canvas apanha (*"um grupo entra
e sai da selecção INTEIRO"*). Nenhuma variante, nenhum degrau de migração, nenhum id novo gravado.

⚠️ Isto **muda o desenho** de um documento que aponte para um caminho que hoje esteja agrupado — e é
a mudança pretendida. Hoje é inconsequente (não há projectos gravados).

### §8.2 — As três costuras

- **O assado** — `motion_object_bake::bake_rgba_many`: a caixa é a **união** das dos membros e o
  desenho é um laço sobre eles **na mesma cena de rascunho**. É a receita que o `fx_live::cook_batch`
  já usava para assar um lote num render só. O `bake_rgba` de um id passa a delegar aqui — *uma
  porta, nunca uma reimplementação*, que é a lei que o doc dele já declarava.
- **O memo** — a chave passa de `Option<VecPath>` para `Vec<VecPath>`: **editar qualquer membro**
  re-assa. ⚠️ Com o caminho clicado sozinho, mexer no IRMÃO deixava a tela parada — o defeito exacto
  que o `FxKey` da crate irmã documenta.
- **O ciclo** — a recusa passou de **igualdade** (`id == host`) para **pertença**
  (`membros.contains(&host)`). ⚠️ Com um grupo, o anfitrião pode ser um MEMBRO da arte: assá-la
  exigiria desenhá-lo, desenhá-lo exigiria o ladrilho. *O sintoma não seria um desenho errado —
  seria o app a parar.*

### §8.3 — ⚠️ A pose do grupo, e a metade que ela obriga

> *"O gizmo do objeto pai deveria nascer na posição entre os filhos, mas nasceu no zero do mundo."*

Um grupo não desenha nada, então **a pose dele é o gizmo dele** — e `Transform::default()` punha-a
em `(0,0)`, muitas vezes fora do ecrã, com **girar** o grupo a acontecer em torno do nada.

⇒ ele nasce na **média das poses** dos membros (a âncora que o artista já arrasta em cada objecto;
⛔ não o centro das caixas de desenho, que exigiria resolver geometria e **mudaria** quando um filho
fosse editado sem se ter movido).

⚠️⚠️ **E isso obriga a DUAS compensações, nenhuma das quais existia** — enquanto o grupo nascia na
origem as duas eram somar zero: agrupar subtrai o centro a cada membro (senão agrupar **move** o
desenho), e desagrupar devolve-o (senão dissolver move-o de volta ao contrário). *Uma cura num
sentido que não é aplicada no inverso é meia cura*, e o inverso aqui é o gesto com que o artista
confere o primeiro.

### §8.4 — ⭐⭐⭐ Um defeito que o gate apanhou antes do smoke

`ungroup_entities` exigia `t != e`: só dissolvia o grupo de quem estivesse **dentro** dele. Isso
bastava enquanto o único chamador era o `Ctrl+Shift+G`, que passa CAMINHOS — o verbo novo passa a
**selecção**, e depois de agrupar a selecção **é o grupo** ⇒ *Ungroup* logo a seguir a *Group* era
um no-op que dizia *"nada na selecção está dentro de um grupo"*. **O gesto mais natural era o único
que não funcionava.**

⚠️ E a cura não podia ser apagar a condição: um grupo **aninhado** tem por ancestral de topo o de
FORA, e subir cegamente dissolveria o pai. ⇒ *quem já é um grupo responde por si; quem não é, sobe.*

### §8.5 — Os gates

- `an_art_that_is_a_group_resolves_to_all_of_its_members_in_z_order`
- `a_shape_inside_the_group_it_wears_is_refused` (o ciclo, nas duas metades: a resolução e o aviso)
- `editing_any_member_of_the_group_changes_what_the_memo_sees` (a promessa de que o padrão é vivo)
- `a_group_bakes_into_one_tile_with_both_members_in_it` (`#[ignore]`, GPU) — ⚠️ a régua é a **tinta
  nos dois extremos**, e não a largura: uma caixa larga com um membro só passaria numa medida de
  largura, e o artista veria metade do grupo e um vazio do tamanho da outra metade.
- os cinco da POSE: nasce entre os filhos · agrupar não move · desagrupar não move · aninhar não
  move · dissolver o aninhado dissolve o **clicado**.
- **8 mutações, 8 mortas.**

### §8.6 — ⏳ O que fica

⚠️ O composto herda a cerca já declarada do assador (`PatternTiles::new()`): um membro que ele
próprio tenha estampa assa **chapado**. É o mesmo limite de hoje para uma arte de um caminho só, mas
`N` membros tornam-no `N` vezes mais provável de ser visto.
