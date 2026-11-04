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
| 5 | [ADR-0197](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md) | a arquitetura nova: Tinta · Pincel · Superfície · Tempo · Luz, uma lei de pigmento em volumes; os meios são pontos |
| 6 | `F1_presets_de_fabrica.md` | cada pincel de fábrica × cada knob: o que a migração tem de preservar |

`fotos/` guarda as imagens, uma subpasta por doc. Nenhuma foto entra sem o comando que a regera ao lado.

## Estado

> **Estado a 03/11:** as etapas até a E10c e o FPS do Real Oil ([J13](J13_fps_do_real_oil.md)) estão no `main`.
> ⚠️ **O produto ainda pinta pela lei `Velha`** (`LeiDaTinta`, `crates/ph2d-wet-paint/src/sim.rs`; a `Pigmento` só corre em testes):
> a troca é a E12. **Falta:** E11 (a placa não lê o meio nem a pasta) · E12 · E13 · os outros 3 modos · a ajuda ilustrada (F5, só
> maquete) · os ⚑ furos da E9 · crateras do Filbert/Flat Bristle · o chip de Dilution · os `max` de 35–78 ms do traço.
> **Ordem do dono (03/11):** a refatoração volta depois da integração da UI e do CI. Último estado por janela:
> [HANDOFF_02_11_janela12.md](HANDOFF_02_11_janela12.md) + [J13](J13_fps_do_real_oil.md). Testes de pincel: alarme barato
> ([como se usa](../../DevOps/PLANO_ACELERACAO_2026-11.md)).
>
> Prompt de continuação: [PROMPT_CONTINUACAO_REFATORACAO.md](PROMPT_CONTINUACAO_REFATORACAO.md). A tabela abaixo é a de abertura, com F4 e F5 atualizadas.

| fase | estado |
|---|---|
| F0 varredura | manchas escuras reproduzidas no motor; ferrugem e degraus em curso |
| F1 censo + mapa | feitos; matriz dos presets em curso |
| F2 literatura | feita — lei única com prova + modelo `ferramentas/lei_unica.py` |
| F3 ADR | ADR-0197 proposto |
| F4 refatoração | em curso: até a E10c; falta E11–E13 e os outros 3 modos |
| F5 ajuda ilustrada | só maquete, variante C aprovada (nada no app) |
