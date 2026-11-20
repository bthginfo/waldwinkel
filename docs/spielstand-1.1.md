# Waldwinkel 1.1 im Vergleich zu 1.0

1.0 hat einen funktionierenden Hof, eine zusammenhängende Welt und einen vollständigen Weg durch die Dämmergänge. Die stärksten Szenen waren der Hof und kurze Abschnitte um die Siedlungen. Dazwischen blieben Wege oft ohne eigenen Anlaufpunkt. Die Analyse bezieht sich auf Spielaufnahmen und die exportierten Blender-Modelle.

| Bereich | Beobachtung in 1.0 | Änderung in 1.1 |
| --- | --- | --- |
| Modelle und Landschaft | Häuser, Wege und Tanuki hatten eine erkennbare Formsprache. Auf langen Strecken wiederholten sich Pflanzen und Felsen. Das Wasser hatte in der Kartenansicht eine sichtbare Außenkante. | Erzgleis und Werkplätze in der Klamm, Flachs und ein drehendes Mühlrad auf den Windweiden, Weidenhaine, Laternen und Seerosen im Nebelmoor. Ein breiterer Wasserspiegel füllt die Übersicht. |
| Dichte | Ein Großteil der Details stand nahe den Siedlungen. Die Vegetation abseits davon ließ die Routen weit auseinander wirken. | Neue Details stehen in Gruppen entlang von Wegen. Freie Weiden und Spielwege bleiben als Sicht- und Laufkorridore erkennbar. |
| Aufträge und Geschichte | Stadtaufträge und die regionalen Ketten hatten eigene Belohnungen. Der Spieler musste Hinweise auf nächste Schritte über mehrere Ansichten zusammensuchen. | Das Journal trennt die Geschichte von Nebenaufträgen und markiert erreichbare Ziele. Junas Auftrag führt zu drei Klangsteinen und endet mit einer beleuchteten Hafenleuchte in Eichenhafen. |
| Kartenkampf | Fast alle Gegner wechselten zwischen einem einfachen Angriff und Rüstung. Die Räume änderten ihre Anordnung und behielten eine ähnliche Grundform. | Doppelschläge, Heilung und unterbrechbare Angriffe verändern die Reihenfolge, in der sich Karten lohnen. Fünf Kapitel haben eigene Hüter und Bodenpaletten. Zusätzliche Gänge verbinden bestehende Räume. |
| Oberfläche | Die linke Leiste nahm viel Höhe ein. Im Kampf standen die meisten Werte in denselben Textzeilen. Zurück konnte einen Kampf sofort verlassen. | Eine Werkzeugleiste mit beschrifteten Symbolen, getrennte Lebensbalken und Kraftanzeige, größere Bedienelemente, Spielstandoption für eine größere Darstellung und eine Rückfrage vor dem Rückzug. Das Steuerkreuz springt in geöffneten Android-Menüs zwischen Schaltflächen. |

[Hof](11-hof.png) · [Kupferklamm](11-klamm.png) · [Windweiden](11-windweiden.png) · [Nebelmoor](11-moor.png) · [Hafenleuchte](11-lichtfest.png)

[Journal](11-journal.png) · [Markiertes Ziel auf der Karte](11-karte.png) · [Unterbrochener Angriff in der Glockenhalle](11-kampf.png)

## Geometrie

Aus den GLB-Indizes gezählte Dreiecke. Der größte Teilmesh ist die größte einzelne sichtbarkeitsgeprüfte Einheit einer Region. Die kleineren Teilmeshes erlauben es Bevy, Geometrie außerhalb der Kamera auszulassen.

| Region | Dreiecke 1.0 → 1.1 | Größter Teilmesh 1.0 → 1.1 |
| --- | ---: | ---: |
| Kupferklamm | 86.740 → 109.225 | 86.740 → 24.625 |
| Windweiden | 90.097 → 130.642 | 90.097 → 27.502 |
| Nebelmoor | 110.542 → 151.060 | 110.542 → 23.128 |
| Eichenhafen | 206.330 → 206.330 | 105.461 → 32.901 |

Die Zahlen beschreiben die Modelle. Laufzeit und Akkubelastung hängen vom Gerät und der Kameraposition ab.

## Gestaltung und Eingabe

- Die Android-Leitlinie diente als Sollwert für gut erreichbare Touch-Flächen. Beide Darstellungsgrößen skalieren mit Bevy `UiScale`. [Android Accessibility](https://developer.android.com/design/ui/mobile/guides/foundations/accessibility) · [Bevy UI Scaling](https://bevy.org/examples/ui-user-interface/ui-scaling/)
- Wichtige Angaben im Kampf haben Text und eine visuelle Anzeige. Die Oberflächen orientieren sich an hohem Textkontrast und einer vorhersehbaren Reihenfolge im Menü. [Xbox Accessibility Guidelines für Kontrast](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/102) · [Xbox Accessibility Guidelines für UI-Navigation](https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/112)
- Für Layout, Zustandswechsel und die Prüfung bei unterschiedlichen Fenstergrößen diente der Game-UI/UX-Skill als Arbeitsgrundlage. [Game UI/UX](https://www.skills.sh/gamedev-skills/awesome-gamedev-agent-skills/game-ui-ux)

## Spielprüfung

43 Rust-Tests prüften Vorräte und Spielstände, die Routen zu den Klangsteinen, alle 25 Dämmergänge und die Kampfbalance mit einem wachsenden Deck. Im laufenden Spiel wurden Junas Auftrag, die drei Klangsteine, der Endkampf und das Entzünden der Hafenleuchte geprüft. Auf Android 14 liefen Touch und Controller-Simulation im Emulator. Das Layout wurde bei 1000 × 560, 1280 × 800 und 1400 × 600 geprüft. Der Installationsvergleich prüfte die drei lokalen Spielstandsdateien vor dem Start der neuen Version.
