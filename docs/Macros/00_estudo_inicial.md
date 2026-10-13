# Macros & Automação — estudo inicial (2026-10-13)

> Pedido do dono: **macros** que gravam os passos do artista e os reproduzem **em tempo real**, para que
> o asset NASÇA à frente do jogador (pinceladas do Painter até surgir um splash; nós do Motion a entrar no
> grafo um a um, mostrando a cadeia de eventos) — e **tutoriais em tempo real**. Este doc é o levantamento
> (código do repo + prior art). Nada foi implementado. Decisões de produto pendentes no fim.

## 1. O que o repo JÁ tem (file:line na base `757afebd58`)

| Peça | Estado | Onde |
|---|---|---|
| Barramento de ações do editor, drenado 1×/quadro por ordem de postagem | **existe** — ponto natural de escuta | `crates/ph2d-editor-core/src/action_bus.rs:1-40` |
| Funil de edição do grafo do Motion (`GraphIntent` → `apply_graph_intents`) | **existe** — gravável; ids são de VISTA (subgrafo) | `crates/ph2d-panel-motion-graph/src/snapshot_intent.rs:26` · `crates/ph2d-app-motion/src/motion_bridge_intents.rs:32` |
| Amostra de traço `StrokePoint { pos, pressure }` + ciclo `begin/extend/tick/finish` | **existe, mas morre no motor** — nada a guarda | `crates/ph2d-painter-brush/src/stroke.rs:17` · `crates/ph2d-tool-painter/src/tool/paint/stroke_lifecycle.rs` |
| Porta de entrada do traço sem UI | `on_canvas_pointer(CanvasPointer{pos,pressure,phase})`; o rabisco do smoke já a usa | `crates/ph2d-tool-painter/src/tool/paint/canvas_pointer.rs:11` · `crates/ph2d-app-painter/src/composite_smoke.rs:195` |
| Undo da shell | por **estado** (`ProjectState`), sem tempo nem intenção; `UNDO_CAP=256` | `shells/desktop/src/undo.rs:41,117,190` · `undo_app.rs:334` |
| Undo do Painter | por **pixels** (journal de tiles), 1 entrada por traço | `crates/ph2d-tool-painter/src/undo.rs:275` |
| Projeto do Painter | guarda **pixels**, não traços | `crates/ph2d-tool-painter/src/tool/persist.rs:28` |
| Injeção de clique real (winit) | `smoke_click` — clique, sem roteiro temporal | `shells/desktop/src/build_smoke_drive.rs` |
| Testkit que dirige eventos reais | só em teste (`MockPanelHost`) | `crates/ph2d-ui-testkit/src/lib.rs` |
| Passo fixo / semente | `FixedStep`, `Factory::semear` | `crates/ph2d-core/src/time.rs:34` · `crates/ph2d-ecs/src/factory.rs:127` |
| Marcador da timeline que emite sinal ao ser cruzado | **existe** — serve de gatilho de passo | `crates/ph2d-timeline/src/doc.rs:150` · `signal.rs:26` |
| Script embutido (Luau/mlua) | existe para gameplay; não para macros | `crates/ph2d-script/src/host.rs` |
| Gravador / replay / macro / tutorial no app | **não existe** (nem plano prévio em docs/memória) | — |
| Player de jogo separado do editor | **não existe** — «runtime» hoje = Play dentro do PH2D | — |

### Riscos de determinismo medidos no código
- **Wet Paint**: sim a 40 Hz fixo (`crates/ph2d-wet-paint/src/sim.rs:1-12`), mas o NÚMERO de passos sai da
  dívida do `dt` real (`crates/ph2d-tool-painter/src/tool/paint/wetpaint.rs:356`) ⇒ depende do framerate.
- **Airbrush**: `Stroke::tick(dt)` acumula segundos de parede (`stroke.rs:536-562`).
- **Watercolor**: secagem recebe `dt_s` (`watercolor_dry.rs:23`) — corpo não lido.
- **Jitter/grão**: RNG splitmix64 por traço, semente `paint.seed` incrementada a cada traço — origem e
  persistência da semente **não confirmadas**.
- **Motion**: `mark_dirty` esvazia o anel de scrub e recozinha tudo (`crates/ph2d-eval-motion/src/lib.rs:290`);
  custo de «recozinhar a cada nó novo» **não medido**.

### Contrato congelado
Gravar `GraphIntent` e reaplicar pela API pública de `Graph` **não encosta** no ADR-0039 (cap só em
`NodeOp`/`OpResolver`/`NodeManifest` + gramática `v1`). Encostaria quem mudasse o formato `.ph2d`.

## 2. Prior art (resumo; fontes no relatório da pesquisa, reproduzidas aqui)

- **ArtRage `.arscript`** — único produto de pintura que grava a sessão como DADOS reproduzíveis: por ponto
  `Wait/Loc/Pr/Ti/Ro`, troca de ferramenta como evento, replay em outra resolução, «acelerar» corta o tempo
  parado, `WaitResumeAtMouse()` pausa até clique. Falhas documentadas: pincel com aleatoriedade sai
  «ligeiramente diferente», versões incompatíveis. https://www.artrage.com/manuals/scripts/
- **MyPaint** — traço = settings + **estado interno do pincel** + eventos `(dtime,x,y,p,xtilt,ytilt)`.
- **Procreate / Krita / Clip Studio / Rebelle** — gravam VÍDEO/snapshots, não replay.
- **Krita** removeu o gravador de macros antigo: «código extra em cada coisa gravável» era beco sem saída;
  a fonte única tem de ser dados gerados pela UI. https://phabricator.kde.org/R37:99238df13951e00a9079a5cb3ab71d20a50111a2
- **Blender Info/Replay** falha porque o operador perde o CONTEXTO. https://developer.blender.org/T68656
- **Maya construction history / Houdini HDA / Substance** — o histórico/grafo É o asset; Substance expõe
  parâmetros + **semente** para variações.
- **Grease Pencil Build modifier** — revela pontos na ordem e na **velocidade gravada da caneta**
  (análogo direto do Flip). **Chicory** — tinta como dados numa grelha, não superfícies.
- **Determinismo** — passo fixo é pré-requisito; float na GPU diverge entre drivers; event sourcing usa
  **snapshots a cada N** (gate: replay até N == snapshot N). Factorio: replay = entrada sobre sim determinística.
- **Tutoriais** — Unity IET / Godot Tours / Adobe Discover: cada passo tem **critério sobre o ESTADO**
  (ferramenta ativa, componente adicionado…), reavaliado no undo; realce na UI. Validam estado, não gesto.

## 3. Desenho que o estudo aponta (proposta, a decidir)

1. **Uma gravação = lista de passos com tempo**, cada passo um *comando de domínio* já existente
   (intenção do grafo, traço com amostras, troca de ferramenta/preset), nunca cliques de ecrã nem estado.
2. **Traço guardado como amostras** `(t, x, y, pressão, inclinação)` em coordenadas normalizadas +
   preset + semente — repinta em qualquer resolução.
3. **Reprodução no relógio da macro, nunca no da parede** — curar a dependência de `dt` do Wet Paint,
   Airbrush e Watercolor (o passo de sim atrelado ao tempo da gravação).
4. **Três níveis de reprodução por asset**: (a) simulação completa (bonita, cara) · (b) só a lei do traço,
   determinística em CPU · (c) **assado**: snapshots/máscara de revelação, barato e idêntico em qualquer
   máquina. A mesma gravação alimenta os três.
5. **Snapshots periódicos** para scrub/salto e como gate de determinismo.
6. **Parâmetros nomeados** (cor, tamanho, semente, velocidade) ⇒ variações do mesmo asset.
7. **Tutorial = macro com portões**: cada passo espera um critério de estado, com realce e texto i18n.
8. **Transporte pela Timeline** (play/pausa/scrub/velocidade) e a macro como **componente do objeto**
   (encaixa no modelo «a Animação é do objeto»).

## 4. Decisões do dono (13/10)
- **Toca em tudo e em qualquer lugar, inclusive dentro dos jogos** ⇒ o tocador NÃO pode depender da
  shell/UI do editor: dirige os motores diretamente (painter-brush, wet-paint, nodegraph…) e cada família
  pluga o seu tipo de passo (ECS, drop-crate). Hoje o «jogo» é o Play do PH2D; o tocador nasce pronto para
  viajar com o export quando ele existir.
- **Tutorial: os dois** — o app faz o passo para o aluno ver E espera o aluno fazer (portão de estado).
- **Simulação ao vivo, idêntica pelo tempo gravado** (o dono: «se o caminho do traço e o tempo da
  simulação — o quanto o usuário espera para a próxima ação — forem considerados, os resultados são iguais»).
  Medido no código, a forma exata disto:
  - o Wet Paint já tem as duas portas: o **worker** (padrão do app, anda no relógio dele) e o
    **`relogio_fixo`** (síncrono, passos = dívida de `dt`; `crates/ph2d-tool-painter/src/tool/paint/wetpaint.rs:356`;
    hoje só os testes o armam, `set_wet_relogio_fixo`). A sim é CPU pura (sem `wgpu` em `ph2d-wet-paint`).
  - ⇒ **gravar o CONTADOR de passos da sim** (`sess.sim_steps()`, atômico) junto de cada ação, não só os
    segundos: o worker pode ter dado menos passos que o relógio pedia (máquina carregada), e é o contador
    que reproduz o que o artista VIU. Na reprodução: `relogio_fixo` + exatamente esse número de passos.
  - `dt` sozinho mantém a VELOCIDADE certa, não o resultado igual; o que dá a igualdade é contar passos fixos.
- **Pendente:** por que família começar (proposta: núcleo + Painter/Wet Paint + Motion, os dois exemplos do dono).
