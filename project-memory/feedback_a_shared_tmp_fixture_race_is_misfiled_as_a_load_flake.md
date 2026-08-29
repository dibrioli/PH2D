---
name: feedback_a_shared_tmp_fixture_race_is_misfiled_as_a_load_flake
description: Um teste que escreve um caminho FIXO em /tmp não é flake de carga — é corrida de fixtura, e a cura é outra
metadata:
  type: feedback
---

O `CLAUDE.md` §5.0 lista `only_the_lower_row_breathes_and_it_moves_with_the_playhead` (demos de
áudio, «max delta 0») como membro da **família de flakes de RECURSO sob fan-out** — aquela cujo
mecanismo é *um gate que mede um recurso partilhado (razão de dois relógios, contagem de
alocações) e reprova sob carga*.

**Ele não mede recurso nenhum.** O corpo coza o documento em `t=1.5` e `t=4.5` e afirma que as
barras mudaram: zero relógios, zero contadores. Medido na integração da `line/components`
(2026-08-27).

O mecanismo real é uma **corrida de fixtura partilhada**:
`motion_state_conferencia_demos_audio.rs::write_sweep()` escreve
`std::env::temp_dir().join("ph2d_audio_bands_sweep.wav")` — **caminho fixo, sem sufixo, no `/tmp`
partilhado** — com `std::fs::write`, que **trunca antes de escrever**. ⚠️ **Worktree isola o git,
não isola o `/tmp`**: N sessões em N worktrees correm o mesmo binário de teste e escrevem o
**mesmo** ficheiro. Quem lê durante a escrita alheia recebe um WAV truncado ⇒ bandas chapadas ⇒
`max delta 0`.

**Why:** a família prescreve a cura. Um gate de razão sob carga cura-se com folga (barra mais
larga, menos paralelismo, re-rodar na máquina calma) — e foi o que resolveu na prática
(`NEXTEST_TEST_THREADS=6` levou a suíte de 11376/11377 a **11377/11377**). Mas a folga só esconde
esta: o dia em que duas linhas coincidirem na janela de escrita, ela volta com qualquer barra.
A cura verdadeira é **um caminho por processo** (pid/uuid no nome), e é barata. *Arquivar um
defeito na família errada é prescrever-lhe o remédio errado e nunca mais o rever.*

**How to apply:** antes de acrescentar um teste à lista de flakes de fan-out do §5.0, pergunte
**o que ele mede**. Se não houver relógio nem contador no corpo, não é da família — procure
estado partilhado FORA da worktree (`/tmp`, `~/.ph2d/`, uma porta, um device). Um caminho fixo
em `temp_dir()` é o suspeito nº 1, e a assinatura dele é o valor **zero/vazio** (o leitor apanhou
o ficheiro truncado), não um valor ligeiramente fora da barra. Ver
[[feedback_a_flake_red_hides_the_rest_of_the_suite]] e
[[feedback_where_new_objects_are_born_is_the_fixture_your_gates_are_missing]].
