# HANDOFF DE CONTINUAÇÃO — `line/MiroClone` → W6 «Planear» (2026-10-08)

> Para a PRÓXIMA JANELA desta linha. Antecessor: [integração W5b](HANDOFF_INTEGRACAO_line_MiroClone_W5b_2026-10-08.md)
> (W5 + W5b fechadas; **smoke do dono OK nas duas**, 08/10). Plano: [`../02_plano.md`](../02_plano.md) §3 W6
> (§6: as recusas medidas — leia antes de propor desenho). Pesquisa: [`../01_pesquisa_miro_excalidraw.md`](../01_pesquisa_miro_excalidraw.md)
> §4 (mind map) e §8. **Ordem do dono (08/10): «próxima janela segue com W6».** Decisões técnicas são suas
> (padrão-ouro); só uma de PRODUTO que o estudo não resolva volta ao dono, em linguagem dele e no fim.

## §0 — Identidade e estado da árvore

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`. A W5 e a W5b **não foram integradas**
  (o dono escolheu seguir para a W6); a W6 continua POR CIMA, na mesma branch.
- Base `2a925859d`. **Antes de tudo** (MODELO_TROCA §FASE 1): `git rev-parse main` — se o `main` andou
  (alguém integrou a linha entretanto), `git cherry main HEAD | grep -c '^+'`: `0` ⇒ `git reset --keep main`;
  `>0` ⇒ `git range-diff main...HEAD` e só depois `git rebase main`.
- Formato do quadro: `FORMAT_VERSION = 5`. Campo novo no documento (ex.: a relação pai/filho do mapa
  mental, o «recolhido») = **6**, com leitor do 5 e fixture GRAVADA pelo build antigo (idioma de
  `legacy.rs` + `fixtures/format_v4.bin`; ⛔ nunca bytes copiados à mão — recusa medida no plano §6).
  O `PROJECT_SCHEMA` (187) não se toca.
- Cena de smoke seguinte: **8** (conta-se em `crates/ph2d-app-board/src/smoke.rs`, hoje `max_level: 7`).
- ⚠️ Tectos de LOC (700) perto: `ph2d-board-edit/src/lib.rs` **693**, `gesture.rs` 675, `notes.rs` 598,
  `organize.rs` 593. **Corte antes de acrescentar** — o mapa mental nasce num módulo próprio
  (`mindmap.rs` + `mindmap_tests.rs`), nunca a engordar estes.

## §1 — O que a W6 é (plano §3)

1. **Mapa mental**: `Tab` filho, `Enter` irmão, setas navegam, ramos recolhem, **layout automático**.
2. **Modelos**: retrospectiva, brainstorm, kanban, mapa mental, fluxograma, mapa de jornada.
3. **O dono vê:** começar um planeamento a partir de um modelo e expandi-lo pelo teclado.

## §2 — O que já existe e NÃO se reconstrói

- Setas presas ao centro/ponto fixo, curva do Miro, cache de rota por diferença (W2) — os ramos do mapa
  são SETAS do quadro (ou a mesma lei de rota), não um desenho novo.
- Notas e formas com texto rico, crescer com o texto, `Tab` na nota em escrita = a seguinte à direita
  (W3, `board_keys.rs:92`) — ⚠️ **colisão de tecla**: no mapa mental o `Tab` é «filho». Resolva pelo
  idioma do Miro (o `Tab` do mapa só actua num NÓ do mapa) e gate a fronteira dos dois.
- Frames, grupos, ordem, alinhar/distribuir, minimapa, busca, nível de detalhe (W5); guias e encaixe
  (W5b: `guides.rs`, `snap.rs`, `03_guias_e_alinhamento.md`). Um nó do mapa arrastado continua a
  encaixar pelas leis da W5b — ou o layout automático manda nele, e as guias calam-se: decida e gate.
- O `taffy` já está na árvore (`ph2d-vec-layout`, flexbox+grid) — ⚠️ é layout de CAIXAS, não de ÁRVORE.
  **Meça antes** se exprime o layout de um mapa mental (CLAUDE.md §5.0: «meça se a composição já o
  exprime») — senão, a lei clássica de árvore (Reingold–Tilford / «tidy tree») é sua, com o oráculo.

## §3 — Fase A: o ESTUDO (um documento antes do código)

Escreva `docs/MiroClone/04_planear_mapa_e_modelos.md`, no molde do `03_guias_e_alinhamento.md` (veredito ·
como os apps fazem, com FONTE · tabela lei → fonte → número · ⛔ recusas medidas · medir antes de
limitar · a cena):

1. **Miro** (produto do dono): o help do mapa mental e dos modelos (API Zendesk: `help.miro.com/api/v2/help_center/articles/search.json?query=…`
   — ⚠️ a forma com `en-us` dá 404, aprendido na W5b); teclas, layout (direcções, espaço entre ramos,
   equilíbrio esquerda/direita), recolher, arrastar um ramo, apagar, cores por ramo, o que cada modelo traz.
2. **Oráculos que se CORREM** (§0.9 — nunca ler o fonte; triagem de licença por ARTEFACTO instalado):
   candidatos de mapa mental com licença permissiva para correr por script — `mind-elixir`, `simple-mind-map`,
   `markmap` (confira a licença no `LICENSE` instalado de cada um) — ao lado dos oráculos que já existem em
   `ferramentas/` (Playwright, o idioma de `excalidraw_oracle/` e `tldraw_oracle/`). Árvores NOSSAS
   (1 raiz, 3 filhos; desequilibrada; 50 nós; texto longo num nó; recolher a meio), gravar as posições de
   cada nó como fixtures `saidas/mapa_*.json` + `meta.json`, duas corridas byte a byte iguais. O
   Excalidraw não tem mapa mental (só `Ctrl+seta`, já medido na W2).
3. **A tabela lei → fonte → número**: o layout (vão entre irmãos, entre níveis, de que lado nasce o
   filho, re-equilíbrio), as teclas (`Tab`, `Enter`, setas, `Shift+Enter`, `Delete` apaga o ramo, `Esc`),
   recolher/expandir e o que se vê recolhido, arrastar um nó para outro pai, o layout automático
   desligável no pai (o Miro), modelos: o que cada um contém (frames? notas? setas?) e como se insere
   (onde nasce, a escala, o nome).
4. **Medir antes de limitar (§0.0):** o custo do layout a 100 / 1 000 / 10 000 nós por tecla (régua ao lado
   de `measure_snap_cost.rs`, variantes no MESMO processo, intercaladas, o mínimo) — nenhum `MAX_*` de nós
   sem a tabela.

## §4 — Fase B: IMPLEMENTAR (sem voltar ao dono)

- Leis no `ph2d-board-edit` (módulo novo), desenho no `ph2d-board-render`, a barra/teclas no
  `ph2d-editor-core` (`board_keys.rs`, `board_bar*`), nada na shell. Os modelos: um catálogo no
  `ph2d-board-model` ou `ph2d-app-board`, com o texto pelo `ph2d-i18n` (inglês, como o resto da interface).
- **Reproduza cada lei num teste VERMELHO antes de a curar**; números do oráculo TRANSCRITOS no cabeçalho
  do teste (idioma de `organize_tests.rs`/`guides_tests.rs`), e a divergência ESCOLHIDA escrita no teste.
- Controlo novo na barra (inserir modelo, recolher): arnês `shell_click_bar` + o gate «nenhuma zona de
  gesto da shell sobre um botão da barra»; zero texto fora do i18n; nada de `←→↑↓` em frases (tofu);
  números de desenho com `LITERAL-PX-OK` em constantes com nome, uma por linha.
- **Cena de smoke 8** (`PH2D_BOARD_SMOKE=8`): um mapa mental começado e os modelos à mão. ⛔ **Lição da
  W5b:** a cena tem de ENSINAR o que acontece — escreva o gate que a conduz pelo editor real com as
  medidas da interface (o idioma de `smoke_guides_tests.rs::every_lesson_teaches_its_own_alignment`),
  e **fotografe** (`docs/Components/ferramentas/fotografa_cena.sh`). ⚠️ O enquadrar das cenas centra na
  janela inteira e os painéis laterais tapam as bordas (handoff W5b §10.1): desenhe a cena para o meio.

## §5 — Fecho da W6

`/pd-linha-fechar`: portão batched 1× sobre `2a925859d..HEAD` (ou sobre o merge-base novo, se o `main`
andou): `nextest-impacted` com `PH2D_GPU=1`, clippy do `ship.sh` `-D warnings`, `fmt`, `ph2d-editor-core
--test it` inteiro, a shell `--test it`, os censos da árvore combinada, o `typos` do projecto inteiro
(exclua as `saidas/` de um oráculo novo no `.typos.toml`, como as dos outros dois); **auditoria de 2
lentes** (correcção das leis · costura/desenho/cena) ANTES da mutação; **prova de mutação** em cada lei
nova (4 controlos — e um arrasto/tecla que acaba no sítio exacto não prova o encaixe: pare antes); o
handoff de integração W6 (que diz o que mudou desde o da W5b e o substitui no CLAUDE.md §5.1);
`rm -rf target/*/incremental`; o binário `smoke` compilado 2× (a 2.ª em segundos, colada no handoff). Ao
dono: o que ele consegue FAZER agora e o smoke em passos numerados (comando com `cd` · onde clicar · o que
tem de acontecer · como saber que deu errado). **Não integre nem faça push.**
