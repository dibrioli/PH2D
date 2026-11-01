# J7 — o óleo pouco diluído pousa como PASTA e mistura (janela 7, 01/11)

Defeito (handoff da janela 6 §3.1–§3.2): na lei Velha, óleo P600 × Water 0,5 (Dilution ≈ 0,127, 3 % de líquido livre),
o amarelo sobre o azul TAPAVA (bege) em vez de dar verde como de manhã; o Filbert Dry pr0,6 ficava salpicado.

## 1. O mecanismo (medido, sonda `sonda_varredura_oleo`, uma célula)

- Não é o tempo de secar: pausa 300 / 1 200 / 6 000 ticks antes do amarelo dão o MESMO pixel `(226,209,159)`.
- Não é o meio: `medium=0` dá `(227,209,161)`; `opaque/oilDepth/glaze/wetCover/pickup/rewet = 0` — sem efeito ao bit.
- É o REGIME: até à E10g o óleo do tubo pousava sem água ⇒ `ponta_seca` ⇒ `pousa_seco` (a cor entra no assentado
  pela mistura K–M: verde). Com o diluente líquido (E10g), 3 % de líquido livre fazem o dab inteiro ir pelo ramo
  molhado: o amarelo fica em suspensão POR CIMA do azul e o ecrã `Cobre` opaco tapa-o; o assentar preserva o ecrã.
  Um degrau no limiar Dilution 0,1: de «mistura» para «tapa».
- Filbert Dry pr0,6 com a fracção de pasta forçada (sonda temporária): `q = 1` → 5 degraus (a manhã), `q = 0,95` → 12,
  `0,9` → 13. Os 5 % molhados que ficam por cima tapam o verde cerda a cerda.

## 2. A lei

- `wp/meio.rs::pasta(m, v, água)` — a fracção do pigmento que o óleo PRENDE: `m·L·v/K_W ÷ (m·L·v/K_W + água)`.
  `m = 0` ⇒ `0` (a água ao bit); sem líquido livre ⇒ `1` (a ponta seca); contínua no limiar. Dilution 0,127 ⇒ 0,764;
  0,9 ⇒ 0,012.
- `wp/trail/transfer.rs` — `q` UMA vez por dab (as médias da janela: a receita do dab é uma só) e só fora do regime
  da ponta seca (o tubo de fábrica fica ao bit).
- `wp/trail/transfer/deposito_tinta.rs::pousa_pasta` — a fracção `q` pousa como a ponta seca (`pousa_seco`); o resto
  fica molhado com a cor da MISTURA na fracção `q` (`mix(ponta, assentado, q)`): óleo molhado sobre óleo molhado
  mexe-se; o líquido livre leva a mistura. Sem a mistura da cor o Filbert Dry ficava em 21 degraus.
- `wp/meio.rs::liquido_livre` — `1 − φ·(1 + m·L)`: no limiar do óleo puro dá 0 exacto (a forma de antes deixava
  `1,1e−16` e o `meio::tests::o_oleo_do_tubo_e_pasta…` estava vermelho).

## 3. Medições (Velha, papel padrão, P600 W0,5 n1)

| pincel | manhã (`e9_0111`) | E10i | J7 |
|---|---|---|---|
| Flat Bristle pr0,2 — cruzamento (plano) | verde `(189,196,163)` · 3 degraus | bege `(226,209,159)` · 0 | verde `(188,195,164)` · 2 |
| Tilted Shade pr0,2 | verde · 2 | bege · 0 | verde · 1 |
| Filbert pr0,2 | verde · 3 | bege · 0 | verde · 5 |
| Filbert Dry pr0,6 | verde · 5, ferrugem 0 | ferrugem 3,48 · 16 | verde · 9, ferrugem 0 |

Escorrer (`oleo_diluido_tests::diag_o_oleo_diluido_escorre`): Water idêntico em todas as linhas; Oil d 0,9 igual
(Wet Drip 0,023 → 0,308, água 5 135; Filbert 0,100); Oil d 0,3 muda na 3.ª casa (Dabber 0,040 → 0,036).

## 4. Testes

| teste | antes | depois |
|---|---|---|
| tp `o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` (novo, 4 pincéis: G > R, ferrugem 0, degraus ≤ 10) | ✗ (cura desligada) | ✓ |
| wp `meio::tests` 5 (+ `a_pasta_do_dab_e_continua_no_limiar_e_nula_na_agua`) | ✗ 1 (o 1e−16) | ✓ |
| wp `oleo_celula` 6 · `receita` 6 · `bingham` · `meio_corpo` 3 · `advect_jacobi_` 4 · `o_invariante_soma_massa` · `oleo_sobre_oleo` 5 · `flow_bit` 2 · `deposit` 14 · `assenta` 27 · `bit` 31 · `tinta` 72 | ✓ | ✓ (hashes ao bit) |
| tp `remolha_tests` 3 · `controlos_e10` 6 · `pastel_seco` 2 · `portao_mae_f0` 3 · `receita_e4` 4 · `km_no_preset` 3 · `glaze_no_preset` 5 | ✓ | ✓ |
| tp `a_queda_cresce_com_a_diluicao` · `a_agua_nao_sobe_a_ladeira_do_oleo` · it `a_guache_opaca_cobre_o_azul_seco` | ✗ 5 020 · 54 477 · ✗ | ✗ iguais (vermelhos antigos) |

## 5. Aberto

**Smoke do dono (01/11, Paint type + óleo diluído + cruzamento): OK.** Duas observações, para a FILA (o dono: «não
precisa ver agora, manter o plano»):

- **O escorrimento ignora o relevo da tinta já pousada**: o líquido passa por cima do relevo anterior como se não
  existisse, em vez de desviar o trajeto. A mistura de cor ao escorrer está boa. (Família da ladeira do óleo —
  `agua_sobre_oleo_tests::a_agua_nao_sobe_a_ladeira_do_oleo`, vermelho antigo 54 477.)
- **Desempenho muito aquém** com o óleo diluído (fica molhado ⇒ a simulação corre muito mais tempo; a grelha E9 ficou
  ~4× mais lenta). Medir antes de mexer (E11 GPU lê o meio?).

1. A GPU (`ph2d-wet-gpu`) não tem a pasta (E11, com o resto do meio).
2. A lei `Pigmento` (`deposito_tinta::Pigmento::pousa`) não passa pela pasta — o produto é a Velha; decidir na E12.
3. **Grelha E9 inteira FEITA** (`target/e9_j7/`, 3 240 células, 2 h 28 min com load ~70): contra `e9_0111`,
   ferrugem 0 em 3 238 de 3 240 (máx. 0,01, Knife Stain P2000), manchas −0,23 em média; Pigmento idêntica em ferrugem,
   manchas e degraus. Piores (fotos lado a lado manhã · E10g · J7): Filbert P600 W0,5 e P2000 W1 pr1 n3 (degraus 7 →
   32/34) — de manhã o amarelo tapava, agora as duas tintas molhadas misturam-se em verde (a mistura aprovada no smoke);
   Knife Oily P600 W1 pr0,2 n3 (`mix_p99` 0 → 64,6) e Express Oil P600 W0,5 pr0,6 n3 (furos +50) — a faixa
   verde-clara do líquido que escorre com a mistura, frente em degraus: a família da observação 1 do dono. Parcial
   anterior
   (582 células, `scratchpad/cmp.py`-style, contra `e9_0111`): **ferrugem 0 em todas**, manchas −0,18 em média;
   contra `e9_e10g`: degraus −0,01, furos +0,05 — a J7 não mexe nos números da E10g. Os «degraus» que subiram
   (Filbert P40 W0,5 pr1: 11 → 19 na E10g, 20 na J7) e os «furos» (Dabber 4 P40 W1: +25, foto MAIS limpa) vêm da E10g
   e do ⚑ da régua. Visto na foto do Filbert P40: com a J7 o líquido diluído leva a mistura verde-clara 2–4 px para
   baixo do cruzamento (a tela inclina), com a frente em degrau da grelha do escoamento.
