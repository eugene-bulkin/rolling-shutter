# rolling-shutter
A program that generates rolling-shutter images from a set of frames.

To run, simply run the binary with a file mask (full folders will be supported later) and an output, and optionally a 
direction:

```
rolling-shutter frames/%03d.png -o out.png -d N
```

Which will take all frames `frames/000.png` to `frames/999.png` if they exist. Make sure that the file mask has room for
all the frames you want. The program will take the first frame that exists starting at 0 and stop once it doesn't find a
new frame sequentially (even if there are more after that; i.e. a gap).