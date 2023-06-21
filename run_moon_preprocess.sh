#!/bin/bash

if [ $# -lt 1 ]; then
    echo "USAGE: run.sh </dataroot>"
    exit 1
fi

MASKROOT=~/repos/solar_ha_processing/masks/
DATAROOT=$1

if [ $# -eq 2 ]; then
    VERSION=_$2
else
    VERSION=""
fi

MOON_ROOT=Moon
MOON_DARK_ROOT=Moon_-_Dark
MOON_OUTPUT=$DATAROOT/preprocessed

# location.sh should be an executable script setting the variables
# LOC_LATITUDE and LOC_LONGITUDE for the location the observations
# were made.
source location.sh

MOON_MAX_SCALE=95

CROP_WIDTH=2400
CROP_HEIGHT=2400
check_file=`ls -1 $DATAROOT/$MOON_ROOT/*/*ser | head -n 1`
BIT_DEPTH=`solha ser-info -i $check_file | grep "Pixel Depth" | cut -d ' ' -f 3`


FRAME_LIMIT=1000

DATA_TS=`ls $DATAROOT/$MOON_ROOT/ | tail -n 1`

MOON_THRESH=2000
MOON_SIGMA_MIN=0.0
MOON_SIGMA_MAX=5000.0
MOON_TOP_PCT=40
DRIZZLE_SCALE=1.0

echo Data Root: $DATAROOT
echo Moon Root: $DATAROOT/$MOON_ROOT
echo Moon Dark Root: $DATAROOT/$MOON_DARK_ROOT
echo Expected Bit Depth: $BIT_DEPTH
echo Data Timestamp: $DATA_TS
echo Version Text: $VERSION

echo
echo Output Moon: $DATAROOT/Moon_${DATA_TS}${VERSION}.png

DARK_FRAME=$DATAROOT/Dark_${DATA_TS}${VERSION}.png
# if [ ! -f $DARK_FRAME ]; then
#     echo Creating calibration frames...
#     solha -v mean -i $DATAROOT/$MOON_DARK_ROOT/*/*ser -o $DARK_FRAME
#     if [ ! -f $DARK_FRAME -o $? -ne 0 ]; then
#         echo Error: Failed to generate dark frame
#         exit 1
#     fi
# fi

if [ ! -d $MOON_OUTPUT ]; then
    mkdir $MOON_OUTPUT
fi


echo Generating threshold test frame...
solha -v thresh-test -i $DATAROOT/$MOON_ROOT/*/*ser \
                -d $DARK_FRAME \
                -o $DATAROOT/ThreshTest_${DATA_TS}${VERSION}.png \
                -t $MOON_THRESH

echo "Starting Moon Processing..."
solha -v pre-process -i $DATAROOT/$MOON_ROOT/*/Moon*ser \
                -o $MOON_OUTPUT \
                -t $MOON_THRESH \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -n $FRAME_LIMIT \
                -T moon \
                -u $DRIZZLE_SCALE \
                -q 
