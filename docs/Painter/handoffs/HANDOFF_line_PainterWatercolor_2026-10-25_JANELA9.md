# Handoff — line/PainterWatercolor, janela 9 (25/10) — Wet bloom = o tubo do Rebelle; a queda de FPS do dono AINDA não foi medida

Commit: `5291edbc75`, sobre `b03ce4f216`. Texto exato: `git show 5291edbc75`.
Anterior: [`HANDOFF_..._JANELA8.md`](HANDOFF_line_PainterWatercolor_2026-10-25_JANELA8.md). Seguinte: [`PROMPT_NOVA_JANELA_10.md`](../smoke_2026-10-24/PROMPT_NOVA_JANELA_10.md).

## 1. Wet bloom reescrito (`5291edbc75`)

Lei de `Knob::WetOpen` em `crates/ph2d-wet-paint/src/trail/molhado.rs` (+ `trail/transfer.rs`: `TouchedRect::union` do levantar no sujo).
Passou de «um disco a meio da faixa» para «o tubo do Rebelle». Três mecanismos achados (sonda sem janela, `wetpaint::molhado_tests`):

1. **Aresta reta.** A lei J5 punha UM disco a meio da faixa medida pela linha do traço; o traço que COMEÇA dentro da tinta molhada e corre ao
   longo dela esticava só dentro do disco (o esticão subia 0 → 0,37 em ~10 px): aresta perpendicular ao traço, retângulo nos eixos num traço horizontal.
2. **Escrita fora do sujo.** O que a lei escreve no grid FORA da pegada (empurrão antigo / levantar novo) não entrava no sujo; a tela mostrava a tinta
   de antes até a simulação lá passar («começam retangulares e resolvem-se»).
3. **Empurrão O(raio³).** 212 ms num só dab (raio 162 células, tela 3×).

Tip pickup (`Knob::TipReservoir`) é inocente nos dois (0,58 × 0,52 ms/quadro).

**Oráculo** (Rebelle 8, Liner): [`oraculo_janela9/README.md`](../smoke_2026-10-24/oraculo_janela9/README.md), resposta (b). O traço dentro do vermelho
molhado alarga 1,47–1,53× por TODO o caminho, largura constante, borda macia 8–10 px, só o lado molhado abre, a borda da faixa não se move;
o `sobre` (1,45) é o mesmo tubo recortado pela faixa.

**Lei nova:** sonda por dab de cada lado até onde há tinta molhada de outra cor (≤ r·(1+WetOpen)), suavizada em 0,3 r de caminho; esticão por janela
pela direcção da célula ao caminho (raios nos extremos interiores, tampa redonda no início), 5 destinos ±16 % (borda macia), buffers da caixa
reaproveitados; levantar `WetOpenPush × outra_tinta` debaixo do tubo, uma vez por traço (vetor denso + retângulo tocado), em linhas do grid
(`walk_rows2`, rota do transfer quando forçada) e declarado no sujo.

**Medido** (tela 1200×720, pincel 90/72 px, mínimo de 5 intercalados, load ~6): pior quadro ao mesclar 215 → 11,9 ms; média 7,0 → 1,9 ms/quadro (sem a lei 0,7).

### ⛔ Recusas MEDIDAS (não reconstruir)

| Recusado | Medido |
|---|---|
| Estender o caminho em raios no levantar | corte reto na borda da caixa |
| Levantar só nas células com pigmento | borda dura (a borda do pigmento) |
| `SUAVE_RAIOS = 1` | Liner 1,27× no `sobre` (estreito demais); hoje 0,3 → 1,36 |
| `HashMap` para o registo do levantar | 15,5 ms/janela; hoje `Vec` denso |
| Levantar em série | ~19 ms no pior quadro; hoje linhas em paralelo → 11,9 |

## 2. Smoke do dono (25/10) — metade resolvida

Veredito: «Smoke das manchas e retângulos OK. Resolveu.» MAS «ainda com severa queda de FPS na mistura de tinta molhada».
Ordem nova: «Auditoria geral dos principais parâmetros, suas ações e performance.»

Cena do dono (fotos `../smoke_2026-10-24/fotos_janela9/painel_wet_flat_cima.png`, `painel_wet_flat_baixo.png`): Paint Mode Wet Paint, preset
Wet Flat, Size 122 px, Water 1.00, Opacity 1.00, Blend Mix, cor amarela, Grid Size (px) 1, Flow Grid (x) 1 → «fluid 2048x2048 - flow 2048x2048»,
Pigment 50, Pickup 0.005, Dry Speed 1.00, Edge Darkening 12, Gravity 0.0050, Leveling 0.50, Level Clamp 0.20, Capillary 0.20, Brake 1.50,
Brush dust 0, Backruns 0, Granulation 0, Diffusion 0, Wet bloom 0.50, Tip pickup 0.46, Smooth Edges 0, Show Wet off, Live Water off.

**CHAVE:** a sonda de FPS da janela 9 (`wetpaint::molhado_tests::sonda_fps_mesclar`) NÃO reproduz isto: tela 1200×720, pincel de fábrica, não grade
2048² / Wet Flat / 122 px. A queda restante está NÃO MEDIDA. Candidatos: passes da sim a 2048² quando a região ativa cresce com duas tintas molhadas
(etapas de `crates/ph2d-wet-paint/tests/it/measure_stage_cost.rs`); `molhado_estica` (6,5 ms/janela a 3× — o scatter ainda é serial); transfer/depósito/K–M por dab;
composite de retângulos sujos grandes.

## 3. Prova de fecho

Testes por nome verdes. ph2d-wet-paint, `tests/it`: `molhado_rows`, `fingerprint`, `deposit_rows`, `parallel_rows`, `resumable_step`, `product_doors`,
`acceptance`, `cor_por_celula`, `flow_symmetry`; lib: `sim_resumable`, `assenta`. ph2d-tool-painter, `tests/it`: `familias::aguarela`, `familias::tintas`,
`cada_preset_parece_a_sua_midia`; lib: `wetpaint::molhado_tests`, `os_knobs_do_pincel_no_cartao_chegam_ao_motor`, `offthread`,
`os_grupos_avancados_do_tuning`, `wetpaint::tests::`. Gates novos (nomes no commit): `o_azul_que_corre_dentro_do_vermelho_molhado_alarga_como_um_tubo`,
`o_azul_abre_uma_mancha_redonda` (1,36 · 1,05), `o_levantar_do_molhado_entra_no_sujo_e_a_tela_e_a_do_recomposite`, `o_levantar_do_molhado_e_o_mesmo_nas_duas_rotas`
(mutações: levantar sem sujo → 3 282 px velhos; linha saltada na paralela → vermelho).

⚠️ O filtro `borracha_por_meio` casou ZERO testes (o handoff da janela 8 o listava como verde): nome velho; achar o teste atual da Borracha por meio antes de citá-lo.

`clippy -D warnings --all-targets` e `fmt` limpos em `ph2d-wet-paint` e `ph2d-tool-painter` (saiu também um import sem uso pré-existente em `tests_doc22.rs`
que partia o clippy). Suíte completa NÃO corrida (é do integrador). Contadores: nenhum registo/FIELDS/Tuning mexeu (delta 0).

**Um merge pode partir:** a assinatura `molhado_estica(g, p, mode)`; o novo `mod molhado_rows` em `crates/ph2d-wet-paint/tests/it/main.rs`; o novo
`mod molhado_tests` em `wetpaint.rs`; qualquer outra linha que mexa em `trail/molhado.rs`/`trail/transfer.rs`.

Sondas (ignoradas; comando no cabeçalho do ficheiro): `wetpaint::molhado_tests::sonda_retangulos` e `sonda_fps_mesclar`.

## 4. ABERTO

1. **NOVO:** queda de FPS ao mesclar na cena do dono (§2), por medir; e a AUDITORIA GERAL dos parâmetros do cartão/painel do Wet Paint (ação, leitor, custo).
2. Decisão do dono PENDENTE — célula do Rebelle em forma fechada (muda bits) para encurtar `a_largura_e_a_prometida`.
3. Decisão do dono PENDENTE — Borracha da Watercolor que mantém a sessão: (a) buraco limpo ou (b) levantar com mata-borrão
   (mapa do modelo da sessão em [`HANDOFF_..._JANELA8.md`](HANDOFF_line_PainterWatercolor_2026-10-25_JANELA8.md) §5.3).
4. Da janela 7: Accumulate ON + passagem translúcida (Flow 0,3) sem Continuous escurece a cauda; ressalto ~+10 % na entrada da janela da cauda sem Continuous;
   Wet presets não gravam Press; S-curve, Light scatter, Speed lag; canal do «Reset to brush preset»; defeito 5 do Pen (Tilt smooth/Rotation); Dry brush
   (`ExtDryBrush`) knob morto; decisão 2 do dono (escondidos do Digital sem linha).
5. Da janela 4: família «mundo» das macros lê a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da Borracha no Wet; Texture Eraser à Strength 1; a shell a 23 linhas do tecto.

Integração no main: só por ordem do Enio (`/pd-integracao`); a suíte completa é do integrador.
