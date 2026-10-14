# 11 — O quadro ao vivo PELA INTERNET (W22): o que se mediu antes de escrever

> Leitor: a próxima LLM. Briefing: [HANDOFF_CONTINUACAO_W22](handoffs/HANDOFF_CONTINUACAO_W22_AO_VIVO_PELA_INTERNET_2026-10-14.md).
> Instrumento: [`ferramentas/internet_oracle/sonda_rede.py`](ferramentas/internet_oracle/sonda_rede.py) — só de LEITURA
> (nenhum mapeamento no router), saídas em `ferramentas/internet_oracle/saidas/` com cabeçalho e endereços
> públicos MASCARADOS (a saída é versionada).

## §1 — Casas medidas

| casa (rótulo) | UPnP-IGD | NAT-PMP | PCP | IPv4 visto pelo STUN | mapeamento v4 | porta preservada | IPv6 global | NAT duplo/CGNAT |
|---|---|---|---|---|---|---|---|---|
| `casa-enio-wifi` (14/10) | ✗ (multicast **e** unicast ao gateway) | ✗ | ✗ | público | **independente do destino** (3 destinos distintos) | ✓ | ✓ (2 destinos v6) | desconhecido (o router não diz o seu IP) |
| telemóvel em partilha de rede | — | — | — | — | — | — | — | — |
| outra casa | — | — | — | — | — | — | — | — |

- O `ufw` está activo nesta máquina e descarta a resposta UNICAST a um M-SEARCH MULTICAST — por isso a sonda
  pergunta também por unicast ao gateway, e o NAT-PMP/PCP respondem pela mesma tupla (o `conntrack` deixa
  passar). Os três calaram ⇒ o router não serve nenhum dos três **como está configurado**; se o UPnP está
  só desligado no painel do router, NÃO foi medido.
- `stun.l.google.com` e `stun1.l.google.com` resolvem para o MESMO destino: o veredito de mapeamento conta
  destinos distintos (`destinos_distintos`), nunca nomes.

## §2 — Oráculos

- **Excalidraw** (`colab_oracle/excalidraw_colab`, corrido 10/10, MIT): o convite é um link
  `#room=<20 hex>,<22 b64>` — sala de 80 bits e **chave de 128 bits no fragmento**, que o navegador nunca
  envia ao servidor; o `excalidraw-room` só retransmite ([08 §1](08_ao_vivo.md), [01 §151](01_pesquisa_miro_excalidraw.md)).
  ⇒ o segredo do nosso convite pela internet tem a mesma ordem: o código de 6 símbolos (~29,8 bits,
  `ph2d-board-net/src/code.rs`) chega na rede local por causa do bloqueio de 5 erros/30 s, não na internet aberta.
- **Godot** (MIT, 4.7.2, classe `UPNP`) e **`upnpc`** (miniupnpc 2.3.3, BSD-3): o Godot usa a MESMA biblioteca;
  a resposta do `upnpc -s` («No IGD») é a dele nesta casa — não foi corrido em separado por isso.

## §3 — Triagem de licença das candidatas (`cargo info`, 14/10)

| crate | versão | licença | nota |
|---|---|---|---|
| `igd-next` | 0.18.0 | MIT | UPnP-IGD; `aio_tokio` é opcional |
| `crab_nat` | 0.8.1 | MIT | NAT-PMP + PCP |
| `quinn-proto` | 0.11.19 | MIT OR Apache-2.0 | QUIC **sem I/O** — cabe nas threads do `Host` (ADR-0181 recusou `tokio`) |
| `quinn` | 0.11.12 | MIT OR Apache-2.0 | `runtime-tokio` por omissão |
| `str0m` | 0.24.1 | MIT OR Apache-2.0 | ICE/WebRTC sem I/O |
| `mainline` | 8.0.1 | MIT | DHT do BitTorrent — ponto de encontro SEM servidor nosso |
| `iroh` | 1.3.0 | MIT OR Apache-2.0 | tudo junto, mas `tokio` + retransmissores de terceiros por omissão |
| `stun-rs` | 0.1.11 | MIT/Apache-2.0 | o pedido STUN cabe em ~40 linhas (a sonda faz) |

Nenhuma está no `Cargo.lock` hoje. `stack-audit --tetos`: `spake2 0.4` segura `rand_core 0.6` e `sha2 0.10` —
uma candidata que peça versões novas destes dá DUAS cópias.

## §4 — O que a medição já muda (premissas do briefing)

1. **Degrau 1 falha na casa do dono.** Fica como degrau barato para outras casas, mas não pode ser o único
   caminho — o veredito depende das outras linhas da tabela §1.
2. **Degrau 2 é viável deste lado:** mapeamento independente do destino com porta preservada é o caso em que
   furar o NAT funciona. Mas furar pede **UDP** e o transporte de hoje é TCP ⇒ um transporte fiável e cifrado
   sobre UDP ao lado do TCP da rede local (candidato: `quinn-proto`, sem I/O). Decisão após §1 completa.
3. **IPv6 global existe** ⇒ caminho DIRECTO sem tradução, se o outro lado também o tiver; hoje o convidado
   recusa todo IPv6 global (`JoinError::NotLan`, de propósito — ADR-0181 item 8).
4. **A «porta `Link`» do ADR-0181 não existe no código:** são `Host`/`Guest` concretos, presos à LAN. Extraí-la é
   o primeiro trabalho, provado pelos testes de `ph2d-board-net` sem mudança.
5. **Ponto de encontro sem servidor nosso** — por medir: (a) o convite leva o endereço do anfitrião e o
   convidado devolve um código de resposta; (b) DHT pública (`mainline`); (c) só então o retransmissor do degrau 3.

## §5 — Próximas medições (precisam do dono)

- Esta máquina (`desktop-linux`) tem Wi-Fi **e** uma porta de cabo (`enp8s0`, hoje sem cabo): com o cabo no
  router e o Wi-Fi na partilha do telemóvel, são DUAS redes num só computador — a prova real de furar dois NATs
  sem precisar de outra casa.
- Telemóvel em partilha de rede: o Wi-Fi desta máquina no telemóvel, e a sonda outra vez.
- Outra casa: a sonda corrida lá (é um ficheiro Python, sem instalar nada).
