# HANDOFF — refatoração do Painter, janela 7 (01/11)

Ramo `line/PainterWatercolor`, de `d4d145c62b` a `ff250d8847` (+ este). Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela6.md](HANDOFF_01_11_janela6.md). **Smoke do dono: OK** (Paint type, óleo diluído a escorrer,
cruzamento amarelo × azul em verde).

## 1. Feito

| commit | o quê |
|---|---|
| `ce1b739ad8` | **J7** — o óleo pouco diluído pousa como PASTA: `meio::pasta(m, v, água)` (a fracção que o óleo prende, contínua no limiar, `0` na água) pousa como a ponta seca e o resto molhado leva a MISTURA na fracção `q` (`deposito_tinta::pousa_pasta`). Amarelo × azul (Velha, P600 W0,5) volta a verde; Filbert Dry ferrugem 3,48 → 0, degraus 21 → 9. `liquido_livre` = `1 − φ(1+mL)` (0 exacto no limiar; curou `meio::tests::o_oleo_do_tubo…`). [J7](J7_pasta_do_oleo.md) |
| `fdc42949c4` `92746b782b` `4d9b02c6dc` | J7 §5: a grelha E9 INTEIRA (`target/e9_j7/`, 3 240 células) e o smoke do dono |
| `ff250d8847` | miniaturas de fábrica regeradas UMA vez (31 com o Fan Brush, só a imagem; metadados iguais); `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` ✓; fotos em `fotos/j7/` |

## 2. Medido

- Grelha E9 inteira contra `e9_0111`: ferrugem 0 em 3 238/3 240 (máx. 0,01), manchas −0,23; a lei Pigmento idêntica.
  Piores lidos nas fotos (J7 §5.3): Filbert pr1 n3 — de manhã o amarelo tapava, agora mistura em verde; Knife Oily e
  Express Oil — faixa verde-clara do líquido a escorrer com a frente em degraus.
- `cada_ferramenta_da_agua_so_ve_o_que_le` (painel): ✓ em 182 s com a máquina calma (o `cargo-test-narrow` corta aos
  900 s sob carga — correr por `cargo test -p ph2d-panel-painter-layers --profile smoke --test it <nome>`).
- Vermelhos antigos com números IGUAIS depois da J7: `a_queda_cresce_com_a_diluicao` 5 020 · `a_agua_nao_sobe_a_ladeira_do_oleo` 54 477 · it `a_guache_opaca_cobre_o_azul_seco`.

## 3. Fila do dono (smoke 01/11 — «não precisa ver agora, manter o plano»)

1. **O escorrimento ignora o relevo da tinta já pousada** — passa por cima em vez de desviar (a mistura ao escorrer
   está boa). Família de `a_agua_nao_sobe_a_ladeira_do_oleo`.
2. **Desempenho muito aquém** com o óleo diluído (molhado ⇒ simulação longa; a grelha E9 ~4× mais lenta). Medir antes.

## 4. Aberto (por ordem)

1. Vermelhos antigos: os dois do §2; wp `o_water_do_smear_no_meio_e_a_lei_de_antes`; tp
   `cada_preset_da_agua_cai_na_foto_do_seu_rebelle` (redecidir a régua — o Rebelle deixou de ser referência, memória
   31/10); it `a_guache_opaca_cobre_o_azul_seco`; o falso vermelho `every_key_of_this_panel_exists_on_both_sides`;
   it `cada_preset_de_fabrica_e_a_sua_receita` / `os_candidatos_da_w2_sao_as_suas_receitas` (chave `levelingOwnStroke`).
2. Fila do dono §3 (1 e 2).
3. E9 §8 ⚑ furos (a régua lê o relevo nivelado como furo — de novo: Dabber 4 P40 W1 +25 com a foto MAIS limpa).
4. Chip rápido de Dilution; `set_wet_lei_da_tinta` sem uso (aviso dead_code); `assenta.rs` e `trail/transfer.rs` em
   700 linhas exactas (partir antes de mexer).
5. E10c; E11 GPU (a placa não lê o meio nem a pasta); E12 (a Velha sai; a lei Pigmento não passa pela pasta) · E13 ·
   outros 3 modos · ajuda ilustrada · sondas para a crate dev-only · crateras.
