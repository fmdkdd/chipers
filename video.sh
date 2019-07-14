#!/bin/sh

# Exit on first error
set -e

GAME=$1

# Grab data
cargo run -- -t ../chip8/c8games/$GAME.c8 > $GAME.ppm

# Split into individual PPM files
mkdir $GAME
cd $GAME
csplit --digits=6 --quiet ../$GAME.ppm '/#/+1' '{*}'
cd ..
rm $GAME.ppm

# Remove first empty PPM, or mogrify will choke
rm $GAME/xx000000

# Convert to PNG and upscale
mkdir $GAME-png
mogrify -format png -scale 640x640 -path $GAME-png $GAME/*
rm -r $GAME

# Convert PNGs to video
ffmpeg -r 60 -f image2 -i $GAME-png/xx%06d.png -vcodec libx264 -crf 25 -pix_fmt yuv420p videos/$GAME.mp4
rm -r $GAME-png
