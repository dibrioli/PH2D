# ADR-0156 — O traço de AO é um GATHER por-vértice, e por isso o `rayon` entra na `ph2d-sdf`

- **Status:** **ACEITO** pelo Enio em 2026-08-06 (*"pode usar rayon. siga"*), depois de a decisão ser
  reapresentada sem jargão — ⚠️ **a primeira formulação foi recusada por ser ininteligível**
  (*"não sei do que vc está falando"*), e o registro disso fica aqui de propósito: um ADR cuja pergunta
  o dono não consegue ler não é uma decisão, é um carimbo.
- ⚠️ **O 0156 é o próximo livre no `main` de 2026-08-06** (o último é o 0155). Número de ADR escolhido
  numa linha paralela é **provisório**: se outra linha reivindicar o mesmo na mesma janela, **renumera na
  integração** — já aconteceu **três** vezes neste repo
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **Data:** 2026-08-06
- **Linha:** `line/sculpt3d`
- **Cofre do módulo:** [`docs/3D/`](../../3D/00-INDEX.md)
- **Ampara-se em:** [ADR-0109](0109-rayon-exception-watercolor-composite.md) (a regra e a cerca) ·
  [ADR-0147](0147-wet-paint-order-invariant-solver.md) (o precedente EXATO da soma em float privada)

## O problema, e a força que obriga a decidir agora

O AO assado (`docs/3D/05.1` §3) marcha cones contra o campo SDF e guarda a visibilidade por vértice.
O kernel existe e está gateado (`ph2d-sdf::bake_ao`, 25 gates). **Medido pela porta do produto**, na
malha que a cena `=16` abre:

| | serial |
|---|---|
| o campo (voxelizar + flood, `res 128`) | 301 ms |
| **o traço** (425 602 vértices, 32 cones) | **786 ms** |
| **o bake completo** | **~1,09 s** |

A força que obriga: o `CLAUDE.md` §0 diz que **o teto é o do hardware, nunca o do caminho lento**. A
máquina tem 32 núcleos e o traço roda em **um**. Deixar 18× na mesa num módulo cujo pedido escrito é
*"performance acima do ZBrush e do Blender"* é exatamente o que aquele parágrafo recusa.

E há uma segunda força, processual: a `ph2d-sdf` **declara no próprio `Cargo.toml`** que não tem
`rayon`, com o mecanismo escrito (as caixas de dois triângulos do voxelizador se sobrepõem ⇒ a escrita
não é disjunta). Essa frase está **certa sobre o voxelizador e não alcança o traço** — e uma dep que
entra sem desfazer a frase deixa o `Cargo.toml` a mentir.

## Decisão

> **O `rayon` entra na `ph2d-sdf` exclusivamente no traço de AO, porque ele é um *gather* por-vértice
> contra um campo imutável — e a byte-identidade não é argumentada, é MEDIDA em 2, 4, 8, 16 e 32
> threads. O voxelizador e o flood fill continuam seriais, pelo mecanismo que o `Cargo.toml` já
> nomeia.**

### Por que ele qualifica — os 3 invariantes do ADR-0109, um a um

1. **Sem redução ENTRE vértices.** Cada `ao[v]` é função pura de entradas imutáveis: o campo (só lido),
   `positions[v]`, `normals[v]` e a tabela de direções (construída antes do laço).
   ⚠️ **Há uma soma — a média sobre os cones — e é o precedente do [ADR-0147](0147-wet-paint-order-invariant-solver.md),
   não uma exceção nova:** ela é **privada do vértice** e percorre a tabela de direções **na mesma ordem**
   em serial e em paralelo, então o resultado em `f32` é o mesmo bit a bit. O que a condição 3 do
   ADR-0145 recusa é soma cuja **ordem entre threads** possa mudar; esta não atravessa thread nenhuma.
2. **Sem estado mutável compartilhado.** Cada tarefa escreve **só** o seu `ao[v]`. O campo entra por
   `&VoxelField` e nenhum caminho do traço o muta (`sample` é `&self`).
3. **Sem RNG e sem transcendental no laço quente.** Os dois únicos transcendentais do módulo
   (`sin`/`cos` da rede de Fibonacci) rodam **uma vez por bake**, ao construir a tabela de direções,
   fora do laço. Dentro dele há `+ − × ÷`, `min`, `max`, `sqrt` e comparação — todos especificados
   exatamente pelo IEEE-754.

### A prova, que é medição e não raciocínio

`measure_whether_the_trace_is_worth_parallelising`, 425 602 vértices:

| threads | ms | speedup | bit-idêntico ao serial |
|---|---|---|---|
| 1 | 807,7 | 1,00 | — |
| 2 | 403,5 | 2,00 | **sim** |
| 4 | 226,5 | 3,57 | **sim** |
| 8 | 133,4 | 6,06 | **sim** |
| 16 | 72,5 | 11,14 | **sim** |
| **32** | **43,7** | **18,49** | **sim** |

⚠️ **E o número foi RE-MEDIDO depois, pela rota que de fato shipa** — o `rayon` contra o
`bake_ao_serial` congelado, em `ao_tests::measure_the_parallel_gain`: **764,8 → 39,3 ms, speedup
19,44×, zero vértices divergentes**. A tabela acima é a evidência que *decidiu*; esta linha é a que
*confirma o que entrou*, e as duas ficam porque medem coisas diferentes (a viabilidade e o produto).

## Alternativas consideradas — e o preço de cada uma

| alternativa | por que NÃO | o número |
|---|---|---|
| **Ficar serial** | é a opção que o §0 nomeia: o caminho lento definindo o teto do rápido | **807,7 → 43,7 ms**; um botão de 1,09 s vira um de 0,35 s |
| **Cortar cones** (8 em vez de 32) | compra velocidade **piorando a resposta**: o viés rasante do módulo cresce quando os cones caem, e há gate medindo isso (`o_vies_rasante_encolhe_com_mais_cones`). O `rayon` compra a mesma velocidade com o resultado **byte-idêntico** | 192 ms serial (4,1× mais barato) e uma peça mais clara do que a geometria manda |
| **`std::thread::scope` com fatias fixas** (sem dep) | ⚠️ **foi o que eu MEDI** — os 18,49× da tabela saíram dele, então ele funciona. Recusado por **desequilíbrio de carga previsto e não medido**: a fixture é uma ESFERA, onde todo vértice custa o mesmo; numa escultura real o vértice numa cavidade marcha mais que o de uma crista, e fatia fixa faz a thread azarada segurar o bake. O work-stealing do `rayon` é a resposta a isso, e ele já é o idioma da crate irmã (`ph2d-mesh` o usa em normais e curvatura) | 18,49× na esfera; **o desequilíbrio não foi medido** e está nomeado como tal |
| **Levar o traço para a GPU** | seria uma **segunda resposta** a *"o que este cone enxerga"*, e o ADR-0150 já decidiu que o kernel de escultura é CPU. Além disso o campo teria de residir no device (9,4 MB a `res 128`) e o ganho é medido contra **43,7 ms**, não contra 807 | não medido, de propósito: o número que ele teria de bater mudou nesta wave |
| **Paralelizar o CAMPO em vez** | é a metade que **não** é byte-idêntica sem resolver a sobreposição de escrita do voxelizador — exatamente o que o `Cargo.toml` diz | fica de fora, e vira a fronteira (ver Consequências) |

## O preço da escolhida (honesto)

- **Uma dep nova numa crate que tinha uma só.** A `ph2d-sdf` passa de 1 para 2 dependências
  (`ph2d-mesh` + `rayon`). É a **4ª exceção** do repo à regra "sem rayon" (ADR-0109, 0145, 0147).
- **O comentário do `Cargo.toml` fica MAIS longo, não menor:** ele passa a dizer as duas coisas — por que
  o traço pode e por que o voxelizador não pode. Apagar a metade antiga seria perder o mecanismo que
  impede a próxima linha de paralelizar o voxelizador por analogia.
- **O piso do pool não é escolhido nesta jornada.** As irmãs da `ph2d-mesh` reusam `normals::PAR_MIN`;
  aqui o custo por vértice é ~1 850 ns (contra dezenas nas normais), então o piso honesto é **muito**
  menor — e ele sai de uma varredura, não de um palpite. Até lá o laço paraleliza sempre, o que é
  seguro (o pior caso é uma malha minúscula pagar o overhead do pool).
- **A cerca continua de pé e este ADR não a alarga:** ele autoriza **`bake_ao` e nada mais**. Qualquer
  uso novo de `rayon` na `ph2d-sdf` — inclusive no voxelizador, inclusive no flood — **exige ADR
  próprio**, exatamente como este exigiu.

## O que fica GATEADO (para ninguém re-litigar por prosa)

| gate | o que trava |
|---|---|
| `o_bake_e_deterministico` | duas corridas dão os MESMOS bytes |
| **`o_bake_paralelo_e_byte_identico_ao_serial`** (novo) | a rota paralela contra a rota serial **congelada**, na mesma malha — a identidade é afirmada por comparação, não por argumento |
| `um_convexo_isolado_enxerga_o_ceu` · `o_aro_interno_de_um_toro_ve_menos_ceu_que_o_externo` | a resposta não muda de forma sob paralelismo |
| o comentário do `Cargo.toml` | cita este ADR e mantém o mecanismo do voxelizador |

## Consequências

⚠️ **O ganho MOVE a fronteira, e isso é o achado mais útil deste ADR.** Medido pela rota que shipa, o
traço cai para **36,9 ms** a 425 k vértices (87 ns/vértice) ⇒ o bake completo vai de **~1,09 s para
~338 ms**, e o **campo passa a ser 89% dele** (301 de 338). Quem for atrás do próximo ganho vai à
voxelização — e lá a pergunta não é de agendamento, é de **representação**: as caixas de dois
triângulos se sobrepõem, então o eixo honesto é a fatia em Z (com a fronteira entre fatias resolvida),
e isso é ADR próprio com medição própria.

## O que este ADR NÃO decide

- **Quantos cones e que alcance o produto usa.** São decisões de LOOK, e o alcance foi **medido como
  gratuito** (custo plano de 5,6 a 6,3 ms enquanto o raio cresce 6×) ⇒ o default atual (`maior lado ÷ 8`)
  é tímido sem economizar nada. Quem decide é o smoke do Enio.
- **Onde o canal de AO mora na malha.** O 6º plano por-vértice e as quatro portas
  (`rebuild` · `refresh_region` · `splice_topology` · `shrink_topology`) são a wave, não este ADR.
- **Se o AO é um botão.** Já estava decidido pela medição do campo, e a medição do traço só o reforça.
