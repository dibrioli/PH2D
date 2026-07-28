# Handoff de integração — `line/Vector`: a PILHA de FX RASTER (9 tipos, GPU-resident)

**Plano:** [`docs/Vector Module/24_plano_fx_raster.md`](Vector%20Module/24_plano_fx_raster.md) ·
**Data:** 2026-07-26 · **W1..W8** na `line/Vector` · bugs em
[`BUGS_vector.md`](Vector%20Module/BUGS_vector.md) **#20-#23**.

O FX raster de alta qualidade para formas vetoriais — a resposta ao pedido *"efeitos FX de alta
qualidade, estado da arte, compatível com o que temos"*, e depois ao *"aqui tudo é para o game em
runtime, precisamos de total performance"*. Melhor que o Rive, cujo FX é **estreito** (feather +
blend, com sombra e brilho DERIVADOS do feather, sem pilha nenhuma).

- **W1 — a forma filtrada** (Blur · Outer Glow · Drop Shadow). **SMOKE APROVADO** pelo Enio
  (`=33`, incluindo maximizar + zoom: FPS liso, sem panic).
- **W2 — a PILHA componível** (escolha do Enio). **SMOKE APROVADO** pelo Enio.
- **W3 — o CATÁLOGO:** `Inner Shadow` · `Inner Glow` · `Outline` · `Color Overlay` (3 tipos → 7).
  **SMOKE APROVADO** pelo Enio, com três observações.
- **W4 — a REVISÃO** que essas observações pediram (o rim de 1 px · o modo `Contour` · a quina do
  contorno) + a auditoria dos sete tipos. **SMOKE APROVADO** pelo Enio.
- **W5 — o FEATHER e o BEVEL** (7 → **9 tipos**), os dois que o campo de distância da W4
  destravou. **Smoke REPROVADO** (pente/serrilha/ponta ceifada) → **W5b**.
- **W5b — os três artefatos do campo** e o gate que media a si mesmo. **Smoke REPROVADO** de novo
  (*"ainda com artefatos"*) → W6.
- **W6 — o campo pela GEOMETRIA e o perfil do bevel.** **Smoke: bevel APROVADO**, feather ainda não.
- **W7 — as DUAS convenções de cor** (luz linear · alfa reto). **Smoke APROVADO** — *"ficou ótimo"*.
- **W8 — a sombra interna deslocada + os modos do Glow** (pedidos do Enio no mesmo smoke).
  **Smoke APROVADO.**

> **Estado: linha FECHADA, TODOS os smokes APROVADOS** (2026-07-26). Tip: **`204bc3103`** (+ este
> handoff). A integração é feita por um **agente integrador dedicado**, por ordem explícita do Enio
> — esta linha entrega o handoff e **PARA** (§0.7). **Ship (push) é ordem SEPARADA.**
>
> ⚠️ **Esta branch carrega TRÊS features não integradas**, cada uma com handoff próprio: as **Color
> Harmonies** (`..._color_harmonies_2026-07-25.md`), o **Sketch + Hatch**
> (`..._sketch_hatch_2026-07-25.md`) e este FX raster. Elas são commits sequenciais da MESMA branch
> ⇒ **integram juntas, num rebase só**.

## A costura (o inegociável)

Um FX raster produz **PIXELS**, não `VecPath` ⇒ **não é PathEffect** (`effect::run_stack` é
`VecPath->VecPath`, puro, sem GPU, dentro da `ph2d-vec-scene`) **nem `LiveGeometry`**. É uma
`FxImages` que o **shell produz** e o `dispatch` só **encoda** no z da forma. É por isso que a
seção do painel se chama **Filters**, distinta de **Effects** (deformadores vetoriais, ADR-0132).

## ⚠️ ARQUITETURA: 100% RESIDENTE NA GPU

O 1º corte foi CPU-first (render→**readback GPU→CPU**→Gaussiana na CPU→**re-upload**) — padrão de
PREVIEW de editor. Em runtime a forma anima, e esse roundtrip roda por frame por forma: o readback
bloqueia o pipeline e o re-upload **vaza o atlas do Vello** (Blob nova por frame = id novo = upload
que cresce sem fim — medido: recook 37→793 ms num smoke parado). **Reescrito GPU-resident:**

- **`ph2d_render::FxStackPass`** (`fx_stack.rs`) — o fold da pilha na GPU: `2n+1` dispatches
  (Gaussiana separável H + V-finalize-composite por degrau, mais um `resolve` no fim).
  Intermediários em **`Rgba16Float`** (guardar premultiplicado em 8 bits e des-premultiplicar
  depois quantiza justamente a borda macia que o borrão existe para produzir). Globals de todos os
  passes escritos de uma vez e indexados por **offset dinâmico** — senão um `write_buffer` por
  passe antes de um único `submit` deixaria o último a valer para todos.
- **`VelloPass::register_texture`/`unregister_texture`** — uma textura da GPU vira imagem
  desenhável por **id ESTÁVEL**, sem upload de CPU; re-cozinhar escreve NA MESMA textura ⇒ zero
  churn de id no atlas. ⚠️ **`override_image` foi REMOVIDO**: ele troca a textura e **NÃO** atualiza
  `width`/`height` da `ImageData` ⇒ o Vello copia além da textura nova (`Copy 0..167 overruns
  source size 166`) — foi o *"panic ao zoom / deforma ao maximizar"*. **Resize RE-REGISTRA.**
- **`fx_live`** — por forma: um scratch `VelloPass` (render isolado, sem readback) → `FxStackPass`
  → textura de saída persistente registrada no **renderer PRINCIPAL** (o que desenha
  `vector_scene`; registrar noutro faz o Vello entrar em pânico). O memo é OTIMIZAÇÃO, não
  correção. Desregistro enfileirado (`forget` não tem `vello_pass`; o `recook` drena).

Custo por frame: 1 render isolado + `2n+1` passes + 1 cópia GPU→GPU no atlas — tudo na placa.
Medido no smoke: **6 pilhas, 0 re-cozidas, recook 0,01 ms** em cena parada.

## ⚠️ W2 — a PILHA, e o que ela MUDA no que já estava aprovado

**Todo op é imagem → imagem** (a frase que governa a pilha de geometria, traduzida para raster) ⇒
**Glow e Drop Shadow compõem o halo POR BAIXO da entrada DENTRO do próprio op**. Um op que
devolvesse duas camadas não poderia alimentar o seguinte.

⚠️ **Consequência de PRODUTO, nomeada:** o `FxMode::Below`/`Replace` **morreu**, e uma forma com
Glow/Drop Shadow passa a ser desenhada INTEIRAMENTE a partir da textura (a W1 desenhava o vetor
crisp por cima). O scratch rasteriza na escala EXATA da tela e o retângulo é alinhado ao pixel,
então é 1:1 — **mas é o olho do smoke que decide**, e a estrela 4 (`=33`) é o controle disso.

⚠️ **`ph2d_vec_render::FxImage` perdeu o campo `mode`** e `FxMode` deixou de existir. Único
chamador do `dispatch` é a `render_loop`.

## ⚠️ W3 — o CATÁLOGO, e as três decisões que ele carrega

Detalhe e números no [plano §8](Vector%20Module/24_plano_fx_raster.md). O resumo que a integração
precisa:

1. **Fora da textura é TRANSPARENTE, não `clamp`.** Trocar a extensão do kernel é o que dá **margem
   ZERO** aos degraus de dentro (para eles, *fora da textura* É *fora da forma*). Nos degraus de
   fora as duas respostas **coincidem** — a margem garante borda transparente —, então nada do que
   a W1/W2 desenhava se move. ⚠️ Um gate da W2 ficou vermelho com isso e a culpa era da **fixture**
   (barra flush contra a borda de uma textura de 8 px, situação que o `stack_reach` nunca produz);
   ela ganhou margem e ficou mais forte.
2. **O Outline é o mesmo kernel com um CORTE em `Φ(−1)`** ⇒ a borda para exactamente na largura
   pedida (medido: 3,5 px para 4 · 7,5 px para 8, transição ≤1 px contra 5-10 px de um Glow).
3. **O Color Overlay é PONTUAL:** um dispatch, margem zero (medido a 512²: 6 overlays 0,282 ms
   contra 0,646 ms de 6 borrões). `passes_of` é porta única — quem escreve os globals e quem
   despacha perguntam à mesma.

⚠️ **UMA tabela, quatro consumidores:** `ph2d_ecs::FxOp::SPECS` responde *o que este tipo é* para o
painel (que rows), o passe (margem + passes), os predicados `tints`/`displaces` e o **WGSL** — cujos
códigos são **GERADOS** (`kind_consts_wgsl`), não repetidos do outro lado da fronteira de linguagem.
O `FilterRowView` **perdeu o `label`** (o nome só pode vir da tabela) e
`set_filter_kind_names` virou **`set_filter_kinds(Vec<FilterKindView>)`**.

⚠️ **`ids::MAX_FILTER_KINDS` 3 → 7** (espelho de `FxOp::KINDS`; há gate na shell, o único lugar que
vê os dois lados).

## ⚠️ W4 — a REVISÃO (três observações do smoke, e a auditoria achou uma quarta)

Detalhe e números no [plano §9](Vector%20Module/24_plano_fx_raster.md).

1. **O rim claro de 1 px era um bug de COBERTURA.** O halo dos degraus de dentro era composto como
   uma CAMADA por cima, o que SOMA alfa (0,5 + 0,25 → 0,625 na borda anti-aliased); o `resolve`
   des-premultiplica, e dividir por um alfa maior CLAREIA. Agora um efeito de dentro **tinge o que
   já está lá** e o alfa não se move um bit.
2. **A auditoria achou o quarto defeito:** `opacidade 0` no Blur **apagava a forma** (`borrado × 0`)
   em vez de ser no-op. Gate de VARREDURA sobre a tabela — escolher um tipo teria acertado 6 em 7.
3. **O modo dos degraus de dentro** (`Proximity` | `Contour`): a proximidade não entra nas
   reentrâncias (medido numa cruz: 219 contra 155 na aresta), a distância entra (115 contra 104).
   **`Contour` é o default**, e o campo vem de um **JFA limitado** (`2 + bits(w)` passes).
4. **A quina do contorno:** miter é **impossível** a partir do alfa (derivação no plano — pediria
   3,24 × a largura numa ponta de estrela, e nenhuma dilatação é `w` na reta e `3,24 w` na quina).
   **Mas a medição achou um defeito real:** o corte num campo borrado ENCOLHE na quina — uma ponta
   de 36° recebia **0,0 px** de contorno. Agora o Outline é uma dilatação sobre o campo de
   distância: a ponta recebe **9,0 px** e a largura é a que o slider promete.

⚠️ **`FxOp` ganhou o campo `mode`** e `FxKindSpec` a lista `modes` (o painel pinta um chip por
modo). O `FxOp` é serializado DENTRO do blob do `VecFilter`, e postcard é posicional — mas o
`VecFilter` **não existe no `main`**, então nenhum arquivo salvo por um build de `main` pode
contê-lo: `PROJECT_SCHEMA` **não bumpa**. (Um projeto salvo por um build ANTERIOR *desta linha* não
carrega; nada fora da worktree é afetado.)

⚠️ **`ids::MAX_FILTER_MODES = 4`** (novo) + `filter_mode_id(row, mode)`.

## ⚠️ W5 — os dois tipos que o campo destravou

Detalhe no [plano §10](Vector%20Module/24_plano_fx_raster.md). **Nenhum dos dois precisou de
maquinaria nova** (são braços do `cs_op_field`) e **o painel não mudou uma linha** — a tabela o
dirige, que é exatamente o que ela existe para fazer.

- **Feather:** a borda vira uma rampa CENTRADA na fronteira e o **miolo fica intacto** — medido com
  listras dentro da forma: contraste 195 no feather contra **1** num borrão do mesmo raio.
- **Bevel:** `off` aponta para a borda mais próxima, então ele **É a normal 2D do rebordo**; medido
  sobre cinza, rim **225 / 30** contra miolo 128, e trocar a luz troca os dois.

⚠️ **A tabela passou a ROTULAR cada knob** (`offset_labels`, `color_label` no lugar dos bools): o
mesmo par de offset é um DESLOCAMENTO numa sombra e uma DIREÇÃO num bevel, e o card diz
**Light X / Y** num e **Offset X / Y** no outro.

⚠️ **A semente do campo é escolhida pelo que o op precisa** — os de dentro semeiam os texels de
FORA (exato onde importa), o feather e o contorno semeiam a CASCA (precisam dos dois lados).
Unificar foi **medido e recusado**: a casca de um lado só estima ~0,6 px pior na quina côncava, que
é onde o modo Contour existe para acertar.

⚠️ **`ids::MAX_FILTER_KINDS` 7 → 9.**

## ⚠️ W5b — os três artefatos, e o gate que media a SONDA

Detalhe no [plano §11](Vector%20Module/24_plano_fx_raster.md).

⚠️ **Primeiro achado: o gate da banda media a si mesmo.** Andar "paralelo à aresta" obriga a
arredondar o `y`, e ±0,5 px de sonda sobre uma banda de ~32 níveis/px são ±16 níveis INVENTADOS —
ele reportava 34 sobre um campo perfeito. E a fixture era a **45°**, o único ângulo onde a
discretização some por simetria. O oráculo virou um **BUCKET** por distância VERDADEIRA, num ângulo
oblíquo.

1. **O PENTE era a DIREÇÃO** (a distância mediu 0 níveis): o bevel tomava a normal como
   `normalize(off)`, que salta entre células de Voronoi ⇒ agora vem do **gradiente do campo**; e o
   feather amostrava a cor com o offset TRUNCADO, pousando às vezes em texel transparente ⇒ agora
   arredonda, entra meio texel e tem fallback.
2. **A SERRILHA do contorno** era a semente sub-texel supor uma rampa de AA de exatamente 1 px:
   numa aresta oblíqua ela é mais larga, e o erro (~0,09 px) lê como serrilha numa borda dura. A
   inclinação real está no gradiente ⇒ `2(a−0,5)/|∇a|`. **24 → 0 níveis.**
3. **A PONTA CEIFADA com traço** era o `path_screen_bounds` inflando só meia largura: a ponta do
   **miter** vai a `½w/sin(θ/2)` = **3,24 × ½w** numa ponta de estrela. Agora infla por
   `½w × miter_limit`, lido do MESMO construtor de traço que o renderer usa.

⚠️ **Isto toca a `ph2d-vec-render`** (`path_screen_bounds`), que tem outros consumidores além do FX
— o bbox fica MAIOR para formas com traço e junta miter, que é a resposta correta para todos.

## ⚠️ W6 — o campo deixa de ADIVINHAR a fronteira, e o bevel troca de PERFIL

**O campo agora vem da GEOMETRIA.** A rampa de anti-aliasing ocupa 1,0–1,41 texel e o estêncil que
a mede ocupa 2, então estimar a fronteira pela COBERTURA sempre inclui amostras saturadas e o erro é
função da **FASE** do texel na escada de rasterização. Medido: até **0,68 px** de erro de
profundidade e — numa aresta a 4,6°, onde a escada tem passo de 12,4 texels e um 3×3 lê a aresta
como horizontal — **erro de direção igual ao ÂNGULO INTEIRO**.

`ph2d_vec_render::silhouette_segments` (irmão de `draw_path_isolated`, MESMA resolução de transform)
entrega a silhueta em espaço de texel; o `cs_op_field` computa o **pé exato por texel**
(`exact_foot`) e — por definição de ponto mais próximo — **o vetor até o pé É a normal**, então não
há gradiente a estimar. Ripple: bevel **59,25 → 2,46** níveis, feather **5,56 → 0,82** (o controle
liso dá 2,40 / 0,00).

⚠️ **Com geometria o JFA não é despachado** (`Plan::Field { jumps, raster_seed }`): o caminho exato é
mais ACURADO **e mais BARATO** (0,51 ms com 64 segmentos contra 0,72 sem, a 512²). O teto
`MAX_SEGMENTS = 4096` é MEDIDO (3,94 ms) e estourá-lo devolve vazio ⇒ cai no raster, nunca trava.

⚠️ **`exact_foot` mede do CENTRO do texel**, não da quina — medir da quina metia 0,707 texel de erro
SISTEMÁTICO entre o campo e o raster que decide o sinal.

**E as "linhas pretas" do bevel eram o PERFIL, não o campo.** `1 − smoothstep(0, w, dist)` vale **1
em `dist = 0`**, ou seja punha o valor EXTREMO do sombreado no texel mais externo. Um bevel é uma
quina arredondada: a superfície começa PLANA na silhueta e o sombreado é a INCLINAÇÃO dela, logo se
anula nas DUAS pontas ⇒ **`4t(1−t)`**, a derivada normalizada de um smoothstep. Excursão do fio da
borda **166,6 → 29,8** com o meio da banda em 167,5 (razão 1,96 → **0,18**).

## ⚠️ W7 — as DUAS convenções de cor, e uma premissa do módulo que era FALSA

Esta é a wave que o integrador precisa ler com atenção: ela muda o **espaço de trabalho** do passe.

**(a) O ESPAÇO.** O `cs_resolve` fazia `rgb/a` sobre bytes sRGB, o que **não é** a
des-premultiplicação — é ela composta com uma transferência não-linear, e o erro cresce quando a
cobertura cai. Agora **o miolo fala LINEAR e só as duas pontas falam sRGB**: passe novo
**`cs_ingest`**, `cs_resolve` codificando, e o `tint` (que chega do painel em sRGB) atravessando a
porta única **`tint_lin`**. É a convenção de toda composição séria (o default `linearRGB` do
`color-interpolation-filters` do SVG, o *Blend Colors Using 1.0 Gamma* do AE, Nuke/Fusion,
OpenEXR/ACES) e a que o Vello já usa a montante.

**(b) O ALFA — e aqui a premissa estava invertida.** O doc deste módulo dizia *"premultiplicada — é
o que o Vello escreve"*. Medido no rasterizador REAL: **1696 de 1696** texels de cobertura parcial
trazem a cor CHEIA `(235,175,60)` com o alfa ao lado. **Alfa RETO.** O `cs_ingest` passou a
premultiplicar (DEPOIS de linearizar), que é o que Porter-Duff exige de toda a álgebra a jusante.

⚠️ **Por que atravessou dezenas de gates verdes:** num texel OPACO e num VAZIO as duas convenções
dão os MESMOS bytes — só a cobertura PARCIAL as separa, e **toda fixture de cobertura parcial do
módulo foi escrita pela mesma mão que escreveu a premissa**. Nenhum gate perguntava ao Vello.

**Mudanças de aparência que o integrador deve ESPERAR** (todas corretas, todas smokadas):

- o **blur/glow/drop shadow** deixam de somar bytes codificados ⇒ some a franja escura clássica (a
  média de preto com branco passa de 128 para os **187,5** que a luz manda);
- **sombras e halos leem mais CLAROS em bytes** para a mesma força — reduzir metade da LUZ é o byte
  188, não 128. Não é regressão: é a aritmética de gama que fazia crer o contrário;
- o **contorno tracejado** do feather **SUMIU**, e os dois rasterizadores da sonda passam a
  concordar texel a texel na banda.

⚠️ **Dois defeitos de FIXTURE consertados junto:** `fx_stack_feather_gpu.rs` carregava uma **CÓPIA**
da fixture oblíqua e ficou para trás (acusou **149 níveis** de desvio sobre produto correto — hoje
importa a partilhada); e o `make_src` não tinha `COPY_SRC`, então o modo analítico da sonda
`fx_look_probe` **nunca rodava** — só o `PH2D_FX_VELLO=1`, o que tornava impossível justamente a
comparação que nomeou a causa (b).

## ⚠️ W8 — a sombra interna deslocada, e os modos do Glow

**Inner Shadow.** Em modo Contour a força era `1 − smoothstep(0, w, dist)` com a distância **SEM
SINAL**, então um texel cujo ponto amostrado caía FORA da forma voltava a ter distância grande e a
sombra **desvanecia justamente do lado onde devia estar saturada**: a banda DESCOLAVA do contorno.
Medido com deslocamento 8 (luminância por profundidade, tinta crua ≈ 180):
`110 96 81 64 45 24 3 9 31 52` — o ponto mais escuro **7 texels dentro**, a borda **3,6× mais
clara**. Com `sdist` (com sinal, que o shader já computava duas linhas acima):
`0 0 0 0 0 0 0 9 31 52`.

⚠️ **Sem deslocamento é byte-idêntico ao anterior** (`sdist == +dist` para todo texel de dentro, e um
de fora é morto pelo `over.a` do `inner_tint`) — o defeito era só do caso deslocado. E o modo
**Proximity nunca o teve**: ele borra uma REGIÃO, e deslocar uma região preenchida continua a cobrir
a borda; deslocar um campo de DISTÂNCIA não.

**Glow ganha Proximity/Contour.** `INNER_MODES` → **`FALLOFF_MODES`**: o nome era acidente de quem
chegou primeiro, não propriedade da escolha. E o roteador dizia `spec.inner && Contour`, uma
**enumeração disfarçada de regra** — agora pergunta *"este tipo oferece escolha, e escolheu
Contour?"*, o que faz o Glow entrar por construção e o próximo tipo com modos também. Braço novo no
`cs_op_field`, o Glow entra no `seeds_shell` (sem semente o modo desenharia **NADA** no caminho do
raster) e o `op_reach` dele passa a ser a **LARGURA**, não `3σ`.

⚠️ **O Glow nasce em PROXIMITY**, não no Contour do `BLANK`: ele sempre foi a silhueta borrada, e
ganhar uma opção não pode repintar o que "Add Glow" quer dizer. Um Glow salvo antes carrega
`mode = 0`, que é exatamente este ⇒ **nenhum arquivo muda de aparência** e o `PROJECT_SCHEMA` não se
move (o campo já existia). A UI vem de graça: painel, `populate` e os dois seam gates são dirigidos
pela **TABELA**.

A lei que a medição fixou (alfa do halo a 3,5 texels da borda, os três sítios à MESMA distância):

| sítio | Proximity | Contour |
|---|---|---|
| quina reentrante | 156 | **202** |
| aresta reta | 110 | **202** |
| quina **convexa** | **45** | **202** |

⚠️ E ela **corrigiu a minha prosa**: eu tinha escrito o gate copiando o enredo do irmão de DENTRO
(*"a reentrância quase não acende"*) sem medir, e num halo EXTERNO o sinal se inverte — quem fica no
escuro é a **PONTA**, não o vão.

## ⚠️ O QUE ESTA WAVE (W6-W8) TOCA FORA DA `ph2d-render` — conferir colisão CROSS-LINHA

- **`crates/ph2d-ecs/src/vec_filter.rs`** — `INNER_MODES` **renomeado** para `FALLOFF_MODES` (o
  único símbolo com nome mudado nesta jornada; grep confirma **um** definidor e nenhum consumidor
  externo), `modes` do Glow, `FxOp::new(GLOW).mode` e um teste novo. **Sem campo novo, sem bump.**
- **`crates/ph2d-vec-render/src/silhouette.rs`** — módulo NOVO (`silhouette_segments` +
  `MAX_SEGMENTS`), aditivo.
- **`shells/desktop/src/fx_dump.rs`** — módulo NOVO (`PH2D_FX_DUMP`), + `mod` no `main.rs` e a
  chamada no `fx_live`. **`ph2d-color`** entra como `use` (já era dependência das duas crates ⇒
  **machete-safe**, nenhum `Cargo.toml` mudou).
- **`PROJECT_SCHEMA`: INTOCADO (31)** · **registry do `ph2d-ecs`: INTOCADO** · **§6 contrato
  congelado: INTACTO** (grep) · **`VEC_SCENE_SCHEMA`: intocado** · **nenhum ADR necessário**.
- **`FxStackPass::run` ganhou um 8º argumento** (`geom: &[[f32; 4]]`) — **um chamador só**, no
  `fx_live` da shell.

## Deltas que a integração precisa CONFERIR (o número se conta, não se escolhe)

| Item | Antes | Depois | Como |
|---|---|---|---|
| `ph2d-ecs` registry | 37 | **38** | `VecFilter` registrado (blob-key) |
| espelhos `ph2d-render`/`ph2d-script` | 38 | **39** | ecs+Sprite / ecs+LuauScript |
| `PROJECT_SCHEMA` | 31 | **31 (INTOCADO)** | componente por blob-key = sem bump posicional |
| `VEC_SCENE_SCHEMA` | 13 | **13 (intocado)** | — |
| **§6 contrato vetorial** | — | **INTACTO** | `architecture_vector_contract_surface` verde |
| `VECTOR_SECTIONS` (painel) | 26 | **27** (append) | Filters (gate de contagem atualizado) |
| `ids::MAX_FILTER_KINDS` | — | **9** | espelha `FxOp::KINDS` (W3 → W5) |
| `ids::MAX_FILTER_MODES` | — | **4** | o chip de modo dos degraus de dentro (W4) |

⚠️ Se outra linha mexer no registry/`PROJECT_SCHEMA` na MESMA janela, o número final **se
recalcula** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). O `VecFilter` não move
`PROJECT_SCHEMA` (blob-key), então esse eixo não conflita; o registry (38/39) pode.

## O que landou

- **`ph2d-ecs::FxOp` + `VecFilter { ops: Vec<FxOp> }`** ([vec_filter.rs](../crates/ph2d-ecs/src/vec_filter.rs)).
  `MAX_OPS = 6` — **o teto é do PAINEL, não da GPU**, e está MEDIDO (RTX, 512×512, sigma 8 px):
  `1 → 0,084 ms · 2 → 0,149 · 3 → 0,220 · 4 → 0,336 · 6 → **0,429 ms**`, linear a ~0,07 ms/degrau
  = **2,6 % de um frame de 60 fps** com a pilha cheia.
- **`ph2d-render::FxStackPass` + `FxOpGpu` + `stack_reach` + `kernel_half`** — `stack_reach` é a
  porta única da margem (as reaches SOMAM ao longo da pilha; assimétrica para a sombra), e ela e o
  shader perguntam o MESMO `kernel_half`.
- **`ph2d-vec-render`**: `FxImages`/`FxImage` + `dispatch(...,fx,...)` + `path_screen_bounds` +
  `draw_path_isolated`.
- **`fx_live`** (shell) — o produtor + `resolve_ops` (mundo→pixel, a câmera mora ali e em mais lado
  nenhum) + **`hit_of`** (id→controle, a porta que os TRÊS sítios da ponte usam).
- **Painel "Filters"** — uma LISTA de cards (o idioma da seção Effects): Add por tipo, e por card
  ✕ / ↑ / ↓ / 👁 + os controles do TIPO da linha. Ids **por-linha** (`filter_*_id(row)`).

## Gates

- **GPU do CATÁLOGO (`fx_stack_kinds_gpu.rs`, 12, W3+W4+W5):** o **feather** amacia a borda e
  deixa o miolo intacto (fixture com LISTRAS — numa forma lisa um borrão também não muda o miolo) ·
  o **bevel** acende a face virada para a luz e **troca com ela** (fixture CINZA — sobre branco o
  realce satura e a metade 'acende' fica verde de graça) · um degrau de dentro **nunca move a
  cobertura** (a fixture tem alfa em RAMPA — o fenômeno vive na fatia fracionária) · **opacidade 0 é
  no-op em TODO tipo** · o modo `Contour` **põe sombra na reentrância e o `Proximity` não** (a cruz,
  com as duas sondas à mesma distância da borda) · a banda **não serrilha** numa diagonal com AA (0
  níveis em 60 texels, com controle positivo) · o contorno é **redondo na quina, por derivação** ·
  e os 6 da W3: **todo tipo da tabela desenha alguma
  coisa** (o gate da wave — varre `FxOp::KINDS`, então um tipo novo entra nele no mesmo commit em
  que entra na tabela) · a sombra de dentro escurece a borda e **não vaza um texel** para fora · o
  contorno alcança a largura e para **duro** (com o Glow do mesmo σ como controle) · o Color
  Overlay repinta **sem mover cobertura** em três forças · o op pontual custa muito menos que um
  borrão · **a margem é um fato do TIPO** (puro, roda sem device).
- **GPU (`ph2d-render/tests/fx_stack_gpu.rs`, 8, `#[ignore]`):** rampa alarga com sigma · o halo é
  do EFEITO e a FORMA sobrevive por cima · **a ORDEM da pilha muda o desenho** (o gate da wave) ·
  pilha vazia é a identidade · o custo por degrau (medição) · `stack_reach` (puro, roda em qualquer
  runner) · os 3 de render/register/resize que reproduzem os panics da W1.
- **Shell (5):** os tetos painel↔motor concordam · degrau desligado nunca chega ao passe · o raio é
  de MUNDO (2× de zoom = 2× de borrão) · o offset cruza a câmera e cai em pixel inteiro · `hit_of`
  decodifica cada controle **e nada mais**.
- **Modelo (7, `ph2d-ecs`):** só os tipos com modos carregam um (quem não tem nasce em ZERO) · reordenar troca vizinhos e as pontas são no-ops · a pilha só está
  ativa com algum degrau ligado · um degrau novo nasce VISÍVEL (**varrendo a tabela**, e pedindo a
  ELA o que exigir) · o teto é resposta da pilha · a tabela é indexada pelo CÓDIGO e os nomes são
  únicos · `tints`/`displaces` são VISTAS da tabela, não uma segunda opinião.
- **Seam (7, `seam_filters.rs`):** os **chips de MODO** chegam ao bus e são pintados só em quem tem
  modos · os Add ao bus · os ícones do card ao bus · as setas das PONTAS
  **não** são desenhadas · cada linha pinta só os controles do TIPO dela, **varrendo a tabela inteira**, presença E ausência
  (a versão da W2 comparava dois tipos escritos à mão e teria ficado verde sobre os quatro novos) · a
  swatch é alvo de PICKER (não botão) · a seção não é oferecida sem forma.
- Fechamento: `cargo fmt` · clippy limpo · LOC caps (workspace + shell + painel) · §6 ·
  `node_id_collisions` (agora cobrindo as DUAS famílias por-linha) · `panel_wiring_parity` ·
  `cargo test --workspace --no-run` exit 0.

- **COR (`fx_stack_linear_gpu.rs`, 6, W7)** — o eixo que nenhum outro gate deste módulo media
  (todos olhavam a *variação AO LONGO da aresta*; um defeito CONSTANTE ao longo dela é invisível a
  esse oráculo, e passou duas vezes): a cor de um texel parcialmente coberto é a cor da FORMA (rampa
  inteira dentro de **2 níveis**) · a média de um borrão na fronteira preto/branco é **187,5**, não
  128 · overlay e halo pintam exatamente a cor da swatch · a ida e volta devolve os **256** bytes ·
  **`the_source_carries_straight_alpha_not_premultiplied`**, que interroga o **Vello de verdade** (é
  o gate que faltava, e a sua ausência custou a wave) · e um **arch-gate** que pina o `g.tint.rgb`
  cru dentro da única porta que o converte (enumerar os cinco sítios apodreceria no sexto).
- **MODOS (`fx_stack_modes_gpu.rs`, 4, W8)** — a sombra interna é mais escura NA BORDA em **três**
  deslocamentos (o zero é metade do gate: ali as duas leis coincidem) · o deslocamento satura a
  própria **PROJEÇÃO** contra a aresta oblíqua · o halo do Contour é função da distância e mais nada
  (202/202/202) e o do Proximity não (varia 111 níveis) · o halo do Contour para na largura autorada,
  **nos DOIS caminhos do campo** (sem a metade "sem geometria" a entrada do Glow no `seeds_shell`
  ficaria sem prova, e uma forma com traço desenharia nada).
- **BEVEL / FEATHER sobre aresta OBLÍQUA ANTIALIASADA (`fx_stack_bevel_gpu.rs` 3 +
  `fx_stack_feather_gpu.rs` 2, W5b/W6):** ⚠️ a inclinação é **IRRACIONAL de propósito** — sob uma
  normal racional todo texel à mesma distância é translação de rede de outro, a fase é idêntica em
  todos e o artefato mede **zero por construção** (foi assim que um pente de dezenas de níveis
  atravessou 13 gates verdes).
- **CUSTO do campo exato (`fx_stack_segment_cost.rs`)** — linear na contagem de segmentos, e o teto
  cabe num frame.
- **Modelo:** `the_falloff_choice_is_offered_where_it_means_something` (na `ph2d-ecs`) — ⚠️ ele mora
  ali porque o seam do painel é dirigido pela TABELA e por isso **não pode** testemunhar que a
  capacidade existe (medido: apagar os modos do Glow deixa o seam **verde**).

⚠️ **Os gates GPU são `#[ignore]`** — o integrador roda, na RTX, **cada um** destes com
`-- --ignored`: `fx_stack_gpu` (8) · `fx_stack_kinds_gpu` (13) · `fx_stack_linear_gpu` (6) ·
`fx_stack_modes_gpu` (4) · `fx_stack_bevel_gpu` (3) · `fx_stack_feather_gpu` (2) ·
`fx_stack_segment_cost` (1). Sem adapter eles fazem *skip gracioso*, **que não é verde**.

⚠️ **`smoke_fixture_renderable` (4 `todo!()`) falha em `--ignored` e é PRÉ-EXISTENTE** — scaffolds
do Sprite Inspector v2, de outro dono, sem relação com esta linha (conferido no `main`).

⚠️ **Barras RE-MEDIDAS na grandeza certa, não afrouxadas** (a W7 muda o espaço, então números
calibrados em GAMA deixaram de significar o que diziam): as sombras Contour/Proximity e o *"a aresta
escurece"* passam a medir em **LUZ** — e com isso a separação entre os dois modos MELHORA de 1,7×
para **2,6×**; o Color Overlay aprende que a mistura é de luz (`encode(1−força)`); o *"a ordem da
pilha muda o desenho"* teve o delta re-medido (28, barra 20 — a contagem de bytes diferentes é que
tem os dentes); e o custo do op pontual passou a ser **MARGINAL** (subtrai a moldura ingest+resolve,
que dilui qualquer razão bruta), o que o deixou em 0,23–0,33 contra uma barra de 0,5.

## ⚠️ Três lições que a wave pagou

1. **A minha 1ª mutação estava ERRADA e sobreviveu.** *"Todo op vira o primeiro"* não reproduz *"a
   ordem é ignorada"* — produz outra coisa errada (`[glow,blur]`→glow-glow e `[blur,glow]`→blur-blur
   **continuam diferentes**). A mutação honesta é **ORDENAR** a pilha: aí os dois lados ficam
   idênticos, `0 bytes diferentes`, RED. *Uma mutação que não sangra pode acusar a MUTAÇÃO, não o
   gate.*
2. **O seam nasceu VERMELHO e apontou um erro meu:** a swatch de cor registada como `button()`, e
   **um id só pode ter UM tipo de widget no store** — o Down abria o picker e nenhum `Click` saía.
   A mesma lição que o `vector_fx_toggle_id` já documentava.
3. **O `node_id_collisions` não cobria nem `vector_fx_*` nem a família nova** — as duas partilham o
   prefixo `vector.f…`, então *"os nomes são diferentes"* era uma afirmação por provar exatamente
   onde é duvidosa. As duas entraram no MESMO conjunto (é isso que prova a distinção).

4. **Uma premissa que só a cobertura PARCIAL pode contradizer não é contradita por fixtures suas**
   (W7). Pergunte ao produtor a montante — o gate que faltava renderiza com o Vello e CONTA os
   texels.
5. **Todo gate de um módulo pode medir o MESMO eixo, e o outro fica cego** (W6/W7). Antes de confiar
   numa suíte, pergunte de que EIXO ela fala.
6. **Copiar o enredo de um gate irmão sem medir** (W8): num halo externo o sinal se inverte, a
   asserção passava e a prosa mentia.
7. **Dois gates nasceram VERMELHOS por culpa do ORÁCULO, não do produto** (W8): eu esperava que um
   deslocamento de 16 saturasse 16 texels e o produto deu 14 (é um VETOR contra aresta oblíqua ⇒
   satura a PROJEÇÃO, `16·cos 23,7° = 14,7`); e a minha sonda "de aresta reta" caía **DENTRO** do
   braço da cruz, lendo o alfa da FORMA em vez do halo — o controle atropelado pelo experimento,
   4ª vez no projeto. A premissa agora é **declarada e verificada** dentro do gate.

## Smoke

`cd <worktree> && env PH2D_BUILD_SMOKE=33 cargo run -p ph2d-host-desktop --release` — **dezasseis
estrelas em quatro fileiras** (a cena imprime a legenda inteira, com os números medidos):

1. **A regressão:** controle · Blur · Glow · Drop Shadow.
2. **Os degraus de DENTRO, lado a lado:** o MESMO Inner Shadow em **Proximity** e em **Contour** ·
   Inner Glow · Color Overlay.
3. **O contorno e a composição:** Outline fino · o STICKER · a PILHA INTEIRA · **Outline GROSSO**
   (olhe as PONTAS).
4. **Os dois novos e o par de ordem:** **FEATHER** (a borda amacia, o miolo fica nítido — compare
   com o Blur da fileira 1) · **BEVEL** (o rebordo com luz de cima-à-esquerda) · `Glow → Blur` ×
   `Blur → Glow`.

Depois: **zoom** e **maximizar**. `PH2D_FX_PERF=1` imprime `N pilha(s), M re-cozida(s), recook X ms`.

**E o gesto do painel:** seção **Filters** → **nove** "Add"; cada card oferece só os controles do
tipo dele e **com o nome certo** (o Bevel diz *Light X/Y* e *Shadow*, a Drop Shadow diz *Offset*, o
Outline diz *Width*, o Feather não tem cor, o Color Overlay não tem raio) — e agora **TRÊS** tipos
trazem o chip **Mode: Proximity | Contour**: Inner Shadow, Inner Glow e o **Glow** (W8).

**Os dois gestos que a W8 acrescenta ao smoke** (foram os aprovados pelo Enio):

1. **Inner Shadow com Offset** — a sombra tem de **encostar no contorno** do lado para onde o
   deslocamento aponta, e clarear dali para dentro. A tira clara entre a borda e a banda era o bug.
2. **Glow → Mode** — *Proximity* dá o brilho que pooling em volta da massa (as pontas da estrela
   apagam); *Contour* dá um aro de largura constante que segue o contorno inteiro, vãos incluídos.

**A sonda headless** (render-and-look, sem app) fica em
`PH2D_FX_VELLO=1 PH2D_FX_LOOK_DIR=<dir> cargo test -p ph2d-render --test fx_look_probe -- --ignored`
— **doze** cenas, incluindo a matriz do Inner Shadow (os 2 modos × com e sem deslocamento) e os 2
modos do Glow lado a lado. ⚠️ **`PH2D_FX_VELLO=1` é o modo honesto**: sem ele a sonda usa o próprio
rasterizador, e foi a divergência entre os dois que nomeou a causa da W7.

## Aberto / follow-ups (nomeados, não contrabandeados)

- **W4 — o feather analítico** (erf da distância / SDF via JFA), quando a nitidez em zoom extremo
  importar. ⚠️ **Reordenado para depois do catálogo, com motivo:** o argumento dele é
  *resolution-crisp*, e o nosso borrão já o é (re-coze na escala da tela por frame, e há gate). O
  que a W3 comprou — tipos que a Gaussiana **não desenha** — é delta de produto.
- **W5 — os tipos que pedem maquinaria NOVA:** Bevel/Emboss (o `blur(alfa)` já é a altura), a
  turbulência + deslocamento (o eixo ORGÂNICO), e o **blend mode por degrau** (um campo a mais no
  `FxOp` e um `mix` a mais no finalize — não mexe na pilha).
- **O Outline arredonda quinas convexas** — é um corte no nível de uma Gaussiana isotrópica, logo
  aproxima a dilatação por DISCO (o que um pincel redondo faz). A dilatação exata é `O(r²)` por
  texel; não se justifica sem pedido.
- **Radius é slider em unidades de MUNDO** (`FILTER_RADIUS_MAX = 2.0`) — fração-do-tamanho seria
  mais robusto para formas de tamanhos diferentes (a mesma nota que o Contour faz do Offset).
- **O deslocamento da sombra é arredondado ao PIXEL** (o halo é amostrado por `textureLoad`, sem
  sampler). Invisível numa sombra; nomeado por honestidade.
- **`MAX_HALF = 96`** (sigma ≈ 32 px de tela): acima, o borrão satura — limite de CUSTO do passe.
- **A pilha de filtros não compõe com a de Effects numa ordem escolhida** (o filtro roda sempre
  DEPOIS da geometria cozida). É decisão de produto se um dia precisar.
- ~~⚠️ **Uma forma com TRAÇO cai no caminho do RASTER**~~ — **FECHADO em 2026-07-27** pela wave da
  silhueta resolvida (handoff `HANDOFF_INTEGRACAO_line_Vector_stroked_silhouette_2026-07-27.md`,
  BUGS #24). Era a causa do *"Linhas no Bevel"* que o Enio fotografou. O texto original fica abaixo,
  porque descreve corretamente o mecanismo:
- ⚠️ **Uma forma com TRAÇO cai no caminho do RASTER** (`silhouette_segments` devolve vazio): a
  silhueta desenhada é `preenchimento ∪ contorno-do-traço`, e a borda INTERNA do traço **não é
  fronteira de silhueta nenhuma** — semeá-la poria a fronteira no meio da forma. A união exata é
  trabalho da **booleana**, e entra quando houver quem a peça. O produto não trava: o campo fica
  menos acurado ali, exatamente como era antes da W6.
- **O `MAX_SEGMENTS = 4096`** é teto de CUSTO medido (3,94 ms a 512²); estourá-lo devolve vazio e
  cai no raster.
- **O erro de arredondamento do `PH2D_FX_DUMP`** — a sonda de pixels do app (`fx_dump.rs`) fica no
  build; ela só escreve quando a env está posta, uma vez por forma. É diagnóstico, não produto.
