# CONTINUAÇÃO — `line/MiroClone` W23: fechar TUDO o que ficou aberto (2026-10-18)

> **Estado em 18/10 (bloco 1 fechado): ver [HANDOFF_CONTINUACAO_W23_BLOCO1_2026-10-18.md](HANDOFF_CONTINUACAO_W23_BLOCO1_2026-10-18.md)**
> **Estado em 18/10 (bloco 2, rede ao vivo, fechado; o próximo é o bloco 3): ver [HANDOFF_CONTINUACAO_W23_BLOCO2_2026-10-18.md](HANDOFF_CONTINUACAO_W23_BLOCO2_2026-10-18.md)**
> **Estado em 18/10 (o cartão «Live» separado por ligação, pedido do dono, fechado; o próximo é o bloco 3): ver [HANDOFF_CONTINUACAO_W23_CARTAO_LIVE_2026-10-18.md](HANDOFF_CONTINUACAO_W23_CARTAO_LIVE_2026-10-18.md)**
> **Estado em 20/10 (bloco 5 fechado: A11 e B9; smoke OK; o próximo é o bloco 6 — B12, B13, B14 e o B15 pedido pelo dono — numa janela nova): ver [HANDOFF_CONTINUACAO_W23_BLOCO5_2026-10-20.md](HANDOFF_CONTINUACAO_W23_BLOCO5_2026-10-20.md)**
> **Estado em 20/10 (bloco 4 fechado: B6 e B7 feitos, B10 e B11 já estavam feitos na W16; o próximo é o bloco 5): ver [HANDOFF_CONTINUACAO_W23_BLOCO4_2026-10-20.md](HANDOFF_CONTINUACAO_W23_BLOCO4_2026-10-20.md)**
> **Estado em 19/10 (B5(4) desfazer das abas feito, mutações do B5(2)/B5(4) corridas; o próximo é o smoke da cena 27, depois o bloco 4): ver [HANDOFF_CONTINUACAO_W23_B5_4_2026-10-19.md](HANDOFF_CONTINUACAO_W23_B5_4_2026-10-19.md)**
> **Estado em 19/10 (B5(2) textos ao longo da linha feito; o próximo é o B5(4), depois o bloco 4): ver [HANDOFF_CONTINUACAO_W23_B5_2_2026-10-19.md](HANDOFF_CONTINUACAO_W23_B5_2_2026-10-19.md)**
> **Estado em 19/10 (smoke do dono curado e B5(1) cotovelo feito; o próximo é o B5(2), depois B5(4) e o bloco 4): ver [HANDOFF_CONTINUACAO_W23_SMOKE19_B5_1_2026-10-19.md](HANDOFF_CONTINUACAO_W23_SMOKE19_B5_1_2026-10-19.md)** (anterior, 18/10: [bloco 3](HANDOFF_CONTINUACAO_W23_BLOCO3_2026-10-18.md))

> Leitor: a LLM da próxima janela. Ordem do dono (18/10): *«vamos corrigir até ficar tudo bom»*. Os TESTES que
> precisam do dono (o furo real pela internet, Windows/Mac reais, sondas noutras casas, smokes por fazer) ficam
> para o futuro — **não** são desta onda. A linha foi reaberta sobre o `main` de 18/10 (W22b+W22c integradas:
> [handoff W22c](HANDOFF_INTEGRACAO_line_MiroClone_W22c_2026-10-17.md)).
>
> Esta lista foi CONSOLIDADA de todos os handoffs da linha (W2–W22c) e dos docs 01–11 por um explorador, com
> cada item conferido contra os handoffs posteriores. ⚠️ **Antes de construir cada item, confira no CÓDIGO se ele
> ainda está aberto** (CLAUDE.md §5.0: «confira o CÓDIGO antes de acreditar numa ausência») — os marcados «não
> confirmado» são os mais prováveis de já estarem feitos.

## (A) Defeitos conhecidos

| # | o quê | fonte | tamanho |
|---|---|---|---|
| A1 | ⭐ O enquadramento das cenas não desconta os painéis laterais: a cena 22 a 1366×768 sai cortada (lições 1 e 6) e o cartão «Live» flutua por cima do quadro. Provável mecanismo (NÃO medido): o `fit_pending` corre na 1.ª pintura, antes de a janela crescer até ao tamanho final. Afeta TODAS as cenas. Prova: `fotografa_cena.sh` a 1366×768 e 1930×1040 | W22c §6.3 · W21 §4(d) · W19 §4(f) | médio |
| A2 | Sem keepalive no TCP da rede local: um convidado mudo só cai quando uma escrita passa de 10 s | `HANDOFF_CONTINUACAO_W11_FECHO_2026-10-10.md` §3(b) | médio |
| A3 | Avatar «+n» não clicável com ≥ 16 pessoas | W11 §9 | pequeno |
| A4 | O fundo tracejado de um vídeo pinta-se sólido | W20 §4(d) · W21 §4(d) | pequeno |
| A5 | O anel «a carregar» do vídeo pode ficar parado se o quadro não redesenhar no arranque (não medido) | W21 §4(e) | pequeno |
| A6 | A barra curta com 14 botões (~400 px) não cabe numa janela baixa; nenhum gate o verifica | W6 §9(7) | pequeno |
| A7 | Título de grupo por palavra em *case folding* («API» → «#api») — não confirmado se curado | W7 §9 | pequeno |
| A8 | O título do frame de fora da cena 9 fica debaixo da barra; os «+/−» de uma sessão dentro de um frame visto de longe continuam clicáveis (o editor não conhece o limiar de detalhe) | W7 §9 · `05_facilitacao.md` | pequeno |
| A9 | Reabrir logo após fechar o anfitrião pode cair na porta 47321 por até 2,5 s; um programa fechado a meio não manda adeus (convidados caem em ~2 s) | W11 §9 | pequeno |
| A10 | O foco do gesto na faixa por fora difere ~4,3 px da referência numa aproximação diagonal; o realce do arrasto marca o foco em vez do ponto de paragem | W10 §9 · W9 §9 · `07_fidelidade_excalidraw.md` | pequeno |
| ~~A11~~ | ✅ 20/10 — o ciclo passou à família (`gather`) com gate de comportamento — [bloco 5](HANDOFF_CONTINUACAO_W23_BLOCO5_2026-10-20.md) | W17 §4(f) · W18 §4(j) | pequeno |
| A12 | Entrada por link: cada tentativa cria um nó UDP e um cliente DHT novos; `link_old`/`link_offline` sem teste da família; em `driver_dial`/`connected()` um caminho que liga mas não abre o fluxo fecha sem reavaliar a tentativa | W22c §6.4 (a)(b)(c) | pequeno |
| A13 | A roda do rato não é encaminhada ao tocador do vídeo (`PlayerInput::Wheel` não aparece no código) — não confirmado | explorador 18/10 | pequeno |

## (B) Combinado e não feito

| # | o quê | fonte | tamanho |
|---|---|---|---|
| B1 | Trocar a forma de uma nota para outra (hoje só quadrada ↔ larga) — a lei da nota protege o cofre do modo privado | W12 §9 | médio |
| B2 | Painel de Frames: reordenar por setas e renomear o título do frame no quadro | W5 §10.5 | pequeno |
| B3 | Mapa mental: a letra da raiz não re-mede os filhos; «−» só aparece sem pairar; o vertical herda o horizontal; ramos saem lisos no rascunho | W6 §9 | pequeno |
| B4 | Ponta curta de seta (o Excalidraw corta a metade do último trecho; a nossa é fixa) — confira contra a XA2 do `07_fidelidade_excalidraw.md`, pode já estar feita | `HANDOFF_INTEGRACAO_line_MiroClone_2026-10-07.md` §10.3 | pequeno |
| B5 | Herdados W2–W4, NÃO verificados um a um: arrastar segmento do cotovelo, rótulo ao longo da linha, pontas UML, transbordo e desfazer das abas, cursor sobre as pegas, trackpad, menu do botão direito numa forma, texto-guia do modo em massa | 2026-10-07 §10.6 | vários pequenos |
| ~~B6~~ | ✅ 20/10 — abaixo de 11 px o rascunho é o final de contorno ×2,5 (o final simples VÊ-SE: recusado) — [doc 16 §1](../16_bloco4_velocidade.md) | W5 §10.1 | médio |
| ~~B7~~ | ✅ 20/10 — ordenar pelo prefixo inteiro da chave de z: baralhado 2,3× mais depressa — [doc 16 §2](../16_bloco4_velocidade.md) | W5 §10.2 | médio |
| B8 | Agrupar as escritas a 30 Hz só na internet | W11 §9(c) · W11 FECHO §3(c) | pequeno |
| ~~B9~~ | ✅ 20/10 — a lei das peças do 0.18.1 medida; losango e rect. redondo 3,9 → ≤ 0,12 px — [bloco 5](HANDOFF_CONTINUACAO_W23_BLOCO5_2026-10-20.md) | W9 §9 | médio |
| ~~B10~~ | ✅ já feito na W16 (161,7 → 1,4 ms) — [doc 16 §3](../16_bloco4_velocidade.md) | W14 §6 | pequeno |
| ~~B11~~ | ✅ já feito na W16 (ladrilhos; o pico não cresce com as camadas) — [doc 16 §3](../16_bloco4_velocidade.md) | `HANDOFF_CONTINUACAO_W16_O_QUE_FICOU_DAS_IMAGENS_2026-10-11.md` item 5 | médio |
| B12 | Laço (seleccionar desenhando à volta) | 2026-10-07 §10.5 | médio |
| B13 | Desenho inteligente do Miro (o traço à mão vira forma) | 2026-10-07 §10.5 | grande |
| B14 | Menções, notificações e votação anónima entre pessoas — confira o que o ao vivo (W12, ADR-0182) já cobre | W8 §9 · W7 §9 | grande |
| B15 | **Pedido do dono (20/10):** mandar o que está seleccionado num QUADRO para OUTRO quadro (o «Send to board…» hoje só vai da cena — Hierarchy — para um quadro; no menu do botão direito do quadro não existe). Decidir no plano: copiar ou mover, as ligações (setas presas, frames, mapa mental) e o desfazer dos dois quadros | smoke do dono 20/10 | médio |

## (C) Decisões aceites / limitações (NÃO são trabalho — não reabra)

O router de casa não faz hairpin (doc 11 §5.1) · degrau 1 UPnP/NAT-PMP/PCP só quando uma casa precisar (ADR-0187 §7) ·
B2 da W22b (furos a pedido de quem tem o link) · o estado de reprodução do vídeo não sincroniza ao vivo, o canal aberto
fora não pausa · limiar do clique 400 ms, laser fora da regra · identidade autodeclarada no `Hello`, regra de firewall
apagada à mão parece liberada · divergências escolhidas do Excalidraw (W5 §10.4) · losango/rect redondo do 0.18.1 sem lei
de vão (W10 §9) · setas curvas/cotovelo editadas no Excalidraw não voltam (W8 §9, recusa medida).

## (D) Fora desta linha / à espera do dono

- **Perguntas de produto ao dono** (faça-as em palavras simples, sem bloquear o resto): o Premium do YouTube (W19 §4(a));
  se o «Rascunho/Final» troca também a letra importada (W9 §9 · W10 §9).
- Pressão da mesa: o `winit` 0.30.13 não a entrega no Linux nem no Windows — decisão de dependência do dono (W4).
- Flake de carga `ph2d-app-light2d … a_luz_recua_quando_a_placa_recusa_a_memoria` — não é desta linha.

## Ordem recomendada (problemas equivalentes num só bloco — CLAUDE.md §0.10)

1. **Bloco «o que se vê mal»**: A1 (enquadramento, com fotos antes/depois em 1366×768 e 1930×1040), A6, A8, A3, A4, A5.
2. **Bloco «rede ao vivo»**: A2, A9, A12, B8.
3. **Bloco «quadro»**: A7, A10, A13, B1, B2, B3, B4, B5 (verifique cada herdado; liste os que já existem).
4. **Bloco «velocidade e memória»** (meça ANTES de mexer, `docs/DevOps/MEDIR_VELOCIDADE.md`): B6, B7, B10, B11.
5. **Bloco «fidelidade»**: B9, A11.
6. **Funcionalidades novas** (`/pd-feature`: plano primeiro, produto final, UI incluída, cena de smoke nova): B12, B13, B14, B15.

Cada bloco fecha com `/pd-onda-fechar` (commit, fotos, passos do smoke ao dono); a linha só fecha (`/pd-linha-fechar`)
no fim de tudo.
