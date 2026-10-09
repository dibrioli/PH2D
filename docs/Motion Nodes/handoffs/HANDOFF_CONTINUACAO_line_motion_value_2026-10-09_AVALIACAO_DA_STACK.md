═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: os ciclos 13–16 fecharam em `line/motion-value` sobre o `main` `2a925859d`, POR INTEGRAR
(último: [handoff do ciclo 16](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_16.md), smoke
aprovado pelo dono em 09/10). Esta janela é SÓ DE AVALIAÇÃO: não integra, não pusha, não implementa.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -4 &&
        git status --short — ANTES de abrir qualquer ficheiro.

⛔ Fale com o Enio em PT-BR. A resposta é técnica (ele pediu assim), mas curta.
⛔ NÃO implemente nada e NÃO crie ficheiros (ordem do dono, abaixo). A resposta vai no chat.
⭐ CUSTO: a leitura do módulo é larga — delegue-a a agentes `explorador` em PARALELO (um por
   família de perguntas: tempo/avaliação · timeline/stagger/easing · sinais/memo/script · dados/
   layouts/cor/paths · pós-processamento/blur · morph · state machines/runtime do jogo) e decida na
   janela principal. Confira no CÓDIGO antes de afirmar uma ausência (CLAUDE.md §5.0); cite
   `ficheiro:linha` para cada «temos»/«parcial».
Pontos de entrada: `docs/Motion Nodes/README.md` (índice), `103_dinamica_dos_ciclos.md` §5 (a fila
dos ciclos e o que cada um entregou), `BUGS_motion_nodes.md`, as crates `ph2d-app-motion`,
`ph2d-nodegraph` (cook, relógio, sub-passos), `ph2d-gpu-cook`, `ph2d-node-*`, `ph2d-timeline`,
`ph2d-runtime` (sinais), e o motor de script (procure as corrotinas Luau antes de as comparar).
Contratos congelados (CLAUDE.md §6: `NodeOp`/`OpResolver`/`NodeManifest`) — toda proposta que os
toque diz o preço (ADR).

O PEDIDO DO DONO (verbatim):

Se hoje eu pedisse o mais extraordinário motion graphics para uma apresentação
em vídeo, uma stack de referência seria: Remotion, GSAP, Three.js/R3F com
shaders GLSL, D3, Motion Canvas, Manim, Rive/Lottie e FFmpeg.

Não quero usar nenhuma delas como dependência. Quero saber: como as ideias de
arquitetura e de UX dessa stack poderiam inspirar o nosso módulo de motion
nodes (estilo Cavalry, Rust/wgpu/Vello) a ficar mais capaz, mais poderoso,
mais intuitivo, mais fácil de usar e mais útil dentro de uma game engine?

Antes de responder, leia a implementação atual do módulo e baseie a análise
no que existe de fato, não em suposições.

Avalie pelo menos estas ideias, dizendo para cada uma se já temos, se temos
em parte ou se falta:

1. Remotion: avaliação pura por frame (cena = f(frame)), tempo como entrada
   explícita do grafo (tempo local, remap, loop, freeze), spring analítico
   scrubável, export offline com timestep fixo via pipe para FFmpeg.
2. GSAP: timeline com posicionamento relativo e labels, stagger avançado
   (origem, grid 2D, easing sobre a distribuição do atraso), biblioteca de
   easings e easing customizado.
3. Motion Canvas: signals reativos lazy com rastreio de dependência
   (dirty propagation e memoização), animação procedural por generators
   (comparar com nossas corrotinas em Luau), eventos de tempo editáveis na
   timeline e ancorados ao áudio.
4. D3: nós de escala (domínio → imagem), layouts como distribuições (force,
   pack, tree, Voronoi), interpolação de cor em OKLab/OKLCH, morphing de
   paths, duplicadores dirigidos por dados com enter/update/exit.
5. Three.js/shaders: motion blur por acumulação de subframes, cadeia de
   pós-processamento como nós sobre a saída do Vello.
6. Manim: transformações entre formas e texto com correspondência de pontos.
7. Rive: state machines com blend e inputs em runtime, para motion que reage
   ao estado do jogo (UI, HUD, feedback).

Inclua também ideias dessa stack que eu não listei e que você considere
relevantes.

Formato da resposta:
- Tabela: ideia | estado atual (temos / parcial / falta) | ganho | esforço |
  risco ou conflito com a arquitetura atual.
- Top 5 priorizado por ganho/esforço, com justificativa técnica curta.
- Para cada item do top 5: esboço do design (nós novos ou alterados, mudanças
  no modelo de avaliação) e o que quebraria em projetos existentes.
- Aponte o que NÃO vale a pena adotar e por quê.

Não implemente nada e não crie arquivos. Só a avaliação. Seja conciso e
técnico.
═══════════════════════════════════════════════════════════════════
