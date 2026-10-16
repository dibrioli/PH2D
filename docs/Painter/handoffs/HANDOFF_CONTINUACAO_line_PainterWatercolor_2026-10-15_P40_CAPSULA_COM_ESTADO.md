# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 40: a cápsula carrega o ESTADO do dab (2026-10-15)

## §1 — Coordenadas

Sobre o `main` `0aab2c1cb6` (merge-base, sem rebase). Sem push. [Prompt 40](../NewFeaturesWetPaint/40_PROMPT_capsula_carrega_o_estado_do_dab.md)
· lista [26](../NewFeaturesWetPaint/26_ABERTOS.md) A11 (feito) · [BUGS #57](../BUGS_painter_fechados_41_a_46.md) ·
[prancha 40](../NewFeaturesWetPaint/40_prancha_capsula_com_estado.png). Anterior: [P39 §8–§9](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P39_W5.md).
Commits: `2f81df8a33` (a cura + portões) · `0456c1a7bb` (régua do traço inteiro, controlo da lei velha, ramos) ·
`5c63352ebe` (portões dos sobreviventes, prancha) · o deste fecho (docs).

## §2 — Passo 0: o pincel real do dono

O app só guarda em disco os pincéis do Wet Paint (`~/.local/share/ph2d/brushes/estado.json`); o do Impasto não
existe em ficheiro. Sonda em lote (`tests/sonda_p40_passo0.rs`, `#[ignore]`): 36 condições de pincel e de gesto
(velocidade do rato 0,5–40 px/evento, tiques, direcção, tamanho 10–200, Spacing 0,1–1, Stabilizer, Input samples,
Jitter, Spray, Simetria, Strength, Accumulate, Speed, Sketchy, Rake, Flow, Flatten, Push, Dash, Rough, métodos)
⇒ **nenhuma condição deixa a caixa ligada e a cápsula MORTA**. A que mais se parece com a foto do dono é a
**caneta com pressão** (o raio muda por dab): Impasto 0,099 contra 0,147 sem a caixa — a cápsula quase não
ajudava. Fica a pergunta ao dono no smoke (rato ou caneta?). As condições que agiam MAL eram a família do prompt.

## §3 — A lei: cada sub-dab é o dab INTERPOLADO

- `Dab::ante: Option<Ante>` (`Ante` e `Dab::capsula` vivem em `capsula.rs`; raio, cobertura, dir, rotation, arc_len do dab de trás), gravado pelo `Stroke`
  (`ultimo`/`dab_ante`, ao lado de `step_from`/`dab_step`; a Simetria orienta-o com o dab).
  `Dab::capsula(&spec)` dá o estado em razões sobre a frente (o rotor relativo precisa da lei do Follow).
- `Capsula::sub_dab(k)` = `lerp(frente, trás, k/n)` de centro, raio, cobertura, rotor (nlerp) e arco. Os TRÊS
  leitores: cobertura (`dab.rs` `silhouette_capsula_com`), cor (`bands.rs` → `CorCapsula`, o mesmo `Sub`),
  relevo (`height_walk.rs` `SubPasso`, que entrega também o raio do sub-dab vencedor: a altura escala com o raio,
  `height::derive_height_at`, e o plano `radius` guarda-o para o recálculo do commit). `TexDabBasis::sub`
  (a pegada girada, o arco do Flow). A caixa e as regiões crescem para o raio de trás. O Wet Paint
  (`capsula::sub_dabs`) interpola os mesmos campos.
- **Sem Continuous, ao byte** (estado ausente ⇒ cápsula neutra; os portões ao bit de antes ficaram verdes).
- Chave de medição `capsula::com_so_o_centro` (a lei velha), só para os controlos dos portões.

## §4 — #4, #5, #6

- **#4 (confirmado: 174 cápsulas com a caixa desligada):** `stroke/speed.rs` `fill_thrown_gap` pergunta
  `lays_capsules()`; os dabs do vão levam o estado do anterior.
- **#5 (Airbrush):** a caixa só APARECE nos métodos de `uses_spacing()`, mas o motor perguntava `allows_jitter()`
  — escolhida no Space e trocado para Dots/Airbrush, ela sumia e continuava a agir. `lays_capsules` pergunta
  agora o mesmo que o painel. (A lei do Rebelle não decide: o Airbrush não tem Spacing, logo não tem o «Spacing
  contínuo».)
- **#6 (Smear e os outros modos):** medido, a caixa só muda o traço em Paint, borracha e Máscara (e no Wet
  Paint); em Smear, Blur, Clone, Inpaint, Liquify, Sculpt e Knife 0 texels. `ContinuoSemPasso` esmaece-a
  por modo; a dica diz *«…with the Paint tool, the Eraser or the Mask»*.

## §5 — Medida (régua do traço inteiro, Δ|RGB| contra o traço denso a 0,025)

| caso (Voronoi + Shape Color) | lei velha | agora | sem a caixa |
|---|---:|---:|---:|
| Impasto recto, pressão | 0,0885 | 0,0212 | 0,1343 |
| Impasto curva, pressão | 0,0793 | 0,0287 | 0,1386 |
| Impasto Flow recto | 0,1030 | 0,0029 | 0,0150 |
| Digital Flow curva, pressão | 0,0394 | 0,0082 | 0,1241 |
| Impasto Rake curva | 0,0289 | 0,0145 | 0,1246 |
| Digital Rake, ponta achatada (curva) | 0,0343 | 0,0211 | 0,0955 |

O resto com pressão é discretização (o traço denso muda de DENSIDADE com o raio; a pressão em `|seno|` tem
quinas entre dois dabs — em rampa linear o Impasto recto mede metade). ⛔ A régua `capsula::miolo` (faixa
recta em volta do eixo) mede só um pedaço de uma curva: o portão novo segue os texels do alvo.

## §6 — Portões e mutações

`capsula_estado.rs`: `o_sub_dab_da_capsula_e_o_dab_interpolado` (24 casos; barra 0,032 e < metade do «sem» e da
lei velha onde passam da barra) · `os_ramos_da_capsula_giram_e_apertam_com_o_dab` · `a_caixa_continuous_esmaece_onde_nao_age`
(13 modos/métodos: esmaecida ⇔ não muda, ao byte, e muda só onde a lei a lê). `capsula_tests::o_sub_dab_leva_o_estado_interpolado`
· `line_speed_tests::the_thrown_gap_lays_capsules_only_with_the_box` · `o_produto_corre_a_uniao` (a chave da lei velha desarmada).

Agente `mutacao`, controlo verde antes e depois: **18/18 sangram**. Os três que sobreviveram à 1.ª corrida
ganharam portão: o arco do Flow no leitor da COBERTURA só se vê com o Shape como silhueta (sob a Shape Color o
padrão é cor, lido pelo leitor da cor) ⇒ ramo «Flow, Shape sem cor» (0,011 × 0,034); as caixas da cor e do
relevo para o raio de trás eram invisíveis ponta a ponta (o dab anterior já pintou o corte) ⇒
`capsula_tests::a_caixa_cresce_para_o_raio_de_tras`. `com_so_o_centro` é o controlo de cada mutação de leitor.

## §7 — Aberto

- **Passo 0:** se o dono pinta com RATO e a grelha voltar com a caixa ligada no binário novo, o pincel dele tem
  uma condição que nenhuma sonda fixou — pedir a foto dos painéis Stroke e Shape.
- A aguada (Watercolor) e o depósito de cor dela: a cápsula só desloca o centro (mídia congelada, C3).
- Jitter de posição/escala: o «traço denso» com jitter não é um alvo bem definido (cada sub-dab sortearia o
  seu); a cápsula interpola entre dois sorteios. Não é defeito medido.
- C26 (imagem + só Falloff) segue por confirmar.

## §8 — Fecho

`cargo fmt --check` ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`; painter-brush, tool-painter,
i18n, panel-painter-layers) ✓ — 2 avisos do motor e 6 dos testes corrigidos · `nextest-impacted` (base `0aab2c1cb6`)
**18 470/18 471** em 455 s, `load` 40–50; o vermelho era o tecto de LOC (`stroke.rs` 771, `stamp_banded_tests.rs`
701) ⇒ `Ante` e `Dab::capsula` mudaram para `capsula.rs` (700/700); refeitos por nome: o tecto ✓ e os portões da
cápsula, Speed, traço, simetria, relevo e banda ✓. Smoke ao dono: na resposta da janela (Impasto com caneta e
rato, Flow, Dots/Airbrush, Smear com a caixa esmaecida).
