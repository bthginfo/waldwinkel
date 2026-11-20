# Waldwinkel 1.4 · Spielstand und Qualitätsprüfung

Die 1.3.1 hatte bereits einen funktionierenden Hof, eine große Insel, vier Jahreszeiten und eine Kartenkampagne mit 25 Ebenen. Für ein längeres Spiel fehlten vor allem Verbindungen zwischen diesen Teilen. Die Wegwachten geben ihrer Erkundung ab dem ersten Spieltag einen eigenen Verlauf.

| Bereich | Beobachtung in 1.3.1 | Umsetzung in 1.4 |
| --- | --- | --- |
| Modelle und Lesbarkeit | Die Holzarchitektur, Farbpalette und Vegetation wirken zusammenhängend. Viele Orte teilen dieselben Silhouetten. | Fünf eigene Blender-Modelle mit markanten Dächern, Steinformen, Werkzeugen und Leuchten. Das Licht ändert sich bei der Reparatur. Beim Kameratest zog die Farnwacht vom verdeckten Hang in eine Lichtung um. |
| Dichte der Karte | Die meisten aktiven Ziele liegen in Siedlungen und an festen Auftragsorten. Einige Wege dazwischen haben wenige Gründe zum Anhalten. | Fünf Wachten, fünf lesbare Wegzeichen und Pflanzen-, Fels- und Pfaddetails in ihrem Umfeld. Die Wege zu den Wachten und Wegzeichen werden mit den Kollisionsdaten des Spiels geprüft. |
| Quests | Der Fischersteg hat einen gut sichtbaren Aufbau in Etappen. Viele andere Aufträge sind einzelne Abgaben. | Jede Wacht hat eine Spur, einen Bauplan und ihren eigenen Einsatz. Drei Wachten verlangen ein Kartenduell. Am Hafenbrett wechseln acht Aufträge im Rhythmus von drei Tagen. Aktive Aufträge bleiben über den Wechsel hinaus erhalten. |
| Handlung | Fischereisteg, Lichtfest und Dämmergänge geben Stadt und Insel Geschichte. Der zweite größere Handlungsbogen stand spät im Spiel. | Bos Streckenbuch und Junas Inselkarte verbinden fünf regionale Geschichten. Eine Szene mit Untertiteln und eine eigene Möbelbelohnung schließen den Bogen ab. |
| Kampf | Angriffsabsichten und die Möglichkeit, einen aufgeladenen Schlag zu unterbrechen, helfen bei der Zugplanung. Der Zugang über das Moor liegt spät. | Ein frühes Duell in der Farnwacht, drei Orte mit je vier Prüfungen und eigene Regeln für Dornen, Rüstung und Nebel. Vier Gerichte liefern je Kampf einmal einen unterschiedlichen Vorteil. |
| Handwerk | Werkbank und Schmiedetisch haben klare Freischaltungen. Material fließt meistens direkt in ein Möbel oder eine Werkzeugstufe. | Fünf Zwischenprodukte, Fertigung in einem, drei oder fünf Arbeitsgängen und ein Webrahmen. Die Zwischenprodukte werden in Aufträgen, bei der Wiederherstellung der Wachten und im Handel gebraucht. |
| Fortschritt über mehrere Abende | Hofausbau, Karten, Talente und Feste geben verschiedene Langzeitziele. Erkundung und Produktion greifen stellenweise locker ineinander. | Der erste Wachtauftrag nutzt einfaches Holz. Spätere Wachten greifen auf Kochen, Schmieden, Tierhaltung und Flachs zurück. Die 25 Dämmer-Ebenen, 12 Wachtprüfungen, vier Saisonfeste und Aufträge können sich über mehrere Abende abwechseln. |
| Xbox auf dem Tablet | Der rechte Stick konnte einen Zeiger bewegen. Für häufige Aktionen fehlten Direktbelegungen und der Fokus orientierte sich an sichtbaren Schaltflächen der gesamten Oberfläche. | D-Pad wählt nahe Ziele und navigiert im aktiven Menü. Direkttasten öffnen Karte, Tasche, Journal und Saatgut. A bestätigt, B geht zurück, und Y beendet im Duell den Zug. Eingabeflanken von Steuerkreuz und Triggern bleiben auch bei langsamen Frames erhalten. |

## Umfang und Prüfverfahren

- Die neue Modellgruppe beansprucht rund 3 MB als GLB und umfasst fünf Bauwerke, Wegzeichen, Leuchten, den Webrahmen und die Wegwachtlampe.
- Im Tablet-Emulator wurde ein virtuelles Eingabegerät mit dem Xbox-One-S-Profil erkannt. Im laufenden Spiel funktionierten D-Pad und A bei der Spielstandwahl, der linke Analogstick beim Laufen, View und LT für Tasche und Saatgut, B zum Schließen, die Menütaste zum Pausieren und Y für die Karte. Das Firmwareverhalten eines physischen Xbox-Controllers über Bluetooth bleibt als Geräteprüfung offen. [Android-Belegung](https://developer.android.com/training/game-controllers/controller-input)


- Die Kampagnenpfade bieten 37 Begegnungen mit steigender Schwierigkeit über Dämmergänge und Wachtprüfungen. Die ersten Ziele eignen sich für kurze Sitzungen, spätere Werkzeuge, Anbauten und Feste für mehrere Spieltage. Die Dauer eines vollständigen Durchlaufs bleibt als Langzeitmessung offen.
- Im Spiel wurden Wegzeichen, Reparatur der Werkbank, Fertigung von sechs Brettern, Reparatur der Farnwacht, ein Duell mit Proviant und die Übergabe bei Juna mit Testspielständen geprüft. Die Stufen und Belohnungen stehen danach im Spielstand. [Farnwacht](14-farnwacht.jpg) · [Wegwachten](14-wegwachten.jpg) · [Fertigung](14-materialien.jpg) · [Duell](14-proviant.jpg)
- Die Tests decken alle zwölf Wachtprüfungen mit einem ausgebauten Deck, den ersten Wachtkampf mit dem Startdeck, die fünf Wege, die Handwerksschritte, den Wechsel der Aufträge, das Ernten über mehrere Tage, die Proviantregeln und das einmalige Auszahlen der Belohnungen ab.
- Die Referenz zeigt die angestrebten Formen und Materialien. Die Modelle stammen aus der Blender-Datei und dem Exportskript des Projekts. [Bildreferenz der fünf Wachten](../art/references/wegwachten.jpg)

## Android 1.4.1

Ein Kaltstart unter dem Android-Emulator hat bei der Erstellung eines Shaders abgebrochen. Der Android-Build verwendet deshalb das Vulkan-Kompatibilitätsprofil, bereitet Instanzen auf der CPU vor und erstellt die Grafikpipelines synchron.
