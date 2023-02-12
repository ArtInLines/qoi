use qoi::*;
// use std::fs::File;
// use std::io::Read;
use std::path::Path;

fn main() {
    let fpath = Path::new("./foo.qoi");

    let mut pixels: [Pixel; 24] = [
        [192, 0, 0].into(),
        [192, 0, 0].into(),
        [0, 0, 192].into(),
        [0, 0, 192].into(),
        [0, 0, 0].into(),
        [0, 0, 0].into(),
        [0, 0, 0].into(),
        [255, 255, 255].into(),
        [128, 128, 128].into(),
        [120, 130, 130].into(),
        [100, 128, 128].into(),
        [125, 125, 125].into(),
        [128, 128, 128].into(),
        [128, 128, 128].into(),
        [255, 255, 255].into(),
        [124, 134, 71].into(),
        [124, 134, 71].into(),
        [124, 134, 71].into(),
        [128, 128, 128].into(),
        [150, 130, 130].into(),
        [150, 130, 130].into(),
        [0, 0, 0].into(),
        [0, 0, 0].into(),
        [0, 0, 0].into(),
    ];
    let header = Header::new(6, 4, ColorChannel::RGB, ColorSpace::LINEAR);
    write(fpath, &header, &mut pixels).unwrap();

    // let mut file = File::open(fpath).unwrap();
    // let mut buf = Vec::<u8>::new();
    // file.read_to_end(&mut buf).unwrap();
    // dbg!(buf);

    // let (header, pixels) = read(fpath).unwrap();
    // dbg!(header, pixels);
}
