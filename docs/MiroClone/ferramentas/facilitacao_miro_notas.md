# Leis do Miro para a W7 «Facilitação» — help público (2026-10-09)

> API Zendesk (`help.miro.com/api/v2/help_center/en-us/articles.json?page=1..7`, 653 artigos varridos, 15 lidos) + Web SDK + comunidade (**[comm]**; **[staff]** =
> resposta de funcionário «Mironeer» na comunidade). Figuras = medidas nas imagens oficiais (em `scratchpad/miro_facil/`).
> URLs: prefixo `https://help.miro.com/hc/en-us/articles/` —
> TM=`360017730933-Timer` · VO=`360017572274-Voting` · RE=`360021249320-Reactions` · SE=`360016026559-Stickers-and-Emojis` ·
> CL=`4409706795410-Clustering` · SN=`360017572054-Sticky-notes` · AI=`28781881506834-Miro-AI-with-Sticky-notes` ·
> IW=`20509314090386-Intelligent-Widgets` · CT=`4402396215698-Collaboration-tools` · AM=`360013358479-Attention-management` ·
> PM=`9794413310482-Private-mode` · UI=`20967864443410-Miro-s-new-simplified-user-interface` · SC=`360017731033-Shortcuts-and-hotkeys` ·
> DEV-T=`developers.miro.com/docs/websdk-reference-timer` · DEV-G=`…/websdk-reference-tag` ·
> cNNNN=`community.miro.com/…-NNNN` (4273 timer 2021 · 9438 cluster 2022)

## 1. Cronómetro (Timer)
| lei | citação | fonte |
|---|---|---|
| vive na barra de colaboração (canto sup. direito), menu «Facilitation tools and Reactions» | "located on the collaboration toolbar, under the Facilitation tools and Reactions icon in the top right corner" | TM |
| definir: escrever ou `+`/`−`; ▶ azul arranca | "set the time period by either typing in the time or using the + and - options, and click the blue play button" | TM |
| a correr: parar, pausar, **+1m** e **+5m** | "Click the +1m or +5m buttons to add time to the countdown." | TM |
| parar = repor; pede confirmação (figura: «Stop timer / Cancel») | "To stop the timer and reset the time if needed, click the stop button." | TM, c4273 GIF |
| passo do `±` acima de 10 min é 5 min; sem segundos; não se reduz um timer a correr **[comm]** | "Time over 10 minutes can only be adjusted by 5 minutes up or down" · "you cannot set seconds" · "cannot be reduced" | c16256 |
| máximo **99:59** **[comm]** | "won't go higher than 99:59 minutes" · moderador: "more than 99 minutes" | c4242 |
| global: começa e acaba para todos; minimizável, o ícone mostra o resto | "starts and ends for everyone on the board. You can minimize the timer and the toolbar icon will always show how much time is left." | TM |
| no zero: aviso visual + campainha a todos | "all the collaborators [are] notified visually and with a chime" | TM |
| a campainha NÃO se desliga **[comm]**; a música opcional sim | "The sound signal at the end of the timer cannot be turned off manually" · "You can also mute the music" | c16256, TM |
| posição fixa | "Can I move the timer…? No" · apresentação: "pop up in the top left corner" | TM |
| estado (SDK): `restDuration`, `totalDuration` em ms, `STARTED/STOPPED/PAUSED`; eventos `start/update/finish` | "Starts timer for specified amount of milliseconds" · "Prolongs timer by provided amount of milliseconds" | DEV-T |
| durações pré-definidas · o que pisca no zero · tempo negativo · atalho · se fica gravado no quadro | **CALADO** (SC não tem tecla; o SDK não tem estado «terminado» nem negativo) | — |

## 2. Votação (Voting)
| lei | citação | fonte |
|---|---|---|
| alvos: área de votação + filtro por tipo (figura: Sticky notes · Shapes · Images · Text …, com contagens) | "setting the voting area. Quickly filter objects to vote on and select sub-categories like sticky notes" | VO, vote_02 |
| votos por pessoa: `±` ou escrever, **máx. 99**; sempre anónima | "note that voting is anonymous, the maximum number of votes is 99" | VO |
| duração `±`/escrever, em minutos, horas ou dias | "choose between minutes, hours, and days" | VO |
| vários votos no mesmo objecto por omissão; toggle «One vote per object» | "If every participant should cast only one vote per object, enable the corresponding toggle" | VO |
| votar = `+` junto ao objecto; tirar = `−` | "clicking the plus icons next to them" · "change your votes … by clicking the plus and minus icons" | VO |
| painel do votante: «N votes left», relógio, «X of Y voted», «I'm done», «End for all», «+1 min» | figura (vote_03); "add minutes by clicking the corresponding buttons" | VO |
| `+` = círculo cinzento no canto inf. direito da nota | figura vote_03 | VO |
| fim: top-3 a todos + «See all results» (lista com barra por objecto) | "all voting participants can see the top 3 results" | VO, vote_05 |
| sessões guardadas por nome/data; toggle «Show results on the board» = selo azul com o número no canto inf. direito | "keep voting results visible on the board using the toggle on the Results tab" | VO, vote_04/05 |
| renomear/apagar sessão; não exporta; não vai na cópia do quadro | "There is no option to export" · "If I duplicate my board… No" | VO |
| «Add to board» prepara sem arrancar | "instead of clicking Start, click on Add to board" | VO |
| default de votos/duração | **CALADO** (figura: 3 votos, 2 min) | — |
| mín. de votos · atalhos · resultados como objecto (frame/nota) | **CALADO** | — |

## 3. Carimbos / reações
**Não há ferramenta «Stamp» no Miro**: a palavra não aparece no help (0 em 653) nem no SC. Os vizinhos:
| lei | citação | fonte |
|---|---|---|
| Reactions: 6 emojis efémeros com o avatar (figura: 👍 ❤️ 🔥 🤯 👋 🎉) | "select from six emojis" · "appear on the board together with your avatar" | RE, stamp_01 |
| emoji NA nota: vai para a base da nota, várias por nota, conta votos | "place it on the bottom of the sticky note. it's possible to add votes to the existing emoji and add several reactions to one sticky note" | SN |
| tirar o seu: clicar nele | "To remove an emoji you've added, simply click it" | SN |
| ver quem: hover **[comm]**; sem limite **[staff]** | "hover over the emoji to see who voted" · "There is not limit … as long as there is space" | c5392, c6706 |
| Stickers & Emojis: objectos soltos, redimensionáveis e rodáveis | "click on it to change its size and rotate it" | SE |
| Dot voting (widget): o ponto prende-se à nota/cartão e anda com ela | "When a Dot is dragged onto a Sticky note or card, it will attach to it. If the object is moved, the Dot moves with it." | IW |
| desenhos não se prendem à nota (só agrupando) | "Can pen drawings… automatically attached…? No" | SN |
| lista de carimbos · tecla · tamanho do emoji-na-nota | **CALADO** | — |

## 4. Agrupar notas (Cluster)
| lei | citação | fonte |
|---|---|---|
| entrada: seleccionar → «Cluster objects» → Color · Tag · Author · Keywords · Create a cluster (+ Sentiment) | "Select Cluster by Color, Tag, Author, Keywords or Create a cluster" | CL |
| resultado = **containers** (caixa com borda + título em etiqueta preta acima) que se movem juntos | "they're grouped into containers. All objects grouped in a container can easily be moved together" | CL, cluster_01/02 |
| por cor: sem título | "you need to manually add a title to each group" | CL |
| por autor: título = nome | "the name of each cluster is the name of the respective author" | CL |
| por tag: título automático; nota com N tags é **duplicada**; sem tag → «no tag» | "it will be duplicated in several clusters" · "grouped in one cluster under a “no tag” title" | CL |
| keywords (IA): título = palavras-chave com `#`; resto → «#uncategorized»; multilingue | "suggest keywords in the titles" · "#uncategorized cluster" · "different languages" | CL |
| sentimento (IA): 3 containers Positive/Neutral/Negative, renomeáveis | "three containers: Positive, Neutral, and Negative" | CL |
| container: renomear (duplo-clique no título), mover pelo título, redimensionar pelas bordas, tirar/pôr notas | "click and drag the title" · "drag its borders" · "moving individual objects in and out" | CL |
| grelha medida (n→colunas): 1→1 · 2→2 · 3→2 · 4→2 · 5→3 (= ⌈√n⌉); anúncio 2022: 7→4 | figuras cluster_01/02; c9438 GIF | — |
| ordem dos clusters: «Q1 · no tag · Q2» (não alfabética, «no tag» no meio) | figura cluster_01 | — |
| folgas, ordem das notas, onde cai o resultado, desfazer, limite de notas, palavra-chave (caixa/plural/stop words) | **CALADO** | — |
| «Sort by» de notas | **CALADO** — não existe no help (só listas de quadros) | — |
| etiquetas: ≤ 8 por nota; título ≤ 120 chars, único, sensível a caixa; 12 cores, `red` por omissão | "up to 8 tags" · "Maximum length: 120 characters" · "Must be unique" | SN, DEV-G |

## 5. Ferramentas de facilitação — ecrã único × multi-utilizador
Menu «Facilitation tools»: Reactions · Timer · Voting · Private mode · Note (UI, figura timer_01). **Multi-utilizador por
natureza:** «Bring everyone to me» / Follow (AM), Private mode (esconde as notas dos outros; ao desligar, contagem de 5 s
— "A 5-second countdown will appear", PM), quem pode iniciar (CT). **Fazem sentido num só ecrã:** Timer (relógio local
+ campainha), Voting (um votante = o dono; selo de contagem persiste), emoji-na-nota, Cluster.
