# Decisões de PRODUTO tomadas sem o dono (revê no smoke final)

| Onda | Decisão | Porquê | Como desfazer |
|---|---|---|---|
| W2b | Título das listas em MAIÚSCULAS, 15 px SemiBold (maquete: 14 px, caixa normal) | o dono pediu «maior e mais destacado, talvez maiúsculas» | `LIST_TITLE_PX` em `ph2d-app-kanban/src/metrics.rs` e `src/list_title.rs` |
| W2b | O buraco deixado pela lista levantada usa o mesmo translúcido de «Add another list» | coerência com a única outra «vaga» de lista no quadro | token usado em `paint_list.rs` |
| W2b | Sombra da lista em voo = sombra do cartão em voo, com o raio da lista | a mesma lei de voo do cartão (pedido do dono) | `paint_list.rs` |
| W3 | Seletor de datas: por omissão escolhe a ENTREGA; ligar «Start date» faz o próximo clique escolher o início | é o que o Trello faz | `widget/date_picker` no editor-core |
| W3 | Clicar fora de uma caixa de data/hora confirma o que se escreveu | Trello confirma ao sair do campo | `date_picker` |
| W3 | Descrição em modo leitura: clicar num link abre-o, numa caixa de checklist marca-a, noutro sítio entra em edição | Trello | `widget/rich_editor` |
| W3 | Botão de bloco já activo volta o bloco a parágrafo | Trello/Notion | `rich_editor` |
| W3 | Desfazer NUNCA apaga a atividade; o desfazer é mais uma entrada | Trello | `History::undo_logged` no `ph2d-kanban-model` |
| W3 | Converter item de checklist em cartão leva a pessoa e a data do item; o cartão nasce logo abaixo do de origem | Trello | `convert_item_to_card` |
| W3 | Mudar etiquetas, ordem, campos e acrescentar item de checklist não entra na atividade | Trello não os regista | `activity_for` |
| W3 | Prazo vermelho só quando passou a hora; amarelo quando falta < 24 h; verde concluído | cores do Trello | `ph2d-kanban-dates::due_tone` |
| W3 | Semana começa à segunda por omissão (inglês, como a interface); a função por região existe | a interface do PH2D é inglesa | `week_start_for_region` |
| W3 | Markdown: títulos 4–6 viram 3; imagens/tabelas/HTML na descrição ficam texto literal | o editor do verso só tem títulos 1–3 (Trello idem) | `ph2d-kanban-md` |
| W4 | Kanban no Quadro redimensiona-se livremente (não proporcional) | é uma moldura com rolagem própria | `ph2d-board-edit` |
| W4 | Entra-se no Kanban do Quadro com um 2.º clique depois de seleccionado (só no zoom alto); Esc/clique fora sai | como o vídeo/embed do Quadro: 1.º clique selecciona o elemento | `board_video/kanban` |
| W4 | Kanban novo no Quadro: 960×540 (três listas de largura) | cabe à vista sem zoom | `board_bar` inserir |
| W3 | Cinco reacções fixas nos comentários (👍 ❤️ 🎉 😄 👀) | o seletor de emojis completo do Trello pede um widget de emojis | `back/` reacções |
| W3 | «Show details» desligado mostra comentários + acontecimentos principais (criado/movido/convertido) | Trello | `back/` atividade |
| W3 | «Copy» do verso copia logo para baixo do cartão com as checklists (o diálogo de opções vem na W6) | atalho útil até ao diálogo do Trello | `back/` ações |
| W3 | Azul de acções do verso = `kanban-blue-bold` (o acento do tema PH2D é magenta) | a maquete é azul-Trello | tokens |
| W3 | Frente: sem distintivo de lembrete; capa de imagem metade = faixa fixa de 128 px; título sobre capa inteira de imagem com véu medido a 4,5:1 | Trello / maquete | `front.rs`, `paint_card.rs` |
| W3 | Menção com espaço grava-se `[@Ana Silva](mention:Ana%20Silva)`; sem espaço, `@Ana` | ida-e-volta sem perder o nome e legível em qualquer leitor de markdown | `ph2d-kanban-md` |
| W6 | Palavra do filtro procura no título, campos, nomes de etiquetas e de membros (não na descrição nem comentários); a pesquisa global `/` procura em tudo | ajuda do Trello + maquete | `ph2d-kanban-filter::matches` |
| W6 | Categorias do filtro: «qualquer correspondência» = basta uma; «exacta» = todas | ajuda do Trello | `Filter.mode` |
| W6 | Envelhecimento: níveis aos 7, 14 e 28 dias sem actividade | a Atlassian não publica os limiares; iguais às janelas de actividade do filtro | `age_level` |
| W6 | Mover/copiar para outro Kanban casa etiquetas e campos por nome+cor/tipo, criando os que faltam; a história fica no Kanban de origem | Trello | `move_card`/`copy_card` |
| W6 | Arquivar/apagar o original remove todos os espelhos; desfazer devolve-os | Trello | `remove_card` |
| W10 | Pessoa importada = `username` do Trello (as @menções casam sem tradução) | Trello | `ph2d-kanban-import` |
| W10 | Cores do Trello: `x_light` = subtil, `x` = normal, `x_dark` = forte | paleta de 30 do plano | `trello.rs` |
| W10 | Export CSV/markdown com cabeçalhos em inglês, como o do Trello; arquivados sob `## Archived`; checkbox do markdown = entrega concluída | formato do Trello | `export_*` |
| W10 | Células do CSV NÃO são protegidas contra injeção de fórmulas de folha de cálculo (iguais ao Trello) | paridade com o Trello; REVER: proteger (prefixo `'`) é mais seguro | `export_csv` |
| W10 | Anexos que o Trello não guarda (ficheiro do projecto, cartão, Kanban) saem como links `ph2d://` e voltam iguais | ida-e-volta sem perda | `export_json.rs` |
| W3 | A 1.ª imagem anexada a um cartão sem capa vira capa (no mesmo desfazer) | Trello | `back/` anexos |
| W3 | Link sem esquema abre como `https://`; `file:`/`javascript:` recusados | Trello + segurança | `back/` links |
| W8 | Comando de calendário perdido com o app fechado corre UMA vez ao abrir; comando de prazo corre uma vez por cartão cuja hora passou; comando novo não corre o passado | política do Butler (não duplica, não perde) | `ph2d-kanban-auto::run_scheduled` |
| W8 | Sugestões de automação a partir de 3 repetições (seguimento até 5 min depois de mover); a regra sugerida fica «por mim» | Butler sugere a partir do que se repete | `suggest` |
| W8 | Comentários do Butler saem em nome dele, datas ISO; cartões-modelo não disparam regras; listas guardadas pelo id (renomear não parte a regra) | Trello/Butler | `ph2d-kanban-auto` |
| W5 | Nomes em listas partilhadas (votos, membros, quem segue, etiquetas) ficam na ordem de entrada; tirar e voltar a pôr manda para o fim | ordem determinística entre colegas | `ph2d-kanban-sync` |
| W5 | Apagar contra editar ao mesmo tempo: fica a lápide E a edição (como no Quadro) | uma só lei de fusão | `ph2d_board_sync::wins` |
| W4 | «Place on board» usa o Quadro activo, senão o último activo, senão o primeiro, senão cria um; depois mostra-o com a referência seleccionada | o caminho mais curto ao que o artista vê | `board_kanban_link.rs` |
| W4 | Kanban em vários Quadros: «Show on board» abre a lista «Show on which board?» | plano §3.2 | idem |
| W4 | Zoom baixo: nomes e contagens nunca abaixo de 11 px no ecrã; lista recolhida tem a altura do título | maquete do canvas | `board-render` LOD |
| W3 | Sem tecto de tamanho por anexo (o tecto real é a RAM: ~3× ao gravar); até 8 MB lê na hora, acima num fio; anexos ficam DENTRO do projecto | medido (MEDICOES §W3 E); anexo por referência fica para o dono decidir | `back/attach` |
| W3 | Abrir link: http/https/mailto (o vídeo do Quadro, que só aceitava https, passa à mesma lista) | uma só política | `ph2d_app_host::open` |
| W5 | Reacções: o emoji aparece pela ordem da 1.ª reacção; reacção sem ninguém some | Trello | `ph2d-kanban-sync` |
| W4 | Zoom interno do Kanban no Quadro: sem gesto (Ctrl+roda é sempre o zoom do Quadro); a roda dentro do Kanban activo rola a lista sob o rato | o Trello não tem zoom; o idioma do Quadro manda | `canvas.rs` |
| W9 | Uma aba «Planner» com três modos (Calendar · Planner · My cards) | a navegação Inbox · Planner · Quadros do Trello | `ph2d-app-planner` |
| W9 | Inbox, blocos do Planner e «lidas» são pessoais e NÃO viajam; eventos viajam e numa sessão os calendários juntam-se | Trello: a Inbox é só tua | `table.rs` |
| W9 | Arrastar uma ocorrência de evento recorrente move a série inteira | o mais simples e previsível; editar só uma ocorrência fica por fazer | `ph2d-app-planner` |
| W9 | Desfazer do Planner é do Planner (não se desfaz na aba do Kanban) | cada aba tem a sua fila (ADR-0195 §5) | — |
| W7 | Painel: anel só «por lista» e «por entrega» (por etiqueta/membro só barras — um cartão conta em vários); o excesso dobra-se em «Other (n)» | um anel tem de somar 100 % | `ph2d-kanban-views` |
| W7 | «+» no calendário pergunta a lista (1.ª por omissão); o cartão novo vence ao meio-dia | Trello | idem |
| W7 | Linha do tempo agrupada por lista: largar a barra noutra faixa muda a lista do cartão | Trello Timeline | idem |
| W8 | API Luau em snake_case (`add_list`, `add_card`, `press_button`); eventos `card_created`, `card_moved`… | o idioma da API `ph2d` existente | `ph2d-kanban-script` |
| W8 | Etiqueta desconhecida num script é erro que lista as existentes (o script não a cria) | erro visível > criação silenciosa | idem |
| W8 | Um script falhado não deixa NADA (corre numa cópia); uma execução = um passo de desfazer | a regra do `ph2d.set` | idem |
| W10 | CSV protegido contra injecção de fórmulas (prefixo `'`, OWASP) — revoga a linha «NÃO são protegidas» | segurança > paridade com o Trello | `export_csv` (`56b1bb9ce6`) |
