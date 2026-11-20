"""The outdoor work area and two visible extensions of the Tanuki's house."""


def models(Model, lantern, window):
    assets=[]
    for stage,(x,z,w,d,side) in enumerate([(0.51,-2.65,2.42,2.02,1),(-5.86,-2.96,2.58,2.3,-1)]):
        room=Model("Hofanbau · Außenansicht",f"cabin_annex_{stage}")
        wall=["c39368","ce9e73","bd875d"]
        room.box(x,z,.22,w+.07,d+.08,.40,"a59984",.059)
        for i in range(9):
            room.box(x,z,.42+i*.208,w,d,.211,wall[i%3],.029)
        for dx in [-.46,.46]:
            for dz in [-.47,.47]:
                room.box(x+dx*w,z+dz*d,1.28,.18,.18,2.09,"976945",.024)
        for s in [-1,1]:
            x0=x+s*w*.57
            for j in range(8):
                zz=z+(j-3.5)*(d+.38)/8
                room.pole((x,zz,3.20),(x0,zz,2.13),.126,["bf825d","d19b71","c58c61"][j%3],7)
        room.pole((x,z-d*.63,3.25),(x,z+d*.63,3.25),.146,"bd855e",8)
        room.face([(x-w*.51,z+d*.53,2.18),(x+w*.51,z+d*.53,2.18),(x,z+d*.53,3.17)],"c79b73")
        window(room,x,z+d*.51,1.39,.71,.68)
        room.box(x,z+d*.68,.51,1.13,.58,.18,"bc8f63",.024)
        lantern(room,x+side*.82,z+d*.56,1.18)
        assets.append(room)

        wing=Model("Hofanbau · Gartenstube" if stage==0 else "Hofanbau · Lesekammer",f"interior_annex_{stage}")
        center=side*5.45
        color=["b9ae76","7e9a8a"][stage]
        for j in range(13):
            for k in range(4):
                wing.box(center+(j-6)*.30,(k-1.5)*1.46,-.073,.295,1.44,.178,
                         ["cc9b74","d7aa80","c49368"][j%3],.024)
        wing.box(center,0,-.245,4.06,6.12,.35,"a57958",.078)
        for j in range(12):
            y=.13+j*.204
            wing.box(center,-2.94,y,3.83,.17,.197,
                     (["dbca9b","e6d3ab"] if stage==0 else ["bcc3a8","cbd1b5"])[j%2],.021)
        wing.box(center,-2.85,2.59,3.95,.26,.24,"9f7656",.024)
        for xx in [center-1.82,center+1.82]:
            wing.box(xx,-2.77,1.25,.17,.22,2.60,"a57b58",.030)
        window(wing,center,-2.82,1.62,1.04,.82)
        wing.box(center,.43,.024,2.65,2.01,.040,color,.030)
        for xx in [center-1.21,center+1.21]:
            wing.box(xx,.43,.052,.093,1.98,.018,"e7d2a8",.009)
        for zz in [-.47,1.34]:
            wing.box(center,zz,.051,2.52,.089,.020,"e7d2a8",.009)
        wing.box(side*3.56,.61,.038,.46,1.96,.077,"bd966b",.023)
        lantern(wing,center+side*1.29,-2.48,1.72)
        assets.append(wing)

    seal=Model("Hütte · Verschlossene Seitenwand","interior_annex_wall")
    for i in range(13):
        seal.box(3.56,.61,.10+i*.197,.18,2.14,.190,
                 ["d9b68e","e9c99c","e2bd91"][i%3],.019)
    assets.append(seal)

    old=Model("Werkbank · Lose Bretter und Werkzeug","workbench_old")
    for x in [-.48,.48]:
        for z in [-.29,.29]:
            old.box(x,z,.32,.14,.14,.65,"98805c",.025)
    for j,z in enumerate([-.31,-.14,.16,.34]):
        old.box(0,z,.65-j*.015,1.19,.162,.096,["b6946a","9e8866","baa077"][j%3],
                .024,(j%2-.4)*.10)
    old.pole((-.44,.29,.12),(.42,.29,.49),.045,"9b7d59",4)
    old.box(.27,-.15,.759,.30,.104,.07,"8e8d7c",.018,.15)
    assets.append(old)

    forge=Model("Schmiedetisch · Amboss und Kohlenbecken","workshop_forge")
    forge.box(0,0,.33,1.33,.76,.65,"92775a",.059)
    forge.box(0,0,.71,1.53,.86,.13,"b28e65",.038)
    forge.box(-.26,.01,.89,.37,.39,.26,"81837f",.044)
    forge.box(-.28,.01,1.08,.68,.36,.16,"a2a29b",.039)
    forge.face([(-.61,-.15,1.17),(-.88,-.025,1.05),(-.64,.155,1.16)],"aaa9a2")
    forge.box(.40,.05,.853,.50,.50,.22,"93806b",.053)
    forge.ellipsoid(.40,.06,.963,.157,.168,.05,"d79253",8,3,mode="light")
    forge.pole((.35,.12,.97),(.50,-.09,.998),.025,"bf7c58",5)
    forge.box(-.35,.31,.823,.42,.075,.041,"9c6d48",.019,.19)
    assets.append(forge)
    return assets
