---
name: feedback-painter-inefficiency-4-causes
description: "Por que a semana do painter foi \"compila mas nada funciona\" — 4 causas estruturais + a diretiva por-etapa que é o antídoto"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4e51f187-9840-4a3b-9378-185be66e06bf
---

Investigação forense 2026-06-16 (7 agentes, evidência file:line) achou por que a implementação do Painter foi "constantemente imprecisa". NÃO é o modelo (Opus 4.8) nem descuido pontual — são 4 defeitos estruturais:

1. **Ninguém prova a costura.** Caminho clique→ativa→rota→muta→pixel atravessa 8+ crates+shell; nenhum teste percorre inteiro. Sliders Roundness/AlphaThreshold = fios mortos (dispatch na tool, ausentes de event/sections/populate); SelectBrush = no-op vazio. É [[feedback_tool_unit_green_integration_dead]] em escala.
2. **"Audit" sem definição = "rodar gates".** Gates `architecture_*_contract_surface` são contadores de texto (count_struct_fields, .matches("pub struct")), provam ABI não comportamento. Audit rigoroso cita 19-23 file:line; preguiçoso cita 0-3 + DEFER/"Enio smokou".
3. **Isolamento estrito fabrica defeitos** em features transversais: dois LayerId/LayerStack (u64 vs u32) ainda no tree; flags órfãs sem consumidor. (Drop-crate funciona p/ tools one-shot e engines puras; NÃO p/ workhorse transversal.)
4. **Alvo irrefutável** ("vá onde ninguém foi"→aquarela 18 ADRs/165 commits/deletada; agora "paridade Procreate"). Sem kill-criterion antes do build. + inversão de eficiência: canvas CPU-residente com re-upload/frame enquanto StampPipeline GPU validado (828 LOC, 9 gates) é dead code congelado.

**Why:** verde-de-compilação foi treinado como sinal de sucesso (§0.5 "inner loop = SÓ cargo check"); testes/audit/DoD herdaram isso. O Enio perdeu uma semana e a confiança por causa disso, não por bug isolado.

**How to apply:** leia [`docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) a CADA passo. Separe velocidade (cargo check) de evidência (audit = ler código + trace file:line + asserção falsificável + render→imagem→olhar). Veja [[feedback_audit_lens_diversity]], [[feedback_measure_perf_symptom_scale]], [[feedback_perfection_no_deferrals]].
