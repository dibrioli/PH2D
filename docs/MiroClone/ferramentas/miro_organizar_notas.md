# Leis do Miro para a W5 «Organizar» — help público (2026-10-07)

> Lidas pela API Zendesk (`help.miro.com/api/v2/help_center/en-us/articles.json?page=1..7`, 653 artigos,
> 30 lidos) + docs de developer + comunidade (marcada **[comm]**, não oficial). Referência de PRODUTO;
> o número, onde o Excalidraw concorda, vem do oráculo (`excalidraw_oracle/`, secção W5 do README).
> URLs: prefixo `https://help.miro.com/hc/en-us/articles/` —
> FR=`360018261813-Frames` · ST=`360017730973-Structuring-board-content` · OBJ=`360017730953-Working-with-objects` ·
> SC=`360017731033-Shortcuts-and-hotkeys` · LK=`4408887253778-Locking-content-on-the-board` ·
> SR=`360018109534-Searching-text-on-the-board` · SN=`360017572054-Sticky-notes` · CD=`360020911193-Cards` ·
> LN=`360017572354-Internal-and-external-linking` · MS=`360017731053-Using-Miro-with-a-mouse-trackpad-or-touchscreen` ·
> UI=`20967864443410-Miro-s-new-simplified-user-interface` · KN=`11997028019858-Keyboard-navigation-while-working-on-boards` ·
> LY=`20258488000786-Layers` · CL=`4409706795410-Clustering` · IP=`8512850754962-Interactive-Presentation-Mode` ·
> EX=`360020567820-Board-export-issues` · PR=`4408887050386-How-to-print-your-board` ·
> DEV-F=`https://developers.miro.com/docs/websdk-reference-frame` · DEV-C=`https://developers.miro.com/docs/children-inside-parent-items`

## 1. Frames
| lei | citação | fonte |
|---|---|---|
| filho = dentro dos limites (SDK) | "the item to add must: … Be located inside the frame" / falha se "The child item is located outside the bounds of the parent frame" | DEV-F |
| encaixar pelo arrasto | "Select the objects and slightly move them - this should attach them to the frame." | FR |
| sinal visual ao encaixar **[comm]** | "the moment the frame highlights blue, the object is 'attached'" | comunidade |
| os filhos movem-se com o frame | "place them on a single frame and move the frame." | OBJ |
| coordenadas do filho relativas ao frame | "moving a parent item on the board along with its children" | DEV-C |
| redimensionar não pode deixar filhos fora (SDK) | "may fail if one or more children fall outside the parent item after resizing it" | DEV-C |
| apagar o frame apaga o conteúdo **[comm]**; o help admite apagar só o frame | "it deletes everything inside" · "delete the corresponding frame (you can do that without deleting the content inside it)" | comm 26115 · IP |
| apagar pelo título | "click on the frame title and press backspace or delete" | FR |
| frames aninhados NÃO se movem juntos **[comm]** (staff 16/08/2024, ideia «under review») | "child frames don't move when the parent frame moves" | comm ideas/3902 |
| frames sempre ATRÁS, fora da ordem | "This feature isn't available for frames. Frames are always placed behind board objects." | OBJ |
| grupo não atravessa frames | "You can't group … objects located in different frames." | ST |
| mover pela borda ou pelo título | "Click and drag the frame's border or title." | FR |
| cor de nascença branca; cor e tamanho na barra | "The default color of a frame is white." | FR |
| renomear | "Double-click the frame heading. Start typing a new name" | FR |
| criar à volta da selecção | "Select the objects, click three dots (...) … and choose Create frame." | FR |
| ferramenta `F` | "press the F hotkey and click anywhere … click and drag to create frames" | FR |
| painel de Frames: navegar e reordenar | "Quickly jump from one frame to another and reorder your frames … drag and drop" | FR |
| arrumar a ordem | "arranges frames according to the order in which they're placed on the board—from left to right and from up to down" | FR |
| travar frame | "Lock frames to prevent them from moving accidentally" | FR |
| recorte no quadro | **CALADO** — só a exportação recorta: "only widgets inside of frames are exported" | EX |
| arrastar um filho para fora solta-o? | **CALADO** | — |

## 2. Agrupar
| lei | citação | fonte |
|---|---|---|
| grupo = um objecto | "using Ctrl + G … they will behave as a single object (although you can still edit each object within the group)" | ST |
| mover um membro: duplo-clique | "To move a single object within a group, double-click it and drag." | ST, OBJ |
| grupos não se alinham | "You can't align grouped objects" | ST |
| grupo é contentor do teclado | "Container objects include frames and grouped objects." | KN |
| grupos aninhados; o que um clique selecciona | **CALADO** | — |

## 3. Travar
| lei | citação | fonte |
|---|---|---|
| o que trava | "Prevent objects from being accidentally moved, edited, or deleted by locking them" | LK |
| atalho | "Ctrl + Shift + L" (o mesmo destrava) | LK |
| destravar todos | "right-click a blank area of the board and select Unlock all" | LK |
| travados ficam fora do laço | "Make sure none of the objects you want to select is locked" | OBJ |
| seleccionar/editar texto de um travado; o sinal no quadro | **CALADO** | — |

## 4. Alinhar e distribuir
"aligned horizontally, vertically, and on left/right sides in relation to each other … (such as sticky notes, pictures, frames, etc.)" (ST); «Auto layout»: arrastar a pega para distribuir (ST). Lista exacta, referência e atalhos: **CALADO** (SC não tem teclas).

## 5. Ordem
"Bring to front … Send to back … Bring forward: Move an object in front of the object directly in front of it. Send backward …" (OBJ). Teclas: **PgUp** frente · **PgDn** trás · **Shift+PgUp** um acima · **Shift+PgDn** um abaixo (SC). Frames excluídos. Camadas (LY) são outra pilha — fora da W5.

## 6. Minimapa
"navigation controls in the bottom-right … the minimap (which can also be shown or hidden via the M hotkey)" (MS). Mostra a extensão de TODO o conteúdo **[comm 6353]**. Clique/arrasto e esconder sozinho: **CALADO**.

## 7. Busca (`Ctrl+F`)
"Found objects will be highlighted on the board, while other content will be dimmed. The results are also listed below the search field." · "Click on any search result to navigate directly to that object" · filtro: "search specifically for stickies with a particular tag … Change All content to another category" · sem substituir (SR). Teclas de seguinte: **CALADO**.

## 8. Etiquetas
"You can add up to 8 tags to each sticky." (SN) · cor por etiqueta (CD) · "To find all the stickies that contain a specific tag, use board search." (SN). Como se desenham na nota: **CALADO**.

## 9. Links entre elementos
"an object on the board can be linked to only one website or Miro object" · "Choose Link to … click another object on the board" · "To go to the linked URL - just click the link icon." · alvo apagado: "Sorry, the linked item has been deleted from this board" (LN). Atalho **Ctrl+Alt+K** (SC). Zoom ao seguir: **CALADO**.

## 10. Zoom e nível de detalhe
"zoom to fit Alt + 1 · zoom to selected item Alt + 2 · zoom to 100% Ctrl + 0" (SC). Apresentação esconde os títulos (FR). O que a letra pequena e os títulos fazem de longe: **CALADO**. Queixas: «unreadable … in size» (ideas/466), texto que some/borra no zoom (27971, 6413, 16739), conselho de ler a ≥ 20 % (947).

## 11. Atalhos relevantes (SC)
frame **F** · minimapa **M** · agrupar **Ctrl+G** · desagrupar **Ctrl+Shift+G** · travar **Ctrl+Shift+L** · frente **PgUp** · trás **PgDn** · um acima **Shift+PgUp** · um abaixo **Shift+PgDn** · link **Ctrl+Alt+K** · busca **Ctrl+F** · enquadrar tudo **Alt+1** · à selecção **Alt+2** · 100 % **Ctrl+0** · entrar/sair de contentor **Ctrl+Shift+↓/↑**.
