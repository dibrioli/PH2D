# A geodésica — a prova VISUAL antes do código

> **Estado:** a medição e as figuras existem; **nenhuma linha de produto mudou**.
> A wave só abre por ordem do dono, e a ordem depende do que estas figuras
> mostram (a promessa foi *«o primeiro que lhe mando é a foto, não código»*).

O instrumento é [`sonda_da_parede_fina.rs`](../../../crates/ph2d-sculpt3d/tests/it/sonda_da_parede_fina.rs),
`#[ignore]`, e corre-se assim:

```text
bash scripts/ph2d-run.sh cargo test -p ph2d-sculpt3d --test it \
  sonda_da_parede_fina -- --ignored --nocapture --test-threads=1
```

## O que se vê

| ficheiro | o que é |
|---|---|
| [`parede_fina_costas.png`](parede_fina_costas.png) | **a foto.** A barbatana vista POR TRÁS — o lado em que o artista não tocou —, com a cor a dizer quanto cada ponto andou. Hoje: uma mancha. Com a cura: nada. |
| [`parede_fina_corte.png`](parede_fina_corte.png) | o **mecanismo**: o corte de lado, com a face de baixo a acompanhar a de cima. |

Os `.svg` ao lado são a fonte (texto, diffável); os `.png` são só para abrir
depressa.

## Os números, pelo caminho do PRODUTO

Barbatana `2,0 × 2,0` com **`0,06` de espessura**, pincel `R = 0,40` a força
`1,00`, carimbo a `0,30` da beira — *o gesto normal de quem esculpe uma orelha*.

| | |
|---|---|
| frente → costas **pelo ar** | `0,060` — `0,15 ×` o raio |
| frente → costas **pela superfície** | `0,660` — **`1,65 ×` o raio** |
| a máscara que shipa (`ALCANCE_TECTO = 2,0`) só corta acima de | `0,800` |
| a **frente** andou | `0,0400` |
| as **costas** andaram | `0,0395` — **`98,8 %`** do que a frente andou |
| do movimento TOTAL do carimbo, o que cai onde a superfície não alcança | **`31,9 %`** |
| ⭐ **CONTROLO** — vértices da FRENTE que a cura tiraria | **`0`** |

## ⛔⛔ A PRIMEIRA medição corrigiu a pergunta, e a correcção é a sonda toda

A 1.ª redacção usou o **tubo** da
[`sonda_do_falloff_pela_superficie`](../../../crates/ph2d-sculpt3d/tests/it/sonda_do_falloff_pela_superficie.rs)
(menor `0,10`, pincel `0,40`) e leu as costas a andar **`68,8 %`** do que a
frente andou. Parecia o achado — e **não é um defeito**: ali a distância *pela
superfície* entre as duas paredes é `π × 0,10 = 0,314`, que é **`0,78 ×` o raio
do pincel** ⇒ *um pincel geodésico honesto TAMBÉM lhes tocaria*, só que com menos
peso. Eu tinha escrito ao lado do número, sem o medir, que «um carimbo honesto
não lhes tocaria».

⇒ **o defeito vive numa BANDA:**

```text
    ar(frente→costas)  <  R          o carimbo de hoje alcança
    superficie         >  R          um pincel honesto NÃO alcançaria
    superficie         <= 2 x R      a máscara que shipa TAMBÉM não corta
```

*Uma régua que mede fora da banda mede um produto que já está certo.*

### ⭐ E a peça que a habita é uma CHAPA, não um tubo

Num tubo de raio `m` a superfície mede `π m` e o ar `2 m` ⇒ a razão é
**`π/2 = 1,571` FIXA**, logo a banda é um intervalo apertado de espessura e o que
escapa é modesto. A varredura mostra-o:

| menor | parede | superfície | sup/R | costas % | veredito |
|---|---|---|---|---|---|
| `0,10` | `0,20` | `0,314` | `0,78` | `68,8 %` | a superfície alcança |
| `0,13` | `0,26` | `0,408` | `1,02` | `43,7 %` | **defeito** |
| `0,15` | `0,30` | `0,471` | `1,18` | `26,2 %` | **defeito** |
| `0,19` | `0,38` | `0,596` | `1,49` | `1,4 %` | **defeito** |
| `0,25` | `0,50` | `0,785` | `1,96` | `0,0 %` | o ar não alcança |

Numa **chapa** a razão é **livre**: o ar é a espessura e a superfície é *ir até à
beira e voltar* (`2d + t`) ⇒ perto da beira ela cresce sem limite à medida que a
peça afina. É por isso que a foto usa uma barbatana e não o tubo.

## ⛔⛔ E porque a geodésica desta fixtura NÃO é o passeio por arestas

O passeio por arestas sobrestima até **`√2`** (numa grelha quadrada, ir a `45°`
custa `2n` arestas onde a superfície mede `n√2`). Usá-lo como régua da **cura**
cortaria vértices da **FRENTE** que estão dentro do raio, só por estarem na
diagonal — o painel verde mostraria a cura a comer o relevo do artista, e isso
seria artefacto do instrumento, não do desenho.

⚠️ *É exactamente o mesmo viés que obriga o tecto da máscara que shipa a ser
`2,00 × R`* — e é **por isso** que ela não apanha este caso.

Numa chapa a geodésica escreve-se à mão (`geodesica_da_barbatana`): recta no
plano para a frente, e **desdobramento** por cima de cada uma das quatro beiras
para as costas (a de `x = +M` manda `x ↦ 2M + t − x`). O mínimo das quatro é um
**limite inferior** — um caminho que contornasse um CANTO seria mais longo —, e a
direcção do erro é a que interessa: ele só pode fazer a cura cortar **menos**.

Com a troca, o que a régua classificava como inalcançável desceu de `532` para
**`270`** vértices: *o passeio por arestas estava a inflar o próprio achado.*

## ⚠️ O que estas figuras NÃO afirmam

* **Não** medem o preço da cura. O método do calor (Crane/Weischedel/Wardetzky
  2013) é a wave, e o substrato dela já existe no repo (Laplaciano cotangente e
  áreas duais em `ph2d-quadflow`, solver linear em `ph2d-gridmap`).
* **Não** afirmam que o tecto novo é `1,0 × R`. O painel da cura usa `1,0` porque
  é a definição de *«o pincel alcança»*; o número que shipa sai de um vale
  medido com o lado aprovado dentro, como o `2,00` de hoje saiu.
* **Não** dizem quantas peças reais caem na banda. A barbatana é construída para
  a habitar; a frequência no trabalho do dono é pergunta dele.
