═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–19 em `line/motion-value` sobre o `main` `2a925859d`, POR INTEGRAR. O ciclo 19
(O JOGO MANDA, [doc 128](../128_ciclo_19_o_jogo_manda.md)) fechou o código e o gate batched; o handoff
de integração é o [CICLO_19](HANDOFF_INTEGRACAO_line_motion_value_2026-10-10_CICLO_19.md).

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 &&
        git status --short — ANTES de abrir qualquer ficheiro.

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8). Integrar/pushar só por ordem dele.
⭐ A OBRA: [doc 126 §0](../126_ciclo_17_o_ritmo.md) — ordem do dono (09/10): fazer tudo da avaliação
   da stack menos o export de vídeo. Fila no [doc 103 §5](../103_dinamica_dos_ciclos.md); protocolo
   de cada ciclo = doc 103 §1 (os sete passos, o smoke é o tutorial em PDF).
⭐ CUSTO: delegue a leitura larga a agentes `explorador` em paralelo e o gate batched a um
   `verificador`; a janela principal desenha e decide.

PRIMEIRO — o smoke do dono do ciclo 19 está ⏳ PENDENTE (tutorial 12
`docs/Motion Nodes/tutoriais/12_o_jogo_manda.pdf`, cena `PH2D_GPU_COOK_DEMO=134`). Se o Enio trouxer
um problema, CORRIJA antes de abrir o ciclo 20 (o relatório do dono manda; foi assim no ciclo 17,
`c7ef7e7e0`). Sem relato, siga.

ABRA O CICLO 20 — **O ACABAMENTO** (doc novo `129_ciclo_20_o_acabamento.md`; linha 20 da tabela do
doc 126 §0): pós-processamento como NÓS sobre a camada do Motion — blur · glow · sombra de pixel ·
warp · grade/LUT · vinheta · grão · aberração.
- REUSA o `FxStackPass` (`ph2d-render/src/fx_stack.rs`) e o alvo HDR isolado do bloom
  (`ph2d-render/src/motion_fx.rs`); só o bloom é pixel hoje — drop-shadow e rgb-split são geometria
  (nota do doc 126), medir antes de decidir qual é qual.
- ⚠️ ANTES de construir: MEÇA se a composição já exprime cada item (CLAUDE.md §5.0) e confira o
  CÓDIGO antes de acreditar numa ausência; o oráculo é um app que se CORRE (Godot, shaders de
  referência) sobre entradas NOSSAS, comparado por passo (§0.9) — triagem de licença por artefacto.
- cena de smoke com FORMAS e SIMULAÇÃO com campos, FOTOGRAFADA antes de mandar
  (`docs/Components/ferramentas/fotografa_cena.sh`); a próxima cena é a `=135` (`MAX_DEMO_LEVEL` está
  em 134 — reconte se o `main` acrescentou cenas); tutorial 13 em PDF com o gate dos passos.
- ⛔ contratos congelados (`NodeOp`/`OpResolver`/`NodeManifest`) não se tocam — tudo entra como nó
  novo, param apendado ou canal ao lado; quem achar o contrário PARA e reporta.
- todo número novo é um número de FÁBRICA do documento: nasce por `register_param_defaults`, e o
  default do manifesto é o que um documento gravado lê.

LIÇÕES DO CICLO 19 (não as repague — detalhe no doc 128 §5–§6):
- o replay de um canal de fora é uma CAMADA POR CIMA, nunca uma substituição (substituir faz o tique
  seguinte gravar a fita como se fosse o presente);
- scrub ≠ tocar sobre chão gravado: a mesma marcha tique a tique é replay com o relógio PARADO e
  gravação a tocar — só o shell sabe qual, e tem de o dizer à bomba;
- um pulso decidido por quem escreve CONTA, não pulsa: a contagem independe dos tiques por quadro;
  o pulso do tique deriva-se onde se sabe o tique;
- uma variável de ambiente que uma ferramenta escreve num script não leva `;` (usa `:`);
- um campo em velocidade-alvo devolve a calma; uma força radial sobre ele não volta (recusa medida,
  doc 128 §5.3) — para um gatilho reversível numa simulação, mexa no campo que a simulação já devolve;
- a régua lê a SAÍDA e a foto acha o que nenhum gate vê (a barra colada ao topo, a faixa por vão
  errado): fotografe CADA cena antes de a mandar, em tempos diferentes;
- o oráculo ensina o instante: no Godot o peso do pedido é o de antes e sobe a partir do quadro
  seguinte; três armadilhas do oráculo estão no doc 128 §5.2;
- um valor de OUTRO tipo não entra numa porta de pulso (`TypeMismatch`): a cena é que achou.

ABERTOS NOMEADOS (doc 128 §7 e herdados):
- a dobra da fita acima do orçamento de 32 MiB (por medir numa cena de MUITOS objectos);
- um gesto para dirigir uma linha de COR inteira (hoje três fios `r g b` do `Input` ao `Tint`);
- a vida e o dano do mesmo objecto pedem dois objectos (um `Motion Input` por objecto);
- o `Bezier` do easing e o `Tint` OKLCH caros na CPU de reserva (5 ms e 21 ms a 32 761 linhas; na
  placa 0,15 e 0,16 ms) — um Newton com a mesma contagem nas duas línguas cortaria-os;
- ⏳ a tabela de provas de mutação do ciclo 19 (a corrida da janela anterior), no handoff de integração.
═══════════════════════════════════════════════════════════════════
