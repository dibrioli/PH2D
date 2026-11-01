# HANDOFF — refatoração do Painter, janela 8 (01/11)

Ramo `line/PainterWatercolor`, de `12e509975a` a este commit. Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela7.md](HANDOFF_01_11_janela7.md). **Smoke do dono: não houve** (nada visível mudou no produto —
só curas de réguas e medições; a janela trocou pelo tamanho da conversa).

## 1. Feito — [J8_vermelhos_antigos.md](J8_vermelhos_antigos.md)

| commit | o quê |
|---|---|
| `f20ed68601` | hook `prazo-do-agente` com caminho absoluto + censo de chaves que não cega com `.git` sem `HEAD` (2 falsos vermelhos) · Water do Smear provado pela conta · régua da água sem a foto do Rebelle (`cada_preset_da_agua_tem_a_sua_cara`) · `set_wet_lei_da_tinta` só nos testes |
| `4fb0c114b1` | guache opaca: tecto 13 → 21 pelo Body da E10 (A/B medido) |
| este | docs J8, régua da queda reescrita (vermelha pelo defeito real), sondas `escorre_contorna_tests`, `desempenho_oleo_tests`, `diag_o_que_trava_a_queda`; o interruptor `DIQUE` saiu (recusado por medição) |

## 2. O achado que manda na próxima janela

**O meio (`OleoCelula::meio`) é propriedade do LÍQUIDO mas vive na CÉLULA e a advecção não o transporta.** A água que
escorre para uma célula de óleo seco herda `m = 1`: mobilidade ÷8 e evaporação ×0,1. Medido: a queda do Wet Flat na
crista 4 862 → 55 411 só com `PAPEL_ETA = 1`. É a mesma família de:
- tp `a_queda_cresce_com_a_diluicao` (vermelho, régua J8 certa: 2 433 > 10 % de 4 862);
- a fila do dono §3.1 (o escorrimento e o relevo) e §3.2 (o óleo diluído fica acordado: 14 000 células vivas aos 120 s);
- a E11 (a GPU não lê o meio).

## 3. Aberto (por ordem)

1. **Transportar o meio com o líquido** (um bloco, CLAUDE.md §0.10): a advecção (`advect_jacobi`/`advect_fundida`, e
   a difusão/backrun que movem `film`) leva `Σ film·m` como leva a massa; a célula seca (sem `film`) não impõe o meio
   à água que chega; `oleo_celula::pousa` continua a misturar no pouso. Água ao bit (`m = 0` em todo o lado ⇒ sem
   aritmética nova) — provar com `flow_bit`, `advect_jacobi_`, `oleo_sobre_oleo`, `assenta_bit`. Gates: tp
   `a_queda_cresce_com_a_diluicao` verde; E10g `diag_o_oleo_diluido_escorre` (o óleo d 0,9 continua mais lento que a
   água); `o_traco_longe_do_cruzamento_nao_derrete`; desempenho (`diag_desempenho_do_oleo_diluido`: células vivas).
2. **A água que sobe o relevo** (tp `a_agua_nao_sobe_a_ladeira_do_oleo`, 54 477 a 4 px): medir PRIMEIRO por que porta
   ela entra (a advecção do suspenso, o espalhar da mancha, a pegada do pincel — o campo de fluxo está refutado, §2 da
   J8). Depois: o óleo molhado (paste com `τ_y > 0`) como chão da ladeira (hoje só o `sett` conta).
3. **A composição do óleo molhado** (82–88 % do CPU do óleo diluído): cronometrar `composite_oleo.rs` por fase e não
   refazer relevo/luz a cada passo enquanto o filme só assenta.
4. Do §4 da janela 7: E9 §8 ⚑ furos; chip rápido de Dilution; `assenta.rs`/`trail/transfer.rs` em 700 linhas; E10c;
   E11 GPU; E12; E13; outros 3 modos; ajuda ilustrada; sondas para a crate dev-only; crateras.

Smoke do dono no fim da onda que mexer no produto (passos numerados, `--profile smoke`, Radeon).
