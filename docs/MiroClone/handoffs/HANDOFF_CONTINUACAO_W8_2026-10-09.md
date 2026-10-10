# HANDOFF DE CONTINUAÇÃO — `line/MiroClone` → W8 «Equipa assíncrona e saída» (2026-10-09)

> Para a PRÓXIMA JANELA desta linha. Antecessor: [integração W7](HANDOFF_INTEGRACAO_line_MiroClone_W7_2026-10-09.md)
> (W5 + W5b + W6 + W7 fechadas; **smoke do dono OK nas quatro**, a W7 a 09/10). Plano: [`../02_plano.md`](../02_plano.md)
> §3 W8 (§4: o oráculo da W8 é o `.excalidraw` de ida e volta; §6: as recusas medidas — leia antes de propor
> desenho). Pesquisa: [`../01_pesquisa_miro_excalidraw.md`](../01_pesquisa_miro_excalidraw.md).
> Decisões técnicas são suas (padrão-ouro); só uma de PRODUTO que o estudo não resolva volta ao dono, em
> linguagem dele e no fim.

## §0 — Identidade e estado da árvore

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`. W5, W5b, W6 e W7 **não foram integradas**;
  a W8 continua POR CIMA, na mesma branch.
- Base `2a925859d`. **Antes de tudo** (MODELO_TROCA §FASE 1): `git rev-parse main` — se andou,
  `git cherry main HEAD | grep -c '^+'`: `0` ⇒ `git reset --keep main`; `>0` ⇒ `git range-diff main...HEAD`
  e só depois `git rebase main`.
- Formato do quadro: `FORMAT_VERSION = 7` (W7: `Org::votes`, `Org::stamps`). Campo novo no documento
  (comentários, versões) = **8**, com leitor do 7 e fixture GRAVADA pelo build actual ANTES de mexer no
  modelo (o idioma de `legacy.rs::read_v6` + `fixtures/format_v6.bin`; ⛔ nunca bytes à mão). O
  `PROJECT_SCHEMA` (187) não se toca: o quadro é um blob com versão própria.
- Cena de smoke seguinte: **10** (`crates/ph2d-app-board/src/smoke.rs`, hoje `max_level: 9`).
- ⚠️ **Tectos de LOC (700, os `*_tests.rs` CONTAM) — corte antes de acrescentar:** `ph2d-board-render/src/lib.rs`
  **693**, `board_bar.rs` **690**, `gesture.rs` **688**, `ph2d-board-route/src/lib.rs` 678, `organize.rs`
  (edit) 620, `lib.rs` (edit) 602, `notes.rs` 601. O novo nasce em módulos próprios (`comments.rs`,
  `versions.rs`, `present.rs`, `export.rs`, `excalidraw.rs` + `*_tests.rs`) e a barra entra por chamadas de
  UMA linha (o idioma de `board_bar_facilitate.rs`); o cromado novo como filho da navegação (o idioma de
  `board_nav/facilitate`).

## §1 — O que a W8 é (plano §3)

1. **Comentários** em fio, presos a um elemento ou a um ponto, com autor e «resolvido», gravados no ficheiro.
2. **Versões nomeadas** dentro do ficheiro; restaurar como CÓPIA (o idioma do Miro).
3. **Apresentação** pelos frames, na ordem do painel de Frames.
4. **Exportar** PNG/SVG/PDF do quadro, de um frame ou da selecção.
5. **Importar e exportar `.excalidraw`** — traz quadros de quem já usa o Excalidraw, e fecha o laço do
   oráculo (ida e volta medida).
6. ⛔ Fora: tudo o que precisa de várias pessoas AO MESMO TEMPO (cursores, presença, menções ao vivo) ⇒
   Etapa 2. O «autor» de um comentário num ecrã só é o nome que o artista escreve (decida e escreva porquê).
7. **O dono vê:** um quadro com comentários e uma versão guardada, apresentado frame a frame, exportado, e
   um ficheiro do Excalidraw aberto.

## §2 — O que já existe e NÃO se reconstrói

- O oráculo **Excalidraw 0.18.1** instalado e corrido por script (`docs/MiroClone/ferramentas/excalidraw_oracle/`,
  README com a triagem de licença por artefacto): já produz `.excalidraw`/`exportToSvg` sobre entradas
  nossas — é a porta da ida e volta (§4 do plano). As saídas novas vão para `excalidraw_oracle/saidas/`
  com `meta.json`; cada corrida vira gate.
- Frames com nome, ordem no painel de Frames (`frames_in_panel_order`), «enquadrar» (`fit_all`, `go_to`),
  o nível de detalhe de longe (`Detail`) — a apresentação parte daqui.
- O desenho do quadro (`ph2d_board_render::paint`) e a régua da placa (`VelloPass` do produto,
  `measure_gpu_raster_cost.rs`) — o PNG sai daqui; ⚠️ confira no CÓDIGO o que o app já tem para SVG/PDF/PNG
  (export de outros módulos) antes de acreditar numa ausência (CLAUDE.md §5.0).
- Links entre elementos e o ícone de link (W5), o aviso pintado no quadro (`organize::paint_notice`), a
  barra de contexto por selecção, o cromado do canto (`board_nav/facilitate`: cartão + painel, o fundo que
  toma o clique, a coluna que se desvia do painel de Frames e da busca).
- A sessão por quadro (`documents_fac.rs`) e a lei de que o estado de um quadro sai com ele em
  `Documents::remove`/`replace` (os `BoardId` recomeçam por projecto — achado da auditoria W7).

## §3 — Fase A: o ESTUDO (um documento antes do código)

Escreva `docs/MiroClone/06_equipa_e_saida.md`, no molde do `05_facilitacao.md` (veredito · como os apps
fazem, com FONTE · imagens oficiais medidas ao píxel · tabela lei → fonte → número · ⛔ recusas medidas ·
medir antes de limitar · a cena):

1. **Miro** (produto do dono), pela ajuda (API Zendesk — a busca SEM locale:
   `help.miro.com/api/v2/help_center/articles/search.json?query=…`; a listagem
   `/api/v2/help_center/en-us/articles.json?per_page=100&page=1..7`): *Comments* (onde se prendem, o fio,
   resolver, o que se vê no quadro, filtrar), *Board history / Versions* (nomear, restaurar como cópia),
   *Presentation mode* (ordem, transições, o que se esconde — o help já diz «Apresentação esconde os
   títulos», `miro_organizar_notas.md` §10), *Export* (formatos, escala, área, o que sai e o que não sai —
   «only widgets inside of frames are exported»). Imagens oficiais medidas ao píxel (o idioma do §1.2 do
   estudo 05): **o produto do dono decide a lei**.
2. **Oráculos que se CORREM** (§0.9): o Excalidraw para o `.excalidraw` (o esquema dele, a ida e volta:
   o nosso → `.excalidraw` → Excalidraw → `exportToSvg` vs o nosso desenho; o deles → o nosso). Procure
   oráculos para PDF/SVG (o que o sistema já tem instalado — triagem por ARTEFACTO, `pacman -Qi`).
3. **A tabela lei → fonte → número**, incluindo o que é DOCUMENTO (comentários, versões: ficheiro) e o que
   é SESSÃO (a apresentação).
4. **Medir antes de limitar (§0.0):** o custo de guardar uma versão de um quadro grande (10 mil / 100 mil
   elementos: bytes e ms), de exportar (PNG à escala, SVG, PDF) e de ler/escrever `.excalidraw` grande —
   régua ao lado de `measure_facilitate_cost.rs`, variantes no MESMO processo, intercaladas, o mínimo.

## §4 — Fase B: IMPLEMENTAR (sem voltar ao dono)

- Leis no `ph2d-board-edit` (módulos novos), o que for documento no `ph2d-board-model` (formato 8), o
  desenho no `ph2d-board-render`, a barra/painéis no `ph2d-editor-core` (submódulos novos), a conversão
  `.excalidraw` numa crate própria se crescer (drop-crate), nada na shell além de composição (abrir/gravar
  ficheiro, se o diálogo for da shell — meça o que a shell já oferece).
- Texto pelo `ph2d-i18n` (inglês, `board_*` em módulo próprio); nada de setas-glifo nem `▶` em frases
  (tofu — o gate `no_tofu_glyphs` apanha até mensagens de teste); números de desenho com `LITERAL-PX-OK` na
  LINHA; `clamp` com `CLAMP-OK` NA linha.
- **Cada lei num teste VERMELHO antes de a curar**; números da fonte TRANSCRITOS no cabeçalho do teste; a
  divergência ESCOLHIDA escrita no teste.
- Controlo novo na barra/cromado: o arnês `shell_click_bar` / `shell_click_id` (`board_bar_pen_tests.rs`),
  o gate «nenhuma zona de gesto da shell sobre um botão», e o fundo do painel registado (toma o clique).
- **Cena 10** (`PH2D_BOARD_SMOKE=10`). ⛔ Lições da W6/W7: o 1.º quadro enquadra o quadro INTEIRO na
  janela — a cena vive num frame com a margem do que os painéis tapam (`MARGIN_X/Y`); o gate conduz cada
  lição na ORDEM REAL e **pelo caminho do ECRÃ** onde a lição o manda (a W7 tinha uma lição falsa que o
  gate não via porque chamava o editor e não o painel); as notas-lição não crescem sozinhas — o texto tem
  de caber; fotografe (`docs/Components/ferramentas/fotografa_cena.sh`) e LEIA a foto antes de mandar —
  na W7 as três fotos apanharam três defeitos.

## §5 — Fecho da W8

`/pd-linha-fechar`: portão batched 1× sobre `2a925859d..HEAD` (ou o merge-base novo): `nextest-impacted`
(`BASE=…`, `PH2D_GPU=1`), `CARGO_BUILD_WARNINGS=deny` no check da workspace com `--all-targets`, o clippy do
`ship.sh` com `--keep-going`, `cargo fmt --all -- --check` (corre desde a W7: o vendor do Snowball tem
`[workspace]` e `rustfmt.toml` próprios), `machete`, `cargo deny`, o `ph2d-editor-core --test it` e a shell
`--test it` inteiros, os censos da árvore combinada, o `typos`; **auditoria de 2 lentes** (leis ·
costura/desenho/cena) ANTES da mutação — na W7 acharam 26 defeitos reais; **prova de mutação** em cada lei
nova (baseline verde, filtro com ≥ 1 teste, restauro por `git checkout` + `touch`; os sobreviventes
tapam-se e re-mutam-se — e uma 2.ª porta para a mesma lei SAI em vez de ganhar teste); o handoff de
integração W8 (que substitui o da W7 no CLAUDE.md §5.1); `rm -rf target/*/incremental`; o binário `smoke`
compilado 2× (a 2.ª em segundos, colada no handoff). Ao dono: o que ele consegue FAZER agora e o smoke em
passos numerados. **Não integre nem faça push.**
