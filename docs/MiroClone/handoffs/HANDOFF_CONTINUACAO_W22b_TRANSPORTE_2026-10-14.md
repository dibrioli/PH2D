# CONTINUAÇÃO — `line/MiroClone` W22b: o transporte pela internet (2026-10-14)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Antecessor: [W22](HANDOFF_CONTINUACAO_W22_AO_VIVO_PELA_INTERNET_2026-10-14.md) (o pedido e a escada
> do dono). Medições e oráculos: [`../11_ao_vivo_pela_internet.md`](../11_ao_vivo_pela_internet.md) — LEIA o §4 antes
> de desenhar. Commit `7a65287773` (sonda + 1.ª medição). Nada de código tocado ainda. O dono: *«escolha e siga»*
> — o técnico é da linha; ele NÃO pode medir hoje (telemóvel/cabo/outra casa ficam para depois, doc 11 §5).

## Decisões técnicas já tomadas (a janela anterior; não re-litigue sem medição nova)

1. **A porta = um FIO fiável, ordenado, nos dois sentidos** (`Read + Write`), mais a ENTRADA do anfitrião. O
   aperto de mão SPAKE2 e a cifra ChaCha20 (`handshake.rs`, `wire.rs`) já são genéricos (`impl Read + Write`) e
   ficam POR CIMA de qualquer fio ⇒ cifra ponta a ponta igual em todos os transportes, retransmissor futuro cego.
2. **O anfitrião aceita por VÁRIAS entradas ao mesmo tempo** (sessão mista: LAN + internet). `ConnId` passa a um
   contador partilhado (`AtomicU64` no `Shared`); uma thread de `accept` por entrada; o `Drop` acorda todas.
3. **Internet = UDP** (furar NAT exige UDP; o TCP de hoje fica para a LAN). Fio fiável sobre UDP =
   **`quinn-proto`** (QUIC sem I/O, MIT/Apache) conduzido por UMA thread com o socket UDP (o mesmo socket faz o
   STUN, os furos e todas as ligações — a porta mapeada é uma só). ⛔ `quinn`/`iroh` puxam `tokio` (ADR-0181
   recusou) e o `iroh` usa retransmissores de terceiros por omissão. Antes de pôr a dependência: `cargo info
   quinn-proto` (features: precisa de `rustls` + provedor — `ring` e `rustls` JÁ estão no `Cargo.lock`) e
   `bash scripts/stack-audit.sh --tetos` (o `spake2` segura `rand_core 0.6`/`sha2 0.10`). Certificado
   auto-assinado: a autenticação é o SPAKE2 por cima.
4. **STUN próprio** (~40 linhas, a sonda `ferramentas/internet_oracle/sonda_rede.py` é a referência, com os 3
   servidores de destinos distintos). IPv6 global entra como candidato DIRECTO.
5. **O segredo pela internet é LONGO** (Excalidraw: chave de 128 bits no `#` do link). Razão que o briefing não
   tinha: o bloqueio de 5 erros/30 s do código curto é GLOBAL ⇒ na internet aberta qualquer um trancaria a sala
   (DoS). Entrada da internet = chave de 128 bits no convite, sem bloqueio global; a LAN mantém o código curto.
   O segredo passa a ser por ENTRADA (o `handshake` recebe os bytes da senha, não o `Code`).
6. **Ponto de encontro sem servidor nosso: a DHT do BitTorrent (`mainline` 8.0.1, MIT)** — o anfitrião publica os
   seus candidatos (cifrados com a chave do convite) e o convidado publica os dele sob uma chave derivada do
   convite; os dois furam ao mesmo tempo. Verificar ANTES que o `mainline` não puxa `tokio`. Plano B medido, não
   escolhido: código de resposta copiado à mão (dois sentidos, UX pior).
7. **Degrau 1 (UPnP/NAT-PMP/PCP) fica DEPOIS**: o router do dono não fala nenhum (doc 11 §1); entra só quando a
   tabela §1 mostrar uma casa onde furar falha e o router o aceita. Dizê-lo no handoff de integração.

## Ordem (cada passo fecha com os testes de `ph2d-board-net` por NOME)

1. **Extrair a porta** — rascunho pronto (não gravado): `src/link.rs` com `trait Wire: Read + Write + Send +
   'static { try_clone → Box<dyn Wire>; set_read_timeout; set_write_timeout; shutdown (acorda todos os clones) }`
   e `trait Door: Send + Sync { accept → (Box<dyn Wire>, SocketAddr); wake }`; `src/tcp.rs` com `impl Wire for
   TcpStream`, `TcpDoor` (o `bind` e o filtro `is_lan` + `set_nodelay` saem do `host.rs`; `wake` = ligar a
   `127.0.0.1:port`, o que hoje está no `Drop`). `host.rs`: `wires: BTreeMap<ConnId, Box<dyn Wire>>`,
   `read_loop`/`write_loop` recebem `Box<dyn Wire>` por valor. `guest.rs`: `connect_with` = `is_lan` + TCP +
   `join(Box<dyn Wire>, …)`. API pública INALTERADA (a `ph2d-app-board/src/live.rs` não muda). Prova: os testes
   `tests/it/` (`ponta_a_ponta`, `dois_processos`, `descoberta`) e os `*_tests.rs` verdes SEM edição.
2. **Segredo por entrada** (decisão 5) + o convite como link (`ph2d-board://…#<chave b64>` ou o formato que a UI
   já usa para links — confira) com os candidatos e a chave.
3. **Crate NOVA `ph2d-board-internet`** (drop-crate; dependências pesadas fora da `ph2d-board-net`, que expõe
   `Wire`/`Door` como `pub` só quando esta crate os consumir): socket UDP + STUN + `quinn-proto` + furos. Teste
   ponta a ponta em `127.0.0.1` (dois endpoints) e em `::1`; o furo REAL só com o dono (doc 11 §5).
4. **DHT** (decisão 6).
5. **UI**: «Convidar pela internet» (copia o link) e «Entrar por link»; textos em `ph2d-i18n` (`board_live.rs`);
   o modo privado e o sinal de vida valem iguais.
6. **Cena de smoke 22** (`crates/ph2d-app-board/src/smoke.rs`, `max_level` 21 → 22, o braço e a mensagem
   `(1..=22)`): duas janelas no mesmo computador pelo caminho da internet (loopback), para o dono ensaiar o convite.
7. **ADR novo** (o próximo livre em `docs/architecture/decisions/README.md`): decisões 1–7, recusas com preço
   (`tokio`/`iroh`, TCP com furo simultâneo, código curto na internet, servidor nosso), **critério de aceitação**:
   furo real entre a casa do dono e a partilha do telemóvel; **kill-criterion**: se o furo falhar nas duas
   medições reais do dono, o degrau 3 (retransmissor) sobe a pergunta de produto ao dono com custos lado a lado.

## Abertos que NÃO são desta fase
Handoff W21 §4 (Windows/Mac reais, enquadramento sem descontar painéis, fundo tracejado do vídeo, limiares).
