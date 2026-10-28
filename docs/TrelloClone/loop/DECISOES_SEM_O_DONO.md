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
