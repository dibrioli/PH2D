---
name: a-gate-calibrated-at-the-default-width-is-blind-to-the-width-the-artist-has
description: O gate media `inspector-w = 304` e o dono tem `dock_w_right = 220,9` no ficheiro de arrumação — zero rótulos cortados contra dezasseis, com o gate verde sobre a foto dele.
metadata:
  type: feedback
---

Quando uma superfície do app é **redimensionável pelo utilizador**, a largura de OMISSÃO é a única
que um gate nunca devia medir sozinha: é o único ponto da escala em que ninguém está depois do
primeiro dia de uso. O gate fica verde, a foto do dono mostra o defeito, e a discussão vai para o
sítio errado (*«mas o teste passa»*).

**Caso medido (`line/UIUX`, 2026-09-14).** Report: *«3 pontos (…) sendo usados antes de ficar
estreito»*, com o `every_label_this_panel_paints_fits_its_column` verde. A régua dele derivava a
largura da linha do token `inspector-w = 304`. O ficheiro de arrumação do dono
(`~/.ph2d/layout.txt`) dizia `dock_w_right = 220.9`:

| painel | coluna | rótulos elididos (de 52) |
|---|---|---|
| `304` (o que o gate media) | `120,0` | **0** |
| `220,9` (o que ele tem) | `78,4` | **16** |

A cura não foi medir melhor um ponto: foi medir a **escada** — quatro larguras que cercam a dele,
com o número medido em cada e a metade de obsolescência ao lado (se melhorar, o número desce).

**Why:** um default é um valor de fábrica; o estado do utilizador vive noutro ficheiro, e esse
ficheiro **existe na máquina onde o report nasce**. Uma régua que o ignora está a medir um programa
que só existe na primeira execução.

**How to apply:** para qualquer grandeza que dependa de uma superfície redimensionável, o gate mede
uma **escada** e não um ponto, e pelo menos um degrau tem de vir do **estado real** (`~/.ph2d/…`,
o projecto gravado, o que o report trouxe) — com a proveniência escrita ao lado do número, senão
ele lê-se como escolhido. E ao receber um report de aparência com o gate verde, **a primeira
pergunta é «em que largura/tema/definição é que ele está?»**, não «onde está o bug?».

Vizinhos: [[feedback-a-gate-written-over-a-number-fails-when-the-owner-changes-the-number]] ·
[[feedback-a-wave-that-changes-a-surface-invalidates-the-designs-that-sat-on-it]] ·
[[feedback-a-probe-that-arms-a-module-by-env-var-measures-another-program-than-the-pill]] ·
[[reference-topic-measurement-discipline]]

---

## ⛔⛔⛔ E em 2026-09-19 a MESMA cegueira apareceu **uma escala acima**: a varredura de TODOS os painéis

A régua curada acima era de **um** painel. Cinco dias depois, a varredura de elisões que mede os
**24 painéis do registo** (~3 000 medições) tinha o mesmo furo, e a constante dela **afirmava o
contrário por escrito**:

> *«Três, e não uma: a largura do encaixe sai do viewport, e um rótulo que cabe a `1920` pode não
> caber a `1280`.»*

⭐ **A frase é falsa e uma medição de dois minutos di-lo:** os cinco cortes conhecidos dão
**exactamente o mesmo número** a `1280`, `1366` e `1920`. A largura de uma coluna docada vem do
**TOKEN** (`ChromeBands::DEFAULT`), e a janela só decide **onde** ela fica. As três janelas
compravam a largura do encaixe do **fundo** e mais nada — *três corridas para uma resposta*.

⚠️ **O resto da nota mandava a outra metade para um sítio que não existe:** *«elas não cobrem a
coluna que o artista aperta à mão — essa metade é do gate de coluna de cada painel»*. Não há gate de
coluna para 24 painéis; havia para **um**, que é o de cima.

**O que a escada achou à primeira corrida:** `129` cortes invisíveis, `80` deles no Inspector — e
**três controlos que pintavam NADA** (os botões `M`/`S` do mixer, os chips de variante e o campo de
um número do script), com o portão `nenhum_rotulo_do_app_pinta_nada` verde a `304` e a catraca dele
a **zero**.

⛔ **O degrau estreito sai da LEI, nunca do ficheiro do dono:** `PANEL_MIN_W_PX` (o piso até onde a
borda encolhe). O `~/.ph2d/layout.txt` vive fora do repositório e não existe noutra máquina — ele
serve para dizer **QUAL degrau importa** (e disse: a coluna esquerda do dono estava no mínimo em
**cinco dos seis** espaços de trabalho), nunca para **ser** o degrau.

⭐⭐ **A leitura das duas ocorrências juntas:** curar a régua de um painel não cura a família. *Quando
um gate mede uma superfície que o utilizador redimensiona, a pergunta não é «qual a largura certa?»,
é «esta régua varre uma ESCADA?»* — e o degrau que falta é sempre o estreito, porque é o único em
que alguém está.
