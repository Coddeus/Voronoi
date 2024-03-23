# Voronoi
This project generates images of Voronoi diagrams in the `output` folder. 

## Prerequisites
- A Vulkan-compatible computer
- Vulkan SDK
- Rust
- This repository cloned

## Running
### First time, beforehand
Create a folder named `output` in the cloned project root.
### Every time
After the images generation, I use FFmpeg to concat the output images into a video.  
Useful commands:
``` bash
# Dev
cargo run && ffmpeg -start_number 0 -framerate 60 -i output/%09d.png -vcodec h264 -frames:v 600 -pix_fmt yuv420p -crf 23 -preset veryfast -y out.mp4
cargo run --release && ffmpeg -start_number 0 -framerate 60 -i output/%09d.png -vcodec h264 -frames:v 600 -pix_fmt yuv420p -crf 23 -preset veryfast -y out.mp4

# Better quality
cargo run --release && ffmpeg -start_number 0 -framerate 60 -i output/%09d.png -vcodec h264 -frames:v 600 -pix_fmt yuv420p -crf 0 -preset veryslow -y out.mp4
```
