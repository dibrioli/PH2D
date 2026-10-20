# 05 — Facilitação: cronómetro, votos, carimbos e agrupar (W7)

> **Pedido do dono (plano §3 W7, ordem de 09/10 «smoke ok. Vamos seguir»):** cronómetro visível no quadro;
> carimbos e pontos de voto sobre as notas, com a contagem por nota; agrupar notas por cor, etiqueta ou
> palavra. ⛔ Fora: votação anónima ENTRE PESSOAS AO MESMO TEMPO e modo privado (Etapa 2, rede — ✅ FEITOS
> na W12/W13, ADR-0182/0183: boletins no cofre do anfitrião; confirmado no bloco 6, doc 17 §0). **O dono vê:**
> um brainstorm facilitado num ecrã só — o tempo a correr, votos nas ideias, as ideias agrupadas.
>
> Briefing: [`handoffs/HANDOFF_CONTINUACAO_W7_2026-10-09.md`](handoffs/HANDOFF_CONTINUACAO_W7_2026-10-09.md).
> Fontes: a ajuda do Miro lida pela API Zendesk ([`ferramentas/facilitacao_miro_notas.md`](ferramentas/facilitacao_miro_notas.md),
> 653 artigos varridos, 15 lidos), as imagens oficiais medidas ao píxel (§1.2), o FigJam pela ajuda dele, e o
> oráculo da PALAVRA corrido ([`ferramentas/facilitacao_oracle/`](ferramentas/facilitacao_oracle/README.md)).

## §0 — Veredito

Num ecrã só, a facilitação do Miro é quatro coisas, e três delas já têm peças no quadro:

- **Cronómetro** — estado da SESSÃO do quadro (nunca do ficheiro, nunca do desfazer): um relógio que se
  arma, corre, pausa, soma `+1m`/`+5m` e acaba com aviso visível e campainha. O ecrã já se redesenha a cada
  quadro (a shell corre em `ControlFlow::Poll`): o relógio lê o `Instant` no desenho e não pede gancho novo.
- **Votação** — uma SESSÃO (votos por pessoa, «um voto por objecto», os `+` nos objectos, «I'm done» passa
  o ecrã à pessoa seguinte, «End for all») cujo RESULTADO é documento: o total de cada objecto fica nele
  (`Org::votes`, formato 7) e desenha-se como o selo azul medido do Miro. Num ecrã só, o anonimato é por
  TURNO: cada pessoa vê só os seus votos até ao fim (o que o Miro faz entre pessoas, aqui faz-se entre
  turnos — a parte que precisa de rede continua na Etapa 2).
- **Carimbos** — o Miro **não tem** ferramenta de carimbo (0 ocorrências nos 653 artigos); tem emoji na
  base da nota, cada um com contagem. Nós não temos fonte de emoji (só Inter/Noto/Atkinson/Virgil, sem
  COLR): os carimbos são oito ícones **Lucide** (ISC, a família dos ícones da interface) desenhados em
  vector, com a contagem ao lado (`Org::stamps`, formato 7).
- **Agrupar (Cluster)** — por cor, etiqueta ou palavra-chave; cada grupo é um **frame** (o «container» do
  Miro: move-se junto, tem título, renomeia-se com duplo-clique, redimensiona-se — a composição já o
  exprime), as notas numa grelha de ⌈√n⌉ colunas com as folgas medidas, os grupos lado a lado. A palavra é
  algorítmica e **determinística**: segmentação UAX#29, *case folding*, raiz Snowball 3.1.1 (gerada pelo
  compilador oficial e conferida palavra a palavra contra a `libstemmer` 3.1.1 instalada), palavras vazias
  do Snowball. ⛔ O «Keywords» do Miro é IA e não se corre; o «Author» e o «Sentiment» também saem (§3).

## §1 — Como os apps fazem (fontes)

### §1.1 — Miro (o produto do dono) e FigJam: a ajuda

Detalhe e citações em [`ferramentas/facilitacao_miro_notas.md`](ferramentas/facilitacao_miro_notas.md).
URLs: prefixo `https://help.miro.com/hc/en-us/articles/` — TM=`360017730933-Timer` ·
VO=`360017572274-Voting` · CL=`4409706795410-Clustering` · SE=`360016026559-Stickers-and-Emojis`;
FigJam: FV=`help.figma.com/hc/en-us/articles/9359912208663` · FT=`…/4402269549591`.

- **Cronómetro:** MM:SS; arma-se a escrever ou com `−`/`+` e o ▶ azul (TM). A correr: ■ parar (= repor, pede
  confirmação «Stop timer / Cancel», GIF c4273), ⏸ pausar, **+1m**, **+5m** (TM); o FigJam: *«+1 min … when
  the clock is running or paused»*, *«time can be added but not reduced»*, *«up to 99 minutes and 59
  seconds»*, *«Once paused, the clock will flash»*, no fim *«shake and announce 00:00»* (FT). O `±` anda de
  1 em 1 min até 10 e de 5 em 5 acima; sem segundos **[comm c16256]**. No zero: *«notified visually and
  with a chime»*; minimizado, o ícone mostra o resto; *«Can I move the timer…? No»* (TM).
- **Votação:** *«the maximum number of votes is 99»* por pessoa; *«one vote per object»* é um interruptor
  (por omissão podem ir vários na mesma); vota-se e desvota-se pelos `+`/`−`; no fim *«top 3 results»* e
  *«See all results»* (lista com barra); *«keep voting results visible on the board using the toggle»*;
  duplicar o quadro NÃO leva os resultados (VO). O FigJam: *«Use any stamp to cast your vote»* (FV).
- **Emoji/carimbos:** emoji na base da nota, vários por nota, cada um com contagem; clicar no seu tira-o
  (SE, figura `stamp_02`). Sem ferramenta «Stamp», sem tecla.
- **Agrupar:** *«Cluster by Color, Tag, Author, Keywords or Create a cluster»*; *«If you cluster by color,
  you need to manually add a title to each group»*; *«If a sticky note has several tags, it will be
  duplicated in several clusters. If some stickies have no tag, they will be grouped in one cluster under a
  "no tag" title»*; Keywords é *«AI-powered»* e o resto vai para *«an #uncategorized cluster»*; os grupos
  ficam em *«containers»* que se movem pelo título (CL).
- **Calado:** duração de omissão do cronómetro, o que pisca no zero, tempo negativo, atalhos; votos de
  omissão (a imagem mostra 3); a lista de carimbos; folgas, regra de colunas, ordem dos grupos e das notas,
  onde cai o resultado; o que é uma «palavra».

### §1.2 — As imagens oficiais medidas ao píxel

Régua: scripts Python sobre os píxeis (componentes ligados por cor), no scratchpad da sessão; números
transcritos aqui. Lado da nota = `s`.

- **Selo de resultado** (`vote_05`, 1800×969, notas de `s` ≈ 236): círculo azul `#4262FF` (66, 98, 255) com
  o número branco e uma ponta descolada no canto de cima à esquerda. **2 votos: Ø 80 = 0,34 s** com o
  centro a (+6,5, +5) do canto inferior direito; **1 voto: Ø 52 = 0,22 s**, centro a (+7,5, +8,5). O selo
  CRESCE com o número. `vote_04` (um selo «1» sobre texto): Ø 60.
- **O `+` de votar** (`vote_03`, a 148 %): círculo cinzento Ø **49 = 0,21 s**, centro EM CIMA do canto
  inferior direito (±1 px), igual em todas as notas.
- **Agrupar por etiqueta** (`cluster_01`, último quadro, `s` = 142): três containers numa FILA, alinhados em
  cima — Q1 (5 notas, 3 colunas), «no tag» (1, 1 coluna), Q2 (4, 2 colunas). Passo horizontal 164–165
  (folga **22 = 0,155 s**), passo vertical 181 (**1,275 s**: a folga 39 inclui a sombra da nota, a altura
  228/199 do Miro); margem do container: esquerda 44 (**0,31 s**), cima 47 (**0,33 s**), baixo 62 (**0,44 s**,
  com a sombra); entre containers **58 = 0,41 s**; o título é uma etiqueta preta de 57 de alto (0,40 s) a 9
  (0,06 s) acima do container.
- **Agrupar por palavra** (`cluster_02`, `s` ≈ 126): quatro containers numa fila (2, 3, 2, 2 notas → 2, 2, 2, 2
  colunas); folga horizontal 23 (**0,18 s**), passo vertical 160 (**1,27 s**); margem esquerda 38 (**0,30 s**),
  cima 40 (**0,32 s**), baixo 53 (**0,42 s**); entre containers **48 = 0,38 s**; títulos «#use #need #learn»,
  «#app #love #intuitive», «#features #lacks #use», «#things #difficult #easy» — **três** palavras, e a
  mesma pode estar em dois títulos.
- **Colunas** (as duas imagens): 1→1 · 2→2 · 3→2 · 4→2 · 5→3 = **⌈√n⌉**. ⚠️ O GIF de anúncio de 2022 (versão
  antiga, sem containers) dá 7→4: recusado (§3).
- **A ordem:** «Q1 · no tag · Q2» — nem alfabética (com ou sem maiúsculas), nem por tamanho, nem «o resto
  no fim». As notas de Q1 também não seguem o x original. ⇒ a ordem do Miro é a da primeira aparição
  numa ordem que a imagem não mostra (a de criação?).
- **Cronómetro** (`timer_04`, GIF da Miro na comunidade): um cartão com MM:SS grande, por baixo o
  altifalante, `+1m`, `+5m`; à direita ■ (círculo claro) e ⏸ (círculo azul); um `−` para minimizar fora do
  canto.

### §1.3 — Oráculos que se CORREM (§0.9)

[`ferramentas/facilitacao_oracle/`](ferramentas/facilitacao_oracle/README.md). Triagem por ARTEFACTO:

| candidato | licença (prova) | faz | corre? | veredito |
|---|---|---|---|---|
| tldraw 5.5.2 / Excalidraw 0.18.1 (os de `ferramentas/`) | já triados | **nada** de voto, cronómetro, carimbo ou agrupar nos `.d.ts` (o tldraw tem reacções só em comentários, num pacote que não existe no npm) | — | não servem |
| Scrumlr | MIT | voto, cronómetro | não (Go + Postgres + NATS, sem docker) | só doc, e a doc não tem as regras |
| Retrospected · Parabol · Postfacto | GPL-3 · AGPL-3 · AGPL-3 (o ficheiro, não o comparativo que dizia MIT) | voto, cronómetro, agrupar | não | recusados |
| Miro · FigJam | proprietários | tudo | não | só a ajuda |
| **`libstemmer` 3.1.1** (Snowball) | **BSD-3** (`pacman -Qi`, `/usr/share/licenses/libstemmer/COPYING`) | a raiz da palavra | ✅ por `ctypes` | **oráculo da palavra** |
| `Intl.Segmenter` (Node 24.19, ICU 78.3) | ICU | divisão em palavras UAX#29 | ✅ | oráculo da segmentação |

⇒ Para o cronómetro, o voto e o agrupar **nenhum oráculo corre**: a fonte é o Miro medido (§1.2), com o
FigJam onde o Miro cala. Para a palavra há porta aberta, e corre-se.

**A raiz da palavra, medida contra a `libstemmer` 3.1.1 sobre o vocabulário oficial do Snowball**
(`snowball-data`: 42 649 palavras inglesas, 32 016 portuguesas):

| implementação | inglês | português |
|---|---|---|
| `rust-stemmers` 1.2.0 (MIT/BSD-3) | **57** diferentes (`added → ad`: o algoritmo de 2019) | 0 |
| `waken_snowball` 0.1.0 (BSD-3) | **17** diferentes (todas `inter*`: a regra do 3.1) | 0 |
| ⭐ o compilador oficial **Snowball 3.1.1** (BSD-3) → Rust | **0** | **0** |

⇒ o código Rust gerado pelo próprio Snowball 3.1.1 entra **vendorizado** (o idioma do ADR-0123:
`crates/ph2d-board-words/vendor/snowball/`, fora do workspace, com a licença e o `VENDOR.md`), e a corrida
do oráculo é uma fixture com gate.

Os três achados do oráculo da palavra (entradas nossas, `notas_w7.json`):

1. **Apóstrofo curvo:** `team’s` (U+2019) dá a raiz `team’` e não agrupa com `team`; com `'` dá `team` ⇒ o
   U+2019 vira `'` antes de tudo.
2. **Minúsculas não chegam:** `Straße`/`strasse`, a ligadura `ﬁle` ⇒ *case folding* COMPLETO (o do
   `ph2d-label-fold`). Mas os acentos ficam ATÉ À RAIZ (o Snowball português precisa de `ção`); só a raiz
   perde os acentos, para `Café` = `cafe`.
3. **Listas de palavras vazias grandes demais:** a ISO inglesa tem 1 298 e leva `high`, `test`, `work`,
   `need`, `new`; a portuguesa leva `tempo`, `trabalho`, `novo` ⇒ as do **Snowball** (BSD-3: 174 inglesas,
   203 portuguesas), que só têm palavras funcionais.

## §2 — As leis do PH2D (lei → fonte → número)

### Cronómetro (sessão do quadro)

| # | lei | fonte | número / teste |
|---|---|---|---|
| F1 | Um cronómetro por quadro, da SESSÃO: não vai para o ficheiro nem para o desfazer; trocar de aba não o pára | Miro TM (global, ao vivo), SDK (estado em ms); FigJam «one active timer per file» | `the_timer_is_session_state` |
| F2 | Armar: MM:SS, `−`/`+` de **1 min até 10** e de **5 min daí para cima**; sem segundos; nasce em **5:00** | Miro [comm c16256]; omissão calada ⇒ escolha | `the_step_is_one_minute_below_ten_and_five_above` |
| F3 | Tecto **99:59** (armar e somar) | FigJam FT (oficial); Miro [comm c4242] | `the_timer_never_passes_99_59` |
| F4 | A correr: ⏸ pausa (o relógio PISCA enquanto pausado), ▶ retoma; **+1m**/**+5m** a correr ou pausado; nunca se tira tempo | Miro TM; FigJam FT | `adding_time_works_running_or_paused` |
| F5 | ■ pede confirmação («Stop timer» / «Cancel») e repõe a duração armada | Miro TM + GIF c4273 | `stop_asks_then_resets` |
| F6 | No zero: o cartão fica em `00:00` realçado e treme, e soa a **campainha** (a menos que o altifalante do cartão esteja desligado); fica assim até ■ ou ▶ | Miro TM (visual + campainha); FigJam FT (treme e anuncia 00:00) | `at_zero_it_rings_once_and_stays_at_zero` |
| F7 | O cartão (e o painel da votação por baixo dele) vive no canto de cima À DIREITA da área — o do Miro —, à esquerda do painel de Frames quando ele está aberto e por baixo da busca quando ela está aberta; não se arrasta; o fundo toma o clique; `−` minimiza-o numa pílula com o resto | Miro TM («No») | `the_timer_card_never_covers_the_frames_panel` · `the_card_avoids_the_search_and_hides_dead_steps` · `a_click_on_the_card_never_reaches_the_board` |

### Votação (sessão) e o resultado (documento)

| # | lei | fonte | número / teste |
|---|---|---|---|
| V1 | Abrir: votos por pessoa **1–99**, nasce **3**; «One vote per object» desligado; minutos (0 = sem relógio) que ARMAM o cronómetro do quadro; o que se vota = a selecção (se houver) ou o quadro inteiro, e só notas e formas (não setas, traços, frames) | Miro VO (99, o interruptor, a área e o filtro por tipo; a imagem `vote_02`: 3 votos, 2 min) | `a_session_votes_on_the_selection_or_the_board` |
| V2 | Em sessão, cada objecto votável mostra o `+` cinzento Ø **0,21 s** com o centro no canto inferior direito; clicar dá um voto (se há votos e, com o interruptor, se ainda não votou nele); o `+` passa a mostrar os SEUS votos e ganha o `−` | Miro VO + `vote_03` | `a_click_on_the_plus_casts_one_vote` · `one_vote_per_object_refuses_the_second` |
| V3 | «N votes left»; **I'm done** passa o ecrã à pessoa seguinte (os votos dela escondem-se, os dela começam do zero); **End for all** fecha | Miro VO (o painel); o turno é a forma de um ecrã só | `the_next_voter_sees_none_of_the_previous` |
| V4 | O fim dá o resultado: o total de cada objecto (`Org::votes`, formato 7), num passo de desfazer que tira o resultado anterior do quadro; o painel mostra o **top 3**, «See all results» (a lista por ordem, até onde a área chega — «…and N more») e «New vote» (a votação seguinte) | Miro VO | `ending_writes_the_totals_in_one_step` · `the_result_panel_starts_a_new_vote_and_fits_the_area` |
| V5 | O selo: círculo `#4262FF`, número branco, centro a **0,03 s** do canto inferior direito para fora; Ø de **0,22 s** (o menor total) a **0,34 s** (o maior), linear entre os dois | `vote_05` medido (dois pontos: a lei entre eles é escolha); a cor é do produto | `the_badge_grows_with_the_count` |
| V6 | O cronómetro da sessão a chegar a zero acaba a votação | Miro VO (duração da sessão) | `the_clock_ends_the_vote` |
| V7 | Copiar um objecto NÃO leva os votos dele; «Clear results» tira-os (um passo) | Miro VO (o duplicado não leva) | `a_copy_carries_no_votes` |

### Carimbos (documento)

| # | lei | fonte | número / teste |
|---|---|---|---|
| S1 | Oito carimbos Lucide: polegar acima, polegar abaixo, coração, estrela, chama, lâmpada, pergunta, festa — cada um com a contagem (`Org::stamps`, formato 7) | Miro SE (emoji na nota, cada um conta); FigJam FV («use any stamp to cast your vote»); sem fonte de emoji ⇒ ícones | `a_stamp_counts_on_the_note` |
| S2 | Pôr: o botão «Stamp» da barra de contexto de notas abre a galeria; escolher dá +1 desse carimbo a cada nota seleccionada (um passo) | Miro SE (na barra da nota) | `stamping_the_selection_is_one_step` |
| S3 | Na nota: fichas por baixo da nota, da esquerda, na ordem em que nasceram; clicar numa ficha +1, `Shift`+clicar −1 (a 0 a ficha sai) | Miro SE (clicar no seu tira-o; num ecrã só o clique é de quem está na sala) | `shift_click_takes_one_back` |
| S4 | Uma cópia LEVA os carimbos (são da nota, ao contrário dos votos, que são da sessão) | escolha (o Miro cala) | `a_copy_carries_its_stamps` |

### Agrupar (cria frames: nenhum campo novo)

| # | lei | fonte | número / teste |
|---|---|---|---|
| C1 | **Cluster** › By color · By tag · By keyword, na barra de contexto de 2+ notas — e na de um FRAME com 2+ notas dentro (agrupa as de dentro) | Miro CL (Author e Sentiment fora, §3); o frame: achado do gate da cena 9 (dentro de um frame o vazio é do frame — W5, ordem do dono —, uma caixa à volta das ideias movia o frame) | `cluster_needs_two_notes` · `a_selected_frame_clusters_its_notes_and_grows_to_hold_them` |
| C2 | Cada grupo é um FRAME com as notas dentro (a casa delas passa a ser ele), lado a lado numa FILA alinhada em cima, a começar no canto superior esquerdo da caixa das notas; os grupos moram na casa comum das notas, que CRESCE (e as de fora dela) para os conter; o resultado fica seleccionado; UM passo | `cluster_01`/`_02`; o lugar é calado ⇒ escolha; crescer: um filho mora dentro dos limites do pai (SDK do Miro, DEV-F) | `the_cluster_grid_is_the_measured_one` · `a_selected_frame_clusters_its_notes_and_grows_to_hold_them` |
| C3 | Dentro: ⌈√n⌉ colunas, as notas DIREITAS (uma rodada sairia da célula); folga horizontal **0,17 s**, passo vertical com a folga **0,27 s**; margem **0,30 s** dos lados, **0,32 s** em cima, **0,43 s** em baixo; entre frames **0,40 s** (médias das duas imagens) | §1.2 | `the_cluster_grid_is_the_measured_one` |
| C4 | Ordem dos grupos e das notas: a primeira aparição na ORDEM DE LEITURA da selecção | §1.2 (a do Miro não se vê; nenhuma das óbvias bate) ⇒ escolha, a do `reading_order` | `clusters_keep_reading_order` |
| C5 | Por cor: um grupo por cor, SEM título (o artista dá-lho) | Miro CL | `color_clusters_have_no_title` |
| C6 | Por etiqueta: título = a etiqueta; nota com várias etiquetas é DUPLICADA em cada grupo; as sem etiqueta num grupo «no tag» | Miro CL | `a_note_with_two_tags_is_in_two_clusters` |
| C7 | Por palavra: palavras UAX#29 com letras; `’`→`'`; *case folding*; fora as palavras vazias do Snowball; raiz Snowball 3.1.1; a chave é a raiz sem acentos. Língua = a que tiver mais palavras vazias nas notas (inglês ou português; empate = inglês). Grupo = a raiz partilhada pelo MAIOR número de notas ainda soltas (≥ 2; empate pela primeira aparição), com todas elas; repete; o resto em «#uncategorized». Cada nota num grupo só | Miro CL (títulos, #uncategorized); oráculo §1.3 | `notes_sharing_a_root_cluster_together` · `stems_match_libstemmer` |
| C8 | Título de um grupo por palavra: as **três** raízes mais frequentes nele (empate pela primeira aparição), cada uma escrita na forma mais frequente, com `#` | `cluster_02` | `a_keyword_title_has_three_words` |
| C9 | Notas travadas ficam onde estão | W5 (travado não se move) | `a_locked_note_is_left_alone` |

## §2.1 — A auditoria de duas lentes (09/10): o que ela apanhou e ficou curado

26 achados (12 da lente das leis, 14 da costura/desenho/cena), cada cura presa a um teste:
o projecto carregado (ids de quadro recomeçam) e o quadro apagado levam a facilitação (relógio, painel,
sessão) — `loading_or_deleting_a_board_takes_its_facilitation`; os relógios andam com a aba Cena —
`the_chime_rings_with_the_scene_tab_open`; o zero do relógio da votação espera o fim de um gesto (o
desfazer do arrasto guardaria o antes) — `the_clock_waits_for_the_gesture_to_end`; o ■ larga o relógio
da votação — `stopping_the_timer_releases_the_vote`; a sessão segue o quadro ANTES de ele se desenhar
(um quadro mostrava os `+` do anterior); um alvo apagado devolve os votos, um escondido não mostra o
`+` — `a_deleted_target_gives_its_votes_back` · `a_hidden_node_shows_no_plus`; a casa do agrupar é o
antepassado comum, não cresce se travada, e não recebe a grelha se rodada —
`the_home_is_the_common_ancestor_frame` · `a_locked_home_does_not_grow` ·
`a_rotated_home_passes_the_groups_out_and_notes_straighten`; as fichas param antes do selo —
`stamp_chips_stop_before_the_vote_badge`; o `−` do tecto volta à grelha de 5; o Stamp só com notas
editáveis; os ± dos limites não se pintam; e ⛔ a **lição 2 da cena 9 era falsa** (a cena nasce com um
resultado e o painel dele não tinha como abrir uma votação nova; o gate da cena chamava o editor e não
o painel) — «New vote» e `the_vote_lesson_runs_from_the_panel`.

Ficam ABERTOS (baixos, no handoff): os `+` de uma sessão continuam clicáveis numa nota dentro de um frame
visto de longe (só o título se pinta) — o editor não conhece o limiar do nível de detalhe; o título do
frame de fora da cena 9 fica debaixo da barra (o idioma da cena 8); o título de um grupo por palavra
mostra a forma em *case folding* («API» → «#api»).

## §3 — ⛔ Recusas MEDIDAS

| proposta | estado |
|---|---|
| «Cluster by Keywords» pela IA (como o Miro) | ⛔ não corre por script, não é determinística, e um quadro é um documento offline; a palavra é algorítmica (C7) e o oráculo dela corre |
| «Author» e «Sentiment» | ⛔ Author: num ecrã só todas as notas são da mesma pessoa (Etapa 2). Sentiment: IA |
| a lista de palavras vazias ISO (crate `stop-words`, 1 298 inglesas) | ⛔ medido (§1.3): apaga `need`, `work`, `test`, `new` — o conteúdo de uma nota curta |
| `rust-stemmers` / `waken_snowball` como dependência | ⛔ medido (§1.3): 57 e 17 palavras inglesas diferentes da `libstemmer` 3.1.1 |
| tirar os acentos ANTES da raiz | ⛔ o Snowball português corta `ção`/`ções`; sem acentos, `informação` e `informações` davam raízes diferentes |
| colunas do GIF de 2022 (7→4) | ⛔ versão antiga (sem containers); o help de hoje dá ⌈√n⌉ |
| os carimbos como EMOJI de fonte | ⛔ não há fonte de emoji no programa nem caminho COLR no desenho: seria tofu |
| o selo do resultado com Ø fixo | ⛔ medido (§1.2): 52 → 80 de 1 para 2 votos |
| a campainha pelo som de UI (`UiSound`, opt-in) | ⛔ a lei desse som é «só o que a mão fez» e nasce desligado; a campainha é um ALARME que o facilitador armou — tem o seu próprio altifalante no cartão (F6) |
| votos anónimos entre pessoas em ecrãs diferentes, modo privado | ✅ feitos na Etapa 2 (W12/W13, ADR-0182/0183); o turno do V3 continua a forma de um ecrã só |

## §4 — Medir antes de limitar (§0.0)

Régua `crates/ph2d-board-render/tests/it/measure_facilitate_cost.rs` (o molde da `measure_mind_layout_cost.rs`:
variantes no MESMO processo, intercaladas, o mínimo de 7 rodadas, `--release`): **agrupar** 100 / 1 000 /
10 000 notas por cor, etiqueta e palavra (a operação inteira pelo editor: o lote, os frames, a grelha); e o
**desenho** de um quadro de 10 000 notas sem nada, com selos de voto em todas e com três carimbos em todas.
Nenhum tecto entra sem a tabela; o cronómetro não pede régua própria (o ecrã já se redesenha a cada quadro,
e o cartão é um desenho de interface de tamanho fixo).

### §4.1 — O que a régua mediu (09/10, `--release`, `load` 12–15 na máquina partilhada: os números são um TECTO)

**Agrupar** (a operação inteira pelo editor):

| N notas | por cor | por etiqueta | por palavra |
|---|---|---|---|
| 100 | 0,09 ms (med 0,10) | 0,08 (0,08) | 0,43 (0,44) |
| 1 000 | 0,77 (0,78) | 0,75 (0,75) | 4,31 (4,33) |
| 10 000 | 8,74 (9,02) | 8,33 (8,43) | 79,9 (81,8) |

⇒ **nenhum tecto de notas.** Agrupar é um GESTO (um clique), não um quadro de animação: 80 ms por 10 mil
notas por palavra (dez vezes o maior brainstorm de uma equipa) é um clique que se sente, e as 1 000 de
um workshop grande custam 4 ms. A palavra é ~10× a cor: a segmentação e a raiz de cada palavra.

**Desenhar** 10 mil notas (encode na CPU · parede da placa, mínimos):

| vista | nada | selos em todas | 3 carimbos em todas |
|---|---|---|---|
| letra a 6 px (o limiar da leitura: o máximo de notas com marcas à vista) | 0,71 · 1,05 | 1,30 · 1,21 | **2,03** · 1,85 |
| zoom 1 | 0,46 · 0,46 | 0,53 · 0,49 | 0,60 · 0,55 |
| de longe (blocos: sem marcas — o controlo) | 1,21 · 0,52 | 1,21 · 0,53 | — |

⛔ **Medido e curado na mesma rodada:** a 1.ª versão lia o SVG do ícone por ficha e por imagem — 3
carimbos em todas no limiar da leitura davam **5,44 ms** de encode; com os caminhos lidos uma vez
(`stamp_path` em `OnceLock`), **2,03**. ⛔ E a 1.ª corrida da régua usou a vista «a meio» da régua do
encode (letra a 5 px), que é de BLOCOS: as três variantes deram o mesmo número porque não se desenhava
marca nenhuma — a régua passou a ter a vista no limiar e a de longe como controlo.

⇒ **nenhum tecto de selos nem de carimbos**: o pior caso cabe com folga num quadro de 60 Hz.

**Re-medido 11/10 (W13: o selo lê a sessão MOSTRADA — uma procura num `BTreeMap` por nota desenhada;
`--release`, `load` 16):** letra a 6 px · selos **1,34** ms de encode (nada 0,76; 3 carimbos 2,07); zoom 1 ·
selos 0,54; de longe 1,23. O degrau do selo na placa (selos − nada) é 0,15 ms (era 0,16); as paredes da placa
subiram TODAS, «nada» incluído (1,99 contra 1,05) ⇒ é a carga, não o selo. Nenhum tecto, nenhuma cura. O
cronómetro não pede régua: o ecrã já se redesenha a cada quadro e o cartão é um desenho de tamanho fixo.

## §5 — O que o dono vê (cena 9)

`PH2D_BOARD_SMOKE=9`: uma retrospectiva já com ideias — notas de cores e etiquetas variadas, umas com
carimbos e duas com o resultado de uma votação já feita —, o cronómetro armado a 5:00, e notas-lição na
ordem real: armar e arrancar o cronómetro · votar (abrir a votação, os `+`, «I'm done», «End for all», o
resultado) · carimbar (o botão Stamp, clicar numa ficha, `Shift`+clicar) · agrupar por cor, por etiqueta e
por palavra (clicando no frame «Ideas»: a 1.ª redacção dizia «arraste uma caixa à volta das ideias» e o gate
mostrou que esse arrasto MOVE o frame — o vazio de um frame é dele, W5). Tudo num frame com a margem do que os painéis e a barra de estado tapam (lição da W6), e o gate
conduz cada lição pelo editor na ORDEM REAL, a dois zooms.
