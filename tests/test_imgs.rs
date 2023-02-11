use qoi;
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

#[test]
fn test_decoder() {
    let decoded_path = Path::new("./imgs/testcard.bin");
    let encoded_path = Path::new("./imgs/testcard.qoi");

    let mut decoded_file = File::open(decoded_path).unwrap();
    let mut encoded_file = File::open(encoded_path).unwrap();

    let mut decoded_buf = Vec::<u8>::new();
    decoded_file.read_to_end(&mut decoded_buf).unwrap();

    let mut encoded_buf = Vec::<u8>::new();
    encoded_file.read_to_end(&mut encoded_buf).unwrap();

    let decoded_pixels: Vec<qoi::Pixel> = decoded_buf.chunks(3).map(|chunk| chunk.into()).collect();

    let decoded_header = qoi::Header {
        width: 256,
        height: 256,
        channels: qoi::ColorChannel::RGB,
        colorspace: qoi::ColorSpace::LINEAR,
    };

    let (header, pixels) = qoi::decode::decode_allocated(&encoded_buf).unwrap();

    assert_eq!(header, decoded_header);
    assert_eq!(pixels.len(), decoded_pixels.len());
    assert_eq!(pixels, decoded_pixels);
}

#[test]
fn test_encoder() {
    let decoded_path = Path::new("./imgs/testcard.bin");
    let encoded_path = Path::new("./imgs/testcard.qoi");

    let mut decoded_file = File::open(decoded_path).unwrap();
    let mut encoded_file = File::open(encoded_path).unwrap();

    let mut decoded_buf = Vec::<u8>::new();
    decoded_file.read_to_end(&mut decoded_buf).unwrap();

    let mut encoded_buf = Vec::<u8>::new();
    encoded_file.read_to_end(&mut encoded_buf).unwrap();

    let header = qoi::Header {
        width: 256,
        height: 256,
        channels: qoi::ColorChannel::RGB,
        colorspace: qoi::ColorSpace::LINEAR,
    };

    let pixels: Vec<qoi::Pixel> = decoded_buf.chunks(3).map(|chunk| chunk.into()).collect();

    let res = qoi::encode::encode_allocated(&header, &pixels).unwrap();

    assert_eq!(res.len(), encoded_buf.len());
    assert_eq!(res, encoded_buf);
}

#[test]
fn test_imgs() {
    fs::read_dir(Path::new("./imgs"))
        .unwrap()
        .filter(|x| {
            x.is_ok()
                && format!("{}", x.as_ref().unwrap().file_name().to_str().unwrap())
                    .ends_with(".qoi")
        })
        .for_each(|file| {
            let file = file.unwrap();
            let path = file.path();

            let mut file = File::open(path.clone()).unwrap();
            let mut buf = Vec::<u8>::new();
            file.read_to_end(&mut buf).unwrap();

            let res = qoi::read(path.clone());
            assert!(res.is_ok(), "{:?}", res.unwrap_err());
            let (header, mut pixels) = res.unwrap();

            let res = qoi::encode::encode_allocated(&header, &mut pixels);
            assert!(res.is_ok(), "{:?}", res.unwrap_err());
            let vec = res.unwrap();

            assert_eq!(vec.len(), buf.len());
            assert_eq!(vec, buf);
        })
}
