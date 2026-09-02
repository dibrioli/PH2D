---
name: feedback-a-probe-that-feeds-a-raw-file-misses-what-the-app-does-to-it
description: "Sonda que alimenta o ficheiro CRU mede outro programa — o app normaliza/ancora/transforma na porta, e é aí que o defeito vive"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T22:35:53.273Z
---

Uma sonda que lê um `.obj` (ou qualquer ficheiro) e o entrega directamente ao motor **não
está a correr o caminho do artista**. Entre o ficheiro e o motor há a porta de importação, e
ela transforma: no PH2D, `sculpt3d_import::IMPORT_SPAN = 2.0` **normaliza o tamanho e ancora
a peça fora da origem**.

Medido 2026-08-31 (`line/quadextract`): um dia inteiro de varreduras dizia *«`0` de `4`
pontas cortadas»* sobre a peça que o dono fotografou **amputada**. As duas medições estavam
certas — a sonda corria a peça **centrada**, o app corria-a em `x ≈ 2`, e a MESMA malha só
transladada dá `5 703` quads e `0/4` na origem contra `5 301` e `2/4` em `x = 2`.

⛔⛔ **E o que isso expôs é maior que a porta:** canonicalizar a pose (correr sempre na
origem) faz as entradas diferirem **só no arredondamento de `f32`** — e a saída passou de
`5 703` quads a `4 142`/`3 950`/`4 435`, com pontas a `−77 %` e `−105 %`. *A cadeia é
caótica nos últimos bits; a posição era só a perturbação que o artista consegue introduzir
sem querer.*

**Why:** a diferença entre «não reproduzo» e «reproduzo» era uma transformação que nenhum
dos dois lados mencionava — e sem ela eu ia a caminho de declarar o report do dono
irreproduzível.

**How to apply:** ao escrever uma sonda que substitui um gesto do artista, **liste as
transformações que a porta real aplica** (normalização, ancoragem, reparação, pose) e
reproduza-as, ou corra a porta. E quando uma medição sua discorda de um report com foto,
pergunte primeiro *o que é que o app faz ao meu input que eu não estou a fazer?* — antes de
duvidar do report. Ver [[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]].

⛔⛔⛔ **2026-08-31, o mesmo dia, a forma CARA da mesma lei — e desta vez derivei a
transformação do sítio errado.** A saída que o dono exportou vinha a `0,582×` e ancorada em
`x ≈ 2`, e eu **inferi daí** que a peça vivia ali quando o botão corre. Duas mensagens
inteiras e uma constante de produto foram construídas sobre essa inferência.

⛔ **Bastava LER o importador:** ele **recentra a malha** e põe a escala e a posição numa
**pose que só desenha e exporta**. O motor vê sempre a peça **centrada e na escala
original** — e o ficheiro exportado traz a pose **assada**, que é precisamente o que engana.

⇒ **Lei:** a transformação que a porta aplica lê-se no **código da porta**, nunca inferida da
saída — *uma exportação assa transformações que o motor nunca viu*. E uma constante escolhida
sobre a fixtura errada é pior que nenhuma: ela shipa, e é o dono que a apanha.
