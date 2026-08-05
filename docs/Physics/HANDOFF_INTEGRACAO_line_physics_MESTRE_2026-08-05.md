# HANDOFF MESTRE — `line/physics` → `main` (2026-08-05)

**A linha está FECHADA e PARADA.** 9 commits, 3 waves de produto.
Nada integrado, nada pushado.

> Este handoff **supersede** o [`HANDOFF_INTEGRACAO_line_physics_W11b_2026-08-05.md`](HANDOFF_INTEGRACAO_line_physics_W11b_2026-08-05.md),
> que descreve só o começo da jornada (a W11b/W11c) e cujo título deixou de ser
> verdadeiro quando a W12 e a W13 entraram. O detalhe de mecanismo daquelas duas
> continua lá e não foi copiado para cá.

---

## §1 — O que entra

| commit | o quê |
|---|---|
| `2b4f0df0d` | o degrau **v54** que faltava na escada do `PROJECT_SCHEMA` |
| `865aa410f` | **W11b** — o que cancela a gravidade passa a ser integrado como ela |
| `43852a422` | o handoff da W11b |
| `f3cfb9b96` | o veredito do 1º smoke + duas tentativas medidas e mortas |
| `aec55c5b6` | **W11c** — o default do amortecimento sobe ao TETO |
| `ce78651cf` | **W12** — descer da plataforma jump-through |
| `eebac6d1e` | a W12 no plano e no mapa |
| `14c95c974` | **W13** — AS PAREDES |
| `33dfc5cbc` | a W13 no plano e no mapa |

**Smoke:** W11b/W11c **APROVADAS** pelo Enio (*"Smoke OK"*, 2026-08-05).
**W12 e W13 pendentes** — integrar não é aprovar.

---

## §2 — Os números que se contam

| número | veredito |
|---|---|
| **`PROJECT_SCHEMA`** | ⚠️ **55 → 56** (W13) — ver §3 |
| `FLIP_SCHEMA` · `VEC_SCENE` | intocados (13 · 14) |
| registro `ph2d-physics-ecs` | **INTOCADO** (28) — nenhum componente novo |
| registro `ph2d-ecs` (as 3 casas) | **INTOCADO** |
| gizmo ids | **nenhum novo** — o próximo livre segue **974** |
| ADR | **nenhum** ⇒ a linha fica fora de toda disputa de número |
| contrato congelado | **4/4 verde**, rodado |
| `Cargo.toml` | **zero** — nenhuma dep, nenhuma crate |
| **`physics_ecs_c9`** | **`74d4ea5d…`, 108 corpos, debug ≡ release** |

⚠️ **O `c9` moveu-se UMA vez na jornada inteira, e foi na W11b/W11c** (a altura de
repouso do player mudou). **A W12 e a W13 são byte-neutras** — a descida exige um
botão que a fita do harness não segura, e as paredes nascem **desligadas**. Isso
não é sorte: é a prova executável de que as duas capacidades novas são opt-in.

---

## §3 — ⚠️ O bump, e por que ele é PROVISÓRIO

**W13:** o `PlatformPlayer` ganhou **cinco** campos (`wall_slide_speed`,
`wall_jump_height`, `wall_jump_push`, `wall_jump_lockout`, `wall_reach`).
Apendados ao componente, e o postcard é **posicional** ⇒ um save v55 lido por v56
chega ao fim dos bytes no primeiro campo novo. O número é o que transforma isso
num erro de **VERSÃO** em vez de num postcard a falhar longe da causa.

⚠️ **O valor se CONTA contra o `main` do dia da integração, nunca se escolhe.**
Esta linha escreve **56**; se outra linha da janela bumpar, o certo pode não estar
em nenhum dos dois lados do conflito — foi o que aconteceu três vezes com a
`line/FLIP` (30 · 32/33/34 · 47) e uma com o próprio handoff desta linha, que
contou UM degrau onde havia DOIS.

⚠️ **E o `project.rs` pode não conflitar mesmo assim:** se as duas linhas
escreverem o mesmo literal, o git funde limpo e o bump da segunda **evapora com a
suíte verde**. Quem denuncia é o conflito do `project_schema_tests.rs` ao lado, e
a tripla que ele pina — **`(56, 13, 14)`** aqui.

---

## §4 — W12: descer da plataforma (cena `=91`)

O plano 06 §4 agendava isto como *"o mecanismo existe (`world/oneway.rs`); a
feature é o gesto, e é uma wave curta depois da W8"*. A previsão sobreviveu à
construção.

**O gesto é `down + jump`** — não `down` sozinho: quem segura baixo enquanto anda
não pode cair da plataforma sem ter pedido, e o dia em que existir um agachar o
botão já estará com o significado certo.

⚠️ **A lei diz COMEÇAR; a ponte diz quando ACABA** — a divisão do sensor de quina
da W10. E o **fim da descida não é um relógio**: *"eu já passei?"* tem resposta
exata (a caixa do personagem inteiramente abaixo da caixa da plataforma), e um
temporizador erraria justamente onde dói — plataforma grossa, queda lenta —
re-solidificando com o personagem dentro dela.

⚠️ **O sensor tem de excluir a plataforma, e não só o solver:** quem segura o
personagem no ar é a **MOLA**, e ela age porque o raio achou chão. Daí o
`cast_ray_skipping`, com o `cast_ray` a **delegar** — uma porta, duas faces.

⚠️ **O bit viaja no corpo que CAI** (`DROP_THROUGH_BIT`, o 2º consumidor que o
doc do `ONE_WAY_BIT` previa), escrito em **TODOS** os colliders do corpo (a lição
da W-Compound), e **por tique**, nunca no `BodyDesc`.

**7 mutações, 7 sangram.** Caso degenerado nomeado: um vão menor que o personagem
deixa a descida armada para sempre — cena já quebrada sem descida nenhuma.

---

## §5 — W13: as paredes (cena `=92`)

⚠️ **O §4 previa duas waves e são UMA** — as duas metades partilham a pergunta
*estou agarrado?*, e separá-las daria duas respostas para *o que conta como
parede*. A previsão foi corrigida **na linha em que foi escrita**.

⚠️ **Uma parede é o que a PERNA já recusou.** Sem segundo limiar: um
`wall_min_angle` discordaria do `Max Slope` autorado.

### ⚠️ A medição derrubou DUAS frases minhas

**(1) A lei que eu escrevi.** O escorregamento era um TETO, raciocinado. O knob é
**INERTE**: medido, quem empurra contra uma parede **não cai** — 9 cm em um
segundo, por **atrito** (`DEFAULT_FRICTION = 0,5`) mais a gravidade do **ÁPICE**
(metade do peso, auto-reforçante). A lei que ficou **DEFINE** a velocidade.

| `wall_slide_speed` | desceu em 1 s |
|---|---|
| 0,0 (desligado) | **0,09 m** ← a COLA |
| 1,0 | 0,71 m |
| 3,0 | 2,76 m |
| 6,0 | 5,51 m |
| 12,0 | 11,01 m |

**(2) *"O afastamento satura"*.** Ele **não satura** — cresce linear, porque com o
controle aéreo calado nada freia a horizontal. Quem satura é a **ALTURA**, e é
dali que sai o `jump_lockout = 0,2 s`.

| `jump_lockout` | subiu (de 2,0 autorados) | afastou |
|---|---|---|
| 0,00 s | 1,621 m (81%) | 0,462 m |
| 0,10 s | 1,921 m (96%) | 1,137 m |
| **0,20 s** | **1,932 m (97%)** | **1,737 m** |
| 0,50 s | 1,932 m (97%) | 3,437 m |

⚠️ E o pulo entregar 76% era a **mesma doença que o `lift_momentum` da W10
nomeou** — *"quem apagava era a ASSISTÊNCIA"*.

**5 mutações, 4 sangram**, e a 5ª nomeia uma **defesa em camadas** (o
`drive * side` do `cling` é inalcançável pela ponte; quem o mata é o gate de
unidade). **Nasce DESLIGADA** — card **WALLS** próprio na §14, cinco rows.

---

## §6 — Ordem

1. `git rebase main` (ou merge). Os arquivos compartilhados são o `project.rs`
   (um doc-comment + o literal), o `project_schema_tests.rs` (a tripla) e o
   roteador de smoke — **CONTE o `PROJECT_SCHEMA`, não o copie** (§3).
2. Rodar o gate da árvore combinada **em DEBUG E RELEASE** (esta linha tem
   precedente registado de vermelho só-em-debug).
3. Recomputar o `physics_ecs_c9` **depois** do rebase: deve dar **`74d4ea5d…`**,
   e conferir debug ≡ release. ⚠️ Se der `2278035e…`, a W11c perdeu-se; se
   MUDAR, alguma coisa desta jornada deixou de ser opt-in e isso é o achado.
4. Rodar os gates que **não** correm numa varredura por-crate: o
   `architecture_workspace_file_loc_cap`, o `file_loc_caps` da shell, o
   `no_tofu_glyphs` e o `arch_safe_clamp_only` — esta linha já shipou vermelhos
   latentes por eles não serem alcançados por `cargo test -p`.

---

## §7 — Smoke

```
env PH2D_PHYSICS_SMOKE=81 cargo run -p ph2d-host-desktop --release   # rampa 30° (W11c)
env PH2D_PHYSICS_SMOKE=88 cargo run -p ph2d-host-desktop --release   # o par 40°/50°
env PH2D_PHYSICS_SMOKE=85 cargo run -p ph2d-host-desktop --release   # a jangada (o PESO)
env PH2D_PHYSICS_SMOKE=91 cargo run -p ph2d-host-desktop --release   # A ESCADA DE PRANCHAS (W12)
env PH2D_PHYSICS_SMOKE=92 cargo run -p ph2d-host-desktop --release   # O POÇO (W13)
```

⚠️ **Cada cena imprime o que montou.** Se a linha `[physics-smoke NN]` não
aparecer, pare: a cena não montou e o resto do smoke não diz nada.

- **`=91`** — o que se julga é **um andar por aperto**. Se ele for ao chão de uma
  vez, a retirada da descida quebrou.
- **`=92`** — o vão é **2,4 m de propósito**, mais largo do que um pulo de parede
  atravessa sozinho (1,74 m medidos): subir exige soltar a direção a meio do voo.

---

## §8 — Aberto, com o preço ao lado

- **W11c:** o pouso perdeu os 24 mm de quique que o `Spring Damping` em meio curso
  dava. O slider devolve-o; a troca está nomeada no handoff da W11b §5.
- **W12:** um vão entre plataformas **menor que o personagem** deixa a descida
  armada para sempre. A cena já está quebrada sem descida nenhuma.
- **W13:** o sensor lateral olha só a altura do **MEIO** do corpo — uma beirada
  que alcance só os pés não é vista (a mesma limitação honesta da folga lateral da
  W10). E não há *wall grab*: ficar **parado** numa parede é outra mecânica, com
  botão próprio, e não se alcança escrevendo `0` no `Wall Slide`.
- **Do plano 06 §4, o que sobra:** *dash* · *agachar* · **bake de um player**
  (desbloqueado desde a W7 — *"com a fita, assar passa a fazer sentido"*) ·
  persistir a fita · player Kinematic.
