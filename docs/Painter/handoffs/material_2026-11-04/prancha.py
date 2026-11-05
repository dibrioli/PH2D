import numpy as np, sys
from PIL import Image, ImageDraw, ImageFont
sys.argv=["x"]; exec(open("mede2.py").read().split("if __name__")[0])
F=lambda n: ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans.ttf",n)
try: F(10)
except Exception:
    import glob
    p=glob.glob("/usr/share/fonts/**/*DejaVuSans.ttf",recursive=True)+glob.glob("/usr/share/fonts/**/*.ttf",recursive=True)
    F=lambda n,p=p: ImageFont.truetype(p[0],n)
COLW=480
def op(f): return Image.open(f).convert("RGB")
def zoom(im,box,w): 
    c=im.crop(box); return c.resize((w,int(w*c.height/c.width)),Image.NEAREST)
def fit(im,box,w):
    c=im.crop(box); return c.resize((w,int(w*c.height/c.width)),Image.LANCZOS)
def plot(fs,w=COLW,h=130):
    im=Image.new("RGB",(w,h),"white");d=ImageDraw.Draw(im)
    r,p,i0,i1,m=mede(fs); v=p[i0-8:i0+34]/m
    for y in (0,1): 
        Y=h-10-y*(h-30)/1.9; d.line([(0,Y),(w,Y)],fill=(190,190,190))
    pts=[(k*w/len(v),h-10-max(min(float(x),1.9),-.05)*(h-30)/1.9) for k,x in enumerate(v)]
    d.line(pts,fill=(160,20,20),width=2)
    d.text((4,2),"perfil, borda de cima: 1 = miolo",fill="black",font=F(13))
    return im
def bloco(titulo,cols,yg,xc=340):
    # cols=[(rótulo, s, d)]
    ym=120
    linhas=[("traço + cruzado (seco)",lambda s,d: fit(op(s),(40,0,960,640),COLW)),
      ("borda do traço reto, 6×",lambda s,d: zoom(op(s),(xc,ym-yg-6,xc+80,ym-yg+22),COLW)),
      ("borda em diagonal, 6×",lambda s,d: zoom(op(d),(440,int(110+.2857*400-yg-16),520,int(110+.2857*400-yg+20)),COLW)),
      ("cruzamento, 2×",lambda s,d: zoom(op(s),(500-max(yg+30,60),430-max(yg+30,60),500+max(yg+30,60),430+max(yg+30,60)),COLW//2)),
      ("perfil",lambda s,d: plot(s))]
    rows=[]
    for nome,fn in linhas:
        ims=[fn(s,d) for _,s,d in cols]; rows.append((nome,ims,max(i.height for i in ims)))
    H=44+sum(r[2]+26 for r in rows)
    out=Image.new("RGB",(COLW*len(cols)+12*(len(cols)+1),H),(245,245,245));D=ImageDraw.Draw(out)
    D.text((12,4),titulo,fill="black",font=F(22))
    for i,(rot,_,_) in enumerate(cols): D.text((12+i*(COLW+12),30),rot,fill=(30,30,120),font=F(15))
    y=52
    for nome,ims,h in rows:
        D.text((12,y),nome,fill=(90,90,90),font=F(13)); y+=18
        for i,im in enumerate(ims): out.paste(im,(12+i*(COLW+12),y))
        y+=h+8
    return out
R="fotos/WATERCOLOR/";N="nosso/"
A=bloco("Wet Liner · traço 40 px · vermelho 190,30,30 · seco",[("Rebelle: Watercolor/Liner (tamanho 40)",R+"liner_s40_B.png",R+"liner_d40_B.png"),
 ("PH2D de fábrica",N+"Wet_Liner_20_fabrica.ppm",N+"Wet_Liner_20_d_fabrica.ppm"),
 ("PH2D Edge Darkening 0",N+"Wet_Liner_20_edge0.ppm",N+"Wet_Liner_20_d_edge0.ppm"),
 ("PH2D Edge Darkening 0 + Body 0",N+"Wet_Liner_20_edge0_body0.ppm",N+"Wet_Liner_20_d_edge0_body0.ppm")],20)
B=bloco("Wet Wash · traço 120 px · vermelho 190,30,30 · seco",[("Rebelle: Watercolor/Wash Round (tamanho 120)",R+"washround_s120_B.png",R+"washround_d120_B.png"),
 ("PH2D de fábrica",N+"Wet_Wash_60_fabrica.ppm",N+"Wet_Wash_60_d_fabrica.ppm"),
 ("PH2D Edge Darkening 0",N+"Wet_Wash_60_edge0.ppm",N+"Wet_Wash_60_d_edge0.ppm"),
 ("PH2D Edge Darkening 0 + Body 0",N+"Wet_Wash_60_edge0_body0.ppm",N+"Wet_Wash_60_d_edge0_body0.ppm")],60)
W=max(A.width,B.width);H=A.height+B.height+50
o=Image.new("RGB",(W,H),(245,245,245));o.paste(A,(0,0));o.paste(B,(0,A.height+10))
D=ImageDraw.Draw(o);D.text((12,H-30),"Rebelle 8.3.4 (oráculo, tela virtual, render por software): o papel com relevo dele aparece e não pôde ser retirado. PH2D: tela lisa sem papel. O tamanho 20/60 do PH2D dá a largura 40/120.",fill=(90,90,90),font=F(13))
o.save("prancha_rebelle_x_ph2d.png");print(o.size)
for nome,f in [("reb liner 40",R+"liner_s40_B.png"),("reb liner 120",R+"liner_s120_B.png"),("reb wash 40",R+"washround_s40_B.png"),("reb wash 120",R+"washround_s120_B.png"),("nos liner 40",N+"Wet_Liner_20_fabrica.ppm"),("nos liner 120",N+"Wet_Liner_60_fabrica.ppm"),("nos wash 40",N+"Wet_Wash_20_fabrica.ppm"),("nos wash 120",N+"Wet_Wash_60_fabrica.ppm"),("nos liner120 e0",N+"Wet_Liner_60_edge0.ppm"),("nos wash120 e0",N+"Wet_Wash_60_edge0.ppm"),("nos liner120 e0b0",N+"Wet_Liner_60_edge0_body0.ppm"),("nos wash120 e0b0",N+"Wet_Wash_60_edge0_body0.ppm")]:
    r,p,i0,i1,m=mede(f);v=p/m
    print(nome.ljust(20),"topo",[round(float(x),2) for x in v[i0-1:i0+10]],"base",[round(float(x),2) for x in v[i1+1:i1-10:-1]],"centro",round(float(v[(i0+i1)//2]),2))
