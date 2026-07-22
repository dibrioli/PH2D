# HANDOFF de CONTINUAÇÃO — `line/physics`, pós-integração (2026-07-19)

> ⚠️ **VENCIDO (2026-07-22).** O plano deste doc foi todo executado — 21 waves, W6 → W-FormDrag,
> integradas ao `main`. Quem assume a linha começa por
> [`HANDOFF_REABERTURA_line_physics_2026-07-22.md`](HANDOFF_REABERTURA_line_physics_2026-07-22.md).
> Este fica como registro do que a jornada de 19/07 planejou.

> **Para o próximo agente que assumir esta linha.** A linha **INTEGROU** ao `main` e o **plano
> original acabou** — as 8 waves + W4b + W5, todas com smoke aprovado. Este doc te diz **como
> reabrir**, **onde paramos** e **quais os planos a seguir**. Não há tarefa pendente forçada:
> os planos abaixo são um **cardápio** para o Enio escolher, não uma fila.
>
> Estado por-wave e gotchas: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) (o tracker vivo).
> Bugs cuja causa enganava: [`BUGS_physics.md`](BUGS_physics.md) (#1 o toggle, #2 corpos-filhos —
> **as 9 lições ali valem para toda wave nova**). O *porquê*:
> [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md).

---

## §0 — REABRA A LINHA primeiro (não leia código antes disto)

⛔ **Você começa na RAIZ do repo, que está em `main`.** Os mesmos paths relativos existem aqui e
na worktree — editar `crates/...` daqui edita a árvore errada, compila e commita **sem um único
erro**, e ninguém descobre até a próxima integração. Faça a FASE 0 do
[`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
**antes de tudo**. Resumo, com os valores desta linha:

```
cd Worktrees/line-physics && pwd && git branch --show-current
     → pwd TEM de terminar em /Worktrees/line-physics
     → a branch TEM de ser line/physics
git log --oneline -5 && git status -sb
git rebase main                          # FASE 1 — obrigatório no início da jornada
cargo check -p ph2d-physics-ecs          # 1º build frio é esperado, não investigue
```

⚠️ **O que o `git rebase main` vai fazer aqui, e por que é o esperado:** a linha está **100%
contida no `main`** (a integração foi *fast-forward puro*), então o rebase **avança a `line/physics`
até a `main`** — a branch vira a main, com todo o trabalho da linha já lá dentro. Você começa **do
zero sobre o trabalho integrado**, não sobre o fork antigo. Isso é correto; não é perda.

⚠️ **A `main` andou MUITO desde a integração (134 commits — Painter/sculpt, e outras linhas).**
Três números que meu handoff de integração citava **já mudaram na main** e você tem de ler os
**atuais**, nunca os do handoff velho:

| Número | Handoff de integração dizia | **Na `main` de hoje** |
|---|---|---|
| `PROJECT_SCHEMA` | 21 | **26** (`shells/desktop/src/project.rs:94`) |
| `const ITEMS` (transporte) | `[Item; 14]` | **`[Item; 15]`** (outra linha somou um item depois de mim) |
| Commits da linha | 45 | irrelevante — está tudo na main |

**Regra:** todo número de schema/contagem/allowance que você for tocar, **re-meça na `main`** — o
valor se conta, não se escolhe ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

---

## §1 — Onde paramos (o que está PRONTO e integrado)

**As 8 waves do plano `00_plano_waves.md` + duas que o Enio pediu depois do smoke, TODAS aprovadas
e no `main`:**

| Wave | O quê | Smoke |
|---|---|---|
| W1 | ponte ECS + tick no Playhead + hash cross-OS | ✅ `=1` |
| W1.5 | scrub bit-exato (checkpoint ring, `STRIDE=10`) | ✅ `=2` |
| W2a | autoria: seção "Physics Body" no Inspector | ✅ `=3` |
| W2b | painel global de mundo (tecla `W`) + Air Drag vs Damping | ✅ `=4` |
| W2c | camadas de colisão (matriz da Unity, 8 camadas) | ✅ `=5` |
| W3 | joints (pino/mola/corda/motor; entidade-joint) | ✅ `=6` |
| W4 | bake-to-timeline (a sim vira animação editável) | ✅ `=7` |
| **W4b** | toggle **Physics** no transporte (o Play dirige o solver *iff* armado, **off por default**) | ✅ `=7` |
| **W5** | corpos **FILHOS** na hierarquia (o collider volta pra debaixo do sprite) | ✅ `=8` |

**O que a linha construiu, em uma frase cada:** um sprite ganha corpo pelo Inspector e cai; Play/
Pause/Reset dirigem a sim; arrastar a régua pra trás re-simula **bit-exato**; joints são entidades
(herdam undo/save/hierarquia); o bake anota a sim em curvas da timeline; o transporte separa
"tocar animação" de "simular física"; e um corpo físico parenteado simula **onde é desenhado**.

**Verificado no `main` de hoje** (não confie, confira você também): `PROJECT_SCHEMA=26` ·
`world_transform{,_into}` é a porta única de "onde está esta entidade" (`ph2d-ecs`, usada pela
ponte E pelo overlay) · 15 arquivos de teste em `crates/ph2d-physics-ecs/tests/` · `BUGS_physics.md`
e o tracker presentes.

**Duas correções que ficaram registradas e NÃO são código** (leia antes de "consertar" algo que já
foi decidido): o plano dizia que um interruptor de física *"seria o desenho errado de qualquer
jeito"* (era resposta a **outra** pergunta — o Bake, não o transporte) e o `readback` prometia
corpos-filhos *"no W2"* desde o W1, quatro waves atrás. As duas em `BUGS_physics.md` #1 e #2.

---

## §2 — Os planos a seguir (cardápio para o Enio; reporte e PARE, não escolha sozinho)

O plano original acabou. Abaixo, o que ficou **explicitamente aberto**, ordenado por natureza.
Cada item diz **por que não foi feito** — quase todos são cercas de Chesterton
([[feedback_documented_decision_chesterton_fence]]), não esquecimento.

### A) CORREÇÃO nomeada — a mais forte candidata técnica

- **A escala não alcança o collider** (aberto no W5). Um sprite escalado 2× tem collider do tamanho
  **autorado** — `body_desc` lê `col.shape` verbatim. ⚠️ **Pré-existente e vale IGUAL para
  corpo-raiz**, então é ortogonal ao W5 e é **wave própria**. Não é mecânico: exige decidir o que
  fazer sob **escala não-uniforme** (uma bola vira elipse — rapier não tem colisor elipse nativo; a
  resposta honesta pode ser "escala uniforme escala o raio, não-uniforme recusa ou aproxima"). É
  decisão de design antes de código. É o único item da lista que é **correção**, não capacidade.

### B) O MULTIPLICADOR de capacidade — precisa de ordem do Enio

- **Sensores / triggers.** O ADR-0131 já esboça `is_sensor: bool` no `Collider` e o comentário no
  componente diz *"waits for a consumer of its own"*. O consumidor natural é **"colisão → sinal"**,
  que casa com o item `markers→signals` aberto na **timeline** (outra linha). É o item que abre
  *gameplay* (gatilhos, zonas, pickups) em cima do rígido. **É cerca de Chesterton** — o `is_sensor`
  foi deixado sem consumidor de propósito; construir exige o Enio dizer que quer o consumidor.

### C) POLIMENTO do W3 (joints) — todos pequenos, todos com motivo de estarem fora

- **Assar um JOINT** (aberto no W4). Hoje o bake lê a pose de **corpos**: uma corrente assada vira N
  kinematic com curvas próprias — reproduz o movimento, **descarta a articulação**. Assar *a
  restrição* (ou recusar assar corpos unidos, com toast) é decisão de design.
- **Gizmo de âncora no canvas** — um handle de **PONTO**; os 3 publicadores de `GizmoView` são
  CAIXAS com alças de escala. A âncora já é autorável pelos campos Position (§12), então é
  **refinamento**, não buraco.
- **Re-escolher os corpos de um joint** — precisa de um *picker de entidade* que o Inspector não
  tem. Hoje: apague o joint e faça outro.
- **Weld (`FixedJoint`)** — ~4 linhas e um chip, deliberadamente FORA: nada no smoke o exercita, e
  um 4º chip que a wave não fuma é chip shipado às cegas.
- **Motor em mola/corda** — o rapier expõe; nenhum consumidor pediu.

### D) POLIMENTO do W4 (bake) e do W4b (toggle)

- **Escolher os canais do bake** — hoje escreve X/Y/Rotation sempre que se movem; não há "só a
  rotação".
- **Alcance com INÍCIO** — o bake parte sempre do tick 0 (a sim é função do tick); assar `[2s, 5s]`
  seria assar de 0 e descartar o começo. Nada pede ainda.
- **Um Ctrl+Z para as duas metades do bake** — as chaves (fila da timeline) e o `BodyKind::Kinematic`
  (fila global do editor) vivem em filas de undo diferentes; unificar é mudança na arquitetura de
  undo, não no bake.
- **Atalho de teclado para o toggle Physics** (o painel tem `L`, o de mundo `W`; o toggle não tem).

### E) DÍVIDA de arquitetura (não faça sem pedir — é redesenho)

- **`GlobalTransform` não é consultado** pela ponte: ela compõe a cadeia ela mesma porque o
  `GlobalTransform` é `PresentComponent` (vive no mundo de apresentação) e a física roda no
  `SimWorld`. Se algum dia a propagação publicar no sim, `world_transform` é a segunda porta a
  fechar. Hoje **não** há bug — as duas rotas concordam (gate `the_scratch_walk_is_the_plain_walk`).

**Recomendação, se o Enio pedir "escolha você":** **(A) escala→collider** primeiro (é a única
correção, e todo o resto do módulo herda um collider certo), depois **(B) sensores** (é o que
transforma "física que cai" em "física que joga"). O resto (C/D) é polimento oportunista.

---

## §3 — O que NÃO reabrir (fora de escopo por ADR — D9)

- **Soft-body XPBD** (`ph2d-physics-soft`, M13+) e **fluidos FLIP/PIC** (`ph2d-fluids`, M13+) são
  **linhas próprias**, não waves desta. O motor foi projetado com um ponto de extensão central e
  isolado para elas estenderem sem colidir (ADR §D9) — mas **abrir isso é decisão do Enio e linha
  nova**, não continuação daqui.
- **Collider-gen vetorial + fratura** (ADR-0063) foi **aposentada** com a ADR-0108. O motor
  app-level **não reabre a 0108** nem herda a 0063.

---

## §4 — O terreno que carrega para qualquer wave nova (não re-derive)

1. **`ph2d_ecs::world_transform(world, entity)` é A resposta a "onde esta entidade está no mundo?"**
   sobre o `SimWorld`. Quem lê `Transform` **cru** e o trata como mundo está certo **só enquanto a
   entidade for raiz** — e a premissa é invisível, porque toda fixture feita de raízes passa. O W5
   achou **seis** desses sítios; o último só apareceu no smoke, com os colliders empilhados no
   centro da cena. **Todo código novo que computa em mundo e lê `Transform` deve usar a porta**, e o
   gate que pega a classe é ter **um pai** na fixture ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
2. **A ORDEM do frame é lei:** o apply da timeline escreve o `Transform`, o `readback` da física
   escreve **depois**. Um corpo dinâmico cuja pose a timeline dirige seria sobrescrito pelo solver —
   é por isso que o bake vira `Kinematic` e o toggle Physics existe. Qualquer coisa que escreva
   `Transform` tem de saber quem escreve por último.
3. **`BTreeMap`, não `HashMap`, é a espinha do determinismo** (itera por `Entity`, estável
   cross-OS). A lint disallowed-`HashMap` é o guarda; o hash `physics-ecs-c9` roda na matriz 3-OS do
   `spike.yml` e **nenhum é pinado em literal** (o CI compara os OSes entre si).
4. **O componente de física é CONFIG, nunca estado vivo de solver** — o `canonicalize` do undo
   ordena por BYTES do componente, então guardar velocidade/sleep ali faria cada frame virar um
   passo de undo. O mundo rapier **não é persistido**: é derivado dos componentes a cada frame.
5. **MEÇA antes de limitar** (§0 do CLAUDE.md). Todo teto do módulo (`STRIDE=10`, `DEFAULT_SUBSTEPS=4`,
   `MAX_AIR_DRAG=10`, 8 camadas) tem uma tabela de medição ao lado. Um teto novo sem medição é um
   palpite esperando um smoke.
6. **`rapier2d` NÃO pode ganhar `parallel`/`simd-*`** — quebra HR-5 (determinismo). Fica OFF.
7. **Gate red-first + mutação, sempre.** Todo gate desta linha nasceu vermelho sobre o bug real com
   os números do PRODUTO e morreu por uma razão nomeável; e cada um foi **mutado** para confirmar
   que sangra. A auditoria de 2 lentes do W4 achou 3 gates que nasceram **verdes sobre o bug que
   existiam pra pegar** — procure essa forma (oráculo que computa o esperado *com a função sob
   teste*) antes de confiar num gate que passou de primeira.

---

## §5 — Ao FECHAR a próxima jornada (lembrete do protocolo)

Você **fecha a linha, escreve o handoff de integração (DIRETRIZ §1.5.9) e PARA** — **NÃO integra
nem pusha sozinho** (CLAUDE.md §0.7). Integração e ship só por **ordem EXPLÍCITA do Enio**, via
agente integrador dedicado. E rode o **`scripts/ship.sh` inteiro** no fechamento, não confie na
tabela do handoff anterior: no fechamento do W5 o ship achou um `typos` real que o handoff declarava
limpo — entre uma wave e o fechamento entra prosa e código que a tabela velha não viu
([[feedback_ship_parity_gaps_ci_only]]).

⚠️ **Nesta máquina, o default do rustup se perdeu no meio da sessão** e só o pin `1.95` está
instalado. Rode os cargos com `rustup run 1.95 cargo …` (ou `env RUSTUP_TOOLCHAIN=1.95 bash
scripts/ship.sh`, que chama `cargo` nu). É ambiente, não código
([[feedback_a_ship_x_can_be_the_environment_not_the_code]]).
