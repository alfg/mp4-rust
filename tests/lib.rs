use mp4;
use std::fs::File;
use std::io::BufReader;


#[test]
fn test_read_mp4() {
    let filename = "tests/samples/minimal.mp4";
    let f = File::open(filename).unwrap();
    let size = f.metadata().unwrap().len();
    let reader = BufReader::new(f);

    let mut mp4 = mp4::Mp4Reader::new(reader);
    mp4.read(size).unwrap();

    assert_eq!(2591, mp4.size());

    // ftyp.
    println!("{:?}", mp4.ftyp.compatible_brands);
    assert_eq!(4, mp4.ftyp.compatible_brands.len());

    // Check compatible_brands.
    let brands = vec![
        String::from("isom"),
        String::from("iso2"),
        String::from("avc1"),
        String::from("mp41")
    ];

    for b in brands {
        let t = mp4.ftyp.compatible_brands.iter().any(|x| x.to_string() == b);
        assert_eq!(t, true);
    }

    // moov.
    let moov = mp4.moov.unwrap();
    assert_eq!(moov.mvhd.version, 0);
    assert_eq!(moov.mvhd.creation_time, 0);
    assert_eq!(moov.mvhd.duration, 62);
    assert_eq!(moov.mvhd.timescale, 1000);
    assert_eq!(moov.traks.len(), 2);

}
