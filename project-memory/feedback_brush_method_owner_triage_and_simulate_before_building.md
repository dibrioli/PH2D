---
name: feedback_brush_method_owner_triage_and_simulate_before_building
description: "Dono 28/10 (J6 Pincéis 2): um dia, ~25 h de agentes, 2 pincéis aprovados — a metodologia nova: o dono faz a triagem, simula-se o conserto antes de construir, prazo 45 min, pesado 1× no fim, rendimento por hora"
metadata:
  type: feedback
---

A JANELA 6 (28/10) seguiu «uma lacuna de motor por agente, padrão-ouro, gate + mutação + exame» e gastou ~20 agentes
(~6,5 M tokens, > 25 h) para **2 pincéis aprovados** (Soft Pastel, Wax Crayon). O dono: «o tempo gasto para tão pouco
resultado não justifica … precisamos de metodologia correta».

**Why:** (1) a J5 já tinha medido que depois da 2.ª ronda o rendimento é zero e o que trava é motor — e ninguém
conferiu ANTES se a peça de motor viraria um aprovado; (2) 4 de 10 diagnósticos antigos estavam errados (lâmina não
gira, secador, fio do arrasto, borda molhada) e só se descobriu depois de horas; (3) a cega reprova qualquer
diferença média ⇒ ~20 «quase»; (4) máquina a load 15–54 (9 sessões); (5) trabalho pesado repetido sem precisar: a
regravação dos 8 Oil/* correu 2h45 a 1 085 % de CPU porque a tabela de largura pinta uma cena com o Real Oil por raio
e a luz do óleo (72 amostras/célula) recalcula a cada passo — a largura nem precisa da luz.

**How to apply** (missões de pincéis/presets, e todo trabalho «aproximar X de um oráculo»):
1. **Triagem pelo DONO primeiro, barata**: folha nosso | oráculo com fotos que já existem; ele marca serve / quase /
   não serve. «Serve» entra; só «quase» recebe trabalho. O dono decide a lei ([[feedback_the_owners_reference_product_decides_the_law]]).
2. **Simular o conserto antes de construir**: um retoque da imagem que imita a peça de motor (~15 min); só se o
   retoque passar a cega/o dono é que se constrói.
3. **Prazo de 45 min por agente**; estourou ⇒ pára, diz onde chegou, o coordenador decide.
4. **Pesado 1× no fim** (regerar presets, tabela de largura, suíte); medir largura com o óleo DESLIGADO; luz do óleo só na foto final.
5. **Rendimento por hora** (aprovados/h) a cada rodada; abaixo do combinado ⇒ parar e reavaliar com o dono.
Texto completo: `docs/Painter/Pinceis/REGRAS_DE_CUSTO.md` §Metodologia. Ver [[feedback_brush_mission_cost_budget]].
