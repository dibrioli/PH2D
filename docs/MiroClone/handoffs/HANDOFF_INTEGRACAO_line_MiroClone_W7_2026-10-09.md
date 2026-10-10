# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 + W5b + W6 + W7 «Facilitação» → `main` (2026-10-09)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem
> pusha. **Substitui** o [handoff W6](HANDOFF_INTEGRACAO_line_MiroClone_W6_2026-10-08.md) como o último da
> linha. W5, W5b e W6 (smoke do dono OK nas três) não foram integradas e a W7 continua por cima, na mesma
> branch — **integra-se tudo de uma vez**; para W5/W5b/W6 valem os handoffs deles. Briefing:
> [continuação W7](HANDOFF_CONTINUACAO_W7_2026-10-09.md). Estudo (as fontes, as imagens medidas, as leis
> F1–F7 · V1–V7 · S1–S4 · C1–C9 com fonte e número, as recusas, a régua):
> [`../05_facilitacao.md`](../05_facilitacao.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `2a925859d` — **o `main` não andou** no fecho (`git rev-parse main` = `2a925859d`) ⇒ `--ff-only` se ninguém aterrar antes |
| commits da W7 | `aaf7cc88c`..`a08bec3d0` (11) + este handoff; o total da linha: `git log --oneline 2a925859d..line/MiroClone` |
| diff da W7 | código: 55 ficheiros em `crates/`+`shells/`, +6 474 / −55; mais 13 ficheiros de dados/vendor (+10 104: o Snowball gerado e as fixtures do oráculo) |
| smoke | **por fazer pelo dono** (cena 9, §8) |

O que a W7 trouxe ao Quadro, num ecrã só: **cronómetro** (cartão no canto de cima à direita: ± de 1 até 10
min e de 5 acima, Play/Pause, +1m/+5m, ■ com confirmação, no zero treme e toca a campainha, minimiza
numa pílula), **votação** (votos por pessoa 1–99, «um voto por objecto», minutos que armam o cronómetro,
o `+`/`−` no canto de cada nota/forma, «I'm done» passa o ecrã à pessoa seguinte, «End for all» escreve
o resultado; o selo azul medido do Miro; top 3, «See all results», «New vote», «Clear results»),
**carimbos** (oito ícones Lucide com contagem, por baixo da nota; clicar +1, `Shift`+clicar −1) e
**agrupar** notas por cor, etiqueta ou palavra (cada grupo um frame, a grelha ⌈√n⌉ com as folgas medidas
nas imagens oficiais do Miro; também a partir de um frame seleccionado).

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | porque é aditivo |
|---|---|---|
| `ph2d-board-model` | **formato 7** (`FORMAT_VERSION` 6 → 7): `Org::votes: u32`, `Org::stamps: Vec<StampCount>` no FIM da caixa de organizar; `Stamp` (append-only); geometria do selo/`+`/fichas (`facilitate.rs`) | `legacy.rs::read_v6` lê o formato 6; fixture `fixtures/format_v6.bin` **gravada pelo build `6a9b47240`** (teste temporário, nunca bytes à mão) — gate `a_format_6_file_still_opens_with_no_votes_or_stamps`. ⛔ O `PROJECT_SCHEMA` (187) **não** se toca: o quadro é um blob com versão própria |
| `ph2d-board-words` (**crate nova**) | segmentação UAX#29 (`icu_segmenter` 2.3, já no lock via parley), palavras vazias do Snowball, a raiz | drop-crate; só o `ph2d-board-edit` a usa |
| `crates/ph2d-board-words/vendor/snowball` (**vendorizado**) | Snowball 3.1.1: o runtime Rust e os stemmers inglês/português **gerados pelo compilador oficial**, verbatim | o idioma do ADR-0123 (`deep_filter`): excluído do workspace (`Cargo.toml` raiz), `[workspace]` próprio, `rustfmt.toml` com `disable_all_formatting` (o `cargo fmt --all` alcança path deps e reescrevia código gerado), BSD-3 em `COPYING` + `VENDOR.md` + `THIRD_PARTY_LICENSES.md`; `cargo deny` verde |
| `ph2d-label-fold` | `case_fold` (o *folding* completo SEM tirar acentos, NFC) | função nova ao lado de `fold`, que não muda |
| `ph2d-editor-core` | `documents_fac.rs` (estado de SESSÃO: cronómetros, painéis, sessões estacionadas); `Documents::take_chime`; `board_nav/facilitate` (cartão e painel, filho da navegação); `board_bar_facilitate` (Timer/Voting na barra curta, Stamp/Cluster nas de notas e frames); 5 `IconId` (`LayoutGrid`, `Timer`, `Volume2`, `VolumeX`, `Vote`) + SVG Lucide em `docs/design/icons/` | variantes/ficheiros novos; `board_bar.rs` **cortado antes de acrescentar** (693 → 662 → 684: `swatch`/`is_current`/`doc_color` para `board_bar_look.rs`) |
| `ph2d-i18n` | `board_facilitate.rs` (módulo novo) | chaves novas `board.tool.timer/vote`, `board.fac.*`, `board.stamp.*`, `board.timer.*`, `board.vote.*`, `board.notice.*` (4), `board.smoke.fac.*` |
| `ph2d-app-audio` | `AudioSystem::play_chime` (dois dedilhados em quinta, 1,2 s, bus SFX) | método novo; ⛔ NÃO é um `UiSound` (a lei desse é «só o que a mão fez», opt-in): é um alarme armado pelo facilitador, desliga-se no altifalante do cartão |
| shell | `fase_pointer_subjects.rs` +7 linhas: drena `documents.take_chime()` e chama `play_chime` | composição; `the_shell_only_shrinks` verde |
| `Cargo.toml` raiz · `.typos.toml` | `exclude` do vendor · isenção das fixtures do oráculo, do vendor e das listas do Snowball (raízes como `abject` não são erros) | — |

**Contratos congelados (§6):** nenhum tocado — `git diff --name-only 6a9b47240..HEAD` não tem ficheiro de
`ph2d-nodegraph`, `ph2d-tool-runtime`, `ph2d-vector`/`vec-doc` nem `project_schema`; o diff não contém
`PROJECT_SCHEMA`; os gates `architecture_*contract_surface` correram verdes no nextest impactado.

## 3. Números MEDIDOS

- **A raiz** contra a `libstemmer` 3.1.1 instalada, sobre o vocabulário oficial do Snowball (42 649
  inglesas · 32 016 portuguesas): `rust-stemmers` 1.2.0 **57** inglesas diferentes, `waken_snowball` 0.1.0
  **17**, o Rust gerado pelo Snowball 3.1.1 **0** e **0**. Fixture com gate: 7 515 palavras
  (`stems_match_libstemmer`); segmentação = `Intl.Segmenter` do Node (ICU 78.3) nas 20 notas das
  entradas (`segments_match_icu`). Duas corridas iguais de cada oráculo.
- **O selo** (`vote_05` do help do Miro): Ø 52 px (1 voto) e 80 px (2) numa nota de 236 ⇒ 0,22 e 0,34 do
  lado; o `+` (`vote_03`) Ø 49 = 0,21 com o centro no canto. **Agrupar** (`cluster_01`/`_02`): ⌈√n⌉
  colunas; folgas 0,17 / 0,27; margens 0,30 · 0,32 · 0,43; entre frames 0,40 (estudo §1.2).
- **Régua** `crates/ph2d-board-render/tests/it/measure_facilitate_cost.rs` (`--release`, `load` 12–15):
  agrupar 10 000 notas 8,7 ms por cor · 8,3 por etiqueta · 79,9 por palavra (1 000: 0,77 · 0,75 · 4,31);
  desenhar 10 000 notas no limiar da leitura (letra 6 px) 0,71 ms de encode sem nada · 1,30 com selos em
  todas · **2,03** com três carimbos em todas (era 5,44 com o SVG relido por ficha — curado na mesma
  rodada, `stamp_path` em `OnceLock`). ⇒ **nenhum tecto** (estudo §4.1).

## 4. Portão batched (1× sobre `2a925859d..HEAD`, 3 passagens, a última verde)

| portão | resultado |
|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`, `PH2D_GPU=1`) | ✓ 17 146/17 146 |
| `cargo check --workspace --all-targets` com `CARGO_BUILD_WARNINGS=deny` | ✓ |
| clippy do `ship.sh` (`--keep-going`, `-D warnings`) | ✓ 0 avisos |
| `cargo fmt --all -- --check` | ✓ (e o vendor intacto) |
| `cargo machete` · `cargo deny --all-features check` | ✓ · ✓ (`snowball` BSD-3-Clause) |
| `ph2d-editor-core --test it` · `ph2d-host-desktop --test it` | ✓ 536 · ✓ 791 |
| `censos-da-arvore-combinada.sh` | ✓ 114/114, 12/12 censos correram |
| `typos` | ✓ |

Nomeados verdes: `shell_files_respect_hr18_loc_cap`, `workspace_src_files_under_loc_cap`,
`the_shell_only_shrinks`, `clamp_calls_in_ui_are_safe_or_justified`, `no_magic_numeric_in_widget_or_screens`,
`no_tofu_glyphs_in_ui_strings`, `enum_order_matches_svgs`. ⚠️ A 1.ª passagem apanhou: a `clamp` sem
`CLAMP-OK`, três números sem `LITERAL-PX-OK`, um `collapsible_if`, fmt, typos, e o `cargo fmt --all` a
**recusar-se** (o manifesto vendorizado subia até ao `Cargo.toml` do checkout principal); a 2.ª, o
`CLAMP-OK` na linha de cima (tem de ser NA linha) e uma seta `→` numa mensagem de teste (tofu).
O último commit (`a08bec3d0`, só texto i18n e a foto) correu `ph2d-i18n`, `ph2d-app-board` e `no_tofu`.

## 5. Auditoria de 2 lentes (antes da mutação) — 26 achados, todos curados com teste

Estudo §2.1. Os de mais peso: **o projecto carregado ou o quadro apagado não limpavam a facilitação** (os
`BoardId` recomeçam por projecto ⇒ «End for all» escreveria votos noutro documento) —
`loading_or_deleting_a_board_takes_its_facilitation`; **a lição 2 da cena 9 era FALSA** (a cena nasce com um
resultado e o painel dele não abria uma votação nova; o gate da cena chamava o editor, não o painel) —
«New vote» e `the_vote_lesson_runs_from_the_panel`; o relógio da votação acabava-a a meio de um arrasto
(o desfazer do gesto apagava os totais) — `the_clock_waits_for_the_gesture_to_end`; o cronómetro calava-se
com a aba Cena — `the_chime_rings_with_the_scene_tab_open`; um clique no mostrador do cartão caía no quadro
(fazia uma nota) — `a_click_on_the_card_never_reaches_the_board`; a casa do agrupar (antepassado comum,
travado, rodado), as fichas debaixo do selo, o painel «See all» a sair do ecrã, o cartão por baixo da
busca, os ± dos limites, o ■ que não largava o relógio, alvos apagados/escondidos.

## 6. Prova de mutação — 39/39

39 mutações nas leis novas (cronómetro 6, votação 6, cópia 1, carimbos 2, agrupar 9, modelo 3, palavras 4,
interface 7, cena 1), baseline verde e cada filtro com ≥ 1 teste, restauro por `git checkout` + `touch`.
36 sangraram à primeira; os 3 sobreviventes tapados e re-mutados (sangram): carimbar com uma nota
travada na selecção; a casa do agrupar com a 1.ª nota no subframe; e o `retain` dos relógios era uma
2.ª porta para o que o `Documents::remove` já faz — saiu, e o `forget` (a porta única), mutado, sangra.

## 7. Fixtures, instrumentos e o que vive versionado

- `crates/ph2d-board-model/fixtures/format_v6.bin` — gravado pelo build `6a9b47240`.
- `crates/ph2d-board-words/tests/fixtures/{raizes_libstemmer_3_1_1.tsv, segmentos_icu.json}` — saídas dos
  oráculos, com cabeçalho; scripts e entradas em `docs/MiroClone/ferramentas/facilitacao_oracle/`.
- `docs/MiroClone/ferramentas/facilitacao_miro_notas.md` — a ajuda do Miro lida (citações e URLs).
- `docs/MiroClone/ferramentas/cena9_foto.png` — a foto da cena 9 (3.ª, depois das curas).

## 8. Smoke (o comando exacto)

```sh
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=9 ./target/smoke/ph2d-host-desktop
```

A cena 9: o frame «Ideas» com sete ideias (duas com selos de uma votação anterior, 3 e 1; duas com
carimbos), o cartão do cronómetro aberto a 05:00 no canto de cima à direita, e cinco notas-lição na ordem
real (cronómetro · votar · turnos · carimbos · agrupar). O gate `every_facilitation_lesson_does_what_it_says`
conduz cada lição a dois zooms; `the_vote_lesson_runs_from_the_panel` conduz a 2.ª pelo painel.

## 9. O que fica ABERTO (baixo; nenhum é defeito de lei)

- Os `+`/`−` de uma sessão continuam clicáveis numa nota dentro de um frame visto de longe (o modo «só o
  título»): o editor não conhece o limiar do nível de detalhe (`Detail::frame_title_only_px`, do desenho).
  Cura: passar o limiar ao editor (`Metrics`) e filtrar o `vote_marks` — toca no construtor de `Metrics`
  em todos os testes.
- O título do frame de fora da cena 9 («Facilitate a brainstorm») fica debaixo da barra de ferramentas
  (o idioma da cena 8: a margem afasta as notas dos painéis, não o título).
- O título de um grupo por palavra mostra a forma em *case folding* («API» → «#api»).
- A votação ANÓNIMA entre pessoas em ecrãs diferentes e o modo privado: Etapa 2 (rede). O turno («I'm
  done») é a forma de um ecrã só.

## 10. Para o integrador

- Integra-se W5+W5b+W6+W7 juntas (handoffs W5/W5b/W6 para as delas). Base `2a925859d` = `main` ⇒ `--ff-only`.
- O `Cargo.toml` raiz ganha um `exclude` (o vendor do Snowball): ao fundir com outra linha que mexa no
  `exclude`, a lista **soma** (Mergiraf apaga remoções em arrays — confira por `git range-diff`).
- `.typos.toml`: três entradas novas no `extend-exclude`.
- CLAUDE.md §5.1: a entrada do Quadro aponta este handoff (feito nesta linha).

## 11. A máquina

`rm -rf target/*/incremental` feito no fecho (34 G do `debug`, 3,9 G do `smoke`); nenhum processo da linha a
correr (`pgrep -af 'ph2d|cargo|rustc'`).

## 11-bis. O binário do smoke

Compilado na worktree (a foto final da cena 9 construiu-o depois do último commit de código, `a08bec3d0`);
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, a 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.24s
```

## 12. Smoke do dono — OK (09/10)

*«smoke OK»* — a cena 9 (`PH2D_BOARD_SMOKE=9`): cronómetro, votação com turnos e resultado, carimbos e
agrupar por palavra a partir do frame «Ideas». Nenhuma correcção pedida. A próxima janela segue com a W8
([continuação W8](HANDOFF_CONTINUACAO_W8_2026-10-09.md)), por cima, na mesma branch.
