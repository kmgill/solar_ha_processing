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
PROM_DARK_ROOT=Sun_-_Prominance_-_Dark
FLAT_ROOT=Sun_-_Flat
DARK_FLAT_ROOT=Sun_-_Flat_Dark
PHOTO_ROOT=Sun_-_Whitelight_-_Tamron
PHOTO_DARK_ROOT=Sun_-_Whitelight_-_Tamron_-_Dark
PHOTO_FLAT_ROOT=Sun_-_Whitelight_-_Tamron_-_Flat
PHOTO_DARK_FLAT_ROOT=Sun_-_Whitelight_-_Tamron_-_Flat_Dark


# location.sh should be an executable script setting the variables
# LOC_LATITUDE and LOC_LONGITUDE for the location the observations
# were made.
source location.sh


CHROME_MAX_SCALE=99
PROM_MAX_SCALE=100
PHOTO_MAX_SCALE=90

CROP_WIDTH=1200
CROP_HEIGHT=1200

DRIZZLE_SCALE=2.0

check_file=`ls -1 $DATAROOT/$CHROME_ROOT/*/*ser | head -n 1`
BIT_DEPTH=`solha ser-info -i $check_file | grep "Pixel Depth" | cut -d ' ' -f 3`

INITIAL_ROTATION=`solha frame-stats -i $check_file  -l $LOC_LATITUDE -L $LOC_LONGITUDE 2> /dev/null | head -n 2 | tail -n 1 | tr -s ' '  | cut -d ' ' -f 6`

FRAME_LIMIT=2000

if [ $BIT_DEPTH -eq 8 ]; then
    # 8 Bit
    CHROME_THRESH=80
    CHROME_SIGMA_MIN=1.6
    CHROME_SIGMA_MAX=5.0
    CHROME_TOP_PCT=15

    PROM_THRESH=160
    PROM_SIGMA_MIN=1.6
    PROM_SIGMA_MAX=5.0
    PROM_TOP_PCT=50

    PHOTO_THRESH=80
    PHOTO_SIGMA_MIN=0.9
    PHOTO_SIGMA_MAX=2.0
    PHOTO_TOP_PCT=30
elif [ $BIT_DEPTH -eq 16 ]; then
    # 16 Bit
    CHROME_THRESH=20560
    CHROME_SIGMA_MIN=349
    CHROME_SIGMA_MAX=1285
    CHROME_TOP_PCT=80

    PROM_THRESH=50000
    PROM_SIGMA_MIN=349
    PROM_SIGMA_MAX=1285
    PROM_TOP_PCT=20

    PHOTO_THRESH=20560
    PHOTO_SIGMA_MIN=155.0
    PHOTO_SIGMA_MAX=771
    PHOTO_TOP_PCT=40
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
echo Prominance Root: $DATAROOT/$PROM_ROOT
echo Photosphere Root: $DATAROOT/$PHOTO_ROOT
echo Flat Root: $DATAROOT/$FLAT_ROOT
echo Dark Root: $DATAROOT/$DARK_ROOT
echo Prominance Dark Root: $DATAROOT/$PROM_DARK_ROOT
echo Dark Flat Root: $DATAROOT/$DARK_FLAT_ROOT
echo Expected Bit Depth: $BIT_DEPTH
echo Data Timestamp: $DATA_TS
echo Version Text: $VERSION
echo Chromosphere Threshold: $CHROME_THRESH
echo Chromosphere Top Percentage: $CHROME_TOP_PCT
echo Prominance Threshold: $PROM_THRESH
echo Prominance Top Percentage: $PROM_TOP_PCT
echo Photosphere Threshold: $PHOTO_THRESH
echo Photosphere Top Percentage: $PHOTO_TOP_PCT
echo Initial Rotation: $INITIAL_ROTATION
echo Drizzle Upscale Amount: $DRIZZLE_SCALE

echo
echo Output Chromosphere: $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png
echo Output Prominance: $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png
echo Output Composite: $DATAROOT/Sun_Composite_${DATA_TS}${VERSION}.png
echo Output Photosphere: $DATAROOT/Sun_Photo_${DATA_TS}${VERSION}.png

echo
echo Observation Latitude: $LOC_LATITUDE
echo Observation Longitude: $LOC_LONGITUDE

HAS_PROM=0
if [ -d $DATAROOT/$PROM_ROOT ]; then
    HAS_PROM=1
fi

HAS_PHOTO=0
if [ -d $DATAROOT/$PHOTO_ROOT ]; then
    HAS_PHOTO=1
fi

echo
echo Including Chromosphere Input\(s\):
ls -1 $DATAROOT/$CHROME_ROOT/*/*ser 
echo
if [ $HAS_PROM -eq 1 ]; then
    echo Including Prominance Input\(s\):
    ls -1 $DATAROOT/$PROM_ROOT/*/*ser 
    echo
fi 
if [ $HAS_PHOTO -eq 1 ]; then
    echo Including Photosphere Input\(s\):
    ls -1 $DATAROOT/$PHOTO_ROOT/*/*ser 
    echo
fi
echo Including Darkfield input\(s\):
ls -1 $DATAROOT/$DARK_ROOT/*/*ser
echo
echo Including Flatfield inpu\(s\):
ls -1 $DATAROOT/$FLAT_ROOT/*/*ser
echo

                # -D $DATAROOT/$DARK_FLAT_ROOT/*/*ser \
echo "Starting Chromosphere Processing..."
solha -v process -i $DATAROOT/$CHROME_ROOT/*/*ser \
                -d $DATAROOT/$DARK_ROOT/*/*ser \
                -f $DATAROOT/$FLAT_ROOT/*/*ser \
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
                -I 0 \
                -T sun \
                -u $DRIZZLE_SCALE \
                -P $CHROME_MAX_SCALE 2>&1 | tee $DATAROOT/chromosphere_${DATA_TS}${VERSION}.log
                #-m $MASKROOT/Sun_Chromosphere_1200x1200_v2.png
 
if [ $HAS_PROM -eq 1 ]; then
    echo "Starting Prominance Processing..."
    solha -v process -i $DATAROOT/$PROM_ROOT/*/*ser \
                    -d $DATAROOT/$PROM_DARK_ROOT/*/*ser \
                    -f $DATAROOT/$FLAT_ROOT/*/*ser \
                    -D $DATAROOT/$DARK_FLAT_ROOT/*/*ser \
                    -o $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png \
                    -t $PROM_THRESH \
                    -w $CROP_WIDTH \
                    -H $CROP_HEIGHT \
                    -l $LOC_LATITUDE \
                    -L $LOC_LONGITUDE \
                    -q $PROM_TOP_PCT \
                    -S $PROM_SIGMA_MAX \
                    -s $PROM_SIGMA_MIN \
                    -n $FRAME_LIMIT \
                    -I $INITIAL_ROTATION \
                    -T sun \
                    -u $DRIZZLE_SCALE \
                    -P $PROM_MAX_SCALE 2>&1 | tee $DATAROOT/prominance_${DATA_TS}${VERSION}.log
                    #-m $MASKROOT/Sun_Prominence_1200x1200_v2.png


    echo "Assembling Chrome/Prom Composite..."
    solha -v composite -i $DATAROOT/Sun_Prom_${DATA_TS}${VERSION}.png \
            $DATAROOT/Sun_Chrome_${DATA_TS}${VERSION}.png \
            -o $DATAROOT/Sun_Composite_${DATA_TS}${VERSION}.png \
            -s 0.7 \
             2>&1 | tee $DATAROOT/composite_${DATA_TS}${VERSION}.log
fi

if [ $HAS_PHOTO -eq 1 ]; then
    echo "Starting Photosphere Processing..."
    solha -v process -i $DATAROOT/$PHOTO_ROOT/*/*ser \
                -o $DATAROOT/Sun_Photo_${DATA_TS}${VERSION}.png \
                -d $DATAROOT/$PHOTO_DARK_ROOT/*/*ser \
                -f $DATAROOT/$PHOTO_FLAT_ROOT/*/*ser \
                -D $DATAROOT/$PHOTO_DARK_FLAT_ROOT/*/*ser \
                -t $PHOTO_THRESH \
                -w $CROP_WIDTH \
                -H $CROP_HEIGHT \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q $PHOTO_TOP_PCT \
                -S $PHOTO_SIGMA_MAX \
                -s $PHOTO_SIGMA_MIN \
                -I $INITIAL_ROTATION \
                -T sun \
                -u $DRIZZLE_SCALE \
                -P $PHOTO_MAX_SCALE 2>&1 | tee $DATAROOT/photosphere_${DATA_TS}${VERSION}.log
fi


echo Done