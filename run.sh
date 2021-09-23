#!/bin/bash

if [ $# -lt 1 ]; then
    echo "USAGE: run.sh </dataroot>"
    exit 1
fi

MASKROOT=~/repos/solar_ha_processing/masks/
DATAROOT=$1

CHROME_ROOT=Sun
PROM_ROOT=Sun_-_Prominance
DARK_ROOT=Sun_-_Dark
FLAT_ROOT=Sun_-_Flat
PHOTO_ROOT=Sun_-_Whitelight_-_Tamron

# Los Angeles: 34.05 -118.25 
LOC_LATITUDE=0.0
LOC_LONGITUDE=0.0

CHROME_MAX_SCALE=80
PROM_MAX_SCALE=100
PHOTO_MAX_SCALE=90

# 8 Bit
CHROME_THRESH=80
CHROME_SIGMA_MIN=1.8
CHROME_SIGMA_MAX=5.0

PROM_THRESH=160
PROM_SIGMA_MIN=1.6
PROM_SIGMA_MAX=2.0

PHOTO_THRESH=80
PHOTO_SIGMA_MIN=1.23
PHOTO_SIGMA_MAX=2.0

# 16 Bit
# CHROME_THRESH=20560
# CHROME_SIGMA_MIN=1.6
# CHROME_SIGMA_MAX=3.0

# PROM_THRESH=40960
# PROM_SIGMA_MIN=1.7
# PROM_SIGMA_MAX=5.0

# PHOTO_THRESH=20560
# PHOTO_SIGMA_MIN=1.23
# PHOTO_SIGMA_MAX=2.0

if [ ! -d $DATAROOT ]; then
    echo "Error: Data root not found: $DATAROOT"
    exit 1
fi

if [ $# -eq 2 ]; then
    VERSION=_$2
else
    VERSION=""
fi

DATA_TS=`ls $DATAROOT/$CHROME_ROOT/ | tail -n 1`

echo Data Root: $DATAROOT
echo Chromosphere Root: $DATAROOT/$CHROME_ROOT
echo Prominance Root: $DATAROOT/$PROM_ROOT
echo Photosphere Root: $DATAROOT/$PHOTO_ROOT
echo Flat Root: $DATAROOT/$FLAT_ROOT
echo Dark Root: $DATAROOT/$DARK_ROOT
echo Data Timestamp: $DATA_TS
echo Version Text: $VERSION
echo Chromosphere Threshold: $CHROME_THRESH
echo Prominance Threshold: $PROM_THRESH
echo Photosphere Threshold: $PHOTO_THRESH

echo
echo Output Chromosphere: $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png
echo Output Prominance: $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png
echo Output Composite: $DATAROOT/Sun_Composite_${DATA_TS}${VERSION}.png
echo Output Photosphere: $DATAROOT/Sun_Photo_${DATA_TS}${VERSION}.png

echo
echo Including Chromosphere Input\(s\):
ls -1 $DATAROOT/$CHROME_ROOT/*/*ser 
echo
echo Including Prominance Input\(s\):
ls -1 $DATAROOT/$PROM_ROOT/*/*ser 
echo
echo Including Darkfield input\(s\):
ls -1 $DATAROOT/$DARK_ROOT/*/*ser
echo
echo Including Flatfield inpu\(s\):
ls -1 $DATAROOT/$FLAT_ROOT/*/*ser
echo
if [ -d $DATAROOT/$PHOTO_ROOT ]; then
    echo Including Photosphere Input\(s\):
    ls -1 $DATAROOT/$PHOTO_ROOT/*/*ser 
    echo
fi

echo "Starting Chromosphere Processing..."
process_ha -v -i $DATAROOT/$CHROME_ROOT/*/*ser \
                -d $DATAROOT/$DARK_ROOT/*/*ser \
                -f $DATAROOT/$FLAT_ROOT/*/*ser \
                -o $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png \
                -t $CHROME_THRESH \
                -w 1200 \
                -h 1200 \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q 25 \
                -S $CHROME_SIGMA_MAX \
                -s $CHROME_SIGMA_MIN \
                -P $CHROME_MAX_SCALE
                #-m $MASKROOT/Sun_Chromosphere_1200x1200_v2.png

echo "Starting Prominance Processing..."
process_ha -v -i $DATAROOT/$PROM_ROOT/*/*ser \
                -d $DATAROOT/$DARK_ROOT/*/*ser \
                -f $DATAROOT/$FLAT_ROOT/*/*ser \
                -o $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png \
                -t $PROM_THRESH \
                -w 1200 \
                -h 1200 \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q 25 \
                -S $PROM_SIGMA_MAX \
                -s $PROM_SIGMA_MIN \
                -P $PROM_MAX_SCALE
                #-m $MASKROOT/Sun_Prominence_1200x1200_v2.png


echo "Assembling Chrome/Prom Composite..."
ha_subtract -i $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png \
          $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png \
          -o $DATAROOT/Sun_Composite_${DATA_TS}${VERSION}.png \
          -v


if [ -d $DATAROOT/$PHOTO_ROOT ]; then
    echo "Starting Photosphere Processing..."
    process_ha -v -i $DATAROOT/$PHOTO_ROOT/*/*ser \
                -o $DATAROOT/Sun_Photo_${DATA_TS}${VERSION}.png \
                -t $PHOTO_THRESH \
                -w 1200 \
                -h 1200 \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q 25 \
                -S $PHOTO_SIGMA_MAX \
                -s $PHOTO_SIGMA_MIN \
                -P $PHOTO_MAX_SCALE
                #-m $MASKROOT/Sun_Prominence_1200x1200_v2.png
fi


echo Done