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
