use qoi::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let mut pixels: [u8; 18] = [
        255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0,
    ];
    let header = Header::new(3, 2, ColorChannel::RGB, ColorSpace::LINEAR);
    write(Path::new("./foo.qoi"), &header, &mut pixels).unwrap();

    let mut file = File::open(Path::new("foo.qoi")).unwrap();
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf).unwrap();
    dbg!(buf);
}
