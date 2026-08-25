---
name: feedback_write_on_an_existing_path_says_updated_not_created
description: "A ferramenta Write num caminho que já existe responde «updated», não «created» — e um nome de arquivo plausível para uma cena nova é exactamente o nome que a cena velha já tem"
metadata:
  type: feedback
---

Criando a cena `=84` da folha 11, escrevi `motion_state_conferencia_demos_fx.rs`. O
arquivo **já existia**: era a cena `=70`, a família `fx.*` inteira, com 140 linhas de
gates. O `Write` respondeu

> `File ... has been updated successfully`

— **«updated», e não «created»** — e eu li a resposta como sucesso sem reparar no
verbo. Só o compilador acusou, três passos depois, com `the name ... is defined
multiple times`.

**Why:** o modo de falha é silencioso na direcção que importa. A ferramenta não
recusa, o texto do sucesso é quase idêntico nos dois casos, e o conteúdo perdido só
volta pelo git. ⚠️ **E o gatilho é estrutural, não distração**: um nome BOM para uma
cena nova (`..._demos_fx.rs` para uma cena de FX) é exactamente o nome que a cena
velha do mesmo assunto já escolheu, pela mesma boa razão. *Quanto melhor o nome, maior
a chance de ele estar ocupado.*

⚠️⚠️ **ACONTECEU OUTRA VEZ NO DIA SEGUINTE, com esta memória já escrita** (2026-08-23,
cena `=85`): escrevi `motion_state_conferencia_demos_shape.rs` para a cena da FORMA
desenhada, e aquele nome já era a cena `=55` (Pulse Width / Offset) — **da mesma folha
06**. Dois arquivos por cima, o `Write` a responder *"updated"* nos dois. ⇒ *a regra
existia e não estava no CAMINHO de quem executa*: eu tinha-a escrito como o passo 1 de
uma lista, e uma lista não corre. **Ela vale como GATILHO, não como conselho:** todo
`Write` para um caminho que eu não li nesta sessão é um `ls` primeiro, sem exceção e
sem julgar se "este nome parece livre" — foi precisamente o julgamento que falhou as
duas vezes.

⚠️⚠️⚠️ **TERCEIRA vez, 2026-08-24, e a PIOR — porque não foi o `Write`.** Ao cortar
`motion_bridge_rowcap_tests.rs` pelo teto de LOC, escrevi a metade da altura em
`motion_bridge_dock_tests.rs` com um `open(p,'w')` de python. Aquele nome já era um
arquivo de **88 linhas com 3 gates de costura do dock da timeline** (W4.T4). O `Write`
pelo menos diz *«updated»*; **um `open('w')` não diz nada** — nem uma palavra, nem um
verbo para ler. Quem acusou foi o **clippy**, dez passos depois, com *«file is loaded
as a module multiple times»*, e só porque o `mod` antigo continuava a apontar para o
mesmo caminho. Se o módulo antigo tivesse outro nome de path, a perda passava.

⇒ **O gatilho do item 1 vale para TODA escrita, não para a ferramenta `Write`**: um
`open('w')`, um `>` de shell, um `cp` para um destino. E o padrão do nome repetiu-se
pela terceira vez com a mesma forma — *dock* era o nome certo para a metade que mede o
dock, e por isso já estava ocupado por outra coisa que mede o dock.

**How to apply:**
1. ⚠️ **GATILHO, não conselho:** *`Write` num caminho que não foi lido nesta sessão ⇒
   `ls` do diretório PRIMEIRO.* Um comando, resposta imediata, e ele custa menos que os
   três passos do item 3. Vale igual para um **nome de módulo** (`mod x;`) e para um
   símbolo público: o colisor não é sempre um arquivo.
2. **Leia o verbo da resposta.** `created` é o que se esperava; `updated` num caminho
   que devia ser novo é um alerta, não um sucesso.
3. Se aconteceu: `git checkout -- <arquivo>` **primeiro**, com o que você escreveu
   guardado ao lado (`cp` para o scratchpad). Restaurar antes de continuar — o
   conteúdo perdido não tem outra cópia, e cada passo seguinte o afasta.
4. Um arquivo restaurado tem de sair do `git status`: é assim que se sabe que a
   restauração foi completa e não parcial.

*Irmã de [[feedback_python_replace_silent_noop_after_fmt]]: as duas são a ferramenta a
imprimir sucesso sobre um resultado que não é o pedido.*
