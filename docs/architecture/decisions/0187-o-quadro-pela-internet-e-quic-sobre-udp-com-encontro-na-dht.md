# ADR-0187 — O quadro ao vivo PELA INTERNET é QUIC sobre UDP, com encontro na DHT e furo no NAT

- **Status:** Accepted (decisão técnica delegada — `feedback_architecture_decisions_are_delegated_to_the_gold_standard`;
  produto do dono 14/10: *«escolha e siga»*, a escada «o anfitrião abre a porta → furar os NATs →
  retransmissor só se preciso»). **Aceitação CONDICIONAL** ao furo real (§Critério de aceitação).
- **Data:** 2026-10-14
- **Linha:** `line/MiroClone` (W22/W22b — [briefing](../../MiroClone/handoffs/HANDOFF_CONTINUACAO_W22_AO_VIVO_PELA_INTERNET_2026-10-14.md),
  [decisões](../../MiroClone/handoffs/HANDOFF_CONTINUACAO_W22b_TRANSPORTE_2026-10-14.md))
- **Amparo:** [ADR-0181](0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md) (o transporte é uma
  porta; threads std, sem `tokio`) · a medição [`11_ao_vivo_pela_internet.md`](../../MiroClone/11_ao_vivo_pela_internet.md)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — contado contra o `main` de 14/10 (`0186`); quem integrar
> reconta-o.

## Contexto

O quadro ao vivo (ADR-0181) só funcionava na rede local: TCP na porta fixa, origens não-privadas recusadas, o
código curto de 6 símbolos. O dono quer o mesmo pela internet, sem servidor nosso e sem custo. A medição da
casa do dono (doc 11 §1) disse: o router NÃO fala UPnP-IGD, NAT-PMP nem PCP; o NAT mapeia
**independentemente do destino** com a porta preservada (o caso em que furar funciona); há IPv6 global. E a
«porta `Link`» do ADR-0181 não existia no código — eram `Host`/`Guest` presos ao TCP.

## Decisão

1. **A porta é um FIO fiável, ordenado, nos dois sentidos** (`ph2d_board_net::Wire`: `Read + Write`,
   `try_clone`, prazos, `shutdown` que acorda todos os clones) **e a ENTRADA do anfitrião** (`Door`:
   `accept`, `wake`). O TCP da rede local é uma implementação (`tcp.rs`); o aperto de mão SPAKE2 e a cifra
   ChaCha20-Poly1305 ficam POR CIMA de qualquer fio ⇒ a mesma cifra ponta a ponta em todos os transportes, e
   um retransmissor futuro fica cego. Prova: os 39 testes da `ph2d-board-net` (+ 3 de descoberta) verdes sem
   edição depois da extracção.
2. **O anfitrião aceita por VÁRIAS entradas ao mesmo tempo** (sessão mista, rede local + internet):
   `Host::open_door`, um `ConnId` partilhado (`AtomicU64`), uma thread de `accept` por entrada, o `Drop`
   acorda todas.
3. **Internet = UDP; o fio fiável = `quinn-proto`** (QUIC SEM I/O, MIT/Apache) conduzido por UMA thread nossa
   que é dona dos sockets (um por família): o mesmo socket faz o STUN, os furos e todas as ligações — a porta
   mapeada pelo NAT é uma só, e é ela que vai no convite. Uma ligação = um fluxo bidireccional = um `Wire`.
   O fecho é com GRAÇA, como no TCP: o envio termina depois da fila, e a ligação só fecha quando o outro lado
   confirmou tudo e acabou o seu (ou `CLOSE_GRACE` = 3 s) — o adeus do `Host::close` chega. O TLS do QUIC é
   só envelope (certificado `rcgen` por sessão, o convidado aceita qualquer um mas confere a assinatura); a
   identidade é do SPAKE2. `grease_quic_bit(false)` separa STUN e furos do QUIC pelos 2 bits de cima.
4. **STUN próprio** (RFC 5389, ~60 linhas, o vector oficial do RFC 5769 como teste), os 3 servidores de
   DESTINOS distintos da sonda; `mapping()` diz se o NAT mapeia por destino (simétrico ⇒ furar não chega).
   IPv6 global entra como candidato DIRECTO.
5. **O segredo pela internet é LONGO e é por ENTRADA**: a chave de 128 bits do convite (como o Excalidraw:
   no fragmento do link, `ph2d-board://v1/<candidatos>#<chave base64url>`), SEM o bloqueio global de 5
   erros/30 s — na internet aberta esse bloqueio deixava qualquer um trancar a sala. A rede local mantém o
   código curto com o bloqueio. O aperto de mão recebe os bytes do segredo da entrada.
6. **Ponto de encontro sem servidor nosso: a DHT do BitTorrent** (`mainline` 8.0.1, MIT; BEP 44, itens mutáveis
   assinados). Da chave do convite saem (HKDF) a chave de assinatura do anfitrião, a dos convidados e a cifra
   dos valores; o anfitrião publica os seus candidatos e vigia o item dos convidados; cada convidado publica
   os seus e repete até entrar (o mais recente ganha — dois a entrar no mesmo segundo só se atrasam); o
   anfitrião FURA para cada convidado que lá aparece, e o convidado liga-se a todos os candidatos ao mesmo
   tempo (o primeiro que responde ganha). A API síncrona do `mainline` 8 está obsoleta: usa-se a assíncrona
   com `futures_lite::future::block_on` na nossa thread (sem runtime). `cargo tree` (14/10): **nenhum
   `tokio`** na árvore.
7. **O degrau 1 (UPnP/NAT-PMP/PCP) fica DEPOIS**: o router do dono não fala nenhum (doc 11 §1); entra só
   quando a tabela §1 mostrar uma casa onde furar falha e o router o aceita.

## Alternativas recusadas (o preço)

| alternativa | porque não |
|---|---|
| `quinn` / `iroh` | puxam `tokio` (recusado no ADR-0181 §8: threads std); o `iroh` usa por omissão retransmissores de TERCEIROS — os nossos quadros passariam por um servidor que não escolhemos |
| TCP com furo simultâneo (*simultaneous open*) | funciona em muito menos NATs que o UDP (depende do router aceitar um SYN sem estado); o QUIC dá-nos, num socket, o furo, a fiabilidade e o controlo de fluxo |
| o código curto na internet | ~29,7 bits só aguentam com o bloqueio global; na internet o bloqueio é um botão de DoS (qualquer um tranca a sala) — sem ele, o código cai por força bruta online |
| servidor de encontro nosso | custo fixo e um ponto de falha que o dono não quer; a DHT pública é gratuita e já existe há 20 anos |
| código de resposta copiado à mão (plano B medido, não escolhido) | dois sentidos de copiar/colar: a UX é pior; fica como recurso se a DHT falhar numa casa |
| `str0m` (ICE/WebRTC sem I/O) | ICE completo + SCTP/DTLS para um só fluxo fiável entre os nossos programas: muito mais superfície para o mesmo furo |

**Duplicatas aceites:** o `mainline` 8 traz `sha2 0.11`, `rand_core 0.10`, `ed25519-dalek 3` e
`curve25519-dalek 5` ao lado das versões que o `spake2 0.4` segura (`stack-audit --tetos`). Ficam DENTRO da
`ph2d-board-internet`; nenhum tipo delas cruza a fronteira (só bytes e `SocketAddr`).

## Critério de aceitação (congelado)

- **Automático (feito, 14/10):** a `ph2d-board-internet` passa em `127.0.0.1` e `::1` — o fio (10 MiB), 1 em
  cada 7 pacotes perdidos, o `Host`/`Guest` por cima com o adeus ANTES do fim, chave errada recusada, STUN
  pelo mesmo socket, e **duas casas atrás de NATs simulados** (mapeamento independente + filtro por endereço,
  o da casa medida): COM ponto de encontro entra, SEM ele não entra (controlo). Mutações «sem furo», «fecho
  sem graça» e «acha o convidado mas não fura» ⇒ vermelho.
- **Real (por fazer, precisa do dono — doc 11 §5):** um furo real entre a casa do dono (cabo no router) e a
  partilha de rede do telemóvel (Wi-Fi), as DUAS redes no mesmo computador; depois a sonda numa outra casa.

## Kill-criterion

Se o furo **falhar nas duas medições reais do dono** (telemóvel e outra casa), o degrau 3 (retransmissor)
sobe como **pergunta de produto** ao dono, com os custos lado a lado (servidor próprio pago × retransmissor
de terceiros × degrau 1 onde o router o aceitar). A cifra ponta a ponta (decisão 1) já deixa esse
retransmissor cego.

## Consequências

- Crate nova `ph2d-board-internet` (drop-crate): `Node`, `InternetHost`, `join`, `Rendezvous`; a
  `ph2d-board-net` expõe `Wire`/`Door`/`Invite`/`InviteKey` e `Host::open_door`/`Guest::join_with_key`.
- O app ganha «Convidar pela internet» (copia o link) e «Entrar por link»; a cena de teste 22 ensaia o convite
  com duas janelas no mesmo computador.
- Abertos: o furo real (acima); o NAT simétrico detectado (`Mapping::Dependent`) ainda não muda a UI; o
  degrau 1.
