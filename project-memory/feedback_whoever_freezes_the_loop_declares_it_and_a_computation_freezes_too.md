---
name: feedback-whoever-freezes-the-loop-declares-it-and-a-computation-freezes-too
description: Declarar o congelamento cura a MENSAGEM; não cura o congelamento — e o compositor é o segundo observador, que declara a janela morta.
metadata:
  type: feedback
---

Dois reports do Enio no MESMO dia (2026-08-25), sobre a mesma exportação, e são **duas camadas**:

| report | quem observa | mecanismo | cura |
|---|---|---|---|
| *"a mensagem não aparece"* | o relógio do chrome | o quadro seguinte cobra o congelamento ao TTL do toast, que morre num piscar | **declarar** quanto congelou (`crate::modal::stalling` / `note_stall`) |
| *"o linux fica cinza"* | o **compositor** | com 12 s sem responder ao *ping*, o KDE pinta a janela de cinza e oferece *"forçar o encerramento"* | ⭐ **não bloquear** — o trabalho sai da thread que desenha |

⛔ **A primeira cura não alcança a segunda.** *O relógio do chrome não distingue «parado por um
diálogo» de «parado por uma conta»* — mas o **sistema operativo** distingue: um diálogo é uma
janela que ele sabe que abriu; uma conta é o programa a não responder. Declarar salva o toast e
deixa o utilizador a olhar para uma janela que o SO diz estar morta — e o gesto natural a seguir é
matá-la, com o trabalho não gravado dentro.

**Why:** *ser rápido o suficiente é um número que outra pessoa muda.* A exportação foi de 8 min 17 s
para 6,4 s e o cinza continuou (12 s na peça dele); a fronteira do compositor não é nossa.

**How to apply:**
- **Diálogo modal** → a porta que declara (`crate::modal::save_file` / `pick_file`). Continua certa.
- **Cálculo** que possa passar de um quadro → **bancada fora da thread** (o molde é
  `shells/desktop/src/field3d_export_job.rs`): `spawn(FnOnce() -> String + Send + 'static)`, resposta
  drenada uma vez por quadro, uma de cada vez, recusa do segundo **em alto**, e um sentinela que
  liberta a bancada no `Drop` para que um `panic!` não a tranque até ao fim da sessão.
- ⚠️ **Declarar do lado de lá é um no-op silencioso**: `note_stall` escreve num `thread_local` que
  o quadro nunca lê. Ao mover o trabalho, **retire** a declaração em vez de a deixar a mentir
  ([[feedback-a-parameter-that-changes-nothing-is-discarded-downstream]]).
- ⚠️ **O silêncio com o app vivo lê-se como «o botão não fez nada»** — depois de mover, um aviso de
  início passa a ser obrigatório, e ele **não promete prazo** (depende da peça).
- ⚠️ A régua do gate é a **identidade da thread**, nunca o relógio: *"a chamada volta depressa"*
  passa verde numa máquina rápida com o trabalho na thread errada.

⛔ Medido 2026-08-22: há **25** chamadas de `rfd::FileDialog` em **12** arquivos do shell e só as
`field3d_*` passam pela porta — as outras 23 continuam a perder a mensagem que escrevem a seguir.
