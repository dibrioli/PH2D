# HANDOFF — refatoração do Painter, janela 6 (01/11)

Ramo `line/PainterWatercolor`, de `54c96ebc72` a `19ef5bdcac` (+ este). Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela5.md](HANDOFF_01_11_janela5.md). **Smoke do dono: NÃO feito** (o estado ainda não está pronto, §3).

## 1. Feito

| commit | o quê |
|---|---|
| `f8a006c60b` | **E10g** — o óleo diluído escorre numa lei só: o diluente é sempre líquido, o pigmento prende 1/9 no óleo (a tinta do tubo é pasta), o meio muda a viscosidade, a evaporação e o endurecimento; o meio da célula mistura-se pelo volume MOLHADO (cura do Bingham que prendia a aguada sobre o óleo seco, E10e2 §E3). [E10g](E10g_oleo_diluido.md) |
| `92a2d55e59` | **E10f** — o slider Medium vira o seletor «Paint type: Water · Oil» (dropdown, como o Environment do Real Oil); todo valor entra a 0/1; Acrylic → Water com Re-wet 0. [E10f](E10f_paint_type.md) |
| `5dcf477f73` | dois vermelhos antigos curados: o portão da receita de fábrica aceita a chave `levelingOwnStroke` do ficheiro |
| `a91a7e33b1` | **E10h** — pequenos do item 5: chave i18n órfã, `wet_opacidade` morto, `wetpaint_preset/tests.rs` partido, a parede do shader da placa (binding 18), Hardness visível na borracha d'água. [E10h](E10h_pequenos.md) |
| `f69f9cec2e` | E10g §4 — auditoria parcial pela grelha + interruptor `PH2D_SONDA_KNOBS` na sonda da grelha |
| `19ef5bdcac` | **E10i** — o assentar da tinta opaca (Velha, óleo sobre óleo) fica na caixa das duas tintas: ferrugem 52/50/25 → 0. [E10i](E10i_ferrugem_velha.md) |

## 2. Medido

- Água (`m = 0`) ao bit em todo hash corrido (lista em E10g §3 e E10i). Toda célula da grelha com Water 0: idêntica.
- Óleo d 0,9 escorre mais devagar que a água e fica molhado (Wet Drip: água 0,72 escorrida aos 15 s, óleo 0,31 e a
  correr); sem orla escura. O «pingar» em queda livre só existe no Wet Drip (os outros pincéis de água também não pingam).
- A/B no pai: `a_queda_cresce_com_a_diluicao` (4 455 no pai, 5 020 aqui) e `a_agua_nao_sobe_a_ladeira_do_oleo` (56 609 /
  54 477) **já eram vermelhos**; o `o_real_oil_na_mao…` só passava com a água presa (pai com Bingham desligado: 49 níveis).

## 3. Por que não houve smoke (abrir por aqui)

Foto `scratchpad/e10i_antes_depois.png` (sessão; refazer com `target/sonda_e10g/celula.sh`): na lei Velha (o produto),
óleo P600 × Water 0,5 (Dilution ≈ 0,127, 3 % de líquido livre):
1. **O amarelo sobre o azul TAPA (bege) em vez de dar verde** como de manhã (antes o óleo assentava num passe; agora a
   película fina assenta em muitos e o ramo `Cobre` do óleo sobre óleo vence). Decidir pela física (molhado sobre
   molhado mistura; o tapar é da camada que assenta sobre SECO) — medir antes de mudar.
2. Filbert Dry pr0,6 continua salpicado (ferrugem 3,48, degraus 16): outro caminho (re-molhar ou o pouso na película).
3. Depois: a grelha E9 inteira de novo com o binário final (`target/e9_e10g/` é de `f8a006c60b`, parada a 1 186 linhas;
   corre ~4× mais devagar porque o óleo diluído fica molhado) e comparar com `target/e9_0111/` (script em E10g §4).
4. `cada_ferramenta_da_agua_so_ve_o_que_le` (painel, ~600 s) não acabou sob carga 90 — correr com a máquina calma.

## 4. Aberto (por ordem, depois do §3)

1. Smoke do dono do Paint type + óleo diluído (passos no prompt).
2. Miniaturas de fábrica UMA vez (31 + o Fan Brush), fotos antes × depois.
3. Vermelhos antigos: os dois do §2; wp `o_water_do_smear_no_meio_e_a_lei_de_antes`; tp
   `cada_preset_da_agua_cai_na_foto_do_seu_rebelle` (redecidir a régua); `a_miniatura_de_fabrica…`;
   `familias::guache::a_guache_opaca_cobre_o_azul_seco` (já vermelho, E10i); o falso vermelho
   `every_key_of_this_panel_exists_on_both_sides` (chave lida fora da crate, E10h).
4. E9 §8 ⚑ furos (a régua lê o relevo nivelado como furo — confirmado de novo nas fotos da E10g §4).
5. Chip rápido de Dilution; `set_wet_lei_da_tinta` sem uso (E12); `assenta.rs` em 700 linhas exactas (partir antes de mexer).
6. E10c; E11 GPU (a placa ainda não lê o meio: escoamento, secagem, assentar); E12 (a Velha sai) · E13 · outros 3 modos ·
   ajuda ilustrada · sondas para a crate dev-only · crateras.
