# ARQUIVO — parte_2.md (história, 559 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_2.md`](parte_2.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-493` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## Bug #10 — havia uma TERCEIRA rota raster, e era ela: o passe do Flip nunca recebeu o sub-retângulo da cena

**Report (Enio, 2026-08-25/26):** *«no modo motion a imagem de referência sofre um drift no
pan com o mouse»*; refinado para *«o drift acontece para Object e Chip, não para Star»*; e,
depois da cura do [Bug #9](#bug-9), **«nenhuma melhoria»** + *«auditoria com agentes»*.

**Cena:** `PH2D_MOTION_OBJ_SMOKE=9`. No meio da tela ficam os TRÊS objectos que a cena
carimba — a arte de quatro cores (**Object**, um Flip), o quadrado laranja (**Chip**, uma
sprite de átlas) e a **Star** (um `VecPath` vectorial). São eles a *«imagem de referência»*.

### O mecanismo

Sob o split da tool Motion a cena vive num sub-retângulo do alvo
(`CenterSplit::scene_viewport` = `[0, 0, W, floor(H·t)]`, `t = 0,55`). **Três** rotas desenham
lá dentro, e até 2026-08-26 só duas sabiam disso:

| rota | quem | projeta com | px por unidade de mundo |
|---|---|---|---|
| sprites/instâncias | `Chip`, as cópias carimbadas | `uniform_for_subrect(w,h)` + `set_viewport` | `Hs/hw` |
| Vello | `Star` | `world_to_screen_affine(scene_camera_window)` | `Hs/hw` |
| **Flip** | **`Object`** | ⛔ `camera.view_proj(window)` — **a janela CHEIA** | **`H/hw`** |

⇒ o passe do Flip desenhava a `H/floor(H·t) = 1/t ≈ **1,82×**` a escala das outras duas.

⚠️ **E isso não é um offset, é um MULTIPLICADOR do arrasto.** O pan converte o deslocamento
do rato em mundo pela altura **da cena** (`field_gizmo::pan_scene_camera`), então a arte do
Flip anda `1,82×` o que o cursor anda: parada, é um desalinhamento fixo; a arrastar, é uma
abertura que **cresce com a distância percorrida e sem tecto**, e só fecha ao voltar. Era
isto — *«escorrega sempre para o mesmo lado quando se arrasta muito para o mesmo lado»*.

### A cura

`flip_pass::render` passa a receber o mesmo `scene_viewport` que o passe de sprites, e
`flip_pass_camera::camera_scene` projeta pelo sub-retângulo e **remapeia o NDC** para a
região que ele ocupa no alvo (`ndc' = a·ndc + b`, afim em `x`/`y`).

⛔ **A cura NÃO é um `set_viewport`, e é deliberado:** este passe compõe em alvos intermédios
do tamanho do alvo final e termina num blit de alvo INTEIRO — recortar obrigaria a
redimensionar a cadeia toda. A rota do Vello já resolve o mesmo problema sem recorte nenhum
(projeta com as dims da cena num alvo cheio e o conteúdo cai no canto); é essa que se copia.
O remapeamento **preserva a escala em pixel** — o que encolhe é o NDC, porque o alvo é maior
que a cena. Sem split (`None`) devolve a câmera que shipava **ao bit**.

⚠️ O `px_per_world` (a espessura do traço) passa a ser o **da cena**; o `viewport` do uniform
continua a ser o **do alvo**, porque o shader converte `in.clip`, que é coordenada de
fragmento do alvo cheio (`flip.wgsl:581`).

### ⚠️ A lei: era o QUARTO consumidor da mesma lei, e os três anteriores já tinham sido curados

O passe de sprites (M0.T13), o chrome/grade (`field_gizmo`) e as formas vectoriais
(2026-07-25, com o report *«objetos afastados do path, com drift em relação ao canvas»*) já
tinham recebido esta cura, cada um depois de um report próprio. *Uma lei que se cura no sítio
que reportou, e não numa porta, volta pelo consumidor seguinte* — e o seguinte era o único que
não tinha o parâmetro na assinatura, ou seja, o único que **não podia** estar certo por acaso.

### O gate — o instrumento que faltava

[`flip_pass_camera_tests.rs`](../../shells/desktop/src/render_loop/flip_pass_camera_tests.rs):
**as TRÊS portas × DOIS centros de câmera.** Ele exige que as três ponham o mesmo ponto de
mundo no mesmo pixel *e* que se **desloquem o mesmo** quando a câmera pana — que é o que
*drift* significa e o que nenhum quadro sozinho pode responder. Mais dois controles (sem
split é a câmera antiga ao bit; um sub-retângulo que é a janela inteira é inerte) e a escala
do traço. **5 gates, 1 mutação, 3 reprovam.**

### ⚠️ O que este bug ensinou sobre a SONDA — ela só podia imprimir ZERO

A sonda do Bug #8/#9 comparava a rota das sprites com a do Vello. Elas são a **mesma
expressão**: o `orthographic_rh` divide pelo half-extent, que traz o `w` do sub-retângulo, e
esse `w` **cancela** contra o da conversão NDC→pixel. ⇒ depois de as duas passarem a receber
o mesmo inteiro (a cura do Bug #9), `Δpx` e `Δtrunc` são **identidades algébricas** — nenhuma
máquina, nenhum `t`, nenhum centro de câmera as faz imprimir outra coisa. O mesmo vale para
`Δcam`: as duas leem o mesmo `&Camera2d` do mesmo quadro.

⇒ *Uma sonda que compara duas fórmulas SUAS mede a álgebra dela, não o produto.* Pergunte
sempre **que número a resposta contrária imprimiria**: aqui, para todos os cinco mecanismos
candidatos, a resposta era **zero**.

A aritmética mudou-se para o gate acima (que é onde ela pertence) e a sonda ficou só com o
que um teste **não pode** ver: o sub-retângulo **APLICADO** contra o pedido (o passe larga-o
em silêncio quando o quadro tem clip/máscara — decisão por CONTEÚDO, imprevisível para quem
chama; `SpriteRenderer::applied_subrect` é o efeito, não o argumento) e o mundo de uma
amostra de cada rota.

### Fica NOMEADO e não curado (latente, medido pela auditoria)

- **A2** — `renderer_draw.rs`, `scene_viewport.filter(|_| !has_clip && !has_mask)`: um
  `Mask2D`/`MaskInteraction`/`ClipChildren` em qualquer entidade do quadro derruba o
  sub-retângulo do passe raster inteiro, e o Vello não tem condição equivalente ⇒ o mesmo
  `1/t`. Não alcançável na cena `=9` (nenhuma delas nasce lá); a sonda agora **grita** se
  acontecer.
- **A3** — o raster arma o sub-retângulo por `motion_active` e o Vello por `center_split`:
  **dois predicados para o mesmo facto**. Coincidem hoje por convenção (o split é posto e
  limpo na aresta da ferramenta), não por construção.
- **A4** — o passe 3D (`sculpt3d`) escreve no mesmo `game_rt` com a janela cheia. Inerte sem
  cena armada.

---

## Bug #11 — a seta diferente era a ÚNICA CERTA: a família do RIG escreve o ângulo LOCAL na coluna que o desenho lê como MUNDO

**Estado:** ✅ **CURADO em 2026-09-19** (aguarda smoke do dono) — ver *A cura, aplicada*, no fim.

**Estado original (o diagnóstico, mantido verbatim):** ⏳ **ABERTO — causa MEDIDA, cura desenhada e NÃO aplicada** (o preço atravessa a
família inteira; ver *O preço*, abaixo). Report do dono, 2026-09-19, com foto: *«por que em
skeleton o último objeto tem direção diferente?»* — uma fila vertical de setas em que sete
apontam para a direita e a de baixo aponta para cima.

### O que a foto mostra, e o que ela NÃO é

⛔ **A seta diferente é a raiz da cadeia, e é a única que está certa.** O dono chamou-lhe *«o
último»* porque é a última da fila **visual** (a cadeia cresce para cima, logo o joint `0` fica em
baixo) — e a leitura natural, *«sete certas e uma errada»*, está invertida.

### A medição (sonda pelo caminho do produto, `rig.skeleton` de FÁBRICA)

```
n = 8
rot  (o que o DESENHO lê) = [90, 0, 0, 0, 0, 0, 0, 0]
wrot (o ângulo de MUNDO)  = [90, 90, 90, 90, 90, 90, 90, 90]
P                         = [[0,0], [0,0.7], [0,1.4], [0,2.1], …]   ← a coluna vertical da foto
```

O `P` sobe (a pose está certa) e o `rot` entrega `90°` na raiz e `0°` nas outras sete — que é a
imagem, seta a seta.

### A causa

O [`fk::resolve`] produz **duas** colunas de ângulo: `rot`, que fica sendo o ângulo **relativo ao
pai** (o que o autor escreveu), e `wrot`, o ângulo de **MUNDO** acumulado pela cinemática directa.
**O desenho lê `rot`** — [`ph2d_eval_motion::lower`] faz `stream.get("rot")` nas duas rotas, e o
dispositivo assa `"rot"` na posição `2` do [`LOWER_COLUMNS`].

⇒ *o ângulo de mundo é calculado, guardado, e nenhum consumidor de desenho o lê.* Só a raiz
coincide, porque uma raiz não tem pai.

### ⭐ Porque é que `rot` = MUNDO é o significado certo, e não uma opinião

Medido por censo: **72 sítios de produto** escrevem `rot`, e os de fora do rig — `motion.clone`,
`motion.collide`, `motion.look_at`, `motion.orbit`, `motion.distribute_*`, `motion.noise`,
`motion.drive` — escrevem todos a rotação do elemento **no mundo**. *A família do rig é a
excepção, e é ela que tem de se alinhar.*

### ⛔ A cura óbvia foi CONSTRUÍDA, MEDIDA e REVERTIDA — e o que ela partiu é o achado

Tentado: o `resolve` preserva o local numa coluna nova (`lrot`), publica o mundo em `rot`, e
prefere o `lrot` à entrada (para a idempotência que o
`resolve_is_idempotent_and_a_point_cloud_is_untouched` afirma).

Resultado: **quatro crates vermelhas** (`ik-2bone`, `fabrik`, `rubber-hose`, `skin-deformer`),
com a mão da IK a aterrar em `[3.0, 0.0]` em vez de chegar a `[1.5, 1.5]`.

⚠️⚠️ **O mecanismo:** um nó que **POSA** escreve ângulos locais em `rot` e volta a chamar o
`resolve`. Com a leitura a preferir o `lrot`, o solve que a IK acabou de escrever era **ignorado**
em favor do local da resolução anterior. ⇒ *a cura não é só do resolvedor: quem produz uma pose
tem de passar a escrever na coluna do LOCAL, e cada nó de pose tem a sua lei.*

### O preço, medido

- **SEIS cópias byte-a-byte** do `fk.rs` (`sha256 c9447f9c928f`): `rig-skeleton`, `rig-fk`,
  `rig-fabrik`, `rig-ik-2bone`, `rig-rubber-hose`, `rig-skin-deformer`. *A lei está duplicada, e
  é isso que faz a cura ser seis edições em vez de uma.*
- **TRÊS leis de pose** a migrar (a IK de dois ossos, o FABRIK, a mangueira).
- O `source.lsystem` tem o **mesmo defeito** por outro caminho (`turtle.rs` escreve `rot` local e
  `wrot` mundo, sem passar pelo `resolve`) ⇒ uma planta desenhada com formas aponta tudo para a
  direita.
- ⚠️ **O `wrot` NÃO pode ser apagado** por parecer um alias: o `rig.skin_deformer` LÊ-o em dois
  sítios (compara a pose de repouso com a posada).

### A cura que fica desenhada

Uma coluna, um significado: `rot` = **mundo** (o que a casa desenha), `lrot` = **local** (o que o
autor escreve e por onde a re-resolução entra), `wrot` mantido para o `skin_deformer`. Quem posa
escreve `lrot`. ⛔ Ensinar o LOWERING a ler o `wrot` **não** serve: o device assa oito colunas
numa assinatura de pipeline, e uma nona muda essa assinatura.

### Lições

- ⚠️ **Uma coluna partilhada com dois significados não dá erro — dá uma imagem errada.** O `rot`
  do rig e o `rot` do resto do módulo têm o mesmo nome e leis diferentes, e as suítes das duas
  famílias passam, cada uma a afirmar a sua.
- ⚠️ **Um valor CALCULADO que ninguém lê é um defeito à espera:** o `wrot` carrega a resposta
  certa desde que existe, e o único leitor dela fora do rig era um teste.
- ⚠️ **«O último objecto» de um report é o último da FILA VISUAL**, e a ordem do documento pode
  ser a oposta — a raiz de uma cadeia que cresce para cima desenha-se em baixo.

---

### ✅ A cura, APLICADA (2026-09-19)

**Uma coluna, um significado.** O `rot` que sai do [`fk::resolve`] passa a ser o ângulo de
**MUNDO** — a coluna que o desenho lê — e o ângulo relativo ao pai muda-se para `lrot`, por onde
a re-resolução entra. O `wrot` fica, porque tem leitores de produto (o `rig.skin_deformer` e os
três `pose.rs`).

Medido no `rig.skeleton` de fábrica, com a mesma sonda do diagnóstico:

```
antes:  rot = [90,  0,  0,  0,  0,  0,  0,  0]
depois: rot = [90, 90, 90, 90, 90, 90, 90, 90]      ← as posições não mudaram
```

#### ⛔⛔ O que a primeira tentativa provou, e que a cura teve de cobrir

A tentativa revertida ensinou **só o resolvedor**. A cura completa tem **três** consumidores:

1. **a PORTA de leitura** [`fk::local`] — `lrot`, senão `rot` (o segundo degrau é o que mantém
   uma corrente autorada à mão, ou por MCP, a posar);
2. **quem POSA** (`rig.ik_2bone`, `rig.fabrik`, `rig.rubber_hose`) escreve o solve no `lrot` —
   sem isto o `resolve` deitava-o fora e devolvia a pose anterior;
3. **quem LÊ uma pose** (os três `pose.rs`, cópias idênticas) lê pela mesma porta.

⚠️⚠️ **E uma quarta classe apareceu na segunda corrida, também muda:** as FIXTURAS do
`rig.skin_deformer`, cujo `chain()` devolve uma corrente **já resolvida** — elas rodavam a cadeia
escrevendo num campo que ninguém voltava a ler.

#### ⛔ O L-System NÃO tinha o defeito, e a premissa que o dizia era minha

O `source.lsystem` escreve `rot` **e** `wrot`, o que se lê igual ao rig — e ele tem um
interruptor, `orient_world`, com o MUNDO por omissão. *Uma coluna escrita em dois nomes não prova
um defeito; prova-se correndo o produto.* A mudança foi construída e **revertida** quando o gate
de controlo dele (`in_growth_mode_the_shape_faces_along_its_branch`) reprovou.

#### Os gates, e o que cada um nasceu a matar

- `o_esqueleto_entrega_o_angulo_de_mundo` (**produto**, no `rig.skeleton`) — ⚠️ ele existe porque
  o gate irmão no `fk.rs` **entra pelo `resolve` directo e fica ABAIXO do `build()`**: uma
  mutação no `build` sobrevivia-lhe.
- `o_rot_que_sai_e_o_angulo_de_mundo` (nas 6 cópias) — recta · curva · e concorda com o `wrot`.
- `escrever_no_rot_nao_posa_e_o_lrot_posa` — de **comportamento e não de texto**: *um `grep` por
  `with(ROT` fica verde no dia em que alguém escrever a coluna por uma variável*.
- `o_relocal_devolve_o_local_e_nao_o_mundo` (nos 3 `pose.rs`) — nasceu de uma mutação que
  sobreviveu aos vinte testes da crate.

**6 provas de mutação, todas a sangrar** — mais **uma NO-OP registada com a medição**: trocar
`LROT` por `ROT` no `build` não é observável, porque numa corrente acabada de construir não
existe `lrot` e os dois nomes levam ao mesmo sítio. *É a escada tolerante a funcionar, e está
escrita no script para ninguém a ler como buraco.*

#### ⏳ O que fica aberto

- ✅ **(2026-10-05, `line/motion-value`) A cópia acabou, e a família era maior.** O `fk.rs`/`trig.rs` (6 cópias)
  e o `pose.rs` (3) vivem numa crate-folha, `ph2d-rig-kinematics` (`0bb491ca3`); o predicado «quem é pai», que o
  `rig.bones` copiava letra por letra, é a porta `fk::pai`. E a MESMA doença em mais `38` crates — o sorteio
  `hash3`/`rand01`, o `cos_sin_cycles` e a acumulação das forças, cada cópia a dizer «o vocabulário partilhado é o
  comportamento» — vive agora na `ph2d-motion-kit` (`hash`/`trig` sem dependências, `forca` atrás da feature; a
  `ph2d-contact` e a `ph2d-bloom` usam-na). Recenso de ficheiros `.rs` byte-iguais `> 1 KB` entre crates: `5`
  grupos → `0`. **Portão:** `architecture_a_lei_partilhada_dos_nos_vive_numa_porta` (`ph2d-editor-core`) reprova
  um ficheiro byte-igual e uma definição de `hash3`/`cos_sin_cycles`/`sin_cycles` fora da porta — `3/3` mutações a
  sangrar (cópia byte-igual · `fn hash3` noutra crate · o falloff com o seu `mod trig`). Os espelhos WGSL continuam
  em cada nó, presos pelos gates de paridade GPU↔CPU deles.
- O `rig.skeleton` **não** ganhou o interruptor que o L-System tem (`orient_world`). Ninguém o
  pediu, e o dono viu o mundo como a resposta certa — *fica nomeado, não construído*.

---

## Bug #12 — o fio AGIA num ramo e não no outro, e o cartão mentia nos dois

**Estado:** ✅ **CURADO** em 2026-10-03 (aguarda smoke, cena `=128`) · handoff
[03/10 §6.3](../../handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md).

### Sintoma

Report do Enio no smoke de 03/10: *«ligar a saída de um Number à entrada Strength de um Vortex: o
fio aparece, mas o valor não tem efeito (o Vortex continua com o valor do cartão)»* — e, no mesmo
gesto, *«Number não aceita valores negativos»*.

### As medições, nesta ordem (e a conclusão intermédia que estava ERRADA)

| hipótese | medição | resultado |
|---|---|---|
| a CPU ignora o fio | marcha da bomba, `cartão 4 + fio 20` contra `cartão 20` | **iguais ao bit**; `fio −20` é o espelho |
| a placa ignora o fio | `cook_gpu`, cena simples (grelha + Vortex) | rota `fully-GPU`: `fio 20` = `cartão 20` a `1 ULP` |
| o fio ligado a MEIO, pelo gesto, não entra | 30 tiques, `subgraph::drive`, mais 30 | o campo muda (CPU) |
| ⛔ **a cena do DONO, na PLACA** | `=127` (formas + galáxia), `cook_gpu` | rota **HÍBRIDA**: Number `1`, `30` e sem fio dão o MESMO campo, **ao bit** |

⚠️ Depois das três primeiras escrevi *«o motor é inocente, quem mente é o cartão»* e curei o cartão.
**Estava meio certo.** A cena simples ia TODA à placa; a do dono (com formas) vai pelo ramo HÍBRIDO —
e só a sonda na cena dele, corrida quando a placa ficou livre, o mostrou. *Uma reprodução numa cena
mais simples que a do report pode estar a medir OUTRO ramo do mesmo produto.*

### O mecanismo — duas metades

1. **O ramo híbrido do `cook_gpu` não entregava os valores dirigidos.** O `cook_gpu` tem dois laços
   escritos à mão: o `FullyGpu` deriva os valores do fio POR TIQUE e chama `set_driven`; o `Hybrid`
   (prefixo na CPU, simulação como sufixo na placa) **nunca o chamava**. O plano encenava o Vortex com
   fio (os valores do PLANO estavam lá), e o cozedor resolvia o `strength` com o mapa vazio — caía no
   override do cartão. É literalmente *«o Vortex continua com o valor do cartão»*.
2. **O cartão desenhava o número MORTO.** O painel mostra numa row ligada o número que o fio põe
   (`params_stream::driven_value`); o cartão — que vai substituir o painel — desenhava o override, e o
   doc do `CardParam::value` justificava-o com um custo nunca medido (*«faria a row tremer»*).

E o «não aceita negativos»: um Number ligado veste a faixa do destino (`ParamUnit::FromWire`), e o
`Strength` das forças com SENTIDO era `0..40` — o arrasto parava no zero (a digitação passava).

### Os gates que estavam VERDES, e porquê

- `gpu_cpu_parity_driven` testa o `GpuCook` com `set_driven` À MÃO — abaixo da ponte; nenhum gate
  cozia um fio pelo ramo híbrido do `cook_gpu`.
- `the_card_shows_and_drags_the_same_numbers_the_panel_does` compara cartão e painel em todo o
  catálogo — sobre nós SOLTOS; um param ligado nunca entrou na população.

### A cura

- O ramo híbrido deriva os valores por tique ANTES de emprestar os fluxos da fronteira (o
  `valores_dirigidos` coze o condutor na bomba que o empréstimo segura) e um laço só cobre os dois
  sub-casos. Gates por ramo: `the_hybrid_route_reads_the_wire_like_the_cpu` (vermelho antes: *«mudou
  0»*) e `the_fully_gpu_route_reads_the_wire_like_the_cpu`; tirar o `set_driven` de cada ramo sangra.
- O cartão lê pela porta do painel; o doc do `CardParam::value` diz a lei verdadeira.
- `strength` a `±40` no Vortex, Attractor, Curl e Wind — negativo é o outro sentido **ao bit**.
  Densidade e arrasto ficam `≥ 0`: ali o negativo é outra física.
- ⛔ Achado vizinho, pela mutação: a caixa de número do cartão não aplicava faixa digitável nenhuma
  (`999999` num tecto de `100` entrava) e o doc dela dizia que sim.

### Lições generalizáveis

1. ⛔⛔ **Reproduza na cena do REPORT, na rota do PRODUTO.** A cena simplificada tomou outro ramo
   (`fully-GPU` contra `HIBRIDO`) e aprovou o motor; o defeito vivia no ramo que só a cena do dono
   toma. Leia a rota (`route_said`) de toda reprodução.
2. ⛔ **Dois laços escritos à mão que devem concordar esquecem a lei nova num deles** (a mesma família
   de `feedback_two_hand_written_loops_that_must_agree_need_one_door`) — e o gate tem de ser POR RAMO.
3. ⛔ **Um report de «não tem efeito» pode ter DUAS metades**: o efeito e o mostrador. Curar a que se
   achou primeiro e declarar o resto inocente foi o erro desta janela.

---

## Bug #13 — o grafo abria com a parte de cima escondida: o enquadramento media CARTÕES e desenhava PÍLULAS

**Estado:** ✅ **CURADO** em 2026-10-03 (`82f9623e2`) · handoff
[03/10 §6.4](../../handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md).

### Sintoma

Visto nas fotos da cena `=128` e confirmado pelo dono: ao abrir uma cena, os cartões de cima do grafo
ficam tapados — parece a tela de desenho a cobrir o painel.

### O que a medição disse (sonda `[SONDA-FIT]` no `paint`, `=128` a 1930×1040)

| quadro | retângulo do painel | `fitted` |
|---|---|---|
| 0 | `x 6 · y 566,5 · 1918 × 205,5` | `false` → enquadra AQUI |
| 1… | `x 312 · y 566,5 · 1310 × 205,5` | `true` |

⛔ **A tela NÃO tapa o painel** (a pintura recorta e pinta o fundo exactamente no retângulo). Eram dois
mecanismos:

1. **Vertical — o regime errado.** O piso do enquadramento é `0,35` e a pílula começa abaixo de
   `ZOOM_DA_CAPSULA ≈ 0,655`: num painel de `205` px quase todo grafo abre em PÍLULAS, mas o `fit`
   media os cartões ABERTOS (`card_h` com todas as rows). Centrava uma caixa muito mais alta do que o
   desenhado, e as pílulas ficavam no topo dela — cortadas.
2. **Horizontal — o painel do quadro 0.** O enquadramento corre uma vez, no quadro 0, com o painel a
   `1918` px; no quadro 1 os painéis laterais deixam-no a `1310` e os cartões ficam `304` px à direita
   (o gate reproduz os `304` ao píxel).

### A cura

- O `fit` mede pela geometria da PINTURA (`card_x_w_at`/`card_h_at`) no regime em que vai desenhar:
  primeiro abertos; se o zoom que sai é de pílula, mede pílulas (com teto logo abaixo do limiar, onde
  elas reabririam). No regime aberto é o de antes, ao bit.
- Uma vista AUTOMÁTICA que ninguém tocou re-enquadra quando o painel muda de tamanho
  (`refit_if_the_panel_moved_under_an_untouched_view`); a vista que o artista mexeu é dele.
- O que não cabe ao piso (o piso é o da LEITURA do nome na pílula) alinha pelo TOPO: o excesso vai
  para baixo, para onde se rola, em vez de cortar o início.

Gates pela costura real (`paint_fit_tests.rs`, o painel no retângulo MEDIDO): o que se desenha cabe e
fica ao centro · o que não cabe perde o fundo, nunca o topo · o painel que muda re-enquadra (e o
controlo: a vista mexida não muda) · um grafo pequeno continua com cartões abertos. **4 mutações a
sangrar** (medir pílulas como abertos — sobreviveu à 1.ª redacção do gate, que não exigia o centro ·
centrar sempre · não re-enquadrar · re-enquadrar a vista mexida).

### Lições generalizáveis

1. ⛔ **Quem mede para posicionar tem de medir o que se DESENHA** — um LOD que muda a forma do objecto
   muda a caixa, e uma medida do regime errado lê-se como «outra coisa a tapar».
2. ⛔ **Um cálculo feito UMA vez no 1.º quadro herda a geometria provisória do 1.º quadro.** Meça o
   retângulo nos dois primeiros quadros antes de culpar a geometria.

## Bug #14 — formas carimbadas a partir da SAÍDA de uma zona de simulação não aparecem no ecrã (✅ FECHADO 07/10 — NÃO era perda: o tamanho, e uma sonda cega)

### Sintoma
O dono pediu formas no smoke do ciclo 6 (*«vejo apenas gizmos»*). Na `=27` (a centelha que estoura:
`grid → scale → sim.zone(…lifetime → spawn…) → output`) pus `source.shape(estrela) → motion.duplicator`
entre a zona e a saída — e o app **não desenha nada**: nem estrela, nem a cruz de posição. Nas duas rotas
(`PH2D_GPU_COOK=0` também). O app regista `[formas] 3 copias de 1 geometrias pela PLACA` UMA vez.

### O que a medição disse
- **Pela porta do produto em teste** (`coze_o_quadro`, 180 quadros, placa e CPU): as 3 estrelas existem
  em todos os quadros, com posição (`±2,6`), tamanho (`0,22`) e cor certos, e as duas rotas concordam ao
  elemento (o gate esteve escrito: `the_fireworks_draw_the_same_stars_on_both_routes`).
- **Bissecção no app** (`--release`, foto): o MESMO carimbo ligado às SEMENTES (antes da zona) desenha as
  três estrelas; ligado à SAÍDA da zona não desenha nada.
- `PH2D_PAN_DIAG=1`: nos quadros seguintes ao primeiro o app não tem forma nenhuma para desenhar.
⇒ a diferença está entre o `coze_o_quadro` e o desenho (o que o `dispatch` faz à volta dele com uma zona
de simulação, ou o passe das formas) — **não** no cozimento.

### Estado (07/10, ciclo 13 — [doc 122 §1](../../122_ciclo_13_os_abertos_do_modulo.md))
✅ **Fechado, e a «bissecção» acima estava errada nas duas metades.** Pela porta do produto as duas rotas
entregam as estrelas em TODO quadro (buffer da placa lido palavra a palavra, `vector_instances` da CPU,
parado, aos saltos, com as tomadas dos gizmos); a camada da placa pinta-as; no app, instrumentado, o
passe decide e desenha em todo quadro. **A causa era o TAMANHO:** `0,07` em mundo é `~3 px` na câmera
do produto (`48 px` por unidade) — três pontos que a foto ampliada mostra —, e a cruz de posição some
quando o ponto tem forma. A cascata ainda sai do quadro aos `~3 s` (o `curl`), logo uma foto tardia
não tem nada com qualquer tamanho. **E o `PH2D_PAN_DIAG` era cego à rota do dispositivo**: lia as
`vector_instances`, vazias de propósito quando as formas vivem no buffer do cozimento, e imprimia
`de 0` — lido como «o app não tem forma». **Cura:** `ESTRELA = 0,22` (`~10 px`); a sonda diz a rota
da placa (`note_device`); gate `the_sparks_draw_legible_stars_on_both_routes` (vermelho a `0,07`);
censo da família: `1` cena com carimbo a jusante de uma zona (a `=27`).

### Lições
- ⛔ **Uma sonda que lê UMA rota imprime zero sobre a outra** — e um zero lê-se como ausência. Antes de
  acreditar numa ausência, pergunte à sonda POR QUE ROTA o quadro passou.
- ⛔ **«Não aparece» tem um suspeito barato antes do caro: o tamanho EM PIXELS na câmera do produto.**
  Um gate em unidades de mundo aprova uma forma de `3 px`; o gate desta cura mede pixels acesos.

## Bug #15 — o colisor no laço de um corpo mole não segura a distância mínima entre partículas (✅ FECHADO 08/10, ciclo 15 — e o «inchar» era o defeito)

### O que se vê
`motion.soft_body --pre--> motion.collide → soft_body.state` (o «raio de partícula» do doc 89, folha 03,
fixtura `soft_body_radius` do `ph2d-node-registry-init`) faz o corpo INCHAR (`1,92 → 2,76` de extensão,
`5 × 5`, `spacing 0,30`, `radius 0,28`), mas as partículas continuam a aproximar-se mais que o diâmetro:
o par mais próximo DESCE (`0,3000 → 0,2890`), e à SAÍDA do colisor o pior par NÃO vizinho chega a
`0,21` contra `0,56`.

### Porque só agora
O arnês avançava o tique ANTES de cozer e corria o colisor DUAS vezes por tique (um passo e um
re-cozimento com `dt = 0`); ali o par mais próximo SUBIA (`0,3448`) e o gate *«o raio afasta o par mais
próximo»* passava sobre um programa que o app não corre ([doc 123 §4](../../123_ciclo_14_os_oito_abertos.md)).

### Mecanismo (medido)
O colisor empurra TODOS os pares a menos de `2·radius`, os vizinhos de malha incluídos — e esses as
molas seguram a `spacing < 2·radius` de propósito. As `8` iterações do colisor não convergem contra
`25` partículas em que cada uma tem vizinhos sobrepostos; a pose desenhada é a do corpo depois de
mais um passo de molas.

### Estado
✅ **Fechado no ciclo 15** ([doc 124 §2](../../124_ciclo_15_o_tecto_dos_criadores_e_o_colisor_do_corpo_mole.md)).
Três peças, cada uma medida: (1) a LEI — os pares do mesmo corpo que o repouso já sobrepõe não
colidem (o filtro de auto-colisão do FleX; o corpo publica `rest` + `body`, o colisor lê-os na CPU e
no WGSL); (2) o contacto entra na VELOCIDADE do corpo (`sb_pos`/`sb_dt`, o `v = (x_final −
x_antes)/dt` do PBD); (3) o TEMPO — o corpo declara `substeps` ao relógio do grafo (a `1` sub-passo
o próprio corpo nu se atravessa nesta fixtura). A `4`: colisor `0,5600`, desenhado `0,5414`,
extensão `+0,2 %` contra o nu — o número do oráculo (Godot, MIT, a NOSSA fixtura: `0,5563`, sem
inchar).

### Lições
- ⛔ **O «inchar» que o ciclo 14 leu como *«o que um particle radius faz»* era a ASSINATURA do
  defeito.** O oráculo com «todos os pares colidem» deu exactamente o `2,76`; com a lei, o corpo
  não incha. *Uma régua escolhida porque «a outra não passa» mede o que o defeito produz.*
- ⛔ **E o gémeo tinha o MESMO defeito escondido:** o `rope_thickness` passava (`1,05`) com a
  corda ESTICADA `37 %` (`8,2` contra `6,0`). *Um gate de espessura sem régua de COMPRIMENTO
  aprova o colisor a desfazer a restrição.*
- ⚠️ **O critério escrito antes também errou, e a medição corrigiu-o:** o par ligado `≥ 0,27`
  (absoluto) foi refutado pelo corpo NU (`0,2511`) — a régua de uma propriedade do corpo é relativa
  ao corpo sem o nó sob teste.

## Bug #16 — a corda com colisor atravessa-se durante um chicote a 60 Hz (✅ FECHADO 08/10, ciclo 16 — o contacto mora DENTRO das iterações, e a âncora anda)

### O que se via
`motion.verlet_rope --pre--> motion.collide → rope.state` (a fixtura `rope_thickness`: `24` nós,
raio `0,5`, a cabeça a chicotear `±3,0` em `0,6 s`): na pose final os nós distantes ficavam a
`0,811·D`, mas DURANTE o chicote chegavam a `0,0497`. E o laço não era montável pelo artista: o
`motion.collide` está fora do catálogo.

### Mecanismo (medido, doc 125 §1)
Três defeitos com a mesma assinatura, um por baixo do outro:
1. **O contacto corria FORA das restrições** (uma vez por tique, depois da relaxação). Medido no MESMO
   processo: o contacto depois das iterações de cada sub-passo não segura nem a `8` sub-passos
   (`0,73`); DENTRO das iterações (o Vellum) segura a `1` (`0,915·D`) — mas estica `23 %` durante as
   dobras, porque as iterações não convergem contra o contacto.
2. **Faltava o passo seguro**: com os sub-passos a subir até cada nó andar no máximo MEIO alcance
   (a condição CFL), `0,966·D` e o comprimento `1,01×` o da corda nua, a `2,9` sub-passos por quadro.
3. **A âncora saltava o quadro inteiro no 1.º sub-passo** — numa corda FINA (raio `0,12`) a cabeça
   atravessava a conta vizinha (`0,1451` contra `0,24`), e o passo seguro realimentava-se do salto
   (pedia `76` sub-passos). Com a âncora a andar ao longo dos sub-passos: `0,2333`, `12` no máximo.

### A cura (o padrão-ouro, com o oráculo)
A corda ganhou a porta `shape` (`Shape → Verlet Rope`, as peças distribuídas como o `Cycle` do
Duplicator) e LÊ o colisor que o `Collide` do Shape declara, como o corpo mole (doc 124 §6): o
contacto do `ph2d_contact` corre a cada iteração, com o filtro do repouso (os vizinhos são da
corda); com ele, os sub-passos sobem pelo passo seguro e a âncora anda. Sem declaração, a corda de
sempre ao bit. O oráculo Godot estendido a uma corrente (`ferramentas/godot_oracle_corrente/`):
`0,9957·D` com a lei, e colidir os vizinhos INCHA a corrente `4×`. Pela porta do artista: `0,0138 →
0,9598`, comprimento `1,04×` (`the_rope_reads_its_pieces_collider_and_does_not_cross`). Cenas
`=130` (o gesto) e `=131` (o `Collide` depois da corda: o cartão aconselha).

### Lições
- ⛔ **Onde o contacto corre no passo decide tudo, e o número de sub-passos não compensa o sítio
  errado** — por sub-passo a `8` (`0,73`) perde para por iteração a `1` (`0,915`).
- ⛔ **Um alvo cinemático que salta o quadro inteiro no 1.º sub-passo anula os sub-passos** — e um
  passo seguro que lê o deslocamento realimenta-se do salto. A fixtura de raio GRANDE escondia-o
  (`0,9659` com o salto): foi a corda fina da CENA que o mostrou.
- ⛔ `c + (a − c)·1` não devolve `a` ao bit: o último sub-passo usa a âncora exacta (a 1.ª versão
  mudava a corda a `Substeps 1`, `0,9154 → 0,8947`).

## Bug #17 — o passe do fim inchava um corpo de peças no DESENHO (✅ FECHADO 08/10, ciclo 16 — achado pelo censo dos botões)

### O que se via
Um corpo mole de peças (`Grid → Duplicator ← Shape → Soft Body`) com o `Collide` do Shape E o do
Output ligados: o corpo segurava, e o que se DESENHAVA inchava — `3,0281 → 3,7561` de extensão.

### Mecanismo
O passe do fim (`ph2d_contact::passe`) lia a declaração do colisor e NÃO lia o `rest`/`body` que o
corpo publica: empurrava os vizinhos de malha (que o repouso já sobrepõe, e que o corpo segura) até
ao diâmetro — a lei do Bug #15, num quarto leitor que o ciclo 15 não alcançou.

### A cura
O mesmo filtro do `motion.collide` (`Pecas::com_ligados`) quando a corrente traz as duas colunas;
sem elas, byte-idêntico. `the_end_pass_does_not_swell_a_body_of_pieces` (`3,0281 → 3,0285`); a
mutação sem o filtro sangra-o (doc 125 §6, M5).

### Lição
⛔ **Uma lei do colisor tem de chegar a TODO leitor da declaração** — e o censo que a encontrou foi o
que liga cada botão sozinho e mede o DESENHO (doc 125 §2), não um gate do motor.



## Bug #18 — um toque que TREME virava arrasto no grafo, e um campo LEGÍVEL não respondia

**Estado:** ✅ **FECHADO** em 2026-10-10 — cura gateada e **smoke aprovado pelo Enio**
(*«smoke OK. Vamos seguir»*).

### O que se via
Report do dono (10/10, no smoke do ciclo 19): *«apertar botões em todo módulo desde os botões da
barra inferior até os campos dentro dos nós é impreciso e inconsistente»*.

### Mecanismo (três exploradores, o código lido; nenhuma geometria divergente)
- Pintor e alvo do clique saem das MESMAS funções (os chips da barra: o mesmo `chip`; as fileiras:
  `param_row_rect`/`param_track_rect`); a escala física → lógica é uma porta só. ⛔ NÃO era isso.
- **A superfície capturada do grafo não tinha FOLGA** (`pointer_move.rs`: todo `Move` marcava
  «moveu»; `pointer_up.rs`: «moveu» ⇒ `End`). Um pixel de tremor entre apertar e soltar: o chip da
  barra não disparava (só age no `Click`), o número ESCREVIA um valor (o `Update` escreve) e a caixa
  não abria, o toggle não trocava. A timeline e a tira do Flip TINHAM a folga (`4 px`, duas cópias
  da mesma lei) — o grafo era o único sem ela, e o comentário da timeline descrevia o defeito.
- **A faixa de zoom `0,655 ≤ z < 0,818`**: a fileira desenha-se inteira (número, barra, setas) e
  não regista alvo (a decisão MEDIDA de 19/09: a `14,4 px` uma fileira agarrável é uma armadilha no
  arrasto do cartão) — um toque nela caía no corpo e não fazia nada: um controlo que se vê e não
  responde.
- **Os gates de gesto empurravam `Click` sintéticos** e saltavam o despacho real — exactamente a
  metade onde o defeito morava.

### A cura
- `ph2d-editor-core/src/interaction/press_slop.rs`: **uma porta** (`PressSlop`, `PRESS_SLOP_PX` =
  `DRAG_THRESHOLD_PX` = 4 px lógicos) para as TRÊS superfícies — o grafo, a timeline e o Flip (as
  duas cópias saem). No grafo os `Update` só correm DEPOIS da folga: o tremor não escreve um valor
  nem arrasta um cartão; um arrasto real começa com o percurso inteiro (sem movimento perdido).
- **Ler é clicar; agarrar é arrastar** (`interact_param_row.rs`, `tap_on_a_readable_row`): na faixa
  em que a fileira se lê e não se agarra, o TOQUE que o corpo recebe sobre ela faz o que o toque
  faria nela; o arrasto continua a mover o cartão.
- O comentário do limiar dizia «px físicos» — são LÓGICOS (o despacho corre depois do `to_logical`).

### Gates (pelo caminho REAL: pintar → `Down/Move/Up` no despacho → pintar)
`interact_tremor_tests.rs`: um toque que treme 2 px num número ABRE a caixa e não escreve (controlo:
12 px escreve e não abre); um toggle troca com tremor `(0,0) (2,−2) (−3,3) (4,0)`; a `0,72` de zoom
um toque troca e um arrasto move o cartão. Mutações: sem folga (`> 0`) os três sangram; sem o toque
na faixa, o terceiro sangra. Timeline, Flip e Motion: as suites inteiras verdes.

### Lição
Uma lei copiada em duas superfícies e esquecida na terceira é o mesmo defeito à espera — *uma
porta*, não três cópias. E um gate que empurra o gesto sintético já assumiu a resposta do despacho.

