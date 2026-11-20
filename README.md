# Waldwinkel 1.6.0

Ein Cozy-3D-Spiel aus Bevy und Rust. Der Hof und sechs Ausflugsgebiete liegen in einer zusammenhängenden Landschaft. Wege und Brücken verbinden Farnwald, Muschelbucht, Eichenhafen, Kupferklamm, Windweiden und Nebelmoor. Der Tanuki, die neun Figuren und die 3D-Modelle stammen aus dem beigefügten Blender-Projekt.

![Die verbundene Insel mit Gebirgspässen und Flusstälern](docs/13-insel.jpg)

## Installieren

- Lade die signierte APK im Browser herunter und öffne die Datei auf deinem Android-Gerät unter **Downloads**. [Android-APK](https://github.com/bthginfo/waldwinkel/releases/latest/download/Waldwinkel-Android.apk) · [Alle Versionen](https://github.com/bthginfo/waldwinkel/releases)
- Voraussetzungen sind Android 8 oder neuer, Vulkan und ein 64-Bit-System (ARM64 oder x86-64). Das Spiel läuft offline. Installiere das Update über die bisherige Version, damit deine Spielstände erhalten bleiben.

## Neu in 1.6.0

- **Reihenarbeit.** Tippe am Hof auf **Werkzeug** oder drücke **R** auf der Tastatur beziehungsweise **R3** am Controller, um Hacke, Saatgut, Gießkanne oder Erntekorb auszuwählen. Die Markierungen zeigen den nächsten Schwung und das Folgebeet. Kupferwerkzeuge bearbeiten bis zu zwei passende Beete, Silberwerkzeuge bis zu drei.
- **Baupläne und Stellprobe.** Hefte Möbel, Bauprojekte, Werkzeugstufen oder Werkstoffe an. Unter **Journal → Aufträge** findest du fehlende Zutaten und ihre Fundorte. Die Karte zeigt ein passendes Ziel. Mit **Im Haus ansehen** kannst du vor dem Bau durch die Stellplätze wechseln und ein Möbel in deinem aktuellen Grundriss ausprobieren.
- **Maras Abendmarkt.** Nach Maras Küchenauftrag hilfst du ihr mit Brettern, Stein und einer ersten Kostprobe. Lina und Miro machen sich vor der Eröffnung mit ihr auf den Weg zum Stand. Nach der Quest kannst du täglich von 18:00 bis 21:30 Uhr vier Gerichte mit jeweils einem Stück Tagesvorrat kaufen.
- **Proviant im Duell.** Jede Mahlzeit hat eine Grundwirkung und ist gegen bestimmte Hüter besonders hilfreich. Marmelade verstärkt den Schild gegen Nebelgegner. Tomateneintopf hilft beim nächsten Treffer gegen die Rüstung der Tonwächter. Vor einer Wacht und im Proviantmenü steht, welches Gericht zum Gegner passt.
- **Eichenhafen und Bewohner.** Neue Giebel, Walmdächer, Schornsteine und der offene Glockenturm unterscheiden die Häuser. Am Abend leuchten die Laternen auf dem Pflaster. An ihren Stationen arbeiten die Bewohner mit Kochlöffel, Gießkanne, Päckchen, Notizbuch oder Hammer.

[Abendmarkt](docs/160-abendmarkt.jpg) · [Reihenarbeit](docs/160-beete.jpg) · [Möbelvorschau](docs/160-moebel.jpg)

## Neu in 1.5.1

- **Steuerung.** Der Tanuki läuft feinfühliger bei kleinem Stickweg, bremst schneller und reagiert direkter auf Richtungswechsel. Die Kamera folgt gleichmäßiger. Ecken am Boden markieren das gewählte Ziel. Bei dicht stehenden Objekten entscheidet die sichtbare Tiefe.
- **Aufträge und Bewohner.** Ein gelbes **!** zeigt einen neuen Auftrag. Ein grünes **?** zeigt, dass du einen Auftrag abgeben kannst. Die Zeichen stehen über den Figuren und neben ihrem Namen auf der Karte. Tippe auf einen Namen, um dorthin zu laufen. Die Karte zeigt die aktuellen Standorte aller Bewohner erreichbarer Gebiete.
- **Tagesabläufe.** Lina kümmert sich um die Pflanzkübel, Miro geht zum Hafentor und Juna sieht am Brunnen nach den Wegzeichen. Auch die Bewohner der Außenregionen wechseln den Ort. Unter **Journal → Dorf** steht, wo sie gerade sind.
- **Eichenhafen.** Markttücher, Ladenzeichen, bepflanzte Wegkanten und stärker abgestufte Schatten prägen den Platz. Vögel, Schmetterlinge und Käfer tauchen bei passendem Wetter auf. In warmen Nächten sind Glühwürmchen zu sehen.
- **Ton.** Klavier und Harfe reagieren auf Ort, Tageszeit und Gespräche. Beim Fällen hörst du Axtschlag, Splittern und Aufprall. Beete, Fertigung und Kampftalente haben eigene Rückmeldungen. Im Pausenmenü wechselt **Ton** zwischen **Ein**, **Ohne Musik** und **Aus**. [Aufnahmen und Audiodesign](docs/audio.md)

[Eichenhafen im Spiel](docs/151-eichenhafen.jpg) · [Bewohner und Aufträge auf der Karte](docs/151-karte.jpg) · [Baumfällen mit Spielton](https://github.com/bthginfo/waldwinkel/releases/download/v1.5.1/Waldwinkel-Baumfaellen.mp4)

## Neu in 1.5

- **Eine Kampagne bis zum Lichtfest.** Das Journal zeigt Kapitel, nächstes Ziel und fehlende Materialien. Nach dem ersten Beet schickt Bo dich mit Baumaterial zur Farnwacht. Fischersteg, regionale Wachten und Klangsteine führen anschließend bis zur Hafenleuchte. Die 25 Dämmer-Ebenen sind als Expedition neben der Kampagne spielbar.
- **Kampfstile.** Der erste Wachtsieg schaltet am Kartenbinder Rindenwacht, Windspur und Herdfeuer frei. Schutz und Konter, kurze Kartenfolgen und Proviant eröffnen verschiedene Spielweisen. Die ersten Duelle der drei bewachten Wachten haben eigene Angriffsfolgen.
- **Wiederaufbau.** Vollendete Wachten öffnen regionale Wegglocken. Bo geht nach der Wiedereröffnung regelmäßig bis ans Ende des Fischerstegs. Das Dach der Farnwacht, die Hofveranda und das Modell der Hafenleuchte wurden in Blender überarbeitet. Eigene Klangkulissen erklingen am Hof, im Farnwald und am geöffneten Hafen.
- **Sicherungen auf Android.** Öffne im Pausenmenü **Spielstand exportieren**, um eine Datei über die Android-Dateiauswahl zu speichern. Im Hauptmenü kannst du sie mit **Spielstand importieren** in einen freien Platz laden. Bei einer unlesbaren Hauptdatei erscheint **Wiederherstellen**, sobald eine gültige lokale Sicherung verfügbar ist.
- **Android 16.** Der Build zielt auf API 36. Der Randabstand des HUD berücksichtigt Displayausschnitte. Wenn Android einen Controller als getrennt meldet, werden gehaltene Tasten und Stickpositionen zurückgesetzt.

[Hof auf Android 16](docs/15-hof.jpg) · [Kapitel und Journal](docs/15-kampagne.jpg) · [Farnwacht](docs/15-farnwacht.jpg) · [Lichtfest](docs/15-lichtfest.jpg) · [Xbox-Menüfokus](docs/15-xbox.jpg) · [Importierte Sicherung](docs/15-sicherung.jpg)

## Neu in 1.4

- **Die fünf Wegwachten.** Öffne im Journal den Reiter **Wegwachten** oder frag Bo. Beginne in der Farnwacht. Folge den Wegzeichen, fertige Material und wecke die Lichter in Farnwald, Muschelbucht, Kupferklamm, Windweiden und Nebelmoor. Bring die fünf Geschichten zu Juna.
- **Wachtprüfungen.** Die Farnwacht bietet schon früh ein Kartenduell. Drei Wachten haben je vier ansteigende Prüfungen mit eigenen Regeln und Belohnungen. In einem Kampf kannst du einmal ein vorbereitetes Gericht einsetzen.
- **Hafenaufträge.** Am Auftragsbrett kannst du unter anderem Ernten, Fischen und Erzabbau als Auftrag annehmen. Aushänge wechseln alle drei Tage. Ein angenommener Auftrag bleibt bis zum Einlösen oder Aufgeben aktiv.
- **Fertigung.** Werkbank, Schmiedetisch und der neue Webrahmen stellen Fichtenbretter, Muschelglas, Kupfernieten, Segeltuch und Laternenöl her. Wähle ein, drei oder fünf Arbeitsgänge. Neue Rezepte lernst du an den Wegzeichen.
- **Xbox auf dem Tablet.** Linker Stick bewegt die Figur. Links und rechts am Steuerkreuz wählen nahe Ziele. **A** bestätigt. **X** benutzt das gewählte Ziel oder greift direkt auf das passende Ziel in der Nähe zu. **B** geht zurück. **Y** öffnet die Karte oder beendet im Duell den Zug.

[Stand und Qualitätsprüfung](docs/stand-1.4.md) · [Farnwacht](docs/14-farnwacht.jpg) · [Weitere Wegwachten](docs/14-wegwachten.jpg) · [Fertigung](docs/14-materialien.jpg) · [Proviant im Duell](docs/14-proviant.jpg) · [Szenenausschnitt mit Juna](docs/14-finale.jpg) · [Xbox-Fokus im Tabletformat](docs/14-xbox.jpg) · [Bildreferenz](art/references/wegwachten.jpg)

## Seit 1.3

- **Eine zusammenhängende Insel.** Bergkämme rahmen Farnwald, Hof und Kupferklamm. Die Wege ins Nebelmoor und in die Klamm führen durch Pässe, die Windweiden sind über die neue Weidebrücke erreichbar. Felsgruppen und Wälder markieren die Übergänge.
- **Dein erster Morgen.** Ein neuer Spielstand beginnt mit einer kurzen Kamerafahrt und Untertiteln. Danach führt dein erster Auftrag vom Hacken, Säen und Gießen bis zum Gespräch mit Bo. Über **Überspringen** kommst du direkt zum Auftrag.
- **Der alte Fischersteg.** Sprich mit Bo oder lies den Aushang am Bautisch in Eichenhafen. Treibholz vom Hafenufer und zwei Holz genügen für den ersten Anleger. Danach stellst du Kupferbeschläge an der Werkbank her und bringst Baumaterial und etwas aus deiner Ernte. Zur Wiedereröffnung gibt es eine zweite Kamerafahrt, den Zugang zur Seeseite mit Hafenheringen und den Bauplan für die Hafenlaterne.
- **Dorf.** Bo und Fiete laufen zwischen Rathaus, Markt und Hafen. Unter **Journal → Dorf** siehst du ihren aktuellen Aufenthaltsort und kannst ihn auf der Karte markieren. Gespräche ändern sich mit den Jahreszeiten und dem Ausbau des Stegs.
- **Tasche.** Gegenstände, Material, Saatgut und Ernte erscheinen mit eigenen Illustrationen und Stückzahl. Im Garten lassen sich die sichtbaren Blätter antippen, um das dazugehörige Beet zu wählen.
- **Wege und Ton.** Die Figur bremst am Ziel ab und rundet freie Kurven. Wenn die Materialien und Stadtaufträge bereit sind, öffnet die Aktion das Tor und führt dich hindurch. Am Wasser hörst du Wellen, bei Regen leises Trommeln auf Holz. Schritte klingen auf Dielen, Gras und Schnee unterschiedlich. Unter **Menü → Ton** lässt sich die Ausgabe umschalten.
- **Abbau.** Zusätzlich zu den Rohstoffbäumen am Hof kannst du 487 Wildbäume fällen und 172 weitere Findlinge abbauen. Beim Axtschlag fliegen Späne, und der Baum kippt. Felsen und Erzadern brechen beim Spitzhackenschlag in Stücke. Gefällte Bäume wachsen nach drei Spieltagen nach.
- **Hofwerkbank und Baupläne.** Setze die Werkbank vor der Hütte mit zwei Holz und einem Stein instand. Damit lernst du die ersten drei Möbelbaupläne. Weitere Pläne kaufst du bei Bo oder am Bauplanstand, findest sie unterwegs oder verdienst sie mit Aufträgen.
- **Werkzeugstufen.** Axt, Spitzhacke, Hacke und Gießkanne lassen sich auf Kupfer und Silber verbessern. Baue für Silber den Schmiedetisch. Axt und Spitzhacke erhöhen die Ausbeute. Hacke und Gießkanne gewinnen Reichweite.
- **Haus und Tierhaltung.** Die Baupläne der Gartenstube und Lesekammer öffnen zwei begehbare Anbauten. Pro Raum kommen vier Möbelplätze dazu. Kaufe für den Hühnerstall oder die Schafscheune zuerst den Bauplan und baue das Gebäude auf dem Hof. Bei einem bestehenden Spielstand ist die Hofwerkbank bereits instand gesetzt.

[Einführung](docs/13-ankunft.jpg) · [Tasche auf dem Handy](docs/13-tasche.jpg) · [Tasche auf dem Tablet](docs/13-tablet.jpg) · [Fischersteg](docs/13-steg.jpg) · [Wiedereröffnung](docs/13-eroeffnung.jpg) · [Baumfällen im Spiel](docs/13-abbau.mp4) · [Werkzeugausbau](docs/13-werkbank.jpg) · [Haus mit zwei Anbauten](docs/13-haus.jpg)

## Seit 1.2

- **Vier Jahreszeiten.** Alle sieben Spieltage wechseln Laubbäume und Wiesen ihr Aussehen. Im Winter tragen Nadelbäume und Dächer Schnee. Die Obstwiese ruht bis zum Frühling. Gegossene Gartenbeete wachsen weiterhin.
- **Kalender und Sammeln.** Unter **Journal → Kalender** findest du den nächsten Wechsel, Hinweise zu zwölf saisonalen Fundorten und deine Sammlung. Blütenzweige, Sonnenmuscheln, Bernsteinnüsse und Frostbeeren kannst du am Dorftisch abgeben oder verkaufen. Ein abgeernteter Fundort füllt sich nach zwei Tagen derselben Jahreszeit wieder.
- **Dorftisch in Eichenhafen.** Blütentausch, Sommertafel, Laublichter und Frostfrühstück finden jeweils an Tag 5–7 der Jahreszeit von 06 bis 20 Uhr statt. Zwei passende Funde bringen pro Treffen 18 Münzen und drei Samen. Für alle vier Treffen erhältst du den Bauplan fürs Jahreszeitenregal.
- **Licht und Wetter.** Abends gehen die Fenster und Lichter am Hof und in der Stadt an. Nebel hängt in der Ferne, Regen dunkelt Wiesen ab und trifft mit kleinen Ringen auf den Boden. Böen bewegen Wolken, Niederschlag und die Mühle.
- **Bewegung.** Der Tanuki beschleunigt und bremst schneller. Beim Anlaufen gleitet er an Ecken entlang. Figuren und der Dorftisch sind feste Hindernisse.

[Kalender](docs/12-kalender.jpg) · [Blütentausch](docs/12-dorftisch.jpg) · [Schneefall](docs/12-schnee.jpg) · [Nacht am Hof](docs/12-nacht.jpg)

## Seit 1.1

- **Das Licht der Wegglocken.** Sprich mit Juna in Eichenhafen, stimme die drei Klangsteine in Klamm, Weide und Moor und entzünde nach der letzten Dämmerglocke die Hafenleuchte.
- **Wege mit eigenen Orten.** Ein Erzgleis und Werkplätze in der Kupferklamm, Flachsfelder und ein drehendes Mühlrad auf den Windweiden, Weiden und Seerosen im Nebelmoor. Die Landschaftsmodelle sind in sichtbare Bereiche unterteilt.
- **Journal und Bedienung.** Das Journal zeigt Geschichte und Aufträge, markiert ein Ziel auf der Karte und nimmt Hofaufgaben ab. Die Werkzeugleiste fasst Karte, Journal, Saatgut und Tasche zusammen. Unter **Menü → Darstellung** kannst du die Oberfläche vergrößern.
- **Kampf.** Die fünf Kapitel haben eigene Hüter und Raumfarben. Manche Gegner heilen sich oder treffen zweimal. Bei einem aufgeladenen Angriff kannst du zuerst die Rüstung brechen und damit den Schlag unterbrechen. Lebensbalken, Kraft und Zugstapel haben eigene Anzeigen.

[Journal](docs/11-journal.png) · [Weltkarte](docs/11-karte.png) · [Kampf](docs/11-kampf.png) · [Kupferklamm](docs/11-klamm.png) · [Stand und Vergleich zu 1.0](docs/spielstand-1.1.md)

## Dämmergänge und Kartenkämpfe

- **Zugang.** Hilf den Leuten in Eichenhafen, bis der Moorweg freigeschaltet ist. Erfülle anschließend Eddas erste zwei Aufträge und wähle den Steinbogen im Nebelmoor.
- **25 Ebenen.** Die Gänge werden prozedural aufgebaut. Der Hüter jeder Ebene öffnet den Abstieg. Alle fünf Ebenen erreichst du nach dem Sieg eine Glockenrast. Kehre am Steinbogen wahlweise zu einer freigeschalteten Rast zurück oder starte wieder am Eingang. Ausruhen füllt deine Lebenspunkte.
- **Rundenkämpfe.** Jede Karte zeigt ihre Kraftkosten und Wirkung. Der Hüter zeigt vor deinem Zug, ob er angreift oder sich schützt. Siege bringen Münzen, Talentpunkte und Beute für die Verkaufstruhe.
- **Kartenbinder.** Die Sammlung umfasst 18 Karten. Du findest Karten in der Welt, erhältst sie für Aufträge und Kämpfe oder kaufst sie bei Runa. Dein Kampfdeck enthält zwölf Karten mit bis zu zwei Kopien pro Karte. Änderungen speicherst du am Binder in deiner Hütte.
- **Ausrüstung und Talente.** Runa verkauft Kampfausrüstung. An deiner Werkbank kannst du sie aus Holz und Erz sowie später aus Silber, Kristall und Quarz herstellen. Am Kartenbinder investierst du Talentpunkte in Gesundheit, Widerstand, mehr Karten pro Zug und mehr Kraft.

[Kartenkampf](docs/kartenkampf.png) · [Kartenbinder](docs/kartenbinder.png) · [Glockenrast](docs/glockenrast.png) · [Bildreferenz](art/references/daemmergaenge.png)

## Seit 0.9.0

- **Kupferklamm.** Nach einem Stadtauftrag kannst du am östlichen Steg den Weg freischalten. Runa erzählt dir an der Schmiede von der Klamm. An den hellen Felswänden findest du Quarz.
- **Windweiden.** Nach zwei Stadtaufträgen bittet dich die Wache am südlichen Pfad um Baumaterial und Wolle. Nele hat an der Mühle Aufträge für dich. Auf den Wiesen wächst Flachs.
- **Nebelmoor.** Nach drei Stadtaufträgen kannst du den westlichen Moorweg mit Klammquarz und Holz wieder öffnen. Folge den Stegen zur Kräuterkundigen Edda und sammle Nebelminze.
- **Schnellreise.** Jede der sieben Wegglocken hat einen eigenen Reparaturauftrag. An einer fertigen Glocke kannst du eine andere bereits reparierte Glocke als Ziel wählen. Über die Weltkarte kannst du die Regionen zu Fuß ansteuern.

[Kupferklamm](docs/kupferklamm.png) · [Windweiden](docs/windweiden.png) · [Nebelmoor](docs/nebelmoor.png) · [Weltkarte](docs/weltkarte.png)

## Seit 0.8.0

- **Tierhaltung am Hof.** Baue am markierten Platz einen Hühnerstall oder eine Schafscheune. Danach kannst du ein Tier aufnehmen, füttern und streicheln. Gut versorgte Tiere liefern Eier oder Wolle. Zutrauen erhöht den Ertrag.
- **Einrichten.** An der Werkbank gibt es 18 Möbel. Kaufe Baupläne am Markt, finde sie unterwegs oder verdiene sie mit Stadtaufträgen. Sobald das Möbel gebaut ist, wähle es über **Einrichten** und tippe auf einen freien Platz in der Hütte.
- **Hofküche.** Sechs Rezepte nutzen Gemüse, Obst, Pilze, Nüsse und Eier. Beim Kochen triffst du mit dem Rühren den grünen Bereich. Beim Einmachen gibst du Zutaten in der angezeigten Reihenfolge hinzu. Für die Handmühle wechselst du zwischen links und rechts. Fertige Speisen und Mehl können in die Verkaufstruhe.
- **Nachwachsende Kiefern.** Fällbare Kiefern wachsen nach drei Spieltagen nach.

[Hofküche im Spiel](docs/hofkueche.png) · [Hühnerstall](docs/huehnerstall.png)

## Seit 0.7.0

- **Tagesablauf.** Der Tag beginnt um 06:00. Eine reale Sekunde entspricht einer Spielminute. Ab 19:00 wird es dämmrig, der Markt schließt um 20:00. Ab 22:00 ruhen Feldarbeit, Fischfang, Holz- und Erzabbau. Du kannst weiterlaufen, nach Hause gehen und in der Verkaufstruhe verkaufen. Schlafen im Bett beginnt den nächsten Tag.
- **Anbau.** Tippe auf ein Beet und dann auf seine Aktion. Der Tanuki geht selbst in Reichweite und legt das Beet an, sät, gießt oder erntet. Über **Saatgut** wählst du direkt zwischen sechs Kulturen. Ein eigenes Symbol und die Gießkanne zeigen, wann ein Beet Wasser braucht.
- **Saatgut und Qualität.** Samen gibt es beim Saatgutstand, in Farnwald und Muschelbucht, als Auftragsbelohnung und bei 30 % der Ernten. Gedüngte Beete liefern bei der nächsten Ernte feine oder prächtige Ware. Ein Pack mit zwei Einheiten Dünger kostet am Saatgutstand vier Münzen.
- **Verkaufstruhe.** Öffne die Holzkiste neben dem Hofweg und wähle Ware und Qualität. Du kannst ein Stück oder den ausgewählten Stapel direkt verkaufen.

[Saatguttasche im Android-Spiel](docs/saatguttasche.png) · [Weltkarte](docs/weltkarte.png)

## Seit 0.6.0

- **Haus.** Tippe das Dach, die Fassade oder die Veranda an und drücke **Eintreten**. Der Tanuki läuft zur Tür. Drinnen findest du **Hinausgehen** an der Treppe.
- **Bewegung.** Das Lauftempo ist um 40 % gestiegen. Der Touch-Stick erlaubt langsames Gehen mit kurzem Ausschlag. Beim Anlaufen, Drehen und Bremsen geht der Tanuki fließend in die nächste Bewegung über. Häuser, Zäune, Baumstämme, Felsen und Möbel blockieren den Weg.
- **Axt und Spitzhacke.** Markierte Kiefern, große Steinblöcke und Erzadern zeigen die passende Aktion. Einmal drücken genügt. Der Tanuki sucht eine Stelle neben dem Ziel, benutzt sein Werkzeug und legt die Ausbeute in die Tasche. Zwei fällbare Kiefern und ein zusätzlicher Steinblock liegen jetzt näher am Hof.

[Innenansicht der Holzhütte](docs/holzhuette.png) · [Weltkarte](docs/weltkarte.png)

## Spielstände

Im Startmenü kannst du auf bis zu drei getrennten Plätzen ein **Neues Spiel** beginnen oder mit **Fortsetzen** in einen vorhandenen Hof zurückkehren. Speicherdaten liegen lokal auf dem Gerät. Anbau, Sammelfunde, Handel und Aufträge werden bei der Aktion gespeichert. Der Aufenthaltsort wird beim Wechsel zwischen Innen- und Außenbereichen, beim Erreichen eines neuen Landschaftsbereichs und nach Bewegung automatisch gespeichert.

Über **Menü → Jetzt speichern** kannst du den aktuellen Aufenthaltsort und Fortschritt sofort speichern. **Speichern & Hauptmenü** führt zurück zur Spielstandauswahl.

## Spielabläufe

| Ort | So funktioniert es |
| :-- | :-- |
| Außenwelt | Links ist der Touch-Stick. Pinch oder **−/+** ändert die Zoomstufe. Auf der **Weltkarte** siehst du die zusammenhängende Landschaft. |
| Wege | Laufe selbst oder tippe auf begehbaren Boden. Ziele auf der **Weltkarte** kannst du direkt anwählen. Mit dem Stick kannst du unterwegs übernehmen. |
| Farnwald | Sammle Pilze und Haselnüsse auf beiden Seiten des Baches. Drei Holzbrücken verbinden die Ufer. Weitere Sammelstellen liegen an den Lichtungen und Fernwegen. |
| Muschelbucht | Suche am Strand und hinter den Dünen nach Muscheln und Treibholz. Sammelstellen im Farnwald und an der Bucht füllen sich nach zwei Spieltagen wieder. |
| Eichenhafen | Tippe auf eine Figur und dann auf **Ansprechen**. Lina verkauft Saatgut und Miro kauft Erzeugnisse und Funde. Mara, Fiete und Bo geben Aufträge. Juna erzählt dir, wohin die Wege führen. |
| Aufträge | Nimm Aufträge im Gespräch an und kehre mit den geforderten Zutaten und Materialien zurück. Das Journal zeigt den Stand der Stadtaufträge und der drei regionalen Questketten. |
| Garten | Möhren, Rüben, Kürbisse, Erdbeeren, Tomaten und Sonnenblumen brauchen zwei bis fünf gegossene Tage. Eine Düngergabe vor der Reife reicht für die nächste Ernte. |
| Kalender | Uhrzeit und Wetter stehen oben im HUD. Im Journal siehst du Saisonfunde und die nächsten Dorftreffen. Schlafen setzt die Uhr auf 06:00 und lässt gegossene Pflanzen wachsen. Regen gießt Pflanzen und neue Saat automatisch. |
| Obstwiese | Pflücke Äpfel und Waldbeeren von Frühling bis Herbst. Abgeerntete Bäume und Sträucher tragen nach zwei Spieltagen wieder Früchte. |
| Hafenmarkt | Beim Saatgutstand jeweils drei Samen der gewählten Sorte kaufen. Beim Ankäufer wechselt die linke Auswahltaste zwischen den verfügbaren Waren. Obst, Gemüse, Fisch, Eier, Wolle, Silber und Kristalle bringen Münzen. |
| Tiere | Kaufe den Bauplan bei Bo oder am Bauplanstand, baue das passende Gebäude am Hof und nimm ein Tier auf. Pflege und Ernte erfolgen am eigenen Stall. Futter kannst du an der Futterkiste auf der Tierweide nachkaufen. |
| Mine | Kupfer und Silber liegen in den ersten Ebenen. Ab Tiefe 3 kommen Kristallnester und dunkles Gestein hinzu. Mit einer Kupferspitzhacke erhältst du zwei Erze pro Ader, mit einer Silberspitzhacke drei. |
| Aufgaben | Am Brett oder unter **Journal → Aufträge** öffnen. Die acht Hofaufgaben fördern Garten, Sammeln, Handel, Tiere und Mine. Für eine erfüllte Aufgabe gibt es Saatgut und Münzen. |
| Haus, Wald und Steg | Möbel an der Hofwerkbank herstellen und im Haus versetzen. Wildbäume fällen, Findlinge abbauen und am Steg angeln. |

### Steuerung

- **Touch.** Tippe auf Boden, Gegenstände oder Figuren. Drücke die rechte Aktionstaste, um den gewählten Gegenstand zu benutzen oder eine Figur anzusprechen. Tippe auf ein Namensschild auf der Karte, um zu dieser Figur zu laufen.
- **Xbox-Controller auf Android.** Links/rechts am Steuerkreuz wählt Ziele, oben öffnet das Journal und unten die Tasche. **View** öffnet ebenfalls die Tasche. Mit **LT** wählst du Saatgut oder Möbel, mit **RT** düngst du. **LB/RB** zoomen. **Menü** pausiert im Gelände. In Menüs nimm den linken Stick oder das Steuerkreuz und bestätige mit **A**. Der rechte Stick steuert den Zeiger. Drücke ihn am Hof hinein, um das Feldwerkzeug zu wechseln.
- **Tastatur und Maus.** WASD und Pfeiltasten bewegen den Tanuki. Tab wechselt das Beet, Leertaste führt die Aktion aus. Das Mausrad zoomt. R wechselt das Feldwerkzeug. M öffnet die Karte, E die Saatguttasche, Q das Journal und I die Tasche. Im Kampf spielen die Ziffern 1–8 eine Karte aus. Leertaste oder Enter beenden den Zug. F10 blendet das HUD aus, F12 speichert ein Spielbild.

## Projekt ausführen und bauen

Unter Linux braucht Bevy für die Tonausgabe die ALSA-Entwicklungsbibliothek. Auf Debian und Ubuntu heißt das Paket `libasound2-dev`.

```sh
cargo run --release --locked
cargo test --release --locked --lib
```

Für die Android-APK werden JDK 17 oder neuer, Android SDK Platform 36, Build Tools 36.0.0, NDK r28c, Python 3 und die Rust-Ziele für beide ABIs benötigt. `ANDROID_HOME` zeigt auf den SDK-Ordner. Optional zeigt `ANDROID_NDK_HOME` auf eine abweichende NDK-Installation.

```sh
rustup target add aarch64-linux-android x86_64-linux-android
tools/build-android.sh
```

Das Skript schreibt die APK nach `build/android`. NativeActivity startet das Rust-Spiel. Die Android-Activity verwaltet den Vollbildmodus und die Systemdialoge für Spielstanddateien. Mit `WALDWINKEL_BUNDLETOOL=/pfad/bundletool.jar` erzeugt und prüft das Skript zusätzlich ein lokales App Bundle mit dem Entwicklungsschlüssel.

## Blender und Weiterentwicklung

- `art/Waldwinkel.blend` enthält die Modelle und eine Gesamtansicht der Landschaft. `blender -b -P art/build_scene.py` exportiert die Modelle als GLB und erzeugt das Höhenfeld für Ufer, Brücken und Hindernisse. Bevy animiert Beine, Schuhe, Arme und den Schweif des Tanuki über getrennte Drehpunkte.
- `art/draw_icons.py` zeichnet die HUD-Symbole und das App-Icon. Die verwendeten Schriften sind DM Sans und DM Serif Text mit beigefügter OFL-Lizenz.
- `art/draw_inventory.py` zeichnet die Gegenstandsbilder. `art/build_audio.py` erzeugt Originalmusik, Umgebungsklänge und Effekte.
- Android leitet Controller-Ereignisse aus der NDK-Eingabewarteschlange an dasselbe Spiel- und UI-Eingabesystem weiter wie Touch. Die Schnittstelle liegt in `src/controller.rs`.
- Blender liefert die Regen- und Schneemodelle. Bevy animiert den Niederschlag, färbt die Außenansicht nach Wetter und Jahreszeit und zeigt gegossenen Boden.
- Blender exportiert saisonales Laub, Schneeauflagen und Fensterlicht als getrennte GLB-Ebenen. Beim Backen entsteht auch ein Höhenfeld für Regendach und Baumkronen. Die Wiesen verwenden ein gemeinsames Mesh mit saisonalen Vertexfarben. Die Niederschlagspartikel werden wiederverwendet.

## Über Find Skills geprüft

- **Bevy ECS** für Komponenten, Ressourcen, Systemreihenfolge und Tests. Die API-Beispiele wurden auf die im Projekt gepinnte Bevy-Version bezogen. [bevy-ecs](https://skills.sh/gamedev-skills/awesome-gamedev-agent-skills/bevy-ecs)
- **Blender Modeling, Animation und Export** für modellierte Kanten, Drehpunkte und die Prüfung des GLB-Exports. [Modeling](https://skills.sh/roble3/cc-blender-skill/blender-modeling), [Animation](https://skills.sh/roble3/cc-blender-skill/blender-animation), [Export](https://skills.sh/roble3/cc-blender-skill/blender-export)
- **Blender Motion State Inspection** und **Character Rigging** für Achsen, Gelenkpunkte und Fußkontakt. **Blender Weather FX** ist eine Referenz für spätere Wetter-Looks und Oberflächenmasken. [Motion State Inspection](https://skills.sh/affaan-m/ecc/blender-motion-state-inspection), [Character Rigging](https://skills.sh/el4cteo/blender-skills/blender-character-rigging), [Weather FX](https://skills.sh/el4cteo/blender-skills/blender-weather-fx)
- **Bevy 3D Lighting** und **Bevy VFX** für Tiefennebel, lokale Lichtflächen und eine wiederverwendete Partikelmenge. Die Umsetzung verwendet Bevy 0.16.1. [Bevy 3D Lighting](https://skills.sh/thedivergentai/game-dev-agentic-skills/bevy-3d-lighting) · [Bevy VFX](https://skills.sh/chrisgliddon/bevy-skills/bevy-vfx)
- **Physics Tuning** und Rapier-Referenz für begrenzte Bewegungsschritte und das Gleiten entlang von Hindernissen. Waldwinkel verwendet sein Bevy-Höhenfeld für die kinematische Bewegung. [Physics Tuning](https://skills.sh/gamedev-skills/awesome-gamedev-agent-skills/physics-tuning) · [Rapier Character Controller](https://rapier.rs/docs/user_guides/bevy_plugin/character_controller/)

## Tests

74 Rust-Tests prüfen neben Hof, Handwerk und Dämmergängen die neuen Kampfstile, das Lichtfest ohne Expedition, den Weg vom Hafen zur Farnwacht und das Wiederherstellen einer Sicherung. Einstieg und Navigation wurden im Spiel getestet. Die Lichtfest-Szene wurde mit einem vorbereiteten Spielstand aufgerufen. Im Android-14-Emulator blieben drei Testspielstände beim Update erhalten. Im Android-16-Emulator wurde ein Spielstand über den internen Speicher exportiert und in einen freien Platz importiert. Bewegung, Tasche und Karte funktionierten mit einem virtuellen Xbox-Profil. Ein Durchlauf durch die gesamte Kampagne mit neuen Spielern und ein Test mit einem physischen Xbox-Controller auf einem Tablet stehen als Geräte- und Spielzeitprüfungen aus.

Die Blender-Landschaften entstanden nach eigenen Bildreferenzen. [Insel und Pässe](art/references/insel.jpg) · [Kupferklamm](art/references/kupferklamm.png) · [Windweiden](art/references/windweiden.png) · [Nebelmoor](art/references/nebelmoor.png)

Für den Hausausbau dienten Baupläne und Handwerksfortschritt in Palia sowie der Kauf von Haus- und Stallausbauten beim Schreiner in Coral Island als Referenzen. [Palia](https://palia.com/news/dev-update-8) · [Coral Island](https://blog.stairwaygames.com/post/october-2021-dev-update)
