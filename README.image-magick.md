## Building on FreeBSD with libs from a self-compiled ImageMagick

This is only needed if you enable the `with-magick_rust` feature and you
are not able to simply install ImageMagick 7 systemwide using
`sudo pkg install ImageMagick7`.

Compile ImageMagick 7 and install it in `$HOME/local`.
First configure it with the minimum amount of features, then gmake && gmake install:

```
cd ImageMagick-7.1.0-52
mkdir -p $HOME/local
./configure \
  --prefix=$HOME/local \
  --with-pic \
  --without-magick-plus-plus \
  --without-utilities \
  --without-x \
  --without-fpx \
  --without-djvu \
  --without-fontconfig \
  --without-freetype \
  --without-raqm \
  --without-gdi32 \
  --without-heic \
  --without-jbig \
  --without-lcms \
  --without-openjp2 \
  --without-lqr \
  --without-openexr \
  --without-pango \
  --without-raw \
  --without-tiff \
  --without-xml
gmake
gmake install
```

The `magick-rust` crate runs `bindgen`, and it needs the clang.so library which
you can install from ports:

```
pkg install llvm-devel
```

Now set the following environment variables:

```
export IMAGE_MAGICK_INCLUDE_DIRS=$HOME/local/include/ImageMagick-7
export IMAGE_MAGICK_LIB_DIRS=$HOME/local/lib
export IMAGE_MAGICK_LIBS=MagickWand-7.Q16HDRI
```

You _should_ be able to compile this with `IMAGE_MAGICK_STATIC=1`, but alas..
lots and lots of these errors:

```
          ld: error: can't create dynamic relocation R_X86_64_32 against local symbol in readonly segment; recompile object files with -fPIC or pass '-Wl,-z,notext' to allow text relocations in the output
          >>> defined in /home/username/rust/notflix-backend/target/release/deps/libmagick_rust-a5d62d32771de359.rlib(libMagickWand_7_Q16HDRI_la-magick-image.o)
          >>> referenced by magick-image.c:143 (MagickWand/magick-image.c:143)
          >>>               libMagickWand_7_Q16HDRI_la-magick-image.o:(GetImageFromMagickWand) in archive /home/username/rust/notflix-backend/target/release/deps/libmagick_rust-a5d62d32771de359.rlib
```

So, unfortunately, for now this is also needed:

```
export LD_LIBRARY_PATH=$HOME/local/lib
```

