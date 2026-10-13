# Macros & Automação — plano da onda 1 (2026-10-13)

> Núcleo comum + Painter (Wet Paint) + Motion. Decisões do dono e levantamento do código em
> [00_estudo_inicial.md](00_estudo_inicial.md). Base `757afebd58`. Nada de produto implementado ainda; as
> duas SONDAS de medição estão no commit `84523b5c6e`.

## 1. Estado da arte — o que fizeram, e o que ABANDONARAM

| Quem | O que faz | O que falhou / foi abandonado | Lição que tomamos |
|---|---|---|---|
| **ArtRage** `.arscript` | grava por ponto `Wait/Loc/Pr/Ti/Ro`, eventos de ferramenta; repinta noutra resolução; `WaitResumeAtMouse()` | pincel com aleatoriedade sai «ligeiramente diferente»; versões incompatíveis | semente explícita; versão com leitura tolerante |
| **Corel Painter** | Record/Playback Stroke; scripts com «Initial State»; independência de resolução por RETÂNGULO de referência | sem o retângulo o replay cai no canto; proporção diferente distorce | quadro de referência gravado, política de escala declarada |
| **MyPaint** | traço = settings + **estado interno do pincel** + `(dtime,x,y,p,tilt)` | — | o estado inicial é parte da gravação |
| **Krita** | — | **removeu** o gravador de macros: «código extra em cada coisa gravável» era beco sem saída | gravar nos FUNIS que já existem, não ferramenta a ferramenta |
| **Blender** | macros de operadores fixos (2.5); Info/Replay | modal só no fim; x/y do rato inutiliza a macro; select não leva o alvo; propostas GSoC 2009/2013 de replay sem seguimento; add-on não garante história com undo | comando gravado = comando RESOLVIDO (alvo por id, sem contexto implícito) |
| **Illustrator / Photoshop Actions** | gravam menus e diálogos | **o gesto de desenho não é gravável** (Pen, Pencil, Brush); contornam inserindo o CAMINHO pronto; pena perdeu as curvas | o que não é gravável é RECUSADO em voz alta, nunca omitido |
| **After Effects** | Motion Sketch grava a posição com a velocidade do gesto → keyframes | só Position; a suavização alta «perde a forma»; sem gravador de macros (só ExtendScript à mão) | bruto guardado ao lado do processado; razão de velocidade é parâmetro |
| **Unreal Take Recorder / Unity Recorder** | amostram propriedades por quadro | Unity: **só no editor, não em builds**; suaviza descontinuidades «por design» | eventos discretos e amostras contínuas em trilhas separadas; o tocador viaja para o jogo |
| **Rive** | o MESMO renderer no editor e no runtime; `.riv` com major/minor, campos tipados (desconhecido = salta), chave 0 terminadora | major novo = re-exportar | leitura tolerante dentro da major, recusa com mensagem fora dela |
| **Unity IET / Godot Tours / Adobe Discover** | tutorial = passos com **critério sobre o ESTADO**, reavaliado no undo, realce na UI | — | o modo «fazer» valida estado, não gesto |
| **Grease Pencil Build / Chicory** | revela pontos na velocidade gravada; tinta como DADOS | — | o nível «assado» é uma revelação barata |
| **SC2 / Overwatch / CoH2 / Factorio** | replay = entrada sobre sim determinística | patch apaga/recusa replays; 1 tick de desvio dessincroniza | marca de passo EXATA; no jogo, a macro vai com o motor da mesma build |

## 2. O desenho — uma porta por pergunta

**Peças:**
- **`crates/ph2d-macro`** (nova, sem dependência de UI): `MacroDoc` (formato), `codec` (leitura tolerante),
  `MacroRecorder` (escuta), `MacroPlayer` (relógio + despacho), `TrackKind` (registo das famílias, ECS),
  `StepGate` (portões de tutorial).
- **Trilha do Painter** em `ph2d-tool-painter` (gravação nas portas + marca da água) e o tocador em
  `ph2d-app-painter` (motor PRÓPRIO por objeto, sem tocar na ferramenta ativa do editor).
- **Trilha do Motion** em `ph2d-app-motion` (gravação nos dois drenos de intenção + tocador sobre um
  «palco»: cópia do `MotionDoc` do objeto; o documento autorado não é tocado).
- **Componente `Macro`** em `ph2d-ecs` + ponte/cartão em `ph2d-app-components`; o conteúdo viaja como
  BLOB versionado (molde `TimelineDoc`).
- **UI**: painel `ph2d-panel-macros`, cartão do Inspector, clip na Timeline, chrome do tutorial.

| Pergunta | A porta ÚNICA |
|---|---|
| Que passo aconteceu? | `MacroRecorder::push(track, step)` — o único escritor da gravação |
| Quando? | o **relógio da macro**: tique da cena (Motion) e ms da gravação + marca da água (Painter). O tocador NUNCA lê o relógio de parede |
| Quantos passos de água entre duas ações? | **a fita do worker** (W2, `crates/ph2d-tool-painter/src/tool/paint/wetpaint/fita.rs`): cada resposta da thread da água — voltou em `(sim.frame, estágio em voo)`, não voltou, o contador — gravada nas portas do `WetSession`, pela ordem em que se pedem; o tocador anda o motor até cada marca. ⛔ **Não** o `sim_steps()` sozinho: ele não vê passos fechados por outra porta (medido: frame 34 × contador 33) |
| Que semente tinha o traço? | `paint.seed` no ESTADO INICIAL (W2): ela avança por gesto de forma determinística, então a de armar basta (gate `sem_a_semente_ou_sem_um_tique_a_reproducao_muda`) |
| Onde fica o traço noutro canvas? | uma função de normalização pelo **retângulo de referência** gravado (Corel) |
| O que é gravável? | declaração por porta (`Recordable` / `Refused{why}`) + censo que a reprova sem decisão |
| A macro toca agora, e onde? | a Timeline: um clip com dono (`NamedClip.owner`, Autoplay) e o `playhead` (`crates/ph2d-core/src/playhead.rs`) |
| O passo do tutorial foi cumprido? | `StepGate::check(&estado)` — reavaliado no undo |
| Este ficheiro lê-se? | `ph2d_macro::codec::read` — major diferente = recusa com frase; campo desconhecido = salta |

**Formato** (`MacroDoc`, postcard com cabeçalho `PH2DMACRO` + major/minor, campos com etiqueta):
retângulo de referência · estado inicial por trilha (preset/meio/cor/papel/semente; grafo inicial) ·
trilhas `{ família, alvo StableId, eventos }`, cada evento `{ t, carga }` — **eventos discretos e
amostras contínuas separados**: as amostras de um traço vivem DENTRO do evento do traço e a
interpolação nunca atravessa um evento.

**Painter — o que se grava** (a fronteira medida tem várias portas, não um funil):
`on_canvas_pointer` (pos normalizada, pressão, fase, t) · `on_tick(dt)` (o Airbrush e a secagem do
Watercolor dependem do `dt`: `stroke.rs:536`, `watercolor_dry.rs:23`) · `handle_panel_event`
(`trait_impls.rs:50`) · `apply_layer_edit` · `undo_last/redo_last` (gravados COMO passo — exatidão
primeiro; «limpar desfeitos» é uma transformação posterior, com gate) · os setters diretos
(`set_line_constrain`, `set_canvas_warp`, `set_wet_*`). A marca da água entra em cada traço.

⛔ **Recusada na W2: a «lei nova» de drenar o passo em voo no 1.º lote molhado** (o desenho da W1). Ela
mudava a pintura de todos para a gravação caber — até um passo de espera no toque (`39,5 ms` a 4096²) — e
não cobria a Água ao vivo nem o knob que o tique reconcilia com a água a correr. A fita do worker é exacta
por construção e não muda lei nenhuma ([ADR-0186 §Emenda W2](../architecture/decisions/0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md)). O mecanismo
medido continua verdade: o dab cai a meio do passo quando o toque apanha o worker A CORRER um (ele dorme
entre passos) — é esse caso que a fixture do gate 1 exige.

**Motion — o que se grava:** as intenções que mudam o documento (lista em
`crates/ph2d-panel-motion-graph/src/snapshot_intent.rs:26`; as de vista ficam de fora), **já
RESOLVIDAS**: alvo de `SetParam` depois de `apply_param_edits` (`motion_bridge_params_edit.rs:47-103`,
que hoje desvia para o cartão selecionado), ids de vista decodificados (`subgraph::target`), nós
nomeados pela ORDEM DE CRIAÇÃO. Cada passo leva o **tique da cena** em que aconteceu.
Reprodução em dois modos: **fiel** (aplica no tique gravado — reproduz o que o artista viu) e
**explicativo** (re-simula do tique 0 após cada passo — mostra o efeito real de cada nó; `0,7 ms` por
passo medido). O tocador corre os DOIS drenos na ordem do quadro e o `publish_all` antes de cozinhar.

✅ **Feito na W3** ([ADR-0186 §Emenda W3](../architecture/decisions/0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md)).
O «tique da cena» grava-se como o RELÓGIO do quadro: o tempo do playhead, se toca e o `dt`, um passo por
quadro. O que muda o documento por fora dos drenos (desfazer, gizmos, o conta-gotas) grava-o uma
testemunha: as escritas de params, ou o documento inteiro. O anel de checkpoints carrega história, e o
palco repete cada limpeza dele.

**Tocador no jogo:** não há player exportado; hoje o jogo é o Play do PH2D (mesmo `SimWorld`,
`playhead.is_playing()`). O tocador dirige só os motores (Painter headless por objeto, palco do
Motion) — nenhuma dependência da shell nem da ferramenta ativa — para viajar com o export quando nascer.

**Tutorial:** é uma macro cujos passos podem ter um portão. **Assistir** = o editor reproduz pelas
MESMAS portas da ferramenta ativa (o aluno vê o painel mudar e o traço nascer). **Fazer** = o passo
mostra o balão + o realce (`rect_for(id)`, `crates/ph2d-editor-core/src/interaction/hit.rs:238`) e
espera `StepGate` (ferramenta ativa · nó do tipo X ligado a Y · área pintada ≥ p% numa região ·
parâmetro numa faixa). Texto dos tutoriais do PH2D por `ph2d-i18n`; o texto que um utilizador escreve
no seu tutorial é CONTEÚDO do documento, não UI.

## 3. Contratos congelados e schema — prova por grep

- **Tools** (`Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`,
  `crates/ph2d-editor-core/src/tool.rs:26,63`): a gravação vive em métodos **inerentes** do
  `PainterTool` e nos chamadores; nenhum método novo no `trait Tool`, nenhuma variante em `PanelEvent`.
  Gate `architecture_tool_contract_surface` fica como está.
- **Nodes** (`NodeOp≤2`/`OpResolver≤1`/`NodeManifest≤8`,
  `crates/ph2d-nodegraph/tests/it/architecture_contract_surface.rs:30,43,64`): a macro lê intenções e
  chama a API pública de `Graph`; não persiste nada na gramática `v1` (`format.rs`). Não encosta.
- **Vector** congelado: fora do escopo da onda 1.
- **Schema:** o componente `Macro` registado sobe `PROJECT_SCHEMA` **188 → 189** (`main` hoje: `188`,
  `shells/desktop/src/project_schema.rs:441`) nos três sítios — **contado na integração**
  (`scripts/schema-recount.py`), nunca escolhido. O `MacroDoc` tem versão PRÓPRIA (molde `DOC_VERSION`).
- **ADR:** próximo livre `0186` (o maior hoje é `0185`) — «a macro grava comandos resolvidos com o relógio
  da macro» — contado na integração.

## 4. UI — as quatro condições, uma a uma

| Controlo | Existe | Pintado + registado | Clique chega ao barramento | A SEQUÊNCIA leva a algum lugar |
|---|---|---|---|---|
| **Gravar / Parar** (painel Macros) | painel novo `ph2d-panel-macros` (receita §3.B.1, `ph2d-panel-sync`, ICON, DEFAULT_SLOT) | ids em `ph2d-panel-macros/src/ids.rs`, `panel_seam!` | evento → `MacroRecorder::arm/stop` | Parar cria/atualiza o componente `Macro` no objeto e o clip na Timeline |
| **Lista de passos** (scrub, apagar passo, marcar portão) | widget de lista | hit-index por linha | seleção → playhead; apagar → transformação com gate | o passo marcado ganha balão e critério |
| **Cartão Macro no Inspector** (Autoplay, velocidade, laço, modo fiel/explicativo, assistir/fazer) | molde Motion Input (`323e2f75c5`) | Inspector + i18n `component_catalog.rs` | edição do componente | o Play passa a tocar como o cartão diz |
| **Clip Macro na Timeline** | clip com dono (`NamedClip.owner`) | o clip existente | play/pausa/scrub do `playhead` | o tocador segue o playhead, incluindo para trás (instantâneos) |
| **Balão + realce do tutorial** | chrome novo `z=NN` (`ph2d-chrome-sync`) | por cima de tudo | «Mostrar-me» / «Seguinte» | portão cumprido → próximo passo; desfazer → volta |

## 5. Gates — vermelhos primeiro, e a fixture com o fenómeno

1. **Wet Paint reproduz a sessão do worker AO BYTE.** Fixture: 3 traços molhados a 1024² com esperas
   reais, no modo worker, repetidos até apanhar um passo em voo. **Hoje vermelho** (medido: 24 239 texels
   entre corridas; 29 891 na reprodução sem drenagem). Verde = 0 texels, também após +40 passos.
   Controlo positivo: um passo a menos ⇒ ≠ 0 (medido 9 912 / 35 267).
2. **Semente e `dt`.** Traço com jitter e Airbrush com tiques irregulares reproduzem ao byte;
   vermelho se a semente ou um tique não forem gravados.
3. ✅ (W3: 150 quadros ao bit; sem os tiques `0,87`) **Motion fiel = o que o artista viu.** As 24 intenções da sonda tocadas nos tiques gravados = a
   sessão original; vermelho sem os tiques (medido: 3,59 unidades de desvio). **Explicativo** = a
   construção de uma vez, ao bit (medido: igual).
4. **Gravabilidade.** Censo: toda variante de `GraphIntent` que muda o documento e toda porta pública do
   `PainterTool` está `Recordable` ou `Refused{why}`; uma nova sem decisão reprova.
5. **Formato.** Fixture `v1.0` com campo desconhecido lê; major diferente recusa com a frase i18n;
   ida e volta ao byte.
6. **Resolução.** O mesmo traço num canvas 2× cai na mesma posição relativa.
7. **Costura (ph2d-ui-testkit):** Gravar → pincelar → Parar ⇒ componente `Macro` no objeto e clip na
   Timeline; Play ⇒ os passos avançam; tutorial «fazer» cumpre-se pelo estado e regride no undo.
8. **Catálogo e schema:** `every_registered_component_is_described` + a tripla 188 → 189.

## 6. Cenas de smoke — números MEDIDOS (sondas de `84523b5c6e`, mínimos; carga alta, loadavg 11–46)

| | mín. | leitura |
|---|---|---|
| Passo da água 1024² (só o passo · passo + composite) | `1,67` · `4,04 ms` | a 40 Hz cabe folgado num quadro de 16,7 ms |
| Passo da água 2048² | `8,08` · `29,7 ms` | ⚠️ ao vivo a 60 Hz **não cabe** com composite por passo |
| Passo da água 4096² | `39,5` · `107 ms` | ⚠️ idem |
| Reproduzir um traço de 60 amostras 1024² (total · pior quadro) | `14,5` · `0,89 ms` | folgado |
| Motion: aplicar + cozinhar um passo (8 nós, 36 formas) | `≤ 0,02 ms` | folgado |
| Motion: re-simular do tique 0 ao 60 por passo | `0,7 ms` | folgado |

⇒ **O composite custa 2,4–2,7× o próprio passo.** Para 2048²+ ao vivo, compor UMA vez por quadro
(não por passo) entra nesta onda e mede-se antes de qualquer tecto; o nível «assado» fica como saída
para o jogo publicado.

- `PH2D_MACRO_SMOKE=1` — **Splash**: objeto com imagem transparente 1024² e componente Macro (Autoplay);
  no Play, 3–5 pinceladas de Wet Paint pousam e a água escorre entre elas.
- `PH2D_MACRO_SMOKE=2` — **Motion nó a nó**: formas + simulação com vento num campo; o grafo monta-se um
  nó de cada vez e cada ligação muda o que se vê (modo explicativo). Fotografada antes de ir ao dono.
- `PH2D_MACRO_SMOKE=3` — **Tutorial «fazer»**: balão + realce, espera o aluno ligar o nó; desfazer volta o passo.

## 7. Ordem de trabalho (uma janela por onda)

W1 núcleo `ph2d-macro` + ADR + gates 4–5 · W2 trilha Painter + lei da drenagem + gates 1, 2, 6 ·
W3 trilha Motion + palco + gate 3 · W4 componente, blob, Timeline, Play, schema + gate 8 · W5 UI
(painel, cartão, clip, chrome do tutorial) + gate 7 + smokes 1–3.

## Abertos (não decididos pela medição)
- Memória dos **instantâneos** para scrub para trás na água (a grelha a 1024² por instantâneo): medir
  antes de fixar o intervalo.
- O cozimento do Motion na GPU (`PH2D_GPU_COOK`, padrão) na reprodução: a sonda mediu a CPU.
- Re-medir as tabelas a carga baixa antes do smoke de performance.
