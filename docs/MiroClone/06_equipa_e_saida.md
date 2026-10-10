# 06 — Equipa assíncrona e saída: comentários, versões, apresentação, exportar e `.excalidraw` (W8)

> **Pedido do dono (plano §3 W8, ordem de 09/10 «smoke OK. Prompt para próxima janela»):** comentários em
> fio presos a um elemento ou a um ponto, com autor e «resolvido», gravados no ficheiro; versões nomeadas
> dentro do ficheiro, restauradas como CÓPIA; apresentação pelos frames na ordem do painel; exportar
> PNG/SVG/PDF do quadro, de um frame ou da selecção; importar e exportar `.excalidraw`. ⛔ Fora: tudo o que
> precisa de várias pessoas AO MESMO TEMPO (cursores, presença, menções que notificam) — Etapa 2.
> **O dono vê:** um quadro com comentários e uma versão guardada, apresentado frame a frame, exportado, e
> um ficheiro do Excalidraw aberto.
>
> Briefing: [`handoffs/HANDOFF_CONTINUACAO_W8_2026-10-09.md`](handoffs/HANDOFF_CONTINUACAO_W8_2026-10-09.md).
> Fontes: a ajuda do Miro pela API Zendesk ([`ferramentas/equipa_saida_miro_notas.md`](ferramentas/equipa_saida_miro_notas.md):
> 653 artigos varridos, 25 lidos, 9 imagens oficiais medidas ao píxel), o FigJam/Figma pela ajuda dele, o
> oráculo **Excalidraw 0.18.1** corrido sobre entradas nossas ([`ferramentas/excalidraw_oracle/`](ferramentas/excalidraw_oracle/README.md)
> §«W8»), e os rasterizadores/validadores de SVG e PDF que o sistema já tem (§1.4).

## §0 — Veredito

- **Comentários** são DOCUMENTO (o fio vai no ficheiro, o Miro duplica-os com o quadro) mas **não** passam
  pelo desfazer do quadro nem pelas cópias de elementos: o Miro não os leva ao copiar um objecto, não os
  exporta, e um comentário é a conversa SOBRE o quadro, não o quadro. Prendem-se ao objecto sob o clique
  (andam com ele, somem com ele e voltam com o desfazer do apagar — sem código: o alvo é uma lápide) ou a um
  ponto do quadro. A bolha tem tamanho de INTERFACE (medido: 0,37 e 0,21 do lado da nota em duas figuras ⇒
  não escala com o mundo). O «autor» num ecrã só é o **nome que o artista escreve**, lembrado POR MÁQUINA
  (`~/.ph2d/board_author.txt`, o idioma das `prefs`: abrir o quadro de um colega não muda quem eu sou); o
  «novo/lido» do Miro é por PESSOA ⇒ cada fio guarda quem já o leu — é isto que torna o ficheiro uma caixa de
  correio assíncrona: o colega abre o projecto e vê a azul o que ainda não leu.
- **Versões**: o Miro NÃO tem versões com nome (0 ocorrências em 653 artigos) — tem versões automáticas de
  hora a hora na nuvem, e **restaurar cria um quadro separado** («The original board is not changed»). O
  Figma tem versões com nome mas restaura NO ficheiro. O dono pediu as duas metades certas: **nome** (o do
  Figma: é o que um ficheiro local precisa — não há nuvem a fotografar de hora a hora) e **restaurar como
  cópia** (o do Miro: a versão nunca destrói o presente). Guarda-se o DOCUMENTO do quadro (elementos, votos,
  carimbos, rascunho) comprimido; não os comentários (o Figma: *«Figma will keep comments from all
  versions»* — a conversa fica fora das versões), nem a câmara, nem as outras versões.
- **Apresentação** é SESSÃO (nada no ficheiro além de «este frame não se apresenta»): os frames de topo na
  ordem do painel, um de cada vez, a ocupar a área com a margem medida (0,8 %), o resto a preto, sem
  ferramentas, títulos, comentários nem selecção; setas, `Esc`; a transição medida no GIF oficial.
- **Exportar** sai de UMA lei de desenho: o `paint` do quadro grava as operações que emite (`VectorScene`
  com gravação) e três escritores leem a MESMA gravação — **PNG** pela placa (o desenho do ecrã, ao píxel,
  em ladrilhos quando passa o tecto de textura do dispositivo), **SVG** e **PDF** em vector (letras como
  contornos exactos + o texto real por baixo para buscar e copiar). Medido no Miro: o tamanho da imagem é uma
  ÁREA em píxeis (4 · 16 · 49 MP), não um factor; o PDF é uma página por frame.
- **`.excalidraw`** de ida e volta: o esquema aprendido pelas SAÍDAS do oráculo (117 sondas); o que o
  Excalidraw não tem (notas, as formas do fluxograma, etiquetas, votos, mapa mental) viaja em `customData.ph2d`
  — que o `restore` dele preserva — e desenha-se lá com o equivalente mais próximo. ⇒ um quadro nosso vai ao
  Excalidraw e volta IGUAL; um quadro do Excalidraw entra com o que temos.

## §1 — Como os apps fazem (fontes)

### §1.1 — Miro (o produto do dono), pela ajuda

Detalhe e citações em [`ferramentas/equipa_saida_miro_notas.md`](ferramentas/equipa_saida_miro_notas.md).
Prefixo `https://help.miro.com/hc/en-us/articles/`: CO=`360017730873` (Comments) · SH=`360017731033`
(Shortcuts) · BH=`360021668819` (Board history: versions) · PR=`34307373858450` (Presentation, 2026) ·
PL=`8512850754962` (Presentation, legado) · EX=`360017572754` (Export) · EI=`360020567820` (Export issues).

- **Comentários (CO, SH):** tecla `C`; `Shift+C` mostra/esconde todos; *«click anywhere on the board or on an
  object»*, e o comentário anda com o objecto; `Enter` publica, `Shift+Enter` muda de linha, *«Text
  formatting isn't supported»*; fio = comentário + respostas; editar e apagar; *«New comments are blue. Read
  comments are white. Resolved comments have a check»*; resolver *«to hide it from the board»*; o painel
  filtra «All · Mentions · Show resolved» e clicar leva ao sítio; copiar o objecto NÃO leva o comentário;
  *«comments can't be exported»*; duplicar o quadro leva-os.
- **Versões (BH):** automáticas *«every hour if a change has been made»*, 90 dias; *«The restored version is
  created as a separate board and its title is by default dated»*; *«Board background colors won't be
  reflected»*. Sem «Save version» nem nome.
- **Apresentação (PR, PL):** *«full screen and the frames will appear as slides, with board tools and frame
  titles hidden»*; ordem do painel Frames; frames aninhados não se apresentam; setas do teclado ou no canto
  inferior esquerdo, `Esc` sai; contador «1/3»; «Hide frame» tira-o da apresentação; legado: «Dark
  background» (preto à volta do frame); *Reduce motion* desliga a transição.
- **Exportar (EX, EI):** PDF · imagem (JPG) · SVG (o «Vector») · CSV; a imagem pede a ÁREA e depois o tamanho
  Small/Medium/Large/Vector; o PDF: *«Each frame will become a separate page»*, na ordem do painel; *«only
  widgets inside of frames are exported»* (no PDF); NÃO saem títulos de frame, comentários, votos, a cor de
  fundo; ligações clicáveis no PDF vectorial.
- **Calado:** reabrir um resolvido; a ordem do painel de comentários; o limite de texto; margem e DPI do PDF;
  fundo transparente; se as setas e carimbos saem no export; a duração exacta da transição.

### §1.2 — As imagens oficiais medidas ao píxel

Régua: Python (PIL) por componentes ligados por cor, no scratchpad; números transcritos aqui.

- **Bolha de comentário** (`21016057338514`, 1600×900, notas de lado 184): círculo Ø **66–68 px = 0,37** da
  nota, com uma ponta no canto inferior ESQUERDO (balão) e sombra; azul de «novo» `#395AFA`; o selo da
  contagem («3») Ø ≈ 32 = **0,47 da bolha**, no aro em cima à direita. Noutra figura (`21016043986322`,
  nota 233) Ø **48 = 0,21** ⇒ a bolha NÃO escala com o mundo (tamanho de interface). Posição = onde se
  clicou (dois casos, nenhum no canto fixo).
- **Apresentação** (GIF `21019593022866`, 1920×1080, legado): o frame final encaixa pela ALTURA com margem de
  **7 px de 853 = 0,8 %** em cima e em baixo; os lados vazios a preto; a transição chega a **83 % aos 0,50 s**
  e a **99 % aos 0,94 s**, a desacelerar. Ajuste de uma saída `1 − (1 − t/T)^k` aos dois pontos: **T = 1,33 s,
  k = 3,76** (P5).
- **Exportar imagem** (GIF `21016057583890`, três áreas; GIF `21016928435346`, um frame): Small **4,0 MP**
  (2449×1632), Medium **16,0 MP** (4898×3265), Large **49,0 MP** (8573×5715); nas três leituras o lado do
  Large é 1,750× o do Medium; no do frame, 13 359×3 667 = 49 MP ⇒ **o tamanho é uma ÁREA fixa em píxeis**,
  o rácio é o da área escolhida.

### §1.3 — O Excalidraw, pelo ORÁCULO (nunca pelo fonte)

[`ferramentas/excalidraw_oracle/`](ferramentas/excalidraw_oracle/README.md) §«W8», `corre_esquema.mjs`;
saídas `saidas/w8_*` com `meta.json`, duas corridas 12/12 iguais byte a byte.

- **Ficheiro:** `{type:"excalidraw", version:2, source, elements, appState, files}`; o `appState` gravado é
  `{gridSize, gridStep, gridModeEnabled, viewBackgroundColor}`; MIME `application/vnd.excalidraw+json`.
- **O `restore` preenche** (`strokeColor "#1e1e1e"`, `backgroundColor "transparent"`, `fillStyle "solid"`,
  `strokeWidth 2`, `roughness 1`, `opacity 100`, `roundness null`, `groupIds []`, `frameId null`, `locked
  false`…), **corrige** (caixa negativa, pontos que não começam em (0,0), referências a ids que não existem,
  ids repetidos, `javascript:` nos links) e **NÃO valida** (`opacity 150`, `roughness 7`, `fontFamily 99`, cores
  inválidas passam) ⇒ o nosso leitor valida ele mesmo. **Rebenta o ficheiro inteiro:** texto sem `text`,
  linha/seta/caneta sem `points`, `index` inválido.
- **Por tipo** (a tabela completa no README do oráculo): rectangle/ellipse/diamond; text (`containerId`,
  `textAlign`, `verticalAlign`, `lineHeight`, `fontFamily` 1 Virgil · 2 Helvetica · 3 Cascadia · 5 Excalifont
  · 6 Nunito · 7 Lilita · 8 Comic Shanns · 9 Liberation); line/arrow (`points` relativos a `x,y` com o 1.º em
  (0,0), `startBinding`/`endBinding` `{elementId, focus, gap}`, `fixedPoint` no cotovelo, pontas `arrow ·
  bar · dot · circle · circle_outline · triangle · triangle_outline · diamond · diamond_outline`); freedraw
  (`points`, `pressures`, `simulatePressure`); frame (`name`); image (`fileId` → `files`); embeddable.
- **`index`:** quem manda é a ORDEM do array (os que faltam ou violam a ordem são reescritos) ⇒ escrevemos
  sem `index` e por ordem de z.
- **`customData`:** preservado tal e qual pelo `restore` e pelo `serializeAsJSON` ⇒ a porta da ida e volta.
- **Exportar dele:** PNG/SVG/JPEG (zero «pdf» nos `.d.ts`); `exportPadding` (por omissão 10), a escala por
  `getDimensions` (o `exportScale` é IGNORADO pelo `exportToBlob` — o README antigo dizia escala 2 e era 1×);
  «embed scene»: no PNG um `tEXt` `application/vnd.excalidraw+json` com `{"version":"1","encoding":"bstring",
  "compressed":true,"encoded":<zlib>}`, no SVG o mesmo invólucro em base64 dentro de `<metadata>`.

### §1.4 — Oráculos de SVG e PDF que o sistema tem (triagem por ARTEFACTO)

| ferramenta | versão | licença | papel |
|---|---|---|---|
| `resvg`/`usvg` | 0.48.1 | Apache-2.0/MIT | rasterizar o NOSSO SVG e comparar com o nosso PNG |
| `rsvg-convert` (librsvg) | 2.63.2 | LGPL-2.1+ (só correr) | 2.º rasterizador do SVG |
| `pdftoppm`/`pdftotext`/`pdfinfo` (poppler) | 26.08.0 | GPL (só correr) | rasterizar o nosso PDF; o texto extraído (o `ActualText`); páginas e tamanhos |
| `qpdf --check` | 12.4.2 | Apache-2.0 | a estrutura do PDF é válida |
| `xmllint` | 2.15.4 | MIT | o SVG é XML bem formado |
| `gs`, `inkscape`, `magick` | — | AGPL / GPL (só correr) | 3.º e 4.º pareceres, quando dois discordarem |

Crates de PDF (sem nenhuma no `Cargo.lock`): **`pdf-writer` 0.15** (MIT/Apache, deps `bitflags itoa memchr
ryu`) — só serializa, a geometria é nossa. ⛔ `krilla` 0.8.2 recusado (§3): pede `skrifa ^0.42` (nós temos a
0.44 ⇒ duas cópias) e um 2.º moldador (`rustybuzz`).

## §2 — As leis do PH2D (lei → fonte → número)

### Comentários (documento do quadro; fora do desfazer e das cópias de elementos)

| lei | fonte | número |
|---|---|---|
| **K1** `C` arma a ferramenta Comentário (e o botão da barra); um clique abre o fio novo ali | CO, SH | — |
| **K2** preso ao objecto de CIMA sob o clique (o mesmo pick da selecção), no referencial da caixa dele antes de rodar, em fracção `[u,v]` ⇒ anda, roda e escala com ele; no vazio, preso ao ponto do mundo | CO «anda com o objecto» | — |
| **K3** objecto apagado ⇒ o fio some; desfazer o apagar ⇒ volta (o alvo é lápide; nada a mudar) | comunidade (moderador, 2021) | — |
| **K4** fio = mensagens (autor, texto simples, instante); `Enter` publica, `Shift+Enter` muda de linha; vazio não publica; só o autor edita e apaga a SUA mensagem; apagar a 1.ª apaga o fio | CO; FigJam (só o autor) | — |
| **K5** resolver esconde o fio do quadro (com «Show resolved» volta, com um visto); reabrir é o mesmo botão | CO | — |
| **K6** novo = azul (não li), lido = branco, resolvido = visto; abrir o fio marca-o lido POR MIM (`read_by`); o que eu escrevi nasce lido | CO; §1.2 `#395AFA` | — |
| **K7** a bolha tem tamanho de INTERFACE (Ø = a altura de um botão da barra), a ponta no canto inferior esquerdo sobre o ponto; o selo da contagem (respostas + 1, se > 1) Ø 0,47 da bolha no aro superior direito | §1.2 | 0,47 |
| **K8** `Shift+C` mostra/esconde todos; o painel lista os fios abertos (e os resolvidos com «Show resolved»), do mais novo ao mais velho; clicar enquadra o alvo e abre o fio | CO, SH | — |
| **K9** não saem no export, não entram nas versões, não se apresentam, não vão com uma cópia de objecto; duplicar o QUADRO leva-os | CO; Figma | — |
| **K10** o autor é o nome escrito, lembrado por máquina (`~/.ph2d/board_author.txt`); a 1.ª vez, o nome da conta do sistema | §0 | — |

### Versões nomeadas (documento do quadro)

| lei | fonte | número |
|---|---|---|
| **R1** «Save version» pede um nome (nasce «Version N», N = quantas já há + 1) e guarda o DOCUMENTO e o rascunho do quadro, comprimidos (postcard + deflate 1) | dono; §4.1 | 100 mil elementos: 14,5 ms, 2,1 MB |
| **R2** a lista, da mais nova à mais velha, com o nome e há quanto tempo; renomear e apagar | Figma | — |
| **R3** restaurar = um quadro NOVO à direita, «<quadro> — <versão>»; o original não muda | BH | — |
| **R4** sem comentários, sem câmara, sem as outras versões; duplicar o quadro leva as versões (é o quadro) | Figma; §0 | — |
| **R5** nenhum tecto de versões (o recurso é o disco: 2 MB por versão de um quadro de 100 mil) | §4.1 | — |

### Apresentação (sessão; «não apresentar» é documento)

| lei | fonte | número |
|---|---|---|
| **P1** diapositivos = os frames de TOPO na ordem do painel de Frames, menos os marcados «Hide from presentation» (o olho na linha do painel) | PR, PL | — |
| **P2** começa no frame seleccionado (ou no que o contém), senão no 1.º; sem frames visíveis a entrada avisa e não abre | PR | — |
| **P3** o frame enquadrado na área com margem **0,8 %** do lado limitante; fora dele preto; sem barra, painéis, títulos, comentários, selecção, guias | §1.2 | 0,008 |
| **P4** `→ ↓ PageDown Espaço` seguinte · `← ↑ PageUp` anterior · `Home`/`End` · `Esc` sai; as setas e o contador «n/N» no canto inferior esquerdo | PR, PL | — |
| **P5** a câmara viaja `1 − (1 − t/T)^k` com **T = 1,33 s, k = 3,76** (83 % aos 0,5 s, 99 % aos 0,94 s); *reduced motion* ⇒ salta | §1.2 | — |
| **P6** sair devolve a câmara e a ferramenta de antes | — | — |

### Exportar (uma gravação, três escritores)

| lei | fonte | número |
|---|---|---|
| **E1** PNG · SVG · PDF; o JPG do Miro não (a letra em JPG suja-se e o PNG é sem perda) | EX; escolha | — |
| **E2** a área: o quadro inteiro (a caixa dos vivos + 10 de margem, o `exportPadding` do Excalidraw), a selecção, ou cada frame seleccionado | EX; §1.3 | 10 |
| **E3** PNG por ÁREA em píxeis: Small 4 · Medium 16 · Large 49 MP, o rácio o da área; na placa, em ladrilhos do tecto de textura DO DISPOSITIVO | §1.2; §0.0 | 4 · 16 · 49 MP |
| **E4** PDF = uma página por frame de topo (ordem do painel, menos os escondidos; a selecção restringe aos seus frames); sem frames, uma página com a área; 1 unidade do mundo = 1 pt | EX | — |
| **E5** vector: letras como CONTORNOS exactos (a mesma fonte e os mesmos glifos que o ecrã), e no PDF o texto real em `ActualText` (busca e cópia); os links do W5 viram ligações (`<a>`, anotação) | EX «links no PDF vectorial» | — |
| **E6** sai o documento: formas, setas, notas (com sombra), caneta, frames, etiquetas, carimbos; NÃO saem títulos de frame, votos, comentários, selecção, laser | EX | — |
| **E7** a tinta é a do tema CLARO (papel branco), fundo branco; «Transparent background» no PNG e no SVG | EX «cor de fundo não sai»; §1.3 | — |

### `.excalidraw` (ida e volta)

| lei | fonte | número |
|---|---|---|
| **X1** exportar: rectângulo/elipse/losango → os deles; as outras 15 formas → `line` fechada com o nosso contorno (preenchida) + o texto num `text` do mesmo grupo; nota → `rectangle` cheio sem traço com o texto preso; seta → `arrow` com `startBinding`/`endBinding`, curva = `roundness {type:2}`, cotovelo = `elbowed` com `fixedPoint`; caneta → `freedraw` com `pressures`; frame → `frame` (`frameId` = o frame de topo); grupos → `groupIds`; travar → `locked`; `link` = `null` (o do Excalidraw é um URL; a nossa ligação aponta para OUTRO ELEMENTO e viaja em `customData.ph2d`, X2); `seed` = a nossa (o rascunho treme igual lá) | §1.3 | — |
| **X2** o que o Excalidraw não tem vai em `customData.ph2d` (o tipo, as etiquetas, os votos, os carimbos, o mapa, o rascunho, o id e a ligação exacta) e o leitor prefere-o ⇒ nosso → Excalidraw → nosso = IGUAL (a menos dos ids). O que se EDITOU lá ganha ao guardado: o leitor compara o elemento com o que escreveu (`customData.ph2d.wrote` nas setas) e devolve a cor, o tracejado, a opacidade, o tamanho do texto, o redimensionar (polígono e caneta escalam) e os pontos de uma seta RECTA | §1.3 | — |
| **X3** importar: um quadro novo com o nome do ficheiro; texto solto → forma sem preenchimento nem traço com o texto; imagem, embeddable, iframe e tipos desconhecidos SALTAM com um aviso que os conta; valores fora de faixa → os de nascença dele; lê também o `.png`/`.svg` com a cena embebida | §1.3 | — |
| **X4** o array sai pela ordem de z e sem `index` (a ordem manda); `version`/`versionNonce` = os nossos, `updated` = 1 (ficheiro determinístico) | §1.3 | — |

## §3 — ⛔ Recusas MEDIDAS

| proposta | estado |
|---|---|
| menções `@` que notificam, reacções, «seguir o fio», o fio fixado aberto | ⛔ notificar é rede (Etapa 2); num ficheiro, o «novo» por pessoa (K6) é a notificação |
| as cinco cores do fio | ⛔ a paleta mudou entre versões do próprio Miro (a 1.ª era amarela `#F9D054`, hoje cinzenta `#C6C5D1`): não é lei estável, e as etiquetas (W5) já dão cor a um assunto |
| versões AUTOMÁTICAS de hora a hora (Miro) | ⛔ são uma cópia de segurança da nuvem; num ficheiro local cresceriam o `.ph2dproj` em silêncio (2 MB por versão de um quadro grande, §4.1) — a versão é um gesto do artista |
| restaurar a versão NO quadro (Figma) | ⛔ o idioma do Miro (e o pedido do dono): a versão nunca destrói o presente |
| a versão em memória (o `BoardDoc` clonado) | ⛔ medido (§4.1): 13 ms por versão de 100 mil — e o documento inteiro de cada versão a ocupar RAM; os bytes comprimidos são 2,1 MB e 14,5 ms |
| `krilla` para o PDF | ⛔ medido no índice: `skrifa ^0.42` contra a 0.44 do lock (duas cópias) e um 2.º moldador |
| a letra no PDF/SVG como `<text>`/fonte embebida | ⛔ a Inter da casa é VARIÁVEL (o negrito é o eixo `wght`, o itálico é sintético, W3): embeber pede instanciar a fonte; os contornos são a imagem exacta do ecrã e o `ActualText` dá a busca e a cópia |
| o PNG por factor de escala (`2×`) | ⛔ o Miro mede por ÁREA (§1.2): um quadro comprido a `2×` passava do tecto da placa, um pequeno saía minúsculo |
| exportar com o recorte do frame em `ns1:clip-path` (o SVG do Excalidraw) | ⛔ medido no oráculo: o librsvg e o Inkscape não recortam — o nosso `clip-path` é o do SVG 1.1 |
| o laser na apresentação | ⛔ o Miro não o tem; a apresentação esconde as ferramentas |
| pontos editados no Excalidraw numa seta CURVA ou em COTOVELO | ⛔ a nossa curva e o nosso cotovelo são DERIVADOS pelo roteador dos dois pontos de ligação (W2); os pontos do Excalidraw são a forma dele e não têm leitura na nossa lei — a seta volta com a rota guardada (o desenho lá mudado perde-se, a ligação não) |
| imagens no quadro (o `image` do Excalidraw) | ⛔ o quadro não tem elemento imagem (`ElementKind` = forma · seta · caneta · frame): saltam na importação com aviso que as conta — onda própria |

## §4 — Medir antes de limitar (§0.0)

Régua `crates/ph2d-board-render/tests/it/measure_team_cost.rs` (variantes no MESMO processo, intercaladas, o
mínimo de 7 rodadas, `--release`): guardar uma versão; exportar (gravar + SVG + PDF; PNG pela placa a 4/16/49
MP); ler e escrever `.excalidraw`.

### §4.1 — Guardar uma versão (09/10, `--release`, `load` 17–22 na máquina partilhada: um TECTO)

| quadro | clonar | bytes | deflate 1 | deflate 6 |
|---|---|---|---|---|
| 10 mil notas | 1,20 ms | 0,54 ms · 975 KB | 1,26 ms · 173 KB | 5,07 ms · 115 KB |
| 100 mil notas | 13,1 ms | 6,08 ms · 10,1 MB | **14,5 ms · 2,14 MB** | 52,0 ms · 1,43 MB |
| 10 mil (fluxo) | 0,49 ms | 0,36 ms · 974 KB | 0,95 ms · 160 KB | 3,43 ms · 115 KB |
| 100 mil (fluxo) | 12,2 ms | 7,05 ms · 9,98 MB | 13,4 ms · 1,70 MB | 46,0 ms · 1,14 MB |

Restaurar (ler os bytes): 22–28 ms a 100 mil. ⇒ **deflate 1**: 5× menos bytes que o cru por mais 8 ms, e o
nível 6 custa 3,6× o tempo por 33 % menos. Nenhum tecto (R5).

### §4.2 — Exportar e `.excalidraw` (09/10, `--release`, `load` 3: placa AMD, tecto de textura 8 192 px)

Régua `crates/ph2d-board-export/tests/it/measure_export_cost.rs` — o quadro INTEIRO de N notas (uma frase
cada), mínimo de 3 rodadas:

| | 10 mil notas | 100 mil notas |
|---|---|---|
| gravar a página (o `paint` vector, o detalhe inteiro) | 86 ms · 40 003 operações | 883 ms · 400 003 |
| SVG | **96 ms · 16 MB** | **1,0 s · 161 MB** |
| PDF (uma página) | **51 ms · 0,49 MB** | **0,54 s · 4,6 MB** |
| PNG 4 / 16 / 49 MP (placa, ladrilhos) | 13 / 34 / 224 ms | 118 / 134 / 201 ms |
| escrever `.excalidraw` · ler | 277 ms · 15,6 MB · 163 ms | 2,6 s · 156 MB · 1,75 s |
| guardar uma versão · lê-la | 1,3 ms · 168 KB · 2,0 ms | 15,7 ms · 1,6 MB · 23,6 ms |

⛔ **Medido e curado na mesma rodada:** a 1.ª versão escrevia o CONTORNO de cada letra em cada ocorrência —
10 mil notas davam um SVG de **144 MB em 1,4 s** e um PDF de **90 MB em 3,1 s** (100 mil: 1,44 GB em 14 s e
900 MB em 30 s). Cada glifo de cada face passou a definir-se UMA vez e a reusar-se (SVG `<defs>` + `<use>`;
PDF um Form XObject por contorno, pintado com a cor do estado): 15× e 60× menos tempo, 9× e 180× menos bytes,
o MESMO desenho (os oráculos dão os mesmos números ao píxel). E o `.excalidraw` passou a JSON compacto (o
Excalidraw lê-o igual): a entrada do oráculo de 34 KB para 15,6 KB.

⇒ **nenhum tecto.** O mais pesado é o `.excalidraw` de 100 mil elementos (o `customData` de cada um viaja
inteiro — é o que faz a volta ser exacta); o próprio Excalidraw não abre um quadro desses à mão. O PNG
Large de um quadro inteiro de 100 mil notas sai em 0,2 s porque o nível de detalhe do ecrã à escala (E3)
desenha as notas de longe como blocos — o vector desenha tudo, por isso pesa mais.

## §5 — O que o dono vê (cena 10)

`PH2D_BOARD_SMOKE=10`: um quadro de «Lançamento» com três frames (Problema · Ideias · Plano), dois fios de
comentários (um por ler, de outra pessoa; um resolvido), uma versão já guardada, e notas-lição na ordem real:
comentar (`C`, escrever, `Enter`, resolver) · guardar uma versão e restaurá-la como cópia (o menu da aba) ·
apresentar (o botão «Present» na fila de cima do canto de baixo à direita, ou o menu da aba; setas; `Esc`) ·
exportar (PNG/SVG/PDF) · abrir um `.excalidraw` (`docs/MiroClone/example.excalidraw`). A cena enquadra a
caixa do que tem mais a margem do que os painéis tapam (lição W6; `board_nav::show_box`, sem elementos
invisíveis no documento), e o gate conduz cada lição pelo caminho do ECRÃ na ordem real (lição W7),
fotografada antes de mandar.
