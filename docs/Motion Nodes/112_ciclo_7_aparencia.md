# 112 — CICLO 7: APARÊNCIA, a cor e o rasto

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, e o **tutorial é o smoke**.
> **Premissa do tutorial:** *«A cor e o rasto»*.
>
> ⚠️ Este doc é o do CICLO. O que ele mede vive nas sondas de
> [`motion_aparencia_probe.rs`](../../crates/ph2d-app-motion/src/motion_aparencia_probe.rs); o
> mecanismo de cada wave vai para o handoff dela.

---

## §1 — O grupo, DERIVADO da paleta

O grupo é **a categoria `Fx` da paleta** (o cabeçalho magenta que o artista vê), pedida ao
registry e sem as fixturas — e **não** um prefixo: a família mistura `fx.*` e `motion.*`, e uma
lista escrita à mão aqui envelhecia em silêncio no dia em que um nó `Fx` nascesse.

| família | nós |
|---|---|
| `fx.*` | 3 — `drop_shadow` · `glow` · `rgb_split` |
| `motion.*` | 7 — `color_array` · `color_ramp` · `slit_scan` · `strobe` · `sub_uv` · `tint` · `trail` |
| **total** | **10** |

⚠️ Coincide com a linha do doc 103 §5, e isso é uma **verificação**, não a fonte. Gate
`the_fx_group_is_derived_and_not_empty` (piso de `10` e as **duas** metades da família — um filtro
que só apanhasse os `fx.*` leria `3`).

---

## §2 — O RETRATO (sonda `audit_the_fx_group`, 2026-09-16)

```text
  nó                        | params | no cartão | device | portas | efeito
  --------------------------|--------|-----------|--------|--------|--------
  fx.drop_shadow            |      8 |         5 |  NAO   | 1->1   | Pure
  fx.glow                   |     15 |        13 |  NAO   | 1->1   | Pure
  fx.rgb_split              |      8 |         4 |  NAO   | 1->1   | Pure
  motion.color_array        |      0 |         1 |  sim   | 2->1   | Pure
  motion.color_ramp         |      0 |         1 |  sim   | 2->1   | Pure
  motion.slit_scan          |      1 |         1 |  NAO   | 2->1   | Pure
  motion.strobe             |     10 |         8 |  NAO   | 3->1   | Pure
  motion.sub_uv             |      5 |         6 |  sim   | 2->1   | Temporal
  motion.tint               |     10 |         3 |  sim   | 1->1   | Pure
  motion.trail              |     11 |        11 |  NAO   | 2->1   | Pure
```

⚠️ Esta tabela é a fotografia **antes** da W1a e fica assim de propósito — reescrevê-la apagaria o
retrato de que o §3 é a resposta. *Corra a sonda antes de citar a coluna.*

**O que ele já diz, sem uma linha de código:**

1. ⛔ **Seis de dez fora do dispositivo** (`device = NAO`).
2. ✅ **O vocabulário está LIMPO** — as três perguntas da sonda `the_fx_vocabulary_the_artist_reads`
   (um rótulo com várias chaves · uma chave com vários rótulos · palavras de enum que se leem como
   variantes) devolvem **zero** linhas.
3. ⚠️ **As diferenças `params`/`no cartão` são GATES DE MODO, não controlos perdidos** — o `tint`
   mostra `3` de `10` porque a segunda cor só existe em `Gradient`; o `rgb_split` mostra `4` de `8`
   porque o centro, a força e o início só existem em `Aberration`; os `r/g/b/a` de toda cor são UMA
   linha (`Color`). ⏳ A confirmar pelo censo do alcance (W2), não por esta leitura.
4. ⚠️ **O `motion.slit_scan` tem UM botão** (`Lag`). Um slit-scan tem muito mais a dizer (direcção,
   o campo que decide o atraso de cada elemento, a interpolação) — é o candidato óbvio da W3.
5. ⚠️ **Os `motion.color_array`/`color_ramp` declaram `0` params e mostram `1`**: a linha é o editor
   rico (paleta/gradiente), que vive num param de TEXTO. Não é defeito.

---

## §3 — ⛔⛔⛔ O ACHADO QUE DECIDE O CICLO: a aparência é o ÚLTIMO nó, e o último nó decide a rota

A lei 1 do protocolo manda que todo nó do grupo diga onde corre. ⚠️ **Um `NAO` no retrato não é um
preço** (a lição do ciclo 6, doc 110 §11.2): ele só custa se o nó estiver no caminho do OBJECTO, e o
preço é quantos elementos sobem na costura. ⇒ sonda `probe_does_an_fx_chain_stay_on_the_device`:
cada nó no meio de uma cadeia que **já** está no dispositivo.

```text
  grid 320² -> scale -> X -> output  | onde corre  | stages | costura            | elementos que SOBEM
  -----------------------------------|-------------|--------|--------------------|--------------------
  (sem X — o controlo)               | dispositivo |      3 | —                  | — (nada sobe)
  fx.drop_shadow                     | ⛔ CPU       |      1 | fx.drop_shadow:0   | 204 800
  fx.glow                            | ⛔ CPU       |      1 | fx.glow:0          | 102 400
  fx.rgb_split                       | ⛔ CPU       |      1 | fx.rgb_split:0     | 102 400
  motion.color_array                 | dispositivo |      4 | —                  | —
  motion.color_ramp                  | dispositivo |      4 | —                  | —
  motion.slit_scan                   | ⛔ CPU       |      1 | motion.slit_scan:0 | 102 400
  motion.strobe                      | ⛔ CPU       |      1 | motion.strobe:0    | 102 400
  motion.sub_uv                      | dispositivo |      4 | —                  | —
  motion.tint                        | dispositivo |      4 | —                  | —
  motion.trail                       | ⛔ CPU       |      1 | motion.trail:0     | 102 400
  motion.strobe + pulse.beat         | ⛔ CPU       |      1 | motion.strobe:0    | 102 400
```

⭐⭐⭐ **A costura não cai no nó: cai na CADEIA.** O `grid → scale` que estava no dispositivo passa a
cozinhar na CPU para alimentar o nó, e o stream inteiro sobe a cada quadro. E um nó de aparência é,
por natureza, **o último de um grafo** — a cor, o brilho e a sombra aplicam-se depois de tudo o
resto. ⇒ ***todo grafo que usa brilho, sombra, separação RGB, rasto, estroboscópio ou slit-scan
corre inteiro na CPU***, e isso é o `50,9×` do [doc 98](98_auditoria_de_performance_2026-09-01.md).

⛔⛔ **E o pior dos seis era o mais barato de curar: o `fx.glow` é um PASSA-TUDO.** O `eval` dele é
`input.clone()` — os parâmetros são lidos pelo RENDERIZADOR (`from_graph`, no `present_fx`), nunca
pelo cozimento. Ele derrubava a cadeia inteira e subia `102 400` elementos **para lhes não mudar um
byte**.

⚠️ **E há um segundo achado dentro da tabela:** o `fx.rgb_split` subiu `102 400` e não `307 200`,
apesar de triplicar as linhas. A razão é o `MAX_INSTANCES = 262 144` — um tecto **medido no caminho
de CPU** (`~10–15 ns` por linha emitida, `~3 ms` no tecto), que **desliga o efeito em silêncio**
acima dele: a `102 400` objectos a separação RGB **não faz nada**. Enquanto o nó vive na CPU o
tecto está certo; ⚠️ *no dia em que ele for para o dispositivo, o recurso muda e o número tem de ser
re-medido* (`CLAUDE.md` §0.0: **nunca deixe o fallback definir o produto**).

---

## §4 — ✅ W1a FEITA: o brilho deixa de levar o grafo para a CPU

Uma linha: `reg.register_gpu_kernel(MANIFEST.id, GpuKernel::PASSTHROUGH)` — o molde que o
`pulse.signal` já usava (*«um nó que derrubasse a cadeia inteira para a CPU seria a pior espécie de
sonda: a que muda o programa que mede»*). Perante um kernel sem corpo e sem bindings o sequenciador
**não emite passe nenhum** e a corrente atravessa-o, que é literalmente a lei da CPU.

```text
  grid 320² -> scale -> fx.glow -> output | antes         | depois
  ----------------------------------------|---------------|------------------------
  onde corre                              | ⛔ CPU          | dispositivo
  estágios no dispositivo                 | 1             | 4 (o passa-tudo não emite passe)
  elementos que sobem por quadro          | 102 400       | 0
```

⚠️ **O efeito continua a ser desenhado:** o passe do brilho lê o grafo
(`ph2d_node_fx_glow::from_graph(&motion.doc.graph)` no `present_fx` e no `fase_vector_fx_recook`),
e nenhum dos dois pergunta por onde o cozimento passou.

⭐ **A catraca** `the_fx_group_route_only_improves` — um nó do grupo no caminho do objecto não pode
levar a cadeia para a CPU sem estar **nomeado** na lista `NA_CPU`, com a razão; e as **duas
metades**: um nó da lista que passe a ficar no dispositivo reprova até a linha dele ser apagada
(*uma catraca sem censo de obsolescência vira licença*). Mutação (tirar o `PASSTHROUGH`): **RED** —
*«`fx.glow` leva a cadeia para a CPU»*.

---

## §4-bis — ✅ W1b FEITA: os dois que MULTIPLICAM as linhas vão para o dispositivo

`fx.rgb_split` (`×3`) e `fx.drop_shadow` (`×2`, `×17` com a maciez) ganharam kernel — os dois no
molde do `motion.kaleidoscope` (`StreamOp::SourceRows`: a lei de contagem dimensiona o passe, o
corpo escreve `cp_rows` e LÊ a fonte na linha `i mod n`, o sequenciador reúne as outras colunas; a
cópia é `i / n`, e é essa ordem em BLOCOS que põe as franjas e as sombras atrás).
[`fx.rgb_split/kernel.rs`](../../crates/ph2d-node-fx-rgb-split/src/kernel.rs) ·
[`fx.drop_shadow/kernel.rs`](../../crates/ph2d-node-fx-drop-shadow/src/kernel.rs).

```text
  grid 320² -> scale -> X -> output | antes          | depois
  ----------------------------------|----------------|------------------------
  fx.rgb_split                      | ⛔ CPU, 102 400 | dispositivo, 4 estágios, 0 a subir
  fx.drop_shadow                    | ⛔ CPU, 204 800 | dispositivo, 4 estágios, 0 a subir
```

⇒ **7 dos 10 nós do grupo ficam no dispositivo** (eram 4). Sobram os três com ESTADO (W1c).

⚠️ **Quatro coisas que uma leitura rápida do diff entende ao contrário:**

1. **O caso APAGADO não é um `PASSTHROUGH`.** Com a opacidade (ou o alfa da sombra) a `0` a CPU
   devolve a entrada — mas um nó `SourceRows` arranca de uma base VAZIA (o sequenciador troca-a
   antes de olhar para o kernel), e uma variante sem corpo emitiria **zero** elementos. A lei de
   contagem devolve `n` e o corpo copia.
2. **A sombra tem DUAS variantes, escolhidas pelo `shadow_blend`.** No `Sink` a CPU não toca na
   coluna `blend`; a variante `SINK` não a liga e o sequenciador reúne a de montante. Escrever `0`
   daria a mesma imagem (a descida lê `0` como *o modo do sink*) pagando uma coluna por quadro.
3. **A tag do modo é `floor(v + ½)`, nunca o `round` do WGSL** — o do WGSL arredonda o meio para o
   PAR (`4,5 → 4`) e o `f32::round` da CPU para longe do zero (`→ 5`). Há um caso a `4,5` no gate e
   a mutação que troca um pelo outro **morre**.
4. **A trigonometria da sombra é a folha parabólica PORTADA** (`ds_sin_cycles` = `trig.rs`), e os
   literais que atravessam a fronteira (o ângulo de ouro, os 16 taps, o topo dos modos) têm gate
   contra as consts da CPU — e contra estarem DE FACTO no texto que o dispositivo compila.

⭐ **Os gates** (`ph2d-gpu-cook/tests/it/gpu_cpu_parity_fx.rs`, adaptador real): os dois modos da
separação, o eixo deslocado, o raio limpo, o campo a modular o alfa, a sombra dura, colorida,
MACIA, com modo, e os casos que devolvem a entrada (apagado e acima do tecto, o da maciez à parte).
A comparação olha **posição, tamanho, COR e `flip_uv`** (onde o modo por linha desce). Pior medido:
`|Δpos| = 1,9e-6` (o raio limpo), `|Δtint| = 2,4e-7`. **Provas de mutação: 14 de 14 MORTAS** — seis
na separação (eixo ignorado · canais trocados · raio limpo ignorado · `falloff` ignorado · blocos
trocados · lei de contagem sem o apagado) e oito na sombra (modo nunca escolhido · alfa por tap ·
ângulo de ouro · disco ignorado · elementos com a tag · o `round` do WGSL · direcção em meia volta ·
lei de contagem sem maciez). E a catraca da rota reprova quando a sombra perde o kernel.

⚠️ **A coluna `blend` de MONTANTE não está no gate da sombra**, e não por esquecimento: quem a
escreve (`motion.trail`, `motion.strobe`) ainda é CPU (W1c), logo uma cadeia com ela nunca é
reclamada inteira. Os elementos lêem a identidade (`0`), que é o `_ => 0.0` da CPU.

### ⭐⭐ O `MAX_INSTANCES` foi RE-MEDIDO no dispositivo (o achado do §3)

Sonda `fx_row_ceiling_probe` (`--release`, três corridas a `load 5,6–8,1`, o menor dos três;
`grid side² → oscillator → fx → output`, a fonte a mexer-se para o memo da CPU não responder):

```text
  nó             │ linhas     │ disp. ms │ CPU ms │ memória da descida (188 B × linhas)
  fx.rgb_split   │    196 608 │     0,48 │   4,2  │    35 MiB
  fx.rgb_split   │    786 432 │     1,52 │  17,2  │   141 MiB
  fx.rgb_split   │  3 145 728 │     5,66 │  71,0  │   564 MiB   ← o tecto novo
  fx.rgb_split   │  6 290 112 │    11,15 │ 146,2  │ 1 127 MiB
  fx.rgb_split   │ 12 582 912 │ RECUSADO — BindingTooLarge (2 256 MiB > 2 047 MiB do adaptador)
  fx.drop_shadow │    131 072 │     0,26 │   1,7  │    23 MiB
  fx.drop_shadow │    524 288 │     0,93 │   9,5  │    94 MiB
  fx.drop_shadow │  2 097 152 │     3,79 │  44,3  │   376 MiB
  fx.drop_shadow │  4 193 408 │     7,44 │  94,5  │   751 MiB
  fx.drop_shadow │  8 388 608 │    15,14 │ 221,6  │ 1 504 MiB
```

⇒ **~1,8 ns por linha no dispositivo contra ~22–27 ns na CPU.** O tecto passa de `262 144` a
**`3 145 728`** nos dois nós — o ponto em que a cadeia ocupa **cerca de um terço de um quadro**
(`5,66–5,89 ms`, `34–35 %`), o MESMO critério que decidiu o de CPU, agora no recurso certo. Os
limites duros do dispositivo ficam longe (a ligação do adaptador: `11,4 M` linhas nesta máquina,
e acima dela o cozimento RECUSA em voz alta; o despacho e o `ID_WRAP`: `16,7 M`).

| o que o artista via | antes | depois |
|---|---|---|
| separação RGB desligada em silêncio a partir de | `87 382` objectos | `1 048 577` |
| sombra dura desligada a partir de | `131 073` | `1 572 865` |
| sombra macia desligada a partir de | `15 421` | `185 043` |

⚠️ **A CPU computa a mesma resposta e paga o dela** (`~71 ms` no tecto) — é a referência e o
recurso de quem não tem adaptador, nunca quem decide o tecto (`CLAUDE.md` §0.0).

⚠️⚠️ **E isto PARTIU uma lei escrita:** o gate `the_three_instance_ceilings_agree` afirmava que os
tetos do `motion.trail`, dos dois `fx.*` e do `source.lsystem` são UM número (*«linhas emitidas no
caminho de CPU»*). Com os `fx.*` no dispositivo a premissa deixou de os descrever — e a afirmação
antiga seria agora a catraca que segura o produto no caminho lento. Ele passou a
`the_instance_ceilings_agree_per_resource`: **dois grupos, dois literais medidos**, com piso em
cada um. ⚠️ No dia em que o `motion.trail` ganhar kernel (W1c), o teto dele é para MEDIR de novo,
não para copiar — está escrito no doc-comment dele.

---

## §4-ter — ✅ W1c (duas de três): o estroboscópio e o slit-scan vão para o dispositivo

```text
  grid 320² -> scale -> X -> output | antes            | depois
  ----------------------------------|------------------|------------------------
  motion.strobe                     | ⛔ CPU, 102 400   | dispositivo, 4 estágios
  motion.strobe + pulse.beat        | ⛔ CPU, 102 400   | dispositivo, 5 estágios
  motion.slit_scan                  | ⛔ CPU, 102 400   | dispositivo, 4 estágios
```

⇒ **9 dos 10 nós do grupo ficam no dispositivo.** Sobra o `motion.trail` (§5, W1d). E o
estroboscópio é **o consumidor** que o ciclo 6 deixou a apontar para aqui: os nove `pulse.*`
estavam no dispositivo sem ninguém que os lesse (doc 110 §8.6).

### ⭐⭐ O substrato novo: o UNIFORM DERIVADO (`ph2d_nodegraph::gpu::DerivedUniform`)

O slot de um param declarado passa a levar `derive(params)` em vez do valor cru — canal lateral
append-only (`KernelResolver::derived_uniforms`, `NodeRegistry::register_derived_uniforms`),
lido num sítio só (o empacotamento do uniform no `encode.rs`). A lei de contagem, a variante e a
aplicabilidade continuam a ver o valor cru.

⚠️ **Existe para a lei do kernel ser a MESMA função que a da CPU.** O estroboscópio guarda a
DURAÇÃO do flash e multiplica o brilho, a cada tique, por `(1/255)^(1/ticks)` — um `libm::powf`.
Portado como `pow` do WGSL, cada fabricante arredondaria à sua maneira e o erro COMPÕE-SE: ⛔ **a
mutação que o faz MORRE** (o `pow` desta placa não dá o mesmo `f32`). Com o derivado, **o estado do
estroboscópio é EXACTO nas duas rotas** — pior `|Δ| = 0` em 48 tiques, nos dois casos. Os clamps de
documento (`f32::max`, que engole um `NaN`) passam pelo mesmo canal.

⚠️ E o registo ficou no tecto (`lib.rs` a 698): os canais do GPU (as nove `register_*` e o `impl
KernelResolver`) foram para um módulo irmão, `gpu_channels.rs`, **verbatim** — `lib.rs` 595.

### ⭐ O slit-scan guarda o anel em MATRIZES — só no dispositivo

A CPU guarda a linha de atraso em **32 colunas `vec2`**; portadas tal e qual, o passe ligaria
**67** buffers de armazenamento (um Metal pára em 31). O dispositivo guarda o MESMO anel em
**quatro `mat4x4`** por elemento (64 números = 32 posições): **11** ligações, a conta do
`motion.integrate`. ⚠️ **Só é legítimo porque o estado nunca atravessa a costura** (o planeador
recua um `pre` que viria da CPU; a descida só lê o sink). ⚠️ E pediu uma correcção no gerador: a
identidade de uma coluna MATRIZ era um `vec4` (o módulo com o anel ausente não validava) — hoje é
cada coluna igual à declarada, com gate. ⚠️ **Um anel AUSENTE é a pose VIVA**, não uma constante
(o `past` da CPU re-semeia), logo o corpo ramifica no `HAS_*`.

### Os gates e as provas

- `gpu_cpu_parity_strobe` (adaptador real): o flash de omissão e o com FORMA (subida 3, platô 2,
  queda 12, curva, campo, modo a `4,5`, `probability 0,6` — **1 070 pulsos recusados**). Compara as
  TRÊS colunas do estado (exactas) e o look (`size`/`tint`, pior `2,4e-7`) tique a tique; controlo
  de não-vazio (pico, queda, subida). **11 de 11 mutações MORTAS** (sem o derivado · o `pow` do
  WGSL · idade ausente = 0 · pista só nos aceites · hash · curva · campo · modo · `round` do WGSL ·
  platô/subida · alfa).
- ⚠️⚠️ **A 1.ª redacção acusou o estroboscópio de uma divergência que era do METRÓNOMO:** a linha 200
  no tique 9 dava `(0,15 − 0,46)/0,31 = −1` EXACTO — o fio da navalha DECLARADO do `pulse.beat`
  (`floor` com o `playhead` em `f32`). A fixture passou a números longe dele (`0,3137`/`0,001731`) e
  o gate **prova-o** em `f64` antes de correr (margem medida `1,49e-5`, contra `~3e-7` de erro).
- `gpu_cpu_parity_slit_scan`: três `lag` (omissão, fraccionário com campo, acima do anel) e as
  identidades (`0`, um elemento), comparando `P` **e o anel inteiro** (as faixas do dispositivo
  decodificadas para as 32 posições da CPU) em 44 tiques. Pior `7,2e-5` — o ε é do
  `motion.oscillator` a montante. **7 de 7 mutações MORTAS.**
- A catraca da rota perdeu as duas linhas.

---

## §4-quater — ✅ W1d: o rastro vai para o dispositivo — e o grupo inteiro fica lá

```text
  grid 320² -> scale -> X -> output | antes            | depois
  ----------------------------------|------------------|------------------------
  motion.trail                      | ⛔ CPU, 102 400   | dispositivo, 4 estágios
```

⇒ ⭐⭐⭐ **OS DEZ NÓS DO GRUPO FICAM NO DISPOSITIVO** (eram 4 quando o ciclo abriu). A catraca
`the_fx_group_route_only_improves` tem a lista `NA_CPU` **vazia** — e as duas metades ficam: um
nó novo da categoria que nasça sem kernel reprova ali, com o nome.

### ⭐⭐ O substrato novo: `StreamOp::Carry`

Cada tique o rastro é `filtrar(estado) ++ vivo`, e só depois o corpo — e nenhum verbo do
sequenciador o dizia num estágio. O `Carry` são três passos e um corpo:

1. o **predicado** corre sobre a porta do estado CRU, **com** as reduções dele dobradas antes (a
   porta única do espaçamento — *há algum eco na faixa `1..s`?* — é uma pergunta sobre o estado
   INTEIRO) e com os uniforms derivados; o estado é compactado como num `Compact`;
2. os sobreviventes e a porta viva são **juntados** (`encode_join`, de que o `Concat` do
   `motion.combine` passou a ser o caso sem identidades) com as `ConcatFill` a dar a identidade de
   cada coluna que um lado não tem — ⚠️ **o `Concat` enchia de ZEROS**, e um `size`/`tint` a zero
   apaga o eco; os valores que não são zero escrevem-se por uma passagem de preenchimento NA PLACA
   (subir o padrão seria `n × 24` bytes por quadro);
3. o **corpo** corre sobre a junção, e sabe onde acabam os carregados pela contagem viva CRUA.
4. ⚠️ **O caso de UM eco** é a entrada tal e qual na CPU (com o modo por cima quando há modo): o
   `Carry` responde-o por uma lei sobre as contagens cruas e corre então um kernel próprio — ⇒ o
   sequenciador passou a perguntar «é passa-tudo?» à VARIANTE resolvida (um kernel sem variantes
   responde por si; nada do que existia muda).

⭐ **E o `DerivedUniform` cresceu duas coisas**: o contexto passou a ser o da lei de contagem (as
contagens CRUAS — a janela do rastro sai da contagem VIVA com o tecto de instâncias), e o slot pode
ser um NOME NOVO que só a derivação conhece (a matriz de cor do rastro — trigonometria da `libm` —
viaja como nove números; o manifesto não tem nove params para lhe emprestar; o planeador aceita os
dois).

⚠️ **Divergência DECLARADA (um tique, uma borda):** quando nada sobrevive, a CPU ainda junta as
COLUNAS do estado (um `gather` de zero linhas continua a tê-las) e o dispositivo não. Só se vê se a
montante uma coluna deixar de existir exactamente nesse tique. ⚠️ E um `spin` sub-normal liga a
variante do `rot` sem que a CPU o materialize (a coluna existe com os valores que tinha).

### Os gates e as provas

- `gpu_cpu_parity_trail` (adaptador real): o rastro de omissão; o ARMADO (espaçamento 3 · cor ·
  giro · teto de estreia · modo · campo, com cor e giro ANIMADO a montante); as duas variantes do
  meio (só giro, só modo); e as identidades (um eco, um eco com modo, espaçamento acima do tecto).
  Compara, tique a tique, **o conjunto de colunas** (nem a mais) e **todas** as colunas da CPU:
  exacto fora das ondas (o armado a `5,8e-6`, da matriz de cor). **12 de 12 mutações MORTAS**
  (promoção · campo no predicado · identidade do `size` · partição · matriz de cor · teto · a banda
  com a cabeça · a janela sem o derivado · identidade nunca · o `fade` cru · a idade da cabeça · o
  giro também na cabeça).
- ⚠️⚠️ **Três fixturas mentiam antes de medirem o produto:** a matriz de cor não muda o BRANCO (a
  mutação que a desligava sobreviveu), uma cabeça sem `rot` a montante lê zero de qualquer maneira,
  e — a mais subtil — **com o giro PARADO o buffer reciclado de dois quadros antes já tinha o valor
  certo** nas linhas da cabeça, e a mutação «a cabeça não escreve o `rot`» sobrevivia de forma
  INTERMITENTE. A cura foi no KERNEL e não na fixture: **toda coluna escrita é escrita uma vez,
  depois do ramo, para todas as linhas** — a forma de erro deixou de existir.
- A varredura do WGSL ganhou uma secção para o `Carry` (o predicado com as reduções e o canal
  partilhado, as próprias reduções, o kernel do caso identidade, e as variantes do corpo **aos
  PARES** — a de giro-E-modo escapava à varredura de um param só).
- O `stream_op.rs` chegou ao tecto: a projecção do `value.attribute` foi para um módulo filho,
  VERBATIM — e o gate que lia o ficheiro em runtime (`the_projection_modes_agree…`) passou a
  `include_str!` (falha a compilar, se voltar a mudar).

### ⭐⭐ O tecto do rastro, re-medido no dispositivo

`trail_row_ceiling_probe` (`--release`, cauda cheia de 32 ecos, o menor de cinco corridas):

```text
  vivos   │ linhas     │ disp. ms │ CPU ms
    8 100 │    259 200 │     0,85 │   6,81
   32 761 │  1 048 352 │     2,96 │  30,26
   65 536 │  2 097 152 │     5,15 │  65,59   ← o tecto novo (31–33 % de um quadro)
   97 969 │  3 135 008 │     8,24 │ 106,88
  196 249 │  6 279 968 │    16,08 │ 275,11
```

⇒ **~2,6 ns por linha** (a compactação lê 8 bytes e a junção copia — o dobro de um `fx.*`), e o
tecto passa de `262 144` a **`2 097 152`**: a cauda de 32 ecos deixa de encurtar a partir de
`8 193` objectos e passa a encurtar a partir de `65 537`. ⚠️ **Não é o número dos `fx.*`**, e o gate
dos tectos passou a prender CADA tecto ao SEU literal medido (os dois `fx.*` · o rastro · o
L-System, que continua só na CPU).

---

## §4-quinquies — ✅ W2: o cartão e o alcance

As três sondas do ciclo 6 sobre o grupo, e o que elas (e uma que elas não fazem) disseram:

1. ✅ **O alcance** (`motion_param_reach`, sobre o catálogo inteiro): **zero** params enterrados. As
   diferenças `params`/`no cartão` do §2 **são gates de modo** — o `rgb_split` esconde a lente no
   `Split`, o `tint` a segunda cor fora do `Gradient`, e as cores são UMA linha. §2.3 confirmado.
2. ⚠️ **O vocabulário por chave** lista três chaves partilhadas com rótulos diferentes (`ramp` ·
   `saturation` · `source`) — **perguntas diferentes em nós diferentes, sem consumidor partilhado**:
   a regra do doc 110 §9.7 (declarado, não curado; renomear custaria o valor gravado de todo
   documento e compraria zero). ⚠️ O §2.2 dizia «zero linhas» — lia só a lista (1).
3. ⛔⛔ **E a sonda NÃO VIA o defeito real:** três nós escrevem a coluna `blend` com a MESMA escada
   (`Sink · Normal · Add…`) e chamavam-lhe *Shadow Blend*, *Flash Operator* e *Echo Operator* —
   chaves diferentes E rótulos diferentes, logo invisível a um agrupamento por um dos dois. ⇒ os
   três dizem **`<quem> Blend`** (`Shadow Blend` · `Flash Blend` · `Echo Blend`; o `motion.output`
   diz `Blend`). ⚠️ Só o índice é guardado: nenhum documento se move. ⭐ Gate
   **`the_row_blend_speaks_one_word`** — a pergunta é semântica, e o que a torna sintáctica é a
   primeira palavra da escada (`Sink` só existe nesta): todo `Enum` que começa em `Sink` chama-se
   `<quem> Blend`, derivado do registry, piso `3`, **mutação RED** (o `Flash Operator` de volta).
   A legenda da cena `=77`, que mandava trocar o `Echo Operator`/`Flash Operator`, acompanha.

### O que o cartão mudou

| nó | antes | depois |
|---|---|---|
| `motion.strobe` | o `Flash Operator` pintado **no topo**, antes do `Envelope` (não tinha secção) | `Flash Blend` em **Look**, ao lado do tamanho e da cor do flash |
| `motion.trail` | o `Tail Alpha Max` no **topo** (sem secção), longe do `Tail Alpha` — e o comentário do hint dizia que a ordem era a leitura | o par junto em **Decay** (teto → ponta) |
| `motion.trail` | o `Forward Steps` **sempre** visível — e é INERTE no `Remembered` (o próprio código o dizia; a cura tinha sido só a secção) | escondido fora do `Resampled` (`ParamGate`); o alcance continua verde |

```text
  motion.strobe | Attack · Hold · Decay · Shape · Probability · Size Boost · Flash Blend · Flash
                > secções: Envelope@0 · Look@5
  motion.trail  | Length · Spacing · Tail Alpha Max · Tail Alpha · Tail Size · Tail Spin ·
                  Tail Hue Shift · Tail Saturation · Echo Blend · Source
                > secções: Decay@2 · Colour@6 · Source@9
```

⚠️ **O `fx.drop_shadow` fica com o modo no TOPO, e é decisão escrita** (no hint dele: *o modo é a
pergunta que decide o que as outras significam*). ⚠️ E um param fora de secção **não é
gateável** como defeito: é assim que os essenciais sobem (o `Length`/`Spacing` do rastro) — o
retrato do cartão é o instrumento, não uma catraca.

---

## §4-sexies — ✅ W3: o poder que faltava — e ele era UM, com a recusa que o escondia

O placar da conferência está a **zero** nas folhas do grupo (06 · 07 · 09 · 11 — a *dirt texture*
da folha 11 já tinha fechado com a cena `=107`), e as referências de cada nó foram conferidas lá.
O §2.4 marcava o `motion.slit_scan` de um botão só como o candidato óbvio, e a nota que o dizia
*«magro por natureza»* (doc 88 §9.2) foi lida outra vez:

- ⚠️⚠️ **O eixo «por um `motion.sort` a montante» REORDENA o stream para sempre** — a ordem de
  desenho, o pareamento por índice, o `id` —, que é o efeito colateral que a folha 10 da
  conferência nomeou depois e curou no `field.index_range` (o posto por atributo, sem reordenar).
- E o campo **multiplicava** a rampa por índice (`lag · i/(n−1) · falloff`): um atraso só pela
  POSIÇÃO — o *Time Displacement* do AE, onde o mapa diz o atraso de cada pixel — não se exprimia.

⇒ **`Delay By`** (param apendado e neutro): `Order` = a rampa de sempre, **ao bit** · `Field` =
`lag · falloff`, o campo sozinho (um elemento só também atrasa). Com
`field.index_range(Attribute = P.x)` é o slit da esquerda para a direita com a ordem intacta; com
qualquer `field.*` é um mapa de atraso. Nos dois caminhos: a paridade do dispositivo ganhou o caso
(pior `7,2e-5`, o ε do oscilador a montante), e as mutações morrem na LEI **e na LEITURA** — a
primeira prova só apanhava a lei (o gate chamava o `step` direto), e o `eval` que ignorasse o param
sobrevivia; o gate novo coze pelo grafo com `Delay By = Field`. Emenda escrita no doc 88 §9.2 e uma
linha nova na folha 04. Os outros três pontos da §9.2 (a direção, a forma, o *Time Resolution*)
ficam de pé.

---

## §4-septies — ✅ W4: a MEDIÇÃO — `10 de 10` no dispositivo, os DOIS relógios, e um nó em série curado

### A residência (independente da carga)

`probe_does_an_fx_chain_stay_on_the_device` (a tabela do §3, depois): **os dez** ficam no
dispositivo, `4` estágios, **nada sobe** (o controlo `grid → scale → output` tem `3`; o
`motion.strobe` com `pulse.beat` tem `5`).

⭐ **E a residência POR MODO passou a ser uma catraca** — `the_fx_modes_that_leave_the_device_are_named`
varre **todo `Enum` e todo `Toggle`** dos dez nós pelos valores que o hint oferece (a catraca do
§4 só via os defaults, e é cega a um modo cujo `applicable` derruba o kernel). Uma escolha só cai:
`motion.trail · Source = Resampled` (ADR-0163 — re-cozinha a própria entrada, CPU por desenho). As
duas metades (o que cai está nomeado; o nomeado continua a cair), com piso de `20` escolhas varridas;
mutação (esvaziar a lista): **RED**.

### Os DOIS relógios — a sonda nova

⚠️ **Os ciclos 5 e 6 mediram só a CPU de referência** e escreveram que o relógio do dispositivo não
tinha sonda. Este grupo está inteiro na placa, então
[`motion_bridge_aparencia_relogio.rs`](../../crates/ph2d-app-motion/src/motion_bridge_aparencia_relogio.rs)
(`measure_the_fx_group_on_both_engines`) coze a MESMA cadeia pelos dois motores, quadro a quadro,
**em regime** (mediana de `9` quadros depois de `120` tiques), com três cuidados que uma tabela
ingénua deste grupo não teria:

- **o estado** — a cadeia passa pela canalização do PRODUTO (`plumbing::reconcile_after`, a porta que
  o editor corre ao largar o nó), e a coluna das linhas prova que a cauda encheu (`n × 9`);
- **o neutro** — o nó é medido **acordado** (`motion_ciclo_preco::acordar`);
- **o pulso** — uma porta chamada `pulse` (lida do manifesto) recebe um `pulse.beat`.

⭐ **E a sonda confere-se:** a CPU e o dispositivo emitiram o **mesmo número de linhas** em todas as
células (um `⚠️` na coluna seria um desacordo de contagem).

`grid lado² → oscillator → X → output`, `--release`, lançada por
[`medir_quando_calmo.sh`](ferramentas/medir_quando_calmo.sh) (quatro amostras seguidas de `load` ≤
`4,5`; durante as corridas o 1-min foi de `3,75` a `5,47`). Cita-se o **menor de duas corridas**:

```text
  1 000 000 objectos     │ linhas    │ disp ms │ CPU ms │ ns/linha disp
  (sem X — a base)       │ 1 000 000 │   2,05  │   3,48 │  2,05
  fx.drop_shadow         │ 1 000 000 │   1,89  │   3,22 │  1,89   ← sombra macia DESLIGADA pelo tecto
  fx.glow                │ 1 000 000 │   2,03  │   3,41 │  2,03   (passa-tudo)
  fx.rgb_split           │ 3 000 000 │   5,42  │  41,54 │  1,81
  motion.color_array     │ 1 000 000 │   1,81  │  16,16 │  1,81
  motion.color_ramp      │ 1 000 000 │   1,83  │  26,21 │  1,83
  motion.slit_scan       │ 1 000 000 │   3,49  │  41,22 │  3,49   (era 16,33 — ver abaixo)
  motion.strobe          │ 1 000 000 │   1,97  │  18,44 │  1,97
  motion.sub_uv          │ 1 000 000 │   2,24  │  15,59 │  2,24
  motion.tint            │ 1 000 000 │   1,86  │  17,09 │  1,86
  motion.trail           │ 2 000 000 │   4,31  │  44,52 │  2,15   ← cauda CORTADA a 2 gerações pelo tecto

  102 400 objectos       │ linhas    │ disp ms │ CPU ms
  (sem X — a base)       │   102 400 │   0,19  │   0,29
  fx.drop_shadow         │ 1 740 800 │   3,05  │  21,82
  fx.glow                │   102 400 │   0,19  │   0,34
  fx.rgb_split           │   307 200 │   1,09  │   2,47
  motion.color_array     │   102 400 │   0,20  │   1,90
  motion.color_ramp      │   102 400 │   0,22  │   2,84
  motion.slit_scan       │   102 400 │   0,43  │   4,32   (era 1,80)
  motion.strobe          │   102 400 │   0,22  │   1,97
  motion.sub_uv          │   102 400 │   0,21  │   1,42
  motion.tint            │   102 400 │   0,20  │   1,77
  motion.trail           │   921 600 │   2,35  │  23,21
```

⇒ **~1,8–2,3 ns por linha na placa** para nove dos dez, contra `7–15×` isso na CPU nos que mudam
colunas. ⚠️ **As duas linhas de um milhão com `←` não medem o efeito inteiro:** acima do tecto a
sombra macia desliga-se e a cauda encurta — é o comportamento declarado no §4-bis/§4-quater, e é
ele que as mantém abaixo dos `6 ms`; a `102 400` os dois efeitos estão inteiros.

### ⛔⛔ E a medição achou um nó em SÉRIE — o `motion.slit_scan` custava `17,6 ns` por linha

A primeira corrida (a mesma sonda, `load` `2,76`–`3,43`) leu o slit-scan a **`16,33`/`16,41 ms`** a um
milhão — **um quadro inteiro sozinho**, `9×` o custo por linha dos irmãos. A causa era o kernel da
W1c, escrito nesta linha: o corpo chamava `ss_at(ss_old, …)` **34 vezes por elemento**, e o `ss_old`
é um `array<mat4x4<f32>, 4>` passado **por valor** — `64` números copiados a cada chamada.

⇒ o anel é lido **UMA vez** para `16` colunas `vec4`, a consulta é `ss_pick(coluna, vivo, k)` (a
posição `k` mora na coluna `(k−1)/2`, em `.xy` ou `.zw`), e o avanço do anel é uma **translação de
colunas** (`nova[j] = (velha[j−1].zw, velha[j].xy)`, com a pose viva a entrar na `0`).
**`16,33 → 3,49 ms`** a um milhão, **`1,80 → 0,43`** a `102 400`. A paridade do dispositivo corre com
o MESMO erro de antes (pior `P` e pior anel `7,2e-5`, os seis casos), a validação do WGSL passa, e a
mutação que troca a translação (`nova[j] = velha[j].xyxy`): **RED** nos dois gates de paridade.
⚠️ Fica a `~3,5 ns` por linha — o dobro dos irmãos, e é o preço de ler e escrever `64` números por
elemento, não um defeito.

---

## §4-octies — ✅ W5 (a construção): a cena `=118` e o tutorial *«A cor e o rasto»*

### A cena — três fileiras, uma pergunta por fileira

```text
  CIMA    A COR      Tint                      |  Color Ramp (no lugar dele)
  MEIO    O RASTO    cada peça anda em roda    |  + Trail
  BAIXO   O TEMPO    Slit Scan: ORDEM          |  Slit Scan: CAMPO (+ Falloff linear)
```

Seis panos `6 × 6` iguais ([`motion_state_aparencia_demo.rs`](../../crates/ph2d-app-motion/src/motion_state_aparencia_demo.rs)).
A de cima é **parada de propósito** (a cor não precisa de tempo). Em cima muda um cartão (troca), no
meio um cartão a mais; em baixo mudam **a linha `Delay By` e o campo que ela lê** — o anúncio diz as
duas, porque o modo novo SEM campo não tem nada para mostrar. Os dois `Slit Scan` têm **nome**
(`set_label`, o molde da `=115`). ⛔ O brilho, a sombra e a separação ficam de fora: o brilho acende
a cena INTEIRA e não teria par; o tutorial ensina os três no mapa do grupo.

**Catorze gates** (doze da cena, dois do tutorial), um por promessa do anúncio —
incluindo as que pedem ao dono que MEXA num controlo (os gates mexem no mesmo param que a linha do
cartão escreve):

| promessa | a régua | medido |
|---|---|---|
| cima: uma cor contra uma por peça | cores distintas | `1` contra `35` |
| passo 3: o `Tint` pinta a esquerda inteira e não a direita | a coluna `tint` | — |
| meio: roda, e só a direita com cauda | o RAIO de cada linha ao centro da sua célula; a contagem | `±0,1 %` do raio; `36` contra `432` |
| passo 6: `Length` no fim fecha o anel; `Tail Alpha` 1 não apaga | o maior buraco angular numa célula; a alfa mínima | `231°` → `< 30°`; `1` |
| `Tail Size` 1 não encolhe | o `size` de toda linha | `= PECA` |
| baixo: ordem contra lugar | o espalhamento DENTRO de uma coluna | `0,24` contra `1,2e-7` |
| passo 8: `Field` sem campo faz a onda sumir (e o pano continua a mexer) | os dois espalhamentos; duas alturas | `< 1e-4` |
| passo 9: `Invert` troca o lado | qual coluna anda com o oscilador sem atraso | `0` → `5` |
| `Circle`: os cantos primeiro | o canto = vivo; o meio ≠ vivo | — |
| cada par difere pelo que o anúncio diz | contagem de tipos + os params dos dois `Slit Scan` | `36` nós |

**Mutações:** `Delay By` igual dos dois lados · o `phase_stagger = 0` apagado (o default do
oscilador já faz uma onda sozinho) · o rasto retirado · o raio do campo a cobrir os cantos — as
quatro **RED**.

⭐⭐ **Três coisas que só a construção revelou:**

1. ⚠️ **A peça `0` nasce em BAIXO** — a 1.ª redacção do anúncio dizia *«a onda corre como quem lê um
   texto»*; a grade é row-major a partir do `y` menor, então ela SOBE o pano. A frase mudou e há gate.
2. ⚠️⚠️ **A FIGURA do tutorial mostrou um defeito da CENA:** com a peça a `0,16` e a roda a `0,1` o
   rasto saía uma **mancha colada à peça**, não um arco — e era isso que o dono veria. A cena passou
   a peça `0,10`, roda `0,14`, passo `0,40` (o anel inteiro, `0,38`, cabe no passo).
3. ⛔ **Um passo do tutorial era impossível e saiu antes de ser escrito:** *«arraste `Spacing` para a
   cauda virar uma fila de peças»* — doze cópias de `0,1` não cabem separadas num anel de `0,88` de
   perímetro, e com o `Spacing` alto a cauda dá mais de uma volta e as cópias SOBREPÕEM-SE. Trocado
   por `Tail Size`, com gate.

⚠️ E o `every_row_the_appearance_tutorial_names_is_on_the_card` reprovou sobre um
texto certo na 1.ª redacção: a linha `End` do `Tint` só aparece com `Mode = Gradient`, que é
exactamente a ordem do passo 12 — o gate passou a perguntar ao cartão NESSE estado.

### O tutorial

[`07_a_cor_e_o_rasto.pdf`](tutoriais/07_a_cor_e_o_rasto.pdf) — **6 páginas**, fonte em
[`src/07_a_cor_e_o_rasto.html`](tutoriais/src/07_a_cor_e_o_rasto.html). Oito capítulos: abrir · a
cor · o rasto · o tempo · vá além · **o resto do grupo** (a tabela *«quando você quer… procure
por…»* com os dez) · o que isto custa (a tabela de `102 400` da §4-septies, a língua do dono) · se
algo não bater. As seis figuras saem da cena (`write_the_appearance_figures`, as duas primeiras pares
pintam os DADOS: cor, alfa e tamanho de cada peça).

⚠️ **O CSS dos tutoriais ignora o `start` de uma lista** (`counter-reset:s`) — a numeração
recomeçava em `1` em cada capítulo e o texto cita «o passo 7». Este tutorial repõe o contador por
lista; ⏳ **três** dos anteriores têm a mesma forma e não foram tocados (`04`, `05` e `06` usam
`start=`; os `01`–`03` não).

### O smoke, para o dono

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=118 cargo run -p ph2d-host-desktop --profile smoke
```

---

## §5 — A fila do ciclo

1. ✅ **W1a — o brilho passa-tudo** (§4).
2. ✅ **W1b — os que MULTIPLICAM as linhas** (§4-bis) — os dois no dispositivo, e o
   `MAX_INSTANCES` re-medido lá (`262 144 → 3 145 728`).
3. ✅ **W1c — o estroboscópio e o slit-scan** (§4-ter).
4. ✅ **W1d — o `motion.trail` (modo `Remembered`)** (§4-quater). O preço que estava escrito: cada tique o nó é `transformar(filtrar(estado)) ++ materializar(vivo)`. ⛔ Nenhum verbo do
   sequenciador exprime isso num estágio — o `Compact` filtra UMA porta e o `Concat` junta portas
   cruas com ZEROS onde falta coluna, e aqui as reservadas têm identidade própria (`size`/`tint`
   `1`, `uv_rect` o atlas inteiro). E o predicado do filtro faz uma pergunta sobre o estado INTEIRO
   (*há algum eco na faixa `1..spacing`?* — a porta única do espaçamento), que hoje nenhum predicado
   pode fazer (o `encode_compact` não lhe passa reduções). Mais: a janela depende da CONTAGEM viva
   (`MAX_INSTANCES / n`) e a cor é uma MATRIZ de 12 números composta por tique (`colour::compose`,
   com trigonometria) — dois derivados que o canal de hoje não leva (ele só reescreve slots de
   params DECLARADOS). ⚠️ O modo `Resampled` re-cozinha a própria entrada em N instantes
   (ADR-0163) e fica CPU **por desenho** (`applicable`). E o tecto do rasto é para MEDIR no
   dispositivo quando o kernel existir (doc-comment do `MAX_INSTANCES` dele).
5. ✅ **W2 — o cartão e o alcance** (§4-quinquies).
6. ✅ **W3 — o poder que falta** (§4-sexies).
7. ✅ **W4 — a MEDIÇÃO** (§4-septies) — `10 de 10` na placa, os dois relógios, e o slit-scan
   curado (`16,33 → 3,49 ms` a um milhão).
8. ⏳ **W5 — a cena e o TUTORIAL** *«A cor e o rasto»* — construídos (§4-octies); falta o **smoke
   do dono**.

⚠️ **A ordem W1 → W2 não é preferência: é a lei 1 do protocolo.** Um grupo cujo uso normal leva o
grafo inteiro para a CPU não fecha um ciclo com «tem mais botões».
