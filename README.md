# Hydrogen Alpha solar imaging pipeline


A set of utilities and a pipeline for processing raw hydrogen-alpha solar imaging using lucky imaging  ("image stacking"). Designed as a specific use-case replacement for PIPP & Autostakkert. 

The current and planned steps include:
 * Flat and dark correction
 * Glitch frame detection (planned)
 * Quality estimation filtering 
 * Center-of-mass centering alignment
 * Cropping
 * Masking (planned, partially implemented)
 * Radiometric correction
 * Parallactic rotation for altazimuth mounting
 * Hot pixel detection and correction (planned, partially implemented)
 * Debayering (planned, partially implemented)
 * Stacking (Shift-and-add method)

```
USAGE:
    process_ha [FLAGS] [OPTIONS] --inputs <INPUT>... --latitude <LATITUDE> --longitude <LONGITUDE> --output <OUTPUT>

FLAGS:
        --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>              Blue weight
    -d, --dark <DARK>              Dark frame image
    -f, --flat <FLAT>              Flat frame image
    -G, --green <GREEN>            Green weight
    -h, --height <HEIGHT>          Crop height
    -i, --inputs <INPUT>...        Input SER files
    -l, --latitude <LATITUDE>      Observer latitude
    -L, --longitude <LONGITUDE>    Observer longitude
    -o, --output <OUTPUT>          Output file
    -q, --quality <QUALITY>        Quality limit (top % frames)
    -R, --red <RED>                Red weight
    -t, --threshold <THRESHOLD>    Object detection threshold
    -w, --width <WIDTH>            Crop width
```