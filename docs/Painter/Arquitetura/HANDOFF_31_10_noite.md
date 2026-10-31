# HANDOFF — refatoração do Painter, janela 1 (31/10, noite)

Ramo `line/PainterWatercolor`, de `935f650ff8` a `541d84c785`. Régua «antes»: etiqueta
`painter-pre-refatoracao-com-problemas-2026-10-31`. Nada integrado nem empurrado. Arquitetura: [ADR-0197](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md).

## 1. Decisões do dono nesta janela (todas na memória)

| decisão | onde |
|---|---|
| O Rebelle DEIXA de ser referência — régua = física + varredura + olho do dono | `project_rebelle_no_longer_the_reference_2026_10_31` |
| «Não meça esforços»: TODO o Painter nos 4 modos, cada parâmetro vivo, estável, reprodutível | `project_wetpaint_refactor_and_illustrated_help_2026_10_31` |
| Um controlo, uma grandeza: Pigment + Water = só a diluição; o Volume = só a quantidade | ADR §1.1 · `project_painter_volume_is_independent_of_dilution_2026_10_31` |
| Mistura de cor = a mais ROBUSTA (o que os apps de sucesso fazem), não a mais realista ⇒ Oklab pesado pelo volume | ADR «Em aberto» ✅ · [F2b](F2b_mistura_de_cor_nos_apps.md) |
| O app é SÓ em inglês; ajuda em Plain English | `feedback_app_ui_english_only` |
| Cartões de ajuda e réguas sobre o papel texturizado PADRÃO | F5 doc · F0 §4 |
| PRIORIDADE Nº 1: óleo grosso não pode escorrer (ondas, crateras) | [RELEVO](RELEVO_oleo_ondas_e_crateras.md) |

## 2. Feito (com prova)

| item | commit | prova |
|---|---|---|
| Docs F0/F1/F2/F2b/F3(ADR)/F5-maquete nesta pasta | vários | — |
| Varredura do óleo (régua-mãe) reproduz o dono_6: ferrugem (Filbert 5,5 % no plano), azul amontoado (Knife Oily 21,9 %) | `069166a4aa`, `ae55ed55b1` | [F0](F0_varredura.md) |
| Crate `ph2d-paint-tinta` — lei única, mistura Oklab pesada pelo volume, camada Kubelka, conservação; paridade com o oráculo `ferramentas/lei_unica.py` | `0eaf6fa1ed` | 12 testes · [F4_1](F4_1_tinta.md) |
| E4 receita × quantidade + migração dos presets (marca `receitaE4`) | `7a4bc540b7`, `b7f54356d6` | 3 testes da regra vermelho→verde; presets e Blender iguais ao bit |
| Bingham no escoamento do óleo (PRODUTO) | `c5e6daa0a9` | `o_oleo_grosso_com_pouca_agua_nao_escorre_fora_do_pincel`; Thick Impasto W 0,2/0,5 crateras fora → 0 |
| Piso do espalhar do 1.º toque (o óleo grosso já não cava) | `c5e6daa0a9`, `43dd8acc3d` | `o_oleo_grosso_sobre_o_oleo_grosso_soma_massa_e_nao_cava` |
| E2/E3: `Sim::tinta: LeiDaTinta {Velha, Pigmento}` + `CelulaTinta` (inertes na lei velha; a placa recusa Pigmento) | `c5e6daa0a9` | 4 testes; píxeis iguais ao bit |
| BUG papel texturizado só no Draw | `2c79f82966` | teste vermelho→verde; **smoke do dono OK** |
| `passo.rs` 832 → 636, `render.rs` 754 → 696 (portão de LOC verde) | `43dd8acc3d`, `b7f54356d6` | `architecture_workspace_file_loc_cap` |
| 14 miniaturas de óleo regeradas por Bingham (Thick Impasto e Metallic Flat aprovados pelo dono) | `a85b435647`, `541d84c785` | portão das miniaturas verde |

**Smoke do dono (31/10):** papel OK · óleo grosso «bem melhor» · Pigment/Water OK · Thick Impasto/Metallic Flat OK.

## 3. Aberto (por ordem)

1. **Crateras na COSTURA entre pinceladas** (foto [`fotos/smoke_31_10_dono_crateras_entre_pinceladas.png`](fotos/smoke_31_10_dono_crateras_entre_pinceladas.png), mouse, pressão máx.) — o dono mandou deixar para DEPOIS da refatoração completa. Hipóteses: a orla do traço novo sobre o relevo do anterior (piso só no 1.º toque), o relevo das cerdas (`bristles_relevo.rs`), Filbert/Flat Bristle com crateras dentro residuais (RELEVO §Mecanismo 2).
2. **E5–E13 do plano** [F4_2](F4_2_plano_wet_paint.md): ecrã pela lei nova (E5), depósito/pick-up/escoamento por somas (E6–E7), assentar sem Newton (E8), portão-mãe nas DUAS leis (E9), controlos → grandezas + painel por áreas Tinta·Pincel·Superfície·Tempo·Luz e os 177 controlos → N (E10), GPU com paridade (E11), a lei velha sai NUM bloco (E12), portões do Rebelle (E13).
3. **Os outros 3 modos** (Digital, Watercolor, Impasto) para a mesma arquitetura: 5 leis de mistura → 1 (+ Porter-Duff nomeado no Digital), ≥ 6 relevos → 1, 4 secagens/papéis/coberturas → 1 ([F1 mapa](F1_mapa_do_modulo_e_os_4_modos.md) §3, §5). O Impasto tem ZERO pincéis de fábrica.
4. **F5 ajuda ilustrada**: maquete em inglês feita; escolha da variante PENDENTE (recomendada B). Imagens geradas pelo motor sobre o papel padrão.
5. ~39 k linhas de sondas em `src/` → crate dev-only `ph2d-paint-sondas`.
6. Bingham: Filbert Dry W 0,2 resíduo (88 crateras fora); 7 células W 0,5 com ondas 1,16–1,56 dentro do contacto.
7. Vermelhos ANTERIORES (não desta janela): `a_agua_nao_sobe_a_ladeira_do_oleo`, `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada`, `o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos`, `a_aguada_re_molha_o_oleo_soluvel`.
8. Calibração de S (transparência) da `ph2d-paint-tinta` provisória; desempenho na Radeon por medir; RSS +258 MiB a 2048² com a lei Pigmento.

## 4. Perguntas ao dono ainda SEM resposta

1. A foto do defeito reproduzido [`fotos/f0_varredura/F0_para_o_dono.png`](fotos/f0_varredura/F0_para_o_dono.png) é o que ele viu?
2. Cartão de ajuda: A, B (recomendada) ou C ([`fotos/f5_maquete/comparacao.png`](fotos/f5_maquete/comparacao.png))?
3. Quer o opcional por pincel «Pigment mix» (azul + amarelo = verde; WGM 10 bandas)?
