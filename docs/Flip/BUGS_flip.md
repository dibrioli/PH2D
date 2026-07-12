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
