# N10 — família «carimbo e Impasto» (agente F7, 14/11)

Uma linha por id. A janela principal dobra-as no `BACKLOG.md`.

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B26.4 | parcial (memo) | medição por correr: `cargo test -p ph2d-painter-brush --profile smoke --lib mede_b26_4_a_lut_por_dab -- --ignored --nocapture` (série × paralelo, intercalados, mínimo de 41) | este commit | A chave do memo da LUT do filme (`LutKey`) leva os bits da `hardness`; com a dureza da mão (`Dynamics::dureza_da_mao`, `spec_frame.rs:98`) ela muda em TODO dab ⇒ o memo de uma entrada erra sempre e a `FilmLut::new` (16 385 × `film_of(falloff_weight)`) corre por dab. Cura ao bit: a tabela em `into_par_iter().map().collect()` (ordem do índice); e o `Mutex` do memo é LARGADO antes de construir (com o rayon por dentro, um operário que roubasse outro dab pediria o mesmo trinco no mesmo fio — Bug #66). Sem `with_min_len` (recusa medida N7). Gate `a_lut_em_paralelo_e_a_serie_ao_bit` (oráculo = o laço em série verbatim; 11 quedas × 8 durezas; controlo de tabelas distintas): verde; mutação `i ^ 1` no índice ⇒ VERMELHO (amostra 6464), restaurada. Nenhum preset de fábrica liga `pressure_hardness`/`speed_hardness` ⇒ o ganho é só de quem liga a dureza da mão no Impasto com Smooth Edges; alarmes de pincel não aplicáveis (ao bit). Por fazer: «caixa de altura = a da cor» e «bandas pela área pesada» (⛔ Q12) |
| B30.5 | registado (dono) | — | — | LUT no domínio remapeado por preset (deixaria de depender da dureza e acabaria com a reconstrução por dab) muda a aparência ≤ 0,06 nível ⇒ é do dono. Com B26.4 a reconstrução fica paralela, não some |
| B15.7 | aberto (medição) | — | — | Só medição (`sonda_luz_4k` com 4 luzes e Smoothing 1,0); a sonda é da pegada F6k e corre `--ignored` ⇒ fica para a fila |
| B25.7 | registado (não é desempenho) | — | — | `dab_footprint_px` (`dab_spec.rs:197`) lê `self.paint.brush` cru e não o `authored_spec()` (encaixe da macro, raio do motor do Wet): é a pergunta à MALHA da deformação (`painter_canvas_input.rs:187,320`), não um dirty rect ⇒ correção de aparência no canvas deformado, sem ganho 4K. Cura de uma linha (`let brush = self.authored_spec();`), a validar no smoke da malha |
| B25.5 | aberto | — | — | Plano de força das macros na rampa/Per-Layer: correção de gravação (macro tocada == gravada), não desempenho |
| B25.6 | aberto | — | — | `enum RotaDeCor` / uma porta de rota: refatoração sem ganho medido; não coube no prazo |
| B25.8 | aberto | contagem não feita | — | Rota da placa por dispositivo e contagem por preset: «talvez» muda aparência; não coube no prazo |
| B26.1 | parcial (como estava) | — | bf60d6c801 | Invariantes içadas já feitas; RLE da máscara não feito |
| B26.2 | aberto | — | — | Plano do lote + bandas por trabalho: grande (ADR-0200, fork-join por carimbo é o tecto); não coube |
| B26.3 | aberto | — | — | Piso do paralelismo por custo estimado: «ganho pequeno»; não coube |
