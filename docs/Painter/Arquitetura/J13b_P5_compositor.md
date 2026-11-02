# J13b P5 — o compositor da óptica (a maquinaria, sem o tique)

Passo P5 do [plano](J13b_plano_otica_fora.md) §2–§3, sobre o HEAD `637e68a9ad`. Ainda **inerte no produto**: o compositor só sobe com `PH2D_WET_OTICA_FORA=1`, e mesmo assim só trabalha quando alguém publica um pedido, coisa que só o P6 fará.

## O que mudou

- **`wetpaint/offthread_otica.rs`** (novo, módulo `offthread::otica`):
  - Tipos: `PedidoDeImagem { factos, epoca, divida, forma: FormaDaTela { ratio, grade, tela }, com_veu }` (:48).
  - Respostas: `RespostaDaOtica::{Nada { epoca, passos, marca }, Imagem(Box<Imagem>)}` (:93).
  - `Imagem { oleo, marca, epoca, passos, caixas: Option<CaixasDaImagem> }` (:82).
  - `CaixasDaImagem { caixa_cel, caixa_px, pigmento, sombra: Option, grao: Option, sem_oleo, veu: Option<VeuDaCaixa> }` (:69). Os planos levam só a caixa; `None` quer dizer que o plano ficou vazio.
  - `VeuDaCaixa { linha0, s, film, wet }`: as linhas da grade activa que a caixa de píxeis toca, com uma linha a mais de cada lado (:61).
  - `Imagem::aplica(&mut PlanosDaOtica, grade)` (:105) escreve o mesmo que `compoe_a_otica` teria escrito.
  - Portas na sessão: `WetSession::pede_imagem` (:428) e `try_recv_imagem` (:439).
  - Do lado do worker: `DoWorker::serve` (:253). Fio do compositor: `compositor` (:318) e `compoe` (:342), que corre dentro de `no_pool`/`rayon::join` (:332).
- **`offthread.rs`**: `mod otica` (:43) e o campo `SimWorker::otica` (:121).
- **`offthread_laco.rs`**:
  - `spawn` sobe o compositor só ao relógio e com `otica::ligada()` (:32).
  - `worker_loop` recebe `Option<DoWorker>` e chama `serve` no ramo ocioso (:119) e depois de um passo fechado (:144).
- **`wet_diag.rs`**:
  - `note_copia` (:83), `note_otica` (:88) e `take_otica() -> (cópia, óptica)` (:94). As imagens por segundo saem de `óptica.n ÷ janela`.
  - O interruptor: `otica_fora()` (:114) lê o ambiente uma vez; `set_otica_fora()` (:126) muda-o em execução, para as variantes intercaladas.

## Protocolo

O protocolo de `serve`:
1. Corre só com o passo fechado.
2. Exige o espelho na mão, recuperado por `try_recv` do canal de volta.
3. A regra da folga: `!sim_should_run() || acc + copia_ms_ema ≤ STEP_S`.
4. Junta a `divida` com `merge_dirty_otica`. A junção alarga uma célula, o que é conservador.
5. Faz `take_dirty`. Se vier `Clean`, responde `Nada`.
6. Senão chama `copia_para` (as linhas do `espelho_sujo`) e envia ao compositor `EntradaDaOtica` mais a época, os passos e a camada activa, se for pedido o véu.

O crescimento por `alcance_na` acontece dentro de `compoe_a_otica`. O worker não cresce a caixa outra vez: crescer duas vezes quebrava a igualdade ao byte.

O compositor trabalha com planos próprios e põe `campo_cache = None` depois de cada imagem. Devolve primeiro o espelho e só depois envia a imagem.

O ciclo de vida:
- O `Drop` do `SimWorker` arma `want`, o worker sai e fecha o canal de trabalho, e o compositor acaba no `recv`.
- O ritmado, o roteiro e o `wet_step_sync` nunca chamam `serve`.

## Testes (por nome, `--profile smoke`, load ~11)

| teste | resultado |
|---|---|
| `a_imagem_do_compositor_e_a_otica_sincrona_ao_byte` (4 rondas, thread real: sujo inteiro · 8 dabs com cópia incremental · só dívida · limpo → «nada») | verde |
| mutação: sem juntar a `divida` | vermelho (resposta errada) |
| mutação: o espelho copiado só a 1.ª vez | vermelho (pigmento difere) |
| `a_finished_session_does_not_leave_the_compositor_running` | verde |
| `sem_o_interruptor_nao_ha_compositor` (inclui o ritmado com o interruptor ligado) | verde |
| `leak` (4) · `offthread` (13, 4 ignorados) · `por_quadro` (16, 3 ignorados) · `ritmo` (1) | verde |

## Aberto para o P6/P7

- **P6** liga o tique em `ciclo.rs`:
  - publica com `pede_imagem`; recebe com `try_recv_imagem` e aplica com `aplica`;
  - trata `uniao_local`/`divida`, `seen_steps ← passos`, `oleo_na_tela ← oleo.is_some()` e `composta ← marca`;
  - com `Nada`, só a marca;
  - confere `base.len() == w*h*4` antes de pedir;
  - `veu.rs` passa a ler `VeuDaCaixa`, com o índice `i − linha0·s`;
  - remove os `expect(dead_code)` de `offthread_otica.rs:14` e `offthread.rs:120`.
- **P7**:
  - imprime `take_otica()` no relatório do shell (`shells/desktop/src/render_loop/fase_frame_profile_report.rs`), fora deste passo;
  - mede o `passo ms` com o compositor ocupado;
  - mede o custo de `CacheCampo::guarda` no compositor, que guarda e logo deita fora.
