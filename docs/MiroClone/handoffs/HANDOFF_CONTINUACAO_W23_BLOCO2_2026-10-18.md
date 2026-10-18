# CONTINUAÇÃO — `line/MiroClone` W23 bloco 2 (rede ao vivo) fechado; o próximo é o bloco 3 «quadro» — 2026-10-18

> Leitor: a LLM da próxima janela. Lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) ·
> bloco 1: [HANDOFF_CONTINUACAO_W23_BLOCO1](HANDOFF_CONTINUACAO_W23_BLOCO1_2026-10-18.md). Base: `main` de 18/10 (W22c), linha
> rebaseada. Commits do bloco: `1457a96204` (A9) · `59078e5e60` (A12) · `0bbeae1cb8` (B8) · `49292d6941` (gate da mutação M6 do B8).
> Antes de construir, confira no CÓDIGO (CLAUDE.md §5.0). Não há integração nem push: só por ordem do Enio.

## 1. O que o bloco 2 fez

**A2 — já feito antes** (conferido no código): `Liveness{beat 2s, silence 10s}` em `crates/ph2d-board-net/src/host.rs`;
teste `o_silencio_derruba_quem_se_calou_e_o_sinal_de_vida_segura_quem_so_nao_fala` em `tests/it/ponta_a_ponta.rs`.

### A9 — fechar a sala larga a porta JÁ

MEDIDO antes (sonda em `door_tests`, removida): anfitrião sem ninguém / com um convidado ligado: porta livre em 3–5 ms
depois de `close` ou `drop`. **Com um aperto de mão a meio** (alguém ligou e nunca o acabou) `close()` prendia a porta
**2,0 s** (juntava as threads até `JOIN_DEADLINE` antes de largar o listener) e a sala reaberta caía na 47321. Um
convidado autenticado MUDO (escritor bloqueado no write) também prendia 2 s.

Cura: `Host::shut_doors` (`host.rs`): pára, fecha os fios ainda em aperto (os que não estão em `senders`), acorda as
portas, junta as threads de accept, limpa `doors` (os listeners caem) — ANTES do broadcast de adeus/join em `close` e
antes do adeus mDNS no `Drop`.

Gates (`door_tests.rs`): `fechar_a_sala_larga_a_porta_ja_mesmo_com_um_aperto_a_meio` (porta livre < 500 ms E `close`
volta < 500 ms) · `fechar_a_sala_larga_a_porta_ja_mesmo_com_um_convidado_mudo`. Mutações M1 (sem shutdown do aperto →
`close` espera 2,0 s), M2 (sem `doors.clear`→ porta 2,0 s), M3 (código antigo): todas vermelhas.

⚠️ A reordenação no `Drop` (portas antes do mDNS) **não tem gate**: os testes correm com mDNS desligado; medido que o
stop do mDNS leva ms em loopback.

Segunda metade da A9 («programa fechado a meio não manda adeus»): **NÃO é defeito a curar** — processo morto não envia;
o SO fecha os sockets, os convidados veem EOF, o redial é recusado → `room_is_gone` → «host left» com a cópia deles.
Gate já existente: `a_vanished_host_ends_the_guest_soon_and_its_old_code_never_locks_the_next_room` (`live_tests.rs`).
Fecho gracioso por SIGTERM/Ctrl+C foi considerado e **não feito** (cresce a shell + dependência nova para ~2 s de
diferença; a mensagem final é a mesma).

### A12 — entrada por link

(a) `ph2d_board_internet::LinkGuest` (`session.rs`, exportado em `lib.rs`): UM `Node` (mesma porta UDP) + UM
`Rendezvous` (`Arc`, criado preguiçosamente na thread ajudante, repetido se o socket falhou) por sessão de convidado.
`join_noting` = `LinkGuest::new(invite).join(..)`. App: `Route::Link { guest: LinkGuest, local }`
(`live_internet.rs`, construído no `JoinLink` de `live.rs`). Gates: `quem_volta_pelo_link_volta_pela_mesma_porta_e_o_mesmo_ponto_de_encontro`
(`session_tests.rs`, DHT de teste: o anfitrião vê o MESMO endereço de par depois de expulsar e voltar; `Arc::ptr_eq` do
rendezvous) e `coming_back_by_the_link_comes_from_the_same_port` (`live_link_tests.rs`, ficheiro novo).

(b) `every_way_a_link_fails_has_its_sentence_in_the_panel` (tabela `JoinError` → frase, incl. `link_old` e
`link_offline`; a rota LAN devolve `false`) e `a_link_with_another_rooms_key_says_the_link_is_old` (ponta a ponta em
127.0.0.1). `link_offline` não se provoca em 127.0.0.1: só a tabela unitária cobre a frase.

(c) Verificado **NÃO defeito**: o ramo de `connected()` que fecha sem `path_lost` só corre com o nó a fechar (o
`Weak` não sobe) — um `Dialer` vivo segura o `Arc<Inner>`, e `begin_stop` limpa os `dials` para o à espera acordar com
`BrokenPipe`; `streams().open(Bi)` não falha numa ligação nova (o servidor permite 1 bidi). Mutações M1–M5 vermelhas.

### B8 — escritas agrupadas a 30 Hz pela internet (e um eco curado)

MEDIDO antes: `crates/ph2d-app-board/src/live_pace_tests.rs` (`#[ignore]`). Retransmissor UDP contador entre convidado
e anfitrião, transporte internet local, arrasto de 120 posições a 60 Hz, variantes intercaladas no MESMO processo
(override `internet_pace_for_tests`), 3 rodadas idênticas cada. Comando:
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_SUITE="motivo: medição B8" bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-board --lib measure_what_a_drag_sends -- --ignored --nocapture`

| variante | sobe dgr/s | sobe B/s | desce dgr/s | desce B/s | latência mediana/p95 |
|---|---|---|---|---|---|
| antes (a cada imagem, com eco) | 64 | 6770 | 63 | 3830 | 0,4/0,7 ms |
| a cada imagem, sem eco | 62,5 | 6450 | 32 | 1030 | 0,4/0,7 |
| 33 ms, sem eco | 31 | 3280 | 31 | 1000 | 7,5/15 |

O histograma de tamanhos expôs um **defeito de eco**: a cada escrita de convidado o anfitrião mandava `Msg::Fac` (4 B)
a todos — `chair_writes` punha `dirty=true` em todo `Set` de Box/Text porque `Vault::overtaken` não dizia se tinha
removido algo. Cura: `overtaken -> bool` (`vault.rs`), `dirty |= ...`. Gates: `moving_a_note_does_not_send_the_facilitation_again`
(`fac_audit_tests.rs`) e, para o caminho da remoção, uma asserção nova em `a_newcomer_converges_when_a_clear_text_overtakes_the_hosts_sealed_one`
(a mutação M6 «overtaken sempre false» sobrevivia aos 22 testes fac até esta asserção — commit `49292d6941`).

**Ritmo** (`Pace` em `crates/ph2d-board-sync/src/session.rs`, partilhado por presença e escritas): o tecto é **em média**
— o próximo espaço conta a partir do que TERMINOU, não do instante de envio (com quadros de 16 ms e tecto de 33 ms a lei
antiga «desde o último envio» dava 20 Hz, a presença também); mudar a seleção reinicia a contagem da presença (não gasta
o próximo espaço); a 1.ª escrita depois de uma pausa sai logo; tiques vazios não gastam o espaço; `Host::close` reinicia
o ritmo para o adeus levar o que estava retido. API `write_every(ms)`/`writes_every_ms()` no `Host` e no `Guest` síncronos.
App: `INTERNET_WRITE_EVERY_MS = SEND_EVERY_MS` (33) para o convidado por link (`join_by`) e para o anfitrião quando a
porta da internet abre (`internet_tick`); a LAN continua a cada imagem (0).

Troca: o lado de lá vê 30 posições/s em vez de 60 e +7,5 ms de latência mediana pela internet; LAN igual. Nota: a sala
do anfitrião com a porta da internet aberta também ritma as escritas para os convidados da LAN (é UM broadcast).

Gates: `paced_writes_leave_once_per_period_and_the_last_position_always_arrives` ·
`closing_the_room_sends_what_the_pace_was_holding` · `a_moving_cursor_goes_out_at_thirty_hertz_with_sixty_hertz_frames`
(`session_tests.rs`) · `only_the_internet_paces_the_writes` (`live_link_tests.rs`). Mutações M1–M5, M7, M8 vermelhas;
M6 vermelha depois da asserção nova.

## 2. Prova de fecho e o que um merge pode partir

- `cargo fmt --all` limpo; clippy `-D warnings --all-targets` limpo em `ph2d-board-net`, `ph2d-board-internet`,
  `ph2d-board-sync`, `ph2d-app-board`.
- Testes por nome verdes; filtro `live` no app-board 24/24 (5 ignorados); sync `replica` 20/20, `fac` 22/22; internet
  `session` 10/10.
- Suíte (`PH2D_SUITE=onda nextest-impacted`, a cadência deixou correr): 19 308/19 334 corridos, **0 falhas**; 2 TIMEOUT em `ph2d-tool-painter` (`macro_track::tutorial::tests::pular_a_pincelada_e_esperar_nao_cumpre_nenhuma`, `it presets_de_fabrica::a_largura_e_a_prometida`) com `loadavg` 138–143 (outras linhas); sozinhos por nome, 2/2 verdes (`loadavg` 33 → 22). A crate não tem diff nesta linha ⇒ família de [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md). O resto (26) não chegou a correr: a corrida parou no 1.º `error: test run failed`.
- Superfície de colisão: `ph2d-board-net/src/host.rs` (`shut_doors`, ordem em `close`/`Drop`) ·
  `ph2d-board-internet/src/session.rs` (`LinkGuest`) + `lib.rs` (export) · `ph2d-app-board/src/live.rs` (+1 linha) ·
  `live_internet.rs` (forma de `Route::Link`, ritmo) · `ph2d-board-sync` `session.rs` / `session_chair.rs` / `vault.rs`
  (`Pace`, `overtaken`). Quem reordenar o `close` do `Host` parte os dois gates de `door_tests.rs`.
- Nenhum contador de schema/registo/ADR mexido. Versão do protocolo inalterada (nenhum formato de fio mudou).

## 3. Fotos e smoke

Nada visível na tela mudou neste bloco ⇒ **sem fotos**.

**Smoke ao dono (Enio):**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build -p ph2d-host-desktop --profile smoke && PH2D_BOARD_SMOKE=12 ./target/smoke/ph2d-host-desktop`
2. No cartão «Live» a sala está aberta e o endereço termina em `:47320`. Clique «End session» e logo a seguir «Open to the network»: o endereço continua a terminar em `:47320`.
3. Internet no mesmo computador, em dois terminais (ambos depois do `cd` acima): `PH2D_BOARD_INTERNET=local PH2D_BOARD_SMOKE=22 ./target/smoke/ph2d-host-desktop` e `PH2D_BOARD_INTERNET=local ./target/smoke/ph2d-host-desktop`. Na 2.ª janela: Live → cole o link no campo do código → «Join by link». Arraste uma forma numa janela: ela anda na outra, sem saltos, e para exatamente onde você largou.
4. Deu errado se: o endereço passa a `:47321` depois de reabrir, ou a forma para na outra janela num sítio diferente de onde você largou.

## 4. ABERTO

- **Perguntas ao dono, sem resposta:** (1) YouTube Premium; (2) se «Rascunho/Final» troca também a letra importada.
- **Sem gate:** o `Drop` do `Host` (portas antes do mDNS), por os testes correrem sem mDNS.
- **Não feito por decisão:** fecho gracioso do processo (SIGTERM/Ctrl+C) para mandar adeus.
- **Bloco 3 «quadro» (próximo):** A7, A10, A13, B1, B3, B4, B5 (B2 já feito). Depois: 4 = B6, B7, B10, B11 · 5 = B9, A11 ·
  6 = B12, B13, B14 (com `/pd-feature`). Testes que precisam do dono (furo real pela internet, Windows/Mac reais) ficam
  fora desta onda.
