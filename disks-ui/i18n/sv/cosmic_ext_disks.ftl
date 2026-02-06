app-title = Diskar
about = Om
view = Visa
welcome = Välkommen till COSMIC! ✨
page-id = Sida { $num }
git-description = Git commit {$hash} på {$date}

# Menyalternativ
menu-image = Avbild
menu-disk = Disk
menu-view = Visa
new-disk-image = Ny diskavbild
attach-disk-image = Bifoga skivavbildning
create-disk-from-drive = Skapa disk från enhet
restore-image-to-drive = Återställ avbild till disk
create-disk-from-partition = Skapa diskavbild från partition
restore-image-to-partition = Återställ diskavbild till partition
image-file-path = Sökväg till avbildsfil
image-destination-path = Målfilens sökväg
image-source-path = Källavbildens sökväg
image-size = Avbildsstorlek
create-image = Skapa avbild
restore-image = Återställ avbild
attach = Bifoga
restore-warning = Detta kommer att skriva över den valda målenheten. Detta kan inte ångras.
eject = Mata ut
eject-failed = Mata ut misslyckades
power-off = Stäng av
power-off-failed = Stäng av misslyckades
format-disk = Formatera disk
format-disk-failed = Formatera disk misslyckades
benchmark-disk = Jämför disk
smart-data-self-tests = SMART data & självtester
drive-settings = Disk inställningar
standby-now = Vänteläge nu
standby-failed = Vänteläge misslyckades
wake-up-from-standby = Vakna upp från vänteläge
wake-up-failed = Uppvakning misslyckades
unmount-failed = Avmontering misslyckades

# Dialogknappar
ok = Ok
cancel = Avbryt
continue = Fortsätt
working = Arbetar…

# Common
close = Stäng
refresh = Uppdatera
details = Detaljer

# Formatera disk dialogruta
erase-dont-overwrite-quick = Skriv inte över (snabbt)
erase-overwrite-slow = Skriv över (långsamt)
partitioning-dos-mbr = Bakåtkompatibel (DOS/MBR)
partitioning-gpt = Modern (GPT)
partitioning-none = Ingen

# Skapa partition dialogrutan
create-partition = Skapa partition
create-partition-failed = Skapa partition misslyckades
format-partition = Formatera partition
format-partition-description = Detta kommer att formatera den valda volymen. Storlek: { $size }
volume-name = Volymnamn
partition-name = Partitionsnamn
partition-size = Partitionsstorlek
free-space = Ledigt utrymme
erase = Radera
password-protected = Lösenordsskyddad
password = Lösenord
confirm = Bekräfta
password-required = Lösenord krävs.
password-mismatch = Lösenorden matchar inte.
apply = Verkställ
untitled = Namnlös

# Huvudvy
no-disk-selected = Ingen disk vald
no-volumes = Inga volymer tillgängliga
partition-number = Partition { $number }
partition-number-with-name = Partition { $number }: { $name }
volumes = Volymer
unknown = Okänd
unresolved = Olöst

# Informationsetiketter
size = Storlek
usage = Användning
mounted-at = Monterad på
contents = Innehåll
device = Enhet
partition = Partition
path = Sökväg
uuid = UUID
model = Modell
serial = Serienummer
partitioning = Partitionering
backing-file = Bakgrundsfil
disk-usage = Diskanvändning

# Bekräftelsedialogruta
delete = Radera { $name }
delete-partition = Radera
delete-confirmation = Är du säker på att du vill radera { $name }?
delete-failed = Radering misslyckades

# Volymsegment
free-space-segment = Ledigt utrymme
reserved-space-segment = Reserverat
filesystem = Filsystem
free-space-caption = Ledigt utrymme
reserved-space-caption = Reserverat utrymme

# Volymer vy
show-reserved = Visa reserverat

# Encrypted / LUKS
unlock-button = Lås upp
lock = Lås
unlock = Lås upp { $name }
passphrase = Lösenfras
current-passphrase = Nuvarande lösenfras
new-passphrase = Ny lösenfras
change-passphrase = Byt lösenfras
passphrase-mismatch = Lösenfraserna matchar inte.
locked = Låst
unlocked = Upplåst
unlock-failed = Upplåsning misslyckades
lock-failed = Låsning misslyckades
unlock-missing-partition = Kunde inte hitta { $name } i den aktuella enhetslistan.

# Volume commands
mount-toggle = Montera / avmontera
edit-mount-options = Redigera monteringsalternativ…
edit-encryption-options = Redigera krypteringsalternativ…
edit-partition = Redigera partition
edit-partition-no-types = Inga partitionstyper är tillgängliga för denna partitionstabell.
flag-legacy-bios-bootable = Legacy BIOS-startbar
flag-system-partition = Systempartition
flag-hide-from-firmware = Dölj från firmware
resize-partition = Ändra partitionsstorlek
resize-partition-range = Tillåtet intervall: { $min } till { $max }
new-size = Ny storlek
edit-filesystem = Redigera filsystem
filesystem-label = Filsystemetikett
check-filesystem = Kontrollera filsystem
check-filesystem-warning = Att kontrollera ett filsystem kan ta lång tid. Fortsätt?
repair-filesystem = Reparera filsystem
repair-filesystem-warning = Att reparera ett filsystem kan ta lång tid och kan riskera dataförlust. Fortsätt?
take-ownership = Ta ägarskap
take-ownership-warning = Detta kommer att ändra ägarskap för filer till din användare. Detta kan ta lång tid och kan inte enkelt ångras.
take-ownership-recursive = Tillämpa rekursivt

# Mount/encryption options
user-session-defaults = Användarsessionens standardvärden
mount-at-startup = Montera vid systemstart
unlock-at-startup = Lås upp vid systemstart
require-auth-to-mount = Kräv behörighet för att montera eller avmontera
require-auth-to-unlock = Kräv behörighet för att låsa upp
show-in-ui = Visa i användargränssnittet
identify-as = Identifiera som
other-options = Andra alternativ
mount-point = Monteringspunkt
filesystem-type = Filsystemstyp
display-name = Visningsnamn
icon-name = Ikonnamn
symbolic-icon-name = Symboliskt ikonnamn
show-passphrase = Visa lösenfras
name = Namn

# SMART
smart-no-data = Ingen SMART-data tillgänglig.
smart-type = Typ
smart-updated = Uppdaterad
smart-temperature = Temperatur
smart-power-on-hours = Drifttimmar
smart-selftest = Självtest
smart-selftest-short = Kort självtest
smart-selftest-extended = Utökat självtest
smart-selftest-abort = Avbryt självtest
