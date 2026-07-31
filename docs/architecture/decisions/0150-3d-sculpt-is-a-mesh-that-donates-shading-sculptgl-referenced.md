# ADR-0150 — A escultura 3D é uma MALHA que doa sombreamento, referenciada no SculptGL (MIT)

- **Status:** proposto — aguarda aceite do Enio.
  ⚠️ **O número 0145 está livre no `main` de 2026-07-30** (o último é o 0144). Número de ADR escolhido
  numa linha paralela é **provisório**: se outra linha reivindicar o mesmo na mesma janela, **renumera na
  integração** — já aconteceu duas vezes neste repo
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **Data:** 2026-07-29 · **reescrito 2026-07-30**
- **Linha:** `line/sculpt3d`
- **Cofre do módulo:** [`docs/3D/`](../../3D/00-INDEX.md)

> ⚠️ **Este ADR foi reescrito, não emendado.** A versão de 2026-07-29 decidia *campo SDF como
> representação primária* e um *MVP em TypeScript* antes do Rust. As duas decisões caíram, por motivos
> diferentes e registrados: a representação em [`02.1`](../../3D/02-Arquitetura/02.1-Representacao-malha-primaria.md),
> o MVP em [`02.4`](../../3D/02-Arquitetura/02.4-Por-que-nao-ha-MVP-em-TypeScript.md). Reescrever em vez
> de emendar é deliberado: o ADR nunca chegou ao `main`, então não há decisão publicada a preservar — e
> um ADR cuja decisão real mora no cabeçalho de revisão, e não no corpo, é o comentário velho que mente.

## O problema, e a força que obriga a decidir agora

Um artista 2D quer **forma** — silhueta que vira, oclusão, luz coerente — sem virar modelador 3D. A PH2D
hoje tem **relevo 2.5D**: o impasto, um campo de altura com material por-pixel e um passe de luz de 4
lâmpadas. Ele responde *"esta tinta tem corpo?"*. Ele **não pode** responder *"vire a cabeça"* — um campo
de altura não tem silhueta, não se auto-oclui e não gira.

A força que obriga a decidir **antes** da primeira linha de código é a mesma do ADR-0144: há um subsistema
precioso no caminho. O passe de luz (`ImpastoLightPass` + `impasto_light.wgsl`) é guardado por uma política
de paridade escrita à mão — literais pinados por gate CPU-only, contratos estruturais pinados exatamente,
runtime conciliado contra o kernel canônico dentro de um épsilon documentado. **Um segundo consumidor entra
nesse passe ou ao lado dele, e a escolha decide se a tinta continua byte-idêntica.**

E há uma força externa: **o alvo citado no pedido não existe mais.** O 3D do Photoshop foi descontinuado em
22.5 (agosto/2021) — workspace 3D, normal/bump maps, efeitos de iluminação, import/export — porque a Adobe
não conseguiu portar o toolset, todo OpenGL, para APIs de GPU modernas. Copiar a forma dele seria copiar um
desenho que morreu de um problema que aqui já está pago.

## Decisão

**A escultura 3D é uma camada do documento cujo produto é SOMBREAMENTO. A representação primária é uma
MALHA de triângulos; o campo (SDF) é um gerador invocado sob comando. O motor é escrito diretamente em
Rust + wgpu, tendo o SculptGL (MIT) como referência de algoritmo que pode ser LIDA E ADAPTADA.**

Cinco partes, e cada uma é uma escolha:

### 1. Malha primária, campo auxiliar

A malha é a verdade; o campo gera malha (voxel remesh, primitivas paramétricas, esqueleto, assar AO) e
nunca é o display. Três requisitos forçam isso, e o segundo decide sozinho:

| Requisito | Consequência |
|---|---|
| O **Nomad** é a referência maior de UX | o Nomad é malha (voxel remesh + multires + dyntopo); um núcleo de campo entregaria outra ferramenta, com outro tato |
| **R-B: o runtime do jogo rasteriza triângulos** com SSS/AO/Cavity | **uma malha tem de existir no fim, sempre** |
| Performance acima de ZBrush/Blender | exibir um campo custa re-extrair ou traçar a cada traço; exibir uma malha residente custa ~zero |

**R-B é decisivo:** se a malha existe no fim de qualquer jeito, manter o campo como verdade primária
significa pagar a conversão o tempo todo e ter **duas respostas** para *"qual é a forma?"* — a doença que
esta casa já nomeou ([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]).

> **O campo é como a forma é REFEITA. A malha é como a forma É.**

Detalhe, com o que morre da versão anterior: [`02.1`](../../3D/02-Arquitetura/02.1-Representacao-malha-primaria.md).

### 2. Escrito direto em Rust; o SculptGL é a referência, e ele é MIT

Não há protótipo intermediário. O **SculptGL** — de **Stéphane Ginier, o mesmo autor do Nomad** — é um
aplicativo de escultura **completo e funcional**, sob **licença MIT**, com octree, topologia dinâmica,
multiresolução, voxel remesh (marching cubes + surface nets), subdivisão, preenchimento de furo e 13
pincéis. Ele **já é** a prova de que os algoritmos fecham, e a licença permite **ler e adaptar** — ao
contrário do Blender (GPL), onde a política da casa é clean-room de comportamento.

⚠️ **O que ele NÃO prova, e isso é metade da decisão:** ele é JavaScript de thread única sobre **WebGL**,
que não tem compute shader — logo a malha vive na CPU e o render é básico. Ele não diz nada sobre onde o
nosso kernel deve rodar, sobre o nosso orçamento de shader, nem sobre a doação, que ele não tem.
Detalhe, com o mapa arquivo→crate: [`03.4`](../../3D/03-Implementacao/03.4-Referencia-SculptGL.md).

### 3. Onde o motor roda é MEDIDO, atrás de uma porta única

A versão anterior deste ADR afirmava *"GPU-residente, e é por isso que ganhamos do ZBrush e do Blender"*.
**Essa afirmação não estava medida e foi rebaixada a hipótese.** O raciocínio que a derruba é de forma,
não de relógio: **um dab é limitado pela PEGADA** — ele toca os vértices sob o pincel, não a malha —, e a
vantagem de vazão de uma GPU se aplica a trabalho grande e paralelo, que um dab não é. Já as operações que
*são* de malha inteira (remesh, subdivisão, decimação) são **sequenciais e alocadoras**, exatamente a
classe que esta casa já mediu como hostil à GPU.

Decisão: **a malha nasce residente na CPU, com `rayon`; a GPU é dona do render, do G-buffer e da doação.**
A pergunta *"onde este kernel roda?"* é feita a **uma função** — o idioma que o repo já usa em
`plane_copy::worth_parallel` e `plane_fork::fork_par` — com o caminho CPU servindo de **oráculo** para o
caminho GPU, se e quando a medição o pedir.

**Kill-criterion escrito antes do build** (DIRETIVA §5): se um dab numa malha de 5 M triângulos passar de
**8 ms** na CPU — o mesmo teto que o Painter usa para um move —, o kernel migra para a GPU atrás da mesma
porta. O número é do Painter porque é o mesmo gesto humano; se a medição disser outro, o número muda **com
a tabela ao lado**. Detalhe: [`03.5`](../../3D/03-Implementacao/03.5-Onde-roda-o-motor.md).

### 4. A doação é um G-buffer, e o passe de luz vira do DOCUMENTO

A camada escreve `normal · depth · mats ([u8;7], os MESMOS do impasto) · cover · AO`, e `ph2d-light` —
hoje uma crate reservada e vazia (4 linhas, conferido) — passa a ser o modelo de lâmpada compartilhado:
**um rig ilumina a tinta e a forma**. Camada 2D ganha o interruptor *"iluminada pela forma abaixo"*.

```
antes:  normal  ←  ∇h da tinta
depois: normal  ←  ∇h da tinta   OU   G-buffer da malha
```

Lâmpadas, BRDF, LUT, material, quantização e paridade CPU/GPU ficam **idênticos** — é isso que torna a
costura aditiva e o caminho da tinta **byte-idêntico**, sob gate de fingerprint (o molde do
`fade_fingerprint` da timeline).

### 5. No runtime, o 3D já virou canal

Bake para `normal/depth/AO/material` no sprite, iluminado pelo **mesmo** modelo de lâmpada ⇒ WYSIWYG,
custo de sprite normal-mapeado, roda em mobile. Espelho exato do ADR-0131 (*runtime-truth + bake
opcional*), com a rota "malha ao vivo" atrás de medição.

**Escopo negativo, e ele é a decisão tanto quanto o positivo:** isto **não** é um DCC. Não entrega quads
de produção, UVs, retopo manual nem export de asset 3D de pipeline. O produto é sombreamento.

## Alternativas consideradas — e o preço de cada uma

**(A) MVP em TypeScript/WebGPU antes do Rust.** *Rejeitada — era a decisão anterior.* O protótipo existia
para de-riscar seis coisas; o SculptGL de-risca os algoritmos de graça, e as outras três (kernel na GPU,
orçamento de shader, a doação) **ele não de-risca e o protótipo também não de-riscaria bem**, porque
protótipo mede protótipo. Esta casa aprendeu isso três vezes na `line/Painter`: *"eu media peça isolada num
harness meu, em vez do produto"*. As medições vão para `tests/measure_*.rs` **dentro das crates reais**.
Registro completo: [`02.4`](../../3D/02-Arquitetura/02.4-Por-que-nao-ha-MVP-em-TypeScript.md).

**(B) Portar o sculpt do Blender como ele é (malha + PBVH).** *Rejeitada.* São **três** motores por dentro
— `FACES`, `GRIDS` (multires) e `BMESH` (dyntopo) — cada um com estrutura de dados, pilha de undo e render
próprios. Preço: portar três motores e três undos, sob **GPL**, e pôr topologia na frente de um artista 2D.
Os próprios devs do Blender registram que o dyntopo *"foi sempre explicado como otimização quando na
verdade é bem mais lento e come muito mais memória"*.

**(C) Campo SDF como representação primária.** *Rejeitada — era a decisão de 29/07.* Ver a parte 1: R-B a
derruba sozinha.

**(D) Base mesh + multires/displacement (a via ZBrush).** *Rejeitada.* Exige uma malha base, um passo de
autoria que o artista 2D não tem e não quer, e amarra detalhe a UV/topologia.

**(E) Embutir um render 3D de terceiros (bevy_render / rend3 / three-d).** *Rejeitada.* Traz um segundo
modelo de câmera, material e luz para um app que já tem os três, e não daria a doação por-pixel, que é o
pedido.

**(F) 3D fora do app (modele no Blender, importe a passada).** *Rejeitada — é literalmente o caminho que a
Adobe tomou* ao matar o 3D do Photoshop. Quebra o laço de iteração e torna impossível a única coisa pedida:
a forma **dentro** do documento emprestando sombreamento às camadas de cima.

## O preço da escolhida (honesto)

- **Topologia volta a existir como problema.** Mitigação: ela fica **escondida do artista** (um botão
  *Remesh*, um slider de resolução, um interruptor de dyntopo), e o SculptGL já resolveu os algoritmos.
- **A promessa de performance fica em aberto até a comparação existir.** O mecanismo mudou de *"GPU contra
  CPU"* para *"Rust nativo com 32 threads, limitado pela pegada, contra motores de thread quase única"* —
  e isso **não é vitória declarada**: o gate é a mesma malha, o mesmo gesto, os três apps, cronometrados.
- **MIT exige que o aviso viaje.** Adaptar código MIT é permitido e barato, mas a licença condiciona: onde
  um algoritmo for adaptado, o cabeçalho do arquivo nomeia a fonte e o módulo carrega o texto da licença.
  Custo: um arquivo de texto. Não fazer é que é risco.
- **`PROJECT_SCHEMA` bumpa uma vez**, pelo caminho **inverso**: variant apendado em `LayerKind` não move
  índice postcard, mas arquivo novo não abre em binário velho. O asset da escultura viaja como **blob que
  carrega a própria versão** (o precedente do `TimelineDoc`), e é isso que impede um bump por wave.

## Consequências

- **Nenhum contrato congelado é tocado** (§6): `Tool=12` · `CanvasPaintTool=1` · `RasterEditTool=5` ·
  `PanelEvent=4` · `NodeOp=2` · `OpResolver=1` · `NodeManifest=8`. A ferramenta cabe em
  `on_canvas_pointer`; **navegação orbital é do shell**, nunca da ferramenta — ele já é dono de pan/zoom.
- **HR-5 é honrado pela isenção que ele mesmo escreve:** GPU compute é proibido em pipeline determinístico,
  *"Radiance Cascades aceito apenas porque é puramente visual"*. A escultura é puramente visual, e a forma
  é **armazenada**, nunca re-derivada por replay de traços.
- **HR-4 já reserva o espaço:** 3,5 ms de render principal (a tabela nomeia SDF) + 2,5 ms de lighting.
- **HR-13 emendado (ADR-0117) vale desde a W1:** quem declara budget **possui um gate que MEDE** (dhat).
- **O undo herda a lei da casa, e NÃO a do SculptGL** — delta por **janela**, teto em **BYTES**, orçamento
  função do documento (ADR-0117 no áudio, U1 no Painter). Aqui a janela é a **lista de vértices tocados**,
  que o kernel do pincel já produz. ⚠️ O `Reversion.js`/`states/` do SculptGL guarda estados de malha;
  portá-lo importaria exatamente o defeito que esta casa pagou duas vezes.
- **O módulo é removível por construção** — uma feature flag, crates próprias, três costuras aditivas, e um
  gate que falha se qualquer arquivo fora do módulo o mencionar sem `#[cfg(feature)]`
  ([`02.3`](../../3D/02-Arquitetura/02.3-Modulo-removivel-e-mapa-de-crates.md)).

## O que este ADR NÃO decide

Fica para a medição, e entra no código **com a tabela ao lado** (§0.0):

- o teto de contagem de triângulos, por tier de hardware (ADR-0104);
- se o kernel de pincel migra para a GPU (o kill-criterion da parte 3 é quem decide);
- a resolução do G-buffer relativa ao sprite (onde a silhueta começa a serrilhar);
- o tamanho de voxel do remesh e a precisão de armazenamento dos atributos.

Nenhum desses números é escolhido aqui **de propósito**: um limite legítimo diz de que recurso ele é e traz
a medição; um limite que só diz "por segurança" é um palpite esperando um smoke.
