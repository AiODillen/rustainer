# Ziel des Projekts

Ziel des Projekt ist es eine Container Engine in Rust zu schreiben. Es soll mit ihr möglich sein Programme mit bestimmten Software und Hardware Spezifikationen zu starten.
Das Projekt soll hauptsächlich in der Programmiersprache Rust implementiert werden.
Das Zielsystem ist hierbei eine Linux Umgebung.

### Geplante Features

#### Isolierte Komponenten

- PID Namespace Isolation
- FS Isolation
- Netzwerk Isolation
- CPU Thread Begrenzung
- Arbeitsspeicher Begrenzung
- Konsolen Zugriff

Die Anwendung wird ein reines CLI Tool ohne TUI oder GUI.


# Umsetzung der Features

## PID Namespace Isolation

### Ansätze

PID Namespaces können über den Befehl
```
unshare --pid --fork --mound-proc %command%
```
erzeugt werden.

Bsp.: Setzen wir für den command bash ein und führen dann ein pstree aus:
![[Pasted image 20240826135513.png]]
Wir sehen dann nur den aktuellen PID Baum aus unserer Shell und pstree.

### Implementierung

In Rustainer wird der PID Namespace ebenfalls mittels `unshare` isoliert.

## FS Isolation

### Ansätze

Um in Linux ein Programm auf ein bestimmtes Verzeichnis zu beschränken gibt es keine direkten Methoden.
Der funktional beste Ansatz ist mittels `chroot`. Allerdings benötigt `chroot` einige Systemkomponenten um eine Shell zu erzeugen, z.B. `/bin/bash`. 

Es müssen auch einige Verzeichnisse vorhanden sein:
- `/usr`
- `/lib`
- `/lib64`
- `/etc`
- `/dev`

Um dies zu gewährleisten gibt es zwei Möglichkeiten:
1. Die Verzeichnisse und Dateien vom Host System in das Arbeitsverzeichnis zu kopieren
2. Ein minimales OS Dateisystem verwenden

Das Problem an Option 1 ist, dass bei jedem Start von Rustainer die Dateien kopiert werden müssen, was je nach System einige GB sein können. 
Ich konnte diese Methode leider nicht erfolgreich nachstellen.

Option 2 konnte ich erfolgreich umsetzen und implementieren. Getestet habe ich diese Methode mit dem Alpine Linux Minimal Root FS [Link](https://alpinelinux.org/downloads/). Problem mit dieser Lösung ist allerdings, dass sich das Root FS nicht einfach ändern lässt, da nur der Paketmanager `apx` verwendet werden kann. Ohne diesen kann das RootFS nicht angepasst werden.

### Implementierung

Ich habe in Rustainer kein komplett Isoliertes RootFS umgesetzt. 

Rustainer bietet die Möglichkeit ein Arbeitsverzeichnis festzulegen, welches dann auch automatisch erstellt und in der Shell gestartet wird.

Eine weitere Idee war die Nutzung der `rbash` die weniger Rechte besitzt, zwar ist hier der `cd` Befehl gesperrt, allerdings lassen sich Dateien auch in anderen Verzeichnissen löschen...

## Netzwerk Isolation

### Ansätze

Netzwerke können z.B. mit `cgroups` erstellt werden.
Hier ein Beispiel:

`sudo cgcreate -g net_cls:/isolated_group`
`echo "0x100001" | sudo tee /sys/fs/cgroup/net_cls/isolated_group/net_cls.classid`
`sudo iptables -A OUTPUT -m cgroup --cgroup 0x100001 -j DROP`
`sudo cgexec -g net_cls:isolated_group <your_application_command>`

Ein Problem mit `cgroups` ist leider der Versionsunterschied zwischen `cgroups v1` und `cgroups v2` die Implementierungen sind grundlegend verschieden. 
Rustainer funktioniert folglich nur auf einer bestimmten Version. Deswegen habe ich mich gegen die Umsetzung entschieden. Rustainer wäre sonst nur auf 50% aller Linux Systeme nutzbar.
### Implementierung

Eine Netzwerk Isolation ist in Rustainer nicht implementiert.

## CPU Thread / Memory Isolation

### Ansätze

CPU Threads und Arbeitsspeicher können auf viele Weisen für Programme manipuliert werden. Der direkteste Ansatz erfolgt über die `cgroups` 
Bei Verwendung der `cgroups` müssen allerdings, sofern keine externen Bibliotheken verwendet werden möchten, neue Verzeichnisse in root Verzeichnissen erstellt werden.

Auch `systemd` bietet mit `systemd-run` Zugriff auf die Cgroups eines Prozesses. Hier können die Parameter direkt mitgegeben werden.

Es gibt auch Rust crates, die die Manipulation von Cgroups erlauben, allerdings ist das eine, welches noch entwickelt wird sehr schlecht für die v2 cgroups dokumentiert und ich konnte es nicht verwenden.

Beispiel um Thread 1, 2 und 4 zu benutzen
`systemd-run -p AllowedCPUs=1,2,4`

Beispiel für Speicherbegrenzung
`systemd-run -p MemoryMax=512M`



### Implementierung

Rustainer implementiert beide Features mittels systemd-run.

# Anleitung

## Software Vorraussetzungen

- Systemd -> `systemd-run`
- `unshare`
- `bash`

## Benutzung

### Zugang zu Rustainer

Option 1:
- Download des latest release auf Github
	- https://github.com/AiODillen/rustainer/releases/latest

Option 2:
- Source code herunterladen via:
	- https://github.com/AiODillen/rustainer/releases/latest
	oder
	- git clone https://github.com/AiODillen/rustainer
- Build der Binärdatei

```
cd rustainer
cargo build --release
```

- Die Binärdatei befindet sich dann unter `./target/release/` mit dem namen `rustainer`

### Benutzen von Rustainer

Rustainer bietet Konsolen Parameter um die Sandbox zu konfigurieren.

Für eine Übersicht über alle Parameter kann`rustainer -h` oder `rustainer --help` aufgerufen werden:
```
Usage: rustainer [OPTIONS]

Options:
  -d, --directory <DIRECTORY>  Directory to store the container [default: ./container]
  
  -c, --cpus <CPUS>            List of CPU Threads to allocate (ex. 1,2,3,4  or 1 5 7 10) [default: 1]
  
  -m, --memory <MEMORY>        Maximum memory to allocate (ex. 512M, 1G, 300K) [default: 512M]
  
  -h, --help                   Print help
  ```
  
Alle Parameter, außer -h, --help, können miteinander beliebig kombiniert werden. Wird ein Parameter nicht angegeben wird ein Standardwert verwendet.

Der minimale Aufruf ist also `rustainer`, der eine Shell mit einem Thread und 512 MB Arbeitsspeicher im relativen Verzeichnis ./container bereitstellt. 

Nach dem Aufruf von `rustainer` öffnet sich eine Shell und eine Übersicht über die Parameter.


> [!NOTE] Berechtigungen
> Für Rustainer werden root Rechte benötigt, da systemd-run und unshare diese voraussetzen

![[Pasted image 20240826175757.png]]

Die Rustainer shell kann mit `exit` wieder verlassen werden, woraufhin auch der Ordner wieder gelöscht wird.


> [!error] Warnung
> Setze den Ordner für den Container NIE auf ein bestehendes Verzeichnis, da dieses gelöscht wird sobald sich der Container schließt


# Was habe ich gelernt

Das Projekt hat mir sehr viel Spass gemacht. Als jemand der auch privat auf allen Geräten Linux benutzt, war es super interessant sich mit den tieferen Schichten eines Systems auseinanderzusetzen. 
Mir ist im Laufe des Projekts klar geworden, dass der Scope des Projekts mehr auf Recherche und Dinge verstehen und ausprobieren liegt. Der Rust Teil des Projekts ist dagegen sehr Trivial, da er nahezu nur Systemfunktionen aufruft. 
# Arbeit für die Zukunft

- `screen` oder ähnliche Software verwenden, um Container im Hintergrund zu starten und frei zwischen ihnen zu wechseln
	- TUI für offene `screen` sessions bauen
- Netzwerk Isolation
	- Verbindungen zwischen Containern
- FS Isolation
	- keine Ahnung wie das schön geht xD

