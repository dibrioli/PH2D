═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–18 em `line/motion-value` sobre o `main` `2a925859d`, POR INTEGRAR. Os ciclos 17
(O RITMO, [doc 126](../126_ciclo_17_o_ritmo.md)) e 18 (A COR QUE O OLHO VÊ,
[doc 127](../127_ciclo_18_a_cor_que_o_olho_ve.md)) fecharam com o smoke do dono APROVADO (10/10).

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 &&
        git status --short — ANTES de abrir qualquer ficheiro.

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8). Integrar/pushar só por ordem dele.
⭐ A OBRA: [doc 126 §0](../126_ciclo_17_o_ritmo.md) — ordem do dono (09/10): fazer tudo da avaliação
   da stack menos o export de vídeo. Fila no [doc 103 §5](../103_dinamica_dos_ciclos.md); protocolo
   de cada ciclo = doc 103 §1 (os sete passos, o smoke é o tutorial em PDF).
⭐ CUSTO: delegue a leitura larga a agentes `explorador` em paralelo e o gate batched a um
   `verificador`; a janela principal desenha e decide.

ABRA O CICLO 19 — **O JOGO MANDA** (doc novo `128_ciclo_19_o_jogo_manda.md`):
- `value.input` (nó novo): uma entrada PUBLICADA do grafo, com nome e tipo (número · sim/não ·
  gatilho · cor), lida por `ctx.external(nome)` — a porta já existe e é memo-aware
  (`ph2d-nodegraph/src/external.rs`, `cook.set_external`); sem publicação ⇒ o default do nó.
  Um gatilho = pulso quando a revisão do valor muda (o domínio `pulse.*`).
- as entradas do grafo aparecem no Inspector do componente Motion e têm override por
  instância/variante; quem escreve: as pontes dos componentes de jogo (vida → HUD), o Luau
  (`motion.set("vida", 0.3)`) e os sinais do runtime (gatilho).
- `TimeMode::Driven` (APENDADO à escada, `ph2d-nodegraph/src/time.rs`): o tempo de uma sub-árvore
  = curva(entrada) — a barra que enche, o blend 1D do Rive.
- crossfade de estados SEM máquina nova: pesos do `mixer` ← entrada → `value.smooth`/`spring`.
- ⚠️ no editor o scrub tem de gravar as entradas (senão o replay diverge); medir antes de desenhar.
- cena de smoke com FORMAS e SIMULAÇÃO com campos (doc 103 §1), fotografada; tutorial 12 em PDF.
- ⛔ contratos congelados (`NodeOp`/`OpResolver`/`NodeManifest`) não se tocam — tudo entra como nó
  novo, param apendado ou canal ao lado; quem achar o contrário PARA e reporta.
- ⚠️ a ponte dos componentes vive em `ph2d-app-components` (a `line/components` também mexe lá):
  mudança append-only, e diga-o no handoff.

LIÇÕES DOS CICLOS 17–18 (não as repague — detalhe nos docs 126 §5–§6 e 127 §5):
- a régua de uma cena lê a SAÍDA (o que se desenha), nunca um nó do meio; uma foto num instante
  prova o instante — tire o lote em tempos diferentes;
- a matemática DETERMINISTA da casa vive na `ph2d-ease` (`pe_div`, `cbrt`, `atan_turn`,
  `sin/cos_turn`, `fract`), e o WGSL gera-se das mesmas tabelas POR BITS; um algoritmo com saída
  antecipada é um degrau que um ULP atravessa (doc 127 §5.2);
- um nó NOVO muda de default por `register_param_defaults`; o default do manifesto é o que um
  documento gravado lê;
- uma janela flutuante do cartão cabe nos `205,5 px` de altura do painel do grafo (medido);
- num nó com `channel` nenhum param se declara `ParamUnit::Length`; `from` é reservado no WGSL;
  uma leitura de coluna OPCIONAL num kernel vive numa VARIANTE (senão `MissingSource` falso).

ABERTOS NOMEADOS: o `Bezier` do easing e o `Tint` OKLCH são caros na CPU de reserva (5 ms e 21 ms a
32 761 linhas; na placa 0,15 e 0,16 ms) — um Newton com a mesma contagem nas duas línguas cortaria-os.
═══════════════════════════════════════════════════════════════════
