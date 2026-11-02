# J13b P2 — o espelho da óptica no motor (02/11)

Plano: [`J13b_plano_otica_fora.md`](J13b_plano_otica_fora.md) §2 e §4.4. Só a crate `ph2d-wet-paint`.

## O que mudou

- `wp/painter.rs:149`: o campo `Engine::espelho_sujo: Dirty` nasce `Full` (`:198`).
  - `merge_dirty` faz a união (`:620`). `mark_dirty_full` põe `Full` (`:637`). `merge_dirty_otica` não lhe toca.
  - A união da caixa passou a ser `Dirty::une` (`wp/painter/espelho.rs:73`): `merge_dirty_otica` (`:630`) e o espelho usam a mesma conta. É a mesma do `if` antigo, agora com `min`/`max`.
- `wp/painter/instantaneo.rs:42,79`: o clone do motor nasce com `espelho_sujo = Full`, porque o espelho da cópia é outro.
- `wp/painter/espelho.rs` (novo):
  - `Espelho { camadas: Vec<CamadaEspelho { grid, opacity, visible }> }`.
  - `camadas_render()`: o render existente lê o espelho sem mudar nada.
  - `bytes()`.
  - `Engine::take_espelho_sujo` (`:98`).
  - `Engine::copia_para` (`:104`).
  - `so_a_otica` (`:195`): o padrão sem `..`, para que um campo novo da `Grid` não compile sem decisão.
- `copia_para` decide assim:
  - Outro nº de camadas ou outro tamanho: reconstrói tudo.
  - `Full`: copia todos os planos no lugar.
  - `Rect`: copia as linhas `y0−1..=y1+1` e o ANEL de `apply_boundaries`.
  - Um plano com outro comprimento (alocado ou largado preguiçosamente) copia-se inteiro.
  - A cópia é em série: `Cargo.toml` diz que qualquer uso novo de rayon nesta crate exige ADR.

## Planos copiados (os lidos)

`film susp susp_rgb sett sett_rgb relevo altura espessura crista nivelado vela material aparencia oleo tinta pigmento wet paper`, mais os escalares (`w h s rows cells oleo_real flow bbox has_fluid spans_enabled paper_*`).

Ficam vazios: `oleo_antes*`, `chao`, `sett_do_traco`, `vel_*`, `flow_*`, `frow_*`, `active`, `bloom`, `row_*`, `live_*`, `scratch`.

Leitores, por grep:

| ficheiro | lê |
|---|---|
| `render.rs` | `altura aparencia oleo relevo sett sett_rgb susp susp_rgb vela paper` |
| `oleo_campo.rs` | `altura crista material oleo paper sett susp`, mais `espessura::factor` |
| `oleo_borda.rs` | `nivelado sett* susp* tinta vela` |
| `render/tinta.rs` | `film material pigmento sett susp tinta` |
| `veu.rs` | `film wet s` |
| `composite_oleo.rs` | `oleo_real sett susp s` |

A prova é de execução: o gate compara byte a byte o render do Real Oil (`render_pigment_row_oleo` com `CampoAltura::de`, papel e relevo) sobre o espelho e sobre as grades, a cada passo. Nenhum plano vazio é lido.

## Escritas fora das portas (listadas, não consertadas)

1. **`solver::apply_boundaries`** (`solver/project.rs:235`) reescreve a cada passo o anel `film/susp/wet` (linhas `0,1,h,h+1`, colunas `0,1,w,w+1`) e `drena_o_anel`, fora do bbox que o passo marca.
   - As linhas `1`/`h` e as colunas `1`/`w` são células VISÍVEIS: a tela de hoje também não recompõe o anel.
   - O espelho copia o anel em todo `Rect`, por nome. Sem isso fica vermelho (mutação 2).
2. **`*::veste_virgens`** (`trail/transfer.rs:438-440`, para aparência, óleo e material) escreve uma orla de 2 a 4 células para lá do `TouchedRect`. O sujo só cresce 1 célula.
   - Só toca células virgens (sem tinta), que não se desenham.
   - O espelho recebe-as quando a tinta lá chega, e isso passa por uma porta.
   - O gate aceita a diferença SÓ em células virgens, nesses três campos (máx. medido: 121 células). O render tem de ser igual ao byte.
3. **`Engine::seed_paper_with_rows`** (`painter/doors.rs:425`) escreve `paper` sem marcar sujo, e a `revisao` também não muda. A cache do campo pode servir papel velho.
4. **`Engine::set_lei_da_tinta`** (`painter/lei_da_tinta.rs:12`) esvazia `tinta`/`pigmento` sem marcar sujo. O espelho cobre isto pela regra do comprimento; o `dirty` da tela não.
5. `Engine::layers` e `active_grid_mut` são públicos: quem escreve por eles tem de marcar sujo. No `ph2d-tool-painter` de produção, só `authoring.rs:97` o faz, e marca (`:99`).

## Medida (`mede_copia_para_full`, `--ignored`, `--profile smoke`, load ~22: o relógio vale pouco)

| tamanho | reconstrói | `Full` no lugar (mín. 5) | B/célula guardados |
|---|---|---|---|
| 1024² | 41,6 ms | 6,4 ms | 157 |
| 4096² | 557,6 ms | 112,1 ms | 157 |

Na cena medida só estavam alocados `oleo` (28 B) e `tinta` (80 B). Com todos os planos alocados são 357 B/célula:
- f32: 10 × 4 B;
- rgb: 2 × 12 B;
- `wet`: 1 B;
- `material`: 52 B;
- `aparencia`: 20 B;
- `oleo`: 28 B;
- `tinta`: 80 B;
- `pigmento`: 112 B.

Isto dá cerca de 2,3× o tempo do `Full`, uma estimativa por proporção de bytes, não uma medida.

## Testes

| teste | antes | depois |
|---|---|---|
| `o_espelho_incremental_e_a_copia_total_ao_bit` | — | ✓ |
| `oleo` | 69 ✓ · 8 ign | 69 ✓ · 8 ign |
| `snapshot` | 1 ✓ | 1 ✓ |
| `dirty` | não casa nenhum (exit 4) | não casa nenhum (exit 4) |

Mutações (todas vermelhas, todas restauradas):
1. Tirar a união de `merge_dirty` dá vermelho no passo 0, em `sett`.
2. Tirar a cópia do anel dá vermelho no passo 2, em `wet`.
3. Tirar o `Full` de `mark_dirty_full` dá vermelho no passo 4, em `wet`.
