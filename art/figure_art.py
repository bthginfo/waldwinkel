"""Costume pieces and face geometry for the Tanuki and Eichenhafen's residents."""
import math


def is_figure(asset):
    return asset.startswith("npc_") or asset in ("gardener", "arm", "leg", "boot", "tail")


def eye(Model, point, label, asset, x, z, y, scale=1.):
    mesh=Model(f"{label}_eye",asset)
    mesh.pivot=point(x,z,y)
    mesh.ellipsoid(x,z,y,.035*scale,.022*scale,.047*scale,"35291f",10,5,mode="fur")
    mesh.ellipsoid(x-.009*scale,z+.018*scale,y+.016*scale,
                   .010*scale,.006*scale,.012*scale,"fff2da",7,3,mode="raw")
    mesh.ellipsoid(x+.011*scale,z+.019*scale,y-.014*scale,
                   .006*scale,.004*scale,.006*scale,"c8a882",7,3,mode="raw")
    return mesh


def button(model,x,z,y,color,r=.023):
    model.ellipsoid(x,z,y,r,.010,r,color,8,4)
    model.ellipsoid(x,z+.009,y,.005,.004,.005,"76583d",6,3)


def hem(model, points, width, color):
    for a,b in zip(points,points[1:]):
        model.pole(a,b,width,color,6)


def hero_clothes(model, arm, leg, boot):
    model.ellipsoid(0,-.005,.647,.254,.158,.282,"617f75",14,6,mode="cloth")
    model.ellipsoid(0,.027,.747,.231,.145,.202,"e2bc83",12,5,mode="cloth")
    model.ellipsoid(0,-.011,.456,.213,.132,.118,"456359",12,5,mode="cloth")
    front=[(-.159,.158,.520),(.159,.158,.520),(.143,.182,.753),(.112,.151,.809),
           (-.113,.151,.809),(-.144,.182,.753)]
    model.face(front,"41776e","cloth")
    hem(model,front+[front[0]],.011,"72a093")
    for side in (-1,1):
        strap=[(side*.129,-.104,.796),(side*.135,-.016,.930),
               (side*.121,.093,.915),(side*.108,.171,.796)]
        hem(model,strap,.028,"54867b")
        button(model,side*.108,.186,.798,"dcb368")
        for n in range(4):
            y=.558+.028*n
            model.box(side*(.148-.002*n),.169,y,.012,.008,.016,"a7b7a1",.003)
    pocket=[(-.092,.181,.607),(.092,.181,.607),(.080,.189,.548),
            (0,.194,.519),(-.080,.189,.548)]
    model.face(pocket,"689389","cloth")
    hem(model,pocket+[pocket[0]],.008,"a5b597")
    hem(model,[(-.10,.186,.613),(.103,.186,.613)],.012,"83ab97")
    model.ellipsoid(0,.007,.934,.181,.142,.073,"bc6556",12,4,mode="cloth")
    model.face([(-.152,.123,.936),(.107,.151,.951),(.140,.174,.865),
                (.012,.19,.787),(-.086,.163,.839)],"ca7d62","cloth")
    hem(model,[(-.152,.126,.936),(-.086,.167,.839),(.012,.193,.787)],.010,"e1ac85")
    model.pole((-.136,-.097,.930),(-.329,-.145,.803),.047,"bc6d58",7,tip=.020)
    model.pole((-.131,-.094,.925),(-.256,-.207,.759),.041,"ce8467",7,tip=.014)
    model.box(0,-.173,.685,.319,.176,.329,"a17559",.063)
    model.box(0,-.272,.749,.27,.046,.184,"bc926a",.035)
    hem(model,[(-.130,-.297,.768),(0,-.300,.696),(.130,-.297,.768)],.008,"e0b689")
    button(model,0,-.305,.707,"d6b170")
    model.pole((.18,-.063,.772),(.18,-.215,.520),.024,"795b4b",7)
    arm.ellipsoid(0,0,-.085,.094,.086,.137,"dfbc88",10,5,mode="cloth")
    arm.ellipsoid(0,.005,-.163,.080,.076,.041,"b69d75",10,4,mode="cloth")
    arm.ellipsoid(0,.013,-.238,.072,.065,.092,"8b634c",10,5,mode="fur")
    for x in [-.039,0.,.039]:
        arm.ellipsoid(x,.05,-.268,.025,.031,.041,"a8795b",7,4,mode="fur")
    leg.ellipsoid(0,0,-.097,.097,.115,.147,"52766c",10,5,mode="cloth")
    kneepad=[(-.044,.096,-.118),(.043,.096,-.118),(.048,.085,-.177),
             (0,.093,-.198),(-.046,.087,-.176)]
    leg.face(kneepad,"759486","cloth")
    hem(leg,kneepad+[kneepad[0]],.005,"96ad94")
    boot.box(0,.044,-.052,.178,.239,.131,"926247",.039)
    boot.box(0,.011,.003,.171,.163,.055,"ae8061",.019)
    boot.box(0,.051,-.104,.186,.245,.049,"5e493b",.017)
    for z in (.060,.099):
        boot.pole((-.048,z,-.005),(.048,z,-.005),.008,"c7aa82",6)


def hero_face(Model,point,model):
    eyes=[]
    model.ellipsoid(0,-.024,1.213,.309,.240,.270,"926348",16,8,mode="fur")
    model.ellipsoid(0,.087,1.160,.284,.224,.222,"c79369",16,8,mode="fur")
    def surface(x,h):
        skull=-.024+.240*math.sqrt(max(0.,1-(x/.309)**2-((h-1.213)/.270)**2))
        face=.087+.224*math.sqrt(max(0.,1-(x/.284)**2-((h-1.160)/.222)**2))
        return (x,max(skull,face)+.006,h)
    blaze=[(-.020,1.367),(.031,1.338),(.044,1.273),(.029,1.220),(0.,1.199),
           (-.034,1.227),(-.055,1.283),(-.043,1.337)]
    inner=[surface(x*.52,1.276+(h-1.276)*.52) for x,h in blaze]
    outer=[surface(x,h) for x,h in blaze]
    faces=[(i,(i+1)%8,(i+1)%8+8,i+8) for i in range(8)]
    faces += [(i+8,(i+1)%8+8,16) for i in range(8)]
    model.add(outer+inner+[surface(0.,1.276)],faces,
              ["ddae84"]*8+["e2b98e"]*8,"fur")
    for side,label in [(-1,"l"),(1,"r")]:
        model.ellipsoid(side*.236,-.031,1.405,.092,.085,.103,"76513d",12,6,mode="fur")
        model.ellipsoid(side*.244,.024,1.407,.060,.040,.070,"c78b76",10,5,mode="fur")
        model.ellipsoid(side*.124,.240,1.208,.124,.079,.103,"634639",12,6,
                        angle=side*.39,mode="fur")
        model.ellipsoid(side*.146,.229,1.080,.122,.090,.082,"e9c69a",12,6,mode="fur")
        model.ellipsoid(side*.119,.274,1.302,.062,.018,.019,"a27658",9,4,mode="fur")
        for n in range(3):
            xx=side*(.102+n*.026)
            model.ellipsoid(xx,.316-.010*n,1.088+.009*(n%2),.008,.005,.008,"ae8163",6,3)
        eyes.append(eye(Model,point,f"gardener_{label}",model.asset,side*.092,.305,1.226,1.11))
    model.ellipsoid(0,.293,1.103,.121,.094,.083,"f0d1a6",12,6,mode="fur")
    model.ellipsoid(0,.376,1.143,.049,.029,.029,"563a30",10,5,mode="fur")
    model.ellipsoid(-.016,.402,1.153,.012,.004,.006,"bb926c",6,3,mode="raw")
    hem(model,[(-.052,.361,1.065),(0,.386,1.052),(.052,.361,1.065)],.007,"835b44")
    return eyes


def hero_hat(model):
    sides=20
    rings=[]
    # The upturned front rim reveals the eyes from the play camera.
    for h,r in [(1.432,.384),(1.468,.414),(1.483,.348),(1.515,.272),
                (1.639,.238),(1.684,.163)]:
        rings.append([(math.cos(a)*r,math.sin(a)*r*.85-.030,
                       h+.047*math.sin(a)+.019*math.cos(2*a+.72))
                      for i in range(sides) for a in [math.tau*i/sides]])
    model.rings(rings,["bb9053"]*20+["edc785"]*20+["d3a866"]*20+
                ["e2ba76"]*20+["eac482"]*20+["c09a62","e9c788"],mode="cloth")
    model.pole((0,-.031,1.511),(0,-.031,1.564),.277,"5c8074",16,tip=.256)
    for r,h in [( .358,1.484),(.389,1.477)]:
        braid=[]
        for i in range(sides+1):
            a=i*math.tau/sides
            braid.append((r*math.cos(a),r*.85*math.sin(a)-.030,
                          h+.047*math.sin(a)+.019*math.cos(2*a+.72)))
        hem(model,braid,.008,"c59a64")
    button(model,.07,.230,1.552,"e3bf73",.035)
    model.pole((.224,.102,1.537),(.324,.055,1.704),.009,"658368",7,tip=.004)
    model.face([(.254,.091,1.591),(.318,.014,1.640),(.336,.031,1.756),
                (.300,.094,1.678)],"83a372","leaf")


def hero_tail(tail):
    sections=[(0.,0.,0.,.078,.071),(.026,-.126,.034,.116,.096),
              (.107,-.237,.078,.164,.131),(.174,-.374,.106,.175,.148),
              (.229,-.505,.124,.149,.125),(.253,-.636,.126,.091,.078),
              (.228,-.719,.125,.016,.024)]
    rings=[]
    for x,z,h,rx,ry in sections:
        rings.append([(x+math.cos(a)*rx,z,h+math.sin(a)*ry)
                      for i in range(12) for a in [i*math.tau/12]])
    colors=[]
    for i,tone in enumerate(("986748","ad7955","ac7b56","9d6b4a","875b42","77533d")):
        colors.extend(tone if j<6 else ("ac805d" if i<5 else "875e46") for j in range(12))
    tail.rings(rings,colors+["936d4b","76543c"],mode="fur")


def apron(model, primary, rim, rank=0):
    shape=[(-.130,.160,.834),(.132,.160,.834),(.173,.188,.464),
           (.102,.192,.423),(-.119,.192,.425),(-.181,.174,.475)]
    model.face(shape,primary,"cloth")
    hem(model,shape+[shape[0]],.010,rim)
    model.pole((-.167,.18,.592),(.164,.184,.593),.018,rim,7)
    model.box(-.055,.200,.52,.127,.046,.1,primary,.019,mode="cloth")
    if rank:
        model.box(.098,.200,.531,.072,.028,.136,rim,.009)


def cap(model, top, brim):
    model.ellipsoid(0,-.042,1.432,.248,.177,.101,top,14,5,mode="cloth")
    model.ellipsoid(0,.186,1.395,.199,.113,.031,brim,12,4,mode="cloth")
    model.pole((0,-.036,1.404),(0,-.036,1.445),.244,brim,14,tip=.229)
    button(model,0,.199,1.464,"e0bc76",.028)


def shawl(model, tint, trim):
    model.ellipsoid(0,-.015,.915,.224,.148,.076,tint,13,4,mode="cloth")
    shape=[(-.193,.101,.926),(.170,.136,.929),(.136,.185,.820),
           (-.030,.194,.726),(-.138,.179,.817)]
    model.face(shape,tint,"cloth")
    hem(model,shape[1:]+shape[:1],.009,trim)
    button(model,.048,.203,.824,trim)


def resident_details(model, arms, asset):
    if asset=="npc_lina":
        apron(model,"a69155","dfbd78")
        model.ellipsoid(0,-.007,1.361,.189,.120,.038,"8f9d63",12,4,mode="cloth")
        model.pole((.13,.087,1.373),(.22,.061,1.438),.018,"adbc83",6,tip=.005)
        model.face([(.172,.089,1.385),(.220,.075,1.463),(.245,.024,1.489),
                    (.220,.038,1.410)],"b1b56f","leaf")
    elif asset=="npc_miro":
        shawl(model,"a94e3e","d6a167")
        model.box(.241,-.058,.592,.168,.151,.241,"8f654d",.043)
        model.box(.232,.025,.634,.145,.065,.119,"c28c62",.028)
        button(model,.244,.066,.621,"e4b268")
    elif asset=="npc_mara":
        apron(model,"e0d4b3","b29373",1)
        model.ellipsoid(0,-.033,1.360,.205,.162,.060,"978492",13,5,mode="cloth")
        model.ellipsoid(0,-.061,1.406,.154,.118,.069,"b8a2a7",12,5,mode="cloth")
    elif asset=="npc_fiete":
        cap(model,"547e85","426471")
        shawl(model,"c69057","e0ba83")
        for side,arm in arms.items():
            arm.ellipsoid(side*.245,.012,.577,.083,.085,.039,"b3ad82",10,4,mode="cloth")
    elif asset=="npc_bo":
        cap(model,"516c73","3d5760")
        shawl(model,"576f71","92a28a")
        for side in (-1,1):
            button(model,side*.128,.173,.693,"dcb475")
    elif asset=="npc_juna":
        shawl(model,"878658","c3ad72")
        model.ellipsoid(0,-.069,1.39,.207,.177,.083,"94805a",12,6,mode="cloth")
        model.ellipsoid(0,.120,1.367,.226,.212,.027,"b49564",14,4,mode="cloth")
        model.pole((-.085,.081,1.394),(-.285,.003,1.518),.021,"b9835d",6,tip=.009)
    elif asset=="npc_runa":
        apron(model,"77513e","b38256",1)
        for dx in [-.101,.101]:
            model.ellipsoid(dx,.152,1.372,.088,.046,.063,"755c4d",10,5)
            model.ellipsoid(dx,.189,1.373,.058,.024,.041,"a88c60",10,5)
        hem(model,[(-.164,.120,1.371),(0,.195,1.381),(.164,.120,1.371)],.014,"725b47")
    elif asset=="npc_nele":
        shawl(model,"c3945f","ead098")
        model.box(-.194,.108,.592,.166,.119,.198,"998174",.037)
        for a,b in [((-.247,.165,.688),(-.206,.174,.827)),((-.196,.172,.673),(-.154,.166,.815))]:
            model.pole(a,b,.013,"b7a46a",7,tip=.008)
    elif asset=="npc_edda":
        shawl(model,"527a73","a2b6a1")
        model.ellipsoid(0,-.119,1.169,.288,.181,.228,"527970",13,6,mode="cloth")
        model.box(.224,-.079,.594,.153,.133,.238,"84745d",.029)
        model.pole((.259,.022,.536),(.259,.022,.771),.040,"d9c8a1",8)
