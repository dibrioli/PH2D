# HANDOFF DE INTEGRAÇÃO — `line/components` · **O ABANÃO DA VISTA** (suplente #25) · 2026-09-19

> *Isto explodiu, e a câmera tremeu.* O **último** dos cinco suplentes do
> [levantamento §7](../00_levantamento_componentes.md). Plano:
> [`21_plano_camera_shake.md`](../21_plano_camera_shake.md).

## §1 — Os contadores, como DELTA

⚠️ **Conte o DELTA contra a árvore em que vai aterrar, nunca o literal** — a coluna `base:` do
`collision-surface.sh` é o **merge-base** e está desactualizada por construção a partir da 2.ª fusão
de uma rodada.

| contador | delta | onde |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (`153 → 154` nesta árvore) | [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) + a **tripla** em `project_schema_tests.rs` |
| registo do `ph2d-ecs` | **+2** (`100 → 102`) | `scene/registry_camera.rs`, contado em `registry_tests.rs` |
| espelho `ph2d-render` | **+2** (`101 → 103`) | `ph2d-render/src/registry.rs` |
| espelho `ph2d-script` | **+2** (`101 → 103`) | `ph2d-script/src/registry.rs` |
| `LIVE_SECTIONS` | **+2** (`35 → 37`) | `ph2d-editor-core/src/ids/live_sections.rs` |
| `any_live_section` | **+2** (`29 → 31`) | `ph2d-panel-inspector/src/paint_frame.rs` |
| `ComponentEdit` | **+2** variantes, append-only | `action_bus_component.rs` |
| contrato congelado (§6) | ⛔ **zero** | — |
| portas novas no `AppHost` | ⛔ **zero** | — |
| verbos novos (`SignalVerb`) | ⛔ **zero** | ver §3 |

⛔⛔ **DOIS componentes e UM degrau de schema, porque o terceiro tipo não se regista:** o
`CameraShakeRuntime` não deriva `Serialize`, logo a linha do registo **nem compila** — o precedente
do `TimerRuntime`. A entrada dele é no `rewind_runtime`, que é a outra metade da mesma lei.

## §2 — O que a §5.0 mediu, antes da primeira linha

Sonda: `mede_o_que_a_composicao_ja_da_ao_abanao` (`ph2d-app-components`, `--ignored`, imprime).

| a pergunta | a resposta MEDIDA | o que ela decidiu |
|---|---|---|
| a tabela de acções sabe dizer «treme»? | ⛔ **não** — `9` verbos, nenhum da câmera | um sinal chega a toda a parte e não sabe pedir um abanão |
| um TWEEN sobre a própria câmera exprime-o? | ⛔⛔ **não, e é o achado:** ele **escreveu** (`Transform.x = 2,0000`) e o **centro da vista ficou em `[0,0]`** | *a vista vem do `CameraRuntime`, que não é um `Transform`* |
| um sinal sabe QUEM gritou? | ⭐ **sim** em `3` das `5` origens amostradas | ⇒ a distância é uma **subtracção**, não um canal novo |
| há gerador determinista? | ⭐ **sim** — *splitmix64*, **PRIVADO** da fábrica | ⇒ ele SAI para a folha (2.º consumidor) |
| rebobinar já renasce? | ⭐ **sim** — o censo existe | ⇒ um vivo NOVO **tem** de entrar nele |

## §3 — As decisões, e o que cada uma custou

### ⭐⭐⭐ A lei do abanão JÁ ESTAVA ESCRITA na shell — faltava o número

O `GameCamera::offset` entra na **VISTA** e não no estado vivo, *«senão ele realimentaria a zona
morta e a câmera afastar-se-ia do alvo um pouco mais a cada quadro»* — escrito no `camera_2d.rs`
desde a wave do #7. O abanão é o **segundo somando no mesmo sítio**.

⚠️ **DEPOIS dos limites, e é decisão declarada:** antes deles a cerca **comeria** o abanão na borda
do nível, e o artista leria *«o abanão parou de funcionar aqui»*. Gate
`na_borda_do_nivel_a_vista_ainda_treme`, com a câmera levada CONTRA a cerca primeiro (`r.limited`),
senão a régua mede uma câmera livre e passa por vácuo.

### ⭐⭐ O abanão é facto do TEMPO, nunca da taxa de quadros

O ruído é de **VALOR** (grelha inteira + `3t² − 2t³`), logo contínuo. ⚠️ A régua é a continuidade,
com o `baralha` como **controlo negativo** — e ela **reprovou-me na prova de mutação**: a 1.ª
redacção varria `t ∈ [0, 1]`, que é **uma** célula da grelha, e ali um ruído reduzido a degrau é
constante. *Uma régua de continuidade que não cruza uma fronteira da grelha mede um planalto.*

### ⛔ E NÃO há um verbo `Shake` na tabela de acções

Seria a **segunda** maneira de pedir a mesma coisa — e a pior, porque um verbo age sobre um **alvo**
e não tem de onde tirar a **distância**. O emissor é a porta.

### ⛔ O abanão ANGULAR fica FORA, com o número

A `CameraView` tem **três** campos (`center`, `height_world`, `cull_mask`) e **nenhum é um ângulo**:
a vista deste app não roda, e pô-la a rodar é uma wave de renderer. *Um limite legítimo diz de que
recurso ele é* (§0.0) — este é da superfície da vista, e está contado.

### ⚠️ O tecto do EXPOENTE não nomeia recurso nenhum, e dizê-lo é a única forma honesta

Nada se parte com `n = 9`: é **faixa de PRODUTO**, e a fonte são as referências (Eiserloh: *«trauma
ao quadrado ou ao cubo»*; o `bevy_trauma_shake` ship `2`). ⭐ **O que ele COMPRA está medido:**
`1 − (3/4)^(n+1)` do movimento cai no primeiro quarto ⇒ `44 %` · `58 %` · `68 %`. Subir é decisão do
dono, com a tabela ao lado.

⭐ E a faixa é **erro de compilação** (`const _: () = assert!(…)` na própria lei) e não um gate —
um `assert!` de teste sobre duas constantes é **dobrado pelo compilador**, e o clippy di-lo em voz
alta. É a cura que a ordem do dono sobre o espaçamento do pincel afiado recebeu em 16/09.

## §4 — Sete coisas que uma leitura rápida do diff entende ao contrário

1. **O `ph2d-shake` não tem `libm` e isso não é esquecimento** — ela é livre de transcendentais por
   desenho (HR-5): o ruído e a atenuação interpolam com `3t² − 2t³`, logo a resposta é a mesma em
   toda máquina sem o pin que as leis de física declaram.
2. **`potencia` não usa `f32::powi`, e é deliberado** — ele baixa para `llvm.powi`, cuja associação
   de multiplicações não é pinada entre plataformas, e esta lei entra numa vista que o
   `physics_ecs_c9` fotografa na matriz de três OS.
3. **A ponte NÃO passa pelo ledger do `preview_drive`**, e não é um esquecimento: ela escreve um
   `CameraShakeRuntime` que não é registado e **devolve** o offset. *Nada do que ela faz entra no
   ficheiro, no undo ou na captura.*
4. **Ela corre ANTES do `camera_2d::update` e isso é load-bearing nas duas metades** — a vista deste
   quadro tem de mostrar o trauma deste quadro, e correr depois dos relógios dá **latência ZERO ao
   contacto da física**. Um sinal da TABELA chega no quadro seguinte, e isso é a janela de graça de
   um quadro que o outbox dá a **todo** consumidor.
5. **O `abanao` é PARÂMETRO do `update` e não uma leitura de componente lá dentro** — assim
   esquecê-lo é **erro de compilação**, e a ordem *«a ponte antes da vista»* deixa de ser uma nota
   que alguém tem de se lembrar de honrar.
6. **A distância sai de QUEM GRITOU quando o sinal traz o sujeito**, e da pose do EMISSOR quando não
   traz — não é a mesma coisa: uma bomba que ouve o estrondo de outra abana a partir de **onde a
   outra está** (gate próprio).
7. **O `relógio do abanão` NÃO se repõe quando o trauma chega a zero** — é isso que faz dois
   estrondos seguidos não terem a mesma cara. Quem o repõe é o `rewind_runtime`.

## §5 — Cinco premissas minhas que a medição derrubou

1. *«a mutação `>=` → `>` na cerca do raio externo prova a lei»* — **NO-OP**: a smoothstep dá
   exactamente `0` em `u = 1`. A cerca é load-bearing para `d > fora` (onde `1 − u²(3−2u)` dispara),
   e é esse regime que a mutação tem de tocar.
2. *«a régua da continuidade cobre o degrau»* — ela varria **uma** célula (§3).
3. *«dois estrondos seguidos comparam-se quadro a quadro»* — ela contava as **caudas de zeros**
   (`13` de `30` idênticos sobre produto CERTO): a `forca` de fábrica dá `18` quadros de abanão.
4. *«o gate do pátio cobre a vista»* — ele media a **JANELA**, e o dono vê a **BANDA** que sobra com
   a timeline aberta (~metade). A foto mostrou **uma** fileira de postes.
5. *«a cena tem as três peças, logo está certa»* — a **bomba estava fora do ecrã**, com os dez gates
   verdes. *Eles perguntam «a cena tem as peças?», e as peças estavam lá.*

## §6 — O que o PORTÃO apanhou (7 vermelhos)

| vermelho | a cura |
|---|---|
| `the_shell_only_shrinks` (`197 268` de `196 990`) | ⭐ **MOVER**, nunca subir — ver §7 |
| `panel_files_under_loc_cap` (`paint_optional_top20` `632`, `event.rs` `605`) | **CORTE** por responsabilidade (os instantâneos · os três cliques de COR) |
| `workspace_src_files_under_loc_cap` (`registry.rs` `702`) | **CORTE**: a família da CÂMERA para `registry_camera.rs` — ⭐ **exactamente** a fronteira do catálogo de descritores |
| `no_magic_numeric` (`0.01` · `0.1` · `0.05`) | `// LITERAL-PX-OK:` com a unidade em cada linha |
| `hr12_widgets_a11y` (2 ficheiros novos) | isenção **MEDIDA**: `0` ocorrências de `NodeId` / `hit_index.` / `register(` |
| `every_registered_component_has_a_descriptor` | os dois descritores em `catalog/camera.rs` — ⚠️ e a **busca binária** reprovou a 1.ª posição |
| `the_cost_of_sampling_a_path_is_flat_in_its_anchors` | ⚠️ **flake de FAN-OUT já NOMEADA** no `CLAUDE.md` §5.0; zero linhas do diff naquela crate, verde na corrida seguinte |

⛔ **Nenhuma entrada nova no `FILE_OVERAGE_OK` nem no `FN_OVERAGE_OK`** — as duas listas continuam
VAZIAS.

## §7 — ⭐⭐⭐ A fase da CÂMERA saiu da shell, e o candidato veio de uma MEDIÇÃO

A catraca pôs a shell `278` linhas acima do tecto. A cura que o `CLAUDE.md` §2 prescreve é **mover
para a crate da família — nunca subir o número**.

⭐ **O candidato não foi escolhido por tamanho:** a `render_loop/camera_2d.rs` (a câmera de jogo, o
TOP-20 #7) **nunca tocou na `App`** — ela recebe um `&mut SimWorld` e devolve o que a vista devia
ser. *O que sai são os CORPOS; o que decide a ordem do quadro fica* (a lei da Fase C da física).

* **873 LOC** (`camera_2d.rs` + `camera_2d_tests.rs`) → `ph2d_app_components::camera_2d`;
* o que fica é a `fase_game_camera`, que é composição (lê a superfície, escolhe se toma a vista,
  imprime o relatório);
* ⭐ **Prova de que nada evaporou:** `cargo nextest list` conta **18 testes com `camera_2d` antes** e
  **18 depois**. *Um ficheiro `.rs` que nenhum `mod` declara não é compilado, e `check`, `clippy` e
  as suítes ficam VERDES com os testes ausentes* (a lei da Fase D do vector).

## §8 — O que só a árvore COMBINADA pode reprovar

1. **Os censos de TEXTO (HR-15)** — as `~30` chaves novas de `ph2d-i18n` e os pintores que as leem
   passam sozinhos; um literal de UI de outra linha reprova na soma. `bash scripts/censos-da-arvore-combinada.sh`.
2. **Os TECTOS de LOC** — os três que esta linha cortou ficam com folga (`555` / `523` / `685`), mas
   eles **somam entre linhas**: outra linha a tocar no `event.rs` ou no `registry.rs` pode reabri-los.
3. **A catraca da SHELL** — ela desce `873 − 278 ≈ 595` linhas abaixo do tecto com esta linha, o que
   dá folga à rodada; ⚠️ mas a folga é da SOMA, e quem a gastar não o vê sozinho.
4. **Os três contadores de registo** — `+2` cada, e os espelhos contam o `ph2d-ecs` mais um. *Conte
   o DELTA.*

## §9 — Aberto, e de quem é cada item

| item | de quem |
|---|---|
| abanão ANGULAR (o *roll* do Cinemachine) | **produto** — pede um campo novo na `CameraView`, que é wave de renderer |
| perfis prontos (*recoil* · *bump* · *explosion*) | **produto** — açúcar sobre os cinco números; o tween (#22) mostrou que um preset se acrescenta sem tocar no motor |
| o abanão sentir a DIRECÇÃO do impulso | **produto** — é outro modelo (o *Impulse* vectorial), e pede um segundo corpus |
| `SignalFrom::Tagged` no emissor | **sem consumidor** (a mesma nota do #24) |
| o tecto de **emissores por quadro** | não varrido — a ponte é `O(sinais × fontes)` e nenhuma cena o aproxima |

## §10 — Como correr

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_SHAKE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

Provas: `bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_shake_2026-09-19.sh`
→ **19 de 19 sangram**.
Portão: `bash scripts/ph2d-run.sh bash scripts/nextest-impacted.sh` → **15 377 verdes**.

---

## §11 — ⛔⛔⛔ O SMOKE do dono devolveu DOIS defeitos, e o segundo saiu da minha família

> *«vc esqueceu de colocar física no jogador. quando eu coloquei a física travou, não consigo dar
> play»* — 2026-09-19.

### §11.1 — O herói não tinha CORPO, e a seta movia `0,0000 m`

A ponte do mover ([`drive_topdown`](../../../crates/ph2d-physics-ecs/src/bridge/topdown.rs)) varre
`self.bodies` — **quem não tem corpo nunca entra no laço**. A cena dava ao herói o `TopDownPlayer` e
mais nada, logo o **passo (2) do roteiro era impossível**. Medido pela ponte real: `60` tiques com a
seta segurada ⇒ `dx = 0`, `dy = 0`, de um orçamento de `6,0000 m`.

⚠️⚠️ **As TRÊS cenas irmãs que carregam este componente dão-lhe as três peças** (`topdown_smoke` ·
`trigger_smoke` · `dano_smoke`, as três com `RigidBody { Kinematic }` + `Collider { Ball }`); esta
dava uma. *Não é uma lei que faltava — é uma linha que eu não copiei.*

⛔⛔ **E nenhum dos DOZE gates da cena o via, com forma:** eles perguntam *«a entidade tem o
COMPONENTE?»* — o `a_camera_segue_o_heroi_que_o_dono_conduz` pergunta exactamente isso — e o
consumidor pergunta *«a entidade tem um CORPO?»*. **A lente do gate é mais larga que a do
consumidor**, que é a família nomeada no `CLAUDE.md` §5.0. ⇒ o gate novo mede o **BARRO**:
`o_heroi_anda_quando_o_dono_carrega_na_seta` conduz a cena montada pela ponte REAL e lê o
`Transform`, com as duas metades (*ele anda* · *e não CAI*).

### §11.2 — «travou» é a gravidade, e o número está medido

Com um corpo **`Dynamic`** debaixo do mover: a pose passa a ser do SOLVER
([`bridge::pose_owner`](../../../crates/ph2d-physics-ecs/src/bridge/pose_owner.rs)), o mover fica
**inerte**, e a gravidade leva o objecto — medido nesta cena, **`y = −492 m` ao fim de dez
segundos**, com a câmera a segui-lo. O pátio cobre `y ∈ [−12, 12]` ⇒ **ao fim de ~1,6 s não há um
poste no ecrã**, e nada do que o dono carregue traz a cena de volta. *É isso que ele leu como
«travou, não consigo dar play»* — e não um bloqueio: o app está a desenhar um fundo vazio.

⚠️ O Inspector **já dizia o porquê em vermelho** (`the body must be kinematic — a dynamic body
belongs to the solver`), e ele nunca o viu: o roteiro manda escolher a **BOMBA**.

### §11.3 — ⭐⭐⭐ E a cura da 2.ª metade é de PRODUTO, na família da física

O gesto que ele fez está **certo**, e o caminho de OMISSÃO da paleta entregava um componente que não
funciona: o `RigidBody::default()` é `Dynamic`, e os dois movers **requerem** `RigidBody` no
catálogo ⇒ escolher *Top-Down Player* na paleta anexa o corpo em cascata, dinâmico.

⇒ semente [`seed_kinematic_controller_body`](../../../crates/ph2d-app-physics/src/physics_seed.rs):
**um corpo debaixo de um controlador cinemático nasce `Kinematic`**.

- ⭐ **A porta é `ph2d_physics_ecs::controlador_cinematico`** — a lista que o `pose_owner` já
  consultava, **aberta** porque a semente faz a mesma pergunta. *Escrita duas vezes divergiria no dia
  do quarto controlador, e o modo de falha é o caro:* a lista do seed a dizer *«este não é um deles»*
  sobre um componente que a ponte trata como sendo.
- ⛔ **O `PlatformPlayer` NÃO entra**, e a ausência é medida: ele funciona nos dois corpos (a perna
  de MOLA em `Dynamic` — o valor de fábrica do `PlayerMode` — e a de POUSO em `Kinematic`), logo
  responder «sim» por ele escolheria por um artista que tem duas leis legítimas.
- ⚠️ **TRÊS nomes na tabela porque há DUAS ordens de chegada**, e a segunda não é opcional: a cascata
  corre **antes** do `attach_one` do dependente (é o que deixa o seed do `PlatformPlayer` medir o
  collider), logo no instante em que o corpo nasce a entidade ainda **não** tem o controlador — uma
  semente só no `RigidBody` leria `false`. *As duas entradas são a MESMA função: uma lei, uma porta,
  três nomes.*
- ⚠️ **Conservadora e idempotente:** só morde no ponto **neutro**, logo nunca rebaixa um `Static`
  autorado.

**4 gates** em [`component_seed_seam_tests`](../../../shells/desktop/src/component_seed_seam_tests.rs),
e **dois deles são CONTROLO** — um corpo num objecto comum continua `Dynamic`, e um autorado fica
intocado. *Sem eles a cura leria como «todo corpo nasce cinemático», que é outro produto.*

### §11.4 — ⚠️ PARA QUEM FUNDE: esta cura sai da `line/components`

| ficheiro | crate | o que muda |
|---|---|---|
| `bridge/pose_owner.rs` · `lib.rs` | **`ph2d-physics-ecs`** (foundational) | a lista vira **porta pública** `controlador_cinematico`; o `pose_owner` passa a lê-la. **Zero** mudança de comportamento. |
| `physics_seed.rs` | **`ph2d-app-physics`** (outra família) | função nova + **três** entradas apendadas na `COMPONENT_SEEDS`. |
| `component_seed_seam_tests.rs` | shell | `+4` gates (`+120` linhas — a catraca tem `594` de folga). |

⛔ **Zero contadores partilhados, zero contrato, zero ADR, zero porta nova no `AppHost`.**
⚠️ **Mudança de comportamento observável e declarada:** anexar *Physics Body* / *Top-Down Player* /
*Projectile Motion* passa a entregar um corpo **cinemático**. O censo
`attaching_is_inert_for_everything_that_does_not_seed` passa a isentar mais três nomes — é o que a
tabela `SEEDS` significa, e o piso de população (`>= 20`) mantém-no honesto.

### §11.5 — ⚠️ E o arnês de mutação do #25 apontava para um ficheiro que se mudou

O `VISTA=` do [`mutacao_shake_2026-09-19.sh`](../ferramentas/mutacao_shake_2026-09-19.sh) apontava
para `shells/desktop/src/render_loop/camera_2d.rs`, que **deixou de existir na mesma jornada** (a §7
mandou a fase da câmera para a crate da família). ⭐ Aqui a falha é **barulhenta** — o `muta` aborta
na âncora e a prova conta como FALHA —, que é a sorte da história: *a forma cara é a que fica verde.*

**Prova de mutação: 9 de 9 sangram**
([`mutacao_corpo_do_jogador_2026-09-19.sh`](../ferramentas/mutacao_corpo_do_jogador_2026-09-19.sh)).
⚠️ A 9.ª **sobreviveu** primeiro: o filtro `pose_owner` corre `5` testes e **nenhum continha o
projéctil** — reapontada para o gate da ponte dele, que é o que prova que a porta tem dois leitores.

### §11.6 — ⚠️ PROMOÇÃO PEDIDA à lista de flakes de fan-out do §5.0

**`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`**
([`ph2d-tool-painter`](../../../crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs)) —
único ✗ de **`15 384`**, com as **três** assinaturas:

| assinatura | medido |
|---|---|
| zero linhas do diff naquela crate | `0` |
| 3 de 3 VERDE sozinho, com a carga ao lado | `load 8,49` · `8,02` · `8,02` |
| mede um RECURSO partilhado | `down / copy` — **duas medianas de relógio de parede** |

⛔ **É o SÉTIMO deste repo cujo doc-comment se declara imune por escrito** (*«medidos juntos, os dois
números sobem e descem juntos, e a razão entre eles é exactamente a afirmação que interessa»*) — e o
doc **narra uma flake anterior** dele, curada por essa mesma razão. ⚠️ *Verdade sobre a deriva do
PERFIL e falsa sobre o FAN-OUT*, que é a distinção que aquela lista existe para guardar.
A re-corrida do MESMO commit fechou **`15 384` de `15 384`**.

---

Portão: **15 384 testes verdes** (2.ª corrida), clippy `-D warnings` a zero nas quatro crates,
`fmt` limpo, **10 de 10 mutações a sangrar**.
