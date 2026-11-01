# HANDOFF — refatoração do Painter, janela 2 (31/10)

Ramo `line/PainterWatercolor`, de `69a15a9b08` a `723e67e39b` (+ o commit deste handoff). Nada integrado nem empurrado.
Anterior: [HANDOFF_31_10_noite.md](HANDOFF_31_10_noite.md). Sem smoke do dono nesta janela: a lei nova vive atrás do
interruptor `Sim::tinta` (por omissão `Velha`), sem controlo no painel — o produto não mudou.

## 1. Respostas do dono (memória `project_wetpaint_refactor_and_illustrated_help_2026_10_31`)

| pergunta | resposta |
|---|---|
| A foto F0 é o defeito que viu? | **Sim** — a régua-mãe F0 está confirmada |
| Cartão de ajuda A/B/C | **C — animado** (pincelada grande low→medium→high + 3 miniaturas) |
| Opcional «Pigment mix» por pincel (azul+amarelo = verde, WGM) | **Sim**, desligado por omissão; a mistura padrão continua Oklab pelo volume |

## 2. Feito (com prova; o detalhe em cada doc de etapa)

| etapa | commit | doc | o essencial |
|---|---|---|---|
| E5 ecrã | `25068364fa`, `1279289911` | [E5_ecra.md](E5_ecra.md) | `wp/render/tinta.rs ecra_da_celula`; uma porta em `pigment_row`; orla e altura do Óleo Real por um ponto |
| E6a depósito | `8a5b179b48` | [E6a_deposito.md](E6a_deposito.md) | `M_UM = 1374` (calibrado pelo branco opaco); `garante` MIGRA a tinta velha; `mix`/`cobre`/`settle` fora da lei nova |
| E7 escoamento | `ed0549721e` | [E7_escoamento.md](E7_escoamento.md) | toda rotina do solver move a mesma fracção da `Soma`; dreno da borda (`project.rs`) |
| E6b pick-up/ferramentas | `1ec590d950`, `84e135ef24` | [E6b_pickup_e_ferramentas.md](E6b_pickup_e_ferramentas.md) | `Trail::ponta_tinta`; Smear/Blend/borracha/arrasto/secador/empurrar/orla por `tools/tinta.rs Foto` |
| E8 tempo | idem | [E8_tempo.md](E8_tempo.md) | assentar/re-molhar por `transfere`; `o(t)`, `τ(m) = 3600 s/m` (água nunca fecha); fecho só seco; `r_g0` guarda a cobertura do fundo curado (célula 76 B) |
| invariante | `84e135ef24` | — | `o_invariante_soma_massa_vale_no_motor` (filme.v·M_UM == susp, aberto.v·M_UM == sett, no caminho de produto) verde, pior 1,1e-5 |
| E9 preparo | `105f0bdf57`, `04aa8826ea` | [E9_portao_mae.md](E9_portao_mae.md) | `set_wet_lei_da_tinta`; eixo papel (padrão Textured Paper · liso); veredicto F0b em `resumo_f0.py`; render plano tira relevo/altura/óleo da CÉLULA; régua `mix_*` (segmento Oklab) para a Pigmento; sessão nova herda a lei; sair da Pigmento esvazia `Grid::tinta` |
| lei seca | `723e67e39b` | [F4_1_tinta.md](F4_1_tinta.md), E9 §6 | **a cor escolhida = masstone SECO no próprio meio**: `𝒦 = v·s_um·s_fator(m,seco)·q`, `𝒮 = v·s_um·s_fator(m,estado)`; aquarela molhada escurece e seca para a cor; oráculo `lei_unica.py` + fixture regenerados |

Decisões do coordenador que valem daqui em diante (estão no briefing comum, reproduzido em §5): **a soma acompanha a
massa** (`susp`/`sett` continuam em massa, a cor vive nas somas, invariante acima); **um `match` por porta**, corpo novo
em ficheiro novo; a lei Velha ao bit.

## 3. O portão-mãe ainda REPROVA (E9 §5–§6)

Célula Thick Impasto P600/W0,5/pr1/N3, papel padrão, render plano, lei Pigmento: manchas_azul **9,18 %** (≤ 1),
mix p99/max **51,1/157,2** (≤ 8/20); ferrugem 0, furos 0, degraus 2/0. Mecanismos medidos:
1. **A sonda lê a tinta MOLHADA** — com a lei seca, molhado escurece de propósito; seca, cada ponto fica ≤ 5,4 níveis
   do segmento (passaria). Cura proposta: o portão lê depois de secar (passe `film ← 0` só da régua, como o `render_plano`).
2. **`m = 0` em toda célula** — o Thick Impasto é óleo; com `m = 1` molhado == seco ao bit. O `m` vem da E10.
3. **`mix_max 157`** = franja saturada na orla — NINGUÉM investigou ainda.
4. **manchas_azul ~9 %** — hipótese molhado×seco entre vizinhas (degrau `film > 0`), não medida célula a célula.
⛔ NÃO corra a grelha inteira (~3 h) antes de 1–2 e da célula passar perto dos limiares.

## 4. Aberto (por ordem)

1. **E9 fecho**: régua lê seco + `m` do pincel (mínimo da E10: o depósito escreve `m` médio pelo volume a partir do meio
   do pincel — óleo 1, aquarela 0) → franja da orla → manchas → célula verde → grelha inteira por `fila-pesada.sh`.
2. **E10** painel por áreas (PAINT · BRUSH · PAPER · TIME · LIGHT, inglês), 177 → N com tabela de migração, presets
   por função; F4_2 §7.2.
3. **«Pigment mix»** (aprovado): `LeiDeMistura::Pigmento` WGM 10 bandas (`crates/ph2d-painter-brush/src/espectral.rs`),
   outra `Soma` (`Σ v·ln R`), controlo por pincel, régua própria; F2b §4.2.
4. E11 GPU com paridade; E12 a velha sai num bloco; E13 portões do Rebelle.
5. Os outros 3 modos na mesma lei; F5 ajuda ilustrada **variante C**; sondas → crate dev-only; crateras da costura.
6. Abertos menores: `Foto` dá à massa ganha a cor MÉDIA da caixa (não a da origem); transparência da tinta solta da
   ponta perde-se; relógio da cura só corre com água; depósito seco não renova `o`; custo do Jacobi novo (`Vec` por
   linha) e do `gather_linear` em série por medir; emboss/papel em `render.rs:542-555` lê a massa (some no fecho);
   `ponta_tinta` reinicia ao mudar a janela; `BlendReach > 0` usa a cor média da janela.
7. Vermelhos ANTERIORES à refatoração: `advect_jacobi_bits_are_the_recorded_ones`, `_by_each_commit_route` (já vermelhos
   em `69a15a9b08`) + os 4 do handoff anterior §3.7.

## 5. Briefing comum dos agentes

O texto que cada agente recebeu está em [`ferramentas/briefing_comum_agentes.md`](ferramentas/briefing_comum_agentes.md)
— reutilize-o (regra «outros 2 agentes» só quando houver paralelo).
