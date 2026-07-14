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

**Terceiro caso (Motion, 2026-07-13) — e o erro fui EU, na direção contrária.** Achei um intent
(`SetBackdropTitle`) **com handler e sem emissor**, e concluí, no commit e num ADR: *"a feature nunca
foi construída"*. **Falso.** Ela existia por **OUTRO caminho** — o painel de params tinha as linhas
Title e Color do backdrop o tempo todo (`backdrops::params_snapshot` + `apply_param_intent`, num
canal de intent diferente). O intent que eu achei era uma **duplicata morta**, não a ausência da
capacidade. Descobri **rodando o seam** e imprimindo as rows — depois de já ter afirmado o contrário
duas vezes por escrito.

**Why:** "código morto" e "feature ausente" **não são a mesma coisa**, e o grep de um símbolo não
distingue as duas. Um caminho morto pode ser o *segundo* caminho para algo que funciona — e declarar
a feature ausente a partir dele é exatamente o erro que este arquivo inteiro descreve, cometido do
outro lado.

**How to apply:**
- **Cace a CAPACIDADE, não o símbolo.** A pergunta não é *"quem emite `SetBackdropColor`?"* — é
  **"o usuário consegue trocar a cor?"**. São perguntas diferentes e só a segunda é a que importa.
- **Responda-a EXECUTANDO** (o seam, o snapshot do painel, o app), nunca por grep
  ([[feedback_no_industrial_claims_without_verification]]). Um `println!` das rows do painel matou uma
  afirmação que dois greps tinham "confirmado".
- **Antes de dizer "isto não existe", procure o segundo caminho**: outro canal de intent, outro
  painel, outro atalho. Um sistema com N caminhos para a mesma ação tem N-1 candidatos a parecerem
  mortos.
- O corolário do começo **continua valendo** (encanamento sem consumidor mente) — só que a conclusão
  certa dele é *"há um caminho a MENOS do que os símbolos sugerem"*, e **não** *"a capacidade não
  existe"*.

