# Handoff — a vez da placa passa a ser de TODO uso (processo, 18/10)

> Trabalho de PROCESSO no `main` (como os `feat(processo 17/10)`), sem linha de módulo. Commits locais, sem
> push. A lei e o runbook: [A_VEZ_DA_PLACA.md](A_VEZ_DA_PLACA.md). O problema: Luz 2D, handoff W6c §2–3
> (`line/Luz2D`, ainda não integrado).

## 1. O que mudou

| onde | o quê |
|---|---|
| [`scripts/ph2d-run.sh`](../../scripts/ph2d-run.sh) | cada árvore ganha um PORTEIRO (subshell): pede o `ph2d-gpu.lock` no 1.º pedido da árvore e segura-o até o comando acabar; árvore dentro de árvore usa o de fora; `PH2D_GPU=1` = pedir já à partida (como antes); `PH2D_GPU_ESPERA` só por omissão com `PH2D_GPU=1` (300 s) |
| [`scripts/vez-da-placa.sh`](../../scripts/vez-da-placa.sh) (novo) | o cliente bash: para o que não é Rust (o kwin, um oráculo) |
| [`crates/ph2d-gpu/src/vez.rs`](../../crates/ph2d-gpu/src/vez.rs) (novo) | o cliente Rust `vez_da_placa()`, chamado pelo `GpuContext::new` antes do `request_adapter` |
| 8 testes de `ph2d-flip-render/tests/it` + `ph2d-gpu` (`pass_profiler_gpu`, o exemplo `o_que_a_placa_anuncia`) | pediam o adaptador por fora do `GpuContext`: chamam a vez à mão |
| portão `ph2d-gpu` `a_vez_da_placa::*` (novo) | o censo (todo `request_adapter(`/`enumerate_adapters(` do repo com a vez antes, na mesma `fn`; controlo de população) + 4 testes do protocolo contra um porteiro de mentira |
| [`fotografa_cena.sh`](../Components/ferramentas/fotografa_cena.sh) · [`mede_custo_no_app.sh`](../Luz2D/ferramentas/mede_custo_no_app.sh) | entram na porta sozinhos (a medição com `PH2D_GPU=1`) e pedem a vez antes do kwin |
| guarda [`tecto-de-recursos.sh`](../../.claude/hooks/tecto-de-recursos.sh) | **R9**: kwin/wine/Blender/Godot sem a vez, binário de `target/` cru; a R1 já não manda pôr `PH2D_GPU=1` em tudo |
| [`nextest-impacted.sh`](../../scripts/nextest-impacted.sh) | `PH2D_GPU_VEZ` (pasta nova a cada corrida) fora da chave do selo — dentro dela nenhum selo voltava a casar |
| docs | `TETOS` §1/§3/§5, `MODELO_ABERTURA` K1, `MODELO_TROCA`, `/pd-onda-fechar`, `CLAUDE.md` §2 (uma linha), duas memórias da placa |

## 2. Os números

Detalhe em [A_VEZ_DA_PLACA.md §3](A_VEZ_DA_PLACA.md). O essencial:

- a placa só «vê» o processo a partir do `request_adapter` (a `Instance` sozinha não aparece no `nvidia-smi`);
- o `flock` dá a vez por ordem de chegada: 20/20, e 20/20 a load 58;
- prova do porteiro 14/14 · guarda 101/101 casos · portão `a_vez_da_placa` 5/5 (mutação: tirar a vez do
  `GpuContext` ⇒ vermelho, restaurado);
- **a rodada real**: com a vez, 0 amostras dos disparos em 2 processos de medição, espera de 103–125 s, 48/48 ·
  48/48 · foto; controlo sem a vez: 45 amostras, acusado. 4 de 9 processos sujos por linhas que ainda não têm
  isto (§3.1) — o que só a integração cura.

## 3. ⛔ Recusas

[A_VEZ_DA_PLACA.md §4](A_VEZ_DA_PLACA.md).

## 4. O que cada linha tem de saber

1. **Nada a fazer para ter a vez**: dentro do `ph2d-run.sh` o teste, a sonda, o app e a foto pedem-na sozinhos. O
   `PH2D_GPU=1` fica só para MEDIR (a vez do princípio ao fim do comando).
2. **Esperar é normal**: «⏳ a placa está com outra árvore — à espera da vez» e depois «▸ a vez da placa chegou
   ao fim de N s». Não force, não mate quem segura; a ordem é a de chegada.
3. **Teste novo que pede o adaptador sem o `GpuContext`** chama `ph2d_gpu::vez_da_placa()` antes — o portão diz
   onde falta.
4. **Script novo que lança o kwin** (ou outro programa de placa que não é Rust): `bash scripts/vez-da-placa.sh ||
   exit $?` antes, e entre na porta (o bloco a copiar está no `fotografa_cena.sh`).
5. **Só vale depois de a linha ter este `main`** (o `ph2d-run.sh`, o guarda e o `ph2d-gpu` são da árvore de cada
   linha). Até lá é compatível: o `PH2D_GPU=1` velho tranca o mesmo cadeado.

## 5. Para a `line/Luz2D` (achado ao correr a régua)

⛔ O `regua_renasce/um_processo.sh` versionado (`04b2310408`) chama `bash "$AQUI/alheios.sh"` — esse ficheiro
**não existe** (só o `alheios.py`). O `bash` falha no stderr, o stdout sai vazio, e o detector lê **sempre
limpo**: a espera de placa livre passa logo e o `alheios.txt` nunca acusa ninguém. A rodada 2 da W6c deve ter
corrido com um `alheios.sh` local que não entrou no commit — confirme. A cópia desta prova trocou as três
chamadas por `python3 "$AQUI/alheios.py"`.

**Resolvido em 18/10 (ordem do dono):** confirmado que a W6c correu com um `alheios.sh` de rascunho
(`exec python3 -I alheios.py`) — o veredito dela vale. Cura na `line/Luz2D`, `fd4d1896a3`: o `um_processo.sh` chama o
`alheios.py` e RECUSA (exit 4) se o detector falhar — provado com um detector partido e com o são. A pausa de 60 s foi
medida contra ~6 s e FICA ([A_VEZ_DA_PLACA.md §3.1](A_VEZ_DA_PLACA.md)).

## 6. Para o integrador

Sem contrato congelado, sem schema. Um `pub mod vez` novo na `ph2d-gpu` (crate de que muitas dependem: o
portão dela é imprescindível na integração). Os 8 ficheiros de teste do Flip ganham uma linha cada (nenhuma
linha aberta os toca). O `docs/Luz2D/ferramentas/mede_custo_no_app.sh` era byte a byte igual ao da
`line/Luz2D` — o rebase dela passa limpo se ela não lhe tiver tocado depois. ⛔ `project-memory/MEMORY.md` e
`project_wacom_pen_needs_tablet_v2_on_kwin_wayland.md` estavam alterados por OUTRA sessão antes desta: não
entraram nos commits desta.
