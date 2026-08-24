#!/usr/bin/env python3
"""VERIFICADOR do mapa de grade inteira — o gate nº4 da SPEC, executável.

A lei: entre duas faces vizinhas, a função de transição tem de ser
    g(x) = R(k)·x + t,  k ∈ {0,1,2,3},  t ∈ Z²
Deriva-se comparando as DUAS imagens da aresta partilhada, uma em cada carta
(SPEC §2.2). Imprime o resíduo de k e de t; um mapa de grade inteira tem os dois a zero.
⚠️ Percentis E contagem — um balde vazio lê-se como perfeito."""
import sys, math, cmath

def carrega(p):
    V=[];F=[];UV={}
    for l in open(p):
        if l[0]=='#': continue
        t=l.split()
        if not t: continue
        if t[0]=='v': V.append((float(t[1]),float(t[2]),float(t[3])))
        elif t[0]=='f': F.append((int(t[1]),int(t[2]),int(t[3])))
        elif t[0]=='canto': UV[(int(t[1]),int(t[2]))]=(float(t[3]),float(t[4]))
    return V,F,UV

def pct(v,q):
    if not v: return float('nan')
    v=sorted(v); return v[min(len(v)-1,max(0,int(round(q*(len(v)-1)))))]

def main(p):
    V,F,UV=carrega(p)
    # arestas interiores: mapa (a,b) ordenado -> [(face, canto_i, canto_j)]
    ar={}
    for fi,tri in enumerate(F):
        for k in range(3):
            a,b=tri[k],tri[(k+1)%3]
            ar.setdefault((min(a,b),max(a,b)),[]).append((fi,k,(k+1)%3,a,b))
    kres=[]; tres=[]; kdist={}; nint=0; nbordo=0; ndegen=0
    for key,lst in ar.items():
        if len(lst)!=2: nbordo+=1; continue
        nint+=1
        (f1,i1,j1,a1,b1),(f2,i2,j2,a2,b2)=lst
        u1=complex(*UV[(f1,i1)]); v1=complex(*UV[(f1,j1)])
        u2=complex(*UV[(f2,i2)]); v2=complex(*UV[(f2,j2)])
        # a MESMA aresta, nos dois sentidos: (a1,b1) e (a2,b2) podem estar invertidos
        if a1!=a2: u2,v2=v2,u2
        d1=v1-u1; d2=v2-u2
        # ⚠️ correcção do R-pré (2026-08-24): a aresta de imagem DEGENERADA saía por
        # `continue` MUDO — o instrumento imprimia `arestas_interiores` e um `n` menor,
        # sem nunca dizer que a diferença existia, quanto mais o que ela era. E é
        # precisamente o caso que a espec §2.1 manda colapsar ANTES de tudo: um gate
        # cego ao fenómeno que o seu próprio insumo prevê. Agora conta-se e imprime-se.
        if abs(d1)<1e-12 or abs(d2)<1e-12: ndegen+=1; continue
        kr=cmath.phase(d2/d1)/(math.pi/2)          # rotação em quartos de volta
        kk=round(kr); kres.append(abs(kr-kk)); kdist[kk%4]=kdist.get(kk%4,0)+1
        R=complex(0,1)**kk
        t=u2-R*u1                                   # translação
        tres.append(max(abs(t.real-round(t.real)), abs(t.imag-round(t.imag))))
    print(f"{p}")
    print(f"  faces={len(F)} arestas_interiores={nint} bordo={nbordo} degeneradas_no_dominio={ndegen} (medidas={nint-ndegen})")
    print(f"  residuo da ROTACAO (quartos de volta)  p50={pct(kres,.5):.3e} p99={pct(kres,.99):.3e} max={max(kres) if kres else float('nan'):.3e}   n={len(kres)}")
    print(f"  residuo da TRANSLACAO (celulas)        p50={pct(tres,.5):.3e} p99={pct(tres,.99):.3e} max={max(tres) if tres else float('nan'):.3e}   n={len(tres)}")
    print(f"  distribuicao das rotacoes por aresta: {dict(sorted(kdist.items()))}")
    mx=max(tres) if tres else float('inf')
    print(f"  ⇒ {'✓ E UM MAPA DE GRADE INTEIRA' if mx<1e-6 else '✗ NAO e inteiro: residuo max=%.3e'%mx}")

for p in sys.argv[1:]: main(p)
