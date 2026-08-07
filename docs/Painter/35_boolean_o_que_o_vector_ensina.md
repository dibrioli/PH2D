# 35 — O boolean do Painter, e o que o módulo Vector ensina sobre ele

> Pergunta do Enio, 2026-08-06: *"deve haver meios de otimizar o boolean do painter. Veja como o
> módulo vector faz o boolean não destrutivo para runtime dele e veja se descobre como tornar o
> painter mais otimizado."*
>
> Este doc é a **resposta medida**. Ele traz (§1) o que o composite do Painter faz hoje e de que os
> milissegundos são feitos, (§2) o que o Vector faz e quanto custa a MESMA pergunta pela rota dele,
> (§3) a otimização byte-idêntica que já shipou nesta linha, e (§4) a rota analítica — com o preço,
> os riscos medidos e a decisão que **é do Enio**, porque muda o DESENHO.

---

## 1. O que o Painter faz hoje

`stroke_boolean_contours` roda **a cada move de ponteiro** (todos os cinco `*_refill` caem em
`restamp_shapes_preview`, que o chama sem cache), e ele **não é um overlay**: o contorno que sai vira
dabs pelo `fill_polyline_preview` e é depositado. O que o artista vê **é** o resultado.

O caminho é:

```
formas autoradas  →  SelectionShape  →  rasteriza a SS=3  →  flood por componente
                                                          →  traça (Moore)  →  polilinha densa  →  dabs
```

### 1.1 A decomposição, medida no código que SHIPA

`measure_what_the_boolean_is_made_of` (`ph2d-tool-painter`, `-- --ignored`), Ellipse de **200 px**,
Digital, canvas 4096², máquina calma:

| formas | converte | rasteriza | traça | TOTAL | células | pts |
|---|---|---|---|---|---|---|
| 1 | **0,000** | 1,671 | 4,777 | **6,448** | 1 444 920 | 3 400 |
| 2 | 0,001 | 5,595 | 8,813 | **14,409** | 2 606 196 | 5 351 |
| 4 | 0,000 | 7,399 | 8,912 | **16,311** | 3 313 440 | 6 786 |

⚠️ **A linha que decide tudo é a primeira coluna: converter custa ZERO.** A geometria que o artista
autorou já está em mãos — uma elipse são quatro cúbicas — e o composite **joga isso fora** para
redescobrir a mesma forma a partir de 1,44 M de células de pixel.

⚠️ **E a coluna `pts` é o preço a jusante:** o traçado devolve **3 400 pontos** para descrever uma
elipse que chegou como **4 segmentos**. Cada ponto é posição de dab.

⚠️ **Com UMA forma marcada o composite é a IDENTIDADE** — não há com quem combinar, a região É a
forma —, e ainda assim custa 6,45 ms. Esse é o caso mais comum, porque `active_is_bool` põe a figura
ativa no composite assim que o artista escolhe a Operation.

---

## 2. O que o Vector faz

`ph2d-vec-boolean` **não rasteriza nada**. A porta única é
`engine::binary_grouped_checked(a: &BezPath, b: &BezPath, rule, op)`, que chama
`linesweeper::binary_op` — uma varredura **exata** sobre as curvas — e devolve contornos já
**orientados** e **agrupados por containment**. O custo é `O(segmentos)`; a tela não entra na conta.

Três decisões desse crate que valem para nós:

1. **Uma passagem só para o motor** (`binary_grouped_checked`), e é isso que faz a guarda de entrada
   cobrir também o Expand e o Shape Builder, que não sabem que ela existe.
2. **A guarda de finitude é NOSSA, não do motor** — medido pelo `line/Vector`: uma coordenada `NaN`
   faz o `linesweeper` **PANICAR** (`geom.rs:63`) em vez de devolver o `Error::NaN` que ele declara,
   porque o `binary_op` só examina o *bounding box* e `min`/`max` com NaN devolvem o outro operando.
3. **O fechamento tem de ser EXATO.** O `to_bez_with(.., Closing)` existe por isso.

### 2.1 Quanto custaria aqui — medido

Sonda descartável (o código está no §5, para reprodução), duas elipses de 200 px:

| rota | 2 elipses sobrepostas | contornos | pts |
|---|---|---|---|
| raster (o Painter hoje) | **14,409 ms** | 1 | 5 351 |
| sweep exato (o Vector) | **0,090 ms** | 1 | **112** |

**160× no tempo, 48× nos pontos** — e os 112 pontos são um achatamento a **1/6 px** de tolerância,
ou seja **mais preciso** que o traçado de raster, que quantiza em 1/3 px por construção.

### 2.2 A robustez, medida antes de recomendar

O Painter roda isto a 60 Hz sobre a geometria que a mão do artista produz, então a pergunta não é só
velocidade. Cinco casos degenerados × três operações:

| caso | Union | Difference | Intersection |
|---|---|---|---|
| freehand auto-intersectante | ok | ok | ok |
| elipse degenerada `rx = 0,5` | ok | ok | ok |
| duas coincidentes | ok | ok | ok |
| tangentes exatas | ok | ok | ok |
| forma de área zero | ok | ok | ok |

**15/15 sem pânico e sem erro.**

⚠️ **Mas há um achado que não é sobre falhar:** *coincidentes → Union* devolve **2 grupos** e
*tangentes exatas → Union* devolve **4**, onde a resposta geométrica é um contorno. O raster **não
tem** esse modo de falha (pixels não dobram). Sob um pincel de baixa opacidade um contorno emitido
duas vezes **aparece**. Quem for construir a rota analítica precisa deduplicar ou aceitar isso —
e isso é trabalho, não detalhe.

⚠️ **E a minha primeira medição de robustez estava ERRADA, com todas as quinze linhas dando o mesmo
erro** — o que já era o sinal: um erro idêntico em cinco formas diferentes é um erro sobre o que elas
COMPARTILHAM, e o que elas compartilhavam era o meu helper `ellipse()`, cujo `kurbo::Ellipse::to_path`
fecha a ~`1e-14` do ponto inicial. O motor exige fechamento exato. A tabela acima é a de depois.

---

## 3. O que shipou agora (byte-idêntico)

A decisão de trocar de rota é do Enio (§4). O que **não** depende dela é o desperdício dentro do
traçado, e ele era grande: `traça` era o maior item da tabela.

Três coisas saíram de `trace_all_contours`, **sem mover um ponto do contorno**:

- o buffer `comp` era `vec![0u8; w*h]` **por componente** — 1,44 MB alocados e zerados a cada blob;
  agora é **um só**, escrito e apagado pelas FAIXAS do componente;
- a busca pelo próximo blob recomeçava do índice **0** a cada volta; agora **retoma** do último
  começo, o que é exato (tudo antes dele já era zero quando ele foi achado);
- o `trace_contour_raw` varria o buffer inteiro de novo só para redescobrir o pixel que o laço tinha
  acabado de achar; agora o começo é **entregue** (`trace_contour_raw_from`).

E o flood pixel-a-pixel virou **varredura por faixas**: uma elipse de 1200 linhas custa 1200 entradas
em vez de 1,13 M de índices numa pilha.

⚠️ **A conectividade é a mesma (4-conexa), e isso não é detalhe:** o traçador de Moore é 8-conexo,
então dois blobs que se tocam só na DIAGONAL são componentes separados no flood e um único contorno
no traçado. Trocar a conectividade mudaria **quantos** contornos o composite entrega.

**Medido** (`measure_the_span_scan_against_the_pixel_flood`), as duas rotas cronometradas **dentro da
mesma corrida** sobre a mesma máscara — que é a única forma honesta nesta máquina, porque ela é
compartilhada e a carga vira fator comum:

| cena | células | flood | faixas | razão |
|---|---|---|---|---|
| 1 disco (SS=3, r=200) | 1 440 000 | 3,908 | **1,454** | **2,69×** |
| 3 discos | 2 160 000 | 3,592 | **1,281** | **2,80×** |
| anel (com buraco) | 1 440 000 | 2,548 | **1,173** | **2,17×** |

E o composite inteiro, pela mesma sonda do §1.1 (as duas corridas com a máquina calma — `load` 0,26 e
3,09 —, mas **corridas diferentes**, então quem carrega o veredito é a razão in-run acima, não esta
tabela):

| formas | rasteriza | traça | TOTAL |
|---|---|---|---|
| 1 | 1,880 | 4,777 → **2,051** | 6,448 → **3,931** |
| 2 | 5,910 | 8,813 → **3,605** | 14,409 → **9,515** |
| 4 | 7,455 | 8,912 → **4,147** | 16,311 → **11,602** |

⚠️ **Com isso o `rasteriza` passou a ser o maior item** — e é ele que a §3.1 ataca.

**Gate:** `the_span_scan_traces_what_the_pixel_flood_traced` compara contra a rota antiga **congelada
sob `cfg(test)`** (`trace_all_contours_flood`) — o precedente do `warp_axis`, do `serial_side` e da
própria rota de tela cheia do boolean. A fixture contém os três casos que separam as rotas: o blob
que **encosta na borda** (onde `sy == 0` faz o vizinho de cima estourar), os blobs que se tocam **só
na diagonal**, e a forma **côncava** (mais de uma corrida por linha, que é o que obriga a semeadura
por-corrida a estar certa). Irmão: `a_reused_buffer_does_not_leak_the_previous_component`, que só
pode falhar a partir do **segundo** blob — daí a fixture ter três em fila.

### 3.1 E cada forma passou a rasterizar na CAIXA dela, não na união

A janela do composite já era a das formas. Dentro dela, porém, **cada forma pagava a união inteira
três vezes** — zerar o `region`, avaliar a elipse em cada texel, compor —, e a união é o que as
*outras* formas ocupam. Com quatro figuras que mal se tocam, isso é quatro vezes a união para desenhar
quatro caixas disjuntas.

Agora cada forma rasteriza numa sub-janela do tamanho da própria caixa (`window_sub_rect`), e o
`combine_into` é chamado por LINHA sobre a fatia correspondente do `crisp`.

⚠️ **A troca é BYTE-IDÊNTICA por aritmética, não por medição:** fora da caixa a `region` é zero, e
`max(c, 0) == c` (união) e `(c · (255 − 0)) / 255 == c` (subtração) **em inteiros, exatamente**. É por
isso que a rota curta é oferecida só a esses dois wires — o wire 0 (Overlay) é um `copy_from_slice`,
que **zeraria** tudo fora da caixa. Nenhum chamador o passa hoje (os dois filtram Overlay antes), e a
guarda existe **em vez de** uma nota dizendo que ninguém passa.

**O A/B, na MESMA corrida** (`measure_the_sub_rect_against_the_full_window`) — a rota de janela cheia é
o `else` do mesmo laço, viva no produto para uma forma sem caixa, e as duas são cronometradas
alternadas sobre o mesmo estado, porque esta máquina é compartilhada e comparar duas corridas
atribuiria a deriva dela ao ganho:

| formas | janela cheia | caixa da forma | razão |
|---|---|---|---|
| 1 | 1,697 | **1,685** | **1,01×** |
| 2 | 5,652 | **3,259** | **1,73×** |
| 4 | 7,342 | **3,027** | **2,43×** |

⚠️ **A linha de UMA forma é o negativo honesto, e estava previsto pelo modelo:** com uma figura só, a
caixa dela **É** a janela — não há o que economizar. E essa é a cena mais comum, porque `active_is_bool`
põe a figura ativa no composite assim que o artista escolhe a Operation. O ganho é dos casos com
várias formas, e cresce com elas exatamente porque o que ele remove é *cada uma pagar a união*.

⚠️ **O `PAD` de um texel é FOLGA DECLARADA, e a mutação que o zera SOBREVIVE de propósito.** Eu o
justifiquei primeiro por dois alcances que a leitura desmentiu — o `max(0.5)` de texel do
`rasterize_ellipse` é **inalcançável por esta porta** (o `stroke_state_to_fill_shape` já clampa em meio
px de IMAGEM, `SS` vezes maior), e `round(v)` fica em `[floor(v), ceil(v)]` **por definição**, então um
span nunca sai da caixa. Com `PAD = 0` ela já seria exata. A folga fica porque a primeira premissa mora
em **outra função**: afrouxado aquele clamp, com folga o preço é um texel de trabalho a mais, e sem
folga é **forma truncada em silêncio**.

**Gates:** o mesmo `the_boolean_window_draws_what_the_whole_canvas_window_drew`, contra a rota de tela
cheia **congelada** — a fixture ganhou a forma **degenerada** (semi-eixo mínimo), onde um off-by-one na
caixa é uma fração grande do desenho em vez de um texel perdido numa figura de 40 px. **Três mutações
sangram** (caixa um texel menor · a linha esquece o `ry` e compõe nas linhas erradas · rasterizar com a
origem da JANELA em vez da sub-janela); a quarta é o `PAD`, acima.

**O composite inteiro**, medido em corrida separada (contexto, não veredito — o número que carrega a
conclusão é a razão in-run acima):

| formas | rasteriza | traça | TOTAL | (antes desta sessão) |
|---|---|---|---|---|
| 1 | 2,17 | 2,15 | **4,31** | 6,45 |
| 2 | 3,56 | 3,66 | **7,22** | 14,41 |
| 4 | 3,50 | 3,99 | **7,49** | 16,31 |

⚠️ **E aqui a rota de raster acaba:** com as duas fases equilibradas e a janela já sendo a figura, o que
sobra é `O(área da figura)` nas duas — irredutível *para este método*. Descer mais é a rota do §4, que
muda o DESENHO.

---

## 4. A rota analítica — o que ela custa construir, e a decisão

**Não construída.** Ela muda o DESENHO (contorno analítico no lugar do traçado de raster suavizado),
e a regra desta casa é que o look é do artista.

O que ela exigiria, nomeado:

1. **Uma aresta de dependência nova.** `ph2d-tool-painter` → o motor exato. O caminho limpo **não** é
   copiar o `engine.rs` (segunda porta para a mesma pergunta); é uma função **aditiva** e de dado
   simples em `ph2d-vec-boolean` — pontos + alças + `closed` + a Operation entrando, polilinhas
   saindo — com `kurbo`/`linesweeper` **contidos lá dentro**, sem chegar ao Painter. Aditiva porque
   `line/Vector` trabalha nesse crate: acrescentar um `pub fn` funde; mover o `engine.rs` não.
2. **A dedup dos contornos degenerados** (§2.2).
3. **O fechamento exato** na conversão (§2, item 3).
4. **Um smoke**, porque a borda muda.

O que ela entrega: **14,4 → 0,09 ms** e **5 351 → 112 pontos**, com o custo deixando de depender da
área e passando a depender só da geometria que o artista autorou.

⚠️ **E o caso da IDENTIDADE é o mais forte dos argumentos:** com uma única forma Add, a rota
analítica devolve a própria curva — que é exatamente o que o mesmo shape em **Overlay** já pinta.
Hoje as duas pintam diferente, e a diferença não é nada que o artista tenha pedido.

---

## 5. Reprodução

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
cargo test -p ph2d-tool-painter --release --lib measure_what_the_boolean_is_made_of \
  -- --ignored --nocapture --test-threads=1
```

⚠️ **Com `load average` acima de ~5 este número não fala sobre o código** — medido nesta própria
sessão: a coluna `rasteriza`, que ninguém tocou, foi de 1,671 a 3,782 ms entre duas corridas só
porque a máquina estava a 30.

A sonda do motor exato (§2.1 e §2.2) foi um arquivo descartável em
`crates/ph2d-vec-boolean/tests/sweep_probe.rs`, **removido depois de medir** para não deixar
superfície de merge num crate de outra linha. Para refazê-la: uma elipse construída com quatro
cúbicas de fechamento EXATO (o `to_path` da kurbo não serve — §2.2), `linesweeper::binary_op` com
`FillRule::NonZero`, e o achatamento a `1.0/6.0` para contar pontos.
