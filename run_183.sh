#!/bin/bash

if [ $# -lt 1 ]; then
    echo "USAGE: run.sh </dataroot>"
    exit 1
fi

MASKROOT=~/repos/solar_ha_processing/masks/
DATAROOT=$1

CHROME_ROOT=Sun_-_ASI183MC-P
DARK_ROOT=Sun_-_ASI183MC-P_-_Dark
FLAT_ROOT=Sun_-_ASI183MC-P_-_Flat
DARK_FLAT_ROOT=Sun_-_ASI183MC-P_-_Flat_Dark

# location.sh should be an executable script setting the variables
# LOC_LATITUDE and LOC_LONGITUDE for the location the observations
# were made.
source location.sh

CHROME_MAX_SCALE=99
PROM_MAX_SCALE=100
PHOTO_MAX_SCALE=90

CROP_WIDTH=1600
CROP_HEIGHT=1600



check_file=`ls -1 $DATAROOT/$CHROME_ROOT/*/*ser | head -n 1`
BIT_DEPTH=`ser_info -i $check_file | grep "Pixel Depth" | cut -d ' ' -f 3`

INITIAL_ROTATION=`solha frame-stats -i $check_file  -l $LOC_LATITUDE -L $LOC_LONGITUDE 2> /dev/null | head -n 2 | tail -n 1 | tr -s ' '  | cut -d ' ' -f 6`

FRAME_LIMIT=2000

if [ $BIT_DEPTH -eq 8 ]; then
    # 8 Bit
    CHROME_THRESH=80
    CHROME_SIGMA_MIN=1.6
    CHROME_SIGMA_MAX=5.0
    CHROME_TOP_PCT=15

elif [ $BIT_DEPTH -eq 16 ]; then
    # 16 Bit
    CHROME_THRESH=20560
    CHROME_SIGMA_MIN=220
    CHROME_SIGMA_MAX=1285
    CHROME_TOP_PCT=20

else
    echo Unsupported bit depth: $BIT_DEPTH
    exit
fi

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
echo Flat Root: $DATAROOT/$FLAT_ROOT
echo Dark Root: $DATAROOT/$DARK_ROOT
echo Dark Flat Root: $DATAROOT/$DARK_FLAT_ROOT
echo Expected Bit Depth: $BIT_DEPTH
echo Data Timestamp: $DATA_TS
echo Version Text: $VERSION
echo Chromosphere Threshold: $CHROME_THRESH
echo Chromosphere Top Percentage: $CHROME_TOP_PCT
echo Initial Rotation: $INITIAL_ROTATION

echo
echo Output Chromosphere: $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png



echo
echo Including Chromosphere Input\(s\):
ls -1 $DATAROOT/$CHROME_ROOT/*/*ser 
echo

echo Including Darkfield input\(s\):
ls -1 $DATAROOT/$DARK_ROOT/*/*ser
echo
echo Including Flatfield inpu\(s\):
ls -1 $DATAROOT/$FLAT_ROOT/*/*ser
echo


echo "Starting Chromosphere Processing..."
solha -v process -i $DATAROOT/$CHROME_ROOT/*/*ser \
                -d $DATAROOT/$DARK_ROOT/*/*ser \
                -f $DATAROOT/$FLAT_ROOT/*/*ser \
                -D $DATAROOT/$DARK_FLAT_ROOT/*/*ser \
                -o $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png \
                -t $CHROME_THRESH \
                -w $CROP_WIDTH \
                -H $CROP_HEIGHT \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q $CHROME_TOP_PCT \
                -S $CHROME_SIGMA_MAX \
                -s $CHROME_SIGMA_MIN \
                -n $FRAME_LIMIT \
                -I $INITIAL_ROTATION \
                -T sun \
                -P $CHROME_MAX_SCALE 2>&1 | tee $DATAROOT/chromosphere_${DATA_TS}${VERSION}.log
                #-m $MASKROOT/Sun_Chromosphere_1200x1200_v2.png





echo Done