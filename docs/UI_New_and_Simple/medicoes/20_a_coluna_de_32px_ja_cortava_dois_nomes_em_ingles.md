# 20 — A coluna de 32 px já cortava dois nomes EM INGLÊS

> **Medido em 2026-09-18, `line/UIUX`.** A 17.ª fatia do HR-15 — e a primeira em que a foto do dono
> mostrava um defeito que ele **não** reportou: ele escreveu *«smoke OK»*.

## §1 — O que a foto dele mostrava

A foto do Audio Mixer com `PH2D_LANG=teste` trazia a fileira de barras do master com **o nome de
quase todas em `[…]`**. Ele aprovou o smoke — a pergunta dele era outra (o `EQ`) —, e o corte ficou
na imagem à espera de alguém o medir.

## §2 — ⛔⛔ Medido, e o defeito já shipava na língua em que o app corre

A coluna era o literal `FX_LABEL_W = 32,0 px`. Com o sistema de texto REAL, à fonte `Xs`:

| rótulo | inglês | idioma de teste |
|---|---:|---:|
| `Low` · `Mid` · `Fbk` | `18,8`–`21,3` | `29,9`–`31,5` |
| `Size` | `20,6` | **`33,8`** |
| `High` | `24,9` | **`38,2`** |
| `Time` | `25,9` | **`39,1`** |
| **`Depth`** | **`32,3`** | **`48,5`** |
| **`Return`** | **`35,9`** | **`55,1`** |

⇒ **dois cortados hoje, em inglês**, e **cinco de oito** sob o idioma de teste — que é o **p90 medido
de 89 832 pares de tradução reais** em cinco línguas (`pseudo.rs`). ⚠️ *Não é uma previsão pessimista:
é o que nove em cada dez traduções fazem.*

⭐ **E o mecanismo tinha nome antes de eu medir:** o gate irmão do painel de camadas já escreve, sobre
a coluna de `60 px` dos chips dele, que *«um literal não cresce com o dock»*. Esta é a mesma doença,
no painel ao lado, dois dias depois.

## §3 — ⭐ A cura: a coluna DERIVA dos nomes que ela pinta

`coluna_dos_nomes` mede o mais largo dos rótulos que a fileira de facto pinta — a população é a
`const FX_ROW_KEYS`, para uma barra nova entrar na medição sem ninguém se lembrar dela —, com:

- **piso** = o `32` de sempre (senão uma secção só de `Low`/`Mid`/`High` puxava as barras e o bloco
  deixava de estar alinhado);
- **tecto** = metade da linha, porque *acima disso o artista lê mais do que arrasta*. ⚠️ **Ele não
  morde**: o pior nome do idioma de teste pede `55,1 px` e metade da linha mais estreita que o dock
  permite são `~98`. O tecto fica com a medição ao lado, para quem um dia o vir morder saber que é a
  lei e não um acidente.

⚠️ **UMA coluna para o bloco inteiro do master, não uma por secção** — ao contrário do painel de
camadas, onde as secções são cartões separados. Aqui elas são uma pilha estreita, e *«as labels
alinhadas todas à direita»* é ordem do dono.

## §4 — ⛔⛔ O gate nasceu VÁCUO, e o que o curou foi uma porta

A 1.ª redacção media o texto deformado contra uma coluna calculada **em inglês** — e acusava o
produto CERTO. A causa é a de sempre nesta casa: o `tr` lê o idioma de um `OnceLock` sobre o ambiente
do PROCESSO, que numa suíte está sempre em inglês.

⇒ a lei ganhou o parâmetro (`coluna_dos_nomes_em(idioma, …)`) e a função de sempre passou a ser o
acessório que lhe entrega o idioma do ambiente — o par `tr`/`tr_em` outra vez. **A coluna de uma
língua mede os nomes DESSA língua**, que é o que o app faz em execução e o que o gate não conseguia
dizer.

⭐ E o gate tem **duas metades**: a lei, e o **CONTROLO** que fixa a fixtura desta wave — *a coluna
literal de 32 px corta exactamente `Depth` e `Return` em inglês*. Sem ele, uma coluna larga por
acidente passaria sem que a medição dos nomes tivesse nada a ver com isso.

## §5 — ⚠️ O que não deu para verificar, e porquê

A fotografia não alcança: o Audio Mixer vive numa **aba atrás do Inspector** na arrumação do dono
(`~/.ph2d/layout.txt`, fora do repo), e os eventos sintéticos não chegam à tela virtual — trazê-lo à
frente é um clique. *A prova é a medição, feita com o mesmo sistema de texto e a mesma fonte que o
pintor usa.*

## §6 — Os números

| | |
|---|---:|
| rótulos que cortavam em inglês | **2** de 12 |
| rótulos que cortavam no idioma de teste | **5** de 12 |
| depois da cura | **0** em ambas, nas 4 larguras do dock |
| portas novas | **2** (`coluna_dos_nomes` · `coluna_dos_nomes_em`) |
| gates novos | **1** (2 metades: a lei · o controlo da fixtura) |
| prova de mutação | cravar a coluna no literal devolve os seis cortes, com `Depth`/`Return` em INGLÊS |
| suíte | `ph2d-panel-audio-mixer` 31 · clippy `--workspace -D warnings` **0** |
