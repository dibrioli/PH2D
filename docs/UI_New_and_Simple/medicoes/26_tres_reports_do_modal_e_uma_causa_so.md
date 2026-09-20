# 26 — Três reports sobre o modal, e uma causa só: a ORDEM do teclado

> **Report do dono, 2026-09-20**, sobre a paleta de pincéis que abriu nesse dia:
>
> 1. *«O modal não captura o que escrevo.»*
> 2. *«O painel lateral captura os atalhos.»*
> 3. *«Ao selecionar o pincel, o modal deveria se fechar automaticamente.»*

## §1 — A cadeia do teclado, medida

`shells/desktop/src/input_dispatch/keyboard.rs`, antes:

```
capture_binding_if_listening   → return      (o press-to-bind do Input Map)
ramo_teclas_3d                 → return      ⛔ AQUI
handler.on_key                              (o dedo do jogador, a fita de input)
F9                             → return
command_palette_keys           → return      ← a paleta só aqui
```

⇒ com a escultura na mão, o `ramo_teclas_3d` devolve `true` em **`1`–`0`, `G`, `H`, `T`, `S`, `A`
e `M`**. Escrever `clay` na busca do modal trocava o **pincel por baixo dele** e não punha uma
letra na caixa.

## §2 — ⛔⛔ A promessa estava escrita, e violada por duas linhas noutro ficheiro

`input_dispatch/keyboard_palette.rs`, no corpo da própria porta:

> *«Vem **PRIMEIRO** (antes dos atalhos de painel/ferramenta) para uma letra digitada nunca vazar
> num atalho de grafo embaixo.»*

E o cabeçalho dela: *«Its keys are captured **before every other shortcut**»*.

## §3 — ⚠️⚠️ E é a MESMA CLASSE que aquele ramo já pagou, com outra pergunta

O doc do `ramo_teclas_3d` conta a primeira vez:

> *«a nota virou o bug. Ela dizia "inerte (e portanto invisível) sem cena armada"… e ficou falso no
> dia do pill: num run normal a cena passa a existir ao primeiro clique, e sair do modo nunca a
> destrói. Como este `return` corre ANTES do `handler.on_key`, uma porta que só perguntava "a cena
> existe?" passou a comer **os dez dígitos e ~26 letras de todo painel do app, para sempre**.»*

A cura de então foi perguntar pelo **PONTEIRO** (`sculpt3d_keys_live`). ⭐ **Um MODAL é outra
pergunta:** enquanto ele está no ecrã, nada por baixo dele tem teclado — esteja o ponteiro onde
estiver. *A cura anterior respondeu a uma das duas perguntas e a nota não disse que havia duas.*

## §4 — Os três reports são UM

| report | o que de facto aconteceu |
|---|---|
| *«não captura o que escrevo»* | as letras foram para o `ramo_teclas_3d` |
| *«o painel lateral captura os atalhos»* | o mesmo, visto do outro lado |
| *«ao selecionar deveria fechar»* | ele **nunca selecionou pela paleta** — o pincel mudou por atalho, e o modal ficou aberto porque ninguém lhe tocou |

⭐ **O caminho do CLIQUE já fechava** (`set_command_pick` + `close_command_palette` na mesma linha),
e isso passou a ser **medido** em vez de deduzido: `a_paleta_de_pinceis_fecha_ao_escolher`, pela
porta real do chrome (`dispatch_all`) e não pelo handler. *Uma causa descartada por raciocínio e
não por medição volta na wave seguinte.*

## §5 — O que fica gateado

`shells/desktop/tests/it/um_modal_aberto_tem_o_teclado_antes_da_cena_3d.rs` — três metades:

| gate | afirma |
|---|---|
| `a_paleta_ve_a_tecla_antes_da_cena_3d` | a ordem que o report quebrou |
| `a_paleta_ve_a_tecla_antes_do_dedo_do_jogador` | com um modal aberto, o que se escreve não é entrada de jogo |
| `so_o_capturador_de_atalhos_ve_a_tecla_antes_do_modal` | a **única** excepção é declarada, e **nada** entra entre as duas (conta os `return;` da fatia) |

⚠️ **Ele lê TEXTO, e a razão é declarada:** o `key_input` pede um `winit::KeyEvent`, que não se
constrói num teste. O que se pode afirmar sem ele é a **ORDEM DOS RAMOS**, que é exactamente a
grandeza que o defeito tinha errada. ⛔ E o gate exige que as três âncoras **existam** — sem isso,
renomear um ramo deixaria as buscas a devolver `None` e o teste **verde a afirmar nada**.

**Mutação:** devolver a chamada da paleta para depois do `ramo_teclas_3d` ⇒ 2 dos 3 sangram (o
terceiro mede outra propriedade, e fica verde — honesto).
