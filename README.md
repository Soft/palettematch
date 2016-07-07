# Palettematch :rainbow:

palettematch — Find similar colors from a limited palette. Have you ever been
in a situation where you had to find a color from a predetermined palette that
most closely resembled some other color? I know, we have all been there. Well,
you're in luck since palettematch makes finding similar colors a breeze. Color
similarity is calculated using either CIE76 or CIE94 formulas.

Let's say you have your set of available colors neatly listed in a file
`palette.txt` and you want to know which one of them resembles lime green
(`#00ff00`) the most. This is easily figured out:

	echo "#00ff00" | palettematch palette.txt

## Input formats

At present, palettematch only supports #rrggbb hex triplets as its input format.
The colors are assumed to be in sRGB color space.

## Fun Things

In the scripts directory there are few fun examples of what can be achieved with
palettematch. `scripts/color-squares.sh` script produces stunning presentations
of RGB space using only ANSI escape sequences and the 256 colors supported by
xterm. `scripts/catff.sh` displays
[farbfeld](http://tools.suckless.org/farbfeld/) images in terminal. Now you can
finally replace your old and bloated image viewer with a simple Bash script!

[!lenna](https://i.imgur.com/SwHfhj2.png)

