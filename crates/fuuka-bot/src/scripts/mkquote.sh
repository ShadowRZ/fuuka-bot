#!/bin/sh

color="#C0E5F5"
font="Sarasa Gothic SC 16"

avatar=$1
[ -z "${avatar}" ] && avatar=xc:white
text=$2

tempfile=$(mktemp --suffix=.png)

render_text() {
    pango-view \
        --background="${color}" \
        --foreground=black \
        --font="${font}" \
        --antialias=gray \
        --margin=0 \
        --hinting=full \
        --markup \
        --width=500 \
        --wrap=word-char \
        -q \
        -o \
        "${tempfile}" \
        "--text=${text}"
}

mkimage() {
    convert "${tempfile}" -trim -bordercolor "${color}" -border 16 png:- | convert \
        '(' \
            '(' "${avatar}" -resize 64x64 ')' \
            '(' -size 64x64 xc:black -fill white -draw "circle 31.5,31.5 31.5,0" ')' \
            -alpha Off -compose CopyOpacity -composite \
        ')' \
        '(' \
            '(' \
                png:- '(' +clone -alpha extract -draw 'fill black polygon 0,0 0,15 15,0 fill white circle 15,15 15,0' \
                '(' +clone -flip ')' -compose Multiply -composite \
                '(' +clone -flop ')' -compose Multiply -composite \
                ')' -alpha off -compose CopyOpacity -composite \
            ')' \
        ')' -background none +smush +8 png:- | convert png:- -alpha set -bordercolor none -border 8 webp:-
}

set -e
render_text
mkimage
