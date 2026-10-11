# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W12 + W13 → `main` (2026-10-11)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> **Substitui** o [handoff da W12](HANDOFF_INTEGRACAO_line_MiroClone_W12_2026-10-11.md) (que segue válido para o que é
> só da W12 — §2/§6/§7 dele não se repetem aqui): W12 e W13 integram-se juntas, numa vez. Mecanismo da W13:
> [ADR-0183](../../architecture/decisions/0183-o-historico-das-votacoes-e-um-instantaneo-da-equipa.md) · oráculo
> [`vote_private_oracle/`](../ferramentas/vote_private_oracle/README.md) (casos `h01`–`h03`) · Miro
> [`facilitacao_miro_notas.md`](../ferramentas/facilitacao_miro_notas.md) §7. Briefing:
> [continuação W13](HANDOFF_CONTINUACAO_W13_2026-10-11.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `8f20c60de` (o `main` de 11/10) — se o `main` não andou, `--ff-only` |
| commits | 22 sobre a base (`git log --oneline 8f20c60de..HEAD`): W12 `cf254345d` … `4e0283c19` (9), a continuação W13 `a78d56b3f`, a W13 `02f3fc596` … `38cd3d860` (10) e este handoff |
| diff em `crates/` | `git diff --shortstat 8f20c60de..HEAD -- crates/`: 96 ficheiros, +7 756 / −681 (só a W13: 71 ficheiros, +3 059 / −516) |
| smoke | W12: **OK do dono** (11/10). W13: **por fazer** (§8); a cena 14 foi vista por fotografia, 11 estados (§7) |

O que a W13 trouxe (escolha do dono 11/10, depois do smoke OK da W12: *«o histórico das votações, como o Miro»* + as
duas pontas abertas da W12):

1. **O histórico das votações** — cada votação acabada (de um ecrã ou da sala) é uma SESSÃO da equipa do quadro:
   instantâneo dos totais, «Voting N», quando acabou, quem a iniciou, quantos votaram. O painel ganha os separadores
   «New session» · «Results» (o do Miro): «Show on the board» (o selo é o da sessão mostrada), a lista da mais nova à
   mais velha com página, abrir uma sessão (a linha de uma nota apagada diz «(deleted)»), renomear (duplo clique no
   título), apagar (o lixo pede o 2.º clique). «Clear results» sai. Não vai na cópia do quadro, nem numa versão, nem
   se exporta.
2. **As pontas da W12**: com «Make names anonymous» nenhum nome de OUTRA pessoa chega ao ecrã (lista e avatares do
   painel Live, quem acolhe, quem iniciou a votação, os votos usados; «Someone» nos avisos); quem não lê uma nota
   escondida não lhe vê o texto antigo em lado nenhum (busca `Ctrl+F`, linhas do resultado, exportação PNG/SVG/PDF
   com o marcador, `.excalidraw` sem o texto).
3. **Formato 11** (`FORMAT_VERSION` 10 → 11) e **protocolo 3** (`PROTOCOL` 2 → 3).
4. Cena 14 `PH2D_BOARD_SMOKE=14`.

## 2. Foundational / partilhado tocado (W13) — e porque é aditivo

| onde | o quê | porque é aditivo / seguro |
|---|---|---|
| `ph2d-board-model` | `poll.rs` (`Poll`, `PollId`, `Team::{add_poll, poll, polls_newest_first, rename_poll, remove_poll, show_poll, shown_poll, label_number, adopt_old_votes}`); `Team.polls`/`Team.shown`; **sai** `Org::votes`, `Element::votes/set_votes`, `Field::Votes`; `FORMAT_VERSION` 11 + `legacy_v10.rs` (`ElementV10`, `read_v10`); os leitores 7–9 levam os votos antigos para uma sessão; `BoardSet::duplicate` larga o histórico | crate da linha. **O formato muda mas todos os anteriores leem-se** (fixture `format_v10.bin` gravada pelo build `02f3fc596` ANTES da mudança + o gate `a_format_10_file_opens_and_its_votes_become_one_shown_session`; 7, 8 e 9 continuam com os gates deles). O que saiu tinha consumidores só no Quadro (`cargo check --workspace --all-targets` verde) |
| `ph2d-board-sync` | `TeamWrite::{Poll, PollName, PollGone, Shown}` e `Key::{PollName, Shown}` no FIM; `TeamState.polls_gone`; `Host::set_wall`; o fim da votação da sala escreve UMA sessão; `Vault::tally` devolve também quantos votaram; `PROTOCOL` 3 | variantes no fim; um par da W12 é recusado com a frase de sempre (nunca funde às cegas) |
| `ph2d-board-edit` | `Editor::end_vote() -> Option<(totais, votantes)>` (fora do desfazer); `clear_votes` sai; `results(doc, &Poll)`/`vote_range(doc, &totais)`; `Editor::unread()` | crate da linha; os chamadores são todos do Quadro |
| `ph2d-board-render` | o selo lê `Team::shown_poll` (cache `VoteRangeCache` pela revisão e pela sessão) | idem |
| `ph2d-board-export` | `paint_page(…, unread, ts)` e `png::render(…, unread, ts)` ganham o conjunto das notas por ler | todos os chamadores no diff (testes com o conjunto vazio = o comportamento de antes) |
| `ph2d-board-excalidraw` | `Import.votes` (os votos de um `.excalidraw` antigo) | campo novo; o escritor já não escreve `org.votes` → **o oráculo do Excalidraw foi corrido de novo** (W8, W9, W10: só mudou o que vinha de `votes`) |
| `ph2d-editor-core` (tela do quadro) | `board_facilitate_history.rs` + `_history_paint.rs` novos; `VotePanel::{Results{poll,all}, History}`; `FacButton::ALL` 34 → 41 (`VoteClear` sai, 8 no fim); `Fac.{history_from, history_page, history_rows, renaming_poll, confirm_poll}`; `ids::BOARD_POLL_NAME_INPUT`; `board_nav::find(…, unread)`; `Documents::import_board(…, old_votes)`; a coluna da facilitação pára acima dos botões do canto | ficheiros do quadro; o id de `FacButton` é `SALT ^ índice` (as filas da lista usam a fatia depois de `ALL`) |
| `ph2d-i18n` | `board.vote.*` (17 chaves novas; `board.vote.clear` sai), `board.smoke.history.*`, `board.private.anonymous_name` | chaves novas; a lição 2 da cena 9 diz «New session» |
| `docs/architecture/decisions/` | ADR **0183** (e a 0182 da W12) + o `README.md` regenerado | o número SOMA entre linhas — reconte contra o `main` do dia |

## 3. Superfície de colisão — `collision-surface.sh` (REFERÊNCIA, nunca prova)

```text
SUPERFÍCIE DE COLISÃO — line/MiroClone contra main
  merge-base 8f20c60de   ·   19 commit(s)   ·   205 arquivo(s)
▸ SCHEMAS            PROJECT_SCHEMA 188 (base 188) · tripla (188, 13, 22) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 20 — intocados
▸ REGISTRO           ph2d-ecs 107 · ph2d-render 108 · ph2d-script 108 — intocados
▸ CONTRATO CONGELADO ph2d-nodegraph/src/node.rs intocado · ph2d-editor-core/src/tool.rs intocado
▸ ADR                esta linha cria 0182 0183 (último no disco 0183) — reconte contra o main do dia
▸ Cargo.lock         nenhum '+name' novo
▸ MARCADORES         nenhum nos arquivos da linha
```

Mesmo-símbolo provável com outra linha: `FacButton::ALL` (lista por índice), a cadeia `.or_else` do
`ph2d-i18n/src/lib.rs` (W12), `LiveRequest`/`LiveView`, o `README.md` dos ADR (gerado).

## 4. Contratos congelados e schemas

Nenhum tocado: `crates/ph2d-nodegraph/src/node.rs` e `crates/ph2d-editor-core/src/tool.rs` intocados; `git diff
--name-only 8f20c60de..HEAD | grep -E 'vector|Cargo.lock|project.rs'` → vazio; `PROJECT_SCHEMA` 188 = base. O
`FORMAT_VERSION` do quadro (11) é a versão própria do blob dos quadros dentro do `.ph2dproj` (o idioma do
`TimelineDoc`), não o `PROJECT_SCHEMA`. `#[cfg(target_os` no código do diff: 0.

## 5. Portão batched (3 passagens sobre `8f20c60de..HEAD`)

| | 1.ª (`4d290a78d`) | 2.ª (`78bbddb81`) | final (`38cd3d860`) |
|---|---|---|---|
| `BASE=8f20c60de scripts/nextest-impacted.sh` | 17 943 / 17 944 | 17 947 / 17 950 | **17 953 / 17 953** (252 s de nextest, `loadavg` 49–58) |
| clippy como o `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | ✗ `type_complexity` (render) | ✗ `dead_code` (cena 14) | ✓ (0 avisos) |
| `cargo fmt --all -- --check` | ✗ 78 hunks | ✓ | ✓ |
| `file_loc_caps` da shell | ✓ 4/4 | ✓ 4/4 | ✓ 4/4 |
| `censos-da-arvore-combinada.sh` | ✓ 117/117 | ✓ 12/12 censos | ✓ 117/117 (12/12 censos) |
| `cargo machete` · `adr-index.sh --check` | ✓ · ✓ | ✓ · ✓ | ✓ · ✓ (188 ADR) · typos ✓ |
| `cargo check --workspace --all-targets` | ✓ | — | — |

Vermelhos e cura: o `3.0` da abertura sem `LITERAL-PX-OK` (`no_magic_numeric`); `type_complexity` → `VoteRangeCache`;
**o sorteio de convergência `every_peer_ends_with_the_same_board_whatever_the_network_does` apanhou a 1.ª cura do «Voting N»
repetido** (renumerar e GRAVAR na fusão dependia da ordem de chegada) → `Team::label_number`, uma função do conjunto
que nunca se grava; `dead_code` no `Lessons.board` da cena 14 → lido pelo gate. Os dois
`g4_o_custo_*_nao_depende_do_numero_de_luzes` de `ph2d-light2d-gpu` (outra linha) reprovaram a `loadavg` 57 e passaram
sozinhos — flake de carga. Na passagem final passaram.

## 6. Auditoria de 2 lentes e prova de mutação (cada cura com o gate visto VERMELHO)

**Lente DADOS** — 4 achados, curados com gate: o anonimato lia-se no INÍCIO da votação da sala (ligado a meio, o nome
ficava) → `a_vote_ended_under_anonymous_names_keeps_no_name` (com o controlo sem a opção) · a votação de UM ecrã acabada
na sala guardava o nome com «Anonymous names» → `a_single_screen_vote_under_anonymous_names_keeps_no_name` · dois pares
com o mesmo «Voting N» → `two_votes_ended_at_once_never_share_a_number` · os votos de uma nota apagada de um ficheiro antigo
→ o ADR-0183 §9 diz que só os vivos (e `only_the_living_bring_their_old_votes`).
**Lente INTERFACE** — 5 achados, curados com gate em `board_bar_history_tests.rs`: o toggle pintado sem sessões (controlo
morto) · a fila lida por índice no CLIQUE (uma sessão da rede entre a pintura e o clique abria outra) → as filas são os
ids que a pintura mostrou · o lixo armado sobrevivia a trocar de quadro · o separador «New session» aceso repunha a
abertura · a lição 3 da cena 14 prometia «a lista mostra-o» sem a lista à vista.
**Fotos dos estados** (antes das lentes) — 5 defeitos que nenhum gate via, curados com gate: a lista ordenava pelo relógio
(sessões sem data caíam por baixo da mais velha) → pelo número · a página da lista passava por baixo dos botões do canto
(Live) · a data «2 d a…» cortada · o lixo armado mudo (só a dica) → «Click the bin again to delete» · «Votes per per…» (da
W7, no painel tocado). E a régua de largura apanhou «Show results on the board» a não caber → «Show on the board» (a frase
do Miro fica na dica).
**Prova de mutação** (agente `mutacao`, 20 mutações nas leis da W13): 16 sangraram; **3 sobreviveram e ganharam gate** — os
votos dos apagados (M6 → `only_the_living_bring_their_old_votes`), a nota por ler na exportação (M18 →
`an_unread_note_is_exported_without_its_text`), o selo com uma votação aberta (M19 →
`the_badge_is_the_shown_sessions_and_hides_during_a_vote`); re-mutados, os três sangram (o teste corre e falha, sem erro de
compilação). M7 (o filtro de `polls_gone` no `Shown`) sobrevive por ser defesa dupla: o `settle` já desliga a mostrada apagada.

## 7. Números, fixtures e instrumentos

- **Oráculo** (Scrumlr v5.4.2, MIT, já instalado pela W12): +3 casos `h01`–`h03` → **26 casos × 2 corridas iguais**. Lei
  medida que o ADR usa: o Scrumlr RECONTA o resultado a partir dos boletins vivos (apagar a nota mais votada tira-a do
  resultado antigo, `3 → 1`), guarda a cancelada sem resultado, não apaga nem renomeia (405/404), e o `INIT` de quem entra
  depois leva os **boletins crus** da última votação «anónima» (`{VOT2, n1, user:B}` ×2) — por isso o nosso é instantâneo.
- **Miro** (VO `updated_at` 2026-09-15 + 2 figuras em `capturas_miro/vo_*.png`): separador Results, toggle no topo, lista
  «Voting N · data» da mais nova, a sessão aberta com «04 Sep, 12:34 by <quem>», lápis e lixo, os selos da sessão aberta.
- **Oráculo do Excalidraw** corrido de novo (W8 `corre_esquema`, W9 e W10 `corre_fidelidade`): 14 ficheiros de saída; a árvore
  JSON sem `votes` é igual nas três entradas (só shas, `semente` e o cenário embutido mudam).
- **Fixture** `crates/ph2d-board-model/fixtures/format_v10.bin` (691 B, build `02f3fc596`).
- **Gates da W13**: modelo `poll_tests` 6 + `only_the_living…` · sync `replica_tests` +4 (convergência a dois, apagar vence
  renomear, mostrar uma apagada, a ordem pelo número, «Voting N» a dois) + o sorteio de 40 sementes com as 4 operações das
  sessões + `fac_tests` (a sala grava UMA sessão com quem iniciou e votantes) + `a_vote_ended_under…` · tela
  `board_bar_history_tests` 10 + `a_row_never_paints_a_hidden_or_a_deleted_note` · app `the_history_scene_does_what_its_lessons_say`
  (rede real 127.0.0.1, duas janelas, 0,3 s) + `anonymous_names_never_reach_the_screen` · render, export, excalidraw 1 cada.
- **Fotos** (`fotos_dos_estados_da_cena_14`, `#[ignore]`, à mão com a placa): 11 estados em
  [`ferramentas/cena14_estados/`](../ferramentas/cena14_estados/).
- **Custo não medido** (ver §9): o selo passou de ler `Org::votes` a uma procura num `BTreeMap` por elemento, só com uma
  sessão mostrada; a régua `measure_marks_paint_cost` (`#[ignore]`, `--release`, placa) foi ajustada mas não corrida.

## 8. Smokes — o comando exacto

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_BOARD_SMOKE=14 ./target/smoke/ph2d-host-desktop      # o histórico (e a sala já aberta para a 2.ª janela)
PH2D_BOARD_SMOKE=1  ./target/smoke/ph2d-host-desktop      # 2.ª janela: Live → «Vote history» → código → Join
PH2D_BOARD_SMOKE=13 ./target/smoke/ph2d-host-desktop      # a W12 (votar entre pessoas e modo privado), já com OK do dono
PH2D_BOARD_SMOKE=9  ./target/smoke/ph2d-host-desktop      # a votação de um ecrã: o resultado agora vai para Results
```

## 9. O que fica ABERTO

- **Smoke do dono da W13** (cena 14). O da W11 (descoberta e firewall num 2.º computador) continua por fazer.
- A régua de custo do selo (`measure_marks_paint_cost`, 10 mil notas com selo) não foi corrida em `--release` com a placa
  depois de o selo passar a ler a sessão mostrada (uma procura num `BTreeMap` por elemento desenhado, só com sessão mostrada).
- Decisões registadas, não defeitos: apagar a sessão de maior número deixa reusar o «Voting N»; o «Voting N» de duas sessões
  acabadas ao mesmo tempo em pares diferentes pode mudar de número quando uma delas se apaga (`label_number` é do conjunto);
  o fim de uma votação não se desfaz (o lixo é o caminho, ADR-0183 §6); o `fit` mínimo da coluna (3 filas) ainda pode passar
  do chão numa janela muito baixa (da W7).
- Do handoff da W12 §9: a identidade do par autodeclarada no `Hello`; fechar a ABA do anfitrião sem «End session».

## 10. Para o integrador

- Ordem: uma linha, 22 commits; `main` = merge-base ⇒ `--ff-only`; senão `git cherry main HEAD` e rebase.
- Conflitos prováveis: `ph2d-i18n/src/lib.rs` (a cadeia `.or_else` da W12), `board_bar.rs` (`TOOLBAR`, W12),
  `docs/architecture/decisions/README.md` (gerado: `scripts/adr-index.sh`), os números dos ADR 0182/0183.
- Um `.ph2dproj` gravado por este build (formato 11) não abre num build do `main` de antes: integrar de uma vez.
- Depois de integrar: `scripts/ship.sh`. CLAUDE.md §5.1 (Quadro) aponta este handoff e `PH2D_BOARD_SMOKE=1..14`.

## 11. A máquina e o binário do smoke

Nenhum processo desta linha vivo no fecho; nada desta linha na placa (`fuser -v /dev/dri/*`). Os servidores do
oráculo (Scrumlr, nats, Postgres) pararam pelo `stop.sh` (por PID).
`rm -rf target/*/incremental`: reclamados **39 GB** (`debug/incremental`).
`agent-loop-profile.sh` (20 sessões): ✗ paralelismo 1,14/passo · ✓ respostas/sessão 206 · ✗ `test`:`check` 2,6× · ✗
edições pela `Edit` 30 % (as multi-trecho foram por `python3` com `assert` de contagem) · ✗ contexto relido 402 mil · ✓
contexto inicial 62 mil.

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, duas vezes (1.ª: 40,9 s); a 2.ª:

```text
▸ linha line_miroclone · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.31s
```
