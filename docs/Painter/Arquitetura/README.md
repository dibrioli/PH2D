# Arquitetura do Painter — a documentação da refatoração (aberta 31/10)

> Pasta ESPECIAL pedida pelo dono (31/10, noite): *«profunda, completa, brutal refatoração de todo módulo painter
> em seus 4 modos … documentação técnica com não apenas as especificações de cada aspecto do projeto, mas com fotos
> demonstrativas nos docs para a própria LLM revisitar, estudar e entender.»*
>
> Leitor: a próxima LLM. Cada afirmação aponta `ficheiro:linha` e cada lei tem a FOTO que o motor pinta hoje
> (gerada, nunca desenhada à mão — a foto não mente). Prompt da refatoração:
> [`49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md`](../49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md).
> Régua «antes»: etiqueta `painter-pre-refatoracao-com-problemas-2026-10-31`.

## Como ler

| ordem | doc | o que responde |
|---|---|---|
| 1 | `F1_mapa_do_modulo_e_os_4_modos.md` | que crates existem, quem chama quem, como um traço anda do painel ao píxel em cada modo (Digital · Watercolor · Impasto · Wet Paint) |
| 2 | `F1_censo_wet_paint.md` | cada controlo do Wet Paint: quem escreve, quem lê, se decide, faixa, unidade, duplicados, mortos |
| 3 | `F0_varredura.md` | a régua-mãe: o espaço que o painel deixa o artista alcançar, varrido; onde a lei de hoje parte |
| 4 | `F2_grandezas_fisicas.md` | controlo ↔ grandeza física ↔ equação ↔ fonte (Kubelka–Munk, Curtis 97, IMPaSTo, dAb, Rebelle) |
| 5 | `F3_lei_nova.md` (+ ADR) | a arquitetura nova: Tinta · Pincel · Superfície · Tempo · Luz, uma lei de camada |

`fotos/` guarda as imagens, uma subpasta por doc. Nenhuma foto entra sem o comando que a regera ao lado.

## Estado

| fase | estado |
|---|---|
| F0 varredura | em curso |
| F1 censo + mapa | em curso |
| F2 literatura | por fazer |
| F3 ADR | por fazer |
| F4 refatoração | por fazer |
| F5 ajuda ilustrada | por fazer (maquete ao dono antes da 1.ª tela) |
