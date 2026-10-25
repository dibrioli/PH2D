# Os braços `97` e `98` do load — RECUSADOS (arquivo verbatim)

> Tirado de `shells/desktop/src/project_load_read.rs` em 2026-10-25 (`line/BlenderConstraints`, W1), para pagar o
> degrau `194` sob a catraca `the_shell_only_shrinks`: descreve braços que já não existem. Texto original:

```rust
            // ⭐ **98 → 99: o corte da `Sprite`** (ADR-0164 F1 passo 6). A forma do ficheiro é a
            // mesma, então ele lê-se com o tipo VIVO — o que muda são os bytes DENTRO do blob da
            // `Sprite`, que o parse atravessa sem olhar. Ver `crate::project_migrate_sprite`.
            //
            // ⚠️⚠️ **ESTE BRAÇO ERA `97 =>` E FOI RE-ENGATADO NA INTEGRAÇÃO de 2026-08-26.** Ele
            // nasceu quando o corte da `Sprite` era o degrau `98`; a `line/Vector` pôs outro degrau
            // no meio (o `morph_shape` do `ObjectPose`) e o corte passou a ser o `99`. ⛔ **Deixá-lo
            // em `97` seria o defeito que o bump existe para impedir:** este braço lê os bytes com o
            // tipo **VIVO**, e o tipo vivo já tem o campo da `line/Vector` — um v97 sairia lixo
            // bem-formado. *Um número que soma entre linhas move o CONSUMIDOR dele, não só a
            // declaração.*
            //
            // ⛔ **Um v97 é RECUSADO**, e é a decisão da `line/Vector` aplicada até ao fim: ela
            // escolheu, com o Enio, não congelar um `ProjectFileV97` (*"não há projetos salvos"*).
            // Sem tipo congelado não existe forma honesta de ler aqueles bytes. O `v95` continua a
            // subir a escada inteira, porque o tipo dele **está** congelado.
            // ⛔⛔ **O braço `98` MORREU em 2026-08-30, e a razão é a mesma que matou o `97`.**
            //
            // Ele lia os bytes com o tipo **VIVO**, e a premissa era *«a forma do ficheiro é a
            // mesma»*. O degrau `105` partiu-a de uma maneira que o `104` não partia: a biblioteca
            // entrou como **último campo do `ProjectState`**, que é o **primeiro campo do
            // `ProjectFile`** ⇒ no fluxo de bytes ela cai **no MEIO**. Um campo apendado no fim
            // fazia o postcard chegar ao fim dos bytes e **recusar em voz alta**; um campo no meio
            // faz-lhe ler *lixo bem-formado*.
            //
            // ⇒ um v98 cai no `_ =>` e é **recusado com o número na frase**, que é a decisão que a
            // `line/Vector` já tomou para o v97 e que o Enio confirmou (*«não há projetos
            // salvos»*). ⚠️ Quem um dia precisar dele **congela um `ProjectFileV98` primeiro** —
            // sem tipo congelado não há forma honesta de ler aqueles bytes.
```
