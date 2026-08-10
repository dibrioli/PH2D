# HANDOFF MESTRE — `line/physics` → `main` (2026-08-08)

**A linha está FECHADA e PARADA.** 47 commits, 148 arquivos, +23.072/−1.227.
Nada integrado, nada pushado. Base: `a4018d203` (= o `main` de hoje).

> Este handoff **supersede** o
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-05.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-05.md),
> que cobre a jornada até a **W23** e cujo título deixou de ser verdadeiro quando
> a W24 entrou. ⚠️ **O detalhe de mecanismo daquelas waves continua lá e NÃO foi
> copiado para cá** — este documento acrescenta o que veio depois (W24..W27, o
> plano 07 inteiro) e re-conta os números contra o `main` de hoje.
>
> O handoff `..._W11b_2026-08-05.md` é o mais antigo dos três e descreve só o
> começo; ele já era superseded pelo de 05/08.

---

## §1 — O que entra, em quatro blocos

| bloco | waves | onde está o detalhe |
|---|---|---|
| **A — a cauda do player dinâmico** | W11b · W11c · W12..W18 · W19..W23 | **MESTRE de 05/08**, §4..§5j |
| **B — os quatro últimos itens da §8** | **W24 · W25 · W26 · W27** | §4 **deste** documento |
| **C — o plano 07, os três reports do dinâmico** | **W-Water · W-Submerged · W-ClingPull · W-Landing** | §5 deste documento + [`07_plano_player_kinematico.md`](../07_plano_player_kinematico.md) §6 |
| **D — o SEGUNDO MODO** | **W-KinMove** + a rodada de smoke de 08/08 | §6 deste documento |

⚠️ **A ordem entre C e D é load-bearing e está escrita no plano 07:** a cura do
pouso (W-Landing) é uma mola mais RÍGIDA, e uma mola mais rígida **amplifica** a
puxada que a W-ClingPull conserta. Consertar o pouso primeiro pioraria a jangada
no mesmo commit. Os commits já estão nessa ordem; um rebase que os reordene
quebra a premissa.

---

## §2 — Os números que se contam

| número | veredito |
|---|---|
| **`PROJECT_SCHEMA`** | ⚠️ **55 → 60** (cinco degraus: W13, W14, W15, W17, W23) — ver §3 |
| `FLIP_SCHEMA` · `VEC_SCENE` | **intocados** (13 · 14) |
| registro `ph2d-physics-ecs` | **28 → 29** (`PlayerMode`, da W-KinMove) |
| registro `ph2d-ecs` (as **três** casas) | **INTOCADO** — a crate não é tocada pela linha |
| gizmo ids | **nenhum novo** — o próximo livre segue **974** |
| **ADR** | **nenhum** ⇒ a linha fica **fora de toda disputa de número** |
| contrato congelado | **4/4 verde**, rodado (`architecture_tool_contract_surface`) |
| `Cargo.toml` / `Cargo.lock` | **ZERO arquivos tocados** — nenhuma crate nova, nenhuma dep nova |
| crates tocadas | `ph2d-platformer` · `ph2d-physics` · `ph2d-physics-ecs` · `ph2d-panel-physics` · `ph2d-panel-inspector` · `ph2d-editor-core` · `ph2d-i18n` · `shells/desktop` |
| **`physics_ecs_c9`** | **`dd5230d7…`, 108 corpos, debug ≡ release** |
| suítes | verdes em **release**; `ph2d-platformer` **133/133** |

### ⚠️ O `c9` moveu-se DUAS vezes, e a segunda tem dono

O `main` de hoje diz `b3dbe792…`. A linha diz `dd5230d7…`.

1. **W11b/W11c** (`b3dbe792…` → `74d4ea5d…`) — o cancelamento da gravidade passou
   a ser integrado *como* a gravidade e o amortecimento default subiu ao teto: a
   altura de repouso do player muda, logo o hash **tem** de mudar.
2. **A jornada do plano 07** (`74d4ea5d…` → `dd5230d7…`) — a **W-ClingPull** mexe
   na metade de baixo da perna e a **W-Landing** troca a rigidez default. As duas
   são mudanças de LEI do corpo dinâmico.

⚠️ **Tudo o mais é byte-neutro, e isso é PROVA, não sorte.** A fita do `c9`
carrega `down: false`, `dash: false`, `grab: false` **com o porquê escrito no
sítio**: as capacidades novas (descida, arranque, agarrar) são **opt-in**, e um
harness que as ligasse moveria o hash sem ganhar cobertura — ele mede
DETERMINISMO. A **W-KinMove** também é byte-neutra (medida antes e depois: o
`PlayerMode` ausente não muda um corpo).

⚠️ **E é por isso que a W17/W24/W25 têm gates PRÓPRIOS de gravação:** um `c9`
byte-idêntico prova que a simulação não mudou e diz **zero** sobre a fita.

---

## §3 — ⚠️ O bump, e por que ele é PROVISÓRIO

A escada que esta linha escreve, cada degrau um campo **apendado** a um
componente ou ao arquivo (postcard é **posicional**):

| degrau | wave | o quê |
|---|---|---|
| **v56** | W13 | `PlatformPlayer` + 5 campos de PAREDE |
| **v57** | W14 | + 3 campos de ARRANQUE |
| **v58** | W15 | + 2 campos de AGACHAR |
| **v59** | W17 | ⚠️ campo de **ARQUIVO** (`player_tape`), fora do `ProjectState` |
| **v60** | W23 | + os campos do AGARRAR-SE |

⚠️ **O valor se CONTA contra o `main` do dia da integração, nunca se copia.** Se
outra linha da janela bumpar, o certo pode não estar em nenhum dos dois lados —
aconteceu **três vezes** com a `line/FLIP` (30 · 32/33/34 · 47), uma com a
`line/Vector`/`sculpt3d` (a jornada de 04/08) e uma **dentro do handoff desta
própria linha**, que contou UM degrau onde havia DOIS.

⚠️ **E o `project.rs` pode não conflitar mesmo assim:** se as duas linhas
escreverem o mesmo literal, o git funde limpo e o bump da segunda **evapora com a
suíte inteira verde**. Quem denuncia é o conflito do `project_schema_tests.rs` ao
lado, e a tripla que ele pina — **`(60, 13, 14)`** aqui.

⚠️ **`project.rs` foi PARTIDO no `main`** pela `line/sculpt3d` (o
`project_load_from` saiu para `project_load.rs`). Um hunk desta linha que edite o
corpo daquela função **funde limpo para o lado errado do corte** e evapora —
confira por `grep`, não por *"o merge foi limpo"*.

---

## §4 — Bloco B: os quatro últimos itens da §8 (não estão no handoff de 05/08)

**W24 — descartar a corrida tem VOLTA.** O *Clear Recorded Run* destruía uma
gravação num clique, sem confirmação e sem desfazer. A fita **não é
`ProjectState` de propósito** (um Ctrl+Z do canvas não deve rebobinar uma
gravação), então ela não herda o undo global e a cura tinha de ser própria:
descartar **MOVE** (`mem::take`) para um guardado de SESSÃO, e o mesmo lugar da
tela oferece *Restore Discarded Run*. ⚠️ O ciclo de vida é **DERIVADO, não
mantido**: o botão de devolver só é oferecido com a fita viva VAZIA.

**W25 — a corrida é um fato do DOCUMENTO, e ganhou casa.** Os dois botões que a
governam moravam só na §14 do Inspector, que é **por-entidade** — e o
`build_player_info` devolve `None` para tudo o que não é um Dynamic selecionado,
então **apagar o personagem prendia a corrida**: ela continua no arquivo (W17),
continua a ser o que o Bake replaya (W16), e não havia gesto que a alcançasse.
⚠️ **A cura NÃO é uma fita por-player:** com um teclado há um dedo, e o
`hand_input_to_players` já o entrega a todos — fitas por-entidade gravariam N
cópias idênticas da mesma corrida e custariam um bump por uma redundância.

**W26 — a deriva e o quique NÃO estão soldados.** A nota dizia *"o pouso perdeu
os 24 mm de quique que o Spring Damping em meio curso dava; o slider devolve-o"*
— uma troca com o MESMO knob nos dois lados. Medido, há um **terceiro eixo**:

| | `spring_damping` | `substeps` |
|---|---|---|
| deriva de rampa | `∝ (1 − d)` | **`∝ 1/n`** |
| quique do pouso | `∝ (1 − d)` | **INDEPENDENTE** |

⇒ no par `d = 0,25 · n = 12` sobram **99% do quique com um terço da deriva**.
⚠️ A tabela do `BUGS_physics.md` §7 é **PRÉ-`gravity_hold`** e leva à conclusão
oposta.

**W27 — a borda de baixo da descida era uma ARMADILHA.** A nota registava o
defeito pelo SINTOMA (*"as pranchas ficam fantasma"*); medido, o preço era o
personagem **descer um degrau e ficar lá para sempre** (vão 1,60: `−0,598` depois
da descida e `−0,598` depois do pulo de volta). ⚠️ A cura não foi nenhuma das
**quatro** já prescritas — as três da W19 trocavam um regime por outro, e a da
W21 foi construída, medida (*nenhuma diferença*) e **revertida**.

---

## §5 — Bloco C: o plano 07, os três reports do player DINÂMICO

Os três vieram do Enio na mesma janela (2026-08-07) e **cada um foi medido antes
de ter cura escrita** — e em dois deles a medição mudou o assunto:

| wave | o report | o que a medição achou |
|---|---|---|
| **W-Water** | *"não interage corretamente com a água"* | **não é sobre água**: o sensor tratava um SENSOR como matéria sólida, e o personagem ficava **de pé sobre a poça** |
| **W-ClingPull** | *"ao primeiro toque a jangada é ATRAÍDA para o player"* | a metade de baixo da mola **puxa**, e a 3ª lei transmite: a jangada **sobe 96,9 mm** |
| **W-Landing** | *"a desaceleração ao encostar no chão é muito lenta"* | **0,500 s** para assentar; a causa é o `1 − k·dt²`, e com a rigidez certa o mesmo pouso custa **0,133 s** |

Mais a **W-Submerged**, que a cena expôs: a água **alimentava** o personagem.

⚠️ **A cura do pouso é a RIGIDEZ, não o amortecimento, e as duas colunas da
direita é que decidem:** baixar o amortecimento dá o mesmo pouso e **gasta** a
deriva que a W11c comprou; a rigidez não a toca (`0,0000` em toda a faixa) e o
soco que a 3ª lei entrega a uma jangada não muda (+1,5%).

**Smoke `=100`: APROVADO** (*"Smoke OK Siga"*, 2026-08-08).

---

## §6 — Bloco D: o SEGUNDO MODO, e a rodada de smoke de 08/08

O `PlayerMode` (componente valuado), o `Support::Snap`, o `move_shape` do rapier,
a velocidade no `PlayerState`, a plataforma móvel e o chip da §14. Detalhe no
plano 07 §6 (`W-KinMove`), que já registra as **três** coisas que a medição
derrubou do próprio plano (a régua da perna · as DUAS perguntas sobre chão · a
barra *"zero por construção"*, que é **4,7 cm de PELE do controlador**).

### ⚠️ O 1º smoke REPROVOU, e os dois reports têm causas DIFERENTES

**(a) *"o ciano está com uma mola extremamente exagerada, um pula-pula"*** — **a
CENA, não o produto.** Ela nascia com o `spring_damping` de FIXTURE (¼ do teto) e
com o personagem a `0,334 m` da rampa quando a perna dele repousa a `0,900`. O
quique precisa das **três** condições ao mesmo tempo; cada uma sozinha dá zero:

| rampa | berço | `damping` | quique |
|---|---|---|---|
| plano | fora do repouso | 0,25 | **0,0 mm** |
| rampa | no repouso | 0,25 | **0,0 mm** |
| rampa | fora do repouso | 0,50 | **0,0 mm** |
| **rampa** | **fora do repouso** | **0,25** | **5913 mm** |

⚠️ A lição não é *"0,25 é um valor mau"* — é a FRONTEIRA: **um número escolhido
para uma fixture não atravessa para a mão do artista.** Gate novo
`shells/desktop/tests/a_smoke_scene_ships_the_default_tuning.rs`.

**(b) *"o laranja ao pousar se aproxima da rampa … na direção da normal"*** — **a
LEI, e a seta que ele desenhou é o mecanismo.** Com `drive = 0` o freio da
caminhada cancela a velocidade ao longo da **tangente** do chão; uma queda
vertical tem componente tangencial em qualquer inclinação, então o freio a lê
como escorregão e a apaga — e **o que sobra de uma queda vertical sem a tangente
é a NORMAL**. Ablação pelo knob `acceleration`: **com freio `−0,0711 m`, sem
freio `+0,0001 m`**.

Cura de **ORDEM, não de lei**: o `settle` deixa no estado a queda que o mundo
bloqueou, o `kinematic_advance` a apaga — e **entre os dois corria a LEI**. A
ponte passa a chamar a MESMA porta (`supported_velocity`, extraída do integrador,
agora com dois consumidores) antes de a lei ler.

| queda | antes | depois |
|---|---|---|
| 0,5 m | −0,102 m | **−0,068 m** |
| 1,5 m | −0,071 m | **−0,044 m** |
| 3,0 m | −0,057 m | **−0,023 m** |

⚠️ **O modo DINÂMICO fica byte-intocado** (`+0,1469 / +0,3079 / +0,3905` antes e
depois) — a correção vive no ramo `writes_own_pose`.

⚠️ **RESÍDUO NOMEADO:** o tique de **CONTATO** ainda chuta (ali o personagem está
mesmo no ar e nenhuma absorção é devida). O que a correção remove são os tiques
SEGUINTES — deslocamento **depois** do contato **4,4 mm contra 39,0 mm** da
mutação (**8,8×**), e é esse o oráculo do gate. Fechar o resto exigiria a `walk`
perguntar ao **controlador** se está apoiada, o que quebra a **K4**: decisão de
produto, não dívida mecânica.

⚠️ **A extração é *pure code motion* inclusive em `NaN`** — o guard ficou
`into < 0.0`, e **não** `!(into >= 0.0)`: as duas divergem ali.

### ⚠️ E a §0 do plano 07 ganhou a coluna que lhe faltava

| queda | afunda SPRING (default) | afunda SNAP (default) |
|---|---|---|
| 0,5 m | **0,0000 m** | 0,0436 m |
| 10,0 m | **0,0000 m** | 0,0465 m |

⇒ **no default que shipa, o cinemático afunda ~4,5 cm onde o dinâmico afunda
zero.** Isto não revoga o modo — a justificação nunca foi *"afunda menos"*, foi
*"o número não depende de knob e não cresce com a queda"*, e as duas metades
continuam medidas. A frase honesta é que **ele troca 4,5 cm de pele por
independência de afinação**, e quem decide se é bom negócio é o Enio.

---

## §7 — Ordem da integração

1. `git rebase main` (ou merge). Os arquivos compartilhados são o `project.rs`
   (doc-comment + literal), o `project_schema_tests.rs` (a tripla) e o roteador
   de smoke — **CONTE o `PROJECT_SCHEMA`, não o copie** (§3), e confira que o
   `project.rs` partido não engoliu nenhum hunk.
2. Rodar o gate da árvore combinada **em DEBUG E RELEASE** — esta linha tem
   precedente registado de vermelho **só-em-debug**.
3. Recomputar o **`physics_ecs_c9` depois** do rebase: deve dar **`dd5230d7…`**,
   108 corpos, debug ≡ release. ⚠️ Se der `74d4ea5d…`, a jornada do plano 07
   perdeu-se; se der `b3dbe792…`, a W11b/W11c perdeu-se; se der **outra coisa**,
   alguma capacidade desta jornada deixou de ser opt-in — **e isso é o achado**.
4. Rodar os gates que **não** correm numa varredura por-crate — esta linha já
   shipou vermelhos latentes por eles: `architecture_workspace_file_loc_cap`,
   `file_loc_caps` da shell, `no_tofu_glyphs`, `arch_safe_clamp_only`.
5. Rodar `shells/desktop/tests/no_two_smoke_scenes_claim_the_same_level` — a
   linha acrescenta **onze** cenas (`=91`..`=101`) e o roteador é uma lista de
   `if level == N` em que **o primeiro vence**.

---

## §8 — Smoke

⚠️ **Cada cena imprime o que montou.** Se a linha `[physics-smoke NN]` não
aparecer, **pare**: a cena não montou e o resto não diz nada.

```
env PH2D_PHYSICS_SMOKE=97  cargo run -p ph2d-host-desktop --release  # as duas bordas da descida (W20/W27)
env PH2D_PHYSICS_SMOKE=98  cargo run -p ph2d-host-desktop --release  # o FLANCO (W22)
env PH2D_PHYSICS_SMOKE=99  cargo run -p ph2d-host-desktop --release  # o agarrar-se (W23)
env PH2D_PHYSICS_SMOKE=100 cargo run -p ph2d-host-desktop --release  # A AGUA (plano 07) -- APROVADA
env PH2D_PHYSICS_SMOKE=101 cargo run -p ph2d-host-desktop --release  # OS DOIS MODOS -- RE-SMOKE
```

**Estado:** blocos A e C **aprovados** (as aprovações por-wave estão no MESTRE de
05/08 §7 e no tracker). **O bloco D está PENDENTE de re-smoke** — é a cena `=101`
com as duas curas de 08/08.

⚠️ **A `=101` mudou de roteiro.** Ela agora roda no **default**, com cada
personagem nascido na própria altura de repouso, e **o passo 4 é a wave inteira**:
baixar o `Spring Damping` do ciano para ~0,25 **pelo Inspector** e derrubar os
dois do plateau — o ciano passa a afundar ~156 mm, o laranja continua nos 44 e
**não cresce com a altura da queda**. Os passos 1-3 e 5 são o resto (altura de
repouso, caminhada idêntica, a rampa quieta nos dois, o chip de ida e volta).

---

## §9 — Aberto, com o preço ao lado

- **O resíduo do tique de contato** (§6): fechá-lo exige a `walk` perguntar ao
  controlador se está apoiada ⇒ quebra a K4. **Decisão do Enio.**
- **O veredito PARCIAL da W11** que veio do `main` (*"o player sobe sozinho bem
  devagar"*) tem cura medida e **não construída** — §14 do handoff da W10/W11; o
  default que shipa deixa **0,164 m de resíduo por 10 s numa rampa de 30°**,
  nomeado e gateado dos dois lados.
- **`W-KinWeight` (cena `=102`)** — a massa AUTORADA. A 3ª lei (K6) já corre nos
  dois modos pela MESMA porta; o que falta é o número.
- **`W-KinPush` (`=103`)**, **`W-KinPure`** (o terceiro modo) e **`W-KinTune`**
  (o que o smoke pedir) — não começadas.
- O horizonte do plano 02 §8 (IK multibody · params keyframáveis · Wheel preset ·
  Rod/soft weld · copiar-colar propriedades) segue **não escalonado**.
