# ADR-0183 — O histórico das votações é um INSTANTÂNEO na equipa do quadro, e o selo deriva da sessão mostrada

- **Status:** Accepted (decisão técnica delegada — `feedback_architecture_decisions_are_delegated_to_the_gold_standard`;
  produto do dono 11/10, depois do smoke da W12: «o histórico das votações, como o Miro»)
- **Data:** 2026-10-11
- **Linha:** `line/MiroClone` (W13, [continuação](../../MiroClone/handoffs/HANDOFF_CONTINUACAO_W13_2026-10-11.md))
- **Amparo:** [ADR-0181](0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md) (registos por campo,
  Lamport) · [ADR-0182](0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md) §7 (o boletim nunca sai do cofre) ·
  o oráculo CORRIDO [`vote_private_oracle/`](../../MiroClone/ferramentas/vote_private_oracle/README.md) (Scrumlr
  v5.4.2, MIT, `h01`–`h03`, 26 casos × 2 corridas iguais) · o Miro medido
  ([`facilitacao_miro_notas.md`](../../MiroClone/ferramentas/facilitacao_miro_notas.md) §7)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — contado contra o máximo do `main` de 11/10 (`0181`) mais
> o `0182` desta linha; quem integrar reconta-o.

## Contexto

Até à W12 o quadro guarda UM resultado: `Org::votes`, um `u32` em cada elemento, escrito pelo fim da votação
(`Editor::end_vote` num passo de desfazer; ao vivo, `Host::end_vote` com uma escrita `Field::Votes` por alvo). A
votação seguinte sobrescreve-o, e «Clear results» zera-o. O dono pediu o histórico do Miro.

O que os outros fazem (corrido/medido):
- **Miro** (o produto de referência do dono, VO 15/09 + duas figuras): separador «Results» com o toggle «Show
  results on the board» no topo e a lista das sessões, a mais nova em cima, «Voting N» e a data curta. Abrir uma
  sessão mostra o nome, o lápis e o lixo, «04 Sep, 12:34 by <quem iniciou>» e uma linha por objecto (miniatura,
  «2 votes», barra relativa ao maior). O quadro mostra os selos DA SESSÃO ABERTA. Não exporta, não vai na cópia
  do quadro, reiniciar = acabar e começar outra.
- **Scrumlr**: uma votação aberta de cada vez (400, h01); todas ficam, a cancelada sem resultado; o quadro mostra
  sozinho a última acabada e não há forma de voltar a mostrar uma antiga (o `PUT showVoting` zera-o, h01). O
  resultado guardado é **recontado** a partir dos boletins vivos: apagar a nota mais votada tira-a, e aos votos
  dela, do resultado antigo (`total 3 → 1`, h02). Não se apaga nem se renomeia (405/404). E a mensagem de entrada
  de quem chega depois leva os **boletins crus** da última votação «anónima» (h03) — o custo de guardar boletins
  para recontar.

## Decisão

1. **A sessão é um registo da EQUIPA do quadro** (`Team::polls: Vec<Poll>`, ao lado das versões):
   `Poll { id: PollId, number, name, at, by, voters, totals: BTreeMap<ElementId, u32> }`. Persiste no ficheiro e
   viaja ao vivo como as versões. Os ids saem do `Team::mint` (com o bit da sessão ao vivo, sem colisão).
2. **O resultado é um INSTANTÂNEO** dos totais no momento do fim, só os `> 0`. Nunca se reconta: os boletins não
   existem fora do cofre do anfitrião (ADR-0182 §7) e guardá-los para recontar é a fuga do h03. Apagar um elemento
   depois não muda a sessão (o oposto do h02): a linha diz «Deleted item» e o total da sessão fica o que foi.
3. **O selo deriva da sessão mostrada** (`Team::shown: Option<PollId>`, um registo do quadro, LWW). Acabar uma
   votação mostra-a (Scrumlr h01, Miro: o resultado aparece a todos). O toggle «Show results on the board» liga
   (à aberta, senão à mais nova) e desliga (`None`). Abrir uma sessão com o toggle ligado passa a mostrá-la
   (figura do Miro). `Org::votes` e `Field::Votes` SAEM: uma verdade só. O selo pinta-se a partir de
   `totals` da sessão mostrada e só nos elementos vivos; durante uma votação continua escondido (`hide_votes`).
   Copiar e colar uma nota deixa de levar um selo que não é dela.
4. **Uma votação de cada vez** (Scrumlr 400). Uma votação de UM ecrã e uma da SALA entram no MESMO histórico. A
   cancelada não entra: o histórico é de resultados.
5. **Nome por omissão «Voting N»** (Miro), `N` = 1 + o maior número vivo; o nome vazio pinta-se «Voting N» no idioma
   do ecrã e o que o artista escreve é texto. A lista ordena pelo NÚMERO (nunca pelo relógio), e dois pares que
   acabam uma votação ao mesmo tempo (o mesmo `N`) são renumerados pela fusão, igual em todos (auditoria W13).
   `by` = o nome de quem iniciou, vazio com «Make names anonymous» ligado no FIM (ADR-0182 §8 — a sessão ficaria a
   dizê-lo para sempre). `at` = o instante do FIM (relógio de parede, segundos Unix).
6. **Renomear** (duplo clique no título da sessão, o idioma das versões — não há ícone de lápis) é LWW por sessão. **Apagar** (lixo, «clique outra vez», o idioma das
   versões) é uma lápide: apagar vence renomear; apagar a mostrada desliga o selo. Nenhum dos dois, nem o fim de
   uma votação, entra no desfazer — são registos da equipa, como versões e comentários. Na W7 o fim era um passo
   de desfazer; deixa de ser, porque desfazer o fim apagaria uma sessão que outros já viram, e o lixo é o caminho.
7. **«Clear results» sai.** Fazia duas coisas que o Miro separa: esconder o selo (o toggle) e esquecer o resultado
   (o lixo). «New vote» volta à configuração; a sessão anterior fica no histórico.
8. **Não vai na cópia do quadro nem numa versão** (Miro: *«If I duplicate my board… No»*); **não se exporta**
   (Miro; o PNG/SVG já sai sem selos). Uma versão restaurada nasce sem histórico.
9. **Formato 11.** A `Team` ganha `polls`, `shown` e o `Org` perde `votes`. Os ficheiros 1–10 leem-se (o 9 e o 10
   pelo `ElementV10`, o idioma do `OrgV8`) e os votos que traziam nos elementos VIVOS viram UMA sessão «Voting 1», sem
   data, mostrada (um voto numa nota apagada não tinha selo: não entra).
   O mesmo vale para o `customData` de um `.excalidraw` antigo. A fixture `format_v10.bin` grava-se ANTES de mudar.
10. **Ao vivo.** `TeamWrite` cresce no FIM (`Poll`, `PollName`, `PollGone`, `Shown`). O fim de uma votação da sala
    é UMA escrita da equipa do anfitrião (a sessão e o `Shown`), no mesmo tique que o estado «acabou» e antes dele
    — o total chega a todos de uma vez (v04), e já não N escritas `Votes`. Quem entra depois recebe o histórico no
    retrato (o h03 sem os boletins). `PROTOCOL` 2 → 3: o `Field` perde `Votes`, e um par da W12 é recusado com a
    frase de sempre.
11. **Quem não lê uma nota escondida não lê o texto dela em lado NENHUM** (pontas da W12): a linha do resultado
    mostra o marcador do modo privado; a busca `Ctrl+F` não a encontra pelo texto; a exportação desenha-a como o
    quadro a mostra (o conjunto `unread` do editor). E com «Make names anonymous» nenhum nome de outra pessoa se
    pinta: lista e avatares do painel ao vivo, avisos de entrada e saída, quem iniciou a votação.

## Alternativas recusadas

| proposta | porquê não |
|---|---|
| recontar o resultado a partir dos boletins guardados (Scrumlr) | os boletins não saem do cofre (ADR-0182 §7); guardá-los é a fuga do h03; e apagar uma nota reescreve o passado (h02) |
| manter `Org::votes` como «o selo mostrado» ao lado do histórico | duas verdades: trocar de sessão seriam N escritas por elemento; copiar uma nota levava um selo alheio |
| guardar o nome por omissão como chave i18n | a chave e o texto do mesmo tipo é defeito à espera (lição do repo); o nome é do artista desde que nasce, como «Version N» |
| a sessão cancelada no histórico (Scrumlr h01) | o Miro lista RESULTADOS; uma linha sem resultado é uma linha que mente |
| o fim de uma votação como passo de desfazer (W7) | desfaria uma sessão que a sala já viu; registos da equipa (versões, comentários) não se desfazem |
| levar o histórico na cópia do quadro ou numa versão | o Miro diz que não; e uma versão é o desenho, nunca a equipa (R4) |
| exportar o histórico | o Miro não exporta |

## Consequências

- `ph2d-board-model`: `poll.rs` (a sessão, o instantâneo, a migração dos votos antigos), `legacy_v10.rs`, `FORMAT_VERSION`
  10 → 11. `Org::votes`, `Element::votes`/`set_votes` e `Field::Votes` saem; quem os lia (selo, painel, Excalidraw,
  exportação, sync) passa a ler a sessão mostrada.
- `ph2d-board-sync`: as quatro escritas da equipa com convergência a três e quedas; `Host::end_vote` escreve a sessão.
- O painel da votação ganha o separador «Results» (o toggle, a lista, a sessão aberta com renomear e apagar).
- Cena 14 (`PH2D_BOARD_SMOKE=14`): três votações seguidas, renomear, apagar, mostrar uma antiga, e o mesmo a dois.
