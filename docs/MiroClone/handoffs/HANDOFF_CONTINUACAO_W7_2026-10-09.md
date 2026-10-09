# HANDOFF DE CONTINUAÇÃO — `line/MiroClone` → W7 «Facilitação» (2026-10-09)

> Para a PRÓXIMA JANELA desta linha. Antecessor: [integração W6](HANDOFF_INTEGRACAO_line_MiroClone_W6_2026-10-08.md)
> (W5 + W5b + W6 fechadas; **smoke do dono OK nas três**, a W6 a 09/10). Plano: [`../02_plano.md`](../02_plano.md)
> §3 W7 (§6: as recusas medidas — leia antes de propor desenho). Pesquisa: [`../01_pesquisa_miro_excalidraw.md`](../01_pesquisa_miro_excalidraw.md)
> §3–§4 (brainstorm: votação, timer, cluster). **Ordem do dono (09/10): «smoke ok. Vamos seguir».**
> Decisões técnicas são suas (padrão-ouro); só uma de PRODUTO que o estudo não resolva volta ao dono, em
> linguagem dele e no fim.

## §0 — Identidade e estado da árvore

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`. W5, W5b e W6 **não foram integradas**; a
  W7 continua POR CIMA, na mesma branch.
- Base `2a925859d`. **Antes de tudo** (MODELO_TROCA §FASE 1): `git rev-parse main` — se o `main` andou,
  `git cherry main HEAD | grep -c '^+'`: `0` ⇒ `git reset --keep main`; `>0` ⇒ `git range-diff main...HEAD`
  e só depois `git rebase main`.
- Formato do quadro: `FORMAT_VERSION = 6` (W6: `Org::mind`). Campo novo no documento (os votos de uma nota,
  um carimbo, o cronómetro se for do ficheiro) = **7**, com leitor do 6 e fixture GRAVADA pelo build antigo
  (o idioma de `legacy.rs::read_v5` + `fixtures/format_v5.bin`: um teste temporário no build actual ANTES
  de mexer no modelo; ⛔ nunca bytes à mão). O `PROJECT_SCHEMA` (187) não se toca.
- Cena de smoke seguinte: **9** (`crates/ph2d-app-board/src/smoke.rs`, hoje `max_level: 8`).
- ⚠️ **Tectos de LOC (700, os `*_tests.rs` CONTAM) — corte antes de acrescentar:** `board_bar.rs` **693**,
  `gesture.rs` **685**, `ph2d-board-render/src/lib.rs` **676**, `ph2d-board-route/src/lib.rs` 678,
  `organize_tests.rs` 695, `guides_tests.rs` 688, `mindmap.rs` 648, `organize.rs` 605, `notes.rs` 601.
  A facilitação nasce em módulos próprios (`vote.rs`, `cluster.rs`, `timer.rs` + `*_tests.rs`, e um
  `board_bar_facilitate.rs`) — a barra entra por chamadas de UMA linha (o idioma de `board_bar_mind.rs`).

## §1 — O que a W7 é (plano §3)

1. **Cronómetro** visível no quadro (começar, pausar, somar tempo, o fim que se nota).
2. **Carimbos / pontos de voto** sobre as notas (e a contagem por nota).
3. **Agrupar notas** por cor, por etiqueta ou por palavra (o *Cluster* do Miro).
4. ⛔ Fora: a votação **anónima** e o modo **privado** precisam de várias pessoas ao mesmo tempo ⇒ Etapa 2.
5. **O dono vê:** um brainstorm facilitado num ecrã só — o tempo a correr, votos nas ideias, as ideias
   agrupadas.

## §2 — O que já existe e NÃO se reconstrói

- Notas (16 cores do Miro, tamanhos, pilha, grelha pela pega de 4 pontos — `notes_layout::grid_layout`,
  `reading_order`), **etiquetas** nas notas (W5, `Org::tags`, `MAX_TAGS` 8) e a **busca com filtro por
  etiqueta** (`board_nav_search::find`) — o agrupar por etiqueta parte daqui.
- Frames (o cluster do Miro pode pôr cada grupo num frame — meça), grupos, organizar (W5), guias (W5b).
- Mapa mental e os seis modelos (W6): a retrospectiva e o brainwriting são os quadros onde se vota e
  agrupa — a cena 9 pode partir de um deles.
- A barra de contexto por tipo de selecção (`Selected`, `style_groups`, os submódulos `board_bar_*`), o
  aviso pintado no quadro (`organize::paint_notice`), o `Overlay` (`overlay.rs`).
- A lei do controlo morto: um botão sem efeito na selecção de agora NÃO se pinta.

## §3 — Fase A: o ESTUDO (um documento antes do código)

Escreva `docs/MiroClone/05_facilitacao.md`, no molde do `04_planear_mapa_e_modelos.md` (veredito · como os
apps fazem, com FONTE · tabela lei → fonte → número · ⛔ recusas medidas · medir antes de limitar · a cena):

1. **Miro** (produto do dono), pela ajuda (API Zendesk — a busca SEM locale:
   `help.miro.com/api/v2/help_center/articles/search.json?query=…`; a listagem
   `/api/v2/help_center/en-us/articles.json?per_page=100&page=1..7`): *Timer* (onde aparece, som, somar
   tempo), *Voting* (votos por pessoa, o que se vota, os resultados, o que fica no quadro),
   *Stamps/Reactions* (os carimbos, onde se prendem, se andam com a nota), *Cluster* / *Sort by* (por
   etiqueta, por palavra, por cor: o desenho dos grupos — grelha? frame? o vão? o título?). Os GIFs e as
   imagens oficiais medem-se ao píxel como na W6 (estudo 04 §1.3): **o produto do dono decide a lei**.
2. **Oráculos que se CORREM** (§0.9 — nunca ler o fonte; triagem de licença por ARTEFACTO instalado):
   procure apps de quadro com voto/cronómetro/cluster que corram por script (o tldraw e o Excalidraw de
   `ferramentas/` não votam — confirme; procure outros com licença permissiva). Se nenhum corre, escreva-o
   como facto e a fonte passa a ser o Miro medido. O AGRUPAR por palavra é algorítmico: meça o do Miro
   sobre entradas NOSSAS (o que conta como «palavra», maiúsculas, plurais, palavras vazias) antes de
   inventar.
3. **A tabela lei → fonte → número**, incluindo o que é DOCUMENTO e o que é SESSÃO: os votos ficam no
   ficheiro? o cronómetro? (num ecrã só não há «os outros» — decida e escreva porquê).
4. **Medir antes de limitar (§0.0):** o custo de agrupar 100 / 1 000 / 10 000 notas (régua ao lado de
   `measure_mind_layout_cost.rs`, variantes no MESMO processo, intercaladas, o mínimo) e o do cronómetro a
   redesenhar (⛔ nenhum redesenho contínuo de um quadro grande sem a régua).

## §4 — Fase B: IMPLEMENTAR (sem voltar ao dono)

- Leis no `ph2d-board-edit` (módulos novos), o que for documento no `ph2d-board-model` (formato 7), o
  desenho no `ph2d-board-render`, a barra/teclas no `ph2d-editor-core` (submódulo novo), nada na shell.
  Texto pelo `ph2d-i18n` (inglês); nada de setas-glifo em frases (tofu); números de desenho com nome e
  `LITERAL-PX-OK` (o portão `no_magic_numeric` reprovou a W6 na 1.ª corrida por isso).
- **Cada lei num teste VERMELHO antes de a curar**; números da fonte TRANSCRITOS no cabeçalho do teste;
  a divergência ESCOLHIDA escrita no teste.
- Controlo novo na barra: o arnês `shell_click_bar` (`board_bar_pen_tests.rs`, `pub(super)`) e o gate
  «nenhuma zona de gesto da shell sobre um botão da barra»; uma janela nova da barra fecha as outras (a W6
  achou a galeria por cima da Nota — `board_bar_mind::sync_gallery`).
- **Cena 9** (`PH2D_BOARD_SMOKE=9`): um brainstorm facilitado. ⛔ Lições da W6: o 1.º quadro ENQUADRA O
  QUADRO INTEIRO NA JANELA e ignora a câmara da cena — a cena vive num frame com a margem do que os painéis
  e a barra de estado tapam (`smoke_mind.rs`, `MARGIN_X/Y`); o gate conduz cada lição na ORDEM REAL (a
  selecção herdada de uma para a outra: a lição 4 da W6 fechava o mapa inteiro); fotografe
  (`docs/Components/ferramentas/fotografa_cena.sh`) e LEIA a foto antes de mandar — três das quatro
  primeiras fotos da W6 tinham um defeito.

## §5 — Fecho da W7

`/pd-linha-fechar`: portão batched 1× sobre `2a925859d..HEAD` (ou o merge-base novo): `nextest-impacted`
com `PH2D_GPU=1`, `CARGO_BUILD_WARNINGS=deny` no check da workspace com `--all-targets`, o clippy do
`ship.sh` com `--keep-going` (o 1.º erro esconde os outros — medido na W6), `fmt`, `machete`, o
`ph2d-editor-core --test it` e a shell `--test it` inteiros, os censos da árvore combinada, o `typos`;
**auditoria de 2 lentes** (leis · costura/desenho/cena) ANTES da mutação — na W6 acharam 24 defeitos
reais, um deles um pânico; **prova de mutação** em cada lei nova (baseline verde, filtro com ≥ 1 teste,
restauro por `git checkout` + `touch`; os sobreviventes tapam-se e re-mutam-se); o handoff de integração
W7 (que substitui o da W6 no CLAUDE.md §5.1); `rm -rf target/*/incremental`; o binário `smoke` compilado
2× (a 2.ª em segundos, colada no handoff). Ao dono: o que ele consegue FAZER agora e o smoke em passos
numerados. **Não integre nem faça push.**
