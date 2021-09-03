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

# Los Angeles: 34.05 -118.25 
LOC_LATITUDE=0.0
LOC_LONGITUDE=0.0

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
echo Flat Root: $DATAROOT/$FLAT_ROOT
echo Dark Root: $DATAROOT/$DARK_ROOT
echo Data Timestamp: $DATA_TS
echo Version Text: $VERSION

echo
echo Output Chromosphere: $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png
echo Output Prominance: $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png
echo Output Composite: $DATAROOT/Sun_Composite_${DATA_TS}${VERSION}.png

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


echo "Starting Chromosphere Processing..."
process_ha -v -i $DATAROOT/$CHROME_ROOT/*/*ser \
                -d $DATAROOT/$DARK_ROOT/*/*ser \
                -f $DATAROOT/$FLAT_ROOT/*/*ser \
                -o $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png \
                -t 80 \
                -w 1200 \
                -h 1200 \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q 25 \
                -S 3.0 \
                -s 1.8 \
                -m $MASKROOT/Sun_Chromosphere_1200x1200_v2.png


echo "Starting Prominance Processing..."
process_ha -v -i $DATAROOT/$PROM_ROOT/*/*ser \
                -d $DATAROOT/$DARK_ROOT/*/*ser \
                -f $DATAROOT/$FLAT_ROOT/*/*ser \
                -o $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png \
                -t 160 \
                -w 1200 \
                -h 1200 \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q 25 \
                -S 2.0 \
                -s 1.6 \
                -m $MASKROOT/Sun_Prominence_1200x1200_v2.png


echo "Assembling Chrome/Prom Composite..."
ha_add -i $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png \
          $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png \
          -o $DATAROOT/Sun_Composite_${DATA_TS}${VERSION}.png \
          -v

echo Done