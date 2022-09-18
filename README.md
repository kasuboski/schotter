# Schotter loop

Following [this](https://github.com/sidwellr/schotter) tutorial to make a schotter image with added color and looping.

`cargo run --release`


`ffmpeg -f image2 -framerate 30 -i shotter4_frames/shotter%04d.png -s 323x548 shotter4-loop.gif`
