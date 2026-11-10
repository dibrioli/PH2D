# 17 — Funções novas da Timeline (linha `line/TimelinePrincipal`)

> Fonte: a crítica `~/Documentos/GameEngineSamples/critica_ph2d.html#timeline` §4 «Acrescentar», maquetes B4–B7 e C2–C4,
> e o que ficou de fora do [plano 15](15_plano_redesenho_molde_rive.md) e da [onda 3](16_onda3_containers_arrange.md).
> Dono (10/11): «escolha e siga» — a ordem é a proposta abaixo (barato primeiro; o grande sozinho no fim).

## Regra

Função nova nos **cartões** (caminho por omissão); `PH2D_CARDS=0` pinta e responde como hoje, salvo onde a função é do
motor (ex.: o trecho em loop toca igual nos dois). HR-15 (tokens, i18n em inglês). Cada gesto novo ganha teste de costura
que clica o id PINTADO e confere o efeito. Fotos lado a lado com a maquete antes do smoke do dono.

## Inventário medido (10/11, leitura do código)

| # | Função | Já existe | Falta | Custo |
|---|---|---|---|---|
| 1 | Contorno da peça escolhida | anel na pintura (`strip_paint`), seleção não-desfazível é estado de painel (`intent.rs:323`) | `selected_strip` no estado, clique sem arrasto seleciona, vazio/Esc limpa | P |
| 2 | Tocar só um trecho (B6) | marcadores com nome (`doc.rs`), loop vivo do `Playhead` (`ph2d-core/src/playhead.rs:61,99`), `view_clock.rs:83` | o trecho TRANSITÓRIO (não gravado, sem undo) que sobrepõe o loop do clipe; o gesto; a etiqueta «takeoff → top» + ✕ | P |
| 3 | Losango de 3 estados | `KeyMark::{Livre,Animado,Chave,Mudado}` (`editor-core/.../key_mark.rs:23`) | `Livre` é um ponto: passa a losango vazio apagado; `Animado` = contorno vivo (maquete B1) | P |
| 4 | Marcas das voltas do loop | `StripView.loop_mode` (`snapshot.rs:138`) | o comprimento de UMA volta (fonte × velocidade, derivado) e a pintura dos riscos | P–M |
| 5 | Keys \| Curves | `graph_paint.rs` (grafo por trilha) | o botão e a troca da área inteira | M |
| 6 | Arrastar chip → faixa (B4) | chips (`barra_clips.rs`), «+» da faixa | arrasto com fantasma do chip, alvo de largada na faixa | M |
| 7 | Arrastar a chave no canvas (C4) | caminho + hit-test (`ph2d-app-motion/src/motion_path_overlay.rs:444,479`) | pontos de chave clicáveis no canvas e o gesto que reescreve a chave | M |
| 8 | Quem ouve um sinal (B5) | `Marker.signal` gravado (v13), `signals_crossed` | registo de ouvintes (componentes, Luau, câmera) — investigar `ph2d-runtime` primeiro | M–G |
| 9 | Animating · Done (B7) | transporte não-desfazível (`intent.rs:14`) | confirmar ONDE a agulha vira passo de undo; capturar a pose à entrada e repô-la ao sair sem passo | G |
| 10 | Container aberto no lugar (C2) | entrar = `nest.rs`, `container_open` | sanfona das lanes de dentro no tempo da cena, instâncias irmãs esmaecidas | G |

Nenhum item toca contrato §6. Nenhum muda formato gravado por si (o 9 só se a pose for gravada — não será).

## Ondas

- **Onda 1 (P):** 1, 2, 3, 4. Smoke: `PH2D_NEST_SMOKE=4` (contorno, voltas) e uma cena com marcadores (trecho, losango).
- **Onda 2 (M):** 5, 6, 7.
- **Onda 3 (M–G):** 8 e 9 — cada um abre por uma investigação curta (onde mora o ouvinte; onde nasce o passo da agulha) que
  pode mudar o desenho; série só aí.
- **Onda 4 (G):** 10, sozinho.

Cada onda fecha com `/pd-onda-fechar` (smoke do dono, janela nova).

## Onda 1 — desenho

1. **Contorno:** clique no corpo de uma strip sem passar a folga de arrasto = escolher; clique no vazio da faixa ou `Esc` =
   nenhuma. A escolha vive no estado do painel (não grava, não desfaz). Strip escolhida = o anel já ligado (`TimelineTinta`
   da maquete B4). A strip que deixou de existir limpa a escolha.
2. **Trecho:** na faixa dos marcadores (maquete B6 «Markers»), clicar ENTRE dois marcadores vizinhos toca esse trecho em
   loop. A barra mostra «takeoff → top» com ✕; ✕, trocar de clipe ou trocar de aba devolve o loop do clipe. O trecho não
   se grava nem entra no desfazer, e não reescreve o `loop_range` do clipe.
3. **Losango:** `Livre` = losango vazio na tinta apagada (`Text3`), `Animado` = contorno na tinta da chave, `Chave` = cheio,
   `Mudado` = cheio laranja; campo sem canal continua o ponto inerte. Os dois Inspectors (cartões e clássico) leem o mesmo.
4. **Voltas:** numa strip em Loop/Ping-Pong, um risco fino na altura da peça a cada volta (no Ping-Pong, cada inversão).
   Derivado no `rebuild` (volta = comprimento da fonte ÷ velocidade). Riscos mais juntos que `TIMELINE_*` px somem (zoom).
