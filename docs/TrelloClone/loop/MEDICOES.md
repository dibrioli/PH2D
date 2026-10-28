# Medições (§0.0) — nenhum MAX_* sem a tabela ao lado

## W2 — 60 fps da aba (herdada)
Cabe a 60 fps até ~20 000 cartões (CPU do layout/pintura); tabela no handoff W2→W3.

## W3 D2 — a frente do cartão com capas de IMAGEM (28/10)

Régua: `paint_card::tests::medir_capas` (ignorada; `bash scripts/ph2d-run.sh cargo test -p
ph2d-app-kanban --profile smoke --lib -- --ignored medir_capas --nocapture`). O tamanho da cena 2
(2 000 cartões numa lista), 20 fotos PNG 1600×900 distintas, ¼ capas inteiras; com × sem capa no
MESMO processo, 7 blocos intercalados, o MÍNIMO. ⚠️ `loadavg` 35,9 na corrida (máquina partilhada):
vale a DIFERENÇA, não o absoluto.

| Fase (2 000 cartões, um quadro) | com capas | sem capas |
|---|---|---|
| `front()` (a capa sai da cache por `FileId`) | 0,38 ms | 0,30 ms |
| altura SEM a cache (a disposição só a refaz quando a frente muda) | 22,07 ms | 21,92 ms |
| pintura (`draw_image_rgba` + recorte por cartão) | 7,87 ms | 6,69 ms |
| descodificar UMA foto 1600×900 (fio de fundo, nunca no desenho) | 13,9 ms | — |

- A descodificação corre num fio `kanban-covers` (um só, nasce à primeira capa); o desenho mostra a
  cor de espera até ela chegar. A cache é por `FileId` (a chave é o conteúdo): 20 ficheiros = 20
  descodificações, nunca 2 000.
- **`cover::TEXELS_W = 512`** — a foto reduz-se por metades enquanto a largura ≥ 2 × 512 (1600 → 800).
  Recurso: MEMÓRIA e o atlas do Vello (um quadrado de 8192² partilhado por todas as imagens à vista,
  `vello_encoding-0.10.0/src/image_cache.rs:10`). 512 = o cartão (256 lógicos) a 200 %; acima disso o
  Vello amplia bilinear. 800×450×4 B = 1,4 MB por foto; 20 fotos à vista = 29 MB, 3 % do atlas.
- O véu da capa inteira é medido sobre a imagem REAL: cada linha da miniatura guarda a luminância
  mais clara; o alfa sobe até o título se ler a 4,5:1 (gate
  `a_full_image_cover_keeps_the_title_legible_on_the_real_image`: imagem branca ≥ 4,5:1, escura =
  o véu da maquete `0,34`).

## W8 U — os tectos do Butler (`ph2d-kanban-auto`, 28/10)

Limites de SEGURANÇA (laço e leque de regras), nunca de hardware. Sonda
`tests/it/sonda.rs` (`--profile smoke`, o MÍNIMO de 5, `loadavg` ≈ 27 em 32 núcleos — vale a ordem de
grandeza): uma regra «quando entra em Done → comentar `{cardname}` e juntar-me», N cópias, um gesto.

| cartões | regras disparadas | antes (spec decodificado por acontecimento) | depois (cache por Kanban) |
|---|---|---|---|
| 100 | 100 | 2,59 ms | 0,27 ms |
| 100 | 1 000 | 223,8 ms | 8,46 ms |
| 1 000 | 1 000 | 223,6 ms | 8,07 ms |

- **Anti-laço** (`run.rs`): uma regra nunca dispara por acontecimentos da sua PRÓPRIA cadeia
  (A → B → A pára em A). Com isso um laço infinito é impossível; prova de mutação: tirar a guarda faz
  o par «Doing → Done / Done → Doing» correr até ao tecto (`[0..15]`), e o gate
  `a_ping_pong_pair_stops_when_a_rule_meets_its_own_chain` sangra.
- **`MAX_DEPTH = 16`** — a cadeia mais longa de regras DISTINTAS. A mais longa dos cenários reais
  medidos é 2 níveis (botão → regra de checklist/comentário); 16 = 8× de folga. Parar regista
  `Stop::Depth` (a UI avisa).
- **`MAX_RUNS = 1000`** por gesto (ou passagem da agenda) — o leque: N regras em cadeia sem laço
  ainda podem multiplicar-se. 1 000 corridas = 8 ms medidos, menos de um quadro de 16,7 ms: um gesto
  descontrolado custa no máximo um quadro. O Trello limita execuções por MÊS (quota da conta, outro
  recurso); aqui o recurso é o tempo do gesto.
- **Agenda** (`agenda.rs`): calendário perdido com o app fechado corre UMA vez (a ocorrência mais
  recente; o Butler corre no servidor e nunca acumula atraso); prazo: cada cartão cuja hora passou
  corre uma vez. `SCAN_DAYS = 367` não é tecto: todo `Every` ocorre ≥ 1×/ano (dia além do fim do mês
  = o último), por isso procurar mais atrás nunca acha ocorrência mais recente.
- **Sugestões**: `SUGGEST_MIN = 3` (o exemplo do Butler: «3 vezes») e `SUGGEST_WINDOW_MS = 5 min`
  depois do movimento — parâmetros de PRODUTO (decisão sem o dono, devolvida ao coordenador).

## W4 V — os níveis do Kanban no canvas do Quadro (28/10)

**O que se decide:** a que escala de ecrã (px por unidade do Kanban = zoom da câmara × zoom interno)
o elemento passa de BAIXO (blocos de cor por lista + contagem, pintados pelo `ph2d-board-render`) a
MÉDIO (títulos, cartões só com título e capa) e a ALTO (o Kanban inteiro, interactivo).

**Régua:** `crates/ph2d-app-kanban/tests/it/medida.rs` (`#[ignore]`) — `bash scripts/ph2d-run.sh cargo
test -p ph2d-app-kanban --profile smoke --test it medida -- --ignored --nocapture`. A cena GRANDE do
Kanban (8 listas × 250 cartões) num elemento de 1900×1000 no Quadro; o hero INTEIRO pintado
(`paint_hero_screen`, janela 1366×1024, CPU: só a codificação da cena); os três níveis forçados à MESMA
escala (`ph2d_board_render::force_kanban_lod`, no mesmo processo), aquecidos e intercalados em 12 rondas
rodadas; o resumo é o MÍNIMO. Perfil `smoke` (herda `release`). ⚠️ `loadavg` 33,9 na corrida (máquina
partilhada): o absoluto vale pouco, a diferença entre níveis intercalados é o número.

| zoom | texto do cartão (px ecrã) | título de lista (px ecrã) | nível pelo zoom | Baixo ms | Médio ms | Alto ms |
|---|---|---|---|---|---|---|
| 0.15 | 2.1 | 2.2 | Low | 3.04 | 4.12 | 4.28 |
| 0.25 | 3.5 | 3.8 | Low | 3.10 | 4.20 | 4.30 |
| 0.4 | 5.6 | 6.0 | Mid | 3.13 | 4.21 | 4.40 |
| 0.55 | 7.7 | 8.2 | Mid | 3.07 | 4.11 | 4.27 |
| 0.7 | 9.8 | 10.5 | Mid | 3.17 | 4.26 | 4.44 |
| 0.786 | 11.0 | 11.8 | High | 3.14 | 4.24 | 4.32 |
| 0.9 | 12.6 | 13.5 | High | 3.11 | 4.19 | 4.35 |
| 1 | 14.0 | 15.0 | High | 3.09 | 4.41 | 4.31 |
| 1.5 | 21.0 | 22.5 | High | 3.13 | 4.24 | 4.36 |

Com o elemento de nascença (677×381) a forma é a mesma: Baixo ~2,95 · Médio ~3,85 · Alto ~3,90 ms.

**Leitura e limiares (os do lote M confirmados, agora com a tabela ao lado):**
- **Médio → Alto: o recurso é a LEGIBILIDADE do texto que se edita.** Alto custa o mesmo que Médio
  (+0,05 a 0,2 ms, dentro do ruído) — o custo NÃO decide. Decide ler o cartão que se arrasta e se
  escreve: o texto do cartão (14 px na aba) chega à menor letra da interface, `TypeToken::Xs` = 11 px,
  à escala **0,786**. Abaixo disso só se entra no Kanban a ver traços.
- **Baixo → Médio: o recurso é a LEGIBILIDADE do título de lista** — 15 px × escala ≥ `READ_PX` = 6 px
  (o limiar de leitura do Quadro, medido 06/10), escala **0,4**. Aqui o custo ajuda: o Baixo poupa
  **~1,1 ms por Kanban grande** à vista (de longe cabem muitos); o Baixo não dispõe os 2 000 cartões.
- O Baixo é legível de longe: o nome e os títulos de lista nunca encolhem abaixo de `TypeToken::Xs` no
  ecrã, e a contagem desce para a linha de baixo (a maquete `Canvas.dc.html`, «the lists at a glance»).

## §W3 E — o tecto de tamanho por anexo (28/10)

Sonda `back::attach::tests::sonda_tecto_do_anexo` (`#[ignore]`; `cargo test --profile smoke -p ph2d-app-kanban
--lib sonda_tecto_do_anexo -- --ignored --nocapture`): bytes pseudo-aleatórios (não-imagem) no NVMe do repo, o
caminho REAL (`fs::read` → `image_size` → `KanbanSet::put_file` (sha-256 + cópia `Vec`→`Arc`) →
`KanbanSet::to_bytes` → cópia para o `ProjectFile` → gravar + `fsync` → abrir), 3 rondas intercaladas, o
MÍNIMO. ⚠️ `loadavg` 32 (máquina partilhada) e a leitura vem da cache de páginas (o ficheiro acabou de ser
escrito): disco frio lê mais devagar.

| tamanho | ler | image_size | put_file | to_bytes | cópia ProjectFile | gravar+fsync | abrir |
|---|---|---|---|---|---|---|---|
| 1 MB | 0,1 ms | 0,0 | 0,4 | 0,0 | 0,0 | 11,7 | 0,1 |
| 10 MB | 0,9 | 0,0 | 4,2 | 1,4 | 0,4 | 6,8 | 2,6 |
| 100 MB | 20,7 | 0,0 | 58,7 | 16,1 | 17,0 | 37,4 | 39,7 |
| 500 MB | 103,8 | 0,0 | 290,3 | 94,5 | 78,4 | 158,3 | 169,2 |

**Leitura:** tudo é linear, ~0,2 ms/MB a ler e ~0,6 ms/MB no `put_file` (o sha-256 é a maior parte). Nada
falha: o recurso que acaba é a **RAM** — o ficheiro vive uma vez no conjunto (`Arc`, o desfazer não o copia),
e o Ctrl+S faz mais DUAS cópias transitórias (`to_bytes` + o `ProjectFile`): pico ≈ 3× o anexo. O Trello
(10 MB grátis / 250 MB pagos) é limite COMERCIAL do servidor dele; não se copia.

**Decisão (padrão-ouro, sem tecto de tamanho):**
- Até **8 MB** lê-se na hora (ler + `put_file` ≈ 0,8 ms/MB ⇒ ≈ meio quadro de 16,7 ms); acima, **num fio**
  (`attach::read_later`): ler e `image_size` saem do fio da janela, o anexo entra num quadro seguinte com o
  seu passo de desfazer (`verso_le_o_ficheiro_grande_num_fio_e_anexa_o_depois`).
- ⚠️ **Fica no fio da janela o sha-256** (~0,4 ms/MB: ~200 ms a 500 MB) — o `KanbanSet` só aceita bytes e
  calcula a chave ele mesmo. Cura: `KanbanSet::put_file_keyed(FileId, Arc<[u8]>, size)` no modelo, com a
  chave calculada no fio (modelo é de outro agente; trecho entregue ao coordenador).
- ⚠️ **O Ctrl+S re-grava tudo** (500 MB ≈ 0,33 s de `to_bytes`+cópia+gravar no fio da janela, por gravação).
  Anexos por REFERÊNCIA (fora do `.ph2dproj`) partiriam o projecto que viaja inteiro — decisão de produto,
  não tomada; o save em fundo é da shell inteira, não do Kanban.

## W5 S2 — o retrato de quem entra (`ph2d-kanban-sync`, 28/10)

Régua: `snapshot::tests::sonda_retrato` (ignorada; `bash scripts/ph2d-run.sh cargo test -p ph2d-kanban-sync
--profile smoke --lib -- --ignored sonda_retrato --nocapture`), o MÍNIMO de 5, `loadavg` ≈ 15–16 em 32
núcleos (vale a ordem de grandeza). O pior caso: TODOS os cartões nasceram na sessão (cada campo com
carimbo), título de ~35 letras, um voto a cada três. «Antes» = o retrato plano do lote S (uma chave
`(Kanban, alvo, campo)` inteira por carimbo — um id sorteado são 10 bytes), no MESMO processo.

| cartões | Kanban (postcard) | carimbos antes | carimbos agora | retrato antes | retrato agora | blocos de 1 MiB | anfitrião: retrato + bytes | convidado: juntar + entrar |
|---|---|---|---|---|---|---|---|---|
| 100 | 8 KB | 24 KB | 5 KB | 32 KB | 13 KB | 1 | 0,2 ms | 0,2 ms |
| 1 000 | 81 KB | 254 KB | 49 KB | 336 KB | 130 KB | 1 | 1,5 ms | 1,6 ms |
| 5 000 | 411 KB | 1 279 KB | 246 KB | 1 690 KB | 656 KB | 1 | 8,9 ms | 8,6 ms |
| 20 000 | 1 667 KB | 5 176 KB | 986 KB | 6 843 KB | 2 654 KB | 3 | 38,0 ms | 46,9 ms |

**Decisão (padrão-ouro):**
- **Carimbos compactos e SEM PERDA** (`snapshot::pack`/`unpack`, gate
  `the_packed_stamps_unpack_to_the_same_maps`): agrupados por `(Kanban, alvo)` — a chave escreve-se uma vez —
  e o carimbo igual ao mais comum do alvo (o do nascimento, quase sempre) é 1 byte. Carimbos **5,2×** menores,
  retrato **2,6×** (20 000 cartões: 6,8 → 2,7 MB). Recusado: deitar fora os carimbos iguais ao do nascimento
  (seria menor, mas é uma prova de fusão a mais por fazer — o sem-perda não pede prova nenhuma).
- **Blocos de 1 MiB** (`SNAPSHOT_BLOCK`, `KMsg::SnapshotPart`): recurso = a moldura do transporte
  (`ph2d_board_net::MAX_FRAME` 256 MiB, cifrada e copiada inteira de cada lado) e o progresso de quem entra;
  o retrato de 20 000 cartões vai em 3. As escritas que chegam a meio dos blocos ficam guardadas e aplicam-se
  depois do retrato (gate `a_guest_joining_by_a_snapshot_in_blocks_converges`). Os blocos saem todos no
  `welcome` (o teto é o da ligação, sem ritmo artificial).
- ⚠️ O anfitrião paga **38 ms** (≈ 2 quadros) quando alguém entra num Kanban de 20 000 cartões nascidos na
  sessão; o convidado ~47 ms uma vez. Aceitável para um evento raro; se incomodar, o retrato sai num fio
  (o `KanbanSet` é `Clone`).

## W7 W — as vistas Tabela, Calendário, Linha do tempo e Painel (`ph2d-kanban-views`, 28/10)

**Tabela a 20 000 cartões** (`measure_table_20k`, `#[ignore]`; `fixture::big(20_000)`, perfil `dev`
(debug), mínimo de 15 corridas, carga 5–10; a cena inteira pintada numa área de 1440 × 900):

| O quê | Tempo |
|---|---|
| Pintar com as linhas em cache, ordem do quadro | 3,86 ms |
| Pintar com as linhas em cache, por entrega | 4,37 ms |
| Pintar com as linhas em cache, por nome | 2,76 ms |
| Refazer as linhas sem cache, ordem do quadro | 4,30 ms |
| Refazer as linhas sem cache, por nome (1.ª versão: 799 ms — normalizava o título em cada comparação) | 104,97 ms |

- **Sem tecto de linhas:** a rolagem é virtual (pintam-se **21** linhas, as visíveis) e o recurso que cresce é
  só a lista de ids em cache. A chave da cache: nº de registos + soma das versões (cartões, listas,
  etiquetas, campos) + ordem + filtro + «eu» + minuto. Reordenar por nome a frio (105 ms, debug) é um
  clique; o quadro de imagem seguinte já lê a cache.

**Painel — a paleta das listas sem cor** (skill de dataviz, `validate_palette.js`; superfície =
`kanban-card`): a ordem `Yellow · Blue · Red · Purple · Lime`, degrau `bold` no claro e `normal` no escuro.

| Tema | Claridade | Croma | CVD vizinhas (pior) | Visão normal (pior) | Contraste |
|---|---|---|---|---|---|
| claro `#ffffff` | PASS 0,43–0,77 | PASS ≥ 0,10 | PASS ΔE 23,7 | PASS ΔE 26,5 | PASS ≥ 3:1 |
| escuro `#22293a` | PASS 0,48–0,67 | PASS ≥ 0,10 | PASS ΔE 22,6 | PASS ΔE 24,9 | WARN < 3:1 → rótulos directos e valores sempre escritos |

- ⛔ Recusado: os degraus `normal` no claro (`#579dff,#fea362,…`: FAIL claridade e croma, `sky` cinzento) e
  as ordens que começam no azul (`blue,yellow,red,…`: FAIL CVD ΔE 3,9 / 1,1). Uma 6.ª lista sem cor fica
  cinzenta — nunca um matiz reciclado. As etiquetas e os membros pintam a cor DELES (identidade escolhida
  pelo artista), com o nome sempre ao lado.
- **Barras que cabem:** `floor((h + 10) / (12 + 10))` (barra mínima `12`, ar `10`); o resto dobra-se em
  «Other (n)» que SOMA o que dobrou (gate `dashboard_folds_what_does_not_fit_into_other`).

## J4 — defesas da sala ao vivo (28/10)

**`ph2d_board_sync::MAX_STAMP = 2^53`** (Quadro e Kanban, uma só lei) — não é medição, é aritmética:
cada escrita sobe o contador de Lamport de 1; `2^53` = 285 anos a 10⁶ escritas/s. Uma escrita com
carimbo acima é lixo de um par e NÃO se funde (o `Clock::see` ignora-a; o retrato também se corta
ao tecto); o `tick` satura. Deixa `2^64 − 2^53` de folga: o relógio nunca transborda.

**`ph2d_kanban_sync::replica::PENDING_MAX = 16 384`** — as escritas à espera de um registo/Kanban
que não chegou; cada `receive` volta a passá-las. Régua: `replica::guard_tests::measure_pending_cost`
(ignorada; `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-kanban-sync measure_pending
-- --ignored --nocapture`), 4 tamanhos no MESMO processo, 30 blocos intercalados, o MÍNIMO.
⚠️ `loadavg` 28,8 (máquina partilhada).

| Escritas à espera | um `receive` vazio |
|---|---|
| 1 024 | 0,027 ms |
| 4 096 | 0,087 ms |
| 16 384 | 0,333 ms |
| 65 536 | 1,396 ms |

Linear (~21 ns por escrita). Recurso: o quadro de imagem da janela (16,7 ms a 60 Hz), e o `receive`
corre por MENSAGEM. 16 384 = 2 % do quadro. Numa sessão sã a fila está vazia (o transporte é
ordenado e a estrela retransmite pela ordem); acima do tecto saem as mais VELHAS, com aviso no
`stderr`.

## J3 — a agenda do Butler no quadro do app (28/10)

Régua: `ph2d-kanban-hooks` `tests::medir_agenda` (ignorada; `PH2D_RELEASE="mede velocidade" bash
scripts/ph2d-run.sh cargo test -p ph2d-kanban-hooks --release medir_agenda -- --ignored --nocapture`).
`n` Kanbans da cena 1 com 21 automações cada (1 comando de calendário + 20 regras), já armadas; o
MÍNIMO de 200 amostras. ⚠️ `loadavg` 41 (máquina partilhada): vale a ordem de grandeza.

| Kanbans × 21 automações | `has_scheduled` (cada verificação) | `run_scheduled` (nada a disparar) |
|---|---|---|
| 1 | 1,9 µs | 12,9 µs |
| 10 | 1,9 µs | 127,6 µs |
| 50 | 2,1 µs | 648,2 µs |

Decisão (`AGENDA_STEP_MS = 1 000`): a corrida cresce com o projecto (0,65 ms a 50 Kanbans = 4 % de um
quadro de 16,7 ms se fosse a cada quadro) e a menor unidade de um comando é o MINUTO; uma vez por
segundo dá a hora certa ao segundo por 0,04 % do tempo. Sem comandos agendados, a verificação pára
antes de correr (`has_scheduled`; o pior caso é decodificar todas as automações de todos os Kanbans).

## R1 — «Ordenar a lista» a 20 000 cartões (28/10)

Régua: `ph2d-kanban-filter` `sort::tests::measure_sort_twenty_thousand_cards` (ignorada;
`bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-kanban-filter --lib measure_sort_twenty_thousand -- --ignored --nocapture`).
A versão de antes (`sort_keys_quadratic`, verbatim no teste) e a de agora no MESMO processo,
intercaladas, o MÍNIMO de 3. ⚠️ `loadavg` 39 em 32 núcleos: vale a ordem de grandeza.

| 20 000 cartões | antes (O(n²) + chaves em fila) | agora (Fenwick O(n log n) + bissecção) |
|---|---|---|
| ao contrário («mais antigo primeiro»; 1 fica) | 2,64 s · chave até 4 002 caracteres | 5,5 ms · chave até 4 |
| ao acaso (por nome) | 0,73 s · chave até 215 | 54 ms · chave até 6 |

O `keep` é o MESMO linha a linha (`the_fast_sort_keeps_exactly_the_rows_the_quadratic_one_kept`,
400 listas com empates de posição × 4 critérios). Gate de escala:
`sorting_a_reversed_list_grows_n_log_n_not_n_squared` (2 000 → 16 000: antes ×65,6, tecto ×24;
chave nova ≤ 12). As chaves em fila («logo acima da anterior») cresciam um carácter a cada ~6 —
a bissecção entre os dois vizinhos que ficam dá log(n).

## J4 — o retrato do anfitrião fora da janela (28/10)

Régua: `snapshot_tests::sonda_retrato_no_fio` (ignorada; `bash scripts/ph2d-run.sh cargo test -p
ph2d-kanban-sync --profile smoke --lib -- --ignored sonda_retrato_no_fio --nocapture`). 20 000 cartões
nascidos na sessão (o `hosted` do W5 S2), as variantes no MESMO processo, 7 blocos intercalados, o
MÍNIMO. ⚠️ `loadavg` 39,7 (máquina partilhada): vale a razão.

| O quê (20 000 cartões) | tempo |
|---|---|
| (a) hoje: retrato + blocos + bytes, tudo na janela | 39,0 ms |
| (b) na janela: clonar o Kanban + os mapas de carimbos | 9,4 ms |
|   — só o Kanban | 4,0 ms |
| no fio: o Kanban → postcard | 5,3 ms |
| no fio: os carimbos → `pack` | 23,5 ms |

**Decisão:** (b) — a janela paga 9,4 ms em vez de 39 (4,1×); o `pack` (o maior) e os bytes correm no
fio `kanban-portrait`. API aditiva: `HostRoom::portraits_off_window` + `take_portraits` →
`Portrait::messages` (no fio); o mesmo retrato byte a byte (gate
`a_portrait_off_the_window_is_the_same_snapshot_and_writes_before_it_wait`). A ordem entre o fio e a
janela deixa de importar: quem entra (`GuestRoom`) guarda as escritas desde o pedido até o retrato
chegar (antes, só durante os blocos). Usado nas duas salas (a do Quadro e a do Kanban sozinho).
Recusado: retrato incremental por blocos ao longo de vários quadros — a cópia continuaria a ser
precisa para um retrato coerente, e partir o `pack` exigiria um cursor sobre mapas que mudam entre
quadros. Os 9,4 ms que restam são a cópia; descê-los pede estruturas persistentes (partilha
estrutural do `Kanban`), uma mudança do modelo fora do âmbito desta linha.

## J4 — a reconexão do Kanban partilhado (28/10)

`share::net_io::{RETRY_FIRST = 250 ms, RETRY_MAX = 8 s, RECONNECT_FOR = 60 s}` — o minuto é o do
Quadro (`ph2d-app-board::live::RECONNECT_FOR`: o Wi-Fi que soluça volta em segundos; o portátil
fechado não). A 1.ª tentativa a 250 ms (um soluço de rede local reabre em < 1 RTT de TCP + aperto),
dobrando até 8 s: no minuto são ≤ 12 tentativas; cada uma custa um fio e, por link, até 30 s de
prazo (`LINK_TIMEOUT`), por isso o tecto não desce. Gate `the_backoff_doubles_up_to_its_ceiling`.

## T1 — quantas sugestões de `@`/`#`/`^` no título do cartão novo (28/10)

Era `MAX = 6`, sem medição. O recurso é o ESPAÇO por baixo do campo: a folha (`sheet::place`) que não
cabe por baixo da âncora sobe e tapa o próprio campo. Lei nova: `rows_fit = ⌊(área.fundo − PAD − (âncora.fundo + LABEL_GAP) − PAD) / ROW⌋`,
mínimo 1 (`ROW` 32, `PAD` 12, `LABEL_GAP` 4, campo 36 px; conta sobre as constantes do código, gate
`the_suggestions_that_fit_follow_the_room_under_the_field`). Não conta o HUD (`floor`) do hero.

| Janela (área útil) | campo termina em y | linhas que cabem |
|---|---|---|
| 1080 px (≈1000) | 136 (lista curta) | 26 |
| 1080 px | 536 (lista a meio) | 13 |
| pequena (420) | 286 | 3 |
| pequena (420) | 336 | 1 |

O Trello não foi corrido (sem oráculo instalado): de memória lista todos os que casam num popover que
rola; aqui as linhas são limitadas só pelo espaço (os candidatos já filtram pelo prefixo).
