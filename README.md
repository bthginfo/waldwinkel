# Waldwinkel

Ein Cozy-3D-Spiel aus Bevy und Rust. Hof, Farnwald, Muschelbucht und Eichenhafen liegen auf einer zusammenhängenden Küstenlandschaft von rund 160 × 110 Spielmetern. Alle Außenbereiche sind zu Fuß erreichbar. Der Tanuki, die sechs Stadtfiguren und die 3D-Modelle stammen aus dem beigefügten Blender-Projekt.

![Der Hof und der auswählbare Eingang der Holzhütte](docs/hof.png)

## Installieren

- Lade die signierte APK herunter und öffne sie auf deinem Android-Gerät unter **Downloads**. [Android-APK](https://github.com/bthginfo/waldwinkel/releases/latest/download/Waldwinkel-Android.apk) · [Alle Versionen](https://github.com/bthginfo/waldwinkel/releases)
- Voraussetzungen sind Android 8 oder neuer, Vulkan und ein 64-Bit-System (ARM64 oder x86-64). Das Spiel läuft offline. Installiere das Update über die bisherige Version, damit deine Spielstände erhalten bleiben.

## Neu in 0.6.0

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
| Stadtaufträge | Nimm Aufträge im Gespräch an und kehre mit den geforderten Zutaten und Materialien zurück. **Stadtaufträge** zeigt in allen drei neuen Gebieten, welche Aufgaben offen, angenommen, abgabebereit oder erfüllt sind. |
| Garten | Möhren, Rüben, Kürbisse, Erdbeeren, Tomaten und Sonnenblumen anbauen. Saat links wechseln. Je nach Sorte sind zwei bis fünf gegossene Tage nötig. |
| Kalender | Schlafen im Bett beginnt den nächsten Tag. Eine Jahreszeit dauert sieben Tage. Nebel, Regen und Winterschnee verändern die Landschaft. Regen gießt Pflanzen und neue Saat automatisch. |
| Obstwiese | Äpfel und Waldbeeren pflücken. Jeder abgeerntete Baum oder Strauch trägt nach zwei Spieltagen wieder Früchte. |
| Hafenmarkt | Beim Saatgutstand jeweils drei Samen der gewählten Sorte kaufen. Beim Ankäufer wechselt die linke Auswahltaste zwischen den verfügbaren Waren. Obst, Gemüse, Fisch, Eier, Wolle, Silber und Kristalle bringen Münzen. |
| Tierweide | Huhn und Schaf adoptieren und am Trog füttern. Nach einem gefütterten Tag ist ein Ei bereit. Nach zwei gefütterten Tagen gibt das Schaf Wolle. An der Futterkiste wird Futter nachgekauft. |
| Mine | Kupfer und Silber liegen in den ersten Ebenen. Ab Tiefe 3 kommen Kristallnester und dunkles Gestein hinzu. Die Hafenschmiede kann die Spitzhacke für zwei Kupfer, zwei Holz und zwölf Münzen verbessern. Die verbesserte Hacke liefert zwei Erze pro Ader. |
| Aufgaben | Am Brett oder links über **Aufgaben** öffnen. Die acht Aufgaben fördern Garten, Sammeln, Handel, Tiere und Mine. Für eine erfüllte Aufgabe gibt es Saatgut und Münzen. |
| Haus, Wald und Steg | Möbel herstellen und versetzen, markierte Kiefern fällen, Stein abbauen und am Steg angeln. |

Am Rechner funktionieren auch WASD oder Pfeiltasten, Tab zum Beetwechsel, Leertaste für die Aktion, Mausrad zum Zoomen, M für die Weltkarte, E zur Auswahl, Q für Aufgaben und I für die Tasche. Escape öffnet das Pausenmenü oder schließt ein Gespräch. F10 blendet das HUD aus, F12 speichert ein Spielbild.

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
- Bewegung, Beetauswahl und Aktionen laufen im Spiel über gemeinsame Eingaben. Für Controller kann dort eine weitere Eingabequelle angeschlossen werden.
- Blender liefert die Regen- und Schneemodelle. Bevy animiert den Niederschlag, färbt die Außenansicht nach Wetter und Jahreszeit und zeigt gegossenen Boden.

## Über Find Skills geprüft

- **Bevy ECS** für Komponenten, Ressourcen, Systemreihenfolge und Tests. Die API-Beispiele wurden auf die im Projekt gepinnte Bevy-Version bezogen. [bevy-ecs](https://skills.sh/gamedev-skills/awesome-gamedev-agent-skills/bevy-ecs)
- **Blender Modeling, Animation und Export** für modellierte Kanten, Drehpunkte und die Prüfung des GLB-Exports. [Modeling](https://skills.sh/roble3/cc-blender-skill/blender-modeling), [Animation](https://skills.sh/roble3/cc-blender-skill/blender-animation), [Export](https://skills.sh/roble3/cc-blender-skill/blender-export)
- **Blender Motion State Inspection** und **Character Rigging** für Achsen, Gelenkpunkte und Fußkontakt. **Blender Weather FX** ist eine Referenz für spätere Wetter-Looks und Oberflächenmasken. [Motion State Inspection](https://skills.sh/affaan-m/ecc/blender-motion-state-inspection), [Character Rigging](https://skills.sh/el4cteo/blender-skills/blender-character-rigging), [Weather FX](https://skills.sh/el4cteo/blender-skills/blender-weather-fx)

## Tests und nächste Etappen

22 Rust-Tests decken unter anderem zusammenhängende Fußwege, Zugriff auf feste Objekte, Kollisionen, Bachquerungen, Speichern und die Übernahme alter Aufenthaltsorte ab. Der Einstieg per Touch und das Update von 0.5.0 wurden im Android-14-Emulator geprüft. Die Aufnahmen stammen aus dem laufenden Bevy-Spiel.

1. **Hofalltag und Ernte.** Zeitgesteuerter Tagesablauf mit Schlafen, besserer Saat- und Gießablauf, Saatgut aus Handel, Funden, Aufträgen und zufälligen Erntedrops, Qualitätsstufen, Dünger und eine optionale Verkaufstruhe am Hof.
2. **Ausbau und Verarbeitung.** Tiergebäude und Tierhaltung auf dem eigenen Hof, sammelbare Möbelrezepte für die Werkbank sowie Koch- und Verarbeitungsminispiele.
3. **Neue Regionen.** Verbundene Gebiete in der Größenordnung der bisherigen Welt, eigene Materialien und Figuren, mehrstufige Questketten und je ein über eine Quest freischaltbarer Schnellreisepunkt. Pro Region entsteht vor den Blender-Modellen eine Bildreferenz.
4. **Kartenkämpfe.** Prozedurale Ebenen mit ansteigender Schwierigkeit und Rückkehr zu freigeschalteten Ruheplätzen. Karten kommen etwa aus Erkundung, Aufträgen, Handel und Kämpfen. Ein Deck umfasst zwölf Karten und wird am Kartenbinder daheim zusammengestellt. Kampftalente verbessern Werte wie Gesundheit und Resistenzen.
