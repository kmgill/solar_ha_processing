# SolHAT: Solar Hydrogen Alpha Telescope Imaging Pipeline

## Overview
A set of utilities and a pipeline for processing raw hydrogen-alpha solar imaging using lucky imaging and drizzle ("image stacking"). Designed as a specific use-case replacement for PIPP & Autostakkert. 

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
 * Debayering (partially implemented)
 * Stacking with Drizzle (1.0x, 1.5x, 2.0x, 3.0x)
 * Support for Solar and Lunar targeting

Future Plans:
 * GUI Support
 * Planetary


## Contributing
Feedback, issues, and contributions are always welcomed. Should enough interest arise in contributing development efforts, I will write up a contribution guide. 


## Building from source
A working Rust (https://www.rust-lang.org/) installation is required for building.

So far I've only tested building on Ubuntu 21.10, natively and within the Windows Subsystem for Linux on Windows 10, and on MacOSX Catalina. Within the project folder, the software can be built for testing via `cargo build` and individual binaries can be run in debug mode via, for example, `cargo run --bin solha -- -h`


### Clone from git
```
git clone git@github.com:kmgill/solar_ha_processing.git
```

### Install via cargo
This is the easiest installation method for *nix-based systems. It has not been tested in Windows.

```
cargo install --path .
```

### Install via apt (Debian, Ubuntu, ...)
*Not yet*

### Install via rpm (RHEL, CentOS, Fedora, ...)
*Not yet*

### Install on MacOS via Homebrew
*Not yet*


## References:
Telea, Alexandru. (2004). An Image Inpainting Technique Based on the Fast Marching Method. Journal of Graphics Tools. 9. 10.1080/10867651.2004.10487596. 
https://www.researchgate.net/publication/238183352_An_Image_Inpainting_Technique_Based_on_the_Fast_Marching_Method

Malvar, Henrique & He, Li-wei & Cutler, Ross. (2004). High-quality linear interpolation for demosaicing of Bayer-patterned color images. Acoustics, Speech, and Signal Processing, 1988. ICASSP-88., 1988 International Conference on. 3. iii - 485. 10.1109/ICASSP.2004.1326587. 
https://www.researchgate.net/publication/4087683_High-quality_linear_interpolation_for_demosaicing_of_Bayer-patterned_color_images

Getreuer, Pascal. (2011). Malvar-He-Cutler Linear Image Demosaicking. Image Processing On Line. 1. 10.5201/ipol.2011.g_mhcd. 
https://www.researchgate.net/publication/270045976_Malvar-He-Cutler_Linear_Image_Demosaicking

Di, K., and Li, R. (2004), CAHVOR camera model and its photogrammetric conversion for planetary applications, J. Geophys. Res., 109, E04004, doi:10.1029/2003JE002199.
https://doi.org/10.1029/2003JE002199

Gennery, D.B. Generalized Camera Calibration Including Fish-Eye Lenses. Int J Comput Vision 68, 239–266 (2006). https://doi.org/10.1007/s11263-006-5168-1