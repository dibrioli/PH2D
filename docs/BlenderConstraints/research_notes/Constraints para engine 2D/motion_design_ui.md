# Constraints em motion design, animação procedural e UI/layout — insumo para o PH2D (2D, Rust)

> Pesquisa de 2026-10-25. Convenção: tudo em **Cited Findings** foi conferido nesta sessão na fonte
> ligada. O que vem de conhecimento prévio e **não** foi reconferido está em **Inferences** com a
> marca *(prévio, não verificado)* ou em **Gaps**. Várias páginas oficiais (Duik, Maya 2025, Apple
> Motion por behavior, Cavalry Magnet/Falloff) devolveram 404, redirecionamento ou só o índice —
> ver Gaps.

## Quais comportamentos são as ferramentas «um clique» mais simples e queridas por animadores não técnicos?

### Takeaway
As ferramentas «um clique» de sucesso têm três coisas em comum: aplicam-se **sobre** a animação
existente (somam movimento secundário sem a apagar), têm **2–4 parâmetros com nome físico**
(amortecimento, massa, frequência) e criam-se arrastando para um alvo. Exemplos: Wiggle/Spring/
Bounce/Swing/Blink/Walk cycle no Duik, Look At e Spring no Cavalry, e Snap Alignment to Motion e
Attracted To no Apple Motion.

### Cited Findings
- **Duik Ángela** (17.ª versão do Duik, para After Effects) agrupa as «automações» que «ajudam a animar mais depressa»: **Wiggle, Random motion, Wheel, Spring, Bounce, Looper, Swing, Blink, Walk and run cycle**. Também tem ferramentas de câmera, de script/expressão e de keyframes/interpolação/tweening. Há um guia pré-release do Ángela.2 — [duik.org](https://duik.org/)
- O Duik é **GPL-3.0** (repositório `RxLaboratory/Duik`, último push 2026-10-12) ⇒ é um **oráculo a correr, nunca uma fonte a ler** (CLAUDE.md §0.9) — [GitHub API: RxLaboratory/Duik](https://github.com/RxLaboratory/Duik)
- **Cavalry**: os Behaviours são «como efeitos» que **animam ou deformam** outras layers. O catálogo atual inclui:
  - **Movimento:** Stagger, Oscillator, Noise, Spring, Look At, Wave, Motion Stretch, Auto-Animate, Random, Modulate.
  - **Deformadores:** Bend, Lattice, Travel, JavaScript Deformer, Pinch, Flare, Skew, Four Point Warp, Morph, Blend Shape.
  - **Path:** Chop, Path Average, Path Offset, Path Relax, Pathfinder, Resample, Reverse, Extend Open Paths.
  - **Valores:** Value, Value2/3, **Value Blend**, **Value Solver**, Number Range, Number Range to Color, **Distance**, Area Range.
  - **Layout:** Align, Apply Layout, **Apply Distribution**, Frame, Manipulator, Mesh Solver, Get Vector, Is Within, Visibility Sequence, **Behaviour Mixer**.
  - «Magnet», «Constrain to Path», «Apply Distance» e «Falloff» **não aparecem com esses nomes** no índice atual.
  - Fonte: [Cavalry — Behaviours](https://cavalry.studio/docs/nodes/behaviours/)
- **Cavalry Spring**: «adiciona movimento secundário dinâmico aos keyframes de posição de uma Shape». Os parâmetros são **Damping, Mass, Speed Limit** e está marcado como **experimental** (é preciso ligá-lo nas Preferences) — [Cavalry — Spring](https://cavalry.studio/docs/nodes/behaviours/spring/)
- **Cavalry Look At** roda shapes para um alvo e tem só **Target** e **Offset**. O fluxo é de ligação explícita: `null.id → lookAt.target`, `lookAt.id → polygon.rotation`. O behaviour é um **nó** que se pluga num atributo, não um modificador escondido — [Cavalry — Look At](https://cavalry.studio/docs/nodes/behaviours/look-at/)
- **Apple Motion — Snap Alignment to Motion**: combina-se com um behavior que mova a posição ou com um caminho de keyframes, e faz a rotação seguir a direção de viagem **sem keyframes extra**. Tem o pop-up Rotation Axis (X/Y/Z/All, default Z) — [Apple Motion User Guide](https://support.apple.com/guide/motion/motn13746057/mac)
- **Apple Motion — Align to Motion** é um behavior de *Simulation* separado. É mais «springy» e vivo, enquanto o Snap Alignment é exato. **Não funciona** com o behavior Motion Path; para esse caso a Apple recomenda o Snap — [Apple Motion User Guide](https://support.apple.com/guide/motion/motn13746057/mac)
- **Apple Motion — categorias**: Link é um behavior de *Parameter* (tem uma página sobre usá-lo para controlar texto). Attracted To, Attractor, Orbit Around, Spring, Drag e Gravity são de *Simulation*. Throw e Snap Alignment to Motion são de *Basic Motion* — [Apple Motion User Guide (índice)](https://support.apple.com/guide/motion/motn13746467/mac)
- **Apple Motion — atração**: tem *Falloff Rate* (quão depressa a força cai com a distância) e *Influence* (raio de efeito) — [Apple Motion User Guide](https://support.apple.com/en-al/guide/motion/motn13745565/mac)

### Inferences
- O padrão «comportamento sem keyframes» (Snap Alignment, Spring do Cavalry e o Spring/Wiggle do Duik) é o mais amado porque **não exige saber o que é uma constraint**: o artista arrasta o comportamento para o objeto e ele já anima.
- *(prévio, não verificado)* No After Effects, o **pick-whip** (arrastar a espiral para outra propriedade ou layer) cria parenting ou uma expressão de ligação num gesto. As expressões mais usadas por não programadores são `wiggle(freq, amp)`, `loopOut("cycle"|"pingpong"|"offset"|"continue")`, `valueAtTime(t)` e `lookAt(a,b)`. O Duik **Connector** liga um controlador (slider/ângulo) a várias propriedades com remapeamento de faixa. O Duik **Kleaner** suaviza/interpola com antecipação e follow-through. O **Swink** é swing+blink. O **Effector** propaga um efeito por distância a um controlador.
- *(prévio, não verificado)* Na mesma família «um clique»: **Joysticks 'n Sliders** (aescripts) é uma pose-interpolação 2D por joystick; **RubberHose** (Battle Axe) faz membros como mangueiras com dois controladores e sem ossos; **Motion Tools Pro** é uma barra de atalhos (anchor, null, easing).
- Para o PH2D, a lista mínima «um clique» seria: **Spring (secundário)** · **Wiggle/Noise** · **Look At / Align to Motion** · **Loop (cycle/pingpong)** · **Follow Path** · **Attract/Orbit** — cada um com ≤ 4 parâmetros.

### Gaps
- Não consegui ler a documentação por ferramenta do Duik (Connector, Effector, Kleaner, Swink, IK/Bones). `rxlaboratory.org/docs/duik-angela/` deu 404 e `duik.org` só tem a lista de automações.
- Faltam as páginas individuais do Apple Motion para Attracted To, Orbit Around, Throw e Link (parâmetros), e as do Cavalry para Stagger, Oscillator, Noise e Behaviour Mixer.
- Não pesquisei Houdini/KineFX, Jitter, Linearity Move nem LottieFiles Creator (orçamento de chamadas).
- Não achei dados quantitativos («mais amado») — o que fica é a inferência pela proeminência nos menus e nas docs.

## Que padrões (pilha de behaviours, falloff, spring/inércia, troca de espaço, bake) uma engine 2D deve copiar?

### Takeaway
Os padrões que se repetem em Maya, C4D, Unreal, Cavalry e Apple Motion são:
1. **Pesos por alvo** com mistura de vários alvos.
2. **Maintain offset** no momento da criação.
3. **Ordem de avaliação explícita** dentro do conjunto de constraints.
4. **Keys de compensação** ligadas à key que liga/desliga a constraint, para trocar de espaço sem pulo.
5. **Bake para keyframes** que desativa a constraint no intervalo cozido.
6. Comportamentos dinâmicos (**spring**) que só avançam no **play**, nunca no refresh do viewport.

### Cited Findings
- **Unreal (Sequencer / Control Rig)**:
  - Os métodos de constraint são **Position, Rotation, Scale, Parent e LookAt**.
  - Criar uma key de constraint também cria **compensating keys** que mantêm a mesma posição no mundo. Essas keys ficam **ligadas** à key da constraint e acompanham-na quando o tempo muda.
  - Se a pose do pai mudar depois, pode haver um **pop** ao ativar ou desativar. A cura é «**Compensate Key**» com o playhead sobre a key da constraint.
  - **Bake** volta à animação por keyframes normal.
  - Para editar constraints é preciso ligar o Animation Mode.
  - Fonte: [Unreal — Animation Constraint Tools](https://dev.epicgames.com/documentation/en-us/unreal-engine/animation-constraint-tools-in-unreal-engine)
- **Unreal Python API**:
  - `add_constraint(tipo, filho, pai, maintain_offset)` — o offset é um flag na criação.
  - `bake_constraint(frames)` com lista vazia usa os intervalos ativos da constraint; depois do bake, a constraint fica keyada **inativa** no intervalo cozido.
  - `bake_control_rig_space` coze num espaço escolhido, com redução de keys.
  - Os nomes dos enums de tempo mudam entre versões (SequenceTimeUnit → MovieSceneTimeUnit).
  - Fonte: [ControlRigSequencerLibrary](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/ControlRigSequencerLibrary)
- **Cinema 4D — Constraint tag**:
  - É **uma tag só** que agrupa PSR, Parent, Up Vector, Aim, Clamp, Spring e Mirror. Cada tipo liga-se com uma checkbox e abre uma aba própria.
  - A ordem de avaliação documentada na versão 2027 é **Parent → Transformation → Mirror → Clamp → Spring → Aim → Up Vector**. Nas versões S22/R21 o PSR vinha em 2.º lugar, e os alvos dentro de uma constraint avaliam-se de cima para baixo.
  - **Frame Update** impede que constraints dinâmicas (Spring) mudem o objeto no refresh do viewport: só avançam no playback.
  - A Maxon diz que a tag é mais rápida que o equivalente em XPresso.
  - Fontes: [Maxon C4D 2027 — Constraint](https://help.maxon.net/c4d/2027/en-us/Content/html/TCACONSTRAINT.html); [C4D S22](https://help.maxon.net/c4d/s22/us/html/TCACONSTRAINT.html)
- C4D (dica de terceiros, Novedge, jan/2026): combinar **Aim com Up Vector** evita o flip; subir a **Priority** da tag faz com que ela avalie depois do IK/deformadores. Não é guia oficial — [Novedge](https://novedge.com/blogs/design-news/cinema-4d-tip-reliable-constraint-workflow-for-cinema-4d)
- **Maya**:
  - Os tipos são point, orient, scale, aim, parent, geometry, tangent e pole vector.
  - **Point** só afeta a translação.
  - **Tangent** orienta o objeto ao longo da curva e costuma combinar-se com **geometry** (ex.: carrinho de montanha-russa).
  - **Pole vector** segue a posição de um alvo ou a **média** de vários.
  - Com vários alvos, o **peso** de cada um vai de 0 (sem influência) a 1 (influência total).
  - Fontes: [Maya — Constraining objects](https://help.autodesk.com/cloudhelp/2017/ENU/Maya/files/GUID-FA047B7D-41AD-4643-9D10-BB1F3B397B4D.htm); [What are constraints?](https://help.autodesk.com/cloudhelp/2015/ENU/MayaLT/files/CSCo_What_are_constraints.htm)
- **Cavalry**: o behaviour é um nó ligado por porta (`id → atributo`). Existem **Behaviour Mixer** e **Value Blend** para misturar saídas — a «pilha» lá é um **grafo de ligações**, não uma lista empilhada — [Cavalry — Behaviours](https://cavalry.studio/docs/nodes/behaviours/), [Look At](https://cavalry.studio/docs/nodes/behaviours/look-at/)
- **Apple Motion**: os behaviors de atração têm **Influence** (raio) e **Falloff Rate** (caída com a distância). É o modelo de «falloff» mais simples de expor ao artista — [Apple Motion](https://support.apple.com/en-al/guide/motion/motn13745565/mac)
- **Lottie** (para exportar): as expressões seguem as do AE em JS e exigem `$bm_rt`. O suporte documentado inclui `valueAtTime`, `loopIn`/`loopOut`, `smooth`, `linear`/`ease`/`clamp`, `lookAt`, `random`/`seedRandom`, `posterizeTime` e `toComp`/`toWorld`. **`wiggle()` não aparece** na referência — [Lottie Docs — Expressions](https://lottiefiles.github.io/lottie-docs/expressions/)

### Inferences
- **O que copiar**:
  1. **Maintain offset = ON por omissão** ao criar com arrastar-para-o-alvo (Maya/Unreal expõem-no como flag).
  2. **Influence 0–1 keyframável** por constraint e **peso por alvo** (Maya).
  3. **Ordem de avaliação fixa e visível** (C4D documenta-a; no PH2D vira ordem de systems ECS com gate de ordem, como o Runtime de sinais já faz).
  4. **Ligar/desligar como key com keys de compensação ligadas** (Unreal) — troca de espaço sem pulo.
  5. **Bake → keys + constraint keyada inativa no intervalo** (Unreal), com redução de keys.
  6. **Spring só no relógio de play** (C4D Frame Update), determinístico, nunca no refresh.
  7. **Falloff = raio + curva** (Apple Motion).
- **Linhas de conexão visuais** entre constrangido e alvo: *(prévio, não verificado)* o Maya/C4D desenham-nas no viewport e a Cavalry mostra-as no grafo. Para o PH2D, desenhá-las no canvas enquanto o objeto está selecionado.
- *(prévio, não verificado)* No Maya, o **parent constraint** mistura vários alvos por peso, e a mistura com animação existente é o atributo `blendParent` criado ao keyar um canal constrangido. Os tipos extra do Maya são normal, point-on-poly, motion path e o «rivet» (seguir um ponto da malha). O **Clamp** do C4D é um limite de faixa; o **Attach** e o **Pole** completam a lista pedida.
- Em 2D, a transformação reduz-se a (x, y, θ, sx, sy). Aim/Up colapsa num só ângulo, sem problema de flip — a combinação Aim+Up do C4D é desnecessária.
- **Wiggle não exportável para Lottie** ⇒ se o PH2D um dia exportar Lottie, o Noise/Wiggle deve **cozer para keys** no export.

### Gaps
- Não reli a página atual do Maya (2025/2026) sobre maintain offset, blendParent, normal, point-on-poly, motion path e rivet — o link 2025 deu 404 e as páginas encontradas são de 2009–2017.
- Não confirmei os tipos **Attach** e **Pole** no C4D 2027 (a busca só listou PSR/Parent/Up/Aim/Clamp/Spring/Mirror).
- Não conferi quais players Lottie (lottie-web, Skottie, ThorVG/dotLottie, rlottie) avaliam expressões.

## Qual é a licença das bibliotecas de solver reutilizáveis (implementações de Cassowary) e de layout?

### Takeaway
Todas as opções Rust relevantes são permissivas (MIT e/ou Apache-2.0) e compatíveis com o PH2D. A
mais viva é **`kasuari`** (fork do `cassowary-rs` mantido pelo ratatui, **MIT OR Apache-2.0**,
atualizada em 2026-10). O `cassowary-rs` original está parado desde 2017–2019. O Kiwi (C++) é
BSD-3. Para layout estilo flex/grid, o **`taffy`** (MIT) é o padrão.

### Cited Findings
| crate / lib | versão | licença | estado | fonte |
|---|---|---|---|---|
| `kasuari` (ratatui) | 0.4.13 | **MIT OR Apache-2.0** (repo tem `LICENSE-MIT` + `LICENSE-APACHE`; o GitHub só detecta Apache-2.0) | atualizado 2026-10-15, ~28 M downloads | [crates.io API](https://crates.io/api/v1/crates/kasuari) · [GitHub](https://github.com/ratatui/kasuari) |
| `cassowary` (cassowary-rs, dylanede) | 0.3.0 | MIT / Apache-2.0 | última versão 2017-09-13, último push 2019-10-24 | [crates.io API](https://crates.io/api/v1/crates/cassowary) · [GitHub](https://github.com/dylanede/cassowary-rs) |
| `casuarius` (schell) | 0.1.1 | MIT / Apache-2.0 | 2022-08, ~3,5 k downloads | [crates.io API](https://crates.io/api/v1/crates/casuarius) |
| Kiwi (nucleic, C++) | — | **Modified BSD (BSD-3)**, © 2013-2026 Nucleic | ativo (push 2026-10-16) | [LICENSE](https://raw.githubusercontent.com/nucleic/kiwi/main/LICENSE) |
| `taffy` (DioxusLabs; flexbox/grid/block) | 0.15.1 | MIT | atualizado 2026-10-25, ~17 M downloads | [crates.io API](https://crates.io/api/v1/crates/taffy) |
| `morphorm` (vizia) | 0.9.0 | MIT | 2026-07 | [crates.io API](https://crates.io/api/v1/crates/morphorm) |

Contexto de UI/layout:
- **Figma constraints**:
  - Horizontal: **Left, Right, Left and right** (estica), **Center**, **Scale** (posição e tamanho em % do frame). Vertical: os equivalentes.
  - O default é Top + Left.
  - **Constraints não se aplicam** a layers dentro de um frame com **auto layout**.
  - Segurar **Cmd/Ctrl** ignora as constraints temporariamente ao redimensionar.
  - Fonte: [Figma Help — constraints](https://help.figma.com/hc/en-us/articles/360039957734-Apply-constraints-to-define-how-layers-resize)
- **Rive Layouts**:
  - Funcionam «como flexbox», com Row/Column/Grid/Stack, **Hug** (o pai abraça os filhos) e **Fill** (os filhos enchem o pai), e posicionamento Absolute ou Relative.
  - Cada objeto escolhe se participa no layout, e as **constraints** permitem «sair do layout de forma controlada».
  - O modo «Layout» dá width/height/fit a uma shape, texto ou imagem.
  - Fonte: [Rive — Layouts & Constraints](https://rive.app/docs/editor/layouts/layouts-&-constraints)
- **Rive N-slicing** (nov/2024): generaliza o 9-slice a vetores e imagens, com escolha de que faixas ficam fixas e quais esticam — [Rive blog](https://rive.app/blog/responsive-layouts-phase-two-n-slicing)
- **Yoga** é um motor que implementa um subconjunto de flexbox (React Native, Unity UI Toolkit). Não encontrei confirmação de que o Rive o use — [Yoga](https://yogalayout.dev/docs/about-yoga)

### Inferences
- *(prévio, não verificado)* O Apple Auto Layout usa o algoritmo Cassowary (simplex incremental) com **prioridades** 1–1000 (1000 = required) e com hugging/compression resistance. O `kasuari` herda as mesmas forças do Cassowary: REQUIRED, STRONG, MEDIUM e WEAK.
- Para o PH2D, o Cassowary só se justifica para **layout de UI/HUD com relações lineares** (A.left = B.right + 8). As constraints de animação (copy transform, look at, IK, spring) **não** são problemas lineares: resolvem-se por avaliação ordenada e por solvers dedicados (IK analítico/FABRIK). Se precisar de layout de HUD, o padrão flex/grid do `taffy` (MIT) cobre o caso Figma Auto Layout/Rive; o `kasuari` cobre o caso Figma constraints + relações arbitrárias.
- Todas as licenças da tabela são compatíveis com um produto fechado ou permissivo. O **Duik (GPL-3.0)** é o único item do catálogo que **não** pode ser lido como fonte — só corrido como oráculo, de acordo com o CLAUDE.md §0.9 e o `/pd-cleanroom`.

### Gaps
- Não verifiquei se o `kasuari` mudou a API em relação ao `cassowary-rs`, nem a sua performance.
- O Figma Auto Layout com **min/max width** não está na página de constraints — falta a página de Auto Layout.
- Não li a doc de Auto Layout da Apple (developer.apple.com) nesta sessão.
- Não li as constraints do Rive (IK, Distance, Transform, Translation, Rotation, Scale, Follow Path, Scroll) — a página fetchada só cobre layouts.
