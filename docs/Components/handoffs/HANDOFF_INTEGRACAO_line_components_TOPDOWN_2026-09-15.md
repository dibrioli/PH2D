# HANDOFF DE INTEGRAÇÃO — `line/components`, o MOVER DE VISTA DE CIMA (TOP-20 #13)

> **Data:** 2026-09-15 · **Merge-base:** `1d43da737` · **Plano:** [`10_plano_topdown_player.md`](../10_plano_topdown_player.md)
> **Estado:** fechada. **NÃO integrada, NÃO pushada** (§0.7).

---

## §1 — O que o ARTISTA consegue fazer agora

Põe um objecto na cena, carrega em **+ Add Component → Top-Down Player**, e:

1. as setas movem-no em **8 direcções** (ou 4, ou livre, ou só num eixo);
2. **ao raspar numa parede em diagonal ele desliza à velocidade CHEIA**, em vez de rastejar;
3. um menu **Viewpoint** reprojecta a entrada para **isometria** sem trocar de componente;
4. um menu **Facing** vira o objecto para onde ele anda;
5. desligando **Default Controls**, o componente vira motor puro.

---

## §2 — Os contadores, como DELTA (⛔ nunca o literal)

| grandeza | delta | de → para |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** | `130 → 131` |
| `register_physics_components` | **+1** | `33 → 34` (delta **+2** contra o `main`, com a wave anterior) |
| espelho `ph2d-render` · espelho `ph2d-script` | **0** | eles contam `ecs + render`/`ecs + script`, **não** a física |
| registo do `ph2d-ecs` | **0** | o componente não é de lá |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `FIELD_DOC_VERSION` · `DOC_VERSION` | **0** | — |
| contrato congelado (§6) | **0** | nenhum dos sete aparece no desenho |

⚠️ **O plano dizia `+1` nos quatro registos e ESTAVA ERRADO.** Um componente de física move **dois**
contadores, não quatro — e a redacção errada teria deixado dois gates vermelhos com a cura errada
escrita ao lado.

---

## §3 — Crates novas: UMA

**[`ph2d-topdown`](../../../crates/ph2d-topdown/)** — a lei pura, folha, com **uma** dependência
(`libm`, pelo mesmo pin cross-OS da irmã `ph2d-platformer`).

⚠️ Foundational tocado: `ph2d-physics` (a porta `move_character_from`), `ph2d-physics-ecs` (o
componente, a ponte, a memória dos controladores), `ph2d-platformer` (o campo `drive_y`),
`ph2d-input` (duas acções), `ph2d-editor-core` (o vocabulário + a tabela das secções vivas).
⭐ **Não há `line/physics` viva** (`git worktree list`, 15/09) — colisão de símbolo é improvável.

---

## §4 — Os SEIS achados, com o mecanismo

### 4.1 ⭐⭐⭐ A casa fazia a lei ERRADA, e ninguém tinha medido

O `PhysicsWorld::move_character` devolve a **projecção** em todos os ângulos (`|d|/tangencial = 1,000`,
medido). O oráculo (Godot `FLOATING`) conserva o **orçamento** — `1,414×` a 45°, `2,92×` a 20°.
⇒ *o item 13 não era empacotar o que existe: era uma lei que a casa não tinha.*

⛔ E não é escolha nossa: o `slide: true` é da `rapier`. A cura é um laço de orçamento por cima de
uma porta irmã (`move_character_from`), com a porta velha a **delegar** e censo a exigir corpo de
uma linha.

### 4.2 ⛔⛔ O doc do `PlatformPlayer` mandava NÃO subir o schema, e estava errado

Ele dizia por escrito: *«Componente NOVO ⇒ blob-key própria ⇒ `PROJECT_SCHEMA` NÃO bumpa»*. Era
verdade na época dele e é falso desde o degrau `123` — um `ComponentBlob` de `type_id` desconhecido
**recusa o load inteiro**. Corrigido no mesmo commit que escreveu o degrau `131`.

⚠️ *Uma nota que descreve a casa de outra época lê-se exactamente como uma que descreve a de agora.*

### 4.3 ⛔⛔ As secções da FÁBRICA pintavam um chevron que não dobrava

Elas shiparam em 14/09 **fora** da `LIVE_SECTIONS`, e o doc daquela tabela já dizia o preço: quem
falta ali não é `mark_collapsible_section`ado e não é `is_section_header_id`. Quem o viu foi esta
wave, ao ir escrever a mesma linha. ⇒ as três entram, **e nasce o censo que faltava**
(`architecture_every_live_section_is_in_the_table`), com piso de população e lendo os dois lados do
disco.

### 4.4 ⚠️ A memória dos controladores era um mapa, e o doc dela já previa o defeito

O doc do `player_state` dizia: *«um segundo mapa teria de ser acrescentado àquele ring à mão — e
esquecê-lo é um scrub que devolve o mundo de um tique e a memória do controlador de outro, sem erro
e sem aviso»*. O segundo controlador chegou. ⇒ o anel passa a guardar um **tipo**
(`ControllerMemory`), e o terceiro controlador **não compila** sem passar pelo `record` e pelo `seed`.

### 4.5 ⚠️ A posse da pose só conhecia o irmão

`pose_owner` respondia `Scene` para um corpo com `TopDownPlayer`, logo o `settle` desfazia a pose e
o `Transform` ficava parado com o `rapier` a andar. ⭐ E a cura **não** é exigir `PlayerMode`: um
mover de vista de cima é controlador **por construção** (não há mola nem chão), e exigi-lo faria o
componente nascer inerte com todos os números certos.

### 4.6 ⛔ Um `-D warnings` que só corre no `ship.sh` é um portão que a LINHA não vê

Três avisos de clippy eram da wave **anterior** desta linha (um `..default()` inerte, dois imports
mortos, um `if/return None` que é um `?`), mais o `late()` a oito argumentos. É a **segunda** vez que
esta linha descobre um vermelho seu na wave seguinte.

---

## §5 — As SETE leituras que o diff inverte

1. **`drive_y` no `PlayerInput` não é «um campo a mais na struct do platformer»** — é o MESMO facto
   (*para onde o jogador empurra*) na struct que a **fita determinística** grava. Um segundo canal
   ficaria de fora do replay. Há gate a provar que a lei de plataforma é **byte-idêntica** com ele
   em qualquer valor, incluindo `NaN`.
2. **`default_controls = false` não inventa um canal** — ele tira a entidade da lista que a FITA
   alimenta, e quem a dirige passa a ser o `set_player_input`, que já existe e já tem chamadores.
   ⛔ Um canal novo só para isto seria uma porta sem consumidor.
3. **`max_slope_deg = 0` não é «sem rampa»** — é o `motion_mode = FLOATING` escrito no vocabulário
   da casa: numa vista de cima não há chão, há obstáculo.
4. **O `TopDownState` não ser componente é a decisão**, não um esquecimento: um campo que muda por
   tique num componente registado faria o undo ver cada quadro como um passo.
5. **A divergência abaixo do limiar é declarada e a alternativa foi CONSTRUÍDA** (§6-bis do plano):
   `slide: false` faz o controlador deixar de reportar contacto de forma fiável, com zeros erráticos
   a 25°, 30°, 40° e 65°.
6. **A identidade `Free + TopDown` é bit-a-bit, e isso custou uma cura**: a 1.ª redacção passava por
   um `normalize` + escala e mudava o último bit.
7. **O `TopDownPlayer` na lista `WRITERS` do censo de autoria não afrouxa o gate** — a lista deixou
   de ser «os escritores da família da física» e passou a ser «os escritores, onde quer que morem».

---

## §6 — As CINCO premissas minhas que a medição derrubou

1. *«O componente move os três registos (+1 cada)»* — move **dois**.
2. *«O limiar do produto é o do oráculo»* — a 1.ª tabela leu um limiar a `22,5°` que era a **fronteira
   do encaixe de 8 direcções**: a sonda media o quantizador.
3. *«Uma parede maior é uma fixtura melhor»* — com meia-altura `20 000` o cast devolve `hits = 0` e o
   corpo **atravessa o cenário**, e a leitura parece um resultado.
4. *«Desligar o deslize da biblioteca é a cura»* — construído, medido, **recusado**.
5. *«44 literais exaustivos de `PlayerInput`»* — eram **37**: o meu regex contava
   `-> PlayerInput {` (o corpo de uma função) e lia `(40..48)` como sintaxe de actualização de
   struct. ⚠️ *A régua e o produto partilhavam o defeito, e o número «esperado» vinha da mesma régua.*

---

## §7 — O SMOKE (o que o dono corre)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_TOPDOWN_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_TOPDOWN_SMOKE=2 cargo run -p ph2d-host-desktop --profile smoke
```

Diagnóstico: `PH2D_TOPDOWN_LOG=1` imprime cada passo do plano de deslize.

---

## §8 — O que fica ABERTO, com o mecanismo

1. **WASD de fábrica** — decisão do dono: o `W` desta shell abre o painel de mundo, e as acções
   nascem atadas às **setas**. O WASD fica a dois cliques no Input Map.
2. **O penhasco dos 15°** — o alvo tem `235×` entre 15° e 16°; nós temos a projecção, que é contínua.
   A rampa seria produto novo sem lado aprovado.
3. **Os âmbitos com prioridade do Input Map** continuam bloqueados no `shells/game`/R1 — o mesmo
   bloqueio de sempre, e **não** um preço desta wave.
4. **O custo a 1 000 movers** é `263 %` de um quadro. Cem são `21 %`. O `max_slides` é a alavanca e
   não foi varrido.
5. **O `late()` da shell tem oito argumentos** com `#[allow]` e a razão escrita. A cura seria pior
   que o aviso (ver o doc).
6. **Um top-down spawnado DENTRO de uma parede atravessa-a** — o primeiro tique de uma cena tem o
   BVH vazio, e isto é partilhado com o irmão de plataforma (a casa já o documenta).

---

## §9 — Onde LER

- o plano, com as tabelas medidas e as recusas: [`10_plano_topdown_player.md`](../10_plano_topdown_player.md);
- a lei: [`ph2d-topdown`](../../../crates/ph2d-topdown/src/lib.rs) — o cabeçalho tem a lei do orçamento;
- a ponte: [`bridge/topdown.rs`](../../../crates/ph2d-physics-ecs/src/bridge/topdown.rs);
- o oráculo: [`godot_topdown_probe.gd`](../ferramentas/godot_topdown_probe.gd) e o corpus em
  [`godot_slide.txt`](../../../crates/ph2d-topdown/tests/fixtures/godot_slide.txt);
- as 24 provas de mutação: [`mutacao_topdown_w1.sh`](../ferramentas/mutacao_topdown_w1.sh) e
  [`mutacao_topdown_w2.sh`](../ferramentas/mutacao_topdown_w2.sh).
