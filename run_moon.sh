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

MOON_ROOT=Moon-ASI183MC

# Los Angeles: 34.05 -118.25 
LOC_LATITUDE=0.0
LOC_LONGITUDE=0.0

MOON_MAX_SCALE=95

CROP_WIDTH=1300
CROP_HEIGHT=1300

check_file=`ls -1 $DATAROOT/$MOON_ROOT/*/*ser | head -n 1`
BIT_DEPTH=`ser_info -i $check_file | grep "Pixel Depth" | cut -d ' ' -f 3`


FRAME_LIMIT=2000

DATA_TS=`ls $DATAROOT/$MOON_ROOT/ | tail -n 1`


MOON_THRESH=20560
MOON_SIGMA_MIN=1.0
MOON_SIGMA_MAX=5.0
MOON_TOP_PCT=80

echo Data Root: $DATAROOT
echo Moon Root: $DATAROOT/$MOON_ROOT
echo Expected Bit Depth: $BIT_DEPTH
echo Data Timestamp: $DATA_TS
echo Version Text: $VERSION

echo
echo Output Moon: $DATAROOT/Moon_${DATA_TS}${VERSION}.png

echo "Starting Moon Processing..."
process_ha -v -i $DATAROOT/$MOON_ROOT/*/Moon*ser \
                -d $DATAROOT/$MOON_ROOT/*/Dark*ser \
                -o $DATAROOT/Moon_RGB_${DATA_TS}${VERSION}.png \
                -t $MOON_THRESH \
                -w $CROP_WIDTH \
                -h $CROP_HEIGHT \
                -l $LOC_LATITUDE \
                -L $LOC_LONGITUDE \
                -q $MOON_TOP_PCT \
                -S $MOON_SIGMA_MAX \
                -s $MOON_SIGMA_MIN \
                -n $FRAME_LIMIT \
                -T moon \
                --norot \
                -P $MOON_MAX_SCALE 2>&1 | tee $DATAROOT/moon_${DATA_TS}${VERSION}.log