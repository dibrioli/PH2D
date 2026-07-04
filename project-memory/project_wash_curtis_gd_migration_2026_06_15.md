---
name: project_wash_curtis_gd_migration_2026_06_15
description: Por que o wash divergia (3 versões) e o plano canônico Curtis g/d que resolve — doc de parâmetros + ADR-0095
metadata: 
  node_type: memory
  type: project
  originSessionId: 4e51f187-9840-4a3b-9378-185be66e06bf
---

As 3 tentativas de wash/fluid divergiram e nenhuma era reprodutível porque **nenhuma implementou o
modelo canônico Curtis 1997** — todas fizeram reduções ad-hoc com constantes inventadas (`D_MAX`,
`V_MAX`, `COVER_K`, `FIELD_CAP`, `WATER_HALO`, `EDGE_EVAP_FLOOR`, `dry_drive`).

**Erro-raiz (auditoria multiagêntica 2026-06-15, 34 achados confirmados / 0 refutados):** falta a
separação pigmento **suspenso `g` ↔ depositado `d`** com **TransferPigment(ρ,ω,γ)** — o coração do
Curtis. Sem ela, diluição e "seca-escurece" são fisicamente impossíveis (viram hacks); staining (ω) e
granulação (γ) por pigmento não existem; edge-darkening é proxy ad-hoc.

**Pesquisa de terceiros (norte):** Curtis 1997 (modelo), Stam Stable Fluids (estabilidade), K–M/Mixbox
(cor — já correto, ADR-0091, NÃO mexer), MoXi (constantes real-time). Parâmetros publicados com faixas
em [`docs/Painter_projeto/wash_parametros_canonicos.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/Painter_projeto/wash_parametros_canonicos.md).

**Plano de migração:** [`docs/Painter_projeto/16_plano_migracao_curtis_gd.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/Painter_projeto/16_plano_migracao_curtis_gd.md)
+ **ADR-0095** (supersede a deposição implícita da 0094; mantém cor Mixbox + topologia GPU). Decisão
Enio 2026-06-15: **shallow-water COMPLETO** (velocidade+pressão+RelaxDivergence+backruns), 2 bugs vivos
(undo morto = crítico; switch-target descarta trabalho) consertados no C5. Ordem: C0 ADR → C1 g/d +
TransferPigment (remove `pigment_load`) → C2 shallow-water → C4 params/UI (5 sliders universais +
tabela 11 pigmentos §1.7) → C5 undo/lifecycle → C3 backruns.

**Lição de método:** adotar um modelo publicado inteiro (estado nomeado + constantes com faixa), não
inventar constantes por tentativa-e-erro — senão cada rebuild diverge. Relaciona [[project_watercolor_v2_gpu_first_refactor]],
[[project_wash_pigment_color_mixbox_residual]], [[project_wash_undo_event_driven_rebuild]].
