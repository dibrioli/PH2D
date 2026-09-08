# 09 — O tecido inteiro nas mãos do artista

> **Ordem do Enio, 2026-09-08:** *«faça o seu melhor para tornar o tecido
> completamente customizável pelo usuário a ponto de conseguirmos resultados
> melhores que o blender»* — depois da pergunta que abriu o buraco: *«cloth
> filter tem as propriedades do tecido? Já foram implementadas para cloth
> filter?»*

Resposta medida à pergunta: **em parte, e por EMPRÉSTIMO.**

---

## §1 — O defeito, em três metades

O filtro lia `brush.cloth_mass`, `brush.cloth_damping` e
`brush.cloth_plasticity` — os números do **pincel**. Na referência são
**propriedades separadas**, e duas delas têm valores diferentes:

| | pincel (espec §8.1) | filtro (espec §7) | o que se passava |
|---|---|---|---|
| amortecimento | omissão `0,01`, faixa `0,01..1` | omissão **`0`**, faixa `0..1` | ⛔ **o filtro nunca alcançava o próprio valor de omissão** |
| plasticidade | do artista | o alvo **fixa `0`** | passávamos o do pincel, em silêncio |
| massa | `1,0` | `1,0` | ✅ |

⚠️ **E os três eram INALCANÇÁVEIS.** A linha do painel pergunta *«o pincel de
tecido está na mão?»* (`verb == Verb::Cloth`), e desde a W9b o filtro corre com
**qualquer** verbo. Com o Draw na mão eles mexiam na simulação sem nada na tela
os mostrar — **vivo e inalcançável**, o espelho do knob morto, e a espécie que
nenhuma sonda deste repo vê.

## §2 — ⛔⛔ Por que a bancada de 17 traços não podia ver nada disto

Duas razões, e as duas valem mais que o defeito:

1. **A bancada monta o `Pincel` da lei directamente do cabeçalho de cada
   fixture** — ela nunca passa pelo mapeamento do produto. Ela julga a LEI;
   **ninguém julgava a tradução** *«os botões do painel → os números do
   solver»*.
2. **A plasticidade nasce em `0`**, que é exactamente o valor da espec. *Uma
   fixtura no ponto neutro de um knob não testa esse knob.*

---

## §3 — A cura estrutural: o filtro deixou de RECEBER um pincel

`SculptStroke::cloth_filter_begin` já não tem um `&Brush` na assinatura; ele
recebe [`ClothFilterProps`]. *Não é possível ler por engano um campo que não
chega* — e é isso que faz a lei valer para o **próximo** campo de tecido que
alguém acrescente ao pincel, sem se lembrar dela. Gate:
`o_filtro_nao_recebe_um_pincel`.

---

## §4 — ⭐⭐⭐ O que o artista ganhou

### Do FILTRO (dele, e não do pincel)

`Filter Mass` · `Filter Damping` · `Filter Plasticity` · `Filter Quality` ·
`Filter Collisions` · `Force Axis` (X/Y/Z)

### Do PINCEL

`Cloth Quality`

### E o que a referência NÃO dá: `Quality`

O alvo **fixa** as varreduras de relaxação em `5` e não as oferece. É o maior
controlo que este pano ganha sobre o dele, e o que ele compra está **medido** —
o esticão máximo de um aperto sobre uma grelha `21×21`, doze passos:

| varreduras | esticão máx | `p95` | ms |
|---:|---:|---:|---:|
| `1` | `1,677` | `0,893` | `0,6` |
| **`5`** (o alvo) | **`0,188`** | `0,100` | `1,9` |
| `8` | `0,066` | `0,051` | `3,0` |
| `16` | `0,050` | `0,038` | `6,3` |
| **`32`** | **`0,039`** | `0,028` | `11,4` |
| `64` | `0,090` | `0,023` | `22,5` |

⇒ **de `5` para `32` o pior esticão cai `4,9×`**, que é o regime em que o pano
rasga.

⛔⛔ **O teto de `32` é MEDIDO:** a `64` o pior caso **piora** (`0,039 → 0,090`)
enquanto o `p95` mal se move. Passado o joelho, o que sobra é tempo.

⚠️⚠️ **E ele NÃO é «mais é melhor»:** na Escala o ótimo medido é o `5` do alvo
(`0,058`) e `8..64` **pioram** (`~0,076`). Por isso o rótulo é *Quality* e não
*Stiffness*, e a omissão é a do alvo. *Um botão rotulado «qualidade» que às
vezes piora é um botão que mente.*

⚠️ **O recurso é TEMPO e cresce com a MALHA:** na malha do smoke (`98 306`
vértices) cada varredura vale `~8,7 ms`, então `5` já custa `47,5 ms` contra um
quadro de `16,7`. *Quem quiser as 32 reduz a malha primeiro — o botão de
retopologia existe.*

---

## §5 — ⛔ A fixtura que não produzia o fenómeno (a quarta desta jornada)

A **primeira** sonda das varreduras usou **gravidade sobre um plano** e leu
`0,00000` de esticão em **todas** as contagens, de `1` a `64`. Um plano em queda
livre é uma **translação rígida**: nenhuma restrição é violada e a relaxação não
tem o que corrigir.

⚠️ **E a segunda régua estava errada em dois dos quatro modos:** *«distância ao
comprimento de repouso»* só significa *violação de restrição* onde o pano deve
manter os comprimentos (aperto, gravidade, agarrar). Na **Escala** e no
**Expand** a mudança de comprimento **é** a deformação pedida — ali a régua
mede o produto, não o erro.

⇒ o gate mede cada propriedade **no gesto em que ela morde**, com o motivo
escrito ao lado.

---

## §6 — As colisões, que a espec dizia existirem

A espec §7 diz que o filtro **tem** colisões (*«idem §5.6, opção nasce
desligada»*) e nós passávamos-lhe uma lista **vazia**. A construção dos
colisores eram ~40 linhas **inline** no traço ⇒ viraram uma **porta**
(`stroke_cloth_ref::caixas_de`) com dois consumidores. *Copiá-las daria duas
ideias de «onde este vértice bate», e a que envelhecesse atravessaria o
obstáculo em silêncio.*

Medido: queda livre chega a `−33,80`; com um obstáculo no caminho o pano **para
em `−1,35`**.

⚠️ **Nasce desligada, e o preço é a razão:** `2,6×` a `6,1×` o custo de um dab, e
**no filtro a peça inteira é o pior caso** — não há banda a limitar quem colide.

⛔ **Divergência declarada, a mesma do traço:** o alvo dá ao raio da colisão uma
**espessura** (`0,3`) e o nosso lança raio fino. Não existe amostra do alvo com
obstáculo, logo esta parte **não tem lado aprovado**.

---

## §7 — A paridade não se mexeu

**Toda omissão é a do alvo**, byte a byte. É isso que faz os seis controlos
serem uma **adição** e não uma quebra: os `86` traços do pincel e os `17` do
filtro dão exactamente o que davam.

---

## §8 — ⏳ O que fica, e o gatilho

| item | porquê | gatilho |
|---|---|---|
| **gravidade da CENA** | a escultura desta casa não tem esse ajuste | o dia em que ela ganhar um |
| **conjuntos de faces** | o app não os tem | idem |
| a **flambagem** do Expand | `0,688534` — a quantidade está certa, o padrão não | quem quiser o número |
| a **espessura** do raio de colisão | divergência declarada, e sem lado aprovado | uma amostra do alvo com obstáculo |
