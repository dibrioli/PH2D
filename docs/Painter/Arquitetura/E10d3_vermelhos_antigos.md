# E10d3 — os quatro vermelhos antigos (bissecção de árvore inteira)

Agente D3, 01/11. HEAD `e8f35b5710`. Bissecção sempre de **árvore inteira** (`git worktree add --detach`, cada
uma com o SEU `CARGO_TARGET_DIR`), sobre `422fec2027..HEAD` (o merge-base com o `main`, 255 commits). O merge-base
dá os quatro **verdes** (`--profile smoke`). Pontos medidos (os três da lib + o censo):

| commit | real_oil (1) | Screen (2) | smooth (3) | censo (4) |
|---|---|---|---|---|
| `422fec2027` merge-base | verde | verde | verde | verde (sem os 3 controlos) |
| `1e9a32d0af` Over Oil | verde | verde | verde | — |
| `776a9ff173` | **vermelho** | verde | verde | verde (sem os 3) |
| `847b472b3d` | vermelho | verde | verde | — |
| `3eb13d38db` K-M de fábrica | vermelho | **vermelho** 5458/Δ9 | verde | — |
| `e881e53ad9` | vermelho | vermelho | verde | **vermelho** (REWET, REWET_DRAG, SOLUBILITY) |
| `0eaf6fa1ed` (pai da E4) | vermelho | vermelho | verde (os 2 hashes antigos) | — |
| `7a4bc540b7` E4 | vermelho | vermelho | **vermelho** | — |
| HEAD | vermelho | vermelho | = E4 nos dois hashes | vermelho → verde com a cura §4 |

Carga: nenhum é (c). Os quatro reproduzem-se ao mesmo número em todas as corridas, com load de 9 a 28, e o
merge-base fica verde sob a mesma carga.

## 1. `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` — (a) regressão, cura fora do meu alcance

- **Primeiro vermelho:** `776a9ff173` (30/10, «reverte o Over Oil»). O pai com código, `1e9a32d0af`, dá verde.
  Entre os dois só há commits de docs.
- **Mecanismo:** `776a9ff173` pôs o `PINCEL_DE_OLEO` (−1) no `Ecra::Cobre::oleo` da SESSÃO quando o Real Oil está
  na mão (`wp/sim.rs:417-424`, `gather_params`). O render usa essa lei da sessão (`wp/render.rs:404`) e
  `Ecra::com_oleo` (`wp/assenta.rs:155-182`) escolhe o ramo pelo sinal: `o < 0` → `com_opaca` (o tapar extra fica);
  `o ≥ 0` → `oleo` = fracção da célula (o extra sai). Ligar ou desligar a caixa troca o ramo de toda a célula com
  óleo já pousada, e a tela recomposta muda. O Over Oil guardava a escolha na CÉLULA (`Params::sobre_oleo` → `None`
  com o Real Oil na mão, «pousa na célula»). O revert tirou isso e a escolha passou a ler a mão.
- **Comentário que mente:** `Sim::oleo` (`wp/sim.rs`) diz «ligá-lo ou desligá-lo não muda a tinta pousada».
- **Mesma família:** `remolha_tests::o_real_oil_na_mao_nao_empalidece_a_aguada_sobre_o_oleo` (`#[ignore]`, J11
  item 0) é o mesmo sintoma visto pelo dono. A 1.ª cura (`43b535277b`, tirar o `PINCEL_DE_OLEO`) foi revertida em
  `a4c8c80e3d` porque o óleo deixava de cobrir o óleo.
- **Cura proposta** (dono 27/10 W3b «o painel só vale para o futuro», 28/10 «o Real Oil é do TRAÇO»): a escolha
  do ramo vai para a célula, como o Over Oil fazia.
  1. `wp/grid/oleo_celula.rs`: um campo novo `OleoCelula::pincel_de_oleo: f32`. O `do_pincel` escreve `p.oleo`
     (1 com a caixa ligada, 0 sem ela). O pouso SOBRESCREVE (a última mão que pintou a célula), sem misturar pela
     massa. Atenção ao portão do tamanho da célula (hoje 24 B).
  2. `wp/assenta.rs` `com_oleo`: o ramo `o < 0` passa a `c.pincel_de_oleo > 0.5`, e o `PINCEL_DE_OLEO` sai.
  3. `wp/sim.rs:417-424`: `oleo: 0.0` sempre na lei da sessão (D2).
  4. Callers sem mudança de assinatura: `render.rs:448` (D1), `drying.rs:177` (D2), `trail/transfer.rs:241,658`,
     `tools/smear.rs:276`, `tools.rs:272`, `grid/canvas_ops.rs:65`.

  Portões a correr: este teste, `pintar_tudo_com_o_real_oil_e_a_imagem_de_antes_ao_byte` (o óleo cobre o óleo),
  `assenta_bit_tests`, `oleo_sobre_oleo_tests` e as miniaturas Oil/*. Não apliquei a cura: toca `wp/render*`
  (D1), `wp/sim.rs` e `wp/drying*` (D2).

## 2. `a_identidade_do_modo_nao_pinta_sobre_a_tela` — (a) o preto do K-M não é preto; cura fora do meu alcance

- **Primeiro vermelho:** `3eb13d38db` (30/10, «Pigment mixing (K-M) é do pincel — ligado de fábrica»). O pai
  `847b472b3d` dá verde.
- **Medição** (sonda temporária no HEAD, já apagada): o pincel preto em Screen com `km_mixing = false` dá 0 texels
  e Δ 0. Com `true` dá 5458 texels e Δ 9. Na grade com K-M, 5458 células (exactamente os texels que mudam) guardam
  cor até **12,709**/255, sendo o pincel 0. O preto em Mix sobre o branco fica 64 sem K-M e 74 com ele.
- **Mecanismo:** a média K-M da advecção (`wp/colorops.rs:380` `km_weighted_mean_color`) passa cada canto por
  `ks_of_srgb255`, que prende a reflectância em `R_FLOOR = 1/255` linear (`ph2d-pigment/src/transfer.rs:175`).
  A volta, `reflectance_of_ks`, devolve 1/255 linear = **12,709** em sRGB. Basta a água mover a tinta para o preto
  puro virar 13/255. O Screen dessa cor clareia o vermelho até Δ 9. O teste não mente: o Blend chega à água, mas a
  tinta na água deixou de ser preta.
- **Por que (a) e não (b):** a lei do K-M já tem os extremos exactos (`km_mix_endpoints_are_bit_exact_at_any_colour`
  para `ColorMix::Km.mix`). A média ponderada fura essa lei quando todos os cantos têm a MESMA cor. Nenhum doc pede
  que o preto do pincel fique a 13.
- **Cura proposta** (`wp/colorops.rs`, de nenhum dos donos listados; não a apliquei porque está fora de `tp/`):
  em `km_weighted_mean_color`, se todos os `count` cantos do canal são iguais, `out[ch] = colors[ch]` sem passar
  pelo K/S. É a identidade dos extremos, como no `mix`. Diff:
  ```rust
  for ch in 0..3 {
      let c0 = colors[ch];
      if (1..count).all(|k| colors[k * 3 + ch] == c0) {
          out[ch] = c0;
          continue;
      }
      let mut ks = 0.0;
      …
  ```
  ⚠️ Isto muda bits em toda cena K-M com cantos iguais (a volta tabela→sqrt não é exacta ao ulp). Antes de
  regravar, rode os portões ao bit com K-M (`advect_jacobi_bits`, `flow_bit_tests`, `assenta_bit_tests`) e as
  miniaturas, e mostre que só sobem os pretos ou escuros abaixo do piso. Alternativa mais estreita: tirar o piso
  só na VOLTA, quando o `ks` médio é o de um canto. A decisão é técnica e fica com o coordenador.

## 3. `smooth_edges_no_zero_e_o_desligado_de_antes_ao_byte` — (b) a lei mudou de propósito, hashes regravados

- **Primeiro vermelho:** `7a4bc540b7` (E4 receita × quantidade). O pai `0eaf6fa1ed` dá os dois hashes gravados.
- **Mecanismo:** a E4 recalcula o pigmento e a água de cada dab por φ·h (ADR-0197 §1.1, dono: «Pigment + Water =
  só a diluição; Volume = só a quantidade; a água invertida pela pressão morre»). O traço do teste pousa outra
  tinta, e a tela muda nas duas razões.
- **Prova:** pai `0eaf6fa1ed`: `0xf8e04b0d33bb552b` · `0xa3505d05ebc7f8b6` (os gravados). E4: `0xed705054b74a3349` ·
  `0x99a8bc53e3aad95f`. HEAD: os mesmos dois da E4, portanto nada depois da E4 os mudou. O E10 tirou o
  `opacidade = 1.0` do `wet_tool` (a Opacity não tem leitor desde a E4) e o hash ficou igual. A E4 regravou pelo
  mesmo porquê `flow_step_bits` (`43dd8acc3d`) e `advect_jacobi_bits` (`286176cd21`).
- **Cura aplicada:** `tp/wetpaint/grid_ratio_tests.rs`. Os dois hashes são medidos ANTES de asserir (a razão 1
  escondia a 2), ficam regravados com o porquê no doc-comment e com este doc citado. O teste do «desligado = o over
  de antes» (`smooth_edges_desligado_…`) continua a provar a semântica.
- **Resultado:** `grid_ratio_tests` 11 verdes, 1 ignorado.

## 4. `o_censo_dos_controlos_so_encolhe` — (b) a régua não exprime um controlo de PAR, entradas com motivo

- **Primeiro vermelho:** os três controlos entraram na tela já inertes no censo: `84ed24c8fc` (Re-wet e
  Solubility no cartão, J10, 30/10) e `6c243555f0` (Drag). O ficheiro do censo não foi tocado nesses commits.
  Vermelho medido em `e881e53ad9` (árvore inteira). No merge-base (sem os controlos) e em `776a9ff173` está verde.
  Ponto não medido: o próprio `84ed24c8fc`.
- **Mecanismo:** a água dissolve `Re-wet × Solubility` da tinta ASSENTADA de um traço anterior
  (`wp/trail/transfer/remolha.rs:42`, `f = forca × soluvel`). O Drag arrasta só o solúvel (`wp/tools/smear.rs:174`).
  A fábrica do Wet Paint tem os três a 0 (`tuning/defs.rs:328-330`, os três com padrão 0; o Re-wet do cartão é o `Knob::Dissolve`, `wetpaint_settings/tabelas.rs:138`)
  e o censo muda UM controlo de cada vez. Sozinho, cada um multiplica pelo outro, que é 0. Não estão mortos: o
  portão `remolha_tests` liga os dois e mede a dissolução.
- **Cura aplicada:** `ph2d-panel-painter-layers/tests/it/o_censo_so_encolhe.rs`, três entradas WetPaint na lista
  com o motivo «age só em PAR». A lista só encolhe: no dia em que a fábrica tiver Solubility > 0, as linhas de
  Re-wet e Drag reprovam com «AGORA MUDA A TINTA» e saem.
- **Resultado:** o censo está verde no HEAD (34,7 s).
- ⚠️ À parte, vermelho ANTERIOR que não é dos quatro: `remolha_tests::a_aguada_re_molha_o_oleo_soluvel` está
  vermelho no HEAD (linha 154, «o azul debaixo da aguada caiu …»). É o portão de vida destes controlos e merece
  uma bissecção própria.
