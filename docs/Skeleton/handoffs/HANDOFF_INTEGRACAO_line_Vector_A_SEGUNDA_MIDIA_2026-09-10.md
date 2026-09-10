# HANDOFF DE INTEGRAÇÃO — `line/Vector` · A SEGUNDA MÍDIA: UMA IMAGEM OBEDECE AO ESQUELETO

> **2026-09-10** · DIRETRIZ §1.5.9. A linha fecha aqui e **PARA** — não integra, não pusha
> ([`CLAUDE.md §0.7`](../../../CLAUDE.md)). Ordem de integração dada pelo Enio em 2026-09-10.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/Vector` |
| HEAD | `cb69e8256` (+ o commit de fecho, ver §10) |
| merge-base com `main` | `39d48cd76` |
| commits | **33** |
| ficheiros | **122** |
| `main` andou desde a merge-base? | **não — `0` commits** ⇒ o `--ff-only` da §1.5.3 tem por onde entrar |

---

## §2 — Foundational / partilhado tocado, e por quê

| ficheiro / área | o quê | aditivo? |
|---|---|---|
| **`crates/ph2d-poly2d/`** (crate NOVA, 10 ficheiros) | a geometria da 2.ª mídia — zero deps obrigatórias, `serde` opcional | sim |
| **`crates/ph2d-panel-skeleton/`** (crate NOVA, 7) | o painel próprio do esqueleto | sim |
| `crates/ph2d-affine/src/lib.rs` | `Xform::from_triangle` | sim (append-only) |
| `crates/ph2d-editor-core/src/ids/chrome/vector_bone.rs` | ids novos (§3) | sim |
| `crates/ph2d-editor-core/src/screens/hero/` (6) | o item *Window → Bones*, a barra, a pintura | mista |
| `crates/ph2d-editor-core/src/panel/{mod,rows}.rs` | o hospedeiro do painel novo | sim |
| `crates/ph2d-editor-core/tests/` (2 gates novos) | *«todo controlo do esqueleto declara de quem é o sujeito»* · *«o controlo pintado alcança um consumidor»* | sim |
| `crates/ph2d-i18n/src/vector.rs` | chaves novas (`panel.vector.bone.deform{,.fast,.smooth}`, e as do painel) | sim |
| `crates/ph2d-panel-vector/` (18) | a secção SKELETON **saiu** daqui para o painel próprio; ficaram o encaminhamento e os ids | mista |
| `crates/ph2d-panel-registry-init/` (2) | regista o painel novo | sim |
| `crates/ph2d-tool-vector/` (5) | `SkinDeform` + o espelho `VectorDrawConfig` | sim |
| `crates/ph2d-timeline/` (5) | os clips do *Smart Bone* (wave anterior desta linha) | sim |
| `crates/ph2d-component-desc/src/catalog/skeleton.rs` | descritores | sim |
| **`scripts/cargo-test-narrow.sh`** | ⚠️⚠️ **TECTO DE TEMPO + varredura de órfãos** — ver §9 | sim, mas **atravessa as 6 linhas** |
| `shells/desktop/src/` (46) | o gesto, o passe, o desenho, o smoke, o schema | mista |

**Crates NOVAS (2):** `ph2d-poly2d` · `ph2d-panel-skeleton`.
⚠️ O `collision-surface.sh` só acusa `ph2d-panel-skeleton` como pacote novo no `Cargo.lock` — a
`ph2d-poly2d` entra como aresta interna.

---

## §3 — Símbolos que podem COLIDIR

### `collision-surface.sh`, colado (⚠️ REFERÊNCIA, não evidência — re-rode antes de fundir)

```text
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 39d48cd76   ·   33 commit(s)   ·   122 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        127   (base: 123)
  ⚠   └ tripla do gate               (127, 13, 22)   (base: (123, 13, 22))
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  80   (base: 80)
    ph2d-script (espelho)                  80   (base: 80)
▸ CONTRATO CONGELADO (§6) — intocado nos dois
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 1 pacote novo: "ph2d-panel-skeleton"
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum ficheiro da linha passa do teto
```

⚠️⚠️ **`PROJECT_SCHEMA` sobe `+4` — CONTE O DELTA, nunca o literal.** Se outra linha também o
subiu, o valor certo não está em nenhum dos dois lados, e ⛔ **a colisão passa MUDA quando as duas
escrevem o mesmo número** (CLAUDE.md §5.0). A escada e a **tripla** vivem em ficheiros IRMÃOS
(`project_schema.rs` / `project_schema_tests.rs`) — um degrau escrito no ficheiro errado funde
limpo e evapora.

⭐ **Os `+4` são todos de waves ANTERIORES desta linha** (os *Smart Bones* e os limites de junta).
**As três waves da 2.ª mídia — a malha, a grelha e o `Smooth` — não mexem em schema nenhum**, e é
de propósito: a malha viaja nos **bytes opacos** do `SkinBind::source`, e o refinamento acontece no
QUADRO e não no documento.

---

## §4 — O que a linha entrega

### §4.1 — F6: a 2.ª mídia existe (uma IMAGEM obedece ao esqueleto)

Pesquisa em [`02_pesquisa_a_malha_sobre_a_imagem.md`](../02_pesquisa_a_malha_sobre_a_imagem.md).
⭐ **O campo inteiro deixou de mandar o artista construir a malha** (Spine · Live2D · OpenToonz ·
o *Puppet* do AE, que não tem interface de malha nenhuma). ⛔ E a porta MIT que temos é o
contra-exemplo: o Godot faz `Create Internal Vertex` um a um, **sem nenhum automático**.

| porta | pergunta |
|---|---|
| `ph2d_poly2d` | onde a tinta acaba · o orçamento · em que triângulos isso se divide |
| `Xform::from_triangle` | o afim que faz a imagem entortar |
| `skeleton_live::bind_image` + `tendons_for` | prender, pela **mesma** lei de tendões das formas |
| `skeleton_skin_image::{pixel_to_local, deform_field, draw_skinned_images}` | a régua da imagem, o campo, o desenho |
| `sim_extract::skinned_image` | a sprite original **sai do passe**, por facto derivado |

⭐⭐ **Zero capacidade nova de render**: o `push_clip` e o `draw_image_rgba_transformed` já existiam.

### §4.2 — F6-b: a malha vira uma GRELHA graduada pelas articulações

Report do dono: *«a malha criada automaticamente é de péssima qualidade … deveria ser um quadmesh
inteligente com maior densidade nas áreas das articulações»*.

| | contorno | grelha | grelha sem articulações |
|---|---:|---:|---:|
| vértices | `18` | `154` | `69` |
| **no MIOLO** | **`0`** | **`114`** | `48` |
| aspecto p50 | `17,42` | **`2,30`** | **`2,00`** |

⛔⛔ O `0` no miolo era a causa inteira. ⚠️ A barra `2` é o **CHÃO** (um quadrado partido em dois),
e a coluna sem articulações lê-o exactamente.

### §4.3 — F6-c/F6-d: o `Smooth`, e o tecto que estava na grandeza errada

Report: *«ao dobrar a articulação temos arestas retas na imagem … coloque como alternativa»*.

⭐⭐⭐ **A lei:** *a malha não é a deformação — ela é uma AMOSTRAGEM dela.* O campo
`Φ(p) = Σ wᵢ(p)·Mᵢ·p` está definido em **todo** ponto (os pesos são derivados). A aresta reta vem
do DESENHO: cada triângulo é **um afim**, a aproximação de 1.ª ordem de um campo curvo. O desvio
converge `O(h²)` (tabela no cabeçalho de [`refine.rs`](../../../crates/ph2d-poly2d/src/refine.rs)).

Depois veio o 2.º report — *«Smooth bugado quebrando a forma»* — e o diagnóstico está no §7.

---

## §5 — Gates novos

| onde | quantos | provados por mutação |
|---|---:|---|
| `ph2d-poly2d` (malha, grelha, refinamento, os dois tectos) | **22** | `5 + 7 + 2 + 2` mortas, com o controlo da árvore limpa |
| `ph2d-affine` (`from_triangle`) | 2 | — |
| `shells/desktop` (a 2.ª mídia, a costura das duas) | 8 | 3 |
| `ph2d-panel-skeleton/tests/seam.rs` | 3 novos (12 no total) | 1 |
| `ph2d-editor-core/tests/` | 2 | — |

⚠️⚠️ **E a prova de mutação expôs o limite dos gates de TEXTO:** pôr `if false &&` à frente do
*Bind* de imagens **SOBREVIVEU** — o texto continua lá. ⛔ A cura **não** é apertar a varredura
(seria uma corrida contra o próximo idioma que a desliga): *um gate de texto responde «isto está
escrito», nunca «isto corre»*, e a nota está no cabeçalho do ficheiro.

---

## §6 — Sete coisas que uma leitura rápida do diff entende ao contrário

1. **`mesh_of` / `contour` / `simplify` / `triangulate` da `ph2d-poly2d` NÃO estão no caminho do
   produto** — a grelha (`grid_mesh_of`) substituiu-os na F6-b. Eles ficam com gates e com os
   tectos curados (§7.5), mas **nenhum consumidor os chama hoje**. *Quem for apagá-los que o faça
   por decisão, não por descuido.*
2. **O `Smooth` NÃO é um segundo motor de deformação.** As duas alternativas desenham o **mesmo
   campo** — a diferença é só quantos pedaços o aproximam. Um segundo motor divergiria do primeiro
   na primeira ramificação, e a forma vectorial passaria a responder a uma lei diferente da imagem.
3. **`posed_local` mudou-se para o ARNÊS** (`skeleton_skin_image_tests.rs`) e isso **não** é código
   morto arrumado: no produto quem pergunta ao campo é o desenho, e ele passou a poder pedir-lhe
   **mais** pontos que os vértices. Deixar a porta antiga seria a segunda resposta à mesma pergunta.
4. **A fileira `Deform` não troca o modo da ferramenta**, ao contrário dos dois segmentos
   *Create/Transform* que estão logo acima dela no mesmo painel. A pergunta é de qualidade de
   desenho, não do que o arrasto faz.
5. **O `state::skinned()` e o `state::skinned_image()` NÃO são a mesma pergunta.** O primeiro é
   *«a SELECÇÃO é uma forma presa?»* e a shell responde-o varrendo `selected_paths()` — caminhos
   **vectoriais**. Uma imagem presa é uma **sprite** e nunca lá aparece. Ver §7.4.
6. **A `RefineOptions::max_pieces` NÃO é um `max_split` renomeado** — é outra grandeza, e a
   diferença é a causa do 2.º report (§7.1).
7. **O `guarda` do `triangulate` foi APAGADO, e isso é a cura e não uma regressão** — ele era
   inalcançável por construção (§7.5).

---

## §7 — As premissas que a medição derrubou

### §7.1 — ⛔⛔⛔ O tecto do `Smooth` estava na grandeza ERRADA (o 2.º report do dono)

*«Smooth bugado quebrando a forma»*, com foto: o braço quase **recto**, a silhueta com **degraus**.

**A malha está provadamente certa** (caminho exacto do produto):

| | |
|---|---|
| área de repouso | `30 720,00` refinada contra `30 720,00` guardada — ao cêntimo |
| triângulos que o desenho saltaria (`from_triangle → None`) | **`0`** |
| arestas com mais de dois donos | **`0`** |
| bordo | `288` = `48 × 6`, exactamente o que `k = 6` uniforme dá |

⇒ o que quebra está **a jusante**. ⭐⭐⭐ **E a experiência que o report correu sem querer é a
prova:** com o braço a **`2°`** de dobra o desvio já é `0,499 px`, logo o `k` saturava e a malha ia
a **`7 776` peças para desenhar a MESMA coisa que o `Fast` desenha em `216`**.

⇒ **o recurso é a CAMADA DE RECORTE do renderer.** Cada triângulo é um `push_clip` do Vello, que
dimensiona os buffers dele por heurística e **degrada em SILÊNCIO** quando estouram. Um tecto no
`k` é **quadrático** na contagem e a malha de partida pode ter qualquer tamanho ⇒ o tecto passou a
ser `max_pieces`, e o `k` sai de uma divisão.

⚠️ **O número do RENDERER não está medido** — o limite é de GPU e não há como o medir sem ecrã. O
intervalo conhecido é o do smoke (`216` desenha, `7 776` parte) e o valor de omissão (`1 024`) fica
do lado seguro dele. ⭐ `PH2D_SKIN_PIECES=<n>` fecha-o **numa corrida só**; `PH2D_BONE_LOG=1`
imprime a contagem para o report do dono a carregar.

### ⭐ A medição de CPU, tirada com a máquina CALMA (`load 3,25`)

⚠️ **Ela existe porque a máquina esteve a `load 30–90` durante quase toda a jornada** (outra linha),
e o `CLAUDE.md` §5.0 é explícito: *nenhuma leitura de relógio desta workstation vale nada acima de
`load ~5`*. Esta é a corrida que apanhou a janela.

| peças por imagem presa | deformar (CPU) | encodar (CPU) | soma |
|---:|---:|---:|---:|
| `216` (o `Fast`) | `0,004 ms` | `0,016 ms` | **`0,1 %`** de um quadro |
| `864` (o orçamento de omissão, `k = 2`) | `~0,18 ms` | `~0,08 ms` | **`~1,6 %`** |
| `3 456` | — | `0,304 ms` | — |
| `7 776` (o que o tecto do `k` dava) | `1,6`–`2,0 ms` | `0,735 ms` | **`10`–`16 %`** |

⇒ **`7 776` peças custam `10`–`16 %` de um quadro só de CPU, por imagem presa** — e a GPU vem por
cima, **não medida**. O orçamento de omissão põe isso em `~1,6 %`.

⛔ **A GPU continua por medir**, e é a metade que decide: cada peça é um `push_clip` do Vello, e o
que ele aguenta só o smoke diz.


### §7.2 — ⛔ O candidato CLÁSSICO do colapso foi construído, medido e REFUTADO

Misturar no **logaritmo do movimento rígido** (`se(2)` — o *dual quaternion skinning* do 2D)
**PIORA**: área no pior ponto `−0,129 → −0,280`, e a fracção da imagem **dobrada sobre si mesma**
vai de `2,52 %` a `4,47 %` numa dobra de `150°`. ⇒ *o colapso não vem de a mistura das matrizes não
ser uma rotação; vem do **gradiente dos pesos**, e o `log` não toca nesse termo.*

### §7.3 — ⛔⛔ A régua mentiu DUAS vezes antes de dizer a verdade

1. **Quina sobre o contorno TRAÇADO** media o artefacto dela própria — o rastreio de Moore devolve
   uma escada de pixels cujos degraus já viram `90°` no repouso. O campo **verdadeiro** lia
   `86°`–`129°`, tão «mau» como o produto.
2. **Quina sobre a silhueta analítica** degenera onde a silhueta raspa a esquina de um triângulo.

⇒ a que ficou é o **desvio em PIXELS** contra o campo verdadeiro: monótona, `O(h²)` limpa, e com
controlo próprio (zero pontos fora da malha).

### §7.4 — ⛔⛔⛔ A fileira `Deform` ia nascer VIVA E INALCANÇÁVEL

O portão óbvio era o `state::skinned()` — a pergunta da **selecção**, respondida varrendo caminhos
**vectoriais**. Uma imagem presa é uma **sprite**: nunca aparece naquela lista. A fileira teria sido
pintada em código e **nunca na tela**, com o gate de costura VERDE (ele arma o estado à mão).
⇒ *VIVO e ALCANÇÁVEL são duas perguntas, e a segunda quase não tem instrumento neste repo.*

### §7.5 — ⛔⛔ Os dois tectos da `ph2d-poly2d` eram MUDOS, e um deles era INALCANÇÁVEL

- **`contour::trace`** devolvia o anel **PARCIAL** ao bater no tecto; **`triangulate`** saía do
  laço com menos triângulos. Os dois **em silêncio**, e nenhum teste observava a guarda ⇒ a próxima
  entrada patológica entregava uma pele com buracos e **ninguém ficava a saber**. Hoje os dois
  **RECUSAM** (`None`), a recusa sobe até ao `bind_image` (que já diz *«não prendeu»*), e cada um
  tem gate com a entrada que o dispara.
- ⭐ **E o `guarda = n² + 8` do `triangulate` era INALCANÇÁVEL** — achado por uma mutação que
  sobreviveu: cada volta ou corta um vértice (no máximo `n − 3`) ou recusa. *Um tecto que não pode
  morder é pior que nenhum: faz quem lê pensar que o caso está tratado.* Foi **apagado**, e quem
  faz o laço terminar é a recusa, que agora tem gate.

### §7.6 — ⚠️ Três gates reprovaram sobre produto CORRECTO

| o gate dizia | o que a medição disse |
|---|---|
| *«o refinamento melhora ESTRITAMENTE a cada aperto»* | a correcção de um passo pode **passar** da tolerância pedida ⇒ a lei é **não-crescente** |
| *«o campo é perguntado uma vez por VÉRTICE»* | a conferência anda pela malha **refinada** (`3` por triângulo) ⇒ o tecto do mecanismo é `~8 V`, não `4 V` |
| *«a marcha põe um corte EM CIMA da articulação»* | a lei é *«a menos de um quarto do passo fino»* — forçar um segundo corte a `1 px` de um que existe produz a tira de `1 px` que a wave existe para apagar |

### §7.7 — ⚠️ Duas mutações SOBREVIVERAM e mudaram as fixturas

- **O caminho de omissão do refinamento:** o `grid_mesh_of` numera os vértices por **ordem de
  primeiro toque**, que é a ordem em que o refinamento os recriaria ⇒ sobre a malha da grelha o
  caminho longo devolve **por acaso** a mesma numeração. A cura é uma fixtura com os vértices
  **PERMUTADOS**.
- **A razão perguntas/vértices não vê a chave canónica:** sem ela as perguntas e os vértices sobem
  **juntos** (`1,89 → 1,92`). O denominador passou a ser as posições **distintas**, e o gate
  re-apontou-se ao **memo do índice**, cuja ausência move a razão para `2,76` — a barra `2,4` sai
  do vale entre as duas medições.
- **O tecto DENTRO da correcção** não era alcançado por nenhuma fixtura suave ⇒ precisou de um
  campo que **aliasa** (onda de período ~ a célula): o estimador lê `k = 5` e a conferência pede
  **`19`**.

---

## §8 — O que fica ABERTO

| item | estado |
|---|---|
| ⛔ **O custo de GPU do `Smooth`** | **não medido** — a CPU está (§7.1: `7 776` peças = `10`–`16 %` de um quadro; o orçamento de omissão = `~1,6 %`), a GPU não. É o que `PH2D_SKIN_PIECES` existe para fechar, e é por isso que o `Smooth` nasce **desligado** |
| ⛔⛔ **O mapa DOBRA sobre si mesmo em dobras fortes** | medido: `−0,129` de área a `60°/junta` (`0,22 %` da imagem) e `−1,017` a `150°` (`2,52 %`). O `Smooth` desenha o campo **com fidelidade, inclusive a dobra**. A causa é o gradiente dos pesos com `raio = comprimento do osso`; a família de curas (pesos mais apertados · *centers of rotation*) **não** foi medida |
| ⏳ um **buraco** dentro de uma forma não é traçado | a malha cobre-o |
| ⏳ a contagem de triângulos **sob cena cheia** | nunca medida |
| ⏳ `ph2d_flip_render::fill::triangulate` é um gémeo em `f32` | atrás de uma parede de `wgpu`; a medição que autoriza fundi-los está nomeada |
| ⏳ F4 *«undo tem poucos passos»* | **não reproduz** — 5 arrastos ⇒ 5 passos; falta a sequência exacta do dono |
| ⏳ o *pole target*, os *Smart Bones* avançados, a 3.ª mídia | fila do módulo |

---

## §9 — ⚠️⚠️ A PROPOSTA QUE ATRAVESSA AS SEIS LINHAS (decisão do Enio, não desta linha)

### O facto medido (2026-09-10)

Dois binários de teste de `ph2d-poly2d` desta worktree ficaram vivos **1h55m** a **299,7 % de CPU
cada** (~6 dos 32 núcleos, `19,7 %` da máquina), **~290 min de CPU queimados por processo**, com
`PPID = systemd --user`: **quem os lançou morreu, logo ninguém ia colher o resultado.**

⛔ **O defeito de código que os pendurou já estava curado** (`08d82693e`, o `backtrack` do rastreio
de Moore). O que ficou aberto não é o algoritmo — é que **nada os teria parado**.

### As duas metades, e só UMA está feita

| | estado |
|---|---|
| **`scripts/cargo-test-narrow.sh`** — tecto de tempo (`900 s`, `PH2D_TEST_TIMEOUT` afina, `0` desliga; exit **3** = pendurou) + varredura de órfãos | ✅ **FEITO nesta linha** (ordem do dono: *«é preciso corrigir os testes completamente»*) |
| **`.config/nextest.toml`** — `slow-timeout` GLOBAL | ⏳ **NÃO feito** — é o que o `ship.sh` e o CI leem, e o número precisa do censo abaixo |

⚠️⚠️ **O `slow-timeout` que existe vive dentro de um `[[profile.default.overrides]]` com
`filter = 'package(ph2d-asset-cooker)'`** (linhas 18-19 e 37). O próprio ficheiro escreve a lei —
*«a hang never returns to trigger a retry»* — e depois cerca-a com o nome de uma crate. ⇒ **nem o
nextest, nem o `ship.sh`, nem o CI têm guarda de pendura fora do asset-cooker.**

⚠️ **Prova de que foi `cargo test` e não nextest:** UM processo com TRÊS threads de teste. O nextest
corre um teste por processo.

### O censo (corrida de `cargo nextest run --workspace --no-fail-fast`, 2026-09-10)

| bucket | testes |
|---|---:|
| `> 60 s` | **43** |
| `> 120 s` | **11** |
| `> 180 s` · `> 240 s` · `> 300 s` · `> 360 s` · `> 420 s` | **1** (o mesmo) |

⚠️ **A corrida foi feita com a máquina a `load 30–90`** (outra linha corria `measure_brush_kernel` e
os gates do `quadfill` ao mesmo tempo) ⇒ **o censo SOBRE-reporta**, que é o lado seguro para
escolher um tecto.

### ⭐ O MÁXIMO LEGÍTIMO, medido SOZINHO (e ele NÃO é uma pendura)

O censo acusou `ph2d-quadchain::veto a_panic_downstream_does_not_take_down_the_caller` a passar dos
**420 s** e a subir, e a 1.ª leitura desta linha chamou-lhe *«candidato a pendura»*. ⚠️ **Errado, e
a medição sozinha desmente-o:**

```
test result: ok. 1 passed  ·  finished in 375.54s  ·  101% cpu  (carga 1,23)
```

⇒ ele **PASSA**, em `6 min 15 s`, mono-thread e limitado por CPU. *É o teste legítimo mais lento do
repo, e não um defeito.* A distinção importa porque é exactamente ela que o tecto tem de respeitar.

⇒ **um tecto de `60 s × 2` global MATARIA testes legítimos** — há `11` acima de `120 s` só em
`ph2d-field-eval`, e este a `375 s`. O número tem de ficar **acima do máximo legítimo medido** e
**muito abaixo de «uma pessoa repara»** (os órfãos do §9 queimaram `1h55m`). A recomendação desta
linha, com as duas âncoras ao lado:

```toml
[profile.default]
# ⛔ O tecto NÃO é de lentidão: é a conversão de uma PENDURA numa FALHA.
# Âncoras MEDIDAS (2026-09-10): o teste legítimo mais lento do repo demora 375 s sozinho
# (`ph2d-quadchain::veto`, e ele PASSA); os órfãos que motivaram isto queimaram 1h55m.
# 15 min ≈ 2,4× o máximo legítimo, com folga para a máquina carregada.
slow-timeout = { period = "60s", terminate-after = 15 }   # avisa a 60 s, mata a 15 min
```

⚠️ **E ele merece um `[[overrides]]` próprio, não uma excepção global mais larga:** um teste que
sozinho custa `6 min` é uma decisão de produto de outra linha, e alargar o tecto de TODOS por causa
dele esconderia a próxima pendura atrás dele.

⛔ **Isto NÃO foi shipado por esta linha** — ele atravessa as seis worktrees vivas e o CI, e a
decisão é do Enio.

### E a terceira metade, que nenhum doc cobria

⛔⛔ **Matar quem lançou NÃO mata o teste** (reparentamento). O `rm -rf target/*/incremental` do
fecho de linha **apaga ficheiros e não mata processo nenhum**.

⚠️⚠️ **E a 1.ª redacção da varredura MATOU 27 binários de uma corrida VIVA** — ela filtrava por
CAMINHO. *Um binário de teste desta árvore não é um órfão: um órfão é aquele cujo LANÇADOR morreu.*
Numa máquina onde seis linhas correm em paralelo, matar por caminho é sabotar o vizinho. ⇒ o
discriminador é o **PAI** (`PPID 1` ou `systemd`), e está no script.

⚠️ **E o padrão do `pkill -f` MATA-SE A SI MESMO** se não levar uma classe de caracteres: ele
aparece na linha de comando do próprio shell que o corre (medido, exit **144**, duas vezes).

---

## §10 — O que o INTEGRADOR faz

1. `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` **em cada worktree**,
   antes do primeiro grep (caminho ABSOLUTO do primário — uma worktree forkada antes do script não
   o tem).
2. ⚠️ **`PROJECT_SCHEMA` `+4`** — conte o delta contra o `main` do dia, e confira nos **três**
   sítios (a constante, a escada, a tripla do gate).
3. `scripts/foundational-integrate.sh` (DIRETRIZ §1.5.3) — o gate da árvore COMBINADA, que é o
   único que apanha os gates de arquitectura que vivem noutras crates.
4. ⚠️ **`scripts/cargo-test-narrow.sh` mudou** (§9) — é partilhado. As mudanças são defensivas e os
   valores de omissão generosos, mas **as outras cinco linhas herdam-nas**.
5. ⚠️ **Dois vermelhos do fecho desta linha são flakes de CARGA, confirmados**:
   `the_ui_clock_does_not_allocate_per_frame` (`ph2d-editor-core`, contador de alocações — verde
   sozinho, e o diff desta linha naquela crate são **14 linhas de constantes**) e um do shell que
   **não reproduziu em 4 corridas seguintes**, com o conjunto de reprovadas a mudar entre corridas
   do mesmo binário. Os dois batem a assinatura do `CLAUDE.md` §5.0.

---

## §11 — Smoke do dono (o que ele já correu, e o que falta)

| cena | estado |
|---|---|
| `PH2D_VEC_BONE_SMOKE=1` — o braço/tentáculo/folha + o **braço pintado** | ✅ aprovado até à F6-b (*«malha bem desenhada»*) |
| a fileira **Deform** (`Fast` × `Smooth`) | ⏳ **por smokar depois da cura do §7.1** |
| `PH2D_SKIN_PIECES=<n>` · `PH2D_BONE_LOG=1` | ⏳ é o par que fecha o número do orçamento |
