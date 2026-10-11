TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), AUDITORIA MULTI-AGENTE + CURA DEFINITIVA da MISTURA DE COR
do Wet Paint («Pigment mixing (K-M)» e «Glaze layering (K-M)»): várias passadas de azul escuro com o Wet Drip, as duas
caixas ligadas, criam manchas de AZUL-CLARO/CIANO que nenhuma das duas tintas tem — nas pontas dos fios, numa faixa
pontilhada na orla de baixo, em fios pixelados e num halo claro na fronteira azul/vermelho. Primeiro a CAUSA medida
célula a célula; depois, se a medição o pedir, uma SIMPLIFICAÇÃO da representação da cor que acabe com a família
inteira (§0.10), não o 6.º remendo.

⭐ **O dono pediu VÁRIOS AGENTES** (10/10: «escreva um prompt para outra janela fazer auditoria com vários agentes»):
isto é a autorização explícita para o `Workflow` (multi-agente). Carregue a skill `workflow-authoring` antes do script.

Smoke do dono (10/10, sobre `c11acc01e` — a claridade pelo PIGMENTO e a lei `Cobre`): *«várias passadas de azul
escuro com pigment mixing e Glaze Layering com pincel wet drip criando manchas inadequadas de azul claro»*. Capturas
(LEIA-AS, são a especificação do defeito), em
[fixtures/smoke_dono_mistura_wet_drip/](fixtures/smoke_dono_mistura_wet_drip/):
1. `1_pontas_dos_fios_ciano.png` — fios de azul escuro a escorrer sobre um traço vermelho: a PONTA de cada fio, onde
   toca o vermelho, é um ponto ciano-claro.
2. `2_faixa_ciano_na_orla_de_baixo.png` — azul por cima do vermelho molhado: pontos ciano nas pontas e uma FAIXA
   pontilhada ciano-claro ao longo da orla de baixo da mancha, mais clara que as duas tintas.
3. `3_fios_ciano_pixelados.png` — os fios do azul sobre o vermelho saem ciano-vivo com textura em BLOCOS (degraus de
   célula), com orla escura fina.
4. `4_halo_claro_na_fronteira.png` — uma banda esbranquiçada na fronteira entre o azul e o vermelho, mais clara que
   ambos.
Comum às quatro: aparece onde o azul ESCURO e o vermelho se encontram com tinta MOLHADA e escorrimento (Wet Drip:
água 1, Tilt no máximo, pigmento 63), com as DUAS caixas K–M ligadas; o resultado é mais CLARO e mais saturado que
qualquer das tintas — numa mistura subtrativa isso não pode acontecer (o pigmento só tira luz).

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -4 && git status --short → a ponta é este prompt («docs(Painter 10/10): o prompt da auditoria
   multi-agente da mistura …»), com c11acc01e logo abaixo; árvore limpa.
3. git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
4. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por bash scripts/ph2d-run.sh; o
   hook recusa «cargo test» fora da porta: o script vai num ficheiro (Write) e corre `bash scripts/ph2d-run.sh bash
   <ficheiro>`. A shell é fish/zsh: laços e arrays em bash vão no ficheiro. ⛔ Os agentes de um workflow partilham a
   MESMA árvore e o mesmo `target/`: só UM agente compila/testa de cada vez (os outros LEEM); mutações só pelo
   agente `mutacao`, em série, com `touch` no restauro.
5. Cercas verdes ANTES e DEPOIS: ferramentas/cerca.sh (com `cerca_isencoes.tsv`) · a suíte de ph2d-wet-paint (as
   impressões digitais do port NÃO movem — `Lei::Porto` e `pigmento = 1` são o port ao bit) · o censo dos controlos
   (`cargo test -p ph2d-panel-registry-init --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`
   e `cargo test -p ph2d-panel-painter-layers`) · a ponte do preview SÓ com `--features panel-painter-layers` no
   `ph2d-app-painter` · `--lib wetpaint` e `--test it` do ph2d-tool-painter (antes deles, os presets de fábrica:
   `PH2D_REGERA_PRESETS=1 … --test it gera_os_presets_de_fabrica -- --ignored`) · ph2d-i18n · `cargo fmt --check` e
   clippy `--all-targets -D warnings` nas crates tocadas. Num só ficheiro de bateria, uma corrida por rodada.

LEIA (só isto): DIRETIVA_IMPLEMENTACAO.md (§3 o template de auditoria, §4 olhar a 8×, §5) · 05 §4b as entradas
«Mistura» (prompt 15), «Assentar não muda o que se vê», «A claridade é o PIGMENTO» (a lei `Cobre`, as recusas, os
portões e as 7 mutações) · BUGS_painter #47 (o portão do papel é uma subtracção ABSOLUTA — o mesmo mecanismo pode
estar aqui). O código, pelos agentes (Fase A), não por você.

O QUE SE SABE (factos do código, NÃO a causa — meça antes de acreditar):
- A cor de uma célula NÃO tem uma representação única. Há pelo menos: a cor «straight» da suspensão (`susp_rgb`) e do
  assentado com a MASSA ao lado; a cobertura `1 − 0,998^m`; a média K–M ponderada pela massa
  (`colorops::km_weighted_mean_color`) usada no pouso, na advecção e no `sedimenta` quando `km_mixing`; a velatura
  (`colorops::velatura`, hoje a 0 no produto); e QUATRO leis do ecrã em `assenta.rs` (`Porto` · `PorCima` · `Camada`
  — a do «Glaze layering» — · `Cobre`), cada uma com a sua inversa para as transferências verticais (assentar, Dry
  canvas, re-molhar: palpite físico + Newton numérico; a `Camada` por fórmula fechada e pela raiz de uma quadrática).
- O render (`render.rs`, `assenta::cor_e_alfa`) e o composite/Smooth Edges da ferramenta (`wetpaint/composite.rs`,
  `smooth_edges.rs`) des-premultiplicam e PRENDEM a 0..255 (`molha`: `(v − 255·(1 − la))/la`, com `la` pequeno nas
  pontas e orlas).
- O Heavy Edge (`solver/sedimenta.rs`) amontoa o pigmento na última água da orla de BAIXO — exactamente onde estão os
  pontos e a faixa das capturas 1 e 2 — e mistura as cores pelo K–M quando `km_mixing`.
- Cada cura das janelas 15 e da claridade foi medida e está gateada; NENHUM portão mede «uma mistura nunca é mais
  clara que as tintas» nem «sem células soltas» (pontilhado), nem o Wet Drip com as duas caixas ligadas.

HIPÓTESES A MEDIR (cada uma: a cena, o número, o veredicto — a tabela H do 05 §4b como modelo):
H1 **des-premultiplicar com `la` pequeno** (render `Camada`/`Cobre`, o composite, o Smooth Edges): ruído de massa
   amplificado e preso a 0..255 dá células claras soltas (o pontilhado e os blocos).
H2 **a inversa da `Camada`** (Glaze) escolhe a raiz errada ou prende fora da gama em células de filme fino/massa
   ínfima: o assentar escreve uma cor CLARA escondida que aparece quando o filme seca (já aconteceu na `PorCima`:
   «+24 de verde ao secar», 05 §4b).
H3 **K–M sobre CORES e não sobre concentrações**: misturar `susp_rgb` (já com a claridade da diluição) pela K–M dá a
   matiz de um pigmento DILUÍDO (o azul escuro pouco carregado é ciano) — a cor da mistura sai da gama das duas.
H4 **o `sedimenta` com K–M** na orla de baixo e nas pontas dos fios (capturas 1–2): meça com o Heavy Edge 0 × ligado,
   K–M 0 × 1.
H5 **a advecção/pouso misturam pesos de massa com cores de leis diferentes** (a suspensão numa lei, o assentado
   noutra): o halo claro da captura 4 na fronteira.
H6 **a grelha** (razão > 1, amostrador): os blocos da captura 3 podem ser decisões por célula visíveis em degraus.
H7 **o pincel sujo** (Pickup do vermelho molhado) — controlo, não suspeito principal.

PLANO (Workflow; < 10 agentes, o tamanho da sessão — diga o custo ao dono ANTES de correr se passar disso):
A. **Inventário em paralelo** (agentes `explorador`, só leitura, um por zona): (1) água — advecção, difusão, pouso,
   `km_weighted_mean_color`; (2) secagem e transferências verticais — `drying.rs`, `assenta.rs`, `grid/canvas_ops.rs`
   (Dry canvas), `trail/transfer.rs` (pickup); (3) `solver/sedimenta.rs` e o Live Water; (4) o ecrã — `render.rs`,
   `wetpaint/composite.rs`, `smooth_edges.rs`, o preview. Cada um devolve a TABELA: sítio (`ficheiro:linha`) · que
   representação de cor lê e escreve (straight / premul / massa / K–M / lei de ecrã) · onde prende ou divide · que
   caixa o liga. A janela principal junta as quatro: é o mapa de CONVERSÕES de cor do motor.
B. **Reprodução headless** (um agente, o único que compila nesta fase): a cena das capturas — uma tela branca, um traço
   vermelho com o Wet Round, depois 3–5 passadas de azul escuro (≈ `[20, 50, 110]`) com o **Wet Drip**, as duas caixas
   ligadas, molhado sobre molhado e sobre seco. Fotografe 1:1 e 8× (`ferramentas/`), confirme os quatro sintomas
   (sem os ver na sua tela, não há auditoria), e grave, para as células defeituosas, o estado do motor (`susp`, `sett`,
   as cores, `film`, `a_s`, `a_f`, a lei) e o PRIMEIRO passo em que a cor sai da gama (instrumente por passo, não pelo
   resultado — «onde ATERRA ≠ o que PROVOCA»).
C. **Uma rodada de ablações** (§0.10, as variantes no MESMO processo): H1–H7 ligadas/desligadas por override em
   execução, cada uma com o nº de células fora da gama e o índice de pontilhado na mesma cena. Série só se o resultado
   de um passo mudar o seguinte — e diga-o.
D. **Desenho da cura — e a pergunta da SIMPLIFICAÇÃO** (agente de desenho + um agente ADVERSÁRIO que tenta partir a
   proposta com as cenas da B e os portões existentes). A pergunta a responder com números: *uma representação ÚNICA da
   cor resolve a família?* A candidata a medir primeiro: a cor de cada camada como **densidade óptica por canal**
   (absorvância `A = −log T`, ou o K/S da K–M) × MASSA — misturar na água = soma ponderada; pôr uma camada por cima =
   soma; secar/assentar = mover massa sem converter (o ecrã preserva-se por construção, sem inversa nem Newton); o ecrã
   = UMA função no fim. Ela tornaria as quatro leis, a velatura e as inversas desnecessárias, e o «Glaze layering» e o
   «Pigment mixing» passariam a ser a MESMA física (a fronteira entre eles seria só se a camada de baixo re-dissolve).
   Meça-a contra TODAS as barras de hoje antes de a adoptar: W1e (γ seco 0,42–0,49 · 0,44–0,51, sangria molhada
   0,27), camadas (0,89 · 0,96), a acumulação com a Opacity baixa, a tinta molhada mais forte (Round 111 → 54), o tom
   ao secar (≤ 0,15; Rebelle 0,05), e o «nunca mais claro que as tintas». Se ela perder uma barra, diga qual e porquê;
   uma cura local (H1–H7) que feche os sintomas sem a simplificar também é resposta legítima — escolha pelo padrão-ouro,
   com os números lado a lado.
   ⚠️ **Decisão de PRODUTO, não sua:** se a simplificação tornar uma das caixas («Pigment mixing», «Glaze layering»)
   redundante, NÃO a tire — pergunte ao dono, sem jargão, com o que ele ganha e perde em cada opção.
E. **Implementação + portões** (um agente; red-first, com o controlo DENTRO de cada portão):
   - `uma_mistura_nunca_e_mais_clara_que_as_tintas` — na cena da B, nenhuma célula pintada com transmitância (luz
     linear, por canal) acima da da tinta mais clara das que ali pousaram (tolerância medida e escrita); controlo: a
     variante defeituosa de hoje reprova.
   - `sem_celulas_soltas` — o índice de pontilhado (células a > N níveis da mediana 3×3 dentro da tinta) abaixo da
     barra medida no Rebelle; controlo: hoje reprova.
   - as barras de hoje continuam verdes (05 §4b «A claridade é o PIGMENTO»: os portões e as 7 mutações).
F. **Mutações** (agente `mutacao`, em série): cada peça da cura ⇒ um portão VERMELHO; sobrevivente = portão a reforçar
   (aconteceu na janela anterior: «o tapar sem a diferença de pigmento» sobreviveu ao 1.º portão).

A BARRA (oráculo, CLAUDE.md §0.9): o Rebelle 8.3.4 pela porta WebSocket (`ferramentas/mistura_papel*.{py,sh}`,
`velatura.py`, `camadas*.py`), a MESMA cena (vermelho, depois azul escuro escorrido por cima, molhado e seco). A barra é
o que ele faz: as pontas dos fios e a orla de baixo ESCURECEM (nunca ciano-claro), a fronteira não clareia, sem
pontilhado. Grave as fotos como fixture com cabeçalho e compare por passo. O que foi medido e recusado não se
reconstrói (05 §4b: a diluição, escalar o carimbo, a velatura por cima da K–M, a derivada assumida no Newton).

CERCAS DE BYTES: as impressões do S/bancada (`opcoes_tests`), do Smooth Edges no zero (`grid_ratio_tests`) e as 4
miniaturas de fábrica PODEM mudar se a representação mudar — regrave-as UMA vez, com a razão escrita (o precedente
do 05 §4b), nunca para «passar». As do port (`ph2d-wet-paint`, `Lei::Porto`) NÃO podem mudar.

FECHO: 05 §4b entrada nova «Mistura II — a representação da cor» (o mapa de conversões da Fase A, a tabela H com os
números, a decisão D com as alternativas e as barras lado a lado, antes · depois · Rebelle, as recusas medidas);
prancha 1:1 e 8× (as quatro capturas do dono reproduzidas · depois · Rebelle); commit por caminho (`--no-verify`;
`git add` explícito para os novos); nunca push. AO DONO, curto e sem jargão: o que causava o azul-claro, o que mudou,
o que (se algo) ele tem de decidir sobre as caixas, e o smoke em passos numerados (o comando com o `cd`; o traço
vermelho; o Wet Drip azul escuro por cima várias vezes com as duas caixas ligadas; o que tem de acontecer — as pontas
e a orla escurecem, nada fica mais claro que as tintas, sem pontinhos; como saber que deu errado).
