# HANDOFF — REABERTURA da `line/physics` (2026-08-10)

> **Você é o agente que assume esta linha.** Ela foi **reaberta do zero** hoje, depois de a
> jornada do modo cinemático integrar ao `main`. Este doc diz **o que existe** (para você não
> reconstruir), **o que está aberto com o preço ao lado**, e os **gotchas operacionais** que já
> custaram tempo a esta linha. O tracker
> [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) é o estado **por-wave**, para consulta
> pontual — não leitura linear.

---

## §1 — O que você recebe (estado literal, medido hoje)

| fato | valor |
|---|---|
| worktree | `Worktrees/line-physics/` (criada 2026-08-10 pela rota `-b`) |
| branch | `line/physics` |
| HEAD | `76788440a` — **idêntico ao `main` do dia** |
| commits desta linha | **ZERO** |
| árvore | **limpa** — não há trabalho não-commitado do agente anterior a resgatar |

⚠️ **Nada foi implementado nesta linha ainda.** O agente anterior fez só a FASE 1 do
[`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) (hw-profile ·
worktree · warm-up · mergiraf · leitura) e parou, como o modelo manda.

⚠️ **O `main` local está `ahead 5` de `origin/main`** — a integração de ontem está **local, não
pushada**. Isto **não é seu**: ship é ordem explícita do Enio (§0.7). Só explica um `git log` que
não bate com o GitHub.

**Tier `workstation`** ⇒ Modo L, RA-as-oracle, mold, muitos cargos, sem slots CoW.

---

## §2 — Os números do estado (MEDIDOS na árvore, não copiados de handoff)

| grandeza | valor | onde |
|---|---|---|
| `PROJECT_SCHEMA` | **70** | `shells/desktop/src/project.rs:379` |
| tripla do pin | **(70, 13, 14)** | o gate de schema do shell |
| registro `ph2d-physics-ecs` | **29** | `registers_every_physics_component` |
| registro `ph2d-ecs` + os **dois** espelhos | intocados pela última jornada | ⚠️ o contador é **TRÊS** |
| `physics_ecs_c9` | `fb27f676…`, **117 corpos**, debug ≡ release | ⚠️ a fonte é o **binário**, nunca este doc |
| gizmo ids | maior **973** ⇒ **próximo livre 974** | `ph2d-editor-core` |
| cenas de smoke | maior **104** ⇒ **próxima livre 105** | `physics_smoke.rs`, `match which.trim()` |

⚠️ **`PH2D_PHYSICS_SMOKE=84` NÃO existe, de propósito** — o roteador é um `match` sobre string e o
buraco é deliberado; não o "conserte".

⚠️ **O `PROJECT_SCHEMA` se CONTA contra o `main` do dia, nunca se escolhe** — e a colisão passa
**MUDA** quando duas linhas escrevem o **mesmo literal** (o `project.rs` não conflita e o git não
sabe o que o número significa). Se você bumpar: confira **os DOIS arquivos** (o `project.rs` **e**
o pin da tripla) e **escreva o degrau na escada** do `project.rs` no MESMO commit — o v69 chegou ao
`main` com a linha ausente, e *quem conta o próximo degrau lê a escada, não o literal*.

---

## §3 — O que NÃO reconstruir (leia antes de propor qualquer coisa)

**Ordem de leitura:**

1. [`HANDOFF_INTEGRACAO_line_physics_kin_2026-08-09.md`](HANDOFF_INTEGRACAO_line_physics_kin_2026-08-09.md)
   — a jornada que acabou de integrar (o modo cinemático: `W-KinWeight` · `W-KinPush` ·
   `W-KinPure` · `W-KinCarry` · `W-FloatFloor` + as cinco medições).
2. [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-08.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-08.md)
   — ⚠️ o de 09/08 o **supersede apenas como *o que integrar agora***; **o mecanismo de todas as
   waves até a W23 continua LÁ e não foi copiado**.
3. [`06_plano_player_plataforma.md`](../06_plano_player_plataforma.md) +
   [`07_plano_player_kinematico.md`](../07_plano_player_kinematico.md) — os planos vivos.
4. [`BUGS_physics.md`](../BUGS_physics.md) — os bugs cuja causa enganava.
5. [ADR-0131](../../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) —
   o *porquê* (runtime-truth + bake opcional; rígido primeiro; esta linha escreve **integração e
   autoria, não solver**).

**⛔ MEDIDO E REJEITADO — não refaça:**

- **A cura do eixo do snap** (trocar o `up` pela NORMAL da rampa) foi **construída, medida e
  REVERTIDA** — o personagem **parado** deriva **0,7297 m** e nunca assenta, e sangram dois gates.
  As duas metades pedem coisas **opostas do mesmo eixo** (plano 07 §8.1). O `up` está lá de
  propósito: é o `floor_stop_on_slope` do Godot.
- **Folgar o orçamento da corda pela violação da trava** compra **0,0007 m** — ruído.
- **O resíduo de rampa da W11 está FECHADO** — **0,0000 m EXATO** de 20° a 45° e em todo número de
  sub-passos, com as demais colunas a colapsar em `deriva(10 s) = 0,153 · sen θ · (1 − d)`. A nota
  antiga (*"0,164 m, nomeado e gateado"*) **sobreviveu ao fato por uma janela** e já foi corrigida;
  o preço que ficou é de PRODUTO e está nomeado: a perna transmite **91%** do peso (era 53%).
- **`form_drag` na lei cinemática** e **a FORÇA de uma zona levando um personagem cinemático** não
  são omissões: o primeiro é kernel **por-aresta** sobre o polígono (não um escalar do meio), e o
  segundo é **DESENHO** — a força precisa do frame, do espelho e do falloff, e re-derivá-los numa
  consulta seria a **segunda resposta** a *"que empurrão esta zona dá neste ponto?"*.

---

## §4 — O que está ABERTO, com o preço ao lado

**A · A cauda do modo cinemático** (o que a jornada de ontem nomeou)

- ~~**O player bobeia ~1,44 m numa poça, nos DOIS modos**~~ — **FECHADO POR MEDIÇÃO (2026-08-10):
  não era um defeito.** A sonda nova `measure_the_bobbing` atribuiu o excesso por ablação da
  ENTRADA: com os quatro multiplicadores de gravidade a `1` a amplitude é **`0,8097` = o controle
  ao quarto decimal**, e largado **já submerso** o player é `1,00×` o controle — ou seja, a trava
  do fluido **contém**, e o excesso inteiro é a modelagem do arco a agir **no AR**, antes do
  primeiro contacto, que é onde ela é autorada para agir. O personagem cruza a superfície a
  **`1,299×`** a velocidade do controle porque `fall_gravity = 2.0`, e o bobeio **DECAI** junto com
  o controle (`0,001 m` aos 30 s) ⇒ transiente, não bomba. Dois gates novos pinam as duas metades
  (plano 07 §8.4); ⚠️ **os três gates que já existiam ficavam VERDES** nas duas mutações (**857 m**
  e **15,3 m**), porque a trava é comum aos dois modos. O que sobra é **decisão de produto**
  (mexer em `fall_gravity` é mexer no platformer inteiro), não dívida.
- A paridade de arrasto entre os modos é **APROXIMADA** — `(1+d·h)⁻⁴` contra `(1+d·4h)⁻¹`, a mesma
  classe que a W-AreaDrag mediu em **1,25%**; um corpo cinemático não tem sub-passo.
- `form_drag` e a FORÇA de zona: ver §3 (⛔ os dois têm motivo escrito).

**B · Dívidas antigas com mecanismo já escrito**

- A explosão **não impõe torque** (decisão medida) · o campo repelindo **arremessa para fora de
  quadro** (−20 N abre a nuvem para 9,23 m em 1 s: força sustentada sem freio fora do alcance) ·
  **Rigid e Rope atravessam parede** (inerente à rigidez infinita, nomeado no enum e no smoke) ·
  soltar a mão deixa **um passo de undo** (pré-existente de QUALQUER clique no play; a cura mora no
  **roteador de undo**, outro domínio) · a trava de corda é restrição de **posição** e um balanço
  violento a ultrapassa por um sub-passo (**0,3685 contra os 0,5 pedidos**, pinado no gate).

**C · Horizonte do plano 02 §8, não escalonado**

- IK multibody · Rod/soft weld · copiar-colar propriedades entre joints.

**D · Produto novo**

- **Nadar** é o único item da lista que é *feature*, não correção — e por isso é **decisão do
  Enio**, não dívida de engenharia.

---

## §5 — Recomendação (se o Enio deixar a escolha com você)

**A**, e por um motivo estrutural: os itens dela são a única classe onde **o segundo modo do player
ainda discorda do primeiro sobre o mesmo mundo**, e as duas metades já vivem em código adjacente
(`kinematic.rs` + as consultas de zona).

**Comece pelo bobeio de 1,44 m**, nesta ordem:

1. **Meça antes de qualquer hipótese** — a sonda existe
   (`cargo test -p ph2d-physics-ecs --release --test measure_player_in_water -- --ignored
   --nocapture`). ⚠️ *O `y` de um INSTANTE não é um repouso*: o oráculo é a **amplitude**, e ela é
   comum aos dois modos.
2. **Um número, dois modos, uma cura** — se a amplitude é a mesma nos dois, a causa **não** é do
   modo cinemático, e procurá-la ali é procurar no lugar errado.
3. Só então escreva o gate red-first e a cena (**`=105`**, a próxima livre).

---

## §6 — Gotchas operacionais desta linha (cada um já custou tempo)

1. ⚠️ **Todo comando começa com o `cd` da worktree.** A cwd do Bash **escorrega** para a árvore
   primária, e o mesmo path relativo existe nas duas: editar a errada **compila e commita sem
   erro**. A `line/Painter` mandou **cinco de oito** arquivos para o `main` assim, mediu *"sem
   ganho"* e quase reportou isso como achado.
2. ⚠️ **Os gates de `shells/desktop/tests/` só correm na varredura IMPACTADA.** Um fechamento por
   `cargo test -p` por crate **não os alcança** — foi assim que esta linha shipou vermelho-latente
   mais de uma vez. **Rode explicitamente no fechamento:** `file_loc_caps` (o teto de 600 do shell,
   que o `architecture_workspace_file_loc_cap` **não** cobre) e `arch_safe_clamp_only`.
3. ⚠️ **Os `--ignored` querem a máquina CALMA e `--test-threads=1`.** Sob `load average` alto os
   kills de relógio dão vermelho **sem uma linha de código mudar** (precedente medido: 11,36 ms sob
   `load 41` contra 5,50 sob `load 0,6`). *Um número que se move sobre código intocado é a máquina.*
4. ⚠️ **O `physics_ecs_c9` roda em debug E release e os dois têm de bater** — e ele **muda** quando
   a pose muda (correto) e **não muda** quando a wave é readout (também correto). Diga qual dos dois
   você espera **antes** de rodar.
5. **Toda wave só fecha com as QUATRO condições de UI** (plano 00): o componente EXISTE · é
   **pintado e registrado** · o **clique chega ao barramento** · e **a SEQUÊNCIA leva a algum
   lugar**. A quarta é categoria própria e **não é implicada** pelas outras três.
6. **Toda wave ganha CENA de smoke com números MEDIDOS** — a sonda headless roda **antes** de a
   mensagem ser escrita. Nesta linha, duas cenas já afirmaram coisas que a medição desmentiu.

---

## §7 — Encerramento (o que você entrega)

Feche pelo gate batched (§2 do CLAUDE.md), escreva o **handoff de integração** (DIRETRIZ §1.5.9)
em `docs/Physics/handoffs/` — **nunca na raiz de `docs/`** — e **PARE**. Você **não** integra e
**não** pusha: os dois são ordem explícita do Enio, por um agente integrador dedicado.
