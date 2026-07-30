# ADR-0145 — A camada 3D é um CAMPO que doa sombreamento, não um DCC embutido

- **Status:** **proposto, e a decisão central foi REVISADA antes do aceite** — aguarda aceite do Enio.
  (O número está livre no `main` de 2026-07-29; se outra linha o reivindicar na mesma janela,
  **renumera na integração** — [[feedback_numbers_that_sum_across_lines_count_dont_pick]].)

> ⚠️ **REVISÃO (2026-07-30), antes de qualquer código:** três requisitos novos chegaram depois deste
> ADR — o **Nomad Sculpt** virou a referência maior (e ele é **malha**, não campo), o runtime do jogo
> precisa de **rasterização com SSS/AO/Cavity** (logo uma malha tem de existir no fim, sempre), e a
> exigência de performance acima de ZBrush/Blender favorece uma malha residente sobre re-extração de
> campo. **A decisão passa a ser: malha primária, campo auxiliar (remesh · primitivas · esqueleto).**
> O escopo (*doar sombreamento, não ser um DCC*), o G-buffer, o `ph2d-light` unificado, o bake de
> runtime e a política de HR-5 **permanecem intactos**.
> Detalhe completo, com o que morre e o que sobrevive: [`docs/3D/02-Arquitetura/02.1-Representacao-malha-primaria.md`](../../3D/02-Arquitetura/02.1-Representacao-malha-primaria.md).
> Este ADR será **emendado** (não substituído) quando a linha abrir.
- **Data:** 2026-07-29
- **Linha:** a abrir (`line/sculpt3d`), sob ordem.
- **Avaliação que o originou:** [`docs/3D/00_avaliacao_camada_3d_sculpt.md`](../../3D/00_avaliacao_camada_3d_sculpt.md)
- **Contexto:** Enio pediu *"um layer 3d com o sculpt do Blender"* para que *"a malha 3d participe
  emprestando seu shading e sua luz por baixo das imagens 2d"*, *"algo como o Photoshop faz"*, e foi
  explícito: *"não quero que traga limitações, quero soluções"*.

## O problema, e a força que obriga a decidir agora

Um artista 2D quer **forma** (silhueta que vira, oclusão, luz coerente) sem virar modelador 3D. Hoje a
PH2D tem **relevo 2.5D** — o impasto: um campo de altura, material por-pixel e um passe de luz com 4
lâmpadas. Ele responde *"esta tinta tem corpo?"*. Ele **não pode** responder *"vire a cabeça"*: um campo
de altura não tem silhueta, não se auto-oclui e não gira.

A força que obriga a decidir **antes** de escrever qualquer linha é a mesma da ADR-0144: existe um
subsistema precioso no caminho. O passe de luz (`ImpastoLightPass` + `impasto_light.wgsl`) é guardado por
uma política de paridade escrita à mão — literais pinados por gate CPU-only, contratos estruturais pinados
EXATAMENTE, runtime conciliado contra o kernel canônico dentro de um épsilon documentado. **Um segundo
consumidor entra nesse passe ou ao lado dele, e a escolha decide se a tinta continua byte-idêntica.**

E há uma segunda força, externa: **o alvo citado no pedido não existe mais**. O 3D do Photoshop foi
descontinuado em 22.5 (agosto/2021) — workspace 3D, normal/bump maps, efeitos de iluminação, import/export
— porque a Adobe não conseguiu portar o toolset, todo OpenGL, para APIs de GPU modernas. Copiar a forma
dele seria copiar um desenho que morreu do problema que aqui já está resolvido.

## Decisão

**A camada 3D é uma camada do documento cujo produto é SOMBREAMENTO, e o núcleo dela é um campo de
distância com sinal (SDF) esparso, residente na GPU.**

Quatro partes, e cada uma é uma escolha:

1. **Representação = campo, não malha.** Bricks esparsos em banda estreita (custo O(superfície)),
   alocados por **compactação via `scan`** — o mesmo `ph2d-gpu-cook::scan` que a `line/gpu-nodes` usou na
   grade espacial (ADR-0140). Sem topologia, logo sem remesh, sem multires, sem dyntopo: a **representação
   apaga o caso especial** ([[feedback_the_representation_can_delete_the_special_case]]).
2. **Os verbos são do Blender; o código não.** Clean-room de comportamento, o precedente já shipado do
   `ph2d-painter-brush` sobre o Blender Texture Paint (*"só comportamento, nunca código"*). Os verbos
   herdam a álgebra que o sculpt 2D do Painter já pagou — `h = pre + k·Δ`, com `pre` **congelado no
   pen-down** — e Grab/Elastic/Snake Hook são **warp de domínio** (`d(p) = sdf(warp⁻¹(p))`), que é o
   *smear field* do Painter uma dimensão acima.
3. **A doação é um G-buffer, e o passe de luz vira do DOCUMENTO.** A camada escreve normal · depth ·
   `mats` (**os mesmos `[u8;7]`**) · cover · AO, e `ph2d-light` — hoje uma crate vazia reservada — passa a
   ser o modelo de lâmpada compartilhado: **um rig ilumina a tinta e a forma**. Camada 2D ganha o toggle
   *"iluminada pela forma abaixo"*; a forma projeta **sombra macia por sphere tracing**.
4. **No runtime, o 3D já virou canal.** Bake para normal/depth/AO/material no sprite, iluminado pelo
   **mesmo** modelo de lâmpada ⇒ WYSIWYG, custo de sprite normal-mapeado, roda em mobile. É o espelho
   exato do ADR-0131 (*runtime-truth + bake opcional*), com o campo em runtime ficando **atrás de
   medição**.

**Escopo negativo, e ele é a decisão tanto quanto o positivo:** isto **não** é um DCC. Não entrega quads,
UVs, retopo, multires nem export de asset 3D de produção. O produto é sombreamento.

## Alternativas consideradas — e o preço de cada uma

**(A) Portar o sculpt do Blender como ele é (malha + PBVH).** *Rejeitada.* O sculpt mode do Blender tem
**três** motores por dentro — `FACES`, `GRIDS` (multires) e `BMESH` (dyntopo) — cada um com estrutura de
dados, **pilha de undo** e código de render próprios, unificados só por uma camada de abstração de
iteração de vértices. Preço: portar três motores e três undos, e **pôr topologia na frente de um artista
2D**. Os próprios devs do Blender registram que o dyntopo *"foi sempre explicado como otimização quando na
verdade é bem mais lento e come muito mais memória"*. Compramos três problemas para resolver zero.

**(B) Base mesh + multires/displacement (a via ZBrush).** *Rejeitada.* Exige uma **malha base** — um passo
de autoria que o artista 2D não tem e não quer — e amarra detalhe a UV/topologia. Grab atravessando
fronteira de patch é exatamente onde esse modelo dói.

**(C) Promover o campo de altura que já temos para "3D".** *Rejeitada, e é a tentação barata.* Custa quase
nada e **não responde à pergunta**: altura não tem silhueta, não se auto-oclui, não gira. Seria entregar o
que já existe com nome novo.

**(D) Embutir um render 3D de terceiros (bevy_render / rend3 / three-d).** *Rejeitada.* Traz um segundo
modelo de câmera, material e luz para dentro de um app que já tem os três, e o compositor teria de
conciliar dois mundos — a doença que a casa já nomeou: *dois motores, um estado é pior que um motor lento*
([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]). E não daria a doação por-pixel, que é o
pedido.

**(E) 3D fora do app (modele no Blender, importe a passada).** *Rejeitada — é literalmente o caminho que a
Adobe tomou* ao matar o 3D do Photoshop e empurrar para o Substance. Quebra o laço de iteração do artista
e torna impossível a única coisa que o Enio pediu: a forma **dentro** do documento emprestando
sombreamento às camadas de cima.

## O preço da escolhida (honesto)

- **Detalhe de alta frequência custa memória.** A banda estreita torna o custo O(superfície), mas o teto
  de resolução é real — e ele **só entra no código depois da W0 medir**, por tier de hardware (§0.0: o
  caminho lento não define o teto do rápido).
- **Sem quads/UVs.** Export de asset 3D de produção exige retopo, e está fora de escopo (nomeado, com a
  porta).
- **A extração tem assinatura própria.** *Surface nets* não produz malha "de artista"; mitigado porque a
  **normal sai do gradiente do campo**, não da malha — mais lisa e sem a costura da tesselação.
- **O passe de luz precisa generalizar**, e generalizar um passe guardado por paridade exige o gate de
  *fingerprint* mantendo o caminho da tinta **byte-idêntico** (o molde do `fade_fingerprint` da timeline).
- **`PROJECT_SCHEMA` bumpa** — pelo caminho **inverso**: variant apendado em `LayerKind` não move índice
  postcard, mas arquivo novo não abre em binário velho. O asset da escultura viaja como **blob que carrega
  a própria versão** (o precedente do `TimelineDoc`), e é isso que impede um bump por wave.

## Consequências

- **Nenhum contrato congelado é tocado** (§6): `Tool=12` · `CanvasPaintTool=1` · `RasterEditTool=5` ·
  `PanelEvent=4` · `NodeOp=2` · `OpResolver=1` · `NodeManifest=8`. A ferramenta cabe em
  `on_canvas_pointer`; **navegação orbital é do shell**, nunca da ferramenta — ele já é dono de pan/zoom.
- **HR-5 é honrado pela isenção que ele mesmo escreve:** GPU compute é proibido em pipeline determinístico
  *"Radiance Cascades aceito apenas porque é puramente visual (não influi em estado simulado)"*. A camada
  3D é puramente visual, e o campo é **armazenado**, nunca re-derivado por replay de traços — não há
  armadilha de reprodutibilidade a administrar.
- **HR-4 já reserva o espaço:** 3,5 ms de render principal (a tabela nomeia **SDF**) + 2,5 ms de lighting.
- **HR-13 emendado (ADR-0117) vale desde a W0:** quem declara budget **possui um gate que MEDE** (dhat).
  A sonda da W0 nasce com ele.
- O undo herda a lei que a casa já pagou duas vezes (ADR-0117 · Painter U1): delta por **janela**, teto em
  **BYTES**, orçamento função do documento — e aqui **a janela é o AABB do pincel**, entregue por quem
  escreve (a lei S1 do doc 28: *a janela vem de quem escreve, nunca é redescoberta*).

## O que esta ADR NÃO decide

Fica para a medição da W0, e entra no código **com a tabela ao lado** (§0.0):

- o teto de resolução efetiva, por tier;
- o tamanho do brick (8³ × 16³);
- se a exibição é *surface nets* rasterizado, sphere tracing direto, ou os dois por contexto;
- a precisão de armazenamento da banda (f16 × 8-bit quantizado).

Nenhum desses números é escolhido nesta ADR **de propósito**: um limite legítimo diz de que recurso ele é
e traz a medição; um limite que só diz "por segurança" é um palpite esperando um smoke.
