---
name: feedback-stale-comment-and-dead-code-lie
description: Comentário desatualizado e código sem consumidor não são neutros — mentem com autoridade e enganam o próximo leitor (inclusive auditorias)
metadata:
  type: feedback
---

Na varredura do Painter (2026-07-12) **duas armadilhas de documentação** custaram rounds e quase
produziram entregas erradas:

1. **Comentário desatualizado.** `stamp_route.rs` afirmava que os shape-editors (Line/Curve/Ellipse/
   Polygon/Free Hand) *"fall through to the plain deposit"* com Watercolor ligado. Era verdade quando
   foi escrito, e **falso desde o doc 13 #3** (eles passaram a rodar a ótica por
   `stamp_drag_preview_watercolor`). Eu li o comentário, acreditei, e **reportei ao Enio uma condição de
   UI errada** ("esconder Accumulate só nos métodos cumulativos"). O Enio corrigiu de cabeça: *"Line/Curve/
   … já atuam em watercolor normalmente"*. Só o `grep` no código resolveu.

2. **Código sem consumidor.** Os botões Paper Rake/Random já tinham sido removidos do painel, mas ids +
   setters + arms de rota + campos ficaram *"for the API"*. Uma auditoria inteira leu os setters
   sobreviventes e concluiu **"há dois botões mortos no painel"** — não havia botão nenhum.

**Why:** o próximo leitor (humano ou LLM) trata comentário e símbolo existente como **evidência**. Um
comentário obsoleto tem a mesma autoridade de um correto, e um setter sem chamador parece uma feature.
Ambos são pior que a ausência: a ausência faz você ir ler o código; a mentira faz você parar de ler.

**How to apply:**
- **Se removeu a UI, remova o encanamento** (ids/setters/rotas/campos). "Deixar para a API" só cria
  armadilha quando não há consumidor externo real.
- **Ao mudar o fluxo, grepe os comentários que descrevem o fluxo antigo** — eles não quebram o build.
  Um comentário que afirma uma PREMISSA (`"X não pode mudar aqui"`) é dívida com vencimento: vire
  `debug_assert`/gate, ou releia-o quando a premissa mudar (foi assim que o `wet_substrate` apodreceu,
  [[feedback_measure_perf_symptom_scale]] · BUGS #13).
- **Nunca aja só com base num comentário** quando a decisão é visível pro usuário: confirme com grep/teste.
- Corolário para auditorias: **"existe um setter" ≠ "existe um botão"**. Rastreie até o site de PINTURA
  ([[feedback_context_menu_closes_on_down_repaint]] tem o gêmeo: grep o id no `populate_*` PRIMEIRO).
- **"Está morto" e "está vivo" são AMBOS claims que exigem teste**: esconder um knob exige provar
  byte-identidade; manter um exige provar que ele muda a saída.
