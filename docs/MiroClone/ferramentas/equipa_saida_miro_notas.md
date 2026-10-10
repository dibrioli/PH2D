# Leis do Miro para a W8 «Equipa assíncrona e saída» — help público (2026-10-09)

> API Zendesk (`help.miro.com/api/v2/help_center/en-us/articles.json?page=1..7`, **653** artigos varridos por
> título e por corpo; busca sem locale `…/articles/search.json?query=` para «comments», «resolve comment»,
> «comment pin», «board versions», «save version», «presentation mode», «hide frame», «transitions»,
> «laser pointer», «export board»; **25** lidos) + comunidade (**[comm]**; nenhum é resposta de funcionário
> salvo quando dito) + FigJam (help.figma.com, mesma API) como 2.ª referência.
> Figuras = imagens OFICIAIS dos artigos, medidas por scripts Python (PIL 12.3, componentes ligados por cor,
> buracos preenchidos) — scripts e downloads em `scratchpad/miro_w8/` (fora do repo).
> URLs: prefixo `https://help.miro.com/hc/en-us/articles/` —
> CM=`360017730873-Comments` · MN=`360017730893-Mentioning-people-on-the-board` ·
> BV=`360021668819-Board-history-versions` · BA=`360017730913-Board-history-activity` ·
> RC=`360019838260-Restoring-board-content` · BK=`360017572774-How-to-save-board-backup` ·
> DU=`360017730113-How-to-duplicate-a-board` · PM=`34307373858450-Presentation-mode` (novo, mar–abr/2026) ·
> IPM=`8512850754962-Interactive-Presentation-Mode` (legado) · FR=`360018261813-Frames` ·
> SL=`25542531289234-Miro-Slides` · RM=`7812052691602-Reduce-Motion` · EX=`360017572754-How-to-export-your-board` ·
> EI=`360020567820-Board-export-issues` · PR=`4408887050386-How-to-print-your-board` ·
> BG=`11371735242770-Board-background-color` · CI=`4413451973266-Copy-as-text-or-as-an-image` ·
> SC=`360017731033-Shortcuts-and-hotkeys` ·
> FJ-C=`help.figma.com/hc/en-us/articles/1500004290941` · F-MV=`…/360041547853` · FJ-E=`…/4407699832855` ·
> FJ-S=`…/5025214483351` · F-VH=`…/360038006754` ·
> c4328=`community.miro.com/ask-the-community-45/comments-lost-on-miro-board-4328` ·
> c13065=`…/ask-the-community-45/interactive-presentation-mode-s-smooth-slide-transition-animations-not-working-13065` ·
> c420=`community.miro.com/ideas/hide-frames-from-presenting-420`

## 1. Comentários (Comments)

| lei | citação | fonte |
|---|---|---|
| ferramenta na barra de criação; tecla **C** | "Click the Comment icon in the Creation toolbar on the left, or press the C hotkey." · SC: `comment · C` | CM, SC |
| prende-se a um PONTO do quadro OU a um objecto | "click anywhere on the board or on an object" | CM |
| preso = anda com o objecto; prende-se por POSIÇÃO (pousar em cima); arrastar muda o dono | "simply position the comment over the desired object. If you need to move a comment to a different object, click and drag it to the new location" | CM |
| também pelo menu de contexto do objecto | "Click on any object on the board to open its context menu and add a comment." | CM |
| publicar: botão azul ou Enter; Shift+Enter = nova linha; **sem formatação** | "click the blue Publish button or press Enter" · "Text formatting isn't supported in comments." | CM |
| rascunho: sair sem publicar guarda-o (toast «Draft saved. Open the thread to continue», figura 21016057341330) | "your comment will be saved as a draft" | CM |
| fio = comentário + respostas (campo «Leave a reply. Use @ to mention.» nas figuras) | "Jump into thread discussions" | CM |
| editar: clicar no texto → Save (figura: «Save · Cancel») | "click on the body of text, update as needed, and then click Save" | CM |
| apagar: `…` → Delete; **um de cada vez** | "Deleting multiple comments or all comments at once from the board isn't possible." | CM |
| cor do fio: 5 bolas no topo do fio aberto | "open the comment and select a color from the options at the top" | CM |
| menções `@`: notificam; `@board`, `@team`, `@Space`; a si próprio = lembrete | "type @board" · "tag yourself to assign tasks and set reminders" | MN, CM |
| @ em TEXTO de objecto (nota/forma/texto) não notifica | "Users won't receive feed notifications for text object mentions." | MN |
| estados da bolha: novo = **azul**, lido = **branco**, resolvido = **✓**; menção = **@**, nova azul / lida branca | "New comments are blue. Read comments are white. Resolved comments have a check." | CM |
| hover = pré-visualização (autor, «3 minutes ago», texto — GIF 21016057339666) | "hover over any comment to see a preview" | CM |
| resolver = toggle «Resolve» no canto sup. esq. do fio; **esconde-o do quadro** | "mark any comment thread as resolved to hide it from the board" | CM |
| ver resolvidos: painel → Show → «Show resolved» | "click the Show dropdown, and toggle on Show resolved" | CM |
| reabrir | **CALADO** (a figura de «Show resolved» mostra ✓ brancos e um ✓ azul no quadro; o toggle é o mesmo «Resolve») | — |
| painel: lista todos; clicar leva ao sítio | "Selecting any comment in the panel instantly takes you to its location on the board." | CM |
| filtro do painel (figura 21016057357842): **All comments · Mentions · Show resolved**; definições (figuras 21537454069394/…79506): **Show comments on board · Group comments · Mark all as read** | "toggle off Show comments on board" · "select Mark all as read" | CM |
| esconder todos: **Shift+C** | SC: "Show/hide comments on the board · Shift + C" | SC |
| `…` do fio (figuras): New comments! (Avatars & mention badges) · Mark as unread · Follow thread · **Pin** · Copy link · Delete | "select Pin" · "Mark as unread" | CM |
| Pin = o fio fica ABERTO no quadro; vários ao mesmo tempo | "By pinning several comments you can keep multiple threads open simultaneously" | CM |
| reações emoji a um comentário | "Click the emoji icon next to a comment to add a reaction." | CM |
| copiar/cortar/colar o objecto **não leva** o comentário | "Comments attached to objects can't be copied, cut, or pasted along with the objects." | CM |
| exportar **não leva** comentários | "comments can't be exported" · EX: "Are comments included in the exported document? No" | CM, EX |
| duplicar o QUADRO **leva** | "if you duplicate a board, any comments will be copied along with it" | CM |
| apagar o objecto apaga o comentário; restaurar o objecto devolve-o; o comentário apagado sozinho **não volta** | **[comm, moderador voluntário 2021]** "when I delete an object that has a Comment attached to it, the Comment is also deleted" · "the Comment is restored along with it" · RC (oficial): não restaura "comments deleted separately" | c4328, RC |
| Activity regista comentários | "all board objects, including comments, and uploaded files will be registered" | BA |
| apresentação: os participantes podem ter «Comments» se o apresentador o permitir | "Participant tools optionally provide … Comments" | PM |
| frame escondido NÃO esconde os seus comentários | "hiding a frame doesn't conceal the frame's comments and title" | FR |
| CALADO | prender a um frame vs ao conteúdo dele; o que «Group comments» agrupa (a figura mostra uma pílula com 3 avatares — interpretação); limite de texto; ordenação do painel; filtrar por pessoa (NÃO existe no filtro da figura); se a bolha escala com o zoom; reabrir; editar o comentário de outro | — |

**Medições** (scripts `comp.py`/`disc2.py`: componente da cor, buracos preenchidos; o Ø é a maior linha/coluna):

- **Forma da bolha:** círculo com uma **ponta no canto inferior esquerdo** (balão de fala; a ponta fica dentro do
  quadrado envolvente) e sombra por baixo. Lida = anel branco com o avatar dentro; nova = anel **azul**;
  resolvida = disco cheio (azul ou branco) com **✓**.
- `21016057338514` (1600×900, «Comments on a board»), notas de lado **184** (amarelo `#FDF7B9`, 184×186 com sombra):
  bolha resolvida azul **Ø 68** (área preenchida 3 692 ≈ círculo Ø68 3 632 + ponta), cor no centro
  `(57, 90, 250)` = `#395AFA`; resolvida branca **Ø 67**; lida com avatar **Ø 66**; nova com avatar e contador
  **Ø 68** de corpo, **anel azul ≈ 8–9** à volta de um avatar ≈ 51. ⇒ **bolha/nota = 0,37**.
  **Selo de contagem** («3»): círculo azul de borda branca **Ø ≈ 32 = 0,47 da bolha**, centro ≈ (+31, −22) do
  centro da bolha (no aro, a ~45° em cima à direita). Menção: pílula azul **114×74** (avatar · `@` · selo).
  Fio com vários autores: pílula branca **159×71** com 3 avatares sobrepostos.
- `21016043986322` (1600×900, «attached to a shape and a sticky note»), nota **233×234**: bolhas **Ø 48**
  ⇒ **bolha/nota = 0,21**. A bolha está DENTRO da nota, canto sup. dir. da bolha a (−30, +21) do canto sup.
  dir. da nota — onde se clicou, não um canto fixo (na outra figura: (−34, +85)).
  ⇒ **O rácio bolha/nota muda ×1,8 entre as duas figuras: a bolha NÃO escala com o mundo** (tamanho de
  interface; inferência — a ajuda cala).
- **Paleta de cor do fio** (bolas no topo; o ponto-âncora e a linha-guia que ligam o fio aberto ao objecto
  tomam a cor escolhida): `21016043972754`/`21016057353490` (1600×900) = **cinza `#C6C5D1` · verde `#6CC55B` ·
  vermelho `#DD4A4A` · azul `#54B4FF` · preto `#000000`**; `21017429401362` (JPEG antigo) = amarelo `#F9D054`
  em 1.º lugar · … · vermelho `#D35D59` · azul `#4467F7` · preto. ⇒ 5 cores; a 1.ª mudou entre versões.
- **Fio aberto** (mesmas figuras): cartão branco ~560×285 px: linha 1 = «Resolve» (toggle) · 5 cores · sino
  (Follow) · `…`; linha 2 = avatar, nome, data («Yesterday, 16:26»), texto com @ azul, + emoji; linha 3 =
  «Leave a reply. Use @ to mention.», emoji, enviar (➤). Ponto-âncora ligado ao cartão por uma linha recta.

## 2. Histórico do quadro: Versões

| lei | citação | fonte |
|---|---|---|
| versões AUTOMÁTICAS: de hora a hora se mudou, e no fim de cada sessão | "Boards are backed up every hour if a change has been made, as well as at the end of each collaborative session" | BV |
| guardadas **90 dias** | "Saved board history versions are stored for 90 days" | BV |
| restaurar = **cópia SEPARADA**; o original fica; título datado | "The original board is not changed. The restored version is created as a separate board and its title is by default dated" | BV |
| caminho: menu → Board → History → separador **Versions** → escolher → modal «Restore as a separate board» → Restore | "The Restore as a separate board modal opens." | BV |
| só planos pagos; dono/co-dono/editor da equipa com «copy» permitido | "the feature isn't supported in free teams" | BV |
| **«Save version» / versão NOMEADA** | **NÃO EXISTE no help**: 0 ocorrências de «save version», «named version», «version name» nos 653 artigos; a busca «save version» devolve BV, backup, export | — |
| o que fica fora de uma versão: | «Board background colors won't be reflected in your board history» | BG |
| Activity (o outro separador): log por utilizador; só a ÚLTIMA mudança de cada objecto; mover não regista; 90 dias; «Highlight changes» a rosa; restaurar objectos apagados no sítio | "if you modify one object several times, all the previous modifications will be erased" · "moving objects - you won't find any records" · "highlighted in pink" | BA |
| restaurar objecto apagado: sessão + 30 min, ou os últimos **1000** | "The last 1000 objects deleted from the board" | RC, BA |
| backup `.rtb`: ficheiro, ≤ **1 GB**, reimporta como quadro novo «Restored» | "The *.rtb file will be saved" · "limit of 1GB" · "The board title will include Restored" | BK |
| duplicar: nome «Copy of…», tecto **61** caracteres | "Board names have a maximum character limit of 61" | DU |
| CALADO | se as versões levam comentários, votos, cronómetro; pré-visualizar antes de restaurar; quantas versões no máximo; o nome/formato exacto da data | — |

FigJam/Figma (F-VH), 2.ª referência — o contrário do Miro em três pontos: checkpoint automático **a cada 30 min**;
**«Save to Version History»** (Ctrl+Alt+S) com Título + Descrição, "Figma clips titles longer than 25 characters",
"descriptions are less than 140 characters"; restaurar **no próprio ficheiro** ("non-destructive … Figma will add
two autosave checkpoints"); e "Figma will keep comments from all versions in the file, so restoring a version won't
remove any comments" — os comentários estão FORA das versões. Duplicar uma versão não leva comentários.

## 3. Apresentação (Presentation mode)

Duas versões convivem: PM (nova, "rolling out between March and April 2026") e IPM (legado).

| lei | citação | fonte |
|---|---|---|
| entrar: «Present» no canto sup. direito; ecrã inteiro; frames = diapositivos; **escondem-se as ferramentas e os TÍTULOS dos frames** | "Your view will change to full screen and the frames will appear as slides, with board tools and frame titles hidden." | FR |
| ordem = a do **painel Frames** (arrastar para reordenar; «Organize frames» ordena esquerda→direita, cima→baixo) | "set the order of frames by simply clicking and dragging the frames on the sidebar" · "from left to right and from up to down" | IPM, FR |
| frames aninhados **não** se apresentam | "Nested frames do not get presented." | FR |
| navegar: **setas do teclado** ou setas no canto inf. esq.; grelha de saltos («four-square Open slides panel») | "use your keyboard arrow keys, or in the bottom-left click the arrow buttons" | PM |
| sair: **Esc** sai do ecrã inteiro; **Stop** termina | "You can exit full-screen mode by pressing Escape" · "Clicking Stop … will exit presentation mode" | IPM |
| «Go to canvas» / «Go to slide» sem sair | "in the bottom-right click Go to canvas" | PM |
| contador «Frame 1/3» (legado) / «11/11» (novo) | figuras 21019593019282, 34307419286034 | — |
| **lightbox** (legado): «Dark background» — preto à volta do frame; desvanece ao afastar | "Create a black border around frames … If you pan away or zoom out, the lightbox effect fades away." | IPM |
| clique num frame = zoom até ele | "Simply click to zoom in on any frame." | IPM |
| transição animada entre frames (desligável: «Reduce motion» abrange "Presentation mode · Navigating between frames") | RM; **[comm]** "the presentation stopped animating from one frame to another" · moderador: com os dois desligados "you should be back to the animated-style transition between frames" | RM, c13065 |
| excluir um frame: ícone «Hide frame» (ou apagar o frame, ou pô-lo no fim do painel) | "use the Hide frame icon in the context menu for the slide(s) you want to exclude" | PM, IPM |
| ⚠️ **contestado**: utilizadores dizem que o frame escondido APARECE (capa cinzenta) | **[comm]** "I don't understand why hidden frames are still showing in presentation mode" (2022); 2023 idem | c420 |
| frame escondido = capa cinzenta com olho fechado e «Hidden for now» (GIF 22431681592594) | "they'll see a blank frame with a closed eye icon" | FR |
| notas do apresentador numa janela separada | "Presenter notes will appear in a separate window that's only visible to you." | PM |
| ferramentas no modo: Timer · Voting · Private mode · reações; ferramentas de participante (Sticky note, Shapes, Text, Pen, Stickers, Comments) | — | PM |
| ponteiro laser | **NÃO EXISTE** no help (0 ocorrências de «laser»; a busca devolve artigos alheios) | — |
| CALADO | duração/curva da transição; margem do frame; frame vazio; rácio | — |

**Medições:**
- `21019593022866` (GIF 1920×1080, 199 quadros, «Click to zoom with lightbox», legado): janela da app x 99–1825,
  y 113–965 (**1727×853**, 2,02:1); frame vermelho final x 215–1707, y 120–958 = **1493×839 (1,78 ≈ 16:9)**.
  O frame **ENCAIXA pela altura** (contain): margem **7 px em cima e em baixo = 0,8 % da altura**, frame =
  **98,4 %** da altura; os lados vazios (**116–117 px** cada) ficam **pretos** (lightbox). A barra do apresentador
  fica POR CIMA do frame, ao fundo.
  **Transição** (largura do frame por quadro, atrasos do GIF 3–4 cs): de 238 px (f98) a 99 % do final (f125) em
  **0,94 s**; 83 % ao fim de **0,50 s** ⇒ desaceleração (ease-out). (Gravação de ecrã: tempo real aproximado.)
- `34307419286034` (PNG 3804×1818, PM novo): barra preta em cima **76–78 px (4,2 %)**, em baixo **80 px (4,4 %)**;
  entre elas branco de borda a borda (3804×1658) — frame e fundo são ambos brancos, o limite do frame não se
  distingue ⇒ a margem do PM novo **não se mede** nesta figura. Em cima: «You are presenting», reações/timer,
  avatar, **Stop**, **Share**; em baixo à esq.: ‹ «11/11» › e a grelha; à dir.: 3 ícones (participant tools, notas, sair do ecrã inteiro).
- `21019593019282` (1375×559, legado): barra «← Frame 1/3 → · participant tools · reações · +pessoa ·
  Preferences · Stop»; Preferences = «Show cursors» · «Dark background».

FigJam (FJ-S): **não tem diapositivos**; tem **Spotlight** — "gather collaborators in a file to follow your view";
os outros têm "a few seconds to click Not now"; terminar = «Stop» no topo. Secções escondíveis.

## 4. Exportar

| lei | citação | fonte |
|---|---|---|
| formatos: **JPG, PDF, SVG, CSV**; **sem PNG nem PPT** | "Currently, you can save your boards as PDF/JPG/SVG/CSV files." · "Can I export my board in PPT/PNG format? No." | EX |
| menu (figura 21537454176914): Board → Export → Save as PDF · **Save as image** · Save board as template · Export to spreadsheet (CSV) · Download board backup · Embed · Save to Google Drive · Share as presentation (BETA) | — | EX |
| imagem: escolher a ÁREA, depois o tamanho **Small · Medium · Large · Vector** (Vector = PDF ou SVG) | "Select the area of the board that you'd like to save as an image." · "Small, Medium, Large, or Vector size. Vector quality is saved as a PDF or SVG file." | EX |
| um frame / selecção: menu `…` do objecto → «Export as image» / «Export as PDF» / «Export to CSV (excel)»; «Copy as image» = **Ctrl+Shift+C** → PNG no clipboard | "The image will be created in PNG format." | EX, FR, CI |
| PDF: **uma página por frame**, na ordem do painel; **Small file size** ou **Best quality (Vector)** | "Each frame will become a separate page in the saved PDF document." · "Select the file size: Small file size or Best quality." | EX |
| PDF sem frames não sai: criar um frame à volta | "Create at least one frame." | EX |
| **só sai o que está DENTRO de frames** (PDF de alta qualidade) | "Put everything you want to export in frames, as only widgets inside of frames are exported." | EI |
| conteúdo de frames escondidos **não** sai | "Content in hidden frames isn't exported." | EI |
| **títulos dos frames NÃO saem** | "the frame titles aren't included in the export" | EI, EX |
| **comentários NÃO saem**; **votos NÃO saem**; **cor de fundo do quadro NÃO sai** | "Are comments included …? No" · "Can I export voting results? No" · "Board background colors aren't included when exporting boards." | EX, BG |
| ligações clicáveis só no PDF vector | "Links are clickable on PDF files exported in Vector quality only" | EX |
| impressão: frame A4/Letter "automatically matches the standard paper size" | — | PR |
| Free: só pequeno, com **marca d'água** | "Free plan customers can only export low-resolution, watermarked PDFs." | PR |
| Slides: «Export image as» JPG/PDF/SVG; JPG "Small, (Default) Medium, or Large" | — | SL |
| CSV: texto (notas, tabelas, cartões, texto, formas), uma coluna: texto, descrição/URL do cartão, etiquetas | "put into a column in the following order: text, card description/URL, sticky note/card tags" | EX |
| CALADO | fundo transparente (0 ocorrências fora de import/upload); a grelha; ligações (setas) e carimbos no export; tamanho de página do PDF; margem; DPI | — |

**Medições** (diálogos «Export image as» nos GIFs; frames recortados e lidos):
- `21016057583890` (GIF 1280×720, 340 quadros, «Exporting a board as an image»): **Small JPG 2449×1632 ·
  Medium JPG 4898×3265 · Large JPG 8573×5715 · Vector PDF ∞**; ao mexer na área: 2387×1675 / 4774×3351 /
  8354×5864 e 2359×1695 / 4718×3391 / 8256×5934. A área é um rectângulo com **cantoneiras em L** arrastáveis
  sobre o quadro escurecido (cinza), e os px actualizam ao vivo.
- `21016928435346` (GIF 1920×1080, 470 quadros, frame): **Small 1908×523 · Medium 7634×2095 · Large 13359×3667 · Vector PDF ∞**.
- ⇒ **a lei é a ÁREA em píxeis, não um factor de escala:** Medium = **16,0 MP** (4898·3265 = 15,99 M; 4718·3391 = 16,00 M;
  7634·2095 = 15,99 M), Large = **49,0 MP** (8573·5715 = 48,99 M; 13359·3667 = 48,99 M) = 7000²; Large/Medium = **1,750**
  de lado nas três. Small = **4,0 MP** (2449·1632; 2359·1695) = 2000², mas **1,0 MP** (1908·523) na figura do frame
  (versão/plano diferente — a ajuda cala). O rácio do recorte mantém-se.
- `21016941396370` (GIF, plano Free): «Export PDF in» **Small file size** · **Best quality** (cinzento) + «Upgrade
  to export this board in best quality.»

FigJam (FJ-E): File → Export as PNG/JPG/PDF; **SVG não** ("SVG exports from FigJam aren't currently supported");
fundo **Transparent** (só PNG) · **Solid** · **Grid** (PNG/JPG); «Quality» para JPG/PDF; CSV de notas com
«Content» + «Created by». Comentários FigJam (FJ-C/F-MV): ponto OU região arrastada; preso ao objecto se se
comenta em cima; editar/apagar só o autor ("You can resolve comments that other collaborators have left, but you
can't delete them"); Shift+C esconde; ordenar Date/Unread, filtro «Only your threads», «Show resolved»; até 5
imagens por comentário; "up to 100 comments per hour"; cortar-colar o objecto **desliga** o comentário; duplicar o
ficheiro **não** leva comentários (o oposto do Miro).
