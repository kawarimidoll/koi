<h1 align="center">恋 (koi)</h1>

恋 (koi) text editor: A practice to build my own text editor

## planned features

I'm not sure when these will be done or in what order.

### basic features

- [x] insert characters
- [x] insert newline
- [x] delete characters
- [x] save file
- [x] status line
- [x] message line
- [ ] search

### advanced features

- [ ] wrap long lines
- [x] modal editing
- [ ] redo / undo
- [ ] repeat editing
- [ ] copy / paste
- [ ] multiple buffers
- [ ] multiple windows
- [ ] syntax highlight
- [ ] jump to previous positions
- [ ] auto indent
- [ ] fold lines
- [ ] line number
- [ ] regex search
- [ ] user configuration
- [ ] completion
- [ ] interact with external process

## License

This project is published under MIT license.

### Attribution & Indication of Changes

Since this work is based on prior work licensed under CC BY 4.0, I am required
to properly attribute the original authors indicate the changes I did to the
original work:

- The original text editor is `kilo` written in C, done by
  [antirez](http://invece.org/).
- The tutorial which built based on `kilo` was done by
  [Paige Ruten](https://viewsourcecode.org/).
- The Rust version is `hecto`, its code and
  [tutorial](https://www.flenker.blog/hecto/) was written by
  [Philipp Flenker](https://philippflenker.com/).

- This project is based on Hecto. The initial code and the idea of using
  TextFragment were inherited from Hecto.
- I modified the struct definitions from Hecto and added features like modes
  myself.

### Acknowledgements

I partially referenced the following projects.

- [helix](https://github.com/helix-editor/helix)
- [kiro-editor](https://github.com/rhysd/kiro-editor)
- [led](https://github.com/cessen/led)
- [mille](https://github.com/ad-sho-loko/mille)
