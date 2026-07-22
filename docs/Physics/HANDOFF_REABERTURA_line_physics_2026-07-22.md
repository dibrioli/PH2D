# HANDOFF de REABERTURA — `line/physics` (para o agente que assume, 2026-07-22)

> Escrito pelo agente que fechou a jornada de 21/07. A linha **integrou** — está a **zero
> commits** de `main`. Você não está herdando trabalho pendente; está herdando um **módulo
> vivo com plano**.
>
> **Leia este arquivo inteiro antes de tocar em qualquer coisa.** Ele tem três partes:
> §1 como reabrir · §2 o que já existe (para você não reconstruir) · §3 o plano.

---

## §1 — REABRA A LINHA (faça isto primeiro, sem pedir confirmação)

Siga [`MODELO_ABERTURA_LINHA.md`](../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) —
o bloco inteiro, com **`$MODULO = physics`**. As regras permanentes A–H dele valem para
você até o fim da sessão, sem exceção.

⚠️ **A worktree e a branch JÁ EXISTEM** (a linha continua, não renasce). Então o passo 4
do modelo é a rota "linha reaberta", e nem `git worktree add` é preciso:

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics
pwd; git branch --show-current    # DEVE dizer .../Worktrees/line-physics e line/physics
git rebase main                   # a worktree está ~93 commits atrás; deve ser fast-forward
```

⚠️ **`pwd` antes de editar, sempre.** O mesmo path relativo existe na raiz e na sua
worktree; editar `crates/...` na raiz compila e commita **sem erro nenhum** e você perde o
trabalho no merge. É a armadilha nº 1 do Modo L.

Depois: `bash scripts/hw-profile.sh` (tem de dizer `workstation`), o warm-up
(`cargo check -p ph2d-physics-ecs`, primeiro build frio é minutos — **não investigue a
demora**), e leia `DIRETRIZ.md` §0/§1.5/§2/§6 + `DIRETIVA_IMPLEMENTACAO.md` inteira.

**Reporte "Linha `physics` reaberta. Aguardo a tarefa." e PARE.** A tarefa vem do Enio.

### O que NUNCA fazer nesta linha

**Integrar. Pushar. Rodar `ship.sh`.** Você fecha a wave, escreve o handoff (regra H) e
**espera**. Quem funde é um agente integrador dedicado, por ordem explícita do Enio.

---

## §2 — O QUE JÁ EXISTE (para você não reconstruir nem re-litigar)

**Norte, e não se re-litiga:** *runtime-truth* + bake opcional · **rígido primeiro** · o
solver é o `rapier2d 0.28` que já existia — **esta linha escreve integração e autoria,
não solver** ([ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)).

**Estado no `main` agora:** `PROJECT_SCHEMA = 29` · registro de componentes = **18** ·
`physics_ecs_c9 body_count = 75` · 28 cenas de smoke (`PH2D_PHYSICS_SMOKE=1..28`), **todas
aprovadas pelo Enio**.

### Onde as coisas moram

| Crate / arquivo | O que é |
|---|---|
| `ph2d-physics` | o wrapper do rapier. `world.rs` + irmãos por responsabilidade: `desc` (plain data), `shape`, `collider_build`, `effector` (zonas), `buoyancy`, `form_drag`, `drag` (ar do mundo), `oneway`, `sensors`, `contacts`, `damping`, `layers`, `joints`, `kinematic`, `checkpoint`, `defaults`, `queries` (leituras) |
| `ph2d-physics-ecs` | a ponte. `bridge.rs` + `bridge/{triggers,contacts,damping,hold,joints,kinematic,space,diagnostics}`, `components.rs` + `components/overrides.rs`, `scale.rs` (a porta ECS→rapier), `bake.rs`, `settings.rs`, `bin/physics_ecs_c9.rs` |
| `ph2d-editor-core` | `ids/inspector.rs` (os `INSP_PHYS_*`) + `screens/hero/inspector_model_physics.rs` (`InspectorPhysicsInfo`, `PhysicsFieldEdit`) |
| `ph2d-panel-inspector` | `sections/physics.rs` + `sections/physics_rows.rs` · `event_physics.rs` · `populate.rs` · `tests/seam_physics.rs` |
| `ph2d-panel-physics` | o painel de MUNDO (tecla `W`), `rows.rs` com **UMA tabela, quatro consumidores** |
| `shells/desktop` | `render_loop/inspector_physics{,_apply,_markers}.rs` · `inspector_joint.rs` · `physics_overlay{,_joints,_contacts}.rs` · `physics_bridge.rs` · `physics_smoke*.rs` |

### As leis do módulo (violar = bug que os gates pegam, e você vai perder tempo)

1. **Componente de física é CONFIG, nunca estado vivo de solver.** O `canonicalize` do
   undo ordena por BYTES do componente ⇒ guardar velocidade/sleep ali faz **cada frame
   virar um passo de undo**.
2. **Tudo que o corpo é tem de rider o `BodyDesc`.** `rewind_to` reconstrói o mundo DOS
   DESCRITORES; um valor que não esteja lá é descartado em silêncio no primeiro scrub.
3. **`BTreeMap`, nunca `HashMap`.** Determinismo cross-OS; há lint estrutural.
4. **Componente NOVO é aditivo (sem bump); campo novo num componente existente É bump.**
   Blob de componente é postcard **posicional**. E ⚠️ **um bump recusa TODO projeto já
   salvo** — foi decidido quatro vezes seguidas, sempre a favor do componente novo.
5. **`PROJECT_SCHEMA` se CONTA, não se escolhe** — se outra linha bumpar na mesma janela,
   some os dois lados.
6. **Nada de `parallel`/`simd-*` no rapier**, e todo transcendental **nosso** por
   `libm` (o `libm::sincosf` das tesselações) — 1 ulp já é bug cross-OS.

### As armadilhas que esta linha já pagou (não repita)

- **Oráculo que usa a função sob teste** para computar o que espera é **sempre verde**.
  Três gates nasceram assim.
- **Fixture que não contém o fenômeno.** O filtro de sobreposição sobreviveu a 5 gates
  porque toda fixture era uma **caixa** (forma == AABB); o sinal do one-way sobreviveu a 3
  porque a plataforma nascia sempre primeiro. **Varra a ordem, use forma redonda.**
- **O controle atropelado pelo experimento** — três vezes: um corpo "que não deve se
  mexer" acertado pelo corpo que o experimento lançou. Controle vai para **fora do
  caminho**, e às vezes o caminho é uma *direção* ou uma *coluna*.
- **Sistema amortecido re-converge** ⇒ meça a **TRAJETÓRIA**, não o endpoint; e não meça
  cedo demais.
- **Defesa em camadas precisa de gate POR camada** — se duas camadas bastam sozinhas,
  mutar uma só deixa tudo verde. Mute as duas para provar que o gate é sobre algo.

---

## §3 — O PLANO

⚠️ **Antes de fechar qualquer wave, releia a §"Toda wave chega à UI" de
[`00_plano_waves.md`](00_plano_waves.md).** É política, não sugestão: as quatro condições
(existe · é pintado e registrado · o clique chega ao barramento · **a SEQUÊNCIA leva a
algum lugar**), a metade visível, e a cena com **números medidos**.

⚠️ **A quarta condição é a que esta jornada descobriu e é a menos óbvia:** todo edit pode
ter gate e o gesto ainda não levar a lugar nenhum. Foi ela que pegou um passo que eu quase
ensinei ao Enio (converter um tronco deitado para Capsule — geometricamente correto, e
destrói o objeto).

### Ordem sugerida — mas a escolha é do Enio

Ele conduz wave a wave: cada "siga"/"próximo" é **uma** wave, fechada com gate batched,
smoke e commit local. **Não encadeie duas sem um novo sinal.**

#### A. Eventos de contato — início e fim *(o mais valioso, e o mais desenhado)*

O `W-Contacts` entrega *quem está tocando agora*. Falta *"eles se tocaram AGORA"* e
*"pararam de se tocar"* — o que gameplay consome (som de impacto, dano, gatilho).

- **É outra estrutura:** exige memória entre frames (o conjunto do frame anterior), onde a
  lista atual é recomputada do zero a cada dispatch. Cuidado com o **scrub**: um replay
  não pode emitir cem eventos de início.
- **A precedência do W7 manda tornar VISÍVEL primeiro** — um canal sem consumidor é flag
  morto. Um flash na cruz branca no frame do início é o mínimo honesto.
- ⚠️ **O consumidor de gameplay (script/marker de timeline) é CROSS-LINE** e continua
  decisão do Enio; não o construa por conta própria.

#### B. Força de impacto real

Hoje o impulso reportado é a **carga** que o par carrega, não o pico — medido, cair de 6 m
dá o mesmo número que estar parado, porque o `step` retorna depois de o solver ter parado o
corpo. O pico vive **entre** os sub-passos. Capturá-lo custa acumular dentro do laço de
sub-passos **em toda cena**, por uma leitura de debug — **meça o custo antes**.

#### C. Gizmo de âncora de joint no canvas

Refinamento, não buraco: a âncora **já é autorável** pelos campos Position da §12. O que
falta é um handle de **PONTO**, e os três publicadores de `GizmoView` são **caixas com
alças de escala** — é trabalho de gizmo, não de física.

#### D. Assar um JOINT

O bake lê a pose de **corpos**; uma corrente assada vira N kinematic com curvas próprias —
reproduz o movimento e **descarta a articulação**. Assar *a restrição* (ou recusar assar
corpos unidos) é **decisão de design**, não mecânica. Junto: alcance com **início** (hoje
sempre parte do tick 0, porque a sim é função do tick) e **um Ctrl+Z para as duas metades**
(as chaves vão na fila da timeline, o `kind` na fila global).

#### E. A família das zonas — o que ficou aberto

- **Falloff dentro da área** (a força é uniforme; a Unity tem gradiente, o Godot não);
- **torque de área** (a zona empurra o centro de massa, então não gira nada);
- **a força é em eixos de MUNDO** ⇒ girar a zona **não gira o vento**;
- **multiplicador por-corpo** (`AreaResponse`) — ⚠️ **não construa sem pedido**: hoje
  *camada + massa* já cobre "não sente" e "sente menos", e a Unity resolve com **máscara**,
  não com peso;
- **o cata-vento aparece sozinho** no dia em que um lastro deslocar o centro de massa (o
  kernel já aplica por aresta) — há gate pinando a ausência dele para ninguém inventar um
  torque;
- **ondas** (a superfície do empuxo é plana) e **`Compound`/`TriMesh`** (o `local_polygon`
  devolve `None` de propósito — nenhum empuxo é melhor que um sobre silhueta inventada).

#### F. Dívidas menores, honestas

- `reconcile` de corpos obsoletos é **O(N²)** (trivial nos counts de hoje — meça antes);
- o `readback` só trata corpo **raiz**;
- escala não-uniforme **+ rotação** compõe cisalhamento que a decomposição joga em
  `scale`+`skew`, e o **skew é ignorado** (rapier não cisalha collider);
- `GlobalTransform` não é consultado (é `PresentComponent`; a física roda no `SimWorld`);
- o readout do painel de mundo conta corpos, não **corpos dormindo** — a pergunta *"por que
  nada se move?"* teria resposta melhor com os dois números;
- o toggle **Physics** do transporte não tem atalho de teclado (`L` é timeline, `W` é
  mundo).

### FORA de escopo — não abra sem ADR (D9)

**Soft-body XPBD · fluidos FLIP/PIC · collider-gen vetorial + fratura.** São módulos
próprios (M13+), não waves desta linha.

---

## §4 — Os documentos, em ordem de leitura

1. [`00_plano_waves.md`](00_plano_waves.md) — o mapa e a **política de UI**;
2. [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) — o tracker: **uma seção por
   wave**, com o *porquê*, as medições e a armadilha de cada uma. É longo de propósito;
   leia a seção da wave que você for tocar, não o arquivo inteiro;
3. [`ADR-0131`](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)
   — o norte;
4. [`BUGS_physics.md`](BUGS_physics.md) — bugs cuja causa enganava;
5. [`HANDOFF_INTEGRACAO_line_physics_2026-07-21.md`](HANDOFF_INTEGRACAO_line_physics_2026-07-21.md)
   — o handoff da integração que acabou de acontecer; útil para saber o que foi entregue.

**CLAUDE.md §5** tem o resumo do módulo — e é ele que a próxima LLM lê primeiro, então
**mantenha-o atualizado a cada wave**.
