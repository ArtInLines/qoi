use qoi::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let fpath = Path::new("./foo.qoi");

    let red = Pixel {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    let mut pixels: [Pixel; 6] = [
        red.clone(),
        red.clone(),
        red.clone(),
        red.clone(),
        red.clone(),
        red.clone(),
    ];
    let header = Header::new(3, 2, ColorChannel::RGB, ColorSpace::LINEAR);
    write(fpath, &header, &mut pixels).unwrap();

    let mut file = File::open(fpath).unwrap();
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf).unwrap();
    dbg!(buf);

    let (header, pixels) = read(fpath).unwrap();
    dbg!(header, pixels);
}
