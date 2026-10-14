# CONTINUAÇÃO — `line/MiroClone` W22: o quadro ao vivo PELA INTERNET (2026-10-14)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: W20+W21 **no main** (13/10, suíte da árvore combinada 17 773/17 773); a linha foi alinhada
> com `git reset --keep main` em `0aab2c1cb6` (14/10, as regras novas do processo: suíte com cadência de 10 commits,
> clippy só no fecho, testes por NOME no meio — CLAUDE.md §2). Nada por integrar.

## O pedido

O dono (13/10): *«temos pendente a fase do trabalho em grupo pela internet»* — a decisão de produto de 10/10 no
[ADR-0181](../../architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md):
**rede local primeiro (feito), internet depois SEM refazer**. O transporte está atrás da porta `Link`
(`ph2d-board-net`, enviar/receber quadros de bytes); a fusão por campo (Lamport + desempate pelo par) já não precisa
de árbitro — um retransmissor CEGO serve.

## O que o dono já ouviu e preferiu (13/10, sem decisão final)

Perguntou *«não há games onde o próprio jogador é o servidor e convida amigos?»*. Explicado: é o modelo da rede local
de hoje (o anfitrião é o retransmissor, estrela — ADR-0181 item 8); pela internet o obstáculo é o ROUTER de cada
casa. A escada proposta, do mais barato ao mais garantido:

1. **O anfitrião pede ao router que abra uma porta** (UPnP-IGD / NAT-PMP / PCP) e o convite leva o endereço
   público: zero servidor. Falha com router que recusa ou com CGNAT (comum em móvel).
2. **Furar os dois NATs** (hole punching, endereço descoberto por STUN público): mais casas; precisa de um ponto de
   encontro mínimo para os dois se apresentarem.
3. **Retransmissor** (o servidor pequeno; o conteúdo cifrado ponta a ponta, chave no convite — o modelo do
   Excalidraw, `01_pesquisa_miro_excalidraw.md` §151) só se 1 e 2 falharem onde importa. Decisão de custo/hospedagem
   do DONO, com as opções medidas lado a lado.

Proposta aceite como direcção: **começar por 1 e 2, sem servidor nosso, e MEDIR em situações reais** (a casa do dono,
um telemóvel em partilha de rede, outra casa) antes de decidir o 3.

## Primeiros passos (a linha decide o técnico; o dono só o produto)

- Oráculo antes de escrever (CLAUDE.md §0.9): como o Excalidraw (`excalidraw-room`, MIT) e um jogo com anfitrião
  convidam — correr, não ler o fonte. Triagem de licença por artefacto de qualquer crate de NAT/STUN/UPnP/QUIC
  (`bash scripts/stack-audit.sh --tetos` antes de pôr dependência).
- Segurança: o código curto da rede local (ADR-0181 item 8, PAKE) não chega para a internet aberta — o convite tem
  de levar segredo suficiente; nada se expõe sem o anfitrião o pedir.
- Uma cena de smoke nova conta-se no roteador do quadro (`max_level` hoje 21).
- Abertos herdados que NÃO são desta fase (estão no handoff da W21 §4): Windows/Mac reais por medir (o dono adiou,
  14/10), o enquadramento que não desconta os painéis, o fundo tracejado do vídeo.
