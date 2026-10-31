# E5 — o ecrã pela lei nova (F4-2 §3.2, §3.3, §7 linha E5)

> 31/10/2026, `line/PainterWatercolor`. Leitor: a próxima LLM. `wp/` = `crates/ph2d-wet-paint/src/`.

## O que mudou

| ficheiro | o quê |
|---|---|
| `wp/render/tinta.rs` (NOVO) | `ecra_da_celula(lg, i) -> Option<([f64;3] sRGB 0..255, α)>` — a porta única do ecrã da célula; `altura_fechada_em_massa(&CelulaTinta) -> f32` (devolve `0.0`, `TODO(E8)`); `massa_da_altura(lg, i)` (o ponto único da altura do Óleo Real); `transparencia_da_celula` (movida de `render.rs` para abrir LOC, mesmo corpo) |
| `wp/render.rs` | `mod tinta`; `pigment_row`: `tinta_nova = p.tinta == Pigmento` (uma vez por linha) e UM `if` por camada que dá `(cor, la, la_comp)`. Ramo `Velha` = o código de hoje com a Transparency (`transparente`) calculada dentro do ramo (pura: o `la` não muda entre `na_caixa_das_tintas` e a composição) — mesmo bit. 696 → 690 linhas |
| `wp/oleo_borda.rs:9` `albedo_da_celula` | com o campo `Grid::tinta` presente usa `ecra_da_celula`; sem ele, `cor_e_alfa` como hoje |
| `wp/oleo_campo.rs:67` | a massa da altura por `massa_da_altura` (= `sett + susp` ao bit sem o campo; `+ altura_fechada_em_massa` com ele, hoje `+0`) |
| `wp/render/tinta_tests.rs` (NOVO) | os três testes abaixo |

## A lei (ramo `Pigmento`)

```
c   = grid::tinta::em(lg, i)
o   = optica_celula(&c.filme, &c.aberto, s_fator(c.m, film[i] > 0))
(cor_lin, α) = ecra_sobre_branco(&o, &Bandas(c.r_g))         // «exacto sobre branco»
cor = linear_para_srgb(cor_lin)·255 ;  la = la_comp = α·opacity
α == 0 ⇒ continue (célula transparente)
```

Somem só no ramo `Pigmento`: `alpha_of_mass`, `Ecra` (`com_vela/vidrado/oleo/opaca`), `cor_e_alfa`,
`na_caixa_das_tintas` (R8) e a Transparency de composição (R7/R20: a transparência é o `S` do pigmento). Ficam: o
papel/emboss somado à cor (ainda lê a MASSA — decisão 1 do coordenador: `susp/sett` seguem em massa), o Porter-Duff
entre camadas, o Óleo Real e o `PreLuz`.

## Portas sem `Params`

`oleo_borda::albedo_da_celula` e `oleo_campo` não recebem `Params`: lá o interruptor é **a grade ter o campo**
`Grid::tinta` (só existe depois de um passo com `Pigmento`, `grid::tinta::garante`). Consequência: uma grade pintada
com `Pigmento` e depois vista com `Velha` tem a orla do Óleo Real pela lei nova e o miolo pela velha. Aceite até à
E12 (quando a lei velha sai); a E8/E6a pode passar `LeiDaTinta` ao `CampoAltura` se o smoke o pedir.

Também: com `p.tinta == Pigmento`, uma camada pintada só com a lei velha (campo vazio ⇒ `NEUTRA`) não se vê. É a
semântica do interruptor (as duas leis não se misturam na mesma tela).

## Testes

| teste | antes | depois |
|---|---|---|
| `render::tinta::tests::o_ecra_da_celula_e_a_lei_da_tinta_ao_bit` (5 células: opaca, só aberto transparente, filme+aberto sobre `R_g` cinza, molhada, `R_g` azul; massas arbitrárias) | não existia (sem a porta o pixel saía da massa) | verde: `ecra_da_celula` == `ecra_sobre_branco`→sRGB **ao bit** (f64), pixel == `clamp_u8` disso |
| `…::celula_vazia_e_transparente` (massa 3000 sem tinta; grade sem o campo) | — | verde: `[0,0,0,0]` |
| `…::sobre_branco_o_pixel_composto_devolve_r_w` (RGBA u8 composto sobre branco em linear vs `camada(o, R_g)` em sRGB) | — | verde, erro ≤ 1/255 |

Lei velha ao bit (corridos por nome, todos verdes): `o_interruptor_da_tinta_nao_muda_um_pixel_com_a_lei_velha`,
`a_transparencia_zero_e_a_imagem_de_antes`, `a_transparencia_mistura_pela_massa_e_e_so_da_celula`,
`o_clear_glass_e_a_lei_de_antes_ao_bit`, `a_transmissao_e_o_relevo_do_papel_a_zero_sao_a_imagem_de_antes`,
`o_relevo_nao_sai_do_pigmento`, `a_fenda_neutra_e_bit_identica_a_de_antes`, `oleo_borda_w3b_tests` (5 + 6 ignorados),
`the_region_render_is_the_full_render_where_it_is_asked`, e em `ph2d-tool-painter`
`o_vidrado_da_celula_e_a_lei_de_velatura_ao_bit` (este com `Some(&Params)`, o produto). ⚠️ Os de `ph2d-wet-paint`
passam `p = None`; o ramo `Velha` é o mesmo código com ou sem `Params`. O portão das miniaturas (hash congelado) não
foi corrido aqui — fica para o fecho.

## Em aberto

- **R21 (dois pipelines óleo/sem, `w_oleo`)**: NÃO mudou nesta etapa (ordem do coordenador). O plano §3.3 pede
  `w = 1` com `Pigmento` e `oleo_real_plano_e_a_identidade` (a luz sobre `h ≈ 0` é a identidade do albedo, ao bit) —
  não escrito; vai com a etapa que unificar o pipeline.
- **Altura** (§3.3): hoje `sett + susp` (massa) + `altura_fechada_em_massa = 0`. E8 troca o `0` por
  `altura_fechada · M_UM` (`M_UM` da E6a, `wp/grid/tinta.rs`).
- **Emboss/papel** (`render.rs` bloco `paper_on || emboss_k`) ainda soma cinza à cor e lê a massa — área Superfície.
- Papel de cor: a regra «exacto sobre branco» sobre papel não-branco (§9 do plano) não foi medida.
- `cobertura_da_celula` (`oleo_borda.rs`) e os pesos de material/brilho em `oleo_campo.rs` seguem a `alpha_of_mass`
  da massa (coerente com a decisão 1; mudar se a E6a calibrar `M_UM` e o smoke mostrar a orla desalinhada da cor).
