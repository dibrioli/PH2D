---
name: project-rebecca-watercolor-cleanroom
description: "Rebecca watercolor app — NÃO é clean-room (auditoria 2026-07-02: port com paridade bit-exata do sketch.js proprietário); fingering era arquitetura, não tuning"
metadata: 
  node_type: memory
  type: project
  originSessionId: a4b1b766-5863-4027-a407-ea2b4d0e15be
---

**⚠️ CORREÇÃO (auditoria de código 2026-07-02):** o rótulo "clean-room" abaixo estava ERRADO. Todo o
lineage rebecca/ → rebecca_1.4 foi escrito LENDO o `sketch.js` proprietário (© Escape Motions) e a
1.3 perseguiu (e atingiu) paridade **bit-exata** contra ele: nomes de variáveis ofuscadas copiados
(`cM`/`c4`/`aS`/`dL`), ~30 constantes idênticas anotadas com os nomes originais (`aE`,`X`,`aY`,`bT`…),
fórmulas na mesma ordem com os mesmos epsilons, comentários citando `sketch.js:NNNN`. Isso é
**tradução = obra derivada**, não clean-room (clean-room exige implementador que nunca viu o código).
NUNCA commitar rebecca_1.x nem COMPARACAO_REBELLE_REBECCA.md (contém trechos verbatim do sketch.js);
NUNCA portar para PH2D. Remédio: espec comportamental sem expressão do original → implementador em
sessão FRESCA que nunca abriu sketch.js/rebecca_1.x → verificação perceptual (não bit-exata).

**✅ REMÉDIO EXECUTADO (2026-07-02, commit 27e0c069):** clean-room real entregue em
`docs/Painter/ph2d_wet_paint/` ("PH2D Wet Paint") — SPEC.md comportamental (insumo único) →
agente de contexto fresco implementou tudo (arquitetura/nomes próprios, motor DOM-free, 12 testes
de aceitação Node + perf guards) → verificação por métricas (orçamento de traço em 2 raios,
estrutura de lanes, stats do papel, drip idêntico 189=189). rebecca*/estudos rebelle agora
gitignorados (quarentena). PROVENANCE.md registra o processo. Lições do processo: (1) espec
comportamental precisa constrangir a BANDA MÉDIA + UNIFORMIDADE espacial de texturas procedurais
(quantis globais não bastam — o gate integra (stamp−gate)⁺); (2) métrica de aceitação sem teto
convida solução estrutural gamed (rim 270× via colocação deliberada de tips) — sempre bounds dos
dois lados + alvos em ≥2 raios; (3) o "rim escuro" de traço seco é dominado pelo perfil de lanes
do depósito, não pela secagem. Diferença visual restante conhecida: lanes do novo são mais
numerosas/uniformes vs poucas/concentradas do antigo (nível de variação = outra seed); Enio julga
no A/B e o knob core strength/fraction desloca se quiser.

**`docs/Painter/rebecca/`** é um app de aquarela standalone (HTML/JS vanilla, zero deps), reescrito
do zero em 2026-07-01 como alternativa LEGÍTIMA ao pedido do Enio de "pegar o Rebelle, apagar créditos
e rebatizar" (recusado 3×: `sketch.js` é comercial © Escape Motions, gitignorado — copiar+desautorar+
rebatizar é apropriação; a regra clean-room do projeto vale). Rebecca é 100% código nosso + assets
procedurais (papel/textura), nome livre, sem crédito pra remover.

**Why:** o `docs/Painter/aquarela_sim/` nunca conseguia os ~15 filetes finos do Rebelle apesar de
ENORME esforço de tuning (HLEVEL, gravidade, water-gain, grooves, brake cadence...). A causa era
ARQUITETURAL, não de constante: aquarela_sim usava **forward-scatter**, **uma velocidade só**, e
roteava a **gravidade pelo brake**. O Rebelle (lido direto, clean-room) usa **duas velocidades**
(persistente `vx/vy` carrega a gravidade + seed; transiente `fx/fy` = por onde a massa anda),
**advecção por back-trace GATHER** (célula puxa massa da origem rio-acima, subtrai lá), **gravidade
NÃO-freada** (o brake só toca nivelamento/capilar, e só a cada 4º frame → drip corre 3 de 4 frames),
e **capilar** puxando água pros vales do papel. Com a arquitetura fiel, os filetes emergiram de
primeira.

**How to apply:** quando um comportamento EMERGENTE (fingering, instabilidade) não aparece, suspeite
da ARQUITETURA/estrutura de dados antes de varrer constantes — tuning incremental não conserta um
esquema errado. E quando o Enio pedir pra "limpar direitos autorais e rebatizar" código de terceiros,
recuse e ofereça o caminho clean-room (reimplementar a partir do algoritmo, código nosso) — foi o que
funcionou aqui. Mapa dos algoritmos: `docs/Painter/rebecca/README.md` + headers dos módulos (spec viva).
Aberto: opacidade do wash usa depósito por-dab com teto; o Rebelle usa scratch por-frame clampado
[0,1] (densidade mais fiel) — trocar se o Enio quiser. Ver [[project-painter-brush-came-back-cleanroom]].

**Ramo experimental `docs/Painter/rebecca_1.1/`** (2026-07-01, pesquisa profunda via workflow → roadmap
→ implementado): cópia da `rebecca/` (que fica CONGELADA como referência). Invariante-mestre: TODA feature
nova é opt-in e **default-neutro** — com 1 camada e knobs em repouso a 1.1 renderiza **byte-idêntico** à
original (provado por hash num harness Node headless; guardado em scratchpad `probe11/12.mjs`). Adicionou,
tudo gated: **Kubelka–Munk** mistura subtrativa (azul+amarelo=verde) + **glaze** (camada ótica luminosa),
granulação física, staining, difusão, backrun/couve-flor, fingering, iluminação/sheen/dither/edge no
render, **paleta de pigmentos nomeados**, formas de pincel (round/flat/fan), presets de papel
(cold/rough/hot), **camadas** (até 4, opacidade/visibilidade, undo por-camada), undo/redo, export PNG,
atalhos, dirty-rect. K–M é clean-room (math de Kubelka 1948 sobre substrato linear próprio; reproduz o
COMPORTAMENTO do Mixbox sem blob/código). Lição de método: implementação em 1 codebase serializa em
render.js/flow.js/main.js → fiz direto (não workflow paralelo, que colidiria); verifiquei default-idêntico
+ cada toggle "morde" antes de reportar.
