# HANDOFF DE INTEGRAÇÃO — `line/motion-value` · 2026-08-29

> **Para o agente integrador.** Escrito sob [DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md).
> ⛔ A linha **fecha e PARA**: não integra, não pusha ([`CLAUDE.md §0.7`](../../../CLAUDE.md)).

---

## 1. Identidade

| | |
|---|---|
| branch | `line/motion-value` |
| HEAD | `2b8660b86` |
| merge-base com `main` | `330582deb` |
| commits | **45** (27/08 → 29/08; **15 em 29/08**) |
| arquivos tocados | **167** |

---

## 2. Foundational / compartilhado tocado, e por que é ADITIVO

| arquivo | o que entrou | aditivo? |
|---|---|---|
| `crates/ph2d-node-registry/src/ui.rs` | `ParamWidget::File { kind: FileKind }` + o enum `FileKind` | **SIM** — variante nova num enum que o painel casa por `match`; nenhum widget existente muda |
| `crates/ph2d-node-registry/src/unit.rs` | o braço `File { .. }` em `unit_of` → `ParamUnit::None` | **SIM** — o `match` era exaustivo e precisava do braço |
| `crates/ph2d-node-registry/src/lib.rs` | re-export de `FileKind` | **SIM** |
| `crates/ph2d-node-registry-init/src/lib.rs` | duas linhas de `register` (`source.lsystem`, `value.number`) | **SIM** — append |
| `crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs` + `hero/paint.rs` | os nove chips da barra do grafo dizem o que fazem | **SIM** — só rótulo/dica |
| `crates/ph2d-panel-motion-params/` | `ParamRow::File`, `MotionParamIntent::PickFile`, `row_state.rs` (split de LOC) | **SIM** — variante nova + split |
| `shells/desktop/src/render_loop/motion_bridge_*` | o resolvedor do diálogo, o `bake`, o `mark_lsystem_custom`, o `drop_preset_drivers`, `motion_bridge_params_text_rows.rs` (split de LOC) | **SIM** |
| `shells/desktop/src/motion_state_demo_router.rs` | cenas **`=107`** e **`=108`**; `MAX_DEMO_LEVEL` **107 → 108** | ⚠️ **NÚMERO QUE SOMA** — ver §3 |
| `crates/ph2d-node-motion-sub-uv/` | `Effect::Pure` → **`Effect::Temporal`** + os *holds* | ⚠️ **não é aditivo** — ver §5 |

---

## 3. Símbolos que podem COLIDIR — saída de `collision-surface.sh`, colada

⚠️ **Isto é REFERÊNCIA, não evidência.** Mede a linha contra o `main` de **2026-08-29**. Se
outra linha for fundida antes desta, toda a coluna «base» muda — **re-rode
`collision-surface.sh` em cada worktree imediatamente antes de fundir** (§1.5.3), e use esta
tabela só para saber *o que a linha ACHAVA que estava a tocar*.

```
SUPERFÍCIE DE COLISÃO — line/motion-value contra main
  merge-base 330582deb   ·   43 commit(s)   ·   167 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                         99   (base: 99)
      └ tripla do gate               (99, 13, 14)   (base: (99, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  78   (base: 78)
    ph2d-script (espelho)                  78   (base: 78)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR
    último no disco: 0167   próximo livre: 0168
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 2 pacote(s) novo(s): "ph2d-node-source-lsystem", "ph2d-node-value-number"
▸ MARCADORES DE CONFLITO — nenhum
```

⭐ **Nenhum schema se mexeu.** Nada nesta linha toca o formato do projeto.

### Os números que SOMAM entre linhas (o que outra linha pode ter escolhido igual)

| símbolo | valor desta linha | onde |
|---|---|---|
| `MAX_DEMO_LEVEL` | **107 → 108** | `shells/desktop/src/motion_state_demo_router.rs:26` |
| cena de demo | **`=107`** (sujidade na lente) e **`=108`** (L-System) | o mesmo roteador |
| cena de objeto | **`PH2D_MOTION_OBJ_SMOKE=11`** (o ritmo) | `motion_object_smoke_holds.rs` |
| crates novas | `ph2d-node-source-lsystem`, `ph2d-node-value-number` | `Cargo.lock` + `registry-init` |

⚠️ **O número da próxima cena CONTA-SE lendo o roteador**, nunca uma nota. Se outra linha
também acrescentou cenas, o `MAX_DEMO_LEVEL` e os níveis **têm de ser recontados** — o gate
`no_two_smoke_scenes_claim_the_same_level` mede o **piso**, não o teto, então duas cenas com o
mesmo número **não** o acordam.

---

## 4. Contratos congelados encostados: **NENHUM**

Prova por grep (vazia):

```
git diff --name-only main...HEAD | grep -E 'ph2d-nodegraph/src/node.rs|ph2d-editor-core/src/tool.rs|ph2d-vector-doc|ph2d-vector-traits'
```

`NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool` intocados. **Nenhum ADR é
exigido, e a linha não cria nenhum** — ela está fora da disputa pelo número `0168`.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **Duas crates novas** (`ph2d-node-source-lsystem`, `ph2d-node-value-number`) ⇒ `cargo-machete`,
  `cargo-deny` e `cargo-audit` só as vêem no ship. As duas declaram `license.workspace = true`
  (a lição de `330582deb`: uma crate nova sem licença é um `✗` que só o ship vê).
- **`fmt` da árvore inteira** — corri `cargo fmt --all`, mas um `fmt` pré-fork noutro arquivo
  aparece no ship.
- **RUSTSEC** — nenhuma dependência EXTERNA nova; as duas crates só puxam caminhos internos
  (`ph2d-expr`, `ph2d-expr-parse`, `ph2d-nodegraph`).
- ⚠️ **`motion.sub_uv` deixou de ser `Effect::Pure` e passou a `Effect::Temporal`.** Não é
  aditivo: a impressão digital do memo passa a incluir o playhead, logo aquele nó **recozinha
  por quadro**. Foi a cura de um defeito que shipou desde que o nó existe (ele lia
  `ctx.playhead()` e declarava-se sem tempo ⇒ congelado). O custo não foi medido sob carga de
  cena cheia — **vale um olho no ship**.

---

## 6. Ordem, dependências e o que SMOKAR

### Ordem

Os 45 commits são **sequenciais e independentes de outras linhas**. Não há dependência de
ordem *entre* eles além da natural (a crate nasce antes de ser registada). O `Cargo.lock` tem
**três** commits próprios (`9bbc74f85`, `258c867f1`, e o da `sub-uv`) — se o lock conflitar,
regenere-o em vez de resolver à mão.

### O que o Enio JÁ smokou e aprovou

| smoke | veredito dele |
|---|---|
| `PH2D_MOTION_OBJ_SMOKE=11` (o ritmo) | *"Anim smoke OK"* |
| `PH2D_GPU_COOK_DEMO=108` (L-System) | quatro rondas de report, todas curadas — ver §7 |

### O que **NÃO** foi smokado (⚠️ a lista honesta)

- ⛔ **A última wave (`Growth`, o fio do molde, os três splits de LOC) NÃO foi smokada.** O
  binário foi construído às 13:42 e o Enio pediu o handoff antes de correr. **É o primeiro
  smoke da próxima janela.**
- ⛔ **O botão de escolher ficheiro do `audio.bands`** — o diálogo abre pela shell e nunca foi
  exercido com um ficheiro real.
- ⛔ **A cena `=107`** (sujidade na lente) não foi re-smokada depois da última mexida no glow.
- ⛔ Os gates de GPU (`gpu_cpu_parity_holds`) são `#[ignore]` e correram **em hardware real**
  no dia em que nasceram, não depois.

### Os comandos, inteiros

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=108 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_MOTION_OBJ_SMOKE=11 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=107 cargo run -p ph2d-host-desktop --release
```

---

## 7. A JORNADA — o mecanismo de cada wave (é isto que o `§5` NÃO recebe)

### 7.1 O `source.lsystem` nasceu, e depois foi auditado quatro vezes pelo dono

Nó novo: **paramétrico** (§1.10 do ABOP), **estocástico** (§1.7), **sensível a contexto**
(§1.8), tropismo (§2.3.2) e **gerações fraccionárias**. Sai no contrato de colunas do `rig.*`,
e `source.lsystem → rig.fk` é a **identidade ao bit**.

Depois o dono do produto smokou **quatro vezes**, e cada report tinha uma causa própria:

| report | causa MEDIDA |
|---|---|
| *"não há nenhuma animação"* | eu mandei-o para uma cena **sem relógio** (o `speed` da `=9` é `0`) — e, por baixo, o `motion.sub_uv` estava **congelado desde que existe** |
| *"a cada ramo tudo se apaga"* | a fracção não tinha **sujeito**: a gramática reescrevia o próprio símbolo que desenha |
| *"o tronco pisca uma vez"* | o LFO da cena levava o `Generations` a **zero** — zero gerações é o axioma por derivar |
| *"as shapes não rotacionam"* | **um nome com dois donos**: o desenho lê `rot` como ângulo de MUNDO, o `rig.*` como LOCAL |

### 7.2 A auditoria multiagêntica (a pedido dele)

**49 agentes** — 8 lentes independentes, um cético por achado instruído a **derrubá-lo**,
síntese e um crítico de completude. **39 achados julgados, 36 de pé, 3 refutados.**

As famílias que ela achou, e as curas:

1. **Um molde não é um TEXTO.** `apply_lsystem_preset` escrevia dois `set_text_param` e mais
   nada. Medido: `maior/step` ia de **2,7** (Tree) a **2 581,8** (Koch) — **963×** entre dois
   itens do mesmo selector, com uma coluna da cena a ter ~4 unidades de mundo. ⇒ `PRESETS` vira
   `&[Preset]` com `angle` · `generations` · `step` · `width` · `reads`, os quatro **contados**
   (o alvo é a mediana dos que o dono já aceitou, `3,522`).
2. **O parser falhava ABERTO** em dois dos três sub-campos de uma regra: uma condição que não
   compila evaporava (medido: `n <= 6` dava **16 384** módulos contra **32** do `n < 6` —
   **512×**, byte-a-byte «sem condição»), e um peso ilegível virava o **neutro** e ia
   DESENHAR (`(40%)` apagava a planta, porque `%` é o corte). ⇒ uma política, a do predecessor.
3. **O `preset` era uma ACÇÃO guardada como ESTADO** — o `build` nunca o lê, e o estado de
   chegada normal deixava o selector a dizer «Tree» sobre uma planta **76 % mais alta**, com o
   clique mudo. ⇒ `PRESET_CUSTOM`, e todo escritor de texto aterra nele.
4. **As réguas mediam o INSTRUMENTO.** O único gate por-molde era `count() > 3` — *uma contagem
   é a única grandeza que SOBE com este defeito*. A bancada que media tudo isto já existia como
   `example` e **nenhum portão a corria**.

### 7.3 O crescimento — três rondas, três causas

| ronda | report | o que eu media | a causa real |
|---|---|---|---|
| 1 | *"não é linear"* | o pior passo | as gramáticas separam-se em duas famílias (razão que **converge** vs **constante**), e eu tinha construído só a lei da primeira |
| 2 | *"melhorou muito, mas não é linear"* | uma travessia de geração | as duas rampas **brigavam**: o comprimento crescia enquanto as dobras, ao abrir, **encurtavam** a projecção — o último quinto do slider andava **para trás** |
| 3 | *"ainda não linear"* | o percurso todo | cada geração **multiplica** a figura ⇒ o arrasto é exponencial |

**As curas, todas medidas:**
- a **âncora**: a geração nova nasce por cima da anterior. ⛔ Ela **não se conta da gramática** —
  `F -> F[+F]F[-F]F` põe **5** módulos e cresce `3,00×`; `F -> F+F-F-F+F` põe 5 sem parênteses
  e cresce `3,00×` na mesma. *A razão é geométrica e mede-se.*
- a **normalização por instante**: escolhe-se o comprimento que põe a figura na rampa recta.
  Ondulação **`0,0×`** nas quatro.
- o param **`Growth`** (0..1, default **1,0 = no-op EXACTO**): remapeia as gerações resolvendo
  `r^g = r + t·(r^G − r)`.
  ⛔ **CORRECÇÃO de 2026-08-30: dois dos quatro números do arrasto abaixo já não reproduzem.**
  A régua da lei mudou (passou a ser invariante à rotação — ver o handoff de 30/08) e a janela
  de amostragem também. Medido hoje: Bush `1,84×` ✅ · Koch `1,85×` ✅ · Weed **`1,33×`** (o doc
  dizia `2,1`) · Dragon **`1,07×`** (dizia `3,7`). *Nenhum instrumento da árvore reproduz hoje
  os dois últimos.* Arrasto (redacção original): Bush `1,8 → 1,0` · Weed `2,1 → 0,7` · Koch `1,9 → 1,0` ·
  Dragon `3,7 → 1,9`, e os que crescem pela ponta **intactos**.
- o **fio que fazia um molde sair 10× mais pequeno**: o `EvalCtx::param` resolve o conduzido
  primeiro, então um fio no `Generations` ganhava ao número do molde. ⇒ aplicar um molde solta
  os fios dos **quatro** números do enquadramento (e só desses), com toast.

**Preço medido:** pior caso **0,615 ms (3,7 % de um quadro)**, e só abaixo de `Growth = 1`.
No tecto (`262 145` módulos) a fracção custa **`1,0×`** a geração inteira — ela satura e não paga.

### 7.4 ⛔ RECUSAS MEDIDAS — não as reconstrua

| recusa | mecanismo |
|---|---|
| **inventar uma sintaxe amigável** para a gramática | `F[+F]F` é a notação de Lindenmayer — trocá-la torna o nó incompatível com o que se copia de qualquer tutorial |
| **reescrever Koch/Dragon/Bush em forma paramétrica** (para ganharem `!` e `"`) | o mesmo motivo; o preço declarado é o campo `reads`, que **esconde** os knobs que aquela gramática não lê |
| o **`Step Scale`** sozinho como cura do salto | estabiliza o TAMANHO (`1/3` é a razão do Bush; ⛔ **a Koch é `3,0028` desde que a régua mudou em 2026-08-30**, logo «exactamente» já não vale para ela) e o melhor que uma varredura de oito valores alcança é **`105 %`** de pior passo. *Estável em tamanho ≠ contínuo na forma* |
| a **âncora como constante** contada da gramática | dá `5` onde a resposta é `3` — ver 7.3 |
| acrescentar **`>=`/`<=`** ao `ph2d-expr-parse` como cura da condição que evapora | ele é o parser partilhado do ADR-0144 (a timeline também o usa), e a condição evaporava **por igual** quando estava vazia ou truncada. O defeito era a política de erro |
| **normalizar a saída** para caber no ecrã | um *fit* divide o `step` para fora e mata o slider — a faixa passaria a ser derivada do objecto que ela reescreve |
| medir um molde contra a **coluna da cena `=108`** | a cena tem tabela própria (`PLANTS`) e **nunca instancia um `PRESET`** |

### 7.5 As LEIS que esta linha pagou (as que valem fora dela)

1. **Uma régua normalizada pelo que a cura leva a zero mede a cura ao contrário** — imprimiu
   `619 050 %` e recomendou não curar.
2. **Comparar duas medições com denominadores diferentes inventa um efeito** — o meu
   `69 % → 138 %` não existia; os dois números vinham de réguas diferentes.
3. **Uma barra COMPARATIVA cujo referencial sai do mesmo dado é auto-referencial** — três
   mutações sobreviveram porque a sabotagem **subia a barra junto**.
4. **`max(w, h)` tem um JOELHO** onde os eixos se cruzam, e o joelho lê-se como
   não-linearidade do produto.
5. **Uma sonda com o default cravado mede o que ela acha que o produto é** — a bancada
   continuou a imprimir os números curados depois de eu desligar a feature.
6. **Uma recusa MEDIDA e um veredito de PRODUTO são coisas diferentes** — vestir um do outro é
   publicar uma medição falsa.
7. **Um gate que classifica por conta própria testa a sua própria classificação.**
8. **Um split re-atribui todo atributo que ficar do lado errado do corte** — `#[must_use]` e
   `#[allow]` colam-se ao item seguinte, e um doc-comment entre eles não os protege.

---

## 8. Provas de mutação

| wave | sabotagens | mortas | sobreviventes que forçaram reparo |
|---|---|---|---|
| modo guiado | 9 | 9 | 1 (a métrica de fracção media «monotonia», e um salto satisfaz isso) |
| moldes + parser | 11 | 11 | 2 |
| âncora | 6 | 6 | 1 (a barra de «melhorou 2×» não distingue duas âncoras) |
| `Growth` | 5 | 5 | 4 (a barra auto-referencial) |
| fio do molde | 3 | 3 | — |

⚠️ Duas mutações ficam registadas como **EQUIVALENTES**, não como sobreviventes: aceitar peso
`0` (o intervalo do sorteio é vazio) e `growth < 1.0` → `<=` (mesmo desenho ao bit, mas paga
**três derivações à toa** — um defeito só de CUSTO, invisível a todo gate de saída).

---

## 9. O PORTÃO DE FECHO — o que ele apanhou

| | |
|---|---|
| `cargo nextest run --workspace --no-fail-fast` | **19 658 testes** · `19 656` verdes · as 2 ✗ são flakes catalogadas (ver abaixo) |
| `cargo clippy --workspace --all-targets` | limpo (**depois** de curar 5 avisos que os splits criaram — ver abaixo) |
| gates de LOC (os três) | verdes |

⚠️⚠️ **E o clippy da WORKSPACE apanhou uma quarta coisa que as corridas por-crate não viam** —
cinco avisos, todos do mesmo mecanismo e todos criados pelos splits de LOC desta mesma sessão:
**um atributo separado do seu item por um doc-comment MUDA DE DONO**. Ao mover
`any_param_editing` para `row_state.rs`, o `#[must_use]` e o doc dele ficaram órfãos e
aterraram no `paint_scroll_chrome`, que passou a exigir que o valor de retorno fosse usado — e
como ele devolve `()`, o clippy queixou-se dos DOIS lados. O mesmo aconteceu ao
`#[allow(too_many_arguments)]` do `paint_rows`, que ficou no meu helper novo.
*Um split não move só linhas: ele re-atribui todo atributo que ficar do lado errado do corte.*

⚠️⚠️ **TRÊS vezes hoje a suíte SEM FILTRO apanhou o que as minhas corridas não alcançavam:**

1. **Três censos da shell** — `motion.sub_uv::holds` sem declaração de alcance, `audio.bands::file`
   órfão no `ALWAYS_READ` (o censo não contava `ParamWidget::File`), e o censo de `ParamGroup` a
   acusar `axiom`/`rules` (ele media o snapshot de UM estado, e um param gateado não aparece lá).
2. **Três tetos de LOC** (`lib.rs` do L-System a **1475/700**, o painel a 614/600, `paint_rows` a
   215/200) — eles vivem nos alvos do `ph2d-editor-core`, e eu tinha corrido a suíte da **shell**
   inteira sem filtro. *Uma corrida sem filtro num crate não é uma corrida sem filtro na workspace.*
3. **O gate do `audio.bands`** afirmava `ParamWidget::Text` para o `file` — a wave do botão
   trocou-o e nenhuma corrida por-crate o alcançava.

⛔ E o gate batched apanhou os três tetos e **CANCELOU 12 598 testes** no primeiro ✗ (nextest é
fail-fast por omissão). **Corra sempre com `--no-fail-fast`.**

### Vermelhos que NÃO são desta linha — e a prova de que são FLAKES

Duas corridas da workspace inteira, e ⭐ **o CONJUNTO de reprovadas MUDOU entre elas** — que é
precisamente a assinatura que o `CLAUDE.md §5.0` dá (*«um defeito de lógica reprova o mesmo caso
sempre»*):

| corrida | reprovadas | diff toca? | sozinha |
|---|---|---|---|
| 1ª (`19 658`) | `ph2d-audio-edit::…::the_trusted_len_collect_allocates_once` | **não** | **3/3 verdes** |
| 2ª (`19 658`) | `flip_smooth::…::orcamento::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` + `…::a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` | **não** | **3/3 verdes** |

As duas famílias estão **nomeadas no `CLAUDE.md §5.0`**: a de ALOCAÇÃO (*«um contador de
alocações parece imune a carga e não é: sob fan-out o alocador global reutiliza arenas de outra
maneira»*) e a `flip_smooth::resample_measurement::precisao::orcamento` (medida em 22/08 pela
`line/3DModeling` e confirmada em 23/08 pela `line/sculpt3d`, *«com a falha a MUDAR de teste
entre corridas»*).

⇒ **`19 656 / 19 658` verdes, e as 2 são flakes de recurso sob fan-out.** ⚠️ O integrador deve
esperá-las e **re-rodar sozinho antes de olhar para o merge**.

---

## 10. A UMA LINHA para o `CLAUDE.md §5` (a escrever **na integração**, no primário)

> Na entrada **Motion Nodes**, na lista de **Aberto**, acrescentar:

```
⭐⭐ **O `source.lsystem` existe** (nó novo, ABOP completo: paramétrico · estocástico ·
sensível a contexto · tropismo · gerações fraccionárias) com **modo GUIADO por omissão** (a
gramática é DERIVADA de sliders e o `Mode` assa-a no texto ao converter), oito moldes que
carregam o próprio **enquadramento** (ângulo/gerações/passo/espessura, todos CONTADOS) e o
param **`Growth`** (0..1, `1` = no-op ao bit) que faz o arrasto crescer por igual — cena
**`=108`**; ⛔ o `Grow Angle` e a `Data Source` (CSV/JSON) seguem ABERTOS
([handoff](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-08-29.md)).
```

⛔ **NÃO acrescente um parágrafo de jornada** — ela está toda aqui, no §7.

---

## 11. O que fica ABERTO da linha

- ⏳ **Data Source (CSV/JSON)** — o terceiro item do [plano 93](../93_plano_lsystem_datasource_celanim.md),
  nunca começado. O `ParamWidget::File` que ele precisa **já existe** (foi construído para o
  `audio.bands`), e o plano §4 tem as quatro decisões de produto sem precedente.
- ⏳ **`Grow Angle` para Bush/Weed** — a lei existe e está medida; o que falta é o veredito do
  dono depois de smokar a última wave.
- ⏳ **Feedback ao vivo de uma regra malformada** — hoje ela é descartada em silêncio (o parser
  falha fechado, mas nada diz ao artista *qual* regra caiu).
- ⏳ **A legenda do alfabeto** no painel (`F`, `+`, `[`, `!`, `%`, `J`) — proposta e não construída.
