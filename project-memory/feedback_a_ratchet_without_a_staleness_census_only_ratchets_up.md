---
name: feedback-a-ratchet-without-a-staleness-census-only-ratchets-up
description: Uma lista de dívida tolerada que ninguém remede vira licença — a catraca precisa de um teste que acuse a entrada obsoleta
metadata:
  type: feedback
---

Toda lista de "dívida conhecida" deste repo é declarada como **catraca que só encolhe**. Nenhuma
delas encolhe sozinha: alguém tem de **medir** e apagar a linha. Sem esse censo, a entrada
sobrevive ao trabalho que a tornou desnecessária e passa a ser uma **licença** silenciosa.

Medido em 2026-08-30, no `architecture_panel_loc_cap.rs`: a lista de funções tinha
`fn_overage_allowlist_has_no_stale_entries`; a lista de **ficheiros** não tinha censo nenhum. Ao
escrever o irmão que faltava, ele acusou **três** entradas obsoletas na primeira corrida:

| entrada | tolerado | medido |
|---|---:|---:|
| `panel-color-equalization/paint_sections.rs` | 660 | **536** (sob o cap desde 2026-05) |
| `panel-painter-layers/paint_adjust.rs` | 829 | **823** |
| `panel-painter-layers/event.rs` | 601 | **570** (sob o cap) |

A primeira estava congelada havia **três meses** sobre um ficheiro que já não a devia.

⚠️ **E a mesma doença tem uma 2.ª forma, que não é uma lista de tolerâncias: a LISTA DE ACHADOS
de uma auditoria** (2026-09-01, `line/motion-value`). A auditoria de seis lentes do `source.lsystem`
listava **24** achados e terminava com *«⛔ Nada disto foi consertado»*; catorze foram curados na
mesma jornada, e o documento **nunca soube** — ele não tem censo, e um doc não se auto-mede. Pior:
o achado **§2.6** (*«`Growth < 1` custa 2,6×–31× a derivação»*) tinha deixado de ser verdade
**sem ninguém o curar** — a `measure_ratio` saiu do caminho do produto quando outra wave a
substituiu pela escada de tamanhos. ⇒ *a auditoria estava certa no dia em que foi escrita*, e ler
uma lista de dívida velha sem re-medir manda alguém consertar o que já não existe (ou, na direcção
oposta, dá por curado o que ninguém tocou). **Uma lista de achados que sobrevive à jornada precisa
da mesma metade que uma allowlist: o alvo ainda existe? ainda falha? o número ainda o descreve?**

**Why:** um número tolerado é lido como "o tamanho certo deste ficheiro". Enquanto ele não for
comparado com a medição, ele deixa de descrever o produto e passa a autorizá-lo — e o próximo
autor cresce até à folga em vez de até ao cap.

**How to apply:** ao criar (ou encontrar) uma lista de tolerâncias, escreva no mesmo commit o
teste que pergunta as três coisas — *o alvo ainda existe? ainda estoura o cap? a folga ainda
descreve o tamanho dele?* — e dê-lhe a metade justa (`achou > N alvos`), senão uma varredura
partida devolve zero obsoletas e lê-se como aprovado ([[feedback_a_bucket_nobody_fills_reads_as_perfect]]).
A cura de um alvo estourado continua a ser o corte por responsabilidade, nunca o número maior
([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).
