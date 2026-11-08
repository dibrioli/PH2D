# Os traços das duas cenas (coordenadas da CÉLULA 400x220) + o gerador do script xdotool.
import math, json, sys
def lerp_poly(pts, step=3.0):
    out=[pts[0]]
    for (x0,y0),(x1,y1) in zip(pts,pts[1:]):
        d=math.hypot(x1-x0,y1-y0); n=max(1,int(d/step))
        for k in range(1,n+1): out.append((x0+(x1-x0)*k/n, y0+(y1-y0)*k/n))
    return out
def cena_a():
    s=[]
    s.append([(80+ (4+66*t/(6*math.pi))*math.cos(t), 110+(4+66*t/(6*math.pi))*math.sin(t)) for t in [i*0.05 for i in range(int(6*math.pi/0.05)+1)]])
    s.append([(170+i*27.5, 20 if i%2==0 else 80) for i in range(9)])
    s.append([(170+u*220, 130+30*math.sin(u*3*math.pi)) for u in [i/80 for i in range(81)]])
    for k in range(6):
        x=180+k*35; s.append([(x,212),(x+30,166)]); s.append([(x,166),(x+30,212)])
    return s
def blocos():
    out=[]
    for x0 in (20,145,270):
        out.append([[ (x0+10, y),(x0+100,y)] for y in range(32,196,12)])
    return out  # 3 listas de traços: azul, amarelo, vermelho
def cena_b():
    s=[]
    for i,y in enumerate(range(50,181,26)):
        s.append([(30,y),(370,y)] if i%2==0 else [(370,y),(30,y)])
    s.append([(30,40),(370,190)]); s.append([(370,40),(30,190)])
    return s
if __name__=='__main__':
    json.dump({'a':cena_a(),'blocos':blocos(),'b':cena_b()}, open(sys.argv[1],'w'))
