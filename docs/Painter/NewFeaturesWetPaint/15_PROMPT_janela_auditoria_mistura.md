TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), AUDITORIA + CURA da MISTURA DE PIGMENTOS: amarelo sobre azul
seco dá PRETO, e por mais amarelo que se ponha nunca pára de reagir — vira verde e o amarelo nunca cobre. Primeiro a
CAUSA medida célula a célula, depois a cura (§0.10: todos os casos da mesma família num bloco).

Smoke do dono (10/10, sobre `711513d42`, o resto da W1g aprovado: «smoke ok»): *«Pigment mixing está errado. Na imagem
tínhamos uma mancha azul seca. O amarelo sobre o azul produz preto e por mais que se coloque tinta amarela, nunca
para de reagir e transformar em verde e o amarelo nunca chega a cobrir o azul e o verde.»* Captura:
[fixtures/smoke_dono_w1g/amarelo_sobre_azul_seco.png](fixtures/smoke_dono_w1g/amarelo_sobre_azul_seco.png) — uma
mancha azul (seca) com fios; por cima, amarelo em várias passadas: o miolo ficou PRETO, à volta verde-escuro, e os
fios do amarelo que escorrem sobre o azul saem verdes (isso é o esperado num pigmento transparente). O defeito: o
PRETO e o «nunca cobre» — cada passada de amarelo afunda a cor em vez de a levar para o amarelo/verde-claro.

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -4 && git status --short → a ponta é este prompt («docs(Painter 10/10): o prompt da auditoria da
   mistura …»), com 711513d42 (o 5.º smoke da W1g) logo abaixo; árvore limpa.
3. git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
4. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por bash scripts/ph2d-run.sh; o
   hook recusa «cargo test» fora da porta: o script vai num ficheiro (Write) e corre `bash scripts/ph2d-run.sh bash
   <ficheiro>`. A shell é fish/zsh: laços e arrays em bash vão no ficheiro (`for par in "a b"` NÃO parte em zsh).
5. Cercas verdes ANTES e DEPOIS: ferramentas/cerca.sh (com `cerca_isencoes.tsv`) · os_outros_meios_nao_mudam · a suíte
   de ph2d-wet-paint (as impressões digitais do port NÃO movem) · o censo dos controlos (`cargo test -p
   ph2d-panel-registry-init --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`) · os
   testes da ponte do preview SÓ com `--features panel-painter-layers` no `ph2d-app-painter` (sem ela não compilam e
   o verde é vácuo — o clippy idem) · `opcoes_tests` (as cercas de bytes da W1g) · `mistura_tests`.

LEIA (só isto): DIRETIVA_IMPLEMENTACAO.md (§3 o template de auditoria, §4 olhar a 8×, §5) · 05 §4b as entradas «W1e»
(F6: a velatura, `VELATURA_MEDIDA` 0,47, γ em luz linear, a barra do Rebelle) e «W1f» (`VELATURA_DO_FILME`) e «W1g»
(`VELATURA_DO_FILME_VIVA`, o papel sem desenho) · `crates/ph2d-wet-paint/src/colorops.rs` (`velatura`, as três
constantes e as tabelas delas) · os sítios onde a velatura entra (05 §4b W1e F6: «pouso sobre suspenso, assentamento,
Dry canvas, re-wet, render») · `wetpaint/mistura_tests.rs` (a régua γ e a sangria, o controlo dentro).

O QUE SE SABE (factos do código, NÃO a causa — meça antes de acreditar):
- `colorops::velatura` subtrai, a cada JUNÇÃO de duas tintas, `k·cima·(1 − baixo/255)` com `k = w·glaze·a_baixo·dif`,
  em sRGB 0..255 (não em luz linear), e a junção acontece em muitos sítios por traço (cada pouso, cada passe de
  secagem, o render). O `baixo` de uma junção pode ser o resultado de junções anteriores.
- A régua que calibrou a velatura (W1e F6) é UMA camada de azul sobre vermelho, durante e 2 s depois — nunca N
  camadas da MESMA cor clara sobre uma escura. Um gate verde ali não diz nada sobre a convergência.
- Há também o pincel sujo (`Pickup`, o pincel apanha tinta assentada) e a mistura K–M opcional (`km_mixing`, desligada).

HIPÓTESES A MEDIR (cada uma com a cena, o número e o veredicto, a tabela H do 05 §4b W1f como modelo):
H1 a velatura COMPÕE-SE: aplicada em cada junção sobre um `baixo` que já é mistura, ela não converge — a cor desce a
   cada passe e nunca sobe para a de cima (o preto). Meça a cor do miolo passada a passada (1…10 camadas de amarelo
   sobre azul seco) com a velatura a 0 · produto · 1, e onde (que junção) ela desce.
H2 o termo em sRGB em vez de luz linear / sem tecto: `cima·(1 − baixo/255)` pode tirar mais do que a física de
   Beer–Lambert tiraria (o resultado pode ficar abaixo de `min(baixo, cima)` canal a canal — o preto).
H3 o pincel sujo: o amarelo apanha azul a cada dab e pinta verde «para sempre» (o «nunca pára de reagir»). Meça com
   `Pickup` 0 × produto.
H4 a cobertura: com a massa assentada a crescer, o `w = a_cima/A` da junção nunca deixa a de cima ganhar (o «nunca
   cobre»). Meça o `w` efectivo por passada.
H5 o filme molhado no render (`glaze_filme`) sobre o assentado escuro (o miolo preto pode ser render, não pigmento —
   compare o pigmento do motor com o pixel do ecrã).

A BARRA (oráculo, CLAUDE.md §0.9): o Rebelle 8.3.4 pela porta WebSocket (`ferramentas/mistura_papel*.{py,sh}`,
`velatura.py`), a MESMA cena: uma mancha azul seca e 1…10 passadas de amarelo (Watercolor, água do preset), a cor
do miolo por passada. A barra é a trajectória dele: (a) nunca mais escura que a mais escura das duas tintas (sem
preto); (b) converge — cada passada move a cor para o amarelo-esverdeado, e N passadas ficam perto do que o Rebelle
dá; (c) o caso de UMA camada (a W1e) continua na barra de hoje (`azul_sobre_vermelho_vela_no_seco_e_sangra…`). Se o
Rebelle disser que em aguarela transparente o amarelo NUNCA cobre o azul por completo, isso é a lei — o defeito é o
preto e o não-convergir, não a transparência. Diga ao dono, sem jargão, o que o Rebelle faz.

PROTOCOLO: §0.10 — as hipóteses e as curas candidatas medem-se NUMA rodada (variantes no mesmo processo); red-first:
o gate novo («N camadas de amarelo sobre azul seco convergem sem preto, na trajectória do Rebelle») nasce VERMELHO no
produto de hoje, com o controlo DENTRO; mutação da cura ⇒ vermelho. As cercas de bytes da W1g (`opcoes_tests`) e as
miniaturas de fábrica PODEM mudar se a lei da mistura mudar — regrave-as UMA vez, com a razão escrita (o precedente
do papel, 05 §4b W1g), nunca para «passar». O que foi medido e recusado não se reconstrói (05 §4b).

FECHO: 05 §4b «Mistura» (a tabela H com os números, a cura, antes · depois · Rebelle, as recusas medidas); prancha 1:1
e 8× (antes · depois · Rebelle, passada a passada); commit por caminho (--no-verify; `git add` explícito para os
novos); nunca push. AO DONO, curto e sem jargão: o que estava errado, o que mudou, e o smoke em passos numerados (a
mancha azul seca, o amarelo por cima várias vezes, o que tem de acontecer, como saber que deu errado).
