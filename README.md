# Boop - The silly image format 🎨

What if we made an image format that's ridiculously simple? Well, here it is!
Boop just takes your pixels, does some delta encoding magic ✨, and squishes
them with zstd compression. That's literally it!

Funny thing is, it gets really close to PNG compression ratios while being super
fast and simple. Sometimes the silliest ideas work out!

## What's in the box? 📦

- A library that makes images go boop
- A CLI tool to convert images
- A simple viewer built with egui/eframe to actually see what you made

## How to use this silly thing

Want to convert images? First grab the tool:

```bash
cargo install --path .
```

Then boop away:

```bash
# Make it go boop
boop image.png

# Make it go... unboop?
boop image.boop image.png
```

Want to see your booped masterpiece? Get the viewer:

```bash
cargo install --path viewer
```

Then look at that beauty

```bash
boop-viewer image.boop
```

## A little note

This whole thing started as a "what if?" project. It actually works pretty well,
which is kind of surprising! There is probably bugs lurking around, and things
might occasionally go weird, but that's part of the fun.

Feel free to contribute if you want, but remember - this is just a fun
experiment. Don't take it too seriously!

## Coming Up (maybe)

- Some proper benchmarks (to prove it's not completely bonkers)
- A real spec (once I figure out how to write one)
- More optimizations (unlikely)
- Integration with real software like kde and gimp (year of the linux desktop?)

## License 📜

This silly project is licensed under the
[Apache-2.0 License](http://www.apache.org/licenses/LICENSE-2.0). For more
information, please see the [LICENSE](LICENSE) file.

<sub><sup>Made with curiosity and probably too much caffeine ☕</sup></sub>
