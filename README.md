## Running
### First time, beforehand
Create a folder named `output` in the project root.
### Every time
I use FFmpeg to concat the output images into a video:
```

cargo run && ffmpeg -start_number 0 -framerate 60 -i output/%09d.png -vcodec hevc -pix_fmt yuv420p -crf 0 -preset fast -y out.mp4
```