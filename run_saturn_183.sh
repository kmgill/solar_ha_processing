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

DRIZZLE_SCALE=2.0
TARGET=saturn
IMAGE_ROOT=Sat
IMAGE_DARK_ROOT=Saturn_-_Dark

# location.sh should be an executable script setting the variables
# LOC_LATITUDE and LOC_LONGITUDE for the location the observations
# were made.
source location.sh

IMAGE_MAX_SCALE=95

CROP_WIDTH=2600
CROP_HEIGHT=2600

check_file=`ls -1 $DATAROOT/$IMAGE_ROOT/*/*ser | head -n 1`
BIT_DEPTH=`solha ser-info -i $check_file | grep "Pixel Depth" | cut -d ' ' -f 3`


FRAME_LIMIT=2000

DATA_TS=`ls $DATAROOT/$IMAGE_ROOT/ | tail -n 1`


IMAGE_THRESH=15000
IMAGE_SIGMA_MIN=370.0
IMAGE_SIGMA_MAX=5000.0
IMAGE_TOP_PCT=90

echo Data Root: $DATAROOT
echo Image Root: $DATAROOT/$IMAGE_ROOT
echo Image Dark Root: $DATAROOT/$IMAGE_DARK_ROOT
echo Expected Bit Depth: $BIT_DEPTH
echo Data Timestamp: $DATA_TS
echo Version Text: $VERSION

echo
echo Output Image: $DATAROOT/${TARGET}_${DATA_TS}${VERSION}.png

                # -w $CROP_WIDTH \
                # -h $CROP_HEIGHT \
echo "Starting Planetary Processing..."
solha -v process -i $DATAROOT/$IMAGE_ROOT/*/Sat*ser \
                -d $DATAROOT/$IMAGE_DARK_ROOT/*/Saturn*ser \
                -o $DATAROOT/Saturn_RGB_${DATA_TS}${VERSION}.png \
                -t $IMAGE_THRESH \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q $IMAGE_TOP_PCT \
                -S $IMAGE_SIGMA_MAX \
                -s $IMAGE_SIGMA_MIN \
                -n $FRAME_LIMIT \
                -T moon \
                -u $DRIZZLE_SCALE \
                -P $IMAGE_MAX_SCALE 2>&1 | tee $DATAROOT/${TARGET}_${DATA_TS}${VERSION}.log