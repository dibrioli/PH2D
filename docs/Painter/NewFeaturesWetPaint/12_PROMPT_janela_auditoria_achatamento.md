TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), AUDITORIA W1f: o ACHATAMENTO do escorrimento — a tinta
que desce para e fica com bordas RECTAS (horizontais e verticais), como se rectângulos invisíveis a cortassem.
Primeiro a CAUSA medida, depois a cura (§0.10: todos os casos da mesma família num bloco).

Smoke do dono (10/10, sobre `a9ce1a4c6`): *«ainda está acontecendo o estranho achatamento do escorrimento que acaba
por formar áreas retas (parece que retângulos virtuais influenciam)»*. Captura:
[fixtures/smoke_dono_w1e/achatamento_s_vermelho.png](fixtures/smoke_dono_w1e/achatamento_s_vermelho.png) — um S
vermelho (Wet Paint) com TRÊS sítios marcados: (1) a ponta baixa da esquerda tem a orla de baixo HORIZONTAL e recta
por ~60 px; (2) sob a crista, um degrau em L (um entalhe de cantos rectos); (3) à direita um «fio» que é um
RECTÂNGULO de ~25×40 px: lados verticais, fundo plano, cantos a 90°. A orla de cima do S é orgânica; o defeito é
só onde a água DESCEU. ⚠️ «ainda» = já existia: a auditoria da W1c (09/10) achou «o fio RECTANGULAR de topo plano
colado por baixo de uma orla oblíqua» (05 §4b, prompt [09](09_PROMPT_janela_auditoria_W1c.md)) e foi desfeita com
as gotas; a W1e (`c91dade9c`) acrescentou uma lei nova no MESMO sítio. Não se sabe o Tilt da captura — reproduza
nos três (direita, meia, máxima).

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -6 && git status --short → a ponta é este prompt («docs(Painter 10/10): o prompt da W1f …»),
   com a9ce1a4c6 · e9a4d8f0e · c91dade9c · 0b64130fa · 2e92420be (a W1e) logo abaixo; árvore limpa.
3. git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
4. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por bash scripts/ph2d-run.sh; o
   hook recusa todo texto de Bash com «cargo test» fora da porta (inclusive heredoc e comentários): escreva o script
   num ficheiro (Write) e corra `bash scripts/ph2d-run.sh bash <ficheiro>`. A shell é fish: laços e arrays em bash
   vão no ficheiro.
5. Cercas verdes ANTES e DEPOIS: bash docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh · os_outros_meios_nao_mudam
   · a suíte de ph2d-wet-paint (as impressões digitais do port NÃO podem mover) · o censo dos controlos do painel
   (`cargo test -p ph2d-panel-registry-init --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`
   — sem as features o censo não vê o wet_tuning e reprova em falso).

⛔ O QUE O DONO VETOU (não reconstrua): a LINHA DE CONTACTO da W1b/W1c (fluxo ZERO na célula de fora — a barreira que
prende a água no papel seco) e o sistema de gotas (Create/Easy Drips, DripLaw, o canal). Estão no histórico
(8e9278993..8d3e8f68a), recusas medidas no 05 §4b. ⚠️ A lei da gota da W1e (`solver::Gota`) é PARENTE dela: limita
o fluxo de células secas. Se a causa for ela, a cura NÃO é voltar à barreira.

LEIA (só isto, depois o código)
1. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md (§3 auditar ≠ compilar — o template por lente; §4 efeito
   perceptual não fecha sem OLHAR a 8×).
2. 05_plano_da_linha_e_o_antes.md §4b: a entrada «🔎 Auditoria da W1c» (a raiz rectangular, a régua da cobertura) e a
   da «W1e» até ao fim (a tabela, a lei da gota, as recusas, o aberto).
3. Código: crates/ph2d-wet-paint/src/solver.rs (`Gota::new/aplica`, `ALTURA_DA_GOTA`, `build_flow_field`,
   `smooth_velocity_rows`, `clamp_sym`), solver/flow_jacobi.rs (o mesmo no Jacobi), solver/advect_jacobi.rs
   (`momentum_rows`, o retro-traço, `outflow_rows`), sim.rs (a ordem dos estágios, `Params::gota`),
   grid.rs (a bbox e a faixa viva `row_lo/row_hi`, `SPAN_PAD`), solver/active_region.rs, flow.rs (a grade de
   FLUXO `rf`); do lado do produto wetpaint/dab_route.rs (a fila `adiados`), wetpaint/composite.rs (o rectângulo
   sujo), wetpaint/smooth_edges.rs.
4. As sondas e réguas que já existem: wetpaint/sonda_gota.rs (`bancada`, PH2D_SONDA_VARRE "G:H:knob=v",
   PH2D_SONDA_PINCEL, PH2D_SONDA_POCA, PH2D_SONDA_DIAG), ferramentas/fios.py, ferramentas/borda_cobertura.py (a régua
   da cobertura do pixel e da raiz, da W1c), fixtures/rebelle_gotas/s_*.png (o S do Rebelle a 1:1 e a 400 %).

HIPÓTESES (NÃO são conclusões — cada uma tem um teste que a mata; meça TODAS numa rodada, variantes no mesmo
processo, antes de escolher)
H1 A lei da gota (W1e) achata: ela pina a componente que desce em células SECAS olhando a célula de cima pelo `ĝ`
   ARREDONDADO (`dx, dy` inteiros — para baixo = (0, 1)), então numa orla OBLÍQUA a conta acumula até à mesma
   LINHA e pára aí: fundo horizontal. Teste: o S com `drip_head = 0` (o port) × `ALTURA_DA_GOTA`, nos três Tilts,
   mesma semente; se o achatamento só existe com a lei, é ela. Mede também em Tilt para o lado (raio 2, diagonal).
H2 O RECORTE POR EIXO da velocidade: o nivelamento é recortado por eixo (`level_clamp` em lx e ly à parte) e a
   velocidade também (`clamp_sym(ex, max_v)`, `clamp_sym(ey, max_v)`, e no `momentum_rows` nvx/nvy à parte). Um
   vector saturado nos dois eixos vira 45°; saturado num só fica preso ao eixo — formas de lados rectos. Teste: as
   mesmas cenas com `maxVelocity`/`levelClamp` altos (PH2D_SONDA_VARRE "0.005:0.2:maxVelocity=3/levelClamp=1") e
   uma variante de recorte pelo MÓDULO (o vector inteiro escalado), medida.
H3 Rectângulos de verdade: a bbox activa / a faixa viva (`span_x`, `SPAN_PAD = 5`, o rebuild a cada 2 passos), a
   grade de fluxo `rf > 1`, a janela do rasto (`Trail`) ou o rectângulo sujo do composite/da fila de lotes da W1e
   cortam a água ou a imagem numa caixa. Teste: `spans_enabled = false` (a ablação que já existe, byte-idêntica por
   desenho) e `rf = 1`, e a imagem do composite contra um render da tela inteira (`run_full`) — se a recta só
   existe no ecrã e não no grid, é composite.
H4 O fio rectangular da direita é um dedo que PAROU (a ponta perdeu a altura crítica): a largura dele é a do
   carimbo/da conta, e os lados rectos vêm de H1/H2. Teste: a forma do fio ao longo do tempo (PH2D_SONDA_DIAG).
⛔ Sem palpite: cada H com a sua imagem a 8× e o número (a régua do RECTO abaixo), e o mecanismo provado por
   mutação/ablação (o defeito desaparece SÓ quando a causa sai, e volta quando volta).

A RÉGUA DO RECTO (escreva-a primeiro, em ferramentas/, e porte-a para o gate): na orla de BAIXO de cada traço
(onde a tinta acaba a descer), a maior corrida de colunas com a MESMA linha de orla (±0 px) e a fração de cantos a
90° (orla que muda de linha em degrau ≥ 3 px de uma coluna para a seguinte); idem nas paredes laterais dos fios
(maior corrida de linhas com a mesma coluna). Calibre-a NO REBELLE (o S de fixtures/rebelle_gotas/s_*.png e a
bancada `inclinado`/`meio`): a orla dele segue a curva (corridas curtas). E no traço do dono (a captura) tem de ler
o defeito nos três sítios marcados.

BARRA (red-first: o gate nasce VERMELHO sobre a captura do dono reproduzida e o controlo do Rebelle passa):
- a régua do recto no S e na bancada nos três Tilts dentro da faixa do Rebelle (meça-a, não a invente);
- e o que a W1e conquistou NÃO volta atrás: a meia inclinação não escorre com o Wet Round e a orla de baixo
  escurece (gate `a_meia_inclinacao_nao_escorre_e_a_orla_de_baixo_escurece`), a tela direita ao byte
  (`com_a_tela_direita_a_lei_da_gota_nao_muda_um_byte`), a mistura (`azul_sobre_vermelho_vela_no_seco_e_sangra_no_molhado_como_no_rebelle`),
  e o pente não volta (`ferramentas/fios.py` na bancada: Round meia 0, máxima 0).
- Se a cura for a lei da gota: a direcção da fonte deixa de ser arredondada ao eixo (o ponto acima ao longo de ĝ
  amostrado bilinearmente, como o retro-traço do advect já faz), ou a lei sai — decida pelo número, com a tabela.

FECHO: os achados no 05 §4b (antes · depois · Rebelle, uma linha por H, com a recusa medida de cada H morta),
prancha a 1:1 e a 8× (a captura do dono reproduzida · depois · o Rebelle); commit por caminho (--no-verify); nunca
push; fábrica regerada (PH2D_REGERA_PRESETS=1) se a lei mudar. AO DONO, curto e sem jargão: o que causava as rectas,
o que mudou, e o smoke em passos numerados (o S vermelho da captura: módulo Painter, Wet Paint, Wet Round, Tilt
direito / meio / máximo).
