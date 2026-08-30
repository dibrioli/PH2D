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

## §8 — ⏳ ABERTO, e é a razão pela qual o Enio pediu isto

**Um grupo ainda NÃO pode ser a arte de um padrão.** A medição diz porquê e quanto custa:

- ⛔ **O modelo é o bloqueio**: `PatternSource::Shape(VecPathId)` endereça **geometria**, e um grupo
  **não tem `VecPathId`** — ele nasce `(Transform, Name, RootOrder)` e mais nada.
- ⭐ **A saída barata existe e não mexe no schema**: manter o id a ser um `VecPathId` e passar a
  **resolvê-lo como OBJECTO** (o grupo a que o caminho pertence, se houver) — que é a lei de selecção
  que o app já tem. ⚠️ Muda a aparência de documentos gravados que apontem para um caminho agrupado;
  hoje isso é inconsequente (não há projectos gravados), e a mudança é a **pretendida**.
- O **assado** é uma dobra de bbox + um laço — a receita já existe em `fx_live::cook_batch`.
- A **guarda de ciclo** tem de passar de *"o anfitrião não é a arte"* para *"o anfitrião não é
  MEMBRO do objecto da arte"*.
- ⚠️ E o composto herda a cerca já declarada do assador (`PatternTiles::new()`): um membro que ele
  próprio tenha estampa assa **chapado** — o mesmo limite de hoje, mas `N` membros tornam-no `N`
  vezes mais provável de ser visto.
