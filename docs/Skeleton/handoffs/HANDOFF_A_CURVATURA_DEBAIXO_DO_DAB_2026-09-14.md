# A CURVATURA DEBAIXO DO DAB — o *«quase bom»* tinha número, e não era inevitável

> **Report do dono**, 2026-09-14, 4.ª foto do mesmo pincel: *«quase bom»*, com uma seta sobre a
> marca pintada no ponto de maior dobra do braço.
>
> **Estado:** ✅ fechado. `15 191` testes, `0` falhas. Clippy limpo nas cinco crates tocadas.
> **Commits:** `fe2ade324` (a medição) · `54719dec6` (a pegada curva) · `283739e0c` (a cadeia) ·
> `a1482a56b` (clippy) · `e0383f71d` (o gate do elo).

## §1 — O que este módulo afirmava, e o que a medição fez com isso

O cabeçalho do `sprite_mesh_warp` dizia, desde a wave anterior:

> *«⏳ O que SOBRA depois disto (`≈1,1`–`1,2` no pior regime) é inerente a **uma elipse por dab**:
> sobre um footprint em que a deformação varia, nenhum afim único a descreve. Os dois diminuidores
> são a malha mais fina (o `Smooth`) e o pincel menor — é uma troca de resolução, não uma parede.»*

⛔⛔ **A primeira metade está REFUTADA.** Variando **só** a contagem de triângulos, a redondeza da
marca (`1` = disco) **estanca**:

| triângulos | dab pequeno | dab grande |
|---|---|---|
| `32` | `1,347` | `1,322` |
| `128` | `1,129` | `1,160` |
| `512` | `1,101` | `1,119` |
| `2 048` | `1,050` | `1,108` |
| `8 192` | `1,050` | **`1,107`** |

Quadruplicar de `2 048` para `8 192` não move o terceiro decimal. ⇒ **o resíduo não é facetagem**, e
o `Smooth` do esqueleto — que é exactamente essa alavanca — **não toca neste defeito**. A nota
mandava o artista usar um botão que não ia ajudá-lo.

⭐ A segunda metade era a pista certa pela razão errada: o resíduo cresce com o raio do pincel
porque ele é a **CURVATURA da dobra dentro do próprio dab**. O kernel avalia `t = |M · p|` por
texel com `M` **linear**, e uma dobra não é linear.

## §2 — A escada que decidiu o desenho

Medida na direcção que o motor **de facto** avalia (texel → ecrã), aproximando o mapa da malha por
um polinómio de grau `g` à volta do centro do dab:

| raio do dab | `g = 1` (hoje) | `g = 2` | `g = 3` |
|---|---|---|---|
| `0,06` | `1,054` | `1,010` | `1,008` |
| `0,125` | `1,118` | `1,021` | `1,005` |
| `0,20` | `1,197` | `1,055` | **`1,006`** |

⭐ **O `g = 1` da sonda reproduz o produto** (`1,054`/`1,118`/`1,197` contra `1,050`/`1,108`/`1,179`)
— o controlo que diz que ela mede o mesmo programa. O grau `2` deixa `5,5 %` no pincel grande, que é
onde o dono aponta; o `3` leva o pior caso a `0,6 %`.

## §3 — O resultado no produto

Pela mesma sonda que refutou a facetagem:

| triângulos | antes | **depois** |
|---|---|---|
| `128` | `1,129` · `1,161` | `1,129` · **`1,069`** |
| `512` | `1,101` · `1,119` | **`1,067`** · **`1,015`** |
| `2 048` | `1,050` · `1,108` | **`1,010`** · **`1,007`** |
| `8 192` | `1,050` · `1,107` | **`1,003`** · **`1,002`** |

⭐⭐ **E agora ele CONVERGE com a malha**, que antes não convergia: a facetagem deixou de ser o chão
porque o que sobrava por cima dela era outra coisa.

## §4 — Onde cada peça vive

| Crate | O que entrou |
|---|---|
| `ph2d-render` | `warp_over` devolve `MeshWarp { linear, curve }` — ajuste de grau `3` em `f64` sobre `2` anéis × `12` ângulos (`sprite_mesh_fit.rs`) |
| `ph2d-painter-brush` | `FootprintCurve` · `CanvasWarp` · `WarpedDab::curve` · `BrushSpec::dab_curve` · `canvas_warp_curve.rs` (a composição) |
| `ph2d-tool-painter` | `set_canvas_warp` recebe o portador; `device_dabs` publica a parte **linear** com a curvatura ao lado |
| `ph2d-paint-gpu` | `GpuDab` `64 → 128` bytes; o `stamp.wgsl` soma os monómios |
| `shells/desktop` | a **única** conversão entre os dois gémeos |

⭐ **`footprint_deform()` é UMA porta**, e é isso que faz o falloff, a silhueta do *Shape*, o Grão e
as bandas herdarem a dobra sem uma linha em cada um deles.

## §5 — As DUAS cercas, as duas medidas, as duas degradam para a elipse

1. **`FACETAS_MIN = 8`.** Abaixo de `8` facetas distintas sob o dab, o que o ajuste lê como
   curvatura são as **arestas** das facetas. Varrido contra a linha de base (só a recta):

   | triângulos | só a recta | piso `4` | piso **`8`** | piso `12` |
   |---|---|---|---|---|
   | `32` | `1,316` · `1,312` | `1,133` · `1,453` ⛔ | `1,316` · `1,312` | `1,316` · `1,312` |
   | `128` | `1,129` · `1,161` | `1,095` · `1,069` | `1,129` · **`1,069`** | `1,129` · `1,161` ⛔ |
   | `512` | `1,101` · `1,119` | `1,067` · `1,015` | **`1,067`** · **`1,015`** | `1,101` ⛔ · `1,015` |

   ⇒ `8` é o **menor** piso em que nenhuma célula fica pior que a recta, e `12` já deita fora ganhos.
2. **`is_sampleable()`.** Uma dobra violenta faz o mapa dobrar sobre si mesmo dentro do dab: o
   `|apply|` deixa de crescer ao longo de um raio e tanto o amostrador como o anel leem o cruzamento
   errado. Medido: com o termo de grau `2` a valer `61 %` do linear, a marca sai a **`0,994`** contra
   **`0,611`** sem correcção nenhuma — *o defeito da W11b um grau acima*. Com a cerca: exactamente
   `0,611`.

## §6 — O preço, medido

| triângulos | a deformação num PONTO | ao tamanho do dab |
|---|---|---|
| `128` (o `Fast`) | `0,07 µs` | `0,83 µs` |
| `2 048` | `0,78 µs` | `11,65 µs` |
| `7 688` (o tecto do `Smooth`) | `3,06 µs` | `43,38 µs` |

⭐ **Triplicar as amostras (`8` → `24`) custou `~7 %`** (`40,7` → `43,4 µs`): *o recurso é a varredura
dos triângulos, não a contagem de amostras*. Pior caso: `0,26 %` de um quadro de 60 fps.

## §7 — As SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **O anel do cursor não mudou, e está certo.** Ele desenha a elipse **autorada**; a lei diz que o
   que o motor pinta, visto pela dobra, **é** essa elipse. Um anel curvo seria o erro de 2026-09-14
   outra vez.
2. **`CanvasWarp` e `MeshWarp` são gémeos DE PROPÓSITO.** `ph2d-painter-brush` é **dev-dependency**
   da `ph2d-render`; o tipo dela não pode atravessar em produção, e uma aresta de release do
   renderer para o motor de pincel puxaria o pincel inteiro para o grafo de desenho.
3. **O atalho da pegada plana é uma cerca de CUSTO, não de bits.** Somar sete zeros é exacto em
   IEEE, e a mutação que o apaga **sobrevive** a todos os gates. Ele tira `14` multiplicações-e-soma
   por texel de toda pincelada sobre arte não dobrada.
4. **A rotação `V` não é decoração.** A decomposição entrega `(raio, flatten, ângulo)`, de que o
   motor reconstrói `L_dab`; essa matriz e a parte linear do mapa exacto descrevem a **mesma elipse**
   e **não são a mesma matriz** — diferem por uma rotação à esquerda que a **norma não vê**. Enquanto
   a pegada foi linear isso nunca incomodou; com curvatura, somar `K` cru seria somar termos de dois
   referenciais. Mutação `V = I`: **vermelha**.
5. **`device_dabs` deixou de tirar `m0`/`m1` dos vectores da base.** `apply([1,0])`/`apply([0,1])` só
   determinam uma matriz enquanto o mapa for linear; com curvatura eles trazem os monómios avaliados
   nos versores **dentro** deles, e o device carimbava uma forma que ninguém autorou **sem um erro em
   lado nenhum**.
6. **O gate da `ph2d-paint-gpu` não apanha o item 5.** Ele reproduz a fórmula do shader a partir de
   `linear_rows()`/`curve_in_input_frame()`, logo mede a **lei** — e ficaria verde sobre uma ponte que
   publicasse outra coisa. *Um gate que reproduz a publicação não lê quem publica.* Daí o gate irmão
   na `ph2d-tool-painter`.

## §8 — As QUATRO premissas minhas que a medição derrubou

1. *«O que sobra é inerente a uma elipse por dab; a malha mais fina diminui-o»* — **refutado** (§1).
2. *«A região do ajuste é a variável: ele corre sobre o círculo de repouso e o que se pinta é a
   elipse `W⁻¹·E`, até `2,1×` mais comprida»* — construído e **refutado**: uma iteração de ponto
   fixo devolve `1,161` contra `1,162`. **Ganho zero**, e nas doze células tanto melhora como piora.
3. *«O atalho da pegada plana é o que mantém os bits»* — **falso**: é uma cerca de custo (§7.3).
4. *«Uma busca por o melhor que uma elipse consegue mede o tecto»* — **não mede**: ela devolveu
   `1,068` contra `1,108` do produto com a escala do vencedor em `0,70` contra `1,16`. *Ela comprava
   redondeza **encolhendo** o dab*, e a resposta dela teria mandado afinar o ajuste em vez de subir
   o grau. ⛔ **Uma busca que deixa o TAMANHO flutuar não mede um tecto — mede outra pergunta.**

## §9 — E uma RÉGUA minha estava errada, do mesmo jeito que este repo já pagou

A 1.ª redacção do gate da identidade comparava os dois contornos por **vizinho mais próximo**, e lia
`0,062` sobre uma composição que a álgebra diz ser **exacta** — ela media a **desigualdade das duas
amostragens**, que uma dobra forte torna muito não-uniforme. *Uma régua que compara dois conjuntos
por vizinho mais próximo mede o espaçamento deles, não a lei.* ⇒ a lei é **pontual**: todo ponto da
fronteira pintada, visto pela dobra, está **sobre** a elipse autorada.

## §10 — O que fica ABERTO

- ⏳ **O chrome que é CAMINHO** (a grelha, os contornos de selecção, a curva e a linha do Painter):
  eles precisam de ser **subdivididos** para seguir a malha — é trabalho de outra natureza, e não foi
  começado.
- ⏳ **F8 — Bendy Bones**, na fila por começar.
- ⏳ **F4 — *«undo tem poucos passos»***, que não reproduz.
- ⏸️ **Ligar o `Smooth` de fábrica** — decisão do dono, com o custo medido (`1/10` de um quadro,
  `1 543` peças), surfaceada duas vezes e ainda sem resposta.
