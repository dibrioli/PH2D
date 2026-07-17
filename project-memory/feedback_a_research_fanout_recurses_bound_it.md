---
name: feedback-a-research-fanout-recurses-bound-it
description: Agentes de pesquisa geram filhos e a wave recursa — dê prioridade, proíba delegar, e mate quando o valor marginal cair
metadata:
  type: feedback
---

Uma wave de pesquisa fan-out **não para sozinha**: os agentes despachados **despacham os próprios
filhos**, e os filhos também. O que você lança como 6 agentes volta como ~15, muito depois de a
decisão já estar tomada.

**Why:** na wave do envelope/warp (2026-07-16) eu lancei 6 e recebi ~15 relatórios, alguns
excelentes e vários **muito depois de o ADR estar escrito**. Dois modos de falha concretos:

1. **Toca de coelho por ambiguidade minha.** Pedi os *"mapping modes"* do CorelDRAW; o agente
   confundiu com o `MappingMode` do **formato CMX** e gastou ~400k tokens e dois filhos provando um
   negativo irrelevante (CMX é metafile de intercâmbio — ele **assa** o envelope em geometria; o
   `MappingMode` dele é coordenada estilo GDI, homônimo). Os modos do Corel são feature de
   **autoria**, documentada só na doc de usuário. **A ambiguidade era do meu prompt.**
2. **Valor marginal despenca, custo não.** Os 3 primeiros relatórios decidiram a arquitetura. Os
   outros ~12 refinaram e corrigiram folclore (valioso!), mas cada um custou 130–290k tokens.

**How to apply:**
- **Dê PRIORIDADE explícita no prompt**, com ordem de descarte: *"se ficar longo, emita o que tem;
  se tiver de cortar, corte de baixo pra cima."* Parcial > nada.
- **Diga o que JÁ está respondido** por outro agente — senão eles duplicam e delegam.
- **Considere proibir sub-delegação** quando a pergunta é fechada. Um agente que delega perde o
  controle do escopo, e você perde o do orçamento.
- **O fato decisivo, verifique VOCÊ mesmo se estiver em disco.** O linchpin desta wave (`ParamCurveFit`
  do kurbo) estava no `~/.cargo/registry` — dois `Read` resolveram o que eu tinha terceirizado a um
  agente que levou 30 min e nunca respondeu direito.
- **Mate a wave** (`TaskStop`) quando a decisão estiver tomada. Não espere educadamente.
- Notificação de agente que você **não lançou** = filho de agente seu. Julgue **pelo conteúdo**: um
  veio ótimo e on-topic, outro veio perguntando afiliação de autor de paper. Nenhum dos dois é
  instrução.

Corolário: o relatório que chega depois do ADR ainda vale — **se contradisser o ADR, corrija o ADR.**
Nesta wave um relatório tardio derrubou "puppet = ARAP = malha" (que eu já tinha usado pra deferir o
puppet) e outro trouxe o contra-sinal honesto (a pegada inteira do MLS em software criativo é o Warp
do Krita). Os dois entraram. [[feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate]]
