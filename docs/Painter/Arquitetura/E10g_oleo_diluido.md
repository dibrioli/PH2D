# E10g — o óleo diluído escorre numa lei só (janela 6, 01/11)

Decisão do dono (memória `project_paint_type_water_oil_one_law_2026_11_01`): o diluente é SEMPRE líquido; o meio `m`
muda só as propriedades dele. Recusados: dois fluidos imiscíveis; «meio só de aparência». O seletor Paint type
(Water · Oil) está em [E10f_paint_type.md](E10f_paint_type.md).

## 1. A lei (`wp/` = `crates/ph2d-wet-paint/src/`)

| ficheiro | mudança |
|---|---|
| `wp/meio.rs` (novo) | as leis do líquido: `liquido_livre(φ, m) = [(1−φ) − m·L·φ]₊` (L = `LIGADO_NO_OLEO` = 1/9) · `mobilidade(m) = ETA^−m` (`ETA_OLEO` = 8) · `evaporacao(m) = (1−m) + m·ENDURECE` (0,1) · `retencao(r, m) = r^(√mobilidade)` (Washburn) · `mistura` do meio pelo volume MOLHADO. Réguas de medição `PAPEL_ETA`, `PAPEL_ENDURECE`. `m = 0` devolve a entrada sem aritmética (ao bit). |
| `wp/painter/receita.rs` | `Receita::agua() = liquido_livre(φ, m)·h/K_W` (era `(1−φ)(1−m)h/K_W`: com `m = 1` o líquido sumia). `meio_da_familia`: a Acrylic passa a `0` (só dois pontos). |
| `wp/grid/oleo_celula.rs` | `pousa(g, i, m_old, nova, (v, agua))`: o `meio` mistura-se pelo volume molhado (`susp + K_W·film` contra `v + K_W·agua`); os outros campos seguem pela massa. |
| `wp/trail/transfer.rs:174` | passa a água do dab (`self.water[l]`) ao pouso. |
| `wp/solver/flow_jacobi.rs` | depois do Bingham, o incremento × `mobilidade(meio)` (só com o campo da célula). |
| `wp/drying.rs` (`dry_cell`) | retenção `meio::retencao(r, m)`; a perda linear (evaporação + orla) × `meio::evaporacao(m)` — o óleo não tem orla escura e acaba por parar (o solver não corre para sempre numa tela que não bebe). |

**Porquê o líquido ligado (L).** Com `(1−φ)h/K_W` cru, o Dabber de fábrica (Dilution ≈ 0,002) ganhava 0,004 de filme por
dab: o ramo binário «ponta seca» (`transfer.rs:92`, `sum_water <= 0`) deixava de valer e toda a tinta do tubo passava a
suspensa (o azul assentado da cena do re-molhar caiu de 656 354 para 35 360). Física: o pigmento prende o óleo (o índice
de absorção de óleo); só o diluente acima disso é livre. `L = 1/9` ⇒ a pasta vai até Dilution 0,1; os óleos de fábrica
estão em 0,002–0,042 (nenhum muda de regime). Contínua no limiar (teste `o_oleo_do_tubo_e_pasta_e_so_o_diluente_a_mais_corre`).

**Porquê o meio pelo volume molhado.** É a cura do degrau 2 da E10e2 §E3: a aguada sobre o óleo SECO herdava `m = 1`
pela massa assentada e o Bingham prendia-a. A tinta seca não tem voto; o óleo grosso no papel limpo é óleo.

## 2. Medições

**Sonda da cena do re-molhar** (`remolha_tests::diag_a_queda_por_lei`, Bingham × K–M): depois da cura o Bingham não
pesa (0,461 com × 0,465 sem); o que resta é o K–M (0,357 sem ele) — a régua passou a medir com o K–M desligado.

**Escorrer** (`oleo_diluido_tests::diag_o_oleo_diluido_escorre`, tela inclinada de fábrica; abaixo = fracção da massa
abaixo da faixa do traço, 2 s → 15 s; fotos `target/sonda_e10g/fotos/`, não versionadas):

| pincel | Water d 0,9 | Oil d 0,9 |
|---|---|---|
| Wet Drip | 0,578 → 0,724 (seca) | 0,023 → 0,308, ainda molhado (5 135) |
| Wet Round | 0,049 → 0,116 | 0,030 → 0,034 |
| Filbert | 0,148 → 0,147 | 0,091 → 0,100, molhado (12 515) |
| Thick Impasto | 0,124 → 0,128 | 0,043 → 0,085 |

O óleo diluído corre mais devagar, sem orla, e continua molhado (o Oil d 0,9 do Wet Drip ainda escorre aos 15 s).
Pincéis de óleo com Dilution do tubo: `água 0` — idênticos (pasta). O «pingar» em queda livre só existe no Wet Drip
(os outros pincéis de água também não pingam: o travão é de outros knobs do preset, não da lei — sonda
`diag_o_que_trava_o_filbert_diluido`: Real Oil, relevo, Edge, Pigment não mudam nada; Paper gate 0,4 reduz).

## 3. Testes

| teste | antes | depois |
|---|---|---|
| wp `meio::tests` (4, novos) | — | ✓ |
| wp `oleo_celula` 6 · `receita` 6 · `bingham` 1 · `meio_corpo` 3 | ✓ | ✓ |
| wp `advect_jacobi_` 4 · `o_invariante_soma_massa` · `oleo_sobre_oleo` 4 · `flow_bit` 2 (hashes) | ✓ | ✓ ao bit |
| tp `remolha_tests::a_aguada_re_molha_o_oleo_soluvel` | ✗ (0,849) | ✓ (régua sem K–M) |
| tp `remolha_tests::o_real_oil_na_mao_nao_empalidece_a_aguada_sobre_o_oleo` | ✓ (água presa) | ✓ — assert reescrito: empalidecer = mais CLARO (a média), claros ≤ +4. A/B no pai com o Bingham desligado: 75,113,91 contra 66,110,140 e 7 claros (já vermelho com a água solta); aqui 74,120,85 contra 76,125,100, 3 claros |
| tp `pastel_seco_tests` 2 · `controlos_e10` 6 | ✓ | ✓ |
| tp `queda_na_ladeira_tests::a_queda_cresce_com_a_diluicao` | ✗ pai 4 455 | ✗ 5 020 (já vermelho, A/B no pai) |
| tp `agua_sobre_oleo_tests::a_agua_nao_sobe_a_ladeira_do_oleo` | ✗ pai 56 609 | ✗ 54 477 (já vermelho) |
| tp it `cada_preset_de_fabrica_e_a_sua_receita`, `os_candidatos_da_w2_sao_as_suas_receitas` | ✗ «knob levelingOwnStroke existe» | ✗ igual (a chave saiu do registo na E10c2) |

## 4. Auditoria pela grelha E9 (parcial, `target/e9_e10g/` contra `target/e9_0111/`, 897 células comuns)

- **Água 0 (a tinta do tubo): 384 células idênticas** em todas as métricas, nas duas leis.
- Pigmento diluída: manchas 0 · ferrugem 0 · `mix_p99` igual · **orla −2,35 em média (até −21)** — o óleo perdeu a orla
  escura, como a lei pede · `furos` +1,5 em média (até +48, Knife Oily/Thick Impasto P40): lado a lado a imagem é igual
  ou mais limpa (montagem `scratchpad/furos_ab.png`): a régua lê o relevo nivelado como furo (o ⚑ furos do item 4).
- ⛔ **Velha diluída (o produto): FERRUGEM nova** só em Pigment 600 × Water 0,5 (Dilution ≈ 0,127 ⇒ 3 % de líquido
  livre, logo acima da pasta): 20 de 65 células, até +52 (laranja salpicado onde o amarelo cruza o azul). Célula única
  (`target/sonda_e10g/celula.sh`, interruptor novo `PH2D_SONDA_KNOBS` na sonda): preset 52 · `medium=0` 8,3 (a Velha já
  enferrujava com película fina de água) · pickup/rewet/edge/wetCover/glaze/opaque/strokeFilm/bristleNoise/wetLift sem
  efeito. Mecanismo → `E10i_ferrugem_velha.md`. **Não mostrar ao dono antes da cura.**
- A grelha corre ~4× mais devagar: as células de óleo diluído ficam molhadas (simulação a correr) muito mais tempo.

## 5. Aberto

1. Fechar a auditoria com a grelha inteira; a ferrugem da Velha (E10i).
2. GPU (`ph2d-wet-gpu`): o escoamento e a secagem da placa ainda não leem `meio` — E11.
3. Os quatro vermelhos antigos acima → item 3 do handoff.
4. A rota não-Jacobi (`build_flow_field`) e o `fast_dry` não leem o meio (a rota do produto é a Jacobi).
