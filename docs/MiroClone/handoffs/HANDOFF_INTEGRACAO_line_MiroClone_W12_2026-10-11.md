# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W12 → `main` (2026-10-11)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem
> pusha. A W5…W11 já está no `main` (10/10); a W12 é a única coisa por integrar. Mecanismo:
> [ADR-0182](../../architecture/decisions/0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md) (o cofre do
> anfitrião) · oráculo [`vote_private_oracle/`](../ferramentas/vote_private_oracle/README.md) · leis do Miro
> [`facilitacao_miro_notas.md`](../ferramentas/facilitacao_miro_notas.md) §6 — **não se repetem aqui**.
> Briefing: [continuação W12](HANDOFF_CONTINUACAO_W12_2026-10-11.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `8f20c60de` (o `main` de 11/10) — se o `main` não andou, `--ff-only` |
| commits | 8 sobre a base (`git log --oneline 8f20c60de..HEAD`), `cf254345d` … `1e7f5966a` |
| diff em `crates/` | `git diff --shortstat 8f20c60de..HEAD -- crates/`: 50 ficheiros, +4 788 / −256 (o resto do diff total são as saídas do oráculo e as fotos) |
| smoke | **por fazer** (§8); a cena 13 foi vista por fotografia, 11 estados com duas janelas (§7) |

O que a W12 trouxe (escolha do dono 11/10: *«sim!»* às duas peças juntas):

1. **Votação anónima entre pessoas** — cada um vota no seu computador pelo painel de sempre; o boletim vai SÓ ao
   anfitrião; quem acabou e o tempo são públicos, os votos usados por pessoa só para quem iniciou; o fim (relógio
   do anfitrião ou «End for all») escreve os totais no quadro de todos de uma vez.
2. **Modo privado** — botão Private (olho fechado) na barra da facilitação; o texto das notas novas/editadas e o
   TAMANHO delas vão só ao cofre do anfitrião; os outros vêem a nota com rabiscos fixos e o olho fechado;
   «Anonymous names», «Allow anyone to turn off», «Let each reveal theirs», «Reveal all my sticky notes»,
   «Turn off private mode for all» com 5 s de contagem; o painel abre e fecha a todos na virada (Miro PM).
3. **Protocolo 2** (`PROTOCOL` 1 → 2): `Msg` cresce só no fim (`Sealed`, `Ballot`, `Fac`, `Ask`, `Denied`); um par
   da W11 é recusado com a frase de sempre.
4. Cena 13 `PH2D_BOARD_SMOKE=13`.

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | porque é aditivo |
|---|---|---|
| `ph2d-board-model` | `VoteSetup` + `VOTES_MAX/DEFAULT` mudam de `ph2d-board-edit` para `facilitate.rs` (viajam na rede); `Board::version_bytes` sai de `save_version` | a edição reexporta-os (`pub use`), nenhum chamador muda; os bytes da versão são os mesmos; **`FORMAT_VERSION` intocado** (o `VoteSetup` não vai a ficheiro) |
| `ph2d-board-sync` | `fac.rs`, `vault.rs`, `replica_sealed.rs`, `session_chair.rs` novos; `PROTOCOL` 2; `Msg`/`Event`/`Refusal` crescem no fim; `Host::fac(now, &Board)` | crate da linha; o índice das variantes antigas não muda |
| `ph2d-board-edit` | `Editor::set_unread`, `text_editable`, `Notice::Private` (`ALL` 9 → 10), `VoteSession::live/ballot`, recusas em `begin_text`/`vote_click`/`toggle_mark`, `vote_marks` | crate da linha; variante nova no FIM do enum |
| `ph2d-board-render` | `RenderCache.unread`/`mine_private`, `private.rs` (o olho Lucide `eye-closed` + rabiscos) | campos novos com `Default`; nada muda sem eles |
| `ph2d-editor-core` (tela do quadro) | `documents_live.rs` (`LiveFac/LivePrivate/LiveVote`, 7 `LiveRequest` novos, `LiveOther.anonymous`), `documents_fac.rs` (estado da votação ao vivo e do painel privado), `board_facilitate*.rs` (+`board_facilitate_live.rs`, `board_private_paint.rs`), `board_bar*.rs` (item `Private`, `TOOLBAR` 17 → 18), `board_keys.rs`, `board_live*.rs`, `board_team_panels.rs` (`column_open`) | ficheiros do quadro; `FacButton::ALL` 27 → 34 (variantes no fim; o id é `SALT ^ índice`) |
| `ph2d-i18n` | `board_private.rs` novo (+ `lib.rs`: `mod` e `.or_else`), `board.rs` (`board.notice.private`), `board_facilitate.rs` (`board.vote.left_one`) | chaves novas |
| `docs/architecture/decisions/` | ADR **0182** + o `README.md` regenerado por `scripts/adr-index.sh` | o número SOMA entre linhas — reconte contra o `main` do dia |

## 3. Símbolos que podem colidir — `collision-surface.sh` (REFERÊNCIA, nunca prova)

```text
SUPERFÍCIE DE COLISÃO — line/MiroClone contra main
  merge-base 8f20c60de   ·   6 commit(s)   ·   109 arquivo(s)
▸ SCHEMAS            PROJECT_SCHEMA 188 (base 188) · tripla (188, 13, 22) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 20 — intocados
▸ REGISTRO           ph2d-ecs 107 · ph2d-render 108 · ph2d-script 108 — intocados
▸ CONTRATO CONGELADO ph2d-nodegraph/src/node.rs intocado · ph2d-editor-core/src/tool.rs intocado
▸ ADR                esta linha cria 0182 (último no disco 0182) — reconte contra o main do dia
▸ Cargo.lock         nenhum pacote externo novo
▸ MARCADORES         nenhum
▸ TETOS DE LOC       (na 1.ª corrida) fac_tests.rs 900/700 — partido depois: 663 + fac_audit_tests.rs + fac_converge_tests.rs
```

Mesmo-símbolo provável com outra linha: `LiveRequest`/`LiveView` (se outra linha tocou o ao vivo), `TOOLBAR`
e `FacButton::ALL` (listas por índice), `Notice::ALL`, a cadeia `.or_else` do `ph2d-i18n/src/lib.rs`.

## 4. Contratos congelados

Nenhum: `crates/ph2d-nodegraph/src/node.rs` e `crates/ph2d-editor-core/src/tool.rs` intocados (`collision-surface.sh`);
o contrato do vetor não foi tocado (`git diff --name-only 8f20c60de..HEAD | grep vector` → vazio).

## 5. Portão batched (3 passagens sobre `8f20c60de..HEAD`) e o que só o `ship.sh` apanha

| | 1.ª (`4e83949d3`) | 2.ª (`98ca39df7`) | final |
|---|---|---|---|
| `BASE=8f20c60de scripts/nextest-impacted.sh` | 17 907 / 17 911 | 17 922 / 17 923 | **17 926 / 17 926** (297 s, `1e7f5966a`) |
| clippy `--all-targets -D warnings`, 7 crates | ✓ | ✗ 2 (`large_enum_variant`, `type_complexity`) → curados | ✓ |
| `file_loc_caps` da shell | ✓ 4/4 | ✓ 4/4 | — |
| `censos-da-arvore-combinada.sh` | ✓ 117/117 | ✓ 117/117 | — |
| fmt · typos · `adr-index.sh --check` · `cargo machete` | ✗ ✗ ✗ · — | ✗ ✓ ✓ ✓ | fmt ✓ |

Vermelhos da 1.ª e a cura: tecto de 700 (session.rs 868 → `session_chair.rs`; replica.rs → `replica_sealed.rs`;
live.rs → `live_fac.rs`; board_facilitate_paint.rs → `room_session` no `fac_live`) · **o censo `node_id_collisions`
cegou**: o `mod live` de TESTE (o filho do `board_bar_facilitate_tests`) fazia o `screens/hero/live.rs` de produção
ler-se como teste — renomeado `fac_live` (e o de produção também) · o vão entre cartões pergunta
`section_gap_px()` · `datas` em `normaliza.mjs`.

**Só o `ship.sh`**: nenhum `#[cfg(target_os` no diff (`git diff 8f20c60de..HEAD | grep -c 'cfg(target_os'` = 0); o
clippy passou sem a feature `ph2d-spike/bevy_ecs` (estas crates não a têm — a 1.ª corrida com ela o cargo recusa).

## 6. Auditoria de 2 lentes e prova de mutação (cada cura com o gate visto VERMELHO)

**Lente REDE** (o que vaza antes do tempo) — 5 achados, todos curados com gate em `fac_audit_tests.rs`:
uma VERSÃO guardada em modo privado levava o texto selado comprimido (o gate de bytes crus não a via) · a nota
CRESCIA para os outros (o comprimento) · o desfazer/negrito de quem não lê trocava o texto da autora no cofre ·
um voto anterior congelava o boletim · quem entrava depois de uma corrida divergia; + a soma do boletim dava a
volta. **Lente INTERFACE** — 3 defeitos e 7 cosméticos curados com gate em `board_facilitate_live_tests.rs`,
`board_private_paint.rs`, `live_fac.rs`: Ctrl+B numa nota escondida · votação de UM ecrã aberta antes da sala
(três botões mortos) · o avatar anónimo prometia «Follow» · «Someone» · «n of m done» · aviso a quem iniciou ·
o painel noutro quadro · a letra que trocava a ferramenta · textos (régua com controlo) · a coluna que saía do
ecrã · literais.

**Prova de mutação** (agente `mutacao`, 22 mutações): 19 sangraram; **3 sobreviveram e ganharam gate** — a
corrida da virada (M13 → `a_clear_text_written_before_hearing_of_the_mode_goes_to_the_vault`), o selado perdido
no fio (M14 → `a_sealed_write_lost_on_the_wire_is_resent_to_the_vault`), o Live que tapava o painel privado
(M22 → `the_live_panel_never_covers_the_private_panel`); re-mutadas, as três sangram agora.

## 7. Números, fixtures e instrumentos

- **Oráculo** (`docs/MiroClone/ferramentas/vote_private_oracle/`): Scrumlr v5.4.2 (MIT, `6bae2131…`) em 127.0.0.1
  sem docker nem root (Go 1.26.9 BSD-3 · nats-server 2.14.4 Apache-2.0 · Postgres 18.4 portátil); **23 casos × 2
  corridas iguais** depois de normalizar (comparador com controlo de 2 diferenças injectadas). Instalação em
  `~/Apps/scrumlr-oracle/` (~1,5 GB, fora do repo).
- **Miro medido** (help PM 2026-09-08, VO 2026-09-15): selo do olho 49/392 do lado, a 12/392 do canto; rabiscos
  130×20/392 (`ph2d-board-render/src/private.rs`).
- **Gates**: `ph2d-board-sync` **45** testes da lib (+1 medição ignorada; os de `fac::tests` + `audit` + `converge`), render 2,
  tela 10 (`fac_live` + `outside_the_room_board…`), vista 1, duas janelas 1
  (`the_private_scene_hides_until_the_reveal_and_votes_without_peeking`, rede real em 127.0.0.1, ~5,2 s).
- **Fotos** (`fotos_dos_estados_da_cena_13`, `#[ignore]`, à mão com a placa): 11 estados em
  [`ferramentas/cena13_estados/`](../ferramentas/cena13_estados/). As fotos acharam 5 defeitos que nenhum gate via
  (o Live tapava o painel; o painel não abria/fechava a todos; 3 textos cortados; «1 votes left») — curados, e o
  1.º ganhou gate na prova de mutação.

## 8. Smokes — o comando exacto (duas janelas no MESMO computador, como a cena 12)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_BOARD_SMOKE=13 ./target/smoke/ph2d-host-desktop      # janela 1 (a Ana): a sala já aberta
PH2D_BOARD_SMOKE=1  ./target/smoke/ph2d-host-desktop      # janela 2 (a Bia): Live → «Private retro» → código → Join
```

## 9. O que fica ABERTO

- **Smoke do dono da W12** (e o da W11, que segue por fazer: a descoberta e a firewall num 2.º computador).
- Decisões registadas, não defeitos: do lado de quem não lê, a busca `Ctrl+F`, o `row_text` dos resultados e a
  exportação vêem o texto ANTIGO de uma nota editada em privado (nunca o novo); com «Anonymous names» os nomes
  continuam na lista de pessoas do painel Live, nos avisos «X joined» e nas linhas `used` de quem iniciou (o ADR §8
  cobre cursores, selecções e seguir); o item Private da barra acende pelo painel, não pelo modo.
- A identidade do par é autodeclarada no `Hello` (o modelo de confiança da W11: quem tem o código é de casa) —
  um par adulterado podia passar-se por outro (`RevealMine`, `used`). Fechar a ABA do anfitrião (não «End
  session») larga a sala sem `close()` (comportamento da W11). Mudar a forma de uma nota para outra não tem ecrã.
- De W11 §9, inalterados.

## 10. Para o integrador

- Ordem: uma linha, 8 commits; `main` = merge-base ⇒ `--ff-only`; senão `git cherry main HEAD` e rebase.
- Conflitos prováveis (palpite por tipo de ficheiro): `ph2d-i18n/src/lib.rs` (a cadeia `.or_else`), `board_bar.rs`
  (`TOOLBAR`), `docs/architecture/decisions/README.md` (gerado: regenere com `scripts/adr-index.sh`), e o número
  do ADR 0182.
- Depois de integrar: `scripts/ship.sh`. CLAUDE.md §5.1 (Quadro) aponta este handoff e `PH2D_BOARD_SMOKE=1..13`.

## 11. A máquina e o binário do smoke

Nenhum processo desta linha vivo no fecho (os `cargo` vivos são de `line-PainterWatercolor` e da Luz 2D).
`rm -rf target/*/incremental`: reclamados ~33 GB (`debug/incremental` 29 G + `smoke/incremental` 3,6 G; `ci-test` vazio).
`agent-loop-profile.sh` (20 sessões): ✗ paralelismo 1,13/passo · ✗ `test`:`check` 2,9× · ✗ edições pela `Edit` 31 %
(as edições multi-trecho foram por `python3` com `assert` de contagem) · ✗ contexto relido 385 mil · ✓ respostas/sessão
172 · ✓ contexto inicial 62 mil.

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, duas vezes; a 2.ª:

```text
▸ linha line_miroclone · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.31s
```
