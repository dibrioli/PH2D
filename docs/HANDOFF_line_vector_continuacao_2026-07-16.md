# HANDOFF — linha `line/Vector`, continuação (2026-07-16)

**Para:** o próximo agente (contexto novo) **e** o agente integrador.
**Estado:** o **Blend Object vivo (ADR-0122) está COMPLETO** — Fases A, B, C1, C2a, C2b e D fechadas.
**+ os itens #2 e #1 da fila FECHARAM** — o buraco do compound path (`62c93fa7`, §8) e o
**MORPH VIVO** (o `t` keyável, §9).
A linha está parada, esperando ordem do Enio. **Não integre nem faça ship** (Modo L, CLAUDE.md §0.7).

> **Leia primeiro:** `CLAUDE.md` (inteiro, é curto) + `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`.
> Este handoff assume os dois. O ADR-0122 é a fonte da verdade do Blend; aqui está o que **não** cabe
> nele: identidade da linha, riscos de integração, a fila, e as minas que eu declaro.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch** | `line/Vector` (worktree `Worktrees/line-Vector/`) |
| **HEAD** | `e2cf1262` |
| **Base do fork** | `4d203d48` (merge-base com `main`) |
| **Commits** | 33 |
| **Contratos congelados encostados** | **NENHUM** (§4 abaixo) |

Os 6 commits desta sessão, do mais novo:

- `e2cf1262` — **MORPH VIVO**: a forma única entre duas, com o `t` keyável (fila #1 — §9)
- `ef41f447` — handoff: fila #2 fechada
- `62c93fa7` — **o blend PERDIA o buraco**: compound path sobrevive ao morph (fila #2 — §8)
- `92553b22` — os pontos LIVRES do spine acompanham quando o conjunto translada
- `22a368be` — **Fase D**: Expand / Release
- `f0706d0b` — Steps responde a qualquer objeto do blend + Shift soma pontos no modo Node

Os 26 anteriores construíram o motor de correspondência (as 4 correções do smoke + o giro do
quadrado) e as Fases A→C2b. O histórico completo está em `git log 4d203d48..HEAD`.

---

## §2 — O que o próximo precisa saber para não quebrar nada

Três ideias governam este código. Elas não são estilo — cada uma foi paga com um bug.

**1. Uma porta só produz um passo.** O `recook` (que desenha o overlay) e o `expand` (que assa os
paths reais) chamam a MESMA `blend_live::cook_links`. Uma 2ª porta faria as formas **saltarem** no
clique do Expand — justo na operação que promete entregar o que está na tela. O gate
`expand_materializes_exactly_what_the_overlay_drew` compara byte a byte. **Note o que ele NÃO prova:**
a correção da cozedura (isso é dos 22 gates do `recook`) — ele prova **acordo**, e uma mutação em
`cook_links` é invisível para ele, como deve ser.

**2. O spine é a geometria da entidade do blend; os passos NÃO estão na cena.** O `VecPath` que o
`VecPathRef` aponta é o **spine** (invisível: `recook` zera o traço todo frame). Os N passos são um
`Vec<VecPath>` de MUNDO que um passe de render desenha. É o que faz o blend ser **um objeto** e não N
formas — e é por isso que um passo não é pickável (o Illustrator faz igual).

**3. A linha é Node-only.** No modo Select o spine **não é selecionável nem tem gizmo** — quem se move
são as FORMAS, cada uma com o gizmo dela. Isso não é preferência: **um gizmo sobre geometria que se
move dobra** (a bbox segue as fontes e o gizmo soma por cima). Cinco tentativas de dar gizmo à linha
foram revertidas; o ADR-0122 lista as cinco e por que cada uma falhou. **Não as tente de novo.**

### As armadilhas que custaram caro (não repita)

- **"A forma andou?" ≠ "a âncora está fora do centro?"** — dão a mesma resposta quase sempre, mas a
  segunda também é SIM quando é a **âncora** que foi arrastada. `pin_spine_anchors` pergunta aos
  **centros entre frames** (por isso `BlendMemo.centers` existe). A versão que usava `centro − âncora`
  derrubou um gate existente que estava **certo**.
- **O gizmo de multi-seleção não registra hit no interior** (`paint_sprite_gizmo_keyed`, de propósito)
  — um commit inteiro foi construído sobre a premissa errada de que registrava.
- **`git checkout` para desfazer mutação apaga a feature** e o gate "passa". Use `cp`. Aconteceu nesta
  sessão; só não passou porque o gate novo pegou.
- **O `recook` roda com o spine JÁ assentado** — o `expand` conta com isso (lê o `spine_authored`
  persistido). Não reordene o frame sem ler `render_loop/mod.rs` §sync/upkeep/recook.

---

## §3 — Riscos de INTEGRAÇÃO (DIRETRIZ §1.5.9.2–3)

### 3.1 Foundational tocado, e por quê

| Arquivo | O quê | Forma |
|---|---|---|
| `crates/ph2d-ecs/src/vec_blend.rs` | **NOVO** — o componente `VecBlend` | Arquivo próprio (isolado por construção, §1.5.2.1) |
| `crates/ph2d-ecs/src/lib.rs` | `mod vec_blend;` + `pub use` | **Aditivo** |
| `crates/ph2d-ecs/src/scene/registry.rs` | `reg.register::<VecBlend>(…)` | **Aditivo** — mas vide 3.2 |
| `crates/ph2d-editor-core/src/ids/chrome/vector.rs` | 5 ids novos | **Aditivo** |
| `crates/ph2d-panel-vector/*` | a seção Blend | Da linha, mas é crate compartilhada |
| `shells/desktop/src/*` | 18 arquivos (o host do blend) | Vide 3.2 |
| `CLAUDE.md`, `.typos.toml` | doc + allowlist | **Ímã de conflito** |

### 3.2 O que o integrador tem de GREPAR (mesmo-símbolo, DIRETRIZ §1.5.5)

**⚠️ NÚMEROS QUE SOMAM — conte, não escolha.** Três gates afirmam a CONTAGEM de componentes
registrados. Se outra linha também registrou um componente, **o valor certo não está em nenhum dos
dois lados** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]):

| Arquivo | Eu mudei | Se outra linha somou, RECONTE |
|---|---|---|
| `ph2d-ecs/src/scene/registry.rs` | `reg.len()` **29 → 31** (`VecBlend` **e** `VecMorph`) | ✔ |
| `ph2d-render/src/registry.rs` | `reg.len()` **30 → 32** | ✔ |
| `ph2d-script/src/registry.rs` | `reg.len()` **30 → 32** | ✔ |

**⚠️ `.typos.toml` — allowlist duplicada MATA o gate no parse** (o TOML morre e nada é escaneado,
[[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]). Eu adicionei 3 chaves: `candidata`,
`regulares`, `fases`. Se outra linha adicionou alguma delas, **dedupe** — não aceite as duas.

**Ids novos** (hash de string, não número — o gate `node_id_collisions` os enumera e pega colisão):

```
VECTOR_BLEND_RESET_SPINE  = hash_node_id("vector.blend.reset_spine")
VECTOR_BLEND_EXPAND       = hash_node_id("vector.blend.expand")
VECTOR_BLEND_RELEASE      = hash_node_id("vector.blend.release")
VECTOR_BLEND_STACK_UP     = hash_node_id("vector.blend.stack_up")
VECTOR_MODE_PICKBLEND     = hash_node_id("vector.mode.pickblend")
VECTOR_MORPH_RUN          = hash_node_id("vector.morph.run")
VECTOR_MORPH_T            = hash_node_id("vector.morph.t")
VECTOR_MORPH_T_NUM        = hash_node_id("vector.morph.t_num")
```
Cada um tem uma linha em `ph2d-editor-core/tests/node_id_collisions.rs` **e** em
`ph2d-panel-vector/src/ids.rs` (re-export por nome). As três listas têm de andar juntas.

**Variant de enum apendado:** `ph2d_tool_vector::params::DrawMode::PickBlend` (o 8º pill). Append-only;
se outra linha apendou outro variant, os dois cabem — só confira a ordem.

**Campo mudou de TIPO:** `AppState.vec_restack`, de `Option<Vec<VecPathId>>` para
**`Vec<Vec<VecPathId>>`** (o Expand age sobre N blends, cada um pedindo a sua fatia contígua de z;
guardar só uma seria um corte silencioso). Quem **usa**: `app_state.rs` (declaração), `main.rs`
(init), `render_loop/mod.rs` (o dreno, virou laço), `build_smoke.rs` (3 atribuições, `.into_iter()
.collect()`). Quem só **cita em doc**: `blend_live_edit.rs`, `blend_live_expand_tests.rs`.

### 3.3 Contratos congelados (§1.5.9.4)

**Nenhum.** `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` intactos —
`DrawMode` vive em `params.rs`, não na trait. `architecture_vector_contract_surface` escaneia só
`ph2d-vector-doc`/`-traits`, que a linha não toca. Os dois gates passam.

### 3.4 O que SÓ o `ship.sh` pega (§1.5.9.5)

- **3 deps novas** (`cargo machete`): `ph2d-color` em `ph2d-vec-blend` (a cor OKLab) · **`ph2d-host`
  como dev-dep** em `ph2d-panel-vector` (o `PointerEvent` do seam dos botões) · **`ph2d-vec-boolean`
  como dev-dep** em `ph2d-vec-blend` (§8: um gate blenda a rosquinha REAL da booleana, em vez da
  minha ideia de rosquinha). As três são usadas; nenhuma cria ciclo.
- **`.typos.toml`** — vide 3.2.
- clippy latente / RUSTSEC / fmt pré-fork: nada conhecido, mas o gate de integração não roda.

> **Latentes que EU drenei nesta sessão** (os dois estavam vermelhos no HEAD, e só apareceram quando
> saí das crates óbvias): o gate HR-12 `every_widget_file_wires_a11y` não achava a delegação dita em
> `paint_blend.rs`, e havia um tofu `U+2192` numa mensagem de `assert` no `blend_live_tests.rs`.
> **Lição para quem fechar a próxima:** rodar só as crates que você tocou **não basta** — os
> arch-gates moram em `ph2d-editor-core` e varrem a árvore inteira. Rode `cargo nextest run
> --workspace --features panel-vector` antes de declarar verde.

---

## §4 — Estado dos gates e do SMOKE (§1.5.9.6)

**Workspace: 7039/7039 verdes** (`cargo nextest run --workspace --features panel-vector`, 102
skipped). Clippy limpo. LOC no teto (`blend_live.rs` 567/600 — **orce um split antes de somar campo**).

**⚠️ Honestidade sobre o que foi SMOKADO pelo Enio, e o que não:**

| Commit | Smoke |
|---|---|
| Fases A→C2b + o modelo de arrasto | ✅ **Smokado e aprovado** (o Enio iterou 5 vezes na interação) |
| `8ba7c889` (o fantasma da linha) | ✅ Aprovado |
| `f0706d0b` Steps por qualquer objeto | ✅ Aprovado |
| `f0706d0b` **Shift+clique em ponto** | ⚠️ **Não confirmado** — ele aprovou a mensagem, não relatou o clique |
| `22a368be` **Expand / Release** | ⚠️ **PENDENTE** — nenhuma evidência de que os botões foram clicados |
| `92553b22` **pontos livres** | ⚠️ **PENDENTE** — landou depois da última resposta dele |

**A costura do shell não é gateada** (por que: dirigir ponteiro em modo Node exige `AppGfx` = janela +
GPU, o mesmo bloqueio do harness headless do `project_save`). Isso vale para o ramo Shift+Down do
`input_dispatch.rs` e para os handlers `pending_expand_blend`/`pending_release_blend` do
`render_loop`. **A decisão mora em funções puras e gateadas; o roteamento depende do olho.**

**Cenas prontas (`feedback_ready_to_smoke_example` — não peça montagem ao Enio):**

```bash
cd Worktrees/line-Vector
PH2D_BLEND_SMOKE=1 cargo run -p ph2d-host-desktop --features panel-vector  # estrela → elipse
PH2D_BLEND_SMOKE=2 …  # cadeia de 3 (a pilha de z do Expand)
PH2D_BLEND_SMOKE=3 …  # spine CURVO (o Expand tem de entregar os passos NA CURVA)
PH2D_BLEND_SMOKE=4 …  # Pick Shapes
```

O que olhar no pendente: **=3** → Node, crie um ponto de dobra a mais; volte ao Select, selecione as
duas formas e arraste (a curva inteira acompanha); arraste UMA (a curva deforma entre elas); depois
Expand (os passos têm de nascer onde estavam — se algum saltar para a reta, virou 2ª porta).

---

## §5 — A FILA (a ordem é do Enio)

1. ~~**Morph vivo**~~ — **FECHADO** (§9). **Aberto por cima dele:** o morph só liga DOIS objetos
   (uma cadeia é o Blend); o `t` não tem alça no canvas (só o slider); e o Expand/Release do Blend
   não têm par aqui (um morph "assado" é `Convert to Curves`, que ainda não existe para ele).
2. ~~**Compound path perde o BURACO**~~ — **FECHADO** em `62c93fa7` (§8).
3. **Envelope / puppet warp.** — **a próxima.**
4. Do Illustrator, o que falta no Blend: **Replace Spine** (os passos seguem um caminho desenhado) e
   **Smooth Color** (o nº de passos sai do degradê).
5. Backlog antigo: **Live Path Effects como nós** (o multiplicador — a costura fonte≠cozido do
   ADR-0121 já é o pré-requisito) · tipos de quina (chamfer é quase de graça: reta em vez de arco) ·
   texto em caminho · trim path · repeater · largura variável · mais primitivas.

---

## §6 — Dívidas e minas que eu declaro

- **[DECLARADO, não é bug] O early-return de "ninguém andou" em `rigid_move` não tem gate.** Um
  mutante sobrevive a ele, e **está certo**: transladar por zero já é exato (`x + 0.0` não muda bit).
  É honestidade de contrato, não barreira. O comentário que dizia o contrário era falso e foi
  corrigido. **Não escreva um teste artificial para "cobrir" isso.**
- **[GAP] Cadeia de 3+ com pontos de dobra extras:** `anchor_source_pairs` liga só a 1ª e a última
  âncora quando `n_verts != live.len()`. As fontes do MEIO ficam sem âncora, e o `rigid_move` não vê o
  movimento delas. Com `n_verts == live.len()` (o caso normal) todas são ligadas.
- **[DÍVIDA, regressão minha, baixo impacto] Figura-8 deixa 1 âncora DUPLICADA** (o `cut` produz peça
  degenerada na auto-interseção). Em `main` não deixava. Geometria patológica; o conserto mexeria na
  costura de handles cruzados que a BUGS #17 acabou de estabilizar. **Decisão do Enio.** Visível a
  olho no modo Node.
- **[PRÉ-EXISTENTE] Precipício de escala:** `ARCLEN_EPS = 1e-11` é **absoluto** — numa forma muito
  grande ou muito pequena ele deixa de separar o que devia.
- **[LIMPEZA] O blend DESTRUTIVO ainda existe** (`shells/desktop/src/vec_blend.rs`, a `BlendSession`).
  O painel **não o usa** — só os smokes `PH2D_BUILD_SMOKE=7/8/9` (correspondência: star→circle etc.).
  Removê-lo exige repontar os smokes para o vivo.
- **[DEFERIDO] `spacing`** (Distance / SmoothColor) — não foi pedido; Distance exige comprimento de
  arco. Vide fila §5.4.
- **[SABIDO] Os botões Arrange de z-order estão MORTOS** — quem manda no z é o `RootOrder` na ÁRVORE,
  e eles chamam `VecScene::reorder_path`, que é a porta errada (a projeção do frame seguinte desfaz).
  Não é da minha linha; está aqui porque quem mexer em z vai tropeçar.

---

## §8 — O item #2 da fila: o buraco (`62c93fa7`)

Blendar uma rosquinha a virava um **disco**, sem aviso. O defeito tinha **três sítios
independentes**, cada um bastando sozinho — e é por isso que ele sobreviveu: consertar um só não
faz o buraco aparecer, então quem tentasse pela metade concluiria que a teoria estava errada.

| Sítio | Era | É |
|---|---|---|
| `ph2d-vec-blend/src/lib.rs` `Outline::of` | lia só `cooked.verts` | `compound::rings` — todos os contornos |
| idem, `path_from` | `..VecPath::default()` (subpaths vazio, `fill_rule` → `NonZero`) | emite subpaths + regra |
| `shells/desktop/src/blend_live.rs` `translate_verts` | laço só sobre `.verts` | `VecPath::for_each_vert_mut` |

O 3º é o mais instrutivo: a scene **já tinha** a porta certa (`for_each_vert_mut`, documentada como
*"base das transformações"*), e o `translate_verts` era uma 2ª porta que divergiu. Ela era
**inalcançável** — o motor nunca produzia um passo com buraco —, então dormiu até a rosquinha
chegar. [[feedback_two_doors_to_the_same_question_diverge]]

**As duas decisões de desenho** (as duas com mutação que mata):

- **O papel de um contorno é a profundidade de aninhamento dele** (0 = fora, 1 = buraco, 2 = ilha),
  medida por continência através da porta única `contains_point`. O par só sai da MESMA
  profundidade. Sem isso o contorno de fora casa com o buraco e a forma vira do avesso no meio.
- **Contorno sem par colapsa num PONTO, no centroide do lado OPOSTO** — o buraco viaja com a forma
  e fecha *dentro* dela. No centroide dele mesmo ficaria parado onde a forma **estava**, e
  encolheria saindo pela borda.

**A pesquisa (5 agentes) — ninguém resolveu isto**, e vale saber para não procurar de novo: o
flubber destrói o winding no `normalizeRing` (`if (area > 0) points.reverse()`) e ignora tudo menos
o contorno de fora (*"Deal with holes?"* aberto desde 2017); o GSAP ordena por tamanho **sem
sinal** — um contorno externo e um buraco de mesma magnitude são indistinguíveis — e a doc manda
*"split your path and morph each one"*; o Illustrator não consegue **formular** a pergunta (uma
âncora por *objeto*); o Inkscape descarta o resto em silêncio; o Blender recusa; e o **Sederberg &
Greenwood 1992** usa a palavra "hole" **uma vez**, no §6 *Future Work*, pondo exatamente este
problema. O modelo de profundidade tem validação independente no `correctContourDirection` do
**defcon** (fontes: paridade-como-continência, ilhas inclusive).

Do que a pesquisa mudou no código: o pareamento era **guloso** e virou o **`bestOrder` do flubber**
(branch-and-bound exato sobre `squaredDistance` de centroides). A degradação silenciosa dele
(> 8 peças ⇒ **ordem identidade**, sem contar a ninguém) **não** foi portada — acima de
`EXACT_MAX` o escape é guloso, que ao menos responde sobre a geometria.

**Teoria por trás do ponto de colapso** (Cohen-Or/Solomovici/Levin, TOG 17(2)): *"pure-warp blending
does not allow changes in the genus"* — um warp é homeomorfismo, e homeomorfismo preserva gênero.
Mantendo a representação por FRONTEIRA, a semente degenerada não é um hack: é **forçada**. A
alternativa é largar o BezPath por um level set.

**[DECLARADO] O quadrado do custo não tem gate.** Trocá-lo por `Σd` não derruba nada — os dois
escolhem igual em toda forma que sabemos desenhar, e só divergem numa configuração construída, onde
os dois têm defesa (o quadrado faz os buracos **cruzarem**; o linear deixa um parado e manda o outro
atravessar a forma). Sem verdade-fundamental publicada, a regra é portar a referência; um gate ali
afirmaria a minha intuição estética com cara de medição. **Não fabrique esse gate.**

**[LIMITAÇÃO conhecida, declarada] O centroide não é garantidamente interior.** Numa lua/C/U ele cai
FORA da forma — e aí o buraco órfão nasce fora dela e pinta uma mancha (que encolhe a zero em `t=1`,
onde um contorno degenerado não desenha nada). Bajaj/Coyle/Lin (GMIP 58(6)) são explícitos: o
ponto-cap só é correto quando a região é um **disco**; a resposta geral é o **eixo medial**. O
conserto barato, se aparecer no smoke: *pole of inaccessibility* em vez do centroide.

**Duas armadilhas que quase passaram, as duas de oráculo/fixture** (as duas documentadas nos gates):

- O probe caía **em cima** da fronteira onde a hipótese errada colapsa (`r=2,5`). Um ponto sobre a
  borda **não tem resposta**: a contagem de cruzamentos vira empate de `f64`. O gate ficava VERDE
  com o filtro de profundidade removido.
- O 1º fixture usava rosquinhas **idênticas e concêntricas**, apostando num empate de custo. Não
  havia empate: geometria idêntica dá centroide **exatamente** igual (`0.0`) e geometria diferente
  dá `1e-16` de ruído — então a distância, por acidente de `f64`, já fazia o trabalho do filtro.
  **Fixture simétrico não arma desempate.**
  [[feedback_identical_fixtures_hide_the_tiebreak_you_meant_to_test]]

**Gates:** 7 novos (`ph2d-vec-blend/src/tests_compound.rs` + `an_authored_spine_flows_the_hole_with_the_step`
no `blend_live_spine_tests.rs`), **5 mutações, 5 mortes**: só-o-primário · sem-o-filtro-de-papel ·
ponto-de-colapso-errado · `fill_rule`-sempre-`NonZero` · `translate_verts`-só-`verts`. Um gate usa a
rosquinha **real da booleana**. **`lib.rs` estourou o teto** (717/700) e foi DIVIDIDO: o `Outline`
(o primitivo de `matching`/`compound`/`spine`) saiu para `outline.rs` — 480 LOC.

**Pendente de smoke.** Nenhuma cena de smoke nova: a rosquinha se faz com **Subtract** de dois
círculos, e o Blend já tem as cenas dele. Sugestão: dois círculos → Subtract → repita → selecione as
duas rosquinhas → Blend. O buraco tem de existir em todo passo.

---

## §9 — O item #1: o MORPH VIVO (`e2cf1262`)

O `t` animável. O Blend mostra os N passos **de uma vez** (ilustração); o Morph mostra **UM**, e o
`t` dele é um número que a timeline keya. É a mesma relação, animada.

**O `Plan` é CACHEADO aqui, e no `blend_live` não é — e a diferença não é gosto.** No blend o `t`
não existe, então montar o plano por frame não custa nada percebido. No morph o `t` muda **todo
frame** (é essa a feature): sem cache, a busca de fase 256×256 rodaria por frame — os **5,9 ms** que
o `Plan` foi inventado para matar, agora dentro do orçamento de quadro, num caminho que roda
**enquanto a timeline toca**. A chave é a **geometria em MUNDO** das duas fontes (geometria *e*
pose), e ela é o próprio input do plano: não há chave paralela que possa divergir dele.

**Herdado do conector, e não é decoração:** o morph vive na **identidade** (a geometria é mundo; uma
pose por cima a deslocaria — e é isso que torna o gizmo inócuo sobre ele: move-se uma FONTE); e uma
fonte apagada **CONGELA** a forma em vez de a apagar.

**O canal da timeline:** `PropKind::Morph = 7`, apendado, **fora do `ALL`** (o `ALL` é a POSE do
autokey, e `t` não é pose de sprite nenhum), `as_sprite_transform() -> None`. O `write_prop` escreve
no `VecMorph.t` — `ph2d-timeline` já depende de `ph2d-ecs`, então **sem dep nova e sem feature
gate**. E o `sample_prop_value` **captura** o `t`, o que faz o **K funcionar de graça**: o artista
estaciona o slider onde a forma fica bem e aperta K.

Três respostas que o compilador cobrou, cada uma com razão própria:
- **`fit_channel` = bounded [0,1]**, igual ao `Opacity`, e não por acaso: os dois são uma fração com
  duas pontas duras. Uma cúbica de mínimos quadrados por um morph que assenta em B passa de `t=1`, o
  motor clampa lá, e o graph editor desenharia uma curva que deixa a forma **parada** — que lê como
  bug do easing, não do fit.
- **`algebra` = `Sum`**: o `t` é uma POSIÇÃO num caminho, não uma proporção. Por razão, duas faixas
  a 0,3 e 0,5 dariam 0,15 — *menos* progresso que qualquer uma delas.
- **`i18n_suffix` = `"morph"`** (+ `panel.timeline.prop.morph`, o rótulo do `+ Track`).

**A colisão de wire que eu tinha escalado NÃO existe:** verifiquei as 8 worktrees — `PropKind` está
intocado fora da `main` (termina em `TimeRemap = 6`) e não há um `= 7` em lugar nenhum. **O
integrador reconfirme**: se outra linha apendar, o discriminante é valor de wire em projeto SALVO, e
renumerar o código não renumera os arquivos do Enio.

### As minas que eu declaro

- **[ARMADILHA nova, gateada] O `filter` do `settle_origins` ENUMERA os seus leitores.** São quatro
  componentes de geometria derivada, e o 5º que esquecer a linha não vê erro de compilação — vê a
  forma nova a saltar, um frame depois, num sítio que não é o dela. Gate novo:
  `shells/desktop/tests/settle_skips_every_derived_geometry.rs`, com um **irmão que guarda a lista
  do gate contra o próprio drift**. [[feedback_a_condition_that_enumerates_its_readers_rots]]
- **A 1ª versão desse gate gritou LOBO**, e a lição vale: ela varria `*_live.rs` e cobrava que todo
  host estivesse na lista — mas o `flip_live.rs` é *"o alvo vivo"* do painel do Flip e não tem uma
  linha de geometria vetorial. **`live` ali é outra palavra.** O sinal certo não é o nome do
  arquivo: é **forçar a identidade**, que é a assinatura de *"a minha geometria é mundo"*. Um gate
  com falso positivo é um gate que alguém desliga. [[reference_topic_oracle_discipline]]
- **[LIMITE] O morph liga DOIS objetos, e recusa em voz alta fora disso.** Uma cadeia é o Blend.
  Morfar "as duas primeiras" de uma seleção de três seria escolher por ele, em silêncio.
- **[ABERTO] O `t` não tem alça no canvas** — só o slider e a timeline. Uma alça (arrastar a forma
  ao longo do caminho) é o gesto óbvio, e não existe.
- **[ABERTO] Não há Expand/Release para o morph.** O par do Blend não foi portado: "assar" um morph
  é um `Convert to Curves`, que ainda não existe para ele.

### Cena pronta (não peça montagem ao Enio)

```bash
cd Worktrees/line-Vector
PH2D_BUILD_SMOKE=10 cargo run -p ph2d-host-desktop --features panel-vector
```

Duas formas já **selecionadas**. Clique **Morph** → nasce UMA forma no meio do caminho (no meio, e
não em `t=0`: em 0 ela seria uma cópia exata de A, em cima de A, e o clique pareceria não fazer
nada). Arraste **Morph t** → ela caminha entre as duas, ao vivo. Mexa numa fonte → ela refaz-se.
**Para animar:** com o morph selecionado, `+ Track → Morph` na timeline, ponha o playhead, **K**.

---

## §7 — Resumo de fechamento (o formato da DIRETRIZ)

> Linha `Vector` pronta (HEAD `e2cf1262`, 33 commits sobre `4d203d48`). **ADR-0122 completo** (Blend
> Object vivo, Fases A→D) **+ os itens #2 e #1 da fila fechados**: o blend perdia o BURACO de um
> compound path (resultado errado em silêncio, pré-existente em `main`; §8) e o **MORPH VIVO** (o
> `t` keyável — `PropKind::Morph = 7` apendado, **valor de wire**; §9). Handoff de integração:
> foundational **aditivo** (`ph2d-ecs::VecBlend` em arquivo próprio + 2 linhas no
> `lib.rs`/`registry.rs`); **3 contagens de registry que SOMAM** (29→30 e 30→31 ×2 — reconte se outra
> linha registrou componente); **3 chaves novas no `.typos.toml`** (dedupe se colidirem); 5 ids novos
> (o gate de colisão os cobre); `DrawMode::PickBlend` apendado; `AppState.vec_restack` mudou de tipo
> (5 sítios). **Contrato congelado: nenhum.** Só o `ship.sh` pega: **3** deps novas (`ph2d-color`,
> `ph2d-host` dev-dep, **`ph2d-vec-boolean` dev-dep**) + typos. Workspace **7046/7046** verde.
> **Pendente de smoke: Expand/Release, os pontos livres, o Shift+clique em ponto, e a rosquinha
> (§8).** Aguardo ordem de integração.
