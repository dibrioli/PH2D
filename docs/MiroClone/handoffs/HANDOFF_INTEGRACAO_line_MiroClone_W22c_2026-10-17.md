# INTEGRAÇÃO — `line/MiroClone` W22c: o quadro pela internet ACABADO antes do teste real (2026-10-17)

> Leitor: o integrador e a próxima LLM da linha. Decisões: [ADR-0187](../../architecture/decisions/0187-o-quadro-pela-internet-e-quic-sobre-udp-com-encontro-na-dht.md)
> (decisões 8 e 9 novas). Antecessor: [W22b](HANDOFF_INTEGRACAO_line_MiroClone_W22b_2026-10-14.md) — **a W22b NÃO está
> no `main`**: esta linha leva as duas (W22b + W22c) de uma vez. Merge-base = `main` `0aab2c1cb6` (rebase de
> 17/10: «atualizado», nada para trás). **A linha PAROU aqui: não integrou nem fez push.**

## 1 — O que entrou (W22c: 4 commits + este handoff, `71552be22e..HEAD`, 26 ficheiros, +997/−108)

| item do pedido | commit | o quê |
|---|---|---|
| 1. falha rápida (auditoria A2) | `d630ee4312` | o motor marca cada caminho de uma tentativa que cai e o porquê (recusa QUIC ⇒ `ConnectionRefused`; `send_to` sem rota num caminho por abrir ⇒ `NetworkUnreachable`/`HostUnreachable`/`NetworkDown`/`AddrNotAvailable`; QUIC esgotado ⇒ `TimedOut`); sem nenhum vivo e com a lista FECHADA desiste já. `Node::dial` (fechada) · `Node::dial_open` + `DialAdder::done` (`Msg::Settled`, pelo mesmo canal, depois do `add`). `join` usa a aberta; o convidado fecha-a quando o item do anfitrião chega da DHT, quando não há ponto de encontro, ou quando a DHT não responde (`Rendezvous::reachable` = `bootstrapped`, só depois de uma publicação falhar). `join_noting` entrega o `Mapping` do STUN do convidado |
| 2. a rede que bloqueia | `d7be086052` | `LiveView.link: LiveLink { searching, strict_nat, failed }`; `Mapping::Dependent` ⇒ `strict_nat` (anfitrião no `internet_tick`, convidado no `Dialing::poll`); o painel diz «This internet connection may keep people in other houses out…» (anfitrião, link pronto) e «Your internet connection may keep you out…» (convidado à espera) |
| 3. a espera e a falha | `d7be086052`, `7857f42ede` | `Joining` por link ⇒ «Looking for the board on the internet…» + «up to {s} seconds…» (o prazo vem do `LINK_TIMEOUT` da família), nunca o «Connecting…»; a falha escolhe a frase pelo porquê (`link_failed`: sala fechada · link velho · sem internet · não achado, com a rede que bloqueia se medida), vai no aviso E fica no painel debaixo de «Join by link» até a próxima tentativa (`LiveLink::ended` guarda-a) |
| 4. Windows e Mac | — (medição, §4) | `cargo check -p ph2d-board-internet` verde para `x86_64-pc-windows-gnu` e `aarch64-apple-darwin` |
| 5. fotos da cena 22 | `7857f42ede` | achado nas fotos: a 1366×768 (janela de notebook, painéis laterais de fábrica) o HUD de estatísticas — centrado na JANELA — tapava metade do «Live» e a fila da navegação inteira. Cura: `bottom_hud_rect` + `board_nav::clear_of`/`corner` (`Nav.floor`): a fila do canto e o que se apoia nela (minimapa a pintar e a agarrar, avatares, fundo dos painéis, chão da facilitação) sobem por cima do HUD quando ele tapa um botão; a câmara nunca usa o corte. A frase da falha mudou para debaixo de «Join by link» (colada ao «Join» do código, lia-se como dele). E a `fotografa_cena.sh` corria `[env]` e `.cargo/config.toml` como comandos (comentário com crases dentro do heredoc `<<EOF`, do W5p): saiu para fora |
| 6. ADR e README | `0848accff7` | ADR-0187 decisões 8–9, medições, abertos; README (cena 22 e o convite) |

## 2 — Foundational e porque é ADITIVO

- **`ph2d-board-internet`** (crate da W22b, só a `ph2d-app-board` a consome): `Node::dial_open`, `DialAdder::done`,
  `Rendezvous::reachable`, `join_noting` ACRESCENTADOS; `Node::dial` e `join` mantêm a assinatura. Muda o
  comportamento de `Node::dial`: a tentativa sem caminhos vivos devolve o erro logo, em vez de `TimedOut` no
  prazo (o único teste que fixava o antigo, `ninguem_a_escuta_e_a_tentativa_esgota`, passou a
  `…_cai_logo` com `ConnectionRefused`).
- **`ph2d-editor-core`**: `LiveLink` + `LiveView.link` (campo novo); `Nav.floor` (campo novo, `pub(crate)`);
  `bottom_hud::bottom_hud_rect` (pub, novo; `paint_bottom_hud` igual por fora); `board_nav::clear_of`/`corner`
  (`pub(crate)`). Nenhum id mudou (o `ALL` do `LiveButton` intocado).
- **`ph2d-i18n`**: 9 chaves `board.live.*` novas (`link_unreachable_strict`, `link_closed`, `link_old`,
  `link_offline`, `link_searching`, `link_searching_hint`, `nat_strict_host`, `nat_strict_guest`) e a frase de
  `link_unreachable` reescrita (a chave é da W22b, só nesta linha).
- **Contratos §6 NÃO tocados**: `git diff --name-only 0aab2c1cb6..HEAD | grep -E "nodegraph|ph2d-tool|ph2d-vector|ph2d-imageio|project.rs|schema"`
  ⇒ vazio. **Formato gravado: nenhum** (`PROJECT_SCHEMA` intacto; o convite e o item da DHT não se gravam).
- **Números que somam entre linhas**: ADR **0187** e cena **22** (da W22b) — **reconte ao integrar**. A W22c não
  acrescenta ADR nem cena.
- **Fora da pasta do módulo**: `docs/Components/ferramentas/fotografa_cena.sh` (ferramenta partilhada; só mudou de
  sítio um comentário — §1, item 5). `crates/ph2d-editor-core/src/screens/hero/{paint.rs,bottom_hud.rs}` (o HUD
  é de todas as famílias; a mudança só lê o rect dele e só no ramo do quadro).
- `Cargo.lock`/dependências: **sem mudança** na W22c.

## 3 — Números MEDIDOS (17/10, esta máquina, `loadavg` 30–47 de outras linhas)

| o quê | resultado |
|---|---|
| link de uma sala FECHADA (o caminho recusa, sem DHT), prazo 30 s | desiste em **< 3 s** (antes: 30 s); no app, a frase chega em < 5 s (os 5 testes da família em 0,19 s) |
| DHT CALADA (nó de arranque mudo) + caminho que recusa | desiste em **4,1 s** |
| link velho + DHT com o endereço novo | entra (a lista aberta não desiste) |
| mutações da falha rápida (M1 sem desistência · M2 nasce fechada · M3 `path_lost` sem reavaliar · M4 recusa sem nome) | **4/4 vermelhas** (controlo verde, 9 testes) |
| mutações do painel e da família (P1–P3 as três frases do painel · F1 `strict` · F2 frase da recusa · F3 o `Mapping` do convidado · F4 a espera) | **7/7 vermelhas** |
| `cargo check -p ph2d-board-internet --target x86_64-pc-windows-gnu` / `aarch64-apple-darwin` | ✓ / ✓ — sem `mingw` nem SDK da Apple aqui: `CC=clang` + `-DRING_CORE_NOSTDLIBINC=1` (o modo cruzado da própria `ring`); no Mac também `-ffreestanding` + um `TargetConditionals.h` de 5 macros. Só o C do build script da `ring` precisou; o `check` não liga objectos — o Rust das duas crates foi verificado |
| fotos (tela virtual, `fotografa_cena.sh`) | cena 22 a 1930×1040 (local), 1600×900 (local), 1366×768 (internet REAL: STUN e DHT públicos, **sem** aviso de rede estrita — bate com o doc 11 §1) |
| portão do fecho | ver §5 |

Comandos das medições: `bash scripts/ph2d-run.sh cargo test -q -p ph2d-board-internet --lib -- cai_logo cai_na_hora aberta_a_tentativa falha_logo nao_acaba_a_espera dht_calada sem_enderecos_acha dois_nats --nocapture`
· `… -p ph2d-app-board --lib -- strict_network searches_the_internet internet_scene link_that_is_not_ours`
· `… -p ph2d-editor-core --lib -- corner_rises strict_network_is_said joining_by_link_says the_host_invites pasted_link each_role_paints`
· Windows: `CC_x86_64_pc_windows_gnu=clang CFLAGS_x86_64_pc_windows_gnu="--target=x86_64-pc-windows-gnu -DRING_CORE_NOSTDLIBINC=1" bash scripts/ph2d-run.sh cargo check -p ph2d-board-internet --target x86_64-pc-windows-gnu`.

## 4 — Smokes (o dono)

- **Cena 22, duas janelas no mesmo computador** (antes da integração, da pasta da LINHA):
  `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build -p ph2d-host-desktop --profile smoke && PH2D_BOARD_SMOKE=22 ./target/smoke/ph2d-host-desktop`
  e, noutro terminal, `./target/smoke/ph2d-host-desktop` sem a variável: Live → colar o link no campo do código →
  «Join by link». Sem rede pública: `PH2D_BOARD_INTERNET=local` nas DUAS.
- **O teste real do dono** (PC na rede do telemóvel + notebook no cabo do modem): o mesmo, uma janela em cada
  máquina; o link vai por mensagem. Windows: o primeiro arranque pode pedir para deixar o PH2D usar a rede.
- **Fotos dos estados por link** (à mão, com a placa):
  `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-board --lib fotos_dos_estados_da_cena_22 -- --ignored --nocapture`
  ⇒ `/tmp/ph2d_w22c_fotos/{a0_link_pronto_rede_estrita,b0_a_procurar_na_internet,b1_a_sala_fechou}.png`.
- Gates que conduzem: `the_internet_scene_copies_a_link_and_a_second_window_joins_by_it` (cena 22),
  `joining_by_link_searches_the_internet_and_a_closed_room_says_what_to_do`,
  `the_corner_rises_above_the_stats_hud_in_a_narrow_window`.

## 5 — Portão do fecho (1× sobre o diff acumulado)

| o quê | 1.ª corrida | depois das curas |
|---|---|---|
| `nextest-impacted` (`PH2D_SUITE=fecho`, `loadavg` 63–168) | 17 403/17 406; vermelhos: o tecto de LOC (`driver.rs` 727 > 700), o glifo `→` num teste (`no_tofu_glyphs`), e o flake abaixo | os dois gates por nome ✓; `ph2d-board-internet` inteira 30 ✓ (1 ignorado: `medida_vazao`); app-board 6 ✓ · editor-core 8 ✓ por nome |
| clippy `--all-targets --all-features -D warnings` (5 crates) | ✗ `unnecessary_mut_passed` (teste novo) | ✓ |
| `cargo fmt --check` (4 crates) | ✗ 8 ficheiros | ✓ |
| `file_loc_caps` da shell (4) · `arch_safe_clamp_only` (2) · `bash -n fotografa_cena.sh` | ✓ | — |

Curas: a contabilidade das tentativas do motor (`Dial`, `dial`, `give_up_if_lost`, `path_lost`, `cancel`,
`unroutable`, `lost_kind`) mudou para o módulo filho `driver_dial.rs` (ficheiro NOVO; `driver.rs` 604); `-> ` no
lugar de `→`; `&t.hero`; `cargo fmt`; e o comentário de `join_noting` (auditoria: o `Mapping` chega antes de o
convidado DESISTIR, não antes de todo resultado).

**Flake de carga, NÃO desta linha:** `ph2d-app-light2d::it os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria`
(«Buffer 'luz2d/buf/cascatas' label is invalid») falhou 1× no `nextest-impacted`; sozinho com `PH2D_GPU=1` 3/3 verde
(`loadavg` 84 · 67 · 64). A crate não tem diff aqui — candidato à família de [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md).

Auditoria (lentes: correção do motor · ordem entre threads · costura painel↔família · recursos): os achados
aceites estão no §6 (4); provas de mutação no §3.

## 6 — Aberto

1. **O furo REAL** (critério de aceitação do ADR-0187) — é o teste do dono a seguir; kill-criterion no ADR.
2. **Windows/Mac REAIS**: compila (§3); ligar, correr e furar num Windows/Mac de verdade continua por medir
   (com o handoff W21 §4 (a)).
3. A 1366×768 a cena 22 aparece cortada nas bordas (lições 1 e 6) e o cartão «Live» flutua por cima do lado
   direito do quadro — o aberto da W21 §4 (d), «o enquadramento das cenas não desconta os painéis laterais»;
   provável mecanismo: o enquadramento corre na 1.ª pintura, antes de a janela crescer. Não feito aqui.
4. Auditoria W22c (achados aceites, não defeitos medidos): (a) voltar a entrar por link depois de uma queda cria um
   nó UDP e um cliente da DHT por tentativa (já era assim; com a falha rápida as tentativas ficam mais frequentes,
   mas cada uma leva ≥ 2 s do STUN); (b) as frases `link_old` e `link_offline` não têm teste da família (a sem-rota
   não se provoca em 127.0.0.1); (c) em `connected()`, um caminho que liga mas não abre o fluxo fecha sem
   reavaliar a tentativa — só acontece com o nó a fechar.
5. Da W22b, sem mudança: a auditoria B2 (furos a pedido de quem tem o link, aceite) e o degrau 1 (UPnP/NAT-PMP/PCP).

## 7 — Integrado no `main` (17/10)

Depois de Motion · UIUX · Vector · CodeEditor · Macros · Luz2D (ordem do dono). Rebase sobre `6341b20dc8`: 3 conflitos,
resolvidos pelos estágios — o índice dos ADR (0187 entra antes do 0188; contador 193, `adr-index.sh --check` ✓), o
`CLAUDE.md` (só a entrada do Quadro vem da linha) e a `fotografa_cena.sh` (o `main` já tinha a mesma cura, com um guarda
a mais: fica a dele). `Cargo.lock` fundido limpo (35 pacotes da `ph2d-board-internet`). Recontagens: ADR 0187 livre ·
`PROJECT_SCHEMA` 191 / tripla `(191, 13, 23)` intacta · registos 108/109/109 · cena 22. Árvore combinada:
`cargo check --workspace --all-targets` (avisos = erro) ✓ · censos 12/12 ✓ · `nextest-impacted` **17 754/17 754** · clippy
`--workspace --all-targets -D warnings` ✓. Merge `--ff-only`; sem push (ship é outra ordem). O ramo
`backup/MiroClone-pre-rebase-0aab2c1` guarda a ponta antiga e pode ser apagado.
