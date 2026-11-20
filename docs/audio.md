# Musik und Geräusche

Die Musik von Waldwinkel ist eine eigene Komposition aus zwei gleich langen Spuren. Die leise Klavierstimme führt das Thema. In Eichenhafen tritt die Harfe deutlicher hervor. Abends, in Gesprächen und im Duell wird die Musik leiser. In den Pausen des Stücks bleiben die Geräusche der Umgebung hörbar.

Die kurzen Rückmeldungen für Gartenarbeit, Handwerk und Duelle sind getrennt abgemischt. Beim Fällen ertönt der Axtschlag beim ersten Kontakt, dann splittert das Holz, und beim Auftreffen des Stamms folgt der Aufprall. Unterschiedliche Aufnahmen und kleine Abweichungen in Tonhöhe und Lautstärke variieren wiederholte Aktionen. [GameSkill Audio Design](https://gameskill.dev/skills/audio-design)

Im Kartenkampf hört man zuerst das Gleiten des Papiers. Treffer, Schild und Sporen klingen mit dem Effekt an. Beim Kochen hört man das Messer auf dem Brett, das Rühren und das fertige Gericht. Glas und Handmühle verwenden andere Geräusche. `python art/action_audio.py` mischt diese Rückmeldungen aus den mitgelieferten Ausschnitten und kurzen Resonanzen.

## Aufnahmen

- Klavier aus der **VS Chamber Orchestra Community Edition**, aufgenommen von Simon Dalzell. Harfe aus derselben Bibliothek. Lizenz CC0 1.0. [VSCO 2 CE](https://github.com/sgossner/VSCO-2-CE/tree/440300901dfe9275fd84e0b7763af1f8443ae62e)
- **Axe chopping wood**, xkeril. Lizenz CC0. [Aufnahme](https://freesound.org/people/xkeril/sounds/753925/)
- **Dead tree cracking & falling to the ground in forest #1**, ozymandias37. Lizenz CC0. [Aufnahme](https://freesound.org/people/ozymandias37/sounds/867699/)
- **Rocks Falling 06 und 04**, Tim Verberne. Lizenz CC0. [Aufnahme 06](https://freesound.org/people/Tim_Verberne/sounds/576107/) · [Aufnahme 04](https://freesound.org/people/Tim_Verberne/sounds/576103/)
- **Birdsong**, JW Audio. Lizenz CC0. [Aufnahme](https://freesound.org/people/JW_Audio/sounds/817480/)
- **Digging Into Soft Soil with Small Trowell**, dhallcomposer. Lizenz CC0. [Aufnahme](https://freesound.org/people/dhallcomposer/sounds/696531/)
- **Scooping Water with Plastic Watering Can**, qubodup. Lizenz CC0. [Aufnahme](https://freesound.org/people/qubodup/sounds/210424/)

Die benutzten Ausschnitte sind gefiltert, zugeschnitten und in den Spielmix integriert. `python art/render_score.py /pfad/zum/sample-cache` rendert Musik und Ausschnitte mit NumPy und FFmpeg. `art/build_audio.py` erzeugt die übrigen Geräusche für Schritte, Wellen und Regen.
