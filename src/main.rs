use qoi::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let fpath = Path::new("./foo.qoi");

    let mut pixels: [u8; 18] = [
        255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0,
    ];
    let header = Header::new(3, 2, ColorChannel::RGB, ColorSpace::LINEAR);
    write(fpath, &header, &mut pixels).unwrap();

    let mut file = File::open(fpath).unwrap();
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf).unwrap();
    dbg!(buf);

    let mut pixels = Vec::<u8>::new();
    let header = read(fpath, &mut pixels).unwrap();
    dbg!(header, pixels);
}
