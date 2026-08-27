#!/usr/bin/env python3
"""**A QUARTA ESPÉCIE DE KNOB MORTO** — o param declarado que NENHUM código lê.

    python3 "docs/Motion Nodes/ferramentas/knobs_declarados_nunca_lidos.py"

⚠️ **Esta é a única das quatro espécies que a execução NÃO acha.** A varredura dinâmica
(`crates/ph2d-node-registry-init/tests/dead_knob_sweep.rs`) cozinha o nó duas vezes e compara
as colunas: ela acusa um param que não muda a saída. Mas um param **nunca lido** também não
muda a saída, então ela o acusaria pelo mesmo motivo que acusa um param lido-e-descartado — e
as duas curas são opostas (uma é ligar o fio, a outra é consertar o cálculo). Separar as duas
é uma pergunta sobre o TEXTO da crate, e é o que este arquivo responde.

**Como ela lê.** Para cada `crates/ph2d-node-*/`: os params declarados saem do `MANIFEST`
(os `ParamSpec { name: "…" }`), e a LEITURA é procurada em duas formas, que são as duas únicas
por onde um valor de param entra num cálculo: uma chamada `…param("x")` (o `EvalCtx::param` e
os seus invólucros) ou o nome dentro da lista `params: &[…]` de um `GpuKernel` (o caminho de
device, onde o WGSL o recebe como campo). Declarado e sem nenhuma das duas ⇒ **acusação**.

⚠️ **A primeira versão desta sonda contava MENÇÕES entre aspas na crate inteira, e era quase
vazia por construção.** Todo param declarado é também nomeado no `ParamUiHint` que o pinta —
logo todo param tem pelo menos duas menções, e o limiar «≤ 1» nunca disparava. Ela imprimiu
*"nenhuma acusação"* sobre 613 params e isso não era uma medição, era a ausência de uma. *Um
instrumento cujo verde é garantido pela forma dos dados não mede nada* — a menção não é a
leitura, e é a leitura que decide se o knob age.

⚠️ **Um nome de param que é também uma palavra comum (`x`, `mode`, `size`) casa por acidente.**
Por isso o casamento exige a forma sintática da leitura (`param("size")`), nunca o nome solto.
"""

import glob
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(__file__), '..', '..', '..')
PARAMSPEC = re.compile(r'ParamSpec\s*\{\s*name:\s*"([A-Za-z0-9_]+)"')


def crate_text(crate):
    """Todo o texto da crate — src, tests, kernels, tudo."""
    out = []
    for root, _, files in os.walk(crate):
        if os.sep + 'target' in root:
            continue
        for f in files:
            if f.endswith(('.rs', '.wgsl', '.toml')):
                with open(os.path.join(root, f), encoding='utf-8', errors='replace') as fh:
                    out.append(fh.read())
    return '\n'.join(out)


def self_test():
    """**O CONTROLE POSITIVO DO PRÓPRIO INSTRUMENTO** — ele ainda consegue ACUSAR?

    ⚠️ **Ele nasceu no dia em que a lista chegou a zero.** A auditoria de 2026-08-27 mostrou que
    as sete acusações que esta sonda imprimia eram **todas falsas** (quatro por tupla, três por
    prefixo de módulo); alargar os padrões curou-as e levou a lista a `0`. Mas o doc desta sonda
    já avisava, sobre a v1, que *"um instrumento cujo verde é garantido pela forma dos dados não
    mede nada"* — e um matcher mais permissivo é exactamente a forma de o verde voltar a ser
    garantido. ⇒ **cada corrida prova primeiro que um morto sintético é apanhado**, e que as
    quatro rotas de leitura conhecidas são vistas.

    Falha aqui sai VERMELHO e não imprime catálogo nenhum: uma lista vazia de um instrumento que
    não sabe acusar não é uma boa notícia, é a ausência de uma medição.
    """
    dead = 'ParamSpec { name: "zz_dead" }'
    reads = {
        'literal': 'ParamSpec { name: "zz_lit" }\n ctx.param("zz_lit")',
        'const nu': 'ParamSpec { name: "zz_c" }\n const ZZ_C: &str = "zz_c";\n ctx.param(ZZ_C)',
        'const qualificada': ('ParamSpec { name: "zz_q" }\n const ZZ_Q: &str = "zz_q";\n'
                              ' ctx.param(rest::ZZ_Q)'),
        'tupla': ('ParamSpec { name: "zz_t" }\n'
                  ' const ZZ_T: (&str, &str) = ("zz_t", "zz_u");\n ctx.param(ZZ_T.0)'),
        'kernel de device': 'ParamSpec { name: "zz_k" }\n params: &["zz_k"]',
    }
    bad = []
    if 'zz_dead' not in scan(dead)[1]:
        bad.append('um param declarado e NUNCA lido nao foi acusado')
    for label, text in reads.items():
        name = re.search(r'name: "([a-z_]+)"', text).group(1)
        if name in scan(text)[1]:
            bad.append(f'a rota de leitura «{label}» foi acusada de morta')
    return bad


def scan(text):
    """Os params declarados e os ACUSADOS deste texto — a lei, separada da travessia."""
    declared = PARAMSPEC.findall(text)
    # As listas `params: &["a", "b"]` dos `GpuKernel` — a rota de device.
    kernel_names = set()
    for block in re.findall(r'params:\s*&\[([^\]]*)\]', text):
        kernel_names.update(re.findall(r'"([A-Za-z0-9_]+)"', block))
    # ⚠️ **O nome do param chega ao `ctx.param` por uma CONSTANTE em metade das crates**
    # (`const POINT_SCALE: &str = "point_scale";` … `ctx.param(POINT_SCALE)`), e um padrão
    # que só procure o literal entre aspas acusa esses de nunca-lidos. Foi assim que a 2ª
    # versão desta sonda produziu **cinco** falsos positivos.
    # *Um padrão sintático mede a SINTAXE que conhece, e o silêncio dele não é ausência.*
    const_of = {}
    for ident, value in re.findall(
        r'const\s+([A-Z0-9_]+)\s*:\s*&\'?\w*\s*str\s*=\s*"([A-Za-z0-9_]+)"', text
    ):
        const_of.setdefault(value, set()).add(ident)
    # ⚠️ A terceira forma: um ARRAY constante de nomes, varrido para dentro do `ctx.param`. O
    # nome nunca aparece ao lado da chamada, então a regra passa a ser a INTENÇÃO da declaração
    # — um agregado de nomes de param só existe para ser aberto em leituras. Indulgência
    # DELIBERADA: deixar passar um morto é barato, acusar um vivo custa a lista inteira.
    # ⚠️ E a QUARTA, achada em 2026-08-27: a TUPLA (`const SIZE_TAPER: (&str, &str, &str)`),
    # lida por campo (`ctx.param(taper::SIZE_TAPER.0)`).
    agg_names = set()
    for pat in (
        r'const\s+[A-Z0-9_]+\s*:\s*(?:\[|&\[)[^=]*=\s*&?\[([^\]]*)\]',
        r'const\s+[A-Z0-9_]+\s*:\s*\([^)]*\)\s*=\s*\(([^)]*)\)',
    ):
        for block in re.findall(pat, text):
            agg_names.update(re.findall(r'"([A-Za-z0-9_]+)"', block))
    accused = []
    for name in sorted(set(declared)):
        cpu = len(re.findall(r'param\w*\(\s*"' + re.escape(name) + r'"', text))
        for ident in const_of.get(name, ()):
            # ⚠️ **O identificador pode vir QUALIFICADO POR MÓDULO** — `ctx.param(rest::
            # REST_START)`. O padrão que só casava o identificador NU acusava metade dos nós
            # que partem os params num ficheiro irmão: em 2026-08-27 as SETE acusações que
            # esta sonda imprimia eram falsas, quatro por tupla e três por este prefixo.
            # *Um instrumento cujas acusações são todas falsas deixa de ser lido, e aí a que
            # for verdadeira também não é.*
            cpu += len(
                re.findall(
                    r'param\w*\(\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*'
                    + re.escape(ident)
                    + r'\b',
                    text,
                )
            )
        if not (cpu or name in kernel_names or name in agg_names):
            accused.append(name)
    return sorted(set(declared)), accused


def main():
    bad = self_test()
    if bad:
        print('!! O CONTROLE DO PROPRIO INSTRUMENTO FALHOU -- a lista abaixo nao vale nada:')
        for b in bad:
            print(f'   - {b}')
        return 1
    crates = sorted(glob.glob(os.path.join(ROOT, 'crates', 'ph2d-node-*')))
    if not crates:
        sys.exit('nenhuma crate de nó encontrada')
    accused, scanned, params_total, read_total = [], 0, 0, 0
    for crate in crates:
        if not os.path.isdir(crate):
            continue
        declared, dead = scan(crate_text(crate))
        if not declared:
            continue
        scanned += 1
        params_total += len(declared)
        read_total += len(declared) - len(dead)
        accused.extend((os.path.basename(crate), n) for n in dead)
    print(f'# {scanned} crates de nó · {params_total} params declarados · {read_total} com leitura')
    # ⚠️ **O controle positivo do próprio instrumento.** Se NENHUM param aparecesse como lido,
    # o padrão de leitura estaria errado e o zero-acusações seria vácuo, não saúde — foi
    # exactamente assim que a 1ª versão passou. Um instrumento que não consegue ver o caso
    # NORMAL não pode ser acreditado quando não vê o caso raro.
    if read_total == 0:
        print('!! o padrao de leitura nao casou NADA — o instrumento esta' + " quebrado, nao o catalogo")
        return 1
    if not accused:
        print('nenhum param declarado-e-nunca-lido. (As outras três espécies são da sonda dinâmica.)')
        return 0
    print(f'\n# {len(accused)} ACUSAÇÕES — declarado no MANIFEST, sem `param("…")` nem entrada de kernel\n')
    print(f"{'crate':<46} param")
    for crate, name in accused:
        print(f'{crate:<46} {name}')
    print('\n⚠️ ACUSAÇÃO, não veredito: um nó pode ler o param por uma via que este padrão não vê.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
