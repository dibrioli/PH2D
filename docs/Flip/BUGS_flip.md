# Bugs do módulo Flip — registro + soluções

> Log vivo dos bugs **não-triviais** do Flip (sintoma → causa-raiz → tentativas que falharam →
> solução → lições). O objetivo não é listar todo fix (o git já faz isso), mas registrar os bugs
> cuja **causa enganava** — aqueles em que a aparência levou a vários rounds na pista errada.
> Cada entrada termina em **lições generalizáveis**, para o próximo agente não repetir o erro de
> diagnóstico.
>
> Contexto técnico do traço: [`03_traco_rasterizacao.md`](03_traco_rasterizacao.md).
> Referência do Blender: [`02_referencia…`](02_referencia_algoritmos_blender_5.2.md).

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--a-mordida-a-borda-macia-de-um-segmento-apagava-o-núcleo-de-outro) | **A "mordida"** — bocados arrancados do traço nas quinas com hardness < 1 (8 rodadas) | `ph2d-flip-render` (fragment/geometria/depth) | ✅ **Resolvido** (smoke Enio 2026-07-12) | 2026-07-12 |
| [2](#bug-2--o-oráculo-que-ficava-verde-com-o-bug-na-tela) | O oráculo GPU ficava **verde com o bug na tela** (modelava a implementação) | Harness de teste (`gpu_render.rs`) | ✅ Resolvido (vira regra do módulo) | 2026-07-12 |
| [3](#bug-3--linha-fina-sumia--o-aa-subestimava-a-cobertura-em-10) | Linha fina **sumia**; AA subestimava a cobertura em 10× | `flip.wgsl` (máscara/AA) | ✅ Resolvido (latente desde o W1) | 2026-07-12 |
| [4](#bug-4--ponto-duplicado-rasgava-o-traço-nan-no-miter) | Ponto duplicado **rasgava o traço** (`normalize(0)` = NaN) | `flip.wgsl` (vertex/miter) | ✅ Resolvido (latente desde o W1) | 2026-07-12 |
| [5](#bug-5--o-grid-do-broadphase-perdia-vizinhos-pad-simétrico--empate-não-determinístico) | O broadphase perdia vizinhos (pad simétrico) e era **não-determinístico** (empate) | `neighbors.rs` | ✅ Resolvido antes de sair do forno (pego por teste) | 2026-07-12 |

---

## Bug #1 — A "mordida": a borda macia de um segmento apagava o NÚCLEO de outro

**Estado:** ✅ resolvido em 2026-07-12 (smoke do Enio aprovado). Custou **8 rodadas** e um estudo
exaustivo da referência. É o bug mais caro do módulo até hoje.

### Sintoma

Com **hardness < 1** (pincel macio — o caso comum), o traço saía com **bocados arrancados** nas
quinas: um pedaço reto/curvo "faltando" no lado interno de cada virada afiada. Zigzags ficavam
picotados. Com hardness = 1 (borda dura) **não aparecia nada** — o que jogou o diagnóstico na
pista errada por rodadas.

### A saga (o que cada rodada consertou e quebrou)

| Rodada | Abordagem | Resultado |
|---|---|---|
| 1-3 | cobertura por `v_perp` (coordenada por-quad) | distorcia nas junções: spikes de miter + double-blend |
| 4 | cobertura **analítica** (distância à linha-de-centro no fragment) | matou o double-blend; **junções redondas de graça** |
| 5 | `GreaterEqual` + geometria "stadium" (quads independentes) | **BEAD** ("mastigado") — premult-over compondo 2× no mesmo pixel |
| 5 | fita CONECTADA por miter | matou o bead; nasceu o **SPIKE** (bowtie: a fita dobrava na quina afiada) |
| 6 | `GREATER` estrito + stadium | matou o acúmulo; nasceu o **ESCAMADO** (corrente de ovais) |
| 7 | **tripé**: fita+`miter_break` · GREATER estrito · `discard a<0.001` | matou acúmulo/spike/bead/escamado — **e nasceu a MORDIDA** |
| 8 | **união global da polilinha** | ✅ fechado |

O escamado da rodada 6 era, na verdade, a **falta do `discard`** (um fragmento transparente
escrevia depth e furava o vizinho) — não culpa do `GREATER`. Descobrir isso exigiu ler o
fragment shader do GP linha a linha.

### Causa-raiz (o invariante que ninguém tinha escrito)

O traço é uma fita de quads (1 por segmento) e a forma sai no fragment, da distância do pixel à
linha-de-centro. Como o depth é **por-stroke** (todos os segmentos no mesmo valor) com teste
**GREATER estrito**, quando dois quads cobrem o mesmo pixel **o primeiro desenhado vence e o
segundo é descartado** — não misturado (é isso que evita o acúmulo).

> **O invariante implícito:** *onde dois quads cobrem o mesmo pixel, ambos DEVEM computar a mesma
> máscara* — porque o vencedor é arbitrário, e só é invisível se tanto faz quem vence.

O sistema quase o cumpre. Ele quebra em **dois lugares**, e ambos produzem a mordida:

**(a) A quina QUEBRADA** (virada > 120°, onde a fita não mitra e cada quad se estende `r` além do
ponto): os dois quads se sobrepõem no disco da junção. Cada fragment mede a distância ao **seu
próprio** segmento — então, na zona de extensão, o quad A calcula um campo **radial** centrado na
junção, enquanto aquele pixel pertence ao **núcleo** do segmento B (cobertura ≈ 1). A vence por
ser o primeiro, e pinta a sua queda macia por cima do miolo de B.

**(b) A auto-aproximação NÃO-ADJACENTE** (o pior caso, e o que a análise inicial subestimou):
todo traço que volta sobre si mesmo — zigzag apertado, laço, letra, hachura — tem segmentos
distantes na sequência mas próximos no espaço. O quad do segmento 0 cobre pixels do núcleo do
segmento 2, e ali a máscara de 0 vale **0.0046** (= 1/255, quase invisível!). Esse alpha ridículo
**sobrevive ao discard, escreve depth e bloqueia o segmento 2**, cuja cobertura é ~1.

> **Em uma frase: a borda quase-transparente de um segmento apagava o miolo opaco de outro.**

**Por que hardness = 1 escondia:** com borda dura a máscara é binária. Onde os dois quads valem
1, tanto faz quem vence. Onde o vencedor vale 0, o `discard` impede a escrita de depth e o
perdedor pinta em seguida. **No caso binário, first-wins + discard produzem a união exata.** Com
hardness < 1 nasce a faixa fatal: valores em `(0.001, 1)` sobrevivem ao discard, escrevem depth
e bloqueiam o vizinho.

### O Blender tem o MESMO bug (e convive com ele)

Confirmado por três vias independentes:
1. **O shader:** no corner type ROUND (o **default**), `gpencil_stroke_segment_mask` recebe os
   vizinhos `p0`/`p3` mas **os ignora** — retorna a cápsula do próprio segmento. As "cunhas" que
   usam p0/p3 só rodam nos tipos BEVEL/MITER.
2. **Issues abertas:** [#140075](https://projects.blender.org/blender/blender/issues/140075)
   ("Worked: Never"; o dev do módulo: *"current limitation of how strokes are generated and drawn
   with transparency"*), [#102927](https://projects.blender.org/blender/blender/issues/102927)
   (*"no clean corners with lower opacity"*, fechada como limitação conhecida),
   [#94252](https://projects.blender.org/blender/blender/issues/94252).
3. **A resposta do próprio Blender (2025):** os *Corner Types* (PR 143688) resolvem a junção
   **com p0/p3 no fragment** — ratificando a direção, sem fechar o caso ROUND macio.

O GP se esconde atrás de `hardness` default = **1.0** + SMAA. **No Flip o pincel macio é o caso
comum, então divergimos de propósito.**

### A solução — a cobertura é a UNIÃO GLOBAL da polilinha

**Ideia:** o fragment para de medir a distância ao *próprio segmento* e passa a medir a distância
à **polilinha**: `dn = min(dn_i)` sobre todas as cápsulas que alcançam o pixel. Como o perfil de
hardness é **monótono decrescente**, `min-distância ⇔ max-cobertura` — os quads sobrepostos passam
a computar o **mesmo** valor e o first-wins volta a ser invisível. **O invariante é restaurado.**

São **4 peças**, todas obrigatórias:

**1. Janela de sequência (`p0`/`p3`).** O vertex já buscava os vizinhos para o miter — agora os
exporta como varyings FLAT, e o fragment inclui as 2 cápsulas no `min`. Fecha o caso (a).
*Sentinela de borda:* sem prev/next o vizinho **coincide** com o extremo; como o varying é FLAT
(sem interpolação), a igualdade é exata e a cápsula degenerada é ignorada.

**2. Vizinhos GEOMÉTRICOS** (`neighbors.rs` — a peça que faltava, e que fecha o caso (b)).
Um **broadphase por grid na CPU**, dentro do `pack` (que já é cacheado por desenho), descobre
para cada segmento quais segmentos **não-adjacentes** podem alcançar os pixels do seu quad, e
emite essa lista curta para o shader. O fragment soma essas cápsulas ao `min`.

- **Critério conservador (sem falso-negativo):** um pixel do quad de `i` está no máximo a `2·r_i`
  do eixo de `i` (o esticão do miter é limitado a 2×), e `j` só o influencia se
  `dist(pixel, j) < r_j`. Pela desigualdade triangular basta testar
  **`dist(seg_i, seg_j) < 2·r_i + r_j`** — **assimétrico**: o raio do *dono do quad* entra
  dobrado.
- **Custo zero no caso comum:** um traço que não volta sobre si (linha, arco, contorno) tem a
  lista **vazia** e o laço do fragment nem executa.
- **Um único passe.** A alternativa que a spec previa (scratch + blend `MAX`, o
  *Stencil-then-Cover* contínuo) daria a mesma união, mas custa **2 render passes por traço**
  (~3 ms de CPU com 300 traços) — e o cache de frame não salva, porque o traço rasteriza em
  screen-space (zoom/pan invalidam). **Foi descartada por ser pior.**

**3. Uma única `capsule_dn`.** O raio efetivo é o interpolado pelo `t` **CLAMPADO** da cápsula —
a mesma função para o próprio segmento e para os vizinhos:

```wgsl
fn capsule_dn(frag: vec2f, a: vec2f, b: vec2f, ra: f32, rb: f32) -> f32 {
    let ab = b - a;
    let len_sq = dot(ab, ab);
    if (len_sq < 1e-6) { return 1e9; }          // cápsula degenerada = sentinela
    let t = clamp(dot(frag - a, ab) / len_sq, 0.0, 1.0);
    return length(frag - a - t * ab) / max(mix(ra, rb, t), 1e-4);
}
```

Se o próprio segmento usasse o `thickness` interpolado sobre o **quad** (que inclui as extensões
do `miter_break`), então **com largura por-ponto** (pressão de tablet — o caso normal!) os dois
quads que cobrem um pixel normalizariam por raios **diferentes** → o invariante quebra de novo e
**a mordida sobrevive em 2ª ordem**. Isso foi apontado pela verificação adversarial e **só um
teste com taper o pega** (todos os outros usam largura uniforme).

**4. Par clamp + fade sub-pixel** — ver [Bug #3](#bug-3--linha-fina-sumia--o-aa-subestimava-a-cobertura-em-10).

**O tripé segue intacto** (fita+`miter_break` · depth por-stroke + GREATER estrito · `discard`).

### Descoberta colateral: o `discard` deixou de ser load-bearing

Com a união global, a mutação "remover o `discard`" **não sangra mais**. A razão: o fragmento que
cobre o núcleo de outro segmento agora tem máscara **alta**, não ~0 — a classe "fragmento
transparente escreve depth e fura o vizinho" simplesmente **deixou de existir**. Ele permanece
por dois motivos honestos: protege a degradação do cap/budget, e evita escrever depth à toa.
**Não o remova** (é barato) — mas saiba que ele não é mais o que segura a correção.

### Degradações declaradas (determinísticas, e onde não doem)

- **`MAX_EXTRAS_PER_SEGMENT = 16`** — num rabisco denso, dezenas de segmentos cruzam o mesmo; os
  16 mais próximos entram. Além disso, aqueles pixels voltam ao first-wins do GP.
- **`PAIR_BUDGET`** — teto de trabalho por traço no broadphase. Só o **borrão sólido** (milhares
  de pontos rabiscados num palmo de tela) o atinge — e ali a mordida é invisível no meio da tinta.

### Perf (release, medida)

| Cenário | Custo do `pack` |
|---|---|
| Traço real longo (onda de 4000 pontos, não volta sobre si) | **1.7 ms** |
| Rabisco browniano patológico (4000 pontos num palmo) | 14 ms (limitado pelo budget; 27 ms sem ele) |
| Par-a-par ingênuo `O(n²)` (o que NÃO fizemos) | ~1 s |

`tests/pack_perf.rs` guarda a **ordem** (não é microbenchmark). Se o preview travar num rabisco
longo, o próximo passo é o **pack incremental** (o traço cresce e só a cauda muda — o
`active_smooth` congela o resto), **não** afrouxar o broadphase.

### Lições (as caras)

1. **Um artefato "cosmético" pode ser o pior bug do módulo.** A mordida parecia um detalhe de
   quina; era a *borda de um segmento apagando o miolo de outro* — uma quebra de invariante que
   afeta todo traço que volta sobre si.
2. **Quando um depth-test elege um vencedor arbitrário, escreva o invariante.** "Os candidatos
   sobrepostos têm de computar a mesma coisa" não estava em lugar nenhum — nem no Blender. Foi ao
   escrevê-lo que o fix ficou óbvio.
3. **Fidelidade à referência não é o objetivo — o resultado é.** Éramos *fiéis demais*:
   reproduzimos um artefato que o Blender tem e esconde com defaults. Antes de portar um
   comportamento, pergunte se ele é uma *decisão* ou uma *limitação*.
4. **A escalada mais cara nem sempre é necessária.** A "solução exata" que a spec previa (2 passes
   com blend MAX) daria o mesmo resultado custando ~3 ms/frame. A solução barata (pré-computar na
   CPU o que o fragment precisa saber) era melhor em tudo.
5. **A verificação adversarial paga o custo dela.** Ela previu o defeito da parametrização de raio
   (peça 3) — que nenhum teste de largura uniforme pegaria, e que teria feito a mordida
   "voltar de leve" com pressão de tablet, meses depois, sem explicação.

---

## Bug #2 — O oráculo que ficava VERDE com o bug na tela

**Estado:** ✅ resolvido — e virou **regra do módulo** (e memória do projeto).

### Sintoma

Na rodada 7 escrevi um teste de paridade CPU↔GPU pixel-a-pixel: 9 testes verdes, **2 mutações
provadas** (asserção-vermelha real), rodando em GPU real. Declarei o traço fechado. O smoke do
Enio reprovou **na hora** — a mordida estava lá, gritante.

### Causa-raiz

O oráculo replicava, na CPU, **o que o shader fazia**: a geometria dos quads, a máscara, e a
regra de depth *first-wins*. Ou seja, ele modelava a **IMPLEMENTAÇÃO**. A mordida **é** o
first-wins — então o oráculo tinha codificado o bug como verdade e o confirmava com entusiasmo.

> Um teste derivado do código só detecta **regressão**. Ele nunca detecta que o código está
> errado — porque "certo" para ele é "igual ao código".

### Solução

O expected passou a derivar da **APARÊNCIA — a definição do objeto**:

> Um traço macio **é** a união dos discos varridos ao longo da polilinha. A cobertura num pixel é
> o perfil de hardness aplicado à **menor** distância normalizada às cápsulas de **todos** os
> segmentos (mais o raio mínimo rasterizável e o fade sub-pixel — que também são aparência).
> **Nada nele sabe de quads, depth, ordem de desenho ou discard.**

Com isso, o oráculo ficou **VERMELHO no código antigo** (4 testes, desvio ~250/255: a GPU pintava
`2` onde a união pede `254`) — um alvo irrefutável, *antes* de tocar no shader. E ele apontou o
pixel exato, o que levou direto à segunda classe do bug (a auto-aproximação).

### A sequência obrigatória (siga-a em qualquer mudança futura no traço)

1. **Escreva/estenda o oráculo primeiro** e prove que ele fica **VERMELHO** no código atual.
2. Só então mexa no shader, até o verde.
3. **Prove as mutações** — o oráculo só vale se elas sangram. (Hoje: 5 mutações, todas sangram.)

### Lição

**Oráculo modela a APARÊNCIA, não a implementação.** Se a fórmula do `expected` saiu de reler o
seu shader, ela não pode falhar por design errado — só por typo. Derive-a da definição do objeto
(a união dos discos; o blend canônico; a transferência de cor de referência), rode-a vermelha
contra o código atual, e só então implemente.
→ memória `feedback_oracle_must_model_appearance_not_implementation`.

---

## Bug #3 — Linha fina sumia; o AA subestimava a cobertura em 10×

**Estado:** ✅ resolvido (2026-07-12). **Latente desde o W1.**

### Sintoma

Ao portar o *fade sub-pixel* do GP (`mask *= smoothstep(0,1, thickness_px)`), o teste mostrou que
uma linha de **0.35 px pintava ZERO** — sumia por completo. E, investigando, uma linha de 1 px
saía ~10× mais fraca do que deveria.

### Causa-raiz (dois bugs, um em cima do outro)

**(a) O fade do GP sozinho não faz nada.** Uma fita de 0.35 px não cobre o centro de **nenhum**
pixel → o rasterizador não emite fragmento → não há o que desbotar. O GP tem um **par**: um
**clamp de largura mínima** (~1.3 px, usado na geometria E na máscara) + o fade pela espessura
**não-clampada**. Só juntos a linha fina "não afina mais — desbota", preservando energia e
matando o pisca/serrilhado ao mover e ao dar zoom.

**(b) A fórmula de AA estava errada desde o W1.** Usávamos
`edge = 1 - smoothstep(1-aa, 1, dn)`, que só aproxima a cobertura quando `aa << 1`. Com traço
fino, `aa = fwidth(dn) > 1` e a fórmula **subestima brutalmente**. A forma correta é a **fração
do pixel coberta**:

```wgsl
let edge = clamp(0.5 + (1.0 - dn) / aa, 0.0, 1.0);   // em dn=1 dá 0.5 = meio pixel coberto
```

### Solução

`MIN_WIDTH_PX = 1.3` (raios clampados no vertex, usados na geometria e nas cápsulas) +
`thickness` **cru** no varying (só para o fade) + a fórmula de cobertura acima.
Teste: `a_subpixel_thin_stroke_fades_instead_of_flickering` (a fina pinta, mas mais fraca que a
de 1 px, que é mais fraca que a grossa — e a grossa fica **intacta**).

### Lição

**Um mecanismo portado pela metade pode não fazer nada — ou piorar.** O fade e o clamp são um
par; portar só o que está no `frag.glsl` (e não o clamp que vive no `vertex`) produziu um no-op
silencioso. Quando um comportamento da referência "não funciona", suspeite de que ele tem uma
segunda metade em outro arquivo.

---

## Bug #4 — Ponto duplicado rasgava o traço (NaN no miter)

**Estado:** ✅ resolvido (2026-07-12). **Latente desde o W1** — nunca observado no smoke, mas
armado.

### Sintoma (potencial)

Um ponto **duplicado** no meio do traço (o tablet repete uma amostra; o smooth/simplify funde
dois) faria o vértice `normalize(sa - sp)` com vetor nulo → **NaN** → o quad inteiro degenerava
→ **rasgo visível no traço**.

### Solução

`safe_dir()` no vertex: normalização que devolve `false` quando o vetor é nulo, e o chamador trata
o vizinho degenerado como **"sem vizinho"** (perpendicular reta, sem miter). No fragment, a
`capsule_dn` já ignorava a cápsula de comprimento zero (`len_sq < 1e-6 → +∞`). Teste:
`a_duplicated_point_does_not_tear_the_stroke` (a aparência tem de ser a da polilinha **sem** o
ponto repetido).

### Lição

**Dado válido do documento nunca pode produzir NaN no shader.** Em vez de exigir uma invariante do
ingest ("nunca duplique pontos"), blinde o consumidor — a invariante seria quebrada mais cedo ou
mais tarde por um caminho novo (import, undo, simplify, tween).

---

## Bug #5 — O grid do broadphase perdia vizinhos (pad simétrico) + empate não-determinístico

**Estado:** ✅ resolvido **antes de sair do forno** — os dois foram pegos por um teste escrito
junto com o código.

### Contexto

O broadphase de vizinhos geométricos (Bug #1, peça 2) trocou um `O(n²)` par-a-par por um grid
espacial. Escrevi, junto, um teste que compara o grid com o par-a-par ingênuo num rabisco denso
de 180 segmentos. Ele pegou **dois bugs reais**:

### (a) O pad simétrico perdia vizinhos mais GROSSOS

O critério é **assimétrico** (`dist < 2·r_i + r_j` — o raio do dono do quad entra dobrado). Eu
tinha usado o mesmo pad (`3·r`) na inserção e na consulta do grid. Consequência: quando o vizinho
`j` é mais **grosso** que o dono `i`, o alcance necessário (`2·r_i + r_j`) excede o pad e o par
**escapa** — a mordida voltaria, em silêncio, só naqueles pixels.

**Fix:** pad de **inserção = `r_j`** (o alcance da cápsula dele) e pad de **consulta = `2·r_i`**
(o alcance dos pixels do quad). Duas regiões que se tocam compartilham ao menos uma célula, então
nenhum par escapa.

### (b) O empate quebrava o determinismo

Num rabisco denso, **146 segmentos cruzavam o mesmo** — todos com distância **0.0**. Com empate
total, o `sort` estável fazia o corte (`top-16`) depender da **ordem de descoberta** — que é
diferente entre o grid e o par-a-par. O mesmo desenho geraria **buffers diferentes**, quebrando o
determinismo (replay-hash é contrato do projeto).

**Fix:** ordenar por **`(distância, índice)`** — o desempate por índice torna o conjunto único,
independente da ordem de descoberta.

### Lição

**Escreva o oráculo do algoritmo junto com a otimização.** Um broadphase "esperto" que perde 1 par
em 10 mil não falha em lugar nenhum — ele só devolve um pixel errado, meses depois, num desenho
qualquer. O teste de equivalência com a versão burra (`O(n²)`) é barato e é a única coisa que
separa "rápido" de "rápido e correto".

**E: empate + corte = não-determinismo.** Sempre que você ordena e trunca, garanta uma chave de
desempate total.

---

## Bug #6 — O fantasma da camada de cima ficava atrás da camada de baixo

**Sintoma (smoke do Enio, W3):** "Ghost da camada de cima está coberto pela camada de baixo." No
demo, o retângulo amarelo opaco do BG engolia os fantasmas do FG.

**A causa não era de COR — era de ORDEM.** O 1º corte desenhava TODOS os fantasmas num passe
direto no `game_rt`, **antes** de compor as camadas. Aí o composite blitava a pilha inteira por
cima: qualquer camada de fundo opaca apagava o fantasma de qualquer camada acima dela. O bug é
invisível num documento de uma camada só — e o demo tinha duas.

**A ideia errada por trás:** tratar o fantasma como um *fundo* ("é uma coisa esmaecida, vai lá
atrás"). **O fantasma não é um fundo — ele pertence à sua camada.** O z dele é o dela.

**Solução.** Cada ghost virou uma **fatia do compositor**, inserida na op-list logo ABAIXO da sua
própria camada (portanto acima de todas as de baixo, e abaixo da arte do quadro atual), com blend
**Normal** e opacity **1.0** — o fade e a opacidade da camada já estão no alpha do tint. Herdar o
blend da camada seria o erro seguinte: um `Multiply` no FG tingiria o fantasma com a arte do BG,
e ele deixaria de ser uma silhueta chapada.

**O gate.** A op-list compõe de baixo para cima, então **a ordem da lista É a ordem de z**:
`a_layers_ghost_sits_above_the_layers_below_it` exige `BG < ghost(FG) < FG`. É um teste de
APARÊNCIA disfarçado de teste estrutural — e a mutação sangra (coletar os fantasmas fora do laço
de camadas devolve `ghost, art, art`).

**Lição generalizável.** *Overlay é uma decisão de z, não de cor.* Quando um elemento novo
"pertence" a uma camada existente, ele entra NA PILHA, na posição dela — nunca num passe separado
por baixo (ou por cima) de tudo. Ver também
[[feedback_overlay_cut_at_boundary_check_draw_order]]: o mesmo reflexo (listar os writers, em
ordem) resolve as duas famílias.

---

## Bug #7 — Os ciclos (Loop/Ping-Pong) não faziam absolutamente NADA

**Sintoma (smoke do Enio, W3):** "Parece que pingpong e loop extrapolam o último quadro e não
funcionam corretamente." O último desenho segurava para sempre, em qualquer modo de ciclo.

**Causa-raiz:** o modelo tinha os ciclos implementados (`cycle.rs`) e **testados** (`map_frame`
com Loop/PingPong/Hold/None, 6 testes verdes) — e o render **nunca os chamava**. O
`collect_layers` do `flip_pass` amostrava `FlipLayer::drawing_at` (o caminho CRU) em vez de
`drawing_at_cycled`. Uma linha.

É o caso de livro de [[feedback_tool_unit_green_integration_dead]]: **unit-verde ≠ funciona no
produto**. Todo teste do ciclo passava porque todos testavam o MODELO. Nenhum testava o caminho
que o app realmente percorre.

**Solução.** O render (e os fantasmas, e a navegação por desenho, e a célula destacada na tira)
passam pelo quadro-FONTE. Gate: `the_render_samples_through_the_cycle` — no `collect_layers` REAL,
o quadro 20 de um Loop de 16 tem de resolver para o desenho do quadro 4.

**O achado colateral (que um teste pegou na hora):** ao rotear a autoria pelo mesmo mapa, eu
quase matei o autokey. **Há TRÊS relógios, não dois:**

| Transform | Responde | Quem usa |
|---|---|---|
| `drawing_at` (cru) | "o que a chave em/antes deste quadro diz" | a mecânica interna |
| `source_frame` | "**qual quadro do vão está na tela**" | render, fantasmas, célula destacada, ops de chave |
| `authoring_frame` | "**em qual quadro este gesto escreve**" | caneta, borracha |

`authoring_frame` mapeia pelo ciclo **só onde o tempo REPETE** (Loop/Ping-Pong: editar a 2ª volta
edita o vão, e a edição aparece em todas as voltas — é o que um ciclo *significa*). Sob `Hold`/
`None` — os **defaults** — o tempo depois do vão é tempo **NOVO**: desenhar ali cria a chave ali,
que é como uma animação cresce. Mapear de volta teria feito a caneta editar o desenho anterior em
vez de criar o próximo.

**Lição generalizável.** [[feedback_derived_coordinate_seed_must_match_sample]] diz "todo caminho
de autoria usa a MESMA transform do de leitura" — e é verdade, **mas a transform de leitura pode
não ser única**. Antes de unificar, pergunte *o que cada quadro/coordenada SIGNIFICA*: aqui,
"repetido" e "novo" são semanticamente diferentes, e colapsá-los quebra uma feature ou a outra.
Duas transforms com nomes honestos > uma transform "unificada" que mente para metade dos callers.

**E um terceiro, de UX:** ligar um ciclo sobre uma tira cuja última chave tinha hold implícito
(infinito) fazia o vão fechar em `última + 1` — ela expunha 1 quadro e o ciclo *piscava* no fim.
Agora escolher Loop/Ping-Pong **materializa a exposição da última célula** (igual à da anterior),
visível na tira e editável no Hold. *Um default derivado de um valor "infinito" precisa virar um
número real no momento em que alguém depende dele.*

---

## #8 — "Não existe botão fill": o gate que faltava no projeto INTEIRO

**Sintoma (Enio, 2026-07-12):** *"não existe botão fill"*, depois de a W4 ser declarada fechada
e verde.

**O botão existe.** O `mode_row` pinta os quatro modos; o `atime` do binário provou que o
executável da W4 **nunca foi rodado** (o Enio estava olhando o app da W3). Mas o relato foi o
melhor da linha, porque a auditoria que ele provocou achou coisa muito pior — e uma pergunta que
nenhum teste do projeto sabia fazer.

**A pergunta que ninguém fazia: o widget é PINTADO?**

Existiam dois tipos de prova, e nenhuma cobria isso:

| prova | o que ela responde |
|---|---|
| `tests/seam.rs` (blindagem Fase 1.2) | "o clique CHEGA na tool?" — roda `populate → apply_event → bus → tool` |
| `architecture_panel_wiring_parity` | lê o **texto-fonte**: o id é hit-indexado e registrado? |

**Nenhuma das duas roda o `paint`.** Então um widget pode estar registrado, wirado, unit-testado e
contract-limpo enquanto a chamada que o desenha mora atrás de um `return` — ou nunca foi escrita.
O usuário relata *"esse botão não existe"*, e **todos os gates continuam verdes**.

**O gate:** `MockPanelHost::paint::<P>()` roda o `Panel::paint` REAL, headless (cena Vello sem GPU
+ `TextSystem::without_system_fonts()`), e devolve **o que ficou clicável** — os `(id, rect)` que a
pintura registrou no hit index. *O que o usuário pode clicar é o que a PINTURA registrou; é isso
que um teste tem de ler.*

**E ele pegou um bug de verdade, no primeiro uso** — não no Fill, mas na **tira da W3** (#9).

---

## #9 — A barra que escondia metade de si mesma (W3)

`Bar::fits()` na tira: *"a barra nunca transborda: um item que não cabe simplesmente não é pintado
nem registrado"*. Parecia prudente. Era o pior dos dois mundos.

Num viewport de **1280px, NOVE dos dezoito controles sumiam**: as ops de chave (+, duplicar,
apagar), o Hold, o ciclo e o **tween inteiro**. Sem scroll, sem overflow, sem qualquer sinal de que
existiam. E como o teste era por-item (`x + w <= right`), uma caixa **estreita entrava depois** que
um botão largo fora descartado — a barra saía com **buracos, fora de ordem**.

O Enio conseguiu clicar o tween no smoke da W3 só porque o monitor dele é largo. Num laptop, a
metade de autoria da tira simplesmente não existiria.

**Fix:** a barra **QUEBRA em linhas** e a tira **cresce** para caber. Nada é escondido. Um item
mais largo que a linha inteira transborda — melhor um controle cortado (que o usuário vê e alcança
redimensionando) do que um controle ausente (que ele conclui que não existe).

> **Lição.** *Esconder um controle é pior que deixá-lo transbordar.* Um layout que "nunca quebra"
> porque descarta o que não cabe não é robusto — é **mentiroso**: ele reporta sucesso enquanto
> entrega um app mutilado, e o único jeito de descobrir é alguém abrir numa tela menor. Se um
> layout precisa ceder, que ceda em **espaço** (mais linhas, scroll), nunca em **existência**.

---

## #10 — A W4 estava morta no produto, e os testes diziam que não

A auditoria de 3 lentes (solver · costura · modelo/render) achou **12 bugs**, três deles matando o
balde no uso mais banal. O padrão que os une é mais importante que qualquer um deles.

**(a) O teste que escolhe o único valor que esconde o bug.** A espessura do traço é em **px de
tela**; os pontos, em **unidades de documento**. A conversão nunca foi aplicada — no zoom padrão um
traço de 6px virava uma linha de **3 unidades de mundo (~324px!)** atravessando um desenho de 2,8
unidades, e o clique caía sempre *dentro* do traço: **"Fill: clicked on a line", sempre**. Os cinco
unit tests do `flip_fill` passavam `px_to_world = 1.0` — o **único** valor em que px de tela ==
unidade de documento. Eles não testavam o produto; testavam um mundo onde o bug não existe.
> *Um teste que escolhe as constantes mais convenientes não é um teste: é uma tautologia. Use os
> números do PRODUTO (a câmera real, a janela real) — foi o `the_bucket_fills_at_the_real_camera_scale`
> que virou vermelho.*

**(b) O teto que corta em vez de ceder.** `MAX_SIDE` clampava as **dimensões** da grade e mantinha
o `scale` — a grade cobria só um pedaço do bbox e a arte além dele **nem era rasterizada**. Bastava
dar **zoom** para o balde recusar uma forma perfeitamente fechada com "Fill leaked". O doc prometia
"a resolução cai, o fill fica mais grosseiro, não quebra"; o código não fazia isso.
> *Quando bater num teto, decida CONSCIENTEMENTE o que cede. Cortar a cobertura é sempre a resposta
> errada — e um comentário que descreve o comportamento certo não o implementa.*

**(c) Os pontos de estrangulamento da cópia.** Três helpers definem o que sobrevive a uma operação:
`FlipStroke::clone_attrs`, `flip_erase::new_like` e o `cleanup_soft`. A W4 acrescentou dois campos
ao modelo (`holes`, `hide_stroke`) e atualizou **um**. Consequências: o tween deixava o furo do "O"
para trás (uma mancha solta viajava pela tela); a borracha picava o fill e o Unpaint não reconhecia
mais os pedaços; e o `cleanup_soft`, que coleta lixo por **opacidade de ponto**, apagava **TODOS os
fechamentos de gap** do desenho a cada toque de borracha macia — em qualquer lugar do canvas —
porque um fechamento nasce com opacidade 0.
> *Ao acrescentar um campo ao modelo, ache os choke points de CÓPIA e audite cada um. "Funciona de
> graça, sem código novo" só vale se todo ponto de cópia acompanhar o crescimento — e nenhum gate
> força isso. O `cleanup_soft` é o mais traiçoeiro: um GC que decide por um campo (opacidade) que,
> para a classe nova, não significa visibilidade.*

**(d) A ambiguidade resolvida sem memória.** No marching squares, a quina onde duas células cheias
se tocam **só pela diagonal** tem DUAS trilhas passando por ela. Resolvê-la com uma escolha fixa
jogava quem chegava por um lado na trilha do outro: o anel nunca fechava, o `guard` estourava, e o
monstro auto-sobreposto resultante — com |área| inflada por dar voltas — podia **vencer a ordenação
por área e virar o contorno externo**. A saída certa depende da **direção de chegada** (vira à
esquerda: mantém a mesma célula à esquerda).
> *Numa bifurcação ambígua, a informação que desempata quase nunca está no estado LOCAL — está em
> como você chegou ali. E um anel que não fechou é LIXO, não um contorno mais pobre: descarte, nunca
> exporte.*

**(e) O buffer de tamanho zero.** Um desenho **só de preenchimento** (apague o line-art de uma
região pintada) tem zero pontos → storage buffer de tamanho 0 → a wgpu recusa o bind group inteiro
e o app **cai**. O `seg_extras` já driblava isso com um dummy; os outros quatro buffers, não.
> *A invariante ("um storage buffer nunca é vazio") pertence a quem CRIA o buffer, não a cada
> chamador que precisa lembrar dela.*

---

## #11 — "Fill impreciso": o teto na unidade errada, e o zoom que estragava o balde

**Sintoma (Enio, 2026-07-12, com screenshot):** o preenchimento **transborda a linha** por ~12px e
o contorno vira um **polígono grosseiro** de ~15 lados.

O solver, medido isoladamente com os números da câmera default, acertava a **1 px** e devolvia 42
vértices. Não havia bug nele. O bug estava numa linha do orquestrador:

```rust
let scale = params.precision.clamp(0.5, 64.0);   // ← px de buffer por unidade de DOCUMENTO
```

**Um teto de resolução em "px por unidade de documento" é um teto na unidade errada.** O desenho
vive em unidades de **mundo**, e aproximar a câmera faz a mesma forma ocupar *menos* unidades. Com
zoom, o teto de 64 cortava a resolução em pedaços — **1 px de buffer chegou a valer 5 px de tela**.
E como o `grow` e a tolerância do RDP vivem em px de **buffer**, os dois incharam junto: a cor
transbordava a linha e o contorno virava um polígono.

Medido (mesmo círculo, mesmo clique, só o zoom muda):

| `height_world` | vértices | transborda |
|---|---|---|
| 10 (default) | 42 | +1,3 px |
| 5 | 32 | +4,0 px |
| 2 | **17** | **+12,3 px** |
| 1 | 14 | +25,8 px |

**Fix:** não há teto. Quem limita a memória é o `MAX_SIDE` do `Grid::new` — e ele cede
**resolução**, não cobertura (#10b). Depois disso, o resultado é **invariante ao zoom**: 40
vértices e precisão sub-pixel em qualquer aproximação. Gate: `the_fill_is_invariant_under_camera_zoom`.

**Dois achados que vieram junto:**

- **O `grow` era em px de BUFFER, e o usuário pensa em px de TELA.** Subir a Precision *encolhia* o
  Grow em silêncio — dois controles nominalmente independentes que secretamente se multiplicavam. O
  shell agora converte (`grow_buffer = grow_px × precision`).
- **O default `grow = +2` era o próprio halo.** A fronteira é rasterizada a um **quarto** da
  espessura (`radius_scale = 0.5` sobre a meia-espessura), então a cor **já nasce por baixo da
  linha** — não há halo para matar. Medido: com `grow = 0`, a borda do fill fica **1,8 px DENTRO**
  da borda externa de uma linha de 6 px; com `+2`, ela sai **1,0 px para FORA**. O default que
  existia para evitar o defeito era a causa dele. Default agora é **0**.

> **Lição — um clamp carrega uma unidade, e a unidade tem de ser a do usuário.** `clamp(0.5, 64.0)`
> não diz em quê. O número que importava era "px de buffer por px de TELA" (uma razão estável), e o
> que estava sendo capeado era "px de buffer por unidade de mundo" (uma razão que o zoom move). *Ao
> escrever um limite, escreva a unidade ao lado — e pergunte se ela é estável sob as transformações
> que o usuário controla (zoom, escala do objeto, DPI). Se não for, o limite vai morder em algum
> lugar imprevisível.*

> **E a lição de método:** o harness com a câmera default dizia "1 px de erro, 42 vértices" — verde.
> O produto dizia "12 px, 17 vértices". A diferença era **um parâmetro que o teste nunca varreu**.
> Varra o eixo que o usuário controla ([[feedback_test_with_product_numbers_not_convenient_ones]]):
> não basta usar os números do produto, é preciso usar a **faixa** deles.
