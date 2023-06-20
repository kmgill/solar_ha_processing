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
BIAS_ROOT=Sun_-_Bias


# location.sh should be an executable script setting the variables
# LOC_LATITUDE and LOC_LONGITUDE for the location the observations
# were made.
source location.sh


CHROME_MAX_SCALE=99
PROM_MAX_SCALE=100
PHOTO_MAX_SCALE=90

CROP_WIDTH=1200
CROP_HEIGHT=1200

DRIZZLE_SCALE=1.0

check_file=`ls -1 $DATAROOT/$CHROME_ROOT/*/*ser | head -n 1`
BIT_DEPTH=`solha ser-info -i $check_file | grep "Pixel Depth" | cut -d ' ' -f 3`

INITIAL_ROTATION=`solha frame-stats -i $check_file  -l $LOC_LATITUDE -L $LOC_LONGITUDE 2> /dev/null | head -n 2 | tail -n 1 | tr -s ' '  | cut -d ' ' -f 6`

FRAME_LIMIT=1000

CHROME_OUTPUT=$DATAROOT/preprocessed
PROM_OUTPUT=$DATAROOT/preprocessed-prom
PHOTO_OUTPUT=$DATAROOT/preprocessed-photo


if [ $BIT_DEPTH -eq 8 ]; then
    # 8 Bit
    CHROME_THRESH=80
    CHROME_SIGMA_MIN=1.6
    CHROME_SIGMA_MAX=5.0
    CHROME_TOP_PCT=75

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
    CHROME_SIGMA_MIN=0
    CHROME_SIGMA_MAX=1500
    CHROME_TOP_PCT=30

    PROM_THRESH=14000
    PROM_SIGMA_MIN=0
    PROM_SIGMA_MAX=1500
    PROM_TOP_PCT=20

    PHOTO_THRESH=15000
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
echo Bias Root:  $DATAROOT/$BIAS_ROOT
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
echo Output Chromosphere: $CHROME_OUTPUT
echo Output Prominance: $PROM_OUTPUT
echo Output Photosphere: $PHOTO_OUTPUT
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


DARK_FRAME=$DATAROOT/Dark_${DATA_TS}${VERSION}.png
FLAT_FRAME=$DATAROOT/Flat_${DATA_TS}${VERSION}.png
DARKFLAT_FRAME=$DATAROOT/Dark-Flat_${DATA_TS}${VERSION}.png
BIAS_FRAME=$DATAROOT/Bias_${DATA_TS}${VERSION}.png

echo Creating calibration frames...
if [ ! -f $DARK_FRAME ]; then
    solha -v mean -i $DATAROOT/$DARK_ROOT/*/*ser -o $DARK_FRAME
    if [ ! -f $DARK_FRAME -o $? -ne 0 ]; then
        echo Error: Failed to generate dark frame
    fi
fi

if [ ! -f $FLAT_FRAME ]; then
    solha -v mean -i $DATAROOT/$FLAT_ROOT/*/*ser -o $FLAT_FRAME
    if [ ! -f $FLAT_FRAME -o $? -ne 0 ]; then
        echo Error: Failed to generate flat frame
    fi
fi

if [ ! -f $DARKFLAT_FRAME ]; then
    solha -v mean -i $DATAROOT/$DARK_FLAT_ROOT/*/*ser -o $DARKFLAT_FRAME
    if [ ! -f $DARKFLAT_FRAME -o $? -ne 0 ]; then
        echo Error: Failed to generate flat-dark frame
    fi
fi

if [ ! -f $BIAS_FRAME ]; then
    solha -v mean -i $DATAROOT/$BIAS_ROOT/*/*ser -o $BIAS_FRAME
    if [ ! -f $BIAS_FRAME -o $? -ne 0 ]; then
        echo Error: Failed to generate bias frame
    fi
fi

echo Generating threshold test frame...
solha -v thresh-test -i $DATAROOT/$CHROME_ROOT/*/*ser \
                -d $DARK_FRAME \
                -f $FLAT_FRAME \
                -D $DARKFLAT_FRAME \
                -o $DATAROOT/ThreshTest_${DATA_TS}${VERSION}.png \
                -t $CHROME_THRESH

if [ $? -ne 0 ]; then
    echo Warning: Failed to generate threshold test image
fi
 
if [ ! -d $CHROME_OUTPUT ]; then
    mkdir $CHROME_OUTPUT
fi

echo "Starting Chromosphere Processing..."
export STUMP_LOG_AT_LEVEL=debug
echo "Running solha command:"
echo solha -v pre-process -i $DATAROOT/$CHROME_ROOT/*/*ser \
                -d $DARK_FRAME \
                -f $FLAT_FRAME \
                -D $DARKFLAT_FRAME \
                -b $BIAS_FRAME \
                -o $CHROME_OUTPUT \
                -t $CHROME_THRESH \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -S $CHROME_SIGMA_MAX \
                -s $CHROME_SIGMA_MIN \
                -I 0 \
                -u $DRIZZLE_SCALE \
                -n $FRAME_LIMIT \
                -q \
                -T sun 

solha -v pre-process -i $DATAROOT/$CHROME_ROOT/*/*ser \
                -d $DARK_FRAME \
                -f $FLAT_FRAME \
                -D $DARKFLAT_FRAME \
                -b $BIAS_FRAME \
                -o $DATAROOT/preprocessed \
                -t $CHROME_THRESH \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -S $CHROME_SIGMA_MAX \
                -s $CHROME_SIGMA_MIN \
                -I 0 \
                -T sun \
                -u $DRIZZLE_SCALE \
                -n $FRAME_LIMIT \
                -q 


HAS_PROM=0
if [ -d $DATAROOT/$PROM_ROOT ]; then
    HAS_PROM=1
fi

if [ $HAS_PROM -eq 1 ]; then

    echo "Starting Prominence Processing..."
    if [ ! -d $PROM_OUTPUT ]; then
        mkdir $PROM_OUTPUT
    fi

    PROM_DARK_FRAME=$DATAROOT/Prom_Dark_${DATA_TS}${VERSION}.png
    echo Creating calibration frames...
    if [ ! -f $PROM_DARK_FRAME ]; then
        solha -v mean -i $DATAROOT/$PROM_DARK_ROOT/*/*ser -o $PROM_DARK_FRAME
        if [ ! -f $PROM_DARK_FRAME -o $? -ne 0 ]; then
            echo Error: Failed to generate dark frame
        fi
    fi

    echo "Running solha command:"
    echo solha -v pre-process -i $DATAROOT/$PROM_ROOT/*/*ser \
                -d $PROM_DARK_FRAME \
                -f $FLAT_FRAME \
                -D $DARKFLAT_FRAME \
                -b $BIAS_FRAME \
                -o $PROM_OUTPUT \
                -t $PROM_THRESH \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -S $PROM_SIGMA_MAX \
                -s $PROM_SIGMA_MIN \
                -I 0 \
                -T sun \
                -u $DRIZZLE_SCALE \
                -n $FRAME_LIMIT

    solha -v pre-process -i $DATAROOT/$PROM_ROOT/*/*ser \
                -d $PROM_DARK_FRAME \
                -f $FLAT_FRAME \
                -D $DARKFLAT_FRAME \
                -b $BIAS_FRAME \
                -o $PROM_OUTPUT \
                -t $PROM_THRESH \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -s $PROM_SIGMA_MIN \
                -I 0 \
                -T sun \
                -u $DRIZZLE_SCALE \
                -n $FRAME_LIMIT \
                -q
fi