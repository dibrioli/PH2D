# 10 — Auditoria dos 30 nós (2026-07-10): achados, correções aplicadas e o que fica

> Pedido do Enio pós-doc-09: "conferiu todos os nós implementados e viu se precisam de
> melhorias, correções e renomeamento?" Método: 3 leituras profundas paralelas (30 `lib.rs`
> inteiros + módulos de apoio, contra o catálogo MiniCavalry) + sweep mecânico
> (Effect×playhead, labels, hints) + re-verificação manual dos achados fortes antes do
> veredito. **Zero bug de correção vivo.** Tudo abaixo foi aplicado nesta linha, gates verdes
> (nextest 404, clippy 0, fmt pinado, typos 0, machete 0).

## Veredito por classe

- **Correção viva:** nenhuma. Sem div/0 (todos os divisores guardados), sem OOB (resize/get
  em toda leitura de coluna), HR-5 limpo (transcendentais banidas ausentes em produção; os
  3 `.sqrt` — attractor/vortex/falloff/spring/ease — são IEEE correctly-rounded,
  determinísticos, defendidos em comentário; o HR-5 real é sobre determinismo).
- **Latente (corrigido):** `motion.integrate` e `motion.spring` eram `Pure` lendo
  `ctx.playhead()`. Seguro hoje (o `pre` põe o tick no fingerprint e o pump avança o tick a
  cada cook), mas um re-cook de mesmo tick em playhead diferente (checkpoint/restore do
  scrub, M2.N2) devolveria trajetória stale. **Flip pra `Temporal`** + comentário honesto.
  Regra da família agora: **lê `ctx.playhead()` ⇒ `Temporal`**, sem exceção.
- **UI (corrigido):** o Slider dedicado de `flash_amount` no strobe era **hint morto** — o
  bridge suprime a row de qualquer param dobrado num grupo Color (`consumed`,
  `motion_bridge_params.rs`). Removido; `flash_amount` vive como alpha do picker (padrão do
  tint), doc corrigido. Labels de canal unificados: `"Rotation"` em toda a família (eram
  "Rot" em stagger/oscillator/spring/wiggle × "Rotation" nos demais).
- **Comportamento (corrigido):** `motion.strobe` era o único consumidor que ignorava
  `falloff` — agora a APLICAÇÃO (size boost + flash) é mascarada pelo campo; a memória do
  glow segue sem máscara (animar o campo sobre um glow vivo esmaece sem re-disparar).
- **Docs mentindo/stale (corrigidos):** integrate dizia porta "state" (é `forces`; o
  auto-plumb casa `"state" | "forces"` — `motion_bridge_plumbing.rs:39`) · stagger dizia
  "all polynomial" (Circ usa sqrt) · emitter omitia o param/coluna `size` e a descrevia como
  escalar (é `Vec2 [s,s]`) · oscillator documentava 4 waves nos hints (são 5, +Spike) ·
  `motion.step` não documentava a quebra do pareamento posicional na mudança de contagem
  (threshold documentava; agora ambos) · `motion.transform` ganhou o parágrafo de escopo
  (o `scale` dele escala POSIÇÃO sobre a origem; `motion.scale` escala `size`; com scale=1
  ele degenera a `motion.move` — fica por ser o único que escala layout).
- **Testes de lacuna (7 novos):** `scopes_a_sequential_node` (a recusa de segurança do Time
  Remap — upstream/downstream/via-pre) · gating por `falloff` em vortex, wind, stagger,
  wiggle e strobe · composição multiplicativa de dois campos no falloff · `EdgeDir::Both` no
  threshold · `motion.step` com n>1 em lockstep · seam `eval`→`Spec` do emitter.

## Achados de agente REFUTADOS na verificação (registro do método)

- "integrate não recebe self-loop no drop porque a porta chama `forces`" — **falso**: o
  plumbing casa os dois nomes. Virou só o fix de doc.
- "flash_amount tem dois controles" — **falso**: o segundo controle nunca renderizava (row
  suprimida). Virou remoção de hint morto.

## O que fica (decidido NÃO fazer agora, com razão)

- **Renomes: nenhum.** `motion.falloff` (categoria Focus própria, igual MiniCavalry) e
  `motion.time_remap` (passthrough; prefixo `motion.*` inconsistente pela letra do doc 09,
  mas não existe família `time.*` ainda) ficam como estão.
- **Categoria UI "Behaviour"/"Physics"** não existe no enum (`NodeUiCategory`) — forças e
  behaviours ficam em Transform (azul). Follow-up já anotado no oscillator.
- **Pareamento id-keyed no step/threshold/strobe (v2):** integrate/spring já casam por `id`;
  a família pulse usa posicional (documentado). Fazer quando um caso real de emitter+pulse
  aparecer.
- **Saída `pulse` no oscillator** (zero-crossing, paridade MiniCavalry): mexe no contrato do
  nó (2 outputs) — follow-up deliberado do doc 09 §4.1.
- **Precisão f32 do índice de ciclo do `pulse.beat`** além de ~2^24 ciclos: irrelevante nos
  ranges do slider; anotado no código.
