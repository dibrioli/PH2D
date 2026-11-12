# Handoff de integração — line/PainterWatercolor, a mistura de cor e o envio (12/11, n.º 3)

Leitor: o agente integrador. Os handoffs [DESEMPENHO_TOTAL](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md)
e [LUZ_E_CAMADAS](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md) **CONTINUAM a valer** (superfície, riscos,
decisões do dono, abertos). Aqui está só o que esta janela acrescentou: `af71e95bda..HEAD` (10 commits). **Nada foi integrado nem enviado.**

## 1. Identidade

- Branch `line/PainterWatercolor`; merge-base com o main `bc4a6ce930`. A linha **NÃO foi rebaseada** (é ordem do prompt n.º 3: o main
  andou ~594 commits de outras linhas).
- Tocado: `ph2d-painter-brush`, `ph2d-tool-painter`, `ph2d-app-painter` e `docs/`. **Ficaram intocados:** `shells/desktop`, os
  `Cargo.toml`/`Cargo.lock` e os contratos do §6.
- Commits (cada mensagem traz o mecanismo e a tabela de medição):
  - `38483595ac` Wet blend: a pegada e a película ao bit;
  - `731eb945c2` B28.4: as máscaras da tela soltas;
  - `a3b9a58681` Color mixing com Shape em paralelo;
  - `3a62b933f6` B20.6: o envio fundido;
  - `51ab488d9c` fmt/clippy;
  - `0fc4dabe3c`, `33e7a2bd88`, `a805a76cf7`: docs do BACKLOG.

## 2. Superfície de colisão

- **`ph2d-painter-brush` (o núcleo do dab de todo pincel Digital):**
  - `silhouette_capsula_com` ganhou o parâmetro `queda: QuedaDoDab`;
  - nova `silhouette_at_com`;
  - `Invariantes` ganhou os campos `queda` e `branco_no_vazio`;
  - `DabCtx` ganhou `vazio_local`;
  - `liso::escreve` ganhou um `bool`;
  - `pegada_textured_masked_po_com_liso` ganhou `com_liso: bool`;
  - nova `pegada_local` (pub, reexportada).

  Uma linha que mexa no kernel do dab (`dab.rs`, `dab/bands*`, `dab/liso.rs`) **colide**. A cura é mecânica: passar
  `spec.queda_do_dab()` e `false`/`vazio_local: false`.
- **`spec/queries.rs`:** `falloff_weight_passo` passa a delegar a `QuedaDoDab::peso` (mesma conta). Quem tocar no falloff edita o
  `QuedaDoDab`.
- **Mistura opaca:** `mistura_opaca/carimbo.rs` (a película em duas metades, `quanto`/`aplica`, e a partição por linhas) e
  `stamp_route/mistura*.rs` (Shape local, `pegada_com_forma`).
- **`ph2d-app-painter/painter_bridge_upload.rs`:** o `UploadPlan::Full` passa por `envio_fundido` com um buffer `thread_local`.
- Três chaves novas, só de A×B e gate (`#[doc(hidden)]`): `mistura_opaca::PELICULA_POR_LINHAS` e `PEGADAS_LOCAIS_COM_FORMA` (ambas
  `true`, que é o produto), mais `PH2D_SONDA_PELICULA` e `PH2D_SONDA_FORMA` na sonda.

## 3. Números que se CONTAM (DELTA)

- `PROJECT_SCHEMA` **+0** · ADR **+0** · registos 0 · pacotes 0 · formato gravado intocado.

## 4. O que entra (medido; régua = sonda do app no mesmo binário, A×B intercalado)

| item | antes | depois |
|---|---|---|
| Wet blend, Hard Round d200, 300 px/q, 2048² | 16,8–17,5 ms/q | 13,0–13,8 |
| Wet blend, Hard Round d84 | 13,7–13,9 | 9,4–9,8 |
| pegadas a 1 fio (55 ns/texel) | 114 ms/q | 73,5 |
| Oil Paint / Gouache (Color mixing com Shape), 360 px/q | 57–58 ms/q | 15–16 |
| Oil Paint / Gouache, 120 px/q | 19,6–21,4 | 5,9–6,2 |
| os 12 de fábrica com Color mixing, 360 px/q (load 7,8) | — | 3,7–15,4 (todos < 16,7) |
| envio inteiro à placa (parte da CPU), 4K | 11,58 ms | 1,79 |
| memória da rota local do Wet blend, 4K | 2 × 33 MB | 0 |

Curas, todas **ao bit**:

- **Queda do dab lida uma vez por dab** (`QuedaDoDab`): a dureza e a chave da área já não se recalculam a cada sub-dab de cada texel.
- **Pegada lisa só com grão:** a pegada local sem Grain, dente nem pó não escreve a lisa.
- **Branco sobre o vazio:** em Mix, o branco sobre o `[0;4]` da pegada local é `[1,1,1,a]` exacto (a prova cobre 2²⁴ valores de `a`).
- **Película por linha:** pinta-se uma LINHA por tarefa; antes eram 13 faixas para 32 fios.
- **Shape também em paralelo:** as pegadas com Shape passam a buffers locais, na união das caixas.
- **Máscaras soltas:** a rota local já não guarda as duas máscaras do tamanho da tela.
- **Envio fundido:** a cópia e a premul fundidas em faixas paralelas, num buffer reusado.

## 5. Prova de fecho (12/11)

- **Testes por nome:**

  | teste | mutação que o faz falhar |
  |---|---|
  | `o_branco_no_vazio_e_a_blend_de_sempre_ao_bit` | — |
  | `a_pelicula_por_linhas_da_a_tela_das_faixas_ao_bit` | → 111 805 bytes |
  | `a_pegada_local_com_forma_da_a_tela_da_rota_em_serie_ao_bit` | → 150 927 |
  | `a_pegada_local_sem_grao_da_a_tela_da_rota_de_antes_ao_bit` | → 7 172 |
  | `as_pegadas_locais_nao_guardam_a_mascara_da_tela` | → vermelho |
  | `envio_fundido_e_o_extract_mais_premul` | → vermelho |

  Também passam `o_corte_do_carimbo_pinta_os_mesmos_bytes_que_a_banda_antiga` e `a_pegada_na_caixa_e_a_da_tela_ao_bit`.
- **Alarmes de pincel:**
  - largura: só Oil Wash e Smear Wet vermelhos, como antes;
  - miniatura: os mesmos **74** vermelhos com o **mesmo fnv** da árvore antes desta janela (a árvore de base corrida no mesmo
    perfil). Ou seja, os 100 pincéis estão ao bit. Nada foi regravado.
- **fmt e clippy:** `PH2D_SUITE=fecho`, `-D warnings`, nas 3 crates: limpos.
- **Não corrido** (é do integrador ou do fecho da linha): a suíte completa, a matriz final do app (`mede_pen_up_4k.sh matriz`) e o
  binário smoke 2×.

## 6. Smoke do dono

Para o dono a mudança só se vê como velocidade (nenhum pincel muda de aparência):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`
2. Abra uma imagem grande e escolha o pincel **Oil Paint** ou **Gouache**. Faça traços rápidos e compridos.
3. Tem de acompanhar a caneta sem engasgar; antes, a tinta ficava para trás.
4. Errado: o traço chega aos saltos, ou a cor parece diferente da de ontem.

## 7. Decisões do dono

Nenhuma nova. Ficam as do handoff anterior: W5, os orçamentos e os vermelhos por decisão.

## 8. ABERTO, por impacto

1. **B20.11, as formigas da seleção.** A análise e a cura estão desenhadas no BACKLOG; falta implementar. Hoje o `Blob::new` gera um
   id novo por chamada, e por isso 33 MB sobem à placa por quadro. A cura: uma imagem com id estável em `ph2d-vector`, mais a fase da
   animação pelo tempo (passo de 100 ms).
2. **B13.6, o passo da sim** (20–22 ms/passo a 4K com 3 traços). Pesam o transport/outflow/prepare do `advect_jacobi` (27 %) e o
   flow (10 %).
   - O quadro da água no app já está coberto (B14.1 + a resposta `Nada`).
   - O B13.3 com ε > 0 foi **recusado**, medido: o sono custa mais do que salta.
3. Por atacar, da onda 5: **B19.10, B01.A1, B40.A3–A5**.
4. Por atacar, da onda 6: **B22.1, B22.4, B23.2**.
5. **B20.6, a parte da placa:** a subida (`replace_individual_pixels`) e o anel StagingBelt não foram medidos.

## 9. Riscos para o integrador

- O kernel do dab é de todo pincel Digital. Se outra linha mudou `dab/bands*` ou `spec/queries.rs`, a prova de que nada mudou é
  re-correr as miniaturas e comparar os fnv com a lista do fecho.
- **Medições:** a régua é o A×B no mesmo binário (as chaves `PH2D_SONDA_*`). A carga variou de 3 a 58 nesta janela, por isso o
  relógio absoluto vale pouco.
