# Hydrogen Alpha Solar Imaging Pipeline

## Overview
A set of utilities and a pipeline for processing raw hydrogen-alpha solar imaging using lucky imaging  ("image stacking"). Designed as a specific use-case replacement for PIPP & Autostakkert. 

The current and planned steps include:
 * Flat and dark correction
 * Glitch frame detection
 * Quality estimation filtering 
 * Center-of-mass centering alignment
 * Cropping
 * Masking 
 * Radiometric correction
 * Parallactic rotation for altazimuth mounting
 * Hot pixel detection and correction (planned, partially implemented)
 * Debayering (planned, partially implemented)
 * Stacking (Shift-and-add method)

Future Plans:
 * GUI Support
 * Lunar & planetary

## `process_ha`
Processes a solar observation.

```
USAGE:
    process_ha [FLAGS] [OPTIONS] --inputs <INPUT>... --latitude <LATITUDE> --longitude <LONGITUDE> --output <OUTPUT>

FLAGS:
        --help       Prints help information
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -B, --blue <BLUE>                      Blue weight
    -d, --dark <DARK>                      Dark frame image
    -f, --flat <FLAT>                      Flat frame image
    -G, --green <GREEN>                    Green weight
    -h, --height <HEIGHT>                  Crop height
    -i, --inputs <INPUT>...                Input SER files
    -l, --latitude <LATITUDE>              Observer latitude
    -L, --longitude <LONGITUDE>            Observer longitude
    -m, --mask <MASK>                      Image mask
    -S, --maxsigma <MAXSIGMA>              Maximum sigma value (quality)
    -s, --minsigma <MINSIGMA>              Minimum sigma value (quality)
    -o, --output <OUTPUT>                  Output file
    -P, --percentofmax <PARAM_PCTOFMAX>    Scale maximum value to percentage max possible (0-100)
    -q, --quality <QUALITY>                Quality limit (top % frames)
    -R, --red <RED>                        Red weight
    -t, --threshold <THRESHOLD>            Object detection threshold
    -w, --width <WIDTH>                    Crop width
```

## `run.sh`
A wrapper for process_ha with standardized parameters for chromosphere, prominenance, and optional white-light stacks. Meant to be easily editable given the requirements/specifications of an observation.

Required parameter: root directory of the observation files.
Optional parameter: version string as output filename suffix.

## `ser_extract`
Extracts individual frames from ser files, optionally performs calibrations and quality limiting/sorting, and saves them to an output directory.

```
SAGE:
    ser_extract [FLAGS] [OPTIONS] --inputs <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -q, --quality    Quality estimation sorting
    -v               Show verbose output
    -V, --version    Prints version information

OPTIONS:
    -d, --dark <DARK>            Dark frame image
    -f, --flat <FLAT>            Flat frame image
    -i, --inputs <INPUT>...      Input
    -S, --maxsigma <MAXSIGMA>    Maximum sigma value (quality)
    -s, --minsigma <MINSIGMA>    Minimum sigma value (quality)
    -o, --output <OUTPUT>        Output directory
```