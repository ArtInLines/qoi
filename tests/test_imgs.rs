use qoi;
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

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
