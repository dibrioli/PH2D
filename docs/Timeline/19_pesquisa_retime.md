# 19 — Pesquisa: como quem faz «Retime» CERTO o edita (dono 15/11)

> Ordem do dono, depois de duas auditorias: «Override OK. Retime precisa ser pesquisado em quem faz isso corretamente.»
> Estado e o que já existe: [handoff do Retime](handoffs/HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-15_RETIME_POR_TRECHO.md)
> (lei `E(W(u))`, `W` = Bézier monótona de 4 números na chave, alças AE sobre `ref·W'`, tecto `s·a ≤ 1`, ponta no ponto
> alcançável mais perto do rato). **Não reconstruir a lei antes de a pesquisa dizer que o modelo de EDIÇÃO está errado.**

## 1. O problema a resolver (o que o dono sente)

- O gesto não é «consistente e reprodutível» como no Override. Causa medida: `W` monótona tem área 1 (`∫W' = 1`), por
  isso a alça não pode ter velocidade alta E braço longo (`s·a ≤ 1`); o Override não tem tecto (a curva de valor pode
  ultrapassar). Qualquer modelo de ritmo com «tempo nunca anda para trás» tem ESTE tecto — a pergunta é como os bons
  programas o APRESENTAM ao gesto (o que a alça é, o que acontece no limite, que curva se vê).

## 2. Perguntas (responder com medição, não com leitura de fonte — CLAUDE.md §0.9)

1. Onde o ritmo de um trecho/clip é editado como CURVA DE VELOCIDADE com alças? O que é a alça (influência + taxa?
   ponto de tempo?) e o que acontece ao arrastar além do possível (a outra alça cede? a área renormaliza? sobe o fim?).
2. O tempo pode andar para trás (overshoot)? Se pode, como o easing de baixo responde (clamp? espelho?).
3. Que curva o artista vê: a taxa (`W'`), o mapa de tempo (`W`), ou a velocidade final do valor (`E'(W)·W'`)?
4. Há «Reset»/presets do ritmo (ease in/out do tempo)? Por trecho, por clip, por camada?
5. Como fica reprodutível: arrastar a alça para o mesmo ponto dá sempre o mesmo resultado (sem depender do caminho)?

## 3. Oráculos (licença por ARTEFACTO primeiro — `pacman -Qi`; método em `docs/_ComoInvestigarApps/`)

| App | Instalado (15/11) | Onde mora o retime | Licença a confirmar |
|---|---|---|---|
| Blender 5.2.3 | `blender 17:5.2.3-5` | NLA strip *Strip Time* animado (F-Curve de tempo); Graph Editor; F-Modifier; `bpy` por script | GPL — oráculo só por corrida |
| Natron 2.5 | `natron-bin` (`/opt/natron`) | nós *Retime* (speed) e *TimeWarp*/curva de tempo | GPL/MPL? confirmar |
| OpenToonz 1.8 | `opentoonz` | Xsheet: *Time Stretch*, curvas de Function Editor | BSD-3 (porta permissiva) |
| Godot 4.7 | `godot` | AnimationPlayer: *time scale*, `AnimationNodeTimeScale`, Bezier track | MIT |
| Synfig 1.4 | `synfigstudio` | *Time Loop*, `Time` param animável (time offset por camada) | GPL |
| Glaxnimate 0.6 | `glaxnimate` | (Lottie) *time stretch* de pré-composição | GPL |
| After Effects / Maya / Cavalry | não | Time Remap no speed graph (AE) · Time Warp (Maya) · referência pela documentação e capturas do dono | proprietário — só comportamento descrito |

Prioridade: Blender (NLA Strip Time + Graph Editor, e o gesto da alça na curva de TEMPO) e Natron (curva de tempo e
de velocidade lado a lado); AE pela documentação (é o produto de referência do dono para a vista de velocidade).

## 4. Entregável

- Para cada oráculo que faz: o modelo (o que a alça é), o gesto no limite, a curva mostrada — com fixtures gravadas
  das corridas por script e fotos.
- Uma tabela «modelo × o que resolve no nosso problema §1» e UMA recomendação; o dono escolhe (maquete/fotos lado a lado
  antes, `feedback_a_mockup_is_compared_side_by_side_before_the_smoke`).
- Só depois: mudar `retime_with_speed_handle`/a pintura; a lei `E(W(u))` e o formato ficam salvo prova em contrário.

## 5. Resultado (15/11) — medido; fixtures em [`oraculos/retime/`](oraculos/retime/README.md)

[Folha lado a lado](fotos/retime_8_pesquisa_lado_a_lado.png) · [Blender, foto do Graph Editor](fotos/retime_9_blender_tempo_volta.png).

| App | Fonte | O que a alça é | O gesto além do possível | Curva que se vê |
|---|---|---|---|---|
| Blender 5.2.3 | corrida (`blender.json`) + foto | ponto `(t, tempo)` na F-Curve do *Strip Time* | Y sem tecto: alça `(0,9; 2×)` ⇒ o tempo vai a **1,051** e volta (não monótono); X além do trecho é encurtado na avaliação, mantendo a direção | o mapa de tempo (valor) |
| Godot 4.7.2 | corrida (`godot.json`) | ponto relativo `(Δt, Δv)` da chave Bezier | **o mesmo número do Blender** nos 8 gestos (ex.: `(1,5; 2×)` ⇒ máx 1,552) | o valor |
| Natron 2.5 | corrida (`natron.json`) + ajuda dos parâmetros | `speed` animável, **integrado** desde o início; `warp` = curva extra entrada→saída | velocidade 1→3: o clip acaba no quadro **50** (não 100) e congela; 1→−1: **toca para trás** (13 → 1) | velocidade, ou o warp |
| After Effects | documentação (Keyframe Velocity, speed graph) | velocidade + influência (o nosso modelo de hoje) | o speed graph cruza o zero = o valor passa e volta (Time Remap anda para trás) | velocidade ou valor |
| Premiere / Resolve (Speed) | documentação | velocidade do trecho (faixa de borracha) | **a duração do clip muda** (ripple ou sobrescreve) | velocidade |
| Resolve (Retime **Frame**) | documentação | Bézier sobre a diagonal quadro-de-origem × tempo | o limite é a própria caixa; segmento invertido = para trás | o mapa de tempo, com a de velocidade ao lado |
| OpenToonz 1.8 | **não corrido** (sem via de script nas curvas) | — | — | — |

**Achado que decide.** Ninguém tem ao mesmo tempo *chaves fixas* + *tempo nunca para trás* + *alça de velocidade sem
tecto* — é impossível (área 1, §1). Cada um larga uma: Blender/Godot/AE largam «nunca para trás»; Natron/Premiere/Resolve
Speed largam «chaves fixas». Os que mantêm as duas mostram a **curva do tempo** numa caixa (Resolve Frame; o editor de
`cubic-bezier` do CSS). E o nosso dado JÁ É esse: `Retime{x1,y1,x2,y2} ∈ [0,1]⁴`, `y1 = s·x1`. O tecto `s·a ≤ 1` é só a
borda de cima da caixa vista no espaço errado — na vista de velocidade vira a hipérbole `s = 1/a`, invisível, e a ponta
«escorrega» (handoff, 2.ª volta).

**Recomendação (UMA, para o dono escolher):** em Retime, a curva âmbar passa a ser o **tempo do trecho**, de chave a
chave numa caixa tracejada; as duas alças são pontos dentro dela e, além da borda, ficam **na borda, debaixo do rato**
(projeção na caixa: o mesmo ponto do rato dá sempre o mesmo resultado, sem depender do caminho). A velocidade âmbar
continua desenhada, só para ler. Formato e lei `E(W(u))` **intactos**; muda só o gesto e o desenho em Retime
(`retime_with_speed_handle`/`alcancavel` saem, a pintura de `graph_ritmo.rs` troca de espaço). Override não muda.
Alternativas que o dono pode preferir: (B) ficar como está; (C) Natron/Premiere: velocidade livre e a chave seguinte
anda; (D) Blender/AE: sem tecto, o tempo pode voltar (o Bounce tocaria de trás para a frente por um instante).
