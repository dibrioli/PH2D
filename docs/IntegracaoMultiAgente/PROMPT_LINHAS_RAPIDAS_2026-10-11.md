# PROMPT — tornar a linha RÁPIDA (11/10/2026)

> **Para:** a janela que assume `line/motion-value`, `line/Vector`, `line/MiroClone`, `line/Luz2D` ou
> `line/PainterWatercolor`.
> **Leia:** §1 + §2 + **só a sua** secção do §3. Não leia as outras.
> **Esta é a 1.ª onda da janela, ANTES de continuar a fila da linha.** A fila espera.
> Medido sobre os registos de 74 janelas (02→11/10) + corridas A/B no `main` `8f20c60de` (11/10).

## §1 — O diagnóstico (comum às cinco)

O que pesa **não** é a compilação: remontar após uma edição custa **1,2–6,1 s** (medido nas cinco
crates principais) e a mediana dos `Finished` nos registos é **1,2–2,8 s**. Por ordem de peso:

| | Motion | Vector | Miro | Luz2D | Painter |
|---|---|---|---|---|---|
| tempo do agente a GERAR | 34,2 h | 14,2 h | 23,0 h | 12,6 h | 12,8 h |
| tempo à espera de ferramentas | 18,6 h | 5,7 h | 6,8 h | 5,9 h | 10,6 h |
| passos por janela (mediana · máx) | 250 · 439 | 270 · 491 | 274 · 589 | 208 · 554 | 249 · 439 |
| contexto médio por passo (mediana das janelas) | 365 k | 396 k | 416 k | 441 k | 422 k |
| edições por script `python`/`sed` × pela ferramenta `Edit` | 1 545 × 787 | 757 × 287 | 1 306 × 330 | 684 × 145 | 660 × 237 |
| corridas de teste na janela principal × em subagentes | 997 × 404 | 434 × 428 | 692 × 231 | 530 × 92 | 361 × 67 |

1. **Janelas longas demais.** O mesmo passo custa **4,2–5,2 s** (mediana) com contexto abaixo de 200 k
   e **8–11,6 s** acima de 600 k. Uma janela de 400 passos paga a 2.ª metade ao dobro do preço.
2. **Edição por script.** 33–46 % de TUDO o que o agente escreve são scripts `python3 - <<EOF` com
   `.replace()`; no Miro, mais 24 % são `.rs` inteiros reescritos por `Write`. Cada script repete o
   texto antigo E o novo, mais o andaime, e costuma levar o teste colado — um «editar» de 10 min.
3. **Poucos testes enormes, corridos dezenas de vezes** (§3 de cada linha). Um só teste faz 94 % da
   suíte do Motion; no esqueleto, um passa do tecto de 180 s; no Painter, a simulação da tinta corre
   sem otimização (6–7× mais lenta) e é a cauda do portão geral de TODAS as linhas.
4. **Esperas que não esperam por nada.** `until grep …; sleep` à frente: 98 (Motion), 47 (Vector),
   32 (Miro), 12 (Luz2D), 52 (Painter); vários batem no limite de 10 min da ferramenta. O harness já
   AVISA quando um comando de fundo acaba.
5. **Docs gigantes relidos.** `121_as_formas_na_placa.md` 248 KB, 45 leituras · `Skeleton/01_a_fila.md`
   329 KB, 37 · `05_plano_da_linha_e_o_antes.md` 131 KB, 35 · `BUGS_motion_nodes.md` 97 KB ·
   `BUGS_painter.md` 94 KB · `110_ciclo_6_valor_e_pulso.md` 75 KB.
6. Só **7–12 %** dos passos fazem mais de uma chamada.

## §2 — As regras (valem para TODAS as janelas da linha daqui em diante)

> Copie este §2 (só as regras R1–R9) para o topo de cada handoff de continuação que escrever.

- **R1 — Janela curta.** Uma onda = uma janela. Passou de **~150 chamadas de ferramenta** ou fechou a
  onda: commit, handoff de continuação **≤ 8 KB**, PARE. A próxima janela lê o handoff e o §5 do
  CLAUDE.md — não o histórico da linha.
- **R2 — Edite pela ferramenta `Edit`.** Arquivo novo = `Write`; arquivo existente = `Edit`, nunca
  `Write` por cima. Script só para a MESMA troca em N arquivos, com `assert` de contagem.
- **R3 — Edição e teste nunca no mesmo comando.** Edite; `bash scripts/cargo-check-narrow.sh <crate>`;
  só então teste.
- **R4 — Teste dirigido por NOME no laço.** `bash scripts/cargo-test-narrow.sh <crate> <filtro>`.
  A suíte inteira da crate e o `nextest-impacted.sh` correm **1× no fecho, pelo `verificador`**.
- **R5 — Nada de `until … sleep` à frente.** Comando > 2 min: `run_in_background`, faça trabalho
  independente e espere pelo aviso. Sem trabalho independente: corra à frente com `timeout` à medida.
  Nunca espere «a máquina acalmar» ([MEDIR_VELOCIDADE.md](../DevOps/MEDIR_VELOCIDADE.md)).
- **R6 — Chamadas independentes na MESMA mensagem** (2 leituras, 2 greps, check + leitura).
- **R7 — Doc > 40 KB nunca se lê inteiro:** `grep -n` e leia só o trecho (`Read` com `offset`/`limit`).
- **R8 — Escreva menos.** Comentário de código: a lei em 1–3 linhas; o porquê vai para o doc. Resposta
  ao Enio: curta (CLAUDE.md §0.8). Prompt de subagente: o objetivo, os arquivos, o formato da resposta.
- **R9 — Tecto por teste: nenhum teste da suíte normal passa de 10 s em `dev`.** Recurso: o relógio do
  laço interno e a cauda do portão (o `nextest` termina quando termina o teste MAIS lento). Cura, por
  ordem: (a) `opt-level = 2` para a crate quente (§2.1); (b) fixture menor que mantém a lei — prove com
  mutação que o teste continua a morder; (c) só o que NÃO afirma nada (`diag_*`, `sonda_*`,
  `measure_*`) vai para `#[ignore]`. ⛔ Lei afirmada nunca vai para `#[ignore]` só por ser lenta —
  isso é silenciar.

### §2.1 — Como medir e ligar a otimização de uma crate quente

Meça ANTES, sem tocar em nada, com um diretório de build próprio na sua scratchpad. ⛔ Nunca o
`target/` de outra worktree:

```bash
T=<scratchpad>/ab; F='test(=<caminho::do::teste>)'
CARGO_TARGET_DIR=$T/o0 bash scripts/ph2d-run.sh cargo nextest run -p <crate> --run-ignored all -E "$F"
CARGO_TARGET_DIR=$T/o2 bash scripts/ph2d-run.sh cargo nextest run -p <crate> --run-ignored all -E "$F" \
  --config 'profile.dev.package.<crate>.opt-level=2'   # + uma --config por crate de que o laço depende
```

Intercale o0/o2 duas vezes e fique com o **mínimo**. Meça também o custo de remontar depois de 1 edição
nas duas variantes. Se ganhar ≥ 2× e remontar não piorar mais de 1 s, acrescente o bloco ao `Cargo.toml`
da raiz, **no fim da lista `[profile.dev.package.*]` (só acrescentar — ADR-0107)**, com a tabela da
medição num comentário de 3 linhas, como os blocos vizinhos. Depois, `rm -rf $T`.

## §3 — A tarefa de cada linha

**Passos comuns:** (1) `bash scripts/agent-loop-profile.sh 10` → guarde os números de ANTES;
(2) `cargo nextest run -p <crate> --status-level pass` → os 10 testes mais lentos; (3) corrija pelo R9;
(4) meça de novo; (5) handoff com a tabela antes × depois e o §2 no topo; **(6) não faça push.**
O que a medição de 11/10 já encontrou está abaixo. Confirme na sua worktree (a sua base pode ser
outra) antes de agir.

### §3.Motion — `line/motion-value`

- `motion_tecto_de_instancias::nenhum_no_emite_mais_do_que_o_tecto`: **113 s**, de uma suíte de 122 s
  (1 461 testes). Com `profile.dev.opt-level=2`: **12,6 s (9×)**, e remontar após 1 edição ficou igual
  (3,8 → 3,7 s). Descubra QUAL crate leva o tempo (`ph2d-app-motion` sozinha? os nós/`ph2d-eval-motion`?)
  e ligue só essa(s) pelo §2.1. Depois, o R9 sobre o resto: `pilha_demo` 11,7 s · `carimbo_demo` 11 s ·
  `nenhum_criador_passa_o_tecto_com_a_contagem_no_extremo` (lento nos registos).
- Parta com `python3 scripts/doc-split.py`: `121_as_formas_na_placa.md` (248 KB), `BUGS_motion_nodes.md`
  (97 KB), `110_ciclo_6_valor_e_pulso.md` (75 KB). Alvo ≤ 40 KB por pedaço + um índice.
- Hábitos que pesam mais aqui: 98 `until/sleep`, 1 545 edições por script, 997 corridas de teste na
  janela principal.

### §3.Vector — `line/Vector` (worktree parada desde 07/10; faça o rebase sobre o `main` antes de medir)

- Suíte de `ph2d-skeleton-live`: **190 s** para 304 testes. A otimização ajuda pouco aqui
  (73 → 51 s, 1,4×): **o defeito é o desenho dos testes.**
  - `skin_live::carrega::tests::uma_forma_ja_cozida_com_pilha_nova_coze_outra_vez` — **TIMEOUT 180 s**
    sob carga e `SLOW > 60 s` no portão geral das OUTRAS linhas. Ache o laço que domina e encolha a
    fixture (R9.b).
  - `skinned_mesh::rive_sondas_tests::diag_d_o_preco_do_segundo_corpo` 132 s ·
    `skinned_mesh::tests::diag_o_desenho_com_e_sem_campo_na_configuracao_do_produto` 66 s ·
    `diag_o_campo_e_invariante_a_escala` 58 s → `diag_*` correm na suíte normal: se não afirmam nada,
    `#[ignore]` (R9.c); se afirmam, R9.b.
  - `skin_desenho::…::densidade::o_braco_nao_tem_dentes_a_densidade_nenhuma` 105 s ·
    `skin_live::carrega::tests::cozer_no_carregamento_nao_move_um_pixel` 75 s → R9.b.
- Parta `docs/Skeleton/01_a_fila.md` (**329 KB**, 37 leituras) com `doc-split.py`: a fila aberta num
  ficheiro curto, o fechado no arquivo.
- Houve uma espera de 10 min por «máquina calma» — proibida (R5).

### §3.Miro — `line/MiroClone`

- Os testes do quadro são rápidos (`ph2d-board-render`: 25 testes em 3 s). **O lento aqui é o agente:**
  janelas até 589 passos, contexto até 967 k, 1 306 edições por script, `.rs` inteiros por `Write`
  (1,6 M caracteres) e 119 subagentes (8,3 h a gerar). R1 + R2 são o grosso do ganho.
- `ph2d-editor-core` é a crate mais cara de remontar (**6,1 s**) e foi testada 146 vezes. Dentro dela,
  `screens::hero::tool_bar::tests::nenhuma_face_de_pulldown_da_coluna_e_cortada` leva **39 s** de 45 s.
  A crate é partilhada: meça e cure pelo R9; se o teste for de outra linha, registe-o no handoff e não
  mexa nele.
- `nextest-impacted.sh` correu 71 vezes (58 s de média): R4, só no fecho.
- `02_plano.md` (38 KB) foi lido 38 vezes: leia pelo `grep -n` (R7).

### §3.Luz2D — `line/Luz2D`

- `ph2d-app-light2d::it`: 4 testes de **72–77 s** e 2 de 40 s, num total de 88 s.
  `a_luz_nao_muda_com_a_vista::a_luz_num_ponto_nao_muda_com_a_vista` com `opt-level=2`: **72 → 17 s
  (4,2×)**; remontar 1,2 → 1,6 s. Ligue as crates quentes (`ph2d-light2d-ref`? `-ecs`?) pelo §2.1;
  depois o R9 sobre `o_interior_do_opaco_nao_depende_da_posicao_na_grade`,
  `…_com_os_sprites_por_texel`, `o_oclusor_fino_faz_a_sombra_do_gabarito`.
- ⚠️ No `main` `8f20c60de`, `o_oclusor_fino::o_oclusor_fino_faz_a_sombra_do_gabarito` **REPROVA**
  («F2: a barra de 5 cm não faz a sombra do gabarito»). Veja se a sua linha já o cura; se não, registe-o
  no handoff.
- A pior latência por passo das cinco: **11,6 s** com 400–600 k de contexto. R1 é o 1.º ganho.
  46 % do que a linha escreve são edições por script (684 × 145 `Edit`).
- 194 min de corridas em `--profile smoke` (medições GPU): uma medição = uma corrida intercalada
  ([MEDIR_VELOCIDADE.md](../DevOps/MEDIR_VELOCIDADE.md)), nunca repetida para «confirmar».

### §3.Painter — `line/PainterWatercolor`

- **1.º e maior ganho, que serve às cinco linhas:** `ph2d-wet-paint` e `ph2d-pigment` compilam SEM
  otimização em `dev` (o `ph2d-tool-painter` já tem `opt-level = 2`; as duas ficaram de fora ao sair
  dele). A/B nos testes de `ph2d-tool-painter --lib`, só com essas duas a `opt-level=2`:

  | teste | hoje | otimizado |
  |---|---|---|
  | `opcoes_tests::a_heavy_edge_a_meio_pesa_na_orla_de_baixo_como_no_rebelle` | 131 s | 20 s |
  | `mistura_camadas_tests::o_traco_nao_muda_de_tom_ao_secar` | 59 s | 10 s |
  | `achatamento_tests::o_escorrimento_do_s_nao_se_corta_em_rectangulos` | 173 s | 26 s |

  Compilar a frio não mudou (1 min 16 s × 1 min 17 s). Confirme pelo §2.1 e acrescente os dois blocos.
  São esses testes que fecham o portão geral das OUTRAS linhas:
  `mistura_camadas_tests::n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto` deu
  **`TIMEOUT 180 s`** · `mistura_wet_drip_tests::uma_mistura_nunca_e_mais_clara_que_as_tintas` 65 s ·
  `mistura_wet_drip_tests::sem_celulas_soltas` 40 s.
- Depois, o R9 sobre o que ficar > 10 s (os três da tabela continuam acima de 10 s mesmo otimizados).
  O commit `8b84cfdb8` já mexeu nos portões de cena (uma cena por thread): meça sobre a SUA base.
- **Os dois portões de LEI da fábrica que a janela de 11/10 criou** (commits `7a7fb3c4d` · `8a9f13ed7`;
  sem `#[ignore]`, na suíte normal) passam do tecto do R9:
  - `tests/it/presets_de_fabrica.rs::a_largura_e_a_prometida` — **80–100 s** (12 presets × 8 raios de
    2 a 300, cada traço com 480 quadros de água; o raio 300 pinta numa tela de 1000 × 1880).
  - `tests/it/midias_de_fabrica.rs::cada_preset_parece_a_sua_midia` — **~60 s** (8 mídias × 2 cenas
    1000 × 640 com 1 800 quadros: o preset e o controlo Wet Round).
  Cura pelo R9: (a) primeiro a otimização acima — os dois pintam pelo `ph2d-wet-paint`; (b) fixture
  menor que ainda morda (provar com as mutações já registadas no handoff
  `HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-11_PRESETS_MIDIAS.md` §5.1: M1 e M8 têm de
  continuar VERMELHAS) — p.ex. o controlo pintado UMA vez por largura em vez de uma vez por mídia, o
  raio 300 numa faixa só da altura do traço, menos quadros onde a tinta já está seca. ⛔ Nenhum dos dois
  vai para `#[ignore]` (são lei).
- O gerador da fábrica (`presets_de_fabrica::gera_os_presets_de_fabrica`, `#[ignore]`) leva **12–15 min**
  para os 12 presets e corre a cada mudança de lei do motor: os presets correm em SÉRIE (só os raios de
  cada um em paralelo). Meça pôr os presets em paralelo também (o tecto da porta é 50 % da máquina).
- A grelha `PONTOS_DA_LEI` subiu 34 → 64 → 128 por `sed`, com corridas de **10 min cada**, e ficou em
  **64** (128 errou o mesmo: a lei ondula de raio para raio). Não a mexa sem medição UMA vez
  (MEDIR_VELOCIDADE): mais pontos só se apanharem um defeito que menos pontos deixam passar.
- **Depois de ligar a otimização, corra os portões que fixam pixels ao byte** (as miniaturas da fábrica
  `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, `opcoes_tests::com_as_opcoes_desligadas…`,
  `grid_ratio_tests::smooth_edges_no_zero…`, `os_outros_meios_nao_mudam`): a otimização não deve mudar
  um bit (o Rust não funde operações de vírgula flutuante), mas é ela que o prova. Se algum mudar, PARE
  e diga-o no handoff antes de regravar.
- Parta `NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md` (**136 KB** depois de 11/10, 35 leituras)
  e `BUGS_painter.md` (94 KB) com `doc-split.py`. ⚠️ **O `05_plano_da_linha_e_o_antes.md` continua a
  existir, como ÍNDICE** com os títulos e os links para os pedaços: o prompt 17, o
  `18_mapa_das_midias.md`, os handoffs e o CLAUDE.md apontam para «05 §4b», e um link partido perde a
  história da linha.
- Hábitos que pesam mais aqui: edição + teste no MESMO comando (R3; vários «editar» de 10 min),
  52 `until/sleep`, 660 edições por script.

## §4 — Fora das linhas (não faça; é do `main`)

- `project-memory/MEMORY.md` no `main` está com 22 164 bytes (alvo 22 000) numa alteração por commitar:
  `o_indice_cabe_no_orcamento_do_carregador` reprova no `main`.
