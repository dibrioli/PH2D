# HANDOFF de REABERTURA — `line/physics` (para o agente que assume, 2026-07-23)

> Você está assumindo uma linha **já integrada ao `main`**. A jornada anterior fechou
> QUATRO waves (a trilogia de contatos + a mesa giratória) e todos os smokes do núcleo
> foram aprovados pelo Enio. Este documento diz **como reabrir**, **o que já existe** (para
> você não reconstruir nem re-litigar) e **o que sobrou** — com a ordem sugerida.
>
> Substitui o [`HANDOFF_REABERTURA_line_physics_2026-07-22.md`](HANDOFF_REABERTURA_line_physics_2026-07-22.md)
> (as frentes A e B dele estão FEITAS; o resto foi carregado para cá).

---

## §1 — REABRA A LINHA (faça isto primeiro, sem pedir confirmação)

A branch e a worktree **já existem**. É a rota "linha reaberta" do
[`MODELO_ABERTURA_LINHA`](../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md):

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics
pwd && git branch --show-current      # DEVE dizer .../line-physics e line/physics
git fetch origin && git rebase main   # a linha entra em cima do main de HOJE
cargo check -p ph2d-physics-ecs       # o inner loop desta linha
```

⚠️ **`cd` + `pwd` + `git branch --show-current` ANTES de ler ou editar qualquer arquivo.**
A janela abre na raiz (`=main`) e **o mesmo path relativo existe nas duas árvores** — editar
a errada compila e commita **sem erro nenhum**. E a **cwd do Bash volta ao primário entre
comandos**: prefixe **todo** comando com o `cd` da worktree
([[feedback_bash_cwd_resets_and_slips_to_the_primary]]).

### O que NUNCA fazer nesta linha

- **Não integre, não pushe, não rode `ship.sh`.** Integração e ship são **ordem explícita do
  Enio**, via agente integrador dedicado (CLAUDE.md §0.7). Você fecha a wave, atualiza o
  tracker, escreve o handoff e **PARA**.
- **Não encadeie duas waves sem um sinal novo do Enio.** Terminou uma? Reporte e espere.
- **Não abra soft-body / fluidos / fratura** (D9 — módulos próprios, exigem ADR).

---

## §2 — O QUE JÁ EXISTE (não reconstrua, não re-litigue)

O módulo está **maduro**: 33 cenas de smoke, o conjunto de propriedades por-corpo
**completo** (Unity/Godot-paridade), a família de zonas com 6 componentes, joints, bake,
scrub bit-exato, e o canal de contatos completo (estado + transição + pico + toque rápido).

### Estado numérico no `main` (⚠️ **RE-CONTE, não assuma**)

| | |
|---|---|
| `PROJECT_SCHEMA` | **29** |
| registro de componentes (`register_physics_components`) | **19** |
| `physics_ecs_c9` | **77 corpos**, hash `27f3c1aa…` |
| cenas de smoke | **1..33** |

⚠️ **Estes números SOMAM entre linhas.** Se `line/Painter`/`line/anim`/`line/Vector`
integraram na mesma janela, o valor certo **se CONTA a partir do que chegou ao `main`
primeiro** — nunca se escolhe ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
Confira lendo o fonte (`project.rs`, `lib.rs`, o c9), não este documento.

### As leis do módulo (violar = os gates pegam, e você perde tempo)

1. **Component de física é CONFIG, nunca estado vivo de solver.** O `canonicalize` do undo
   ordena por BYTES do componente ⇒ guardar velocidade/sleep ali faz **cada frame virar um
   passo de undo**.
2. **Tudo o que o corpo É tem de ridar o `BodyDesc`.** O mundo rapier é *derivado* e
   reconstruído; o que não estiver no descriptor **some no scrub**. Gate obrigatório de toda
   wave nova: *"um rewind re-arma isto"*.
3. **`BTreeMap`, nunca `HashMap`** (determinismo cross-OS; há lint estrutural).
4. **Componente NOVO é aditivo (blob-key própria) ⇒ ZERO bump.** Apendar **campo** a um
   componente existente é postcard POSICIONAL ⇒ **bump**, e um bump **recusa todo projeto já
   salvo**. Prefira sempre o componente novo (é o que as 6 da família de zonas fizeram).
5. **MEÇA antes de limitar** (CLAUDE.md §0.0). Todo teto desta linha tem tabela ao lado.
6. **Toda wave fecha com as QUATRO condições de UI** — o componente EXISTE
   (`every_physics_component_is_authorable`) · é **pintado e registrado**
   (`architecture_panel_wiring_parity`) · o **clique chega ao barramento** (varredura de
   seam) · e a **SEQUÊNCIA leva a algum lugar** (`inspector_physics_gesture_tests`). A quarta
   **não é implicada** pelas outras três.
7. **A metade VISÍVEL conta como UI** — e a resposta pode ser *"nada, de propósito"*, desde
   que escrita. Força tem seta, torque tem glifo de giro, arrasto **não tem** (vê-se nos
   corpos desacelerando).
8. **Toda wave ganha CENA de smoke com números MEDIDOS** — rode a sonda headless **antes** de
   escrever a mensagem. Nesta linha, duas cenas já afirmaram coisas que a medição desmentiu.
9. **Rode debug E release.** Só-release esconde pânico (a lição que a `line/FLIP` pagou).
10. **Gate red-first + mutação.** Um gate que nunca foi visto VERMELHO não prova nada.

### As armadilhas que esta linha já pagou (NÃO repita)

- **Uma coordenada que WRAPA é oráculo ruim.** `Transform.rotation` dá a volta em ±π: com
  torque forte o corpo gira várias revoluções e o ângulo lido vira ruído (medido: compacta
  2,688 / barra −1,254 — sem sentido como taxa). Os gates de MUNDO leem `angvel` cru; os de
  ECS/gesto/smoke leem `rotation` e por isso a fixture **mantém o giro sub-revolução**.
  [[feedback_a_wrapping_coordinate_is_a_bad_oracle_measure_the_rate]]
- **O controle é atropelado pelo próprio experimento** (aconteceu **3×**): o corpo "que não
  deve se mexer" foi arremessado/atingido pelo que o experimento lançou. Ponha o controle
  **fora do caminho** — e "o caminho" às vezes é uma *direção*.
- **A fixture tem de CONTER o fenômeno.** O filtro `intersecting` sobreviveu a 5 gates porque
  toda fixture era uma CAIXA (onde forma e AABB coincidem); o gate que pegou usa zona
  **redonda**. A fixture do sono não continha o fenômeno e 2 mutações passaram.
- **Um oráculo que usa a função sob teste é sempre verde.** Três gates do W2b nasceram assim.
- **Duas grandezas que devem DIFERIR podem coincidir por FASE da fixture** (`max` vs
  `último`): ache a fixture onde diferem por FÍSICA, não por sorte.
- **Um gate ancorado em DISTÂNCIA DE BYTES no fonte é proxy que expira** (a `line/Vector`
  pagou dois na integração de 23/07). Afirme a PROPRIEDADE.
- **O `file_loc_caps` da shell e o `arch_safe_clamp_only` NÃO rodam** num `cargo test -p` por
  crate — entram no gate de fechamento explicitamente (duas waves já ficaram vermelho-latentes
  por isso).
- **As rows de zona são SENSOR-only** (Force/Torque/Drag/Fluid Density/Shape Drag). Num
  collider sólido elas nem são pintadas, de propósito — a narrow phase não reporta overlap
  para sólido. Se o Enio disser *"não vejo na UI"*, é quase sempre isto.
- **⚠️ O `sync_inspector_from_snapshots` deriva a entidade do snapshot de TRANSFORM**, não do
  de física. Um gate que só seta o snapshot de física **não dispara o sync** e nasce verde
  sobre nada.

---

## §3 — O PLANO (o que sobrou)

**FEITO na jornada anterior, não reabra:** eventos de contato (início/fim) · força de impacto
(o pico entre sub-passos) · o toque RÁPIDO virar evento (diff por tick) · o **torque de área**
(a mesa giratória) · o fix das rows de área serem write-only.

### ⚠️ Primeiro: dois itens estão PENDENTES DE SMOKE

A jornada anterior fechou com duas coisas que o Enio **ainda não smokou**:

1. **Cena `=33`** (autoria pela UI: `Add → Static → Sensor → Torque`, mais a mesa já autorada
   que mostra a row preenchida ao selecionar);
2. **o fix de sync** que fez as 5 rows de área **mostrarem** o valor autorado (eram
   write-only — autorar funcionava, re-selecionar lia `0`).

São gated e de baixo risco, mas **peça o smoke antes de empilhar wave nova em cima**.

### Ordem sugerida — mas a escolha é do Enio

Pergunte antes de escolher; ele decide a frente. Da mais valiosa para a menos:

#### A. A família das ZONAS — o que sobrou *(a veia mais rica, e a mais desenhada)*

- **Falloff dentro da área.** Hoje força e torque são **uniformes** dentro da zona; um
  redemoinho real cai com o raio, uma explosão também. A Unity tem gradiente, o Godot não.
  ⚠️ O caro é decidir **de que ponto** o raio se mede numa zona que não é redonda (uma caixa
  não tem centro natural para um gradiente radial) — é design antes de código.
- **O FRAME da zona.** A força e o torque estão em eixos de **MUNDO**, então **girar a zona
  não gira o vento** — o que quebra o caso da esteira diagonal. A cura é autorar no frame
  LOCAL e rodar pelo transform do sensor (zona não-rotacionada fica byte-idêntica). ⚠️ Há
  precedente para os dois lados (o `AreaEffector2D` da Unity tem `useGlobalAngle`), então
  decida com o Enio se vira toggle ou troca de default.
- **NÃO construa `AreaResponse`** (multiplicador por-corpo) sem pedido: *camada + massa* já
  cobre "não sente" e "sente menos", e a Unity resolve com **máscara**, não com peso.
- **Ondas** (a superfície do empuxo é plana) e **`Compound`/`TriMesh`** (o `local_polygon`
  devolve `None` **de propósito** — nenhum empuxo é melhor que um sobre silhueta inventada).
- **O cata-vento aparece sozinho** no dia em que um lastro deslocar o centro de massa (o
  kernel já aplica por aresta) — há gate pinando a ausência dele para ninguém inventar torque.

#### B. Assar um JOINT

O bake lê a pose de **corpos**; uma corrente assada vira N kinematic com curvas próprias —
reproduz o movimento e **descarta a articulação**. Assar *a restrição* (ou **recusar** assar
corpos unidos, que talvez seja a resposta certa) é **decisão de design**, não mecânica. Junto:
alcance com **início** (hoje sempre parte do tick 0, porque a sim é função do tick) e **um
Ctrl+Z para as duas metades** (as chaves vão na fila da timeline, o `kind` na global).

#### C. Gizmo de âncora de joint no canvas

Refinamento, não buraco: a âncora **já é autorável** pelos campos Position da §12. O que falta
é um handle de **PONTO**, e os três publicadores de `GizmoView` são **caixas com alças de
escala** — é trabalho de gizmo, não de física.

#### D. Readout de contatos na §11

Um número ("N contatos, carga total"), não a cruz do overlay. Nunca foi pedido — **confirme
com o Enio** antes (a §11 não tem row de readout nenhuma hoje, então seria um padrão novo).

#### E. Dívidas menores, honestas

- `reconcile` de corpos obsoletos é **O(N²)** (trivial nos counts de hoje — **meça** antes);
- o `readback` só trata corpo **raiz**;
- escala não-uniforme **+ rotação** compõe cisalhamento que a decomposição joga em
  `scale`+`skew`, e o **skew é ignorado** (rapier não cisalha collider);
- `GlobalTransform` não é consultado (é `PresentComponent`; a física roda no `SimWorld`);
- o readout do painel de mundo conta corpos, não **corpos dormindo** — *"por que nada se
  move?"* teria resposta melhor com os dois números;
- o toggle **Physics** do transporte não tem atalho (`L` é timeline, `W` é mundo);
- `damp_mode` **por-eixo** (hoje é um modo só para linear+angular).

#### F. O consumidor de GAMEPLAY — ⚠️ **cross-line, decisão do Enio**

Colisão → som/dano/marker/callback. O canal de eventos **existe e está completo**
(`contact_events()` com `Began`/`Ended`, lugar, carga e pico). O que falta é o CONSUMIDOR, e
ele mora fora da física (timeline `Marker` / `ph2d-script`). **Não comece sem o desenho do
consumidor** — é a fronteira que o W7 traçou e ela ainda vale.

### FORA de escopo — não abra sem ADR (D9)

**Soft-body XPBD · fluidos FLIP/PIC · collider-gen vetorial + fratura.** Módulos próprios
(M13+), não waves desta linha.

---

## §4 — Os documentos, em ordem de leitura

1. **`CLAUDE.md` §0 + §5** (a entrada "Física global") — os inegociáveis e o estado.
2. **[`docs/Physics/HANDOFF_line_physics.md`](HANDOFF_line_physics.md)** — o TRACKER: uma
   seção por wave, com o *porquê* de cada decisão. É onde você confere se algo "novo" já foi
   feito e rejeitado.
3. **[`docs/Physics/00_plano_waves.md`](00_plano_waves.md)** — o mapa das waves (⚠️ wave fora
   do mapa entra nele na MESMA sessão).
4. **[`docs/Physics/BUGS_physics.md`](BUGS_physics.md)** — os bugs cuja causa enganava.
5. **[ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)**
   — o *porquê* do runtime-truth. Não re-litigue.
6. **`project-memory/MEMORY.md`** — as lições duráveis (as famílias de auditoria são 2 saltos).
