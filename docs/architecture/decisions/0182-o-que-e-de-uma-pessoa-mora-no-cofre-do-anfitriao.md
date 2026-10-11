# ADR-0182 — O que é de UMA pessoa mora no COFRE do anfitrião até alguém mandar revelar

- **Status:** Accepted (decisão técnica delegada — `feedback_architecture_decisions_are_delegated_to_the_gold_standard`;
  produto do dono 11/10: votação anónima entre pessoas e modo privado, juntos)
- **Data:** 2026-10-11
- **Linha:** `line/MiroClone` (W12, Etapa 2 do [plano](../../MiroClone/02_plano.md))
- **Amparo:** [ADR-0181](0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md) (registos por campo,
  estrela, desfazer por par) · o oráculo CORRIDO
  [`vote_private_oracle/`](../../MiroClone/ferramentas/vote_private_oracle/README.md) (Scrumlr v5.4.2, MIT, 23 casos
  × 2 corridas iguais) · o Miro medido ([`facilitacao_miro_notas.md`](../../MiroClone/ferramentas/facilitacao_miro_notas.md) §2, §6)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — contado contra o máximo do `main` de 11/10 (`0181`);
> quem integrar reconta-o.

## Contexto

Ao vivo (ADR-0181) tudo o que alguém escreve vai a TODOS: o anfitrião aplica e retransmite (`To::AllBut`) e o
`Welcome` leva o quadro inteiro. Duas peças da Etapa 2 pedem o contrário — **algo que é de uma pessoa e só se
mostra a todos quando alguém manda**: o VOTO de cada um (anónimo; só o total, no fim) e a NOTA escrita em modo
privado (só o autor a lê até revelar). Esconder na VISTA não basta: o que chegou à réplica de B está no
computador de B, entra no ficheiro que B grava e não se des-envia.

O que os outros fazem (corrido/medido):
- **Scrumlr** filtra NO SERVIDOR pelo canal ao vivo (B recebe uma lista vazia, p01; votos: ninguém recebe nada
  durante a votação, v01) — mas a API REST devolve a B o texto e o autor das notas escondidas (p01, p06), apagar
  vaza o id (p05) e **tirar o destaque de uma nota revela TODAS a todos** (p07). Voto de quem sai conta (v03); o
  relógio NÃO acaba a votação (v05); esconder outra vez volta a esconder o revelado (p04).
- **Miro** (o produto de referência do dono): esconde SÓ o texto das notas novas ou editadas durante o modo; a
  nota (lugar, tamanho, cor) fica, com rabiscos e um olho fechado — e os rabiscos seguem as LINHAS do texto
  (vaza o comprimento); revelar = «Turn off for all» + 5 s; «Reveal all my sticky notes»; «Make names
  anonymous»; «Allow anyone to turn off». Votos: «All votes are anonymous», os outros só veem QUEM acabou, quem
  conduz vê quantos votos cada pessoa usou, a votação acaba pelo relógio ou por «End for all».

## Decisão

1. **O cofre.** O que é privado viaja SÓ do dono para o anfitrião (`Msg::Sealed`, `Msg::Ballot`) e mora num
   cofre do anfitrião FORA do quadro e da réplica dele — nunca entra num `Writes` para outro, nem no retrato
   (`Welcome`), nem no ficheiro de ninguém além do autor. O anfitrião é um dos participantes: o PROGRAMA dele
   guarda o cofre e nunca o mostra (o mesmo que o servidor do Miro e do Scrumlr sabem; «não é protecção de
   dados», Miro PM). Uma peça só, `ph2d-board-sync::vault`, com duas políticas de soltar:
   **publicar** (notas: as escritas guardadas saem como escritas normais) e **somar** (votos: sai só o total).
2. **Nota privada = o registo `Text` (e o TAMANHO) adiados.** Com o modo ligado, o `Replica::local` de cada par separa: o
   registo `Text` de uma nota — de uma nota que NASCE (o `Born` sai com o texto vazio e o texto num `Set` com
   carimbo seguinte) ou que é editada — vai selado. A caixa de uma nota MINHA escondida também: a nota cresce com
   o texto (o tamanho diria o comprimento — auditoria W12), por isso sai às claras com a posição nova e o TAMANHO
   de antes, e a caixa inteira segue selada com o carimbo seguinte. Cor, z, apagar e mover são públicos como
   sempre. O autor aplica tudo no próprio quadro. A selagem é só uma ENTREGA ADIADA de escritas de Lamport do
   autor: ao soltar, a fusão do ADR-0181 converge sem caso novo. Uma escrita às claras MAIS NOVA de um registo
   selado passa-lhe à frente (sai do cofre e das marcas).
3. **O anfitrião-autor**: o quadro dele é a fonte do retrato, então o retrato SOBREPÕE aos registos selados dele
   o valor E o carimbo PÚBLICOS (sem o carimbo, quem entra recusaria a escrita solta por `at` igual). Uma VERSÃO
   guardada com o modo ligado (W8) leva o quadro como todos o vêem — a da autora também (uma versão é do quadro
   da sala); sem isto o texto selado viajava comprimido numa `TeamWrite::Version`.
4. **Sombra pública**: o anfitrião difunde o estado da facilitação (`Msg::Fac`) — modo ligado/quem ligou/opções,
   os ids das notas escondidas (sem autor), a votação (alvos, votos por pessoa, tempo, quem acabou). Quem entra
   a meio recebe-o logo depois do `Welcome`. Os outros desenham a nota escondida com um marcador FIXO (nunca
   derivado do texto — que eles não têm) e o olho fechado; o autor vê o texto com o olho.
5. **Revelar só pelo comando de revelar.** «Desligar para todos» (quem ligou; ou qualquer um com «Allow anyone
   to turn off») arma 5 s no RELÓGIO DO ANFITRIÃO e solta tudo; «Revelar as minhas» (com «Anyone can reveal
   their stickies») solta só as do autor. Nenhum outro campo, comando ou saída solta o cofre (o p07 do Scrumlr é
   o defeito que este gate morde). Revelar não se desfaz; ligar o modo outra vez esconde só o NOVO (Miro, não o
   p04 do Scrumlr). Fechar a sala com o modo ligado REVELA antes do `End` (o trabalho do artista não se perde);
   uma queda do anfitrião deixa cada autor com o seu texto.
6. **O texto escondido só o autor o escreve.** Quem não o lê não o escreve por caminho NENHUM (Enter, negrito,
   desfazer, colar…): a escrita desfaz-se ao ser capturada (`Replica::set_unread`) e o editor recusa com o aviso;
   o cofre tem UM dono por nota até revelar. Na virada (quem escreve ainda não sabe do modo), o anfitrião põe no
   cofre o texto de uma nota que chega às claras — o filtro que o Scrumlr faz no servidor; se a nota já tem outro
   dono no cofre, o texto é de quem o escreveu para todos e segue. Mover, cor e apagar continuam de todos. O
   desfazer do autor é o do ADR-0181 §3 — durante o modo, o passo desfeito sai selado.
7. **Voto entre pessoas.** O boletim de cada um (`alvo → n`, «acabei») vai inteiro ao anfitrião a cada mudança
   (substitui o anterior — idempotente); ninguém mais recebe nada sobre escolhas (Scrumlr v01). Público: quem
   acabou e o tempo; para quem iniciou, quantos votos cada pessoa usou (Miro VO). Quem entra a meio vota (v02);
   o boletim de quem sai FICA e conta (v03). O fim é do anfitrião: o relógio dele ou «End for all» de quem
   iniciou (Miro; o relógio do Scrumlr não acaba nada, v05). No fim o anfitrião SOMA e escreve `Votes` em cada
   alvo como escritas normais, no MESMO tique que o estado «acabou» e antes dele (o total chega a todos de uma
   vez, v04). É uma escrita DA SALA, não um gesto de alguém: não entra no desfazer de ninguém (como uma escrita
   alheia, ADR-0181 §3); «Clear results» limpa-o. O resultado é sempre anónimo (só totais, v07). Numa nota
   escondida, quem não a lê não põe um voto NOVO (o que lá estava fica e pode sair); um boletim recusado é um aviso
   a quem votou; a soma confere-se sem dar a volta. Se quem iniciou (ou quem ligou o modo privado) sai, o anfitrião
   — o dono da sala, como o dono do quadro no Miro — pode acabar por ele.
8. **«Make names anonymous»**: as notas não têm autor no modelo; durante o modo os cursores e as selecções dos
   outros perdem nome e cor própria e «seguir» desliga-se.
9. **Protocolo 2.** `Msg` cresce SÓ no fim (`Sealed`, `Ballot`, `Fac`, `Ask`); `PROTOCOL` 1 → 2 (um par antigo
   é recusado com a frase de sempre, nunca funde às cegas). O cofre e o estado da facilitação nunca vão a disco.

## Alternativas recusadas

| proposta | porquê não |
|---|---|
| o privado na réplica partilhada, escondido na vista | está no disco de B e no ficheiro que B grava; é a fuga REST do Scrumlr (p01, p06) |
| guardar no AUTOR até revelar | quem sai a meio leva as notas e o voto (o Scrumlr conta o voto de quem sai, v03; o Miro revela tudo) |
| cifrar e soltar a chave depois | a chave mora em alguém: no autor = o caso acima; no anfitrião = o cofre com mais peças |
| agregação segura do voto (partilha de segredos entre pares) | parte com uma saída a meio; o servidor do Miro sabe na mesma; custo sem pedido |
| esconder a nota inteira (posição, tamanho, contagem) | o Miro mostra a nota e esconde o texto — é a lei do produto de referência; o Scrumlr mostra que alguém escreveu na mesma (p01) |
| rabiscos que seguem as linhas (Miro) | vaza o comprimento; e os outros nem têm o texto para os medir |
| esconder outra vez o que já se revelou (Scrumlr p04) | o Miro esconde só o novo; o que todos já leram não se des-lê |
| qualquer outro caminho que solte (destaque, apagar, foco) | o p07 do Scrumlr: tirar um destaque revela tudo; um gate prova que só o comando solta |

## Consequências

- `ph2d-board-sync` ganha `vault.rs` (o cofre, as duas políticas) e `fac.rs` (o estado da facilitação e os
  pedidos), puros, com testes de convergência e de FUGA (nenhum byte do texto selado em mensagem para B antes de
  revelar). A réplica separa o selado e guarda-o à parte para reenviar depois de uma queda.
- A ligação (`ph2d-app-board/live.rs`) leva o voto e o modo entre o ecrã e a sessão; o painel da votação e a
  barra ganham o caminho ao vivo; o modo privado entra no menu da facilitação, só numa sessão.
- Cena 13 (`PH2D_BOARD_SMOKE=13`): duas janelas, votar sem se ver e escrever em privado, revelar.
