# ADR-0181 — O quadro ao vivo FUNDE POR CAMPO, e o transporte é uma porta

- **Status:** Accepted (decisão técnica delegada — `feedback_architecture_decisions_are_delegated_to_the_gold_standard`;
  produto do dono 10/10: rede local primeiro, internet depois sem refazer)
- **Data:** 2026-10-10
- **Linha:** `line/MiroClone` (W11, Etapa 2 do [plano](../../MiroClone/02_plano.md))
- **Amparo:** ADR-0075 (drop-crates, ECS) · o estudo [`08_ao_vivo.md`](../../MiroClone/08_ao_vivo.md) (os dois
  oráculos CORRIDOS, caso a caso)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — contado contra o máximo do `main` e das worktrees de
> 10/10 (`0180`); quem integrar reconta-o.

## Contexto

O quadro (`ph2d-board-*`) é de uma pessoa. O dono quer várias pessoas no MESMO quadro, ao vivo, primeiro na
rede local (todos com o PH2D, um computador abre, os outros entram, sem servidor nem custo) e depois pela
internet sem refazer. O modelo já trazia `version`/`nonce`/lápide/índice fraccionário e uma porta única de
mudança (`BoardOp::apply`); faltava escolher a reconciliação, a identidade, o desfazer, o transporte, a
entrada na sala e a segurança.

Corremos os dois oráculos sobre os MESMOS 23 casos (`docs/MiroClone/ferramentas/colab_oracle/`, duas corridas
cada): o Excalidraw (o do dono) funde o ELEMENTO INTEIRO pela `version` — perde edições inteiras em campos
diferentes (c02, c03, c14), diverge até ~17 s com três pares (c23), ressuscita o apagado (c04) e reenvia
~500 B por forma a cada mensagem de arrasto (c19); o tldraw funde por CAMPO, mas o desfazer pisa a edição
alheia (c11), ressuscita (c12) e um cliente prende-se numa reordenação (c17).

## Decisão

1. **Reconciliação = registos por campo com carimbo de Lamport** (`(contador, par)`, ordem total; ganha o
   maior). A divisão em registos vive no modelo (`ph2d-board-model`, um enum de campos com ler/escrever por
   campo e um gate que reprova um campo do `Element` não coberto). A caixa `x,y,w,h` é UM registo (mexer e
   redimensionar pelo canto não se misturam); cada campo do `Style`, o texto, cada ponta da seta, `z`,
   `frame`, `angle`, `deleted`… são registos próprios. Contagens que se SOMAM (votos, carimbos) são
   contadores por par, nunca registos (duas pessoas a votar ao mesmo tempo dão dois votos). Junção de
   estado (entrar, voltar depois de cair) = juntar retratos com carimbos; ao vivo = deltas. Comutativo,
   associativo, idempotente ⇒ ordem, atraso e duplicação não importam.
2. **Apagar é um registo** — uma edição concorrente de outro campo não ressuscita (c04, como o tldraw).
3. **Desfazer por par que só repõe o que ninguém mexeu depois**: o passo guarda o estado DEPOIS do gesto, e
   ao desfazer repõe-se um registo só se o valor actual ainda é o que o gesto deixou. É a lei do Excalidraw
   (c10–c12), vale também para uma pessoa só (onde é idêntica à de hoje), e não precisa de saber de rede.
4. **O derivado não viaja**: as rotas das setas recalculam-se em cada par (c08, como o tldraw; já é assim).
5. **Identidade**: fora de sessão o contador local de hoje (os ficheiros e as fixtures não mudam); numa
   sessão, ids sorteados de 64 bits com o bit alto ligado (nunca colidem com os do contador; a colisão
   entre sorteados é ~1e-8 a um milhão de elementos). A `version`/`nonce` de um elemento passam a ser
   função dos carimbos dele (o maior contador e o seu par) — iguais em todos os pares depois de convergir.
   O formato do ficheiro NÃO muda (os carimbos são da sessão).
6. **Presença não é documento**: cursor (enviado a ~30 Hz e INTERPOLADO no receptor — os dois oráculos
   saltam, c18), selecção, câmara, nome (o autor da W7) e cor, por um canal sem carimbo; seguir (c21, as
   leis de interromper do Excalidraw) e «trazer todos» (do Miro; nenhum dos dois o tem, c22).
7. **Duas drop-crates**: `ph2d-board-sync` — o protocolo, os carimbos, a fusão, o desfazer, a sessão de
   anfitrião e de convidado como máquinas de estado PURAS (entradas → saídas, sem IO; testadas com uma rede
   simulada com atraso, reordenação, duplicação e queda); `ph2d-board-net` — o transporte da rede local
   atrás de uma porta (`Link`: enviar/receber quadros de bytes), que a internet implementará depois. A
   família liga-as em `ph2d-app-board`; a shell só compõe.
8. **Rede local**: o anfitrião é também o retransmissor (estrela). Quem entra encontra o quadro pelo nome
   anunciado na rede (mDNS) e escreve um **código curto** que o anfitrião mostra; o código autentica por
   PAKE (SPAKE2) e a chave que dele sai cifra tudo (ChaCha20-Poly1305) — quem está no mesmo Wi-Fi e não tem
   o código nem lê nem escreve, e não o pode adivinhar fora de linha. Nada se expõe fora da rede local e o
   PH2D nunca abre a firewall SEM o artista: no Linux (onde o sistema não pergunta, só bloqueia) o painel
   Live mostra um botão que libera a porta fixa + o mDNS SÓ para a sub-rede local, com a senha do sistema
   (polkit), e outro que desfaz; no Windows e no macOS o próprio sistema pergunta na 1.ª escuta (decisão
   do dono, 10/10). Construído (`ph2d_board_net::firewall` decide, `ph2d-app-board::live` corre): ufw e
   firewalld; a FAIXA inteira das portas que o anfitrião pode tomar (TCP 47320–47329, duas janelas na
   mesma máquina) + UDP 5353; UMA senha (`pkexec /bin/sh -c` com as regras em `&&`); o botão aparece
   antes de abrir e na sala do anfitrião (a descoberta de quem entra também precisa dele), nunca ao
   convidado; o que se liberou lembra-se em `~/.ph2d/live_firewall.txt` (a lista de regras só o root a
   lê ⇒ uma regra apagada à mão continua a «parecer» liberada). Outra firewall ligada (nftables/iptables)
   ou sem `pkexec` ⇒ sem botão, a frase com a porta. Os números (o custo de cada dependência no build, a fiabilidade da descoberta, a latência)
   medem-se ao entrar cada uma e escrevem-se neste ADR, §Medições.

## Alternativas recusadas (preço medido em `08_ao_vivo.md` §1)

| alternativa | preço |
|---|---|
| o elemento inteiro por `version`/`nonce` (Excalidraw) | perde edições inteiras (c02, c03, c14); diverge ~17 s (c23); 795 KB/s a arrastar 50 formas (c19) |
| «o último a chegar ao servidor ganha» (tldraw) | exige árbitro — a internet com retransmissor cego não o tem; o cliente dele prende-se numa reordenação (c17) |
| desfazer que repõe o elemento inteiro | pisa a edição alheia e ressuscita (tldraw c11, c12) |
| CRDT de texto | nenhum oráculo intercala (c09); o texto do quadro é curto |
| carimbos gravados no ficheiro (formato 10) | nada os lê fora de uma sessão; a junção de estado ao entrar refá-los |
| ids por par atribuído pelo anfitrião + contador | colide quando o mesmo número de par volta noutra sessão sobre o mesmo ficheiro; o sorteio de 64 bits não tem estado |
| um motor assíncrono (`tokio`) | um anfitrião com dezenas de ligações cabe em threads; nenhuma crate do repo o usa hoje |

## Consequências

- O `History` passa a guardar, por passo, o estado depois do gesto (memória ~2× por passo, tecto `MAX_STEPS`).
- `BoardDoc` passa a juntar os ids tocados (só enquanto há sessão) — a sessão lê ali o que mandar.
- O canal rede → ecrã é o primeiro do quadro: a sessão drena-se no tique do ecrã.

## Medições (10/10, esta máquina: Linux, Wi-Fi 192.168.15.0/24, `loadavg` 1,4–4,8)

| o quê | número | como |
|---|---|---|
| dependências novas | 21 crates, todas permissivas; 0 duplicados novos (61 → 61) | `cargo deny check licenses bans` ok; diff do `Cargo.lock` |
| build a frio da `ph2d-board-net` + deps | dev 1,9 s · smoke 2,1 s de parede | target vazio, sem sccache |
| descoberta mDNS (`mdns-sd`) com `avahi` e `systemd-resolved` activos | sala já anunciada: 9 / 61 / 1 050 ms (mín/med/máx); sai da lista em ~1 s | 5 corridas, dois processos; o `avahi-browse` vê a sala e o código nunca aparece |
| ida-e-volta cifrada (smoke) | 100 B: 17–19 µs · 25 KB: 88–93 µs | convidado → anfitrião → convidado, 7×30 intercalado |
| retrato de quem entra | 193 B/elemento ao vivo (129 do quadro, 63 dos carimbos compactados; eram 603) · 100 mil elementos ≈ 57 MiB antes de compactar, 104 ms em smoke | `the_snapshot_packs_the_stamps_and_gives_them_back_equal` |
| bytes a ARRASTAR (sem cursor) | 1 forma 60 B/imagem (3,5 KB/s a 60 Hz) · 10: 409 B (24 KB/s) · 50: 1 969 B (115 KB/s) | `measure_the_bytes_of_dragging_1_10_and_50_shapes`; o Excalidraw manda 681 B / 5,2 KB / 25 KB por mensagem (c19) |
| latência ponta-a-ponta de um ARRASTO (rede local real, smoke) | 2 pessoas: med 0,47 · p95 0,59 · máx 0,59 ms · 4: 0,64 / 1,00 / 1,13 · 8: 0,57 / 0,98 / 1,19 | `measure_drag_latency_with_2_4_and_8_people` (60 Hz, 2 s, todos observam) |
| tecto de um quadro do fio | `MAX_FRAME` 256 MiB — RAM de quem recebe, 4,4× o maior retrato medido | um cabeçalho de 4 GiB sem bytes não aloca (teste) |

Nenhum tecto entrou sem a linha dele nesta tabela. A presença sai a ≤ 30 Hz (os dois oráculos, c18); as
escritas a cada imagem (na rede local o custo acima é desprezável; a internet pode agrupá-las a 30 Hz).
