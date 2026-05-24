# Briefing — node-crate (fan-out)

> **Movido + unificado com tool-crate.** Desde 2026-05-22 (DIRETRIZ v6.6) este
> briefing foi integrado no doc único de implementação; desde v6.8 (mesmo dia,
> pós-ADR-0040 FREEZE) ele foi **unificado com o briefing de tool-crate** num
> único balde simétrico em [`DIRETRIZ.md` §3.8 "Fan-out drop-crate (A) — node
> OU tool"](DIRETRIZ.md), com briefing parametrizado por família. Mapa:
>
> - **§3.8.1** — tabela node↔tool (pasta / codegen / wiring / cap / contrato / templates / pegadinhas)
> - **§3.8.2** — briefing pronto-pra-colar (parametrizado `<family>` + blocos `[node]` / `[tool]`)
> - **§3.8.3** — sabores de tool (tool-only)
> - **§3.8.3.1** — status atual do `RasterEditTool` (tool-only, heads-up importante; renomeado de `ImageEditTool` em ADR-0041)
> - **§3.8.4** — garantia sem-colisão (vale pras duas famílias)
> - **§3.8.5** — checklist do revisor (seção comum + node-específica + tool-específica)
>
> A triagem "preciso do Coordenador ou só do Implementador?" está em
> [`DIRETRIZ.md` §1.4](DIRETRIZ.md) — node novo e tool nova são ambos
> caminho **(A) Só Implementador**, simétricos.
>
> **Quer um exemplo paste-ready?** [`examples-fan-out.md`](examples-fan-out.md)
> traz o briefing §3.8.2 **instantiated fim-a-fim** para um node concreto
> (`ph2d-node-shader-blur`) e um tool concreto (`ph2d-tool-grayscale`), com
> todos os arquivos a criar — zero placeholder. Adicionado 2026-05-24 pra
> fechar o atrito identificado na auditoria multi-agente readiness.
>
> Não edite este stub — edite o §3.8 do DIRETRIZ (doc único, não fragmentar).
