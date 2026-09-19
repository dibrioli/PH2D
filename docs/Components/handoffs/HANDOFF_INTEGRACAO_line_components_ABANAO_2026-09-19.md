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
