# 15 — Redesenho da Timeline no molde do Rive (linha `line/TimelinePrincipal`)

> Fonte: a crítica `~/Documentos/GameEngineSamples/critica_ph2d.html#timeline` (secções 1–7, maquete **B**).
> Ordem do dono (09/11): **redesenhar sem mudar funções** — melhor aparência, organização e intuitividade.
> Decisões do dono (09/11): **versão B** (chips de clipes, à Rive); **«Keys | Curves» fica de fora** por ora.

## Regra da linha

Só aparência e organização. Todo controlo de hoje continua a existir, com o MESMO id e o MESMO evento — muda de
sítio, de cara e de nome. Nada de função nova. Fica de fora (é função nova, ver a crítica §4): tocar só o trecho
entre marcadores, o modo «Animating · Done», «quem ouve» o sinal, o losango de 3 estados no Inspector, arrastar a
chave no canvas, abrir o container no lugar (C), «Keys | Curves».

Só a barra em cartões (`transport_cartoes.rs`, o caminho por omissão) muda. `PH2D_CARDS=0` continua a pintar a
barra clássica de `transport.rs` como hoje — é a saída de referência.

## Onda 1 — a barra

Hoje: 32 comandos em duas fileiras. Alvo (maquete B1): três fileiras curtas, cada uma com um assunto.

| Fileira | Conteúdo | Ids (os de hoje) |
|---|---|---|
| Título | `TIMELINE` · abas Keys/Containers/Arrange · migalha · à direita: dono «Hero ▾» · engrenagem «Timeline options» · × | `TIMELINE_TAB_*`, `TIMELINE_CRUMB[n]`, `TIMELINE_ANIM_DD`, novo id da engrenagem, close |
| Clipes | «Clips» · um chip por clipe (o que começa leva ▶) · «+» | as opções de `TIMELINE_CLIP_DD`, `TIMELINE_ADD_CLIP` |
| Tocar | 5 de tocar · Time (s) · Frame · «24 fps · project» (só leitura) · Auto Key · Record while playing · marcador | os de hoje |

- **Menu do clipe** (botão direito no chip, ou «⋯» no chip ativo): Rename, Duplicate, Reverse keys, Starts with this
  clip (o Autoplay), Delete — os mesmos eventos de `TIMELINE_RENAME_CLIP`, `_DUP_CLIP`, `_REVERSE_CLIP`,
  `_CLIP_AUTOPLAY`, `_DELETE_CLIP`. O «I» e o «T» saem da barra.
- **Timeline options** (popover da engrenagem): «When the clip ends: Stop · Loop · Ping-Pong» (os eventos de
  `TIMELINE_LOOP`/`_PINGPONG`), Length (s) (`_LENGTH_NUM`), Snap to frames, Run physics during Play, A position key is
  a point on the path, Show speed in the curves (`_SPEED`), e os fantasmas: Ghosts, Only at keys, Ghost settings
  (abre o cartão de hoje, `_ONION_SETTINGS`).
- **Auto Key** ligado acende a borda da fita (onda 2) e a marca vermelha.
- Containers e Arrange: a fileira de chips mostra as opções da aba (`transport_clips::source_options`).

Contagem alvo no estado Keys: abas 3 + dono 1 + engrenagem 1 + fechar 1 + chips N + «+» 1 + tocar 5 + Time + Frame +
Auto Key + Record + marcador = **17 + N** (a crítica conta 21 com 3 clipes).

## Onda 2 — a fita

- Nomes das trilhas em árvore objeto › propriedade, com a palavra e a unidade do Inspector («Position X (px)»,
  «Rotation (°)»), nunca cortados ao meio de dois iguais.
- A barra entre duas chaves com a cor da interpolação (curva · reta · segura, tracejada), e a legenda no pé — tokens
  novos `TimelineInterp*` (append em `ph2d-tokens`). O losango fica.
- Auto Key ligado: a borda da fita vermelha (token novo, alias de `Danger`).
- Depois do fim do clipe: a fita listrada (hoje escura).
- Abrir a Timeline (ou trocar de clipe) enquadra o clipe — o `F` de hoje, sozinho.

## Onda 3 — Containers e Arrange

Chips das peças (com «used 2×» / «not placed»), lanes com peso e olho na coluna, o mesmo desenho de strips. Sem
gesto novo.

## Provas

Cada controlo que muda de sítio ganha (ou mantém) um teste de costura que clica o id pintado e confere o efeito
(`tests/it/`). Fotos de cada onda lado a lado com a maquete B antes do smoke do dono.
