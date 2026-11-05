# pixelgrid

A small 2D raster image library written in plain Rust (standard library only).

Work in progress. The plan is a generic pixel buffer, Netpbm and BMP readers and writers,
drawing primitives, geometric transforms, filters and a command-line tool that chains them.

## Build

```sh
cargo build -j2
cargo test -j2
```

## License

MIT, see [LICENSE](LICENSE).
