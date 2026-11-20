"""Terraces and vegetation that give the island distinct near, middle and far silhouettes."""
import math
import random


def models(api):
    Model, land = api.Model, api.landscape
    rocks=Model("Eichenhafen · Kalksteinbänder am Nordhang","island_relief")
    crowns=Model("Insel · Eichen, Wacholder und Hangsträucher","island_relief")
    verge=Model("Insel · Blühender Wegrand und Farne","island_relief",casts=False)
    rng=random.Random(99331)
    for x,z,span,angle in [
        (39.9,-12.6,2.9,-.29),(44.4,-13.6,3.1,-.18),(49.1,-13.6,2.5,.12),
        (51.6,-15.9,2.6,-.21),(56.5,-15.8,2.1,.11),
        (35.6,-19.0,3.4,-.29),(43.6,-19.8,3.5,.11),(55.4,-22.3,3.2,.07),
        (60.2,-15.4,2.5,-.61),(65.7,-10.9,2.7,-.48),(64.6,-8.2,2.45,-.53),
    ]:
        for row in range(2):
            for i in range(7):
                at=(i-3)*span*.26 + row*.14
                xx=x+at*math.cos(angle)
                zz=z+at*math.sin(angle)-row*.42
                ground=land.height(xx,zz)
                if ground < -.18:
                    continue
                h=rng.uniform(.39,.61)
                length=span*.32*rng.uniform(.83,1.19)
                rocks.box(xx,zz,ground+h*.29,length,.53+row*.08,h,
                          ("b2a48e","c0b29b","aa9d8d","c7b89e")[(i+row)%4],
                          .097,angle+rng.uniform(-.09,.09),mode="stone")
                rocks.obstacle_circle(xx,zz,length*.28)
                if i%2==0:
                    api.grass(verge,xx+.11,zz-.24,ground+h*.46,.22,"858f61",4)

    # More slender crowns on the ridge leave views over the roofs and the nearby market.
    for x,z,r,count in [
        (36.8,-10.6,3.0,7),(41.9,-14.3,3.0,7),(46.9,-15.2,3.1,7),(49.7,-19.4,3.9,8),
        (54.7,-21.7,3.2,8),(58.3,-16.4,3.7,7),(62.2,-11.3,2.7,6),
        (37.8,-25.0,3.7,7),(50.8,-26.3,3.8,8),(61.4,-19.6,3.7,7),
        (65.5,-7.3,2.5,5),
    ]:
        accepted=[]
        for attempt in range(count*3):
            a=rng.random()*math.tau
            radius=r*math.sqrt(rng.random())
            xx,zz=x+math.cos(a)*radius,z+math.sin(a)*radius
            if any(math.hypot(xx-u,zz-v)<1.15 for u,v in accepted):
                continue
            y=land.height(xx,zz)
            if y<.17 or land.road_distance(xx,zz)<2.65:
                continue
            accepted.append((xx,zz))
            s=rng.uniform(.62,1.25)
            height=2.04*s
            crowns.pole((xx,zz,y-.09),(xx-.11*s,zz,y+height),.115*s,
                        "806449",7,tip=.050*s)
            crowns.obstacle_circle(xx,zz,.18*s)
            if attempt%3==0:
                api.seasonal.bare_tree(crowns,xx,zz,y,s,height)
                for dx,dz,h,c in [(-.38,.09,1.63,.61),(.36,-.02,1.85,.68),(.04,-.16,2.29,.68)]:
                    api.seasonal.crown(crowns,xx+dx*s,zz+dz*s,y+h*s,c*s,c*.86*s,c*.76*s,
                                       "779353",9,4,rough=.15)
            else:
                for i in range(5):
                    fr=i*.25
                    w=(.50-.30*fr)*s
                    crowns.ellipsoid(xx+.19*s*fr,zz-.16*s*fr,y+s*(.94+i*.47),
                                     w,w*.88,s*(.60-.22*fr),
                                     ("66845b","728d5e","64835f","839563")[i%4],
                                     8,4,rough=.14,mode="leaf")
            if len(accepted)==count:
                break

    for x,z,r in [(33.1,7.85,.83),(40.55,9.87,1.1),(44.14,12.75,.79),
                  (53.26,11.16,.73),(57.6,16.9,.84),(60.7,8.25,.88),
                  (-38.2,-8.2,.82),(-44.8,-3.5,.99),(-48.2,-10.58,.91),
                  (-57.4,-10.0,1.1),(-64.2,-2.9,.96),(-70.4,-15.7,.88),
                  (12.2,73.4,.91),(6.7,76.5,1.07),(11.7,80.6,.88)]:
        for i in range(19):
            a=rng.random()*math.tau
            radius=r*math.sqrt(rng.random())
            xx,zz=x+math.cos(a)*radius,z+math.sin(a)*radius
            y=land.height(xx,zz)
            if y < -.12 or land.road_distance(xx,zz)<.87 or land.is_gathering_spot(xx,zz):
                continue
            api.grass(verge,xx,zz,y+.015,rng.uniform(.21,.42),
                      ("799458","939957","a1a65f")[i%3],5)
            if i%4==0:
                api.flowers(verge,xx,zz,y+.028,rng.uniform(.62,.96),
                            ("d7ae91","b490bc","eed4a0")[i%3])
            elif i%5==0:
                api.fern(verge,xx,zz,rng.uniform(.45,.78),y+.017)
    return [rocks,crowns,verge]
