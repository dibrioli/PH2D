# CONTINUAÇÃO — `line/MiroClone` W11 «O quadro ao vivo»: CONSTRUÍDA, FECHO POR FAZER (2026-10-10)

> Para a janela que FECHA a W11. Rota `/pd-linha-assumir`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. W5…W11 estão por integrar na mesma branch. Último handoff de integração:
> [W10](HANDOFF_INTEGRACAO_line_MiroClone_W10_2026-10-10.md). Briefing de origem:
> [HANDOFF_CONTINUACAO_W11](HANDOFF_CONTINUACAO_W11_2026-10-10.md) (a escolha do dono: rede local primeiro,
> internet depois sem refazer).

## 1. O que está feito e comitado (confira com `git log --oneline -20`)

Commits «W11 (n/n)» de `7b114dab1` (3.1) a `75c80bb5e` (11/n), mais `41def4806` (o botão «Rascunho ↔ Final»
alcança as NOTAS, decisão do dono 10/10) e `aa2a8d7c9` (ADR-0181 + estudo [`08_ao_vivo.md`](../08_ao_vivo.md)).

1. **Oráculo corrido** — [`ferramentas/colab_oracle/`](../ferramentas/colab_oracle/) (Excalidraw: app `4c00f31`
   + excalidraw-room `03ff435`; tldraw sync 5.5.2): 23 casos, duas corridas; comparação caso a caso no `08_ao_vivo.md`.
2. **ADR-0181** — funde por REGISTO (campo), apagar é registo, desfazer só repõe o que ninguém mexeu, ids
   sorteados na sessão, presença à parte, duas drop-crates. §Medições preenchida.
3. **Modelo** (`ph2d-board-model`): `Field` (registos, com gate de cobertura); `History` guarda o estado
   depois do gesto (c10–c12); sessão do documento (`set_session`, `SESSION_ID_BIT`, `key_between` = porta
   única das chaves com cauda, `FracKey::between` aceita empate, caderno `take_changes`, `set_mark`);
   `Put` normaliza org vazio; equipa com `MessageId`/`VersionId`, `Team::mint`, `Board::set_session`.
   `FORMAT_VERSION` 10 com leitor do 9 (fixture `crates/ph2d-board-model/fixtures/format_v9.bin`).
4. **`ph2d-board-sync`** (pura, sem IO): réplica (Lamport por registo, carimbos como fichas, retrato
   compactado a 193 B/elemento, volta depois de queda); equipa (fios/mensagens/versões com lápide, «lido» por
   pessoa); sessão Host/Guest (protocolo versionado, presença ≤ 30 Hz interpolada, «trazer todos»); testes
   de convergência (40 sementes P2P, 25 em estrela com quedas).
5. **`ph2d-board-net`**: SPAKE2 + ChaCha20-Poly1305, código de 6, travão 5/30 s, só LAN, porta 47320, mDNS;
   30 testes + dois processos.
6. **Ligação**: `ph2d-editor-core::documents::live` (`LiveRequest` Host/Join/Discover/Leave/Summon,
   `LiveView`) e `ph2d-app-board::live` (chamada de dentro do `io::service` — a shell não cresceu). Ponta a
   ponta em 127.0.0.1: `a_guest_joins_by_code_and_both_see_each_others_work`, `a_wrong_code_is_said_and_nothing_opens`
   (`crates/ph2d-app-board/src/live_tests.rs`).
7. **Interface**: botão «Live» na fila de cima (com Present e Comments), painel em 4 estados (Off · Joining ·
   Hosting · Guest), avatares que se clicam para SEGUIR, selo «Following» com ×, cursores e selecção alheios
   só no quadro da sala, cores Person0..7 (`board_live*.rs`, `ph2d-i18n/src/board_live.rs`). **Nunca vista
   por olhos — só por testes.**
8. **Medições** (ADR §Medições): latência de arrasto na LAN real com 2/4/8 pessoas, mediana 0,47–0,64 ms,
   máx ≤ 1,19 ms; bytes ao arrastar 1/10/50 formas = 60 B / 409 B / 1 969 B por imagem. Os dois
   instrumentos são testes `#[ignore]` que se correm à mão.

## 2. POR FAZER, pela ordem

1. **A CENA 12** — `PH2D_BOARD_SMOKE=12`. O roteador está em `crates/ph2d-app-board/src/smoke.rs` e vai
   até 11: acrescente o `mod` filho (`#[path = "smoke_live.rs"]`), o braço `12 =>`, `max_level: 12`, a mensagem
   «1..=12» do braço `_`, a linha 12 na tabela do cabeçalho. Modelo: `smoke_team.rs` (+ `smoke_team_tests.rs`).
   Abre um quadro com formas e NOTAS como ANFITRIÃO e ensina a entrar com a 2.ª janela. A 2.ª janela é OUTRO
   PH2D: o código aparece no painel Live. Firewall: o `ufw` desta máquina está ligado — para um 2.º
   COMPUTADOR o dono libera a porta uma vez, `sudo ufw allow 47320/tcp`; duas janelas no MESMO computador não
   precisam (e o painel já avisa). Gate da cena como os das W8–W10 (a cena arma o que a lição diz; vê-lo
   vermelho antes da cura). Cuide para a cena não ensinar o contrário do que acontece (CLAUDE.md §5.0).
2. **FOTOGRAFAR as DUAS janelas** com `docs/Components/ferramentas/fotografa_cena.sh` antes de ir ao dono.
   ⛔ Nunca `spectacle` em `kwin --virtual` (fotografa o ecrã REAL; use `import -window`). A foto da W10
   apanhou uma lição que mandava ao botão errado — espere o mesmo aqui: os 4 estados do painel, o cursor
   alheio e o selo «Following» nunca foram vistos.
3. **`/pd-linha-fechar`**: portão batched 1× sobre o diff acumulado (`nextest-impacted`, `check`/clippy
   `-D warnings` `--all-targets`, fmt, typos, machete, deny, censos, `file_loc_caps`, `the_shell_only_shrinks`);
   auditoria de 2 lentes (uma é a REDE: o que se perde, duplica, diverge; a outra a INTERFACE do painel);
   prova de mutação (agente `mutacao`); handoff de INTEGRAÇÃO W11 que SUBSTITUI o da W10 (W5…W11);
   CLAUDE.md §5.1 (o Quadro passa a «ao vivo na rede local», `PH2D_BOARD_SMOKE=1..12`, troca o link do
   handoff; ≤ 700 B); `bash scripts/agent-loop-profile.sh`; `rm -rf target/*/incremental`; binário smoke
   quente (2.ª saída colada).

## 3. PONTAS conhecidas — decida no fecho (cura ou registo com o porquê)

- (a) **A UI das versões ainda renomeia/apaga por ÍNDICE** (`board_team_panels.rs`: `renaming_version`,
  `Confirm::Version(BoardId, usize)`). A sync já funciona por id (`VersionId`); o padrão certo é o das
  mensagens, que passaram a id. Com versões a mudar de lugar na rede, um índice aponta a outra versão.
- (b) **Sem keepalive TCP**: um convidado mudo só cai quando uma escrita passa 10 s (a presença a 30 Hz
  cobre-o enquanto o rato mexe).
- (c) As escritas saem a cada imagem (60 Hz); na internet, agrupar a 30 Hz.
- (d) A lista de achados e de pessoas no painel corta-se ao que cabe.

## 4. Fora de escopo

A votação anónima e o modo privado são a **W12** — não abrir. A internet vem depois e não pode exigir
refazer o protocolo (o transporte está atrás de uma porta em `ph2d-board-net`).

## 5. Lembretes medidos nesta linha

- `cargo-test-narrow.sh` já se embrulha no `ph2d-run.sh` (embrulhá-lo outra vez dá «0 passaram»); um erro de
  compilação aparece como «0 testes»; `cargo check --all-targets` com `CARGO_BUILD_WARNINGS=deny` à parte.
- O tecto de 700 linhas conta os `*_tests.rs`. Igualdade exacta de `f64` entre perfis falha por 1 ULP.
- O guarda de recursos recusa um `heredoc` com `cargo` dentro.
- A prova de mutação acha sobreviventes que a auditoria não vê (W10: 2 de 17) — tape-os e re-mute.
- `chacha20`/`poly1305` estão a `opt-level 2` no perfil dev (a opt-0 a cifra é ~100× mais lenta: 104 ms
  → ~20 s a entrar num quadro de 57 MiB) — não o desfaça.

## 6. Ao dono

PT-BR, curto, sem jargão; smoke em passos numerados (comando completo com `cd` · onde clicar · o que tem de
acontecer · como saber que deu errado). Não integra, não pusha.
