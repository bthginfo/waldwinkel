# Waldwinkel 1.2

Ein Cozy-3D-Spiel aus Bevy und Rust. Der Hof und sechs Ausflugsgebiete liegen in einer zusammenhängenden Landschaft. Wege und Brücken verbinden Farnwald, Muschelbucht, Eichenhafen, Kupferklamm, Windweiden und Nebelmoor. Der Tanuki, die neun Figuren und die 3D-Modelle stammen aus dem beigefügten Blender-Projekt.

![Der Hof im Frühling, Sommer, Herbst und Winter](docs/12-jahreszeiten.jpg)

## Installieren

- Lade die signierte APK im Browser herunter und öffne die Datei auf deinem Android-Gerät unter **Downloads**. [Android-APK](https://github.com/bthginfo/waldwinkel/releases/latest/download/Waldwinkel-Android.apk) · [Alle Versionen](https://github.com/bthginfo/waldwinkel/releases)
- Voraussetzungen sind Android 8 oder neuer, Vulkan und ein 64-Bit-System (ARM64 oder x86-64). Das Spiel läuft offline. Installiere das Update über die bisherige Version, damit deine Spielstände erhalten bleiben.

## Neu in 1.2

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
| Tiere | Baue das passende Gebäude am Hof und nimm ein Tier auf. Pflege und Ernte erfolgen am eigenen Stall. Futter kannst du an der Futterkiste auf der Tierweide nachkaufen. |
| Mine | Kupfer und Silber liegen in den ersten Ebenen. Ab Tiefe 3 kommen Kristallnester und dunkles Gestein hinzu. Die Hafenschmiede kann die Spitzhacke für zwei Kupfer, zwei Holz und zwölf Münzen verbessern. Die verbesserte Hacke liefert zwei Erze pro Ader. |
| Aufgaben | Am Brett oder unter **Journal → Aufträge** öffnen. Die acht Hofaufgaben fördern Garten, Sammeln, Handel, Tiere und Mine. Für eine erfüllte Aufgabe gibt es Saatgut und Münzen. |
| Haus, Wald und Steg | Möbel herstellen und versetzen, markierte Kiefern fällen, Stein abbauen und am Steg angeln. |

### Steuerung

- **Touch.** Tippe auf Boden, Gegenstände oder Figuren. Drücke die rechte Aktionstaste, um den gewählten Gegenstand zu benutzen oder eine Figur anzusprechen.
- **Android-Controller.** Der linke Stick bewegt den Tanuki. Der rechte Stick steuert den Zeiger. In geöffneten Menüs springt das Steuerkreuz zwischen Schaltflächen. **A** wählt aus, **X** führt die ausgewählte Aktion aus und **Y** öffnet die Karte. Die Schultertasten zoomen. **B** oder **Start** schließen ein Fenster oder öffnen das Pausenmenü.
- **Tastatur und Maus.** WASD und Pfeiltasten bewegen den Tanuki. Tab wechselt das Beet, Leertaste führt die Aktion aus. Das Mausrad zoomt. M öffnet die Karte, E die Saatguttasche, Q das Journal und I die Tasche. Im Kampf spielen die Ziffern 1–8 eine Karte aus. Leertaste oder Enter beenden den Zug. F10 blendet das HUD aus, F12 speichert ein Spielbild.

## Projekt ausführen und bauen

```sh
cargo run --release --locked
cargo test --release --locked --lib
```

Für die Android-APK werden JDK 17 oder neuer, Android SDK Platform 36, Build Tools 36.0.0, NDK r28c, Python 3 und die Rust-Ziele für beide ABIs benötigt. `ANDROID_HOME` zeigt auf den SDK-Ordner. Optional zeigt `ANDROID_NDK_HOME` auf eine abweichende NDK-Installation.

```sh
rustup target add aarch64-linux-android x86_64-linux-android
tools/build-android.sh
```

Das Skript schreibt die APK nach `build/android`. NativeActivity startet das Rust-Spiel. Die kleine Android-Activity setzt die Systemleisten auf Vollbild.

## Blender und Weiterentwicklung

- `art/Waldwinkel.blend` enthält die Modelle und eine Gesamtansicht der Landschaft. `blender -b -P art/build_scene.py` exportiert die Modelle als GLB und erzeugt das Höhenfeld für Ufer, Brücken und Hindernisse. Bevy animiert Beine, Schuhe, Arme und den Schweif des Tanuki über getrennte Drehpunkte.
- `art/draw_icons.py` zeichnet die HUD-Symbole und das App-Icon. Die verwendeten Schriften sind DM Sans und DM Serif Text mit beigefügter OFL-Lizenz.
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

50 Rust-Tests decken Jahreszeitenwechsel, Fundorte, Dorftreffen, Spielstandmigration, Kollisionen und die 25 Dämmergänge ab. Die Android-Builds werden im Android-14-Emulator über die vorherige Installation geprüft. Die Aufnahmen stammen aus dem laufenden Bevy-Spiel.

Die Blender-Landschaften entstanden nach eigenen Bildreferenzen. [Kupferklamm](art/references/kupferklamm.png) · [Windweiden](art/references/windweiden.png) · [Nebelmoor](art/references/nebelmoor.png)
