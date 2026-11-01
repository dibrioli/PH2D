# HANDOFF — refatoração do Painter, janela 5 (01/11)

Ramo `line/PainterWatercolor`, de `bf90eb2598` a `4c5e4764cb` (+ este). Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela4.md](HANDOFF_01_11_janela4.md). Smoke do dono feito em 01/11 depois do painel: §2.

## 1. Feito

| commit | o quê |
|---|---|
| `e8f35b5710` | **E10 no painel de hoje**: Dilution, Volume (µm), Medium, Body, Hardness e Pigment mix nas secções existentes, lidos nas DUAS leis. O produto ainda pinta `Velha` ([E10a1](E10a1_receita.md) · [E10a2](E10a2_meio_e_corpo.md) · [E10b1](E10b1_ferramenta_e_presets.md) · [E10b2](E10b2_painel.md) · [E10c2](E10c2_pontas.md); tabela controlo→secção→id em [F3 §9](F3_controlos.md)) |
| `4e725aff41` | **Lote D**: camada fina presa ao matiz (49,2°→0,6° no azul · 15,1°→0,9° no amarelo, [E10d1](E10d1_camada_fina.md)); Absorbency por tela, `r^A`, faixa 0..64 ([E10d2](E10d2_absorbency.md)); dois vermelhos antigos curados — os hashes de `smooth_edges` = E4 DE PROPÓSITO; Re-wet × Solubility × Drag agem em PAR no censo ([E10d3](E10d3_vermelhos_antigos.md)) |
| `04944ad5a5` | **Lote E**: o Real Oil vive na célula (`pincel_de_oleo`, 28 B, [E10e1](E10e1_oleo_na_celula.md)); a mistura K-M fica no invólucro das entradas (o preto não clareia); o `seca` da GPU lê `retencao` ([E10e2](E10e2_km_e_remolha.md)) |
| `4c5e4764cb` | **E9 §8**: a grelha inteira com a E10 — Pigmento 437/810 padrão · 398/810 liso, Velha 0; reprovam só furos e degraus, e os MESMOS furos na Velha ([E9 §8](E9_portao_mae.md)) |

## 2. O smoke do dono (01/11) e a decisão que ele gerou

«Gostei bastante de tudo.» Pergunta: óleo muito diluído nunca escorre. **Causa medida:** a água da receita é
`(1−φ)(1−m)h/K_W`; com `m = 1` o solvente do óleo some. Decisão do dono depois de avaliação séria (memória
`project_paint_type_water_oil_one_law_2026_11_01.md`, em `project-memory/` do repo principal): **UMA lei, sem segundo
fluido**. O solvente é sempre líquido; `m` só muda a viscosidade (mistura em log), a secagem `E·(1−m)`, a cedência e o
endurecimento no tempo. O slider Medium vira SELETOR «Paint type: Water · Oil» (só dois pontos testados; acrílico = Water
com Re-wet 0; o Fan Brush 0,5 migra para um dos dois). **Recusados:** dois fluidos imiscíveis; «meio só de aparência».

## 3. Lições medidas nesta janela

1. Vigia com `pgrep -f <texto>` casa a PRÓPRIA linha de comando e nunca dispara — vigie a SAÍDA a crescer. A grelha
   das 09:15 morreu às 09:18 sem ninguém ver.
2. `/tmp` é tmpfs em RAM de 62 GB e encheu a 100 % com alvos de bisecção: build, worktree e grelha vão em `target/` do
   worktree, em `/home`; um worktree de bisecção de cada vez.
3. Todo «vermelho depois da minha mudança» foi A/B no commit pai com worktree de árvore inteira: 5 de 6 já estavam
   vermelhos (réguas velhas do Rebelle ou regressões antigas). Nunca regravar nem afrouxar sem esse A/B.
4. A grelha E9 inteira leva 45 min (não 3 h) rodada direto por `ph2d-run.sh` com a saída em `/home`.

## 4. Aberto (por ordem)

1. **UM BLOCO:** solvente do óleo líquido + seletor Paint type (Water·Oil: migração do Medium, testes, i18n) + a
   regressão de Bingham que prende a aguada sobre o óleo (`remolha_tests::a_aguada_re_molha_o_oleo_soluvel`,
   [E10e2 §E3](E10e2_km_e_remolha.md): degrau 2 = `c5e6daa0a9`, o pigmento re-molhado entra em `massa_movel`; degrau 1 = cena
   velha desde o K-M por omissão `3eb13d38db`). Depois auditar CADA efeito de água a `m = 1` (escurecimento de borda,
   backrun, granulação, capilar, bloom) com a grelha ANTES de mostrar ao dono; endurecimento no tempo para a tela de
   óleo não rodar o solver para sempre.
2. Regerar as miniaturas de fábrica UMA vez (só a janela principal; 31 mudam, 10 pelo invólucro K-M), fotos antes × depois.
3. Vermelhos antigos que restam: wp `o_water_do_smear_no_meio_e_a_lei_de_antes` (`0x76d1277d6f22d420`); tp
   `cada_preset_da_agua_cai_na_foto_do_seu_rebelle` (Smear Soft/Spikey contra foto do Rebelle — o Rebelle deixou de ser
   referência: a régua se REDECIDE); tp `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`.
4. E9 §8 ⚑ furos: régua × física do toque seco leve; células «sem medida».
5. Pequenos: chip rápido de Dilution; Hardness da borracha d'água (lê o portão do papel); restos da parede do shader GPU
   (binding 18, `Knobs::isola`); `BrushSettingsSnapshot::wet_opacidade` e a chave i18n órfã
   `panel.wet_tuning.knob.levelingOwnStroke`; `wetpaint_settings.rs` com exatamente 700 linhas;
   `wetpaint_preset/tests.rs` > 800; aviso `set_wet_lei_da_tinta` sem uso (a E12 o chama).
6. E10c: a ponta não leva o Espectro; a linha Pigment mix só aparece com a lei Pigmento.
7. Depois: E11 GPU · E12 (a lei velha sai; o produto vira Pigmento) · E13 · os outros 3 modos · ajuda ilustrada (F5, C) ·
   sondas para a crate dev-only · crateras da costura.
