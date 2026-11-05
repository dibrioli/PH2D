#!/usr/bin/env python3
"""Imprime o texto de seções dos dois estudos de game engines (ver ../README.md).

    python3 -I estudo_secao.py --listar
    python3 -I estudo_secao.py "Sprite"
    python3 -I estudo_secao.py "Physics Body" "Corpo físico"

O nome é o título exato da seção (sem diferença de maiúsculas). Os estudos moram fora do
repositório; PH2D_ESTUDOS troca a pasta. Sai com 1 se algum nome não casar com nenhuma seção.
"""
import os
import sys
from html.parser import HTMLParser

PASTA = os.environ.get("PH2D_ESTUDOS", "/home/enio/Documentos/GameEngineSamples")
DOCS = {"crítica": "critica_ph2d.html", "estudo": "estudo_paineis_game_engines.html"}
FECHA_LINHA = {"p", "li", "tr", "div", "h1", "h2", "h3", "td", "th"}


class Secoes(HTMLParser):
    """Parte a página pelos <h2>; o texto antes do primeiro fica em «(topo)»."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.secoes = [["(topo)", []]]
        self.oculto = 0
        self.no_h2 = False

    def handle_starttag(self, tag, attrs):
        if tag in ("script", "style"):
            self.oculto += 1
        elif tag == "h2":
            self.secoes.append(["", []])
            self.no_h2 = True
        elif tag == "h3":
            self.secoes[-1][1].append("\n### ")

    def handle_endtag(self, tag):
        if tag in ("script", "style"):
            self.oculto -= 1
        elif tag == "h2":
            self.no_h2 = False
        elif tag in FECHA_LINHA:
            self.secoes[-1][1].append("\n")

    def handle_data(self, data):
        limpo = " ".join(data.split())
        if self.oculto or not limpo:
            return
        if self.no_h2:
            self.secoes[-1][0] = (self.secoes[-1][0] + " " + limpo).strip()
        else:
            self.secoes[-1][1].append(limpo + " ")


def le(arquivo):
    parser = Secoes()
    with open(os.path.join(PASTA, arquivo), encoding="utf8") as f:
        parser.feed(f.read())
    return [(nome, "".join(partes)) for nome, partes in parser.secoes]


def compacta(texto):
    linhas = (linha.strip() for linha in texto.split("\n"))
    return "\n".join(linha for linha in linhas if linha)


def main(argumentos):
    if not argumentos:
        print(__doc__)
        return 2
    paginas = {rotulo: le(arquivo) for rotulo, arquivo in DOCS.items()}
    if argumentos == ["--listar"]:
        for rotulo, secoes in paginas.items():
            print(f"[{rotulo}] " + " | ".join(nome for nome, _ in secoes[1:]))
        return 0
    sem_secao = []
    for pedido in argumentos:
        achou = False
        for rotulo, secoes in paginas.items():
            for nome, texto in secoes:
                if nome.casefold() == pedido.casefold():
                    achou = True
                    print(f"\n===== [{rotulo}] {nome} =====\n{compacta(texto)}")
        if not achou:
            sem_secao.append(pedido)
    for pedido in sem_secao:
        print(f"⛔ nenhuma seção se chama «{pedido}»; veja --listar", file=sys.stderr)
    return 1 if sem_secao else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
