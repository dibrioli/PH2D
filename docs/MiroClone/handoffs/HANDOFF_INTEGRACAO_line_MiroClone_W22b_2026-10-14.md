# INTEGRAÇÃO — `line/MiroClone` W22b: o quadro ao vivo PELA INTERNET (2026-10-14)

> Leitor: o integrador e a próxima LLM da linha. Decisões: [ADR-0187](../../architecture/decisions/0187-o-quadro-pela-internet-e-quic-sobre-udp-com-encontro-na-dht.md).
> Antecessores: [W22](HANDOFF_CONTINUACAO_W22_AO_VIVO_PELA_INTERNET_2026-10-14.md) (pedido) ·
> [W22b](HANDOFF_CONTINUACAO_W22b_TRANSPORTE_2026-10-14.md) (decisões e ordem) · medição
> [`11_ao_vivo_pela_internet.md`](../11_ao_vivo_pela_internet.md). Merge-base = `main` `0aab2c1cb6`
> (rebase limpo no início, nada para trás). **A linha PAROU aqui: não integrou nem fez push.**

## 1 — O que entrou (12 commits, `c2eff0a7b7..8a24e8765a`, 46 ficheiros, +5 254/−145)

| passo | commit | o quê |
|---|---|---|
| 1 | `3c5f487a05` | `ph2d-board-net`: a porta `Wire` (fio fiável, `Read+Write`, clones, prazos, `shutdown`) + `Door` (`accept`/`wake`); o TCP da rede local é `tcp.rs`. API pública do `Host`/`Guest` INALTERADA |
| 2 | `9658e33721` | segredo por ENTRADA (`handshake::host_with/guest_with`), `Host::open_door(door, InviteKey)` SEM o travão global, `Guest::join_with_key`; `Invite` = `ph2d-board://v1/<candidatos>#<chave base64url 128 bits>` |
| 3 | `2be526f920` | crate NOVA `ph2d-board-internet`: `Node` (socket UDP por família + UMA thread a conduzir o `quinn-proto`), STUN próprio, furos, `candidates()`, `mapping()`; TLS só envelope |
| 4 | `f93dc2226a` | ponto de encontro na DHT (`mainline` 8, BEP 44, API async por `block_on`), `InternetHost::prepare/attach/open`, `join` |
| 7 | `ea06f4f2d1`, `d3e12beab1` | ADR-0187 (decisões, recusas, aceitação, kill-criterion, medições) |
| 5+6 | `482a554406` | ecrã: «Invite over the internet» (Hosting) e «Join by link» (Off); família: `Route {Lan, Link}` no convidado, entrada pela internet preparada numa thread; textos `ph2d-i18n`; cena 22 |
| fecho | `800602eb9a`, `8a24e8765a` | auditoria (teto `MAX_KEY_SHAKES`, fio fechado se o aperto falha, tentativa cancelada não renasce) e portão (`live.rs` partido, clippy, fmt, `rustls` 0.23.46) |

## 2 — Foundational e porque é ADITIVO

- **`ph2d-board-net`** (consumida só pela `ph2d-app-board` e agora pela `ph2d-board-internet`): tudo novo é
  `pub` acrescentado (`Wire`, `Door`, `Invite`, `InviteKey`, `InviteError`, `KEY_LEN`, `MAX_ADDRS`, `SCHEME`,
  `MAX_KEY_SHAKES`, `Host::open_door`, `Guest::join_with_key`). Nenhuma assinatura pública mudou; os 39 testes
  antigos + 3 de descoberta passaram SEM edição depois do passo 1 (prova do refactor).
- **`ph2d-editor-core`**: `LiveRequest::{InviteInternet, JoinLink}`, `LiveInternet` + `LiveView.internet`,
  `LiveButton::{InviteInternet, JoinLink}` **acrescentados no FIM** de `ALL` (o id é a posição; 12 → 14). Nada
  existente mudou de posição.
- **`ph2d-i18n`**: 14 chaves `board.live.*` novas + 8 `board.smoke.net.*` em `board_live.rs`.
- **Contratos §6 NÃO tocados** — `git diff --name-only main...HEAD` só contém `Cargo.lock`, `crates/ph2d-app-board`,
  `crates/ph2d-board-internet`, `crates/ph2d-board-net`, `crates/ph2d-editor-core/src/{documents_live.rs,
  screens/hero/board_live*.rs}`, `crates/ph2d-i18n/src/board_live.rs` e `docs/`. Nenhum ficheiro de
  `ph2d-nodegraph`, das tools, do vetor ou do `ph2d-imageio`. **Formato gravado: nenhum** (o convite e o item da
  DHT não são gravados no projeto; `PROJECT_SCHEMA` intacto).
- **Números que somam entre linhas**: ADR **0187** (contado contra o `main` 14/10, máximo `0186`) e cena
  **22** do `PH2D_BOARD_SMOKE` (`max_level` 21 → 22) — **reconte ao integrar**.

## 3 — Dependências (`Cargo.lock`)

Novas, só na `ph2d-board-internet`: `quinn-proto 0.11.19` (feature `rustls`), `rcgen 0.14.10` (`ring`),
`mainline 8.0.1` (feature `async`, **sem** `tokio`: `cargo tree`), `futures-lite 2.6`, `socket2 0.6`, `bytes 1`.
Cópias paralelas aceites (ADR-0187): `sha2 0.11`, `rand_core 0.10`, `rand 0.10`, `digest 0.11`,
`ed25519-dalek 3`, `curve25519-dalek 5` (o `spake2 0.4` segura as antigas; nenhum tipo cruza). `rustls`
0.23.45 → **0.23.46** (patch; RUSTSEC-2026-0346..0349; afeta o lock inteiro só nesta linha).

## 4 — Números MEDIDOS (14/10, esta máquina)

| o quê | resultado |
|---|---|
| vazão do fio, 64 MiB em `127.0.0.1`, mínimo de 3 intercalados, perfil `smoke` | QUIC **378 MiB/s** · TCP 5 256 MiB/s (`loadavg` 17) |
| `ph2d-board-internet` (24 testes; DHT local, duas casas atrás de NAT simulado) | verde 3/3 seguidas, ~3–17 s |
| `ph2d-board-net` · `ph2d-app-board` · painel ao vivo (editor-core, 10 por nome) | 50 ✓ · 76 ✓ · 10 ✓ |
| `nextest-impacted` (verificador, 12 min) | 17 391/17 393; os 2 vermelhos: LOC do `live.rs` (CORRIGIDO, `8a24e8765a`) e o flake abaixo |
| clippy `--all-targets --all-features -D warnings` nas 5 crates · `cargo fmt --check` | limpos (depois do `8a24e8765a`) |
| `cargo deny` | `bans`/`licenses`/`sources` ✓; `advisories`: só `wasmtime 48.0.5` (do `ph2d-spike`, PRÉ-EXISTENTE, fora desta linha) |

Mutações (vermelho visto, árvore restaurada): travão na porta da chave · sem furo · fecho sem graça · «acha o
convidado mas não fura» · link passado a maiúsculas · sem `MAX_KEY_SHAKES`.

**Flake de carga, NÃO desta linha:** `ph2d-tool-painter … a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs`
falhou 1× a `loadavg` 18,9 no `nextest-impacted`; sozinho 3/3 verde (`loadavg` 8–11); a crate não tem diff
aqui. Candidato à família de [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md) — quem integrar decide.

## 5 — Smokes (o dono)

- **Cena 22, duas janelas no mesmo computador:**
  `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build -p ph2d-host-desktop --profile smoke && PH2D_BOARD_SMOKE=22 ./target/smoke/ph2d-host-desktop`
  (antes da integração, da pasta da LINHA; depois, da raiz) (janela 1: a «Retro online» aberta, link já copiado) e, noutro terminal, `./target/smoke/ph2d-host-desktop`
  SEM a variável (janela 2): botão Live → colar o link no campo do código → «Join by link». Sem rede pública:
  `PH2D_BOARD_INTERNET=local` nas DUAS (só os endereços desta máquina).
- **Gate que conduz a cena:** `the_internet_scene_copies_a_link_and_a_second_window_joins_by_it`
  (`crates/ph2d-app-board/src/smoke_live_tests.rs`). A cena 12 (rede local) continua verde.
- ⚠️ A cena 22 **não foi fotografada** nesta janela; o que se pinta está provado pelo
  `the_host_invites_over_the_internet_and_copies_again` (o botão pinta-se e clica-se pelo caminho da shell).

## 6 — Aberto (não desta fase, ou à espera do dono)

1. **O furo REAL** (critério de aceitação do ADR-0187): cabo no router + Wi-Fi no telemóvel no MESMO
   computador, depois a sonda numa outra casa (doc 11 §5). Kill-criterion no ADR.
2. Auditoria A2: quando TODOS os caminhos caem, o convidado só sabe no prazo (`LINK_TIMEOUT` 30 s), não antes.
3. Auditoria B2: quem tem o link pode fazer o anfitrião mandar furos (10 B, 15 s) a até 16 endereços — aceite
   (só convidados têm o link).
4. `Mapping::Dependent` (NAT simétrico) é medido mas a UI ainda não o diz.
5. Degrau 1 (UPnP/NAT-PMP/PCP) — só quando uma casa precisar (ADR-0187 §7).
6. Handoff W21 §4 (Windows/Mac reais, enquadramento, fundo tracejado do vídeo, limiares) — inalterado.
