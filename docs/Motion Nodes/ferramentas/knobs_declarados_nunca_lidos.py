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


def main():
    crates = sorted(glob.glob(os.path.join(ROOT, 'crates', 'ph2d-node-*')))
    if not crates:
        sys.exit('nenhuma crate de nó encontrada')
    accused, scanned, params_total, read_total = [], 0, 0, 0
    for crate in crates:
        if not os.path.isdir(crate):
            continue
        text = crate_text(crate)
        declared = PARAMSPEC.findall(text)
        if not declared:
            continue
        # As listas `params: &["a", "b"]` dos `GpuKernel` — a rota de device.
        kernel_names = set()
        for block in re.findall(r'params:\s*&\[([^\]]*)\]', text):
            kernel_names.update(re.findall(r'"([A-Za-z0-9_]+)"', block))
        # ⚠️ **O nome do param chega ao `ctx.param` por uma CONSTANTE em metade das crates**
        # (`const POINT_SCALE: &str = "point_scale";` … `ctx.param(POINT_SCALE)`), e um padrão
        # que só procure o literal entre aspas acusa esses de nunca-lidos. Foi assim que a 2ª
        # versão desta sonda produziu **cinco** falsos positivos — os pesos do `motion.mixer`
        # entre eles, que a auditoria de leitura já tinha confirmado vivos no mesmo dia.
        # *Um padrão sintático mede a SINTAXE que conhece, e o silêncio dele não é ausência.*
        const_of = {}
        for ident, value in re.findall(r'const\s+([A-Z0-9_]+)\s*:\s*&\'?\w*\s*str\s*=\s*"([A-Za-z0-9_]+)"', text):
            const_of.setdefault(value, set()).add(ident)
        # ⚠️ E há uma terceira forma: um ARRAY constante de nomes, varrido para dentro do
        # `ctx.param` (`const WEIGHTS: [&str; 4] = […]` … `WEIGHTS.iter().map(|w| ctx.param(w))`).
        # Aqui o nome nunca aparece ao lado da chamada, então casar o call-site é impossível
        # sem ler o fluxo — e a regra passa a ser a INTENÇÃO da declaração: um array de nomes
        # de param só existe para ser aberto em leituras. É uma indulgência DELIBERADA, e o
        # preço dela é a única direcção de erro aceitável aqui — deixar passar um morto é
        # barato, acusar um vivo custa a confiança na lista inteira.
        array_names = set()
        for block in re.findall(r'const\s+[A-Z0-9_]+\s*:\s*(?:\[|&\[)[^=]*=\s*&?\[([^\]]*)\]', text):
            array_names.update(re.findall(r'"([A-Za-z0-9_]+)"', block))
        scanned += 1
        for name in sorted(set(declared)):
            params_total += 1
            cpu = len(re.findall(r'param\w*\(\s*"' + re.escape(name) + r'"', text))
            for ident in const_of.get(name, ()):
                cpu += len(re.findall(r'param\w*\(\s*' + re.escape(ident) + r'\b', text))
            gpu = name in kernel_names or name in array_names
            if cpu or gpu:
                read_total += 1
            else:
                accused.append((os.path.basename(crate), name))

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
