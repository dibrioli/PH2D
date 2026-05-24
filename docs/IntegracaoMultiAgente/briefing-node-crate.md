# Briefing — node-crate (fan-out)

> **Stub — não edite aqui.** Briefing canônico vive em [`DIRETRIZ.md` §3.A](DIRETRIZ.md)
> ("Fan-out drop-crate — node OU tool"). Desde DIRETRIZ v6.6 (2026-05-22) o
> briefing foi integrado no doc único de implementação; desde v6.8 (mesmo dia,
> pós-ADR-0040 FREEZE) ele é **unificado com o briefing de tool-crate** num
> único balde simétrico, com briefing parametrizado por família.
>
> Mapa do §3.A:
>
> - **§3.A.1** — tabela node↔tool (pasta / codegen / wiring / cap / contrato / templates / pegadinhas)
> - **§3.A.2** — briefing pronto-pra-colar (parametrizado `<family>` + blocos `[node]` / `[tool]`)
> - **§3.A.3** — sabores de tool (tool-only)
> - **§3.A.4** — trait `RasterEditTool` (heads-up, renomeado de `ImageEditTool` em ADR-0041)
> - **§3.A.5** — garantia formal sem-colisão
> - **§3.A.6** — checklist do revisor
>
> A triagem "Coordenador ou só Implementador?" está em
> [`DIRETRIZ.md` §2](DIRETRIZ.md). Node novo e tool nova são ambos
> caminho **(A) Só Implementador**, simétricos.
>
> **Variante 100% paste-ready (sem placeholder):** [`examples-fan-out.md`](examples-fan-out.md)
> instancia o briefing fim-a-fim para `ph2d-node-shader-blur` e
> `ph2d-tool-grayscale`, com todos os arquivos a criar.
>
> Não edite este stub — edite o §3.A do DIRETRIZ (doc único, não fragmentar).
