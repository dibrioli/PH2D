# W2_Metallic — os três presets Metallic do Wet Paint (Óleo Real W2, E4; 27/10)

> Oráculo: Rebelle 8.3.4 `Oil & Acrylic › Metallic › {Flat 3, Round 3, Splats}` (fotos de 19–20/10, com RealShader,
> `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/metallic_{flat_3,round_3,splats}_*`, `INDICE_W2.md`). Método: `04_BANCADA.md`
> + `05_REVISAO_CEGA.md`, `REGRAS_DE_CUSTO.md` (revisor `sonnet`, ≤ 4 variantes por cura).

## Ficheiros

`assets/brushes/WetPainter/Oil/Metallic/Metallic {Flat,Round,Splats}.ph2d-brush.png` — escritos por script a partir do
ficheiro vizinho + overrides (JSON canónico conferido ao byte contra os três óleos de fábrica). ⚠️ **Sem receita** em
`tests/it/familias/oleo_acrilica.rs` (crates/ é de outros agentes nesta onda) e **miniatura provisória** (o `curva_s` da
bancada) ⇒ até a janela colar as receitas (o JSON de cada ficheiro, 1:1) e regerar (`PH2D_REGERA_SO=…`), ficam vermelhos:
`the_factory_presets_are_counted`, `cada_preset_de_fabrica_e_a_sua_receita`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`,
`cada_preset_de_fabrica_volta_ao_byte` (zlib do Python ≠ miniz), `cada_pasta_de_fabrica_tem_rotulo` (grupo «Metallic» sem
rótulo: `rotulo_da_pasta` + `ph2d-i18n/src/app_painter.rs`), `a_contagem_por_meio`, e o «45» do CLAUDE.md §5 (→ 48).

## Resultado (com o Óleo Real ligado na bancada, `PH2D_EXAME_OLEO=1`, `d63561d2e2`)

| preset | nasce de | veredito | fnv1a do ficheiro | régua (óleo) |
|---|---|---|---|---|
| Metallic Flat | Filbert | **curado-não-aprovado** — o revisor achou o oráculo 6/6 (média) | `0x7704d059dd4a81d8` | 306/454 |
| Metallic Round | Dabber | **curado-não-aprovado** — o revisor chamou o NOSSO de original 6/6 e o **Dabber** de imitação; o Rebelle foi o «outro» ⇒ nada novo frente ao Dabber e longe do oráculo | `0x525570e06f17cb86` | 272/454 |
| Metallic Splats | Dabber | **curado-não-aprovado** — oráculo 6/6 (média). Sem o Óleo Real (r1) o revisor escolhera o nosso 6/6 | `0xa018fe38eb165954` | 298/454 |

Pranchas finais: `pranchas/W2_Metallic_{flat,round,splats}_oleo.png` (cegas + chaves ao lado); réguas
`exames/W2_Metallic_*_oleo_regua.txt`. Rondas antigas (sem Óleo Real): `*_r1`, `*_r2`.

### As diferenças que ficaram (revisor cego com o Óleo Real, ORIGINAL × IMITAÇÃO)

- **Flat**: halo/sombra cinza fora do traço na nossa (a sombra do Óleo Real no papel); riscas de cerdas e fios soltos nas
  pontas (o `bristleTipRagged` 1,2 da 2.ª ronda sem óleo) contra o cheio de grão fino e pontas rectas do Rebelle; a nossa
  mais escura e translúcida, o 200 px esfiapado; sobre o vermelho a nossa escurece para bordô e marca relevo.
- **Round**: o Rebelle tem vãos FUNDOS e escuros (relevo grumoso de alto contraste, sombra de borda forte); o nosso e o
  Dabber têm relevo raso e mais claro; riscas horizontais no 200 px.
- **Splats**: os nossos pontos maiores, mais escuros, densos e saturados; os do Rebelle finos, pálidos, esparsos; a 200 px
  o nosso cobre por igual, o dele deixa pontos soltos maiores.

## Valores-chave (por cima do ficheiro de onde nasce)

- **Flat** (Filbert): size 40; `pigmentPerDab` 300, `bristleStrength` 0,1, `bristleTipRagged` 1,2, `bristleNoiseSat` 0,05,
  `bristleLaneWidth` 1,5, `bristleHeight` 0, `bristlePressure` 0,3, `pressureSize` 0,1, `pressureOpacity` 0,6, `emboss` 0,2,
  `metallic` 0,6, `roughnessMul` 1,4; pincel `dab_quina` 0,75, sem grão.
- **Round** (Dabber): size 33; `emboss` 1,5, `bristleHeight` 0, `pressureOpacity` 1, `pressureOpacityCurve` 3, `pressureSize`
  0,2, `metallic` 0,6, `roughnessMul` 1,4; pincel `jitter` 0,3, sem grão.
- **Splats** (Dabber): size 18; `pigmentPerDab` 50, `emboss` 0,2, `bristleStrength` 0, `pressureSize` 0, `pressureOpacity` 0,3,
  `metallic` 1, `roughnessMul` 0,8; pincel `spray_count` 1, `jitter` 0, `spacing` 0,1 contínuo, `dab_jitter` = só o PÓ
  (`po` 3, `po_alcance` 0,5, `po_px` 1,5, `po_opacidade` 0,5) — o corpo do dab não pousa (pigmento 50), ficam os grãos.

## Rondas

| ronda | Flat | Round | Splats |
|---|---|---|---|
| pré-cura sem óleo (v1–v4) | `dab_quina` 1 alarga a curva (54 × 42 px); 0 arredonda as pontas ⇒ 0,75 | cerdas → grão Noise 30 + jitter | spray (`spray_count` 10–20) não pousa no Wet; o PÓ sim; pigmento 2000 → 50 (p90 0,85 → 0,23 × 0,16) |
| r1 sem óleo | oráculo 5/6 | oráculo 6/6 | o nosso como original 6/6 |
| r2 sem óleo (Flat, Round) | oráculo 4/6 (baixa) | oráculo 5/6 (baixa) | — |
| cura com óleo (v6–v8, `tamanhos_A`) | com o Óleo Real o grão Noise e o `emboss` 0,8 viram ONDULAÇÃO violenta ⇒ sem grão, `emboss` 0,2, pigmento 300 | o grão Noise vira CRATERAS de 20–40 px ⇒ sem grão, `emboss` 1,5 | — |
| r3 com óleo (revisor novo) | oráculo 6/6 | o nosso ≈ Dabber, o Rebelle à parte | oráculo 6/6 |

⛔ **Recusas medidas**: o grão `Noise` no pincel como «relevo grumoso» (Óleo Real: crateras/ondas, não grumos); afinar o
relevo SEM o Óleo Real (as rondas r1–r2 aprovaram o que o Óleo Real estraga); `metallic` 1 no Round/Flat (lê cromo,
`49_realshader/out/E4_Metallic_folha.jpg`: hoje | metal 1 | 0,6/1,4).

Custo: uma foto com o Óleo Real na bancada custa ~200–600 s sob carga (contra ~1 s sem ele) — 30 fotos = 19 min.

## Fila de motor (não feita: fora dos ficheiros desta tarefa)

1. Relevo grumoso de VÃOS FUNDOS (Round 3): nem o jitter nem o grão Noise dão o contraste de vãos do Rebelle sob o Óleo Real.
2. A sombra do Óleo Real NO PAPEL à volta de um traço liso (Flat: halo cinza que o Rebelle não tem).
3. Pontas que esfarelam sem fios de cerda (Flat) — o `bristleTipRagged` só faz fios.
4. O pó com tamanho relativo ao pincel e mais pálido (`po_px` fixo; o Splats do Rebelle cresce os pontos a 200 px) e o
   spray do Wet que não pousa com `spray_count` > 1 (medido: 0/0 traços medidos).
5. A foto da bancada com o Óleo Real é ~300× mais lenta que sem ele.
