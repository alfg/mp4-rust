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
    assert!(mp4.moov.is_some());
    if let Some(ref moov) = mp4.moov {
        assert_eq!(moov.mvhd.version, 0);
        assert_eq!(moov.mvhd.creation_time, 0);
        assert_eq!(moov.mvhd.duration, 62);
        assert_eq!(moov.mvhd.timescale, 1000);
        assert_eq!(moov.traks.len(), 2);
    }

    let sample_count = mp4.sample_count(1).unwrap();
    assert_eq!(sample_count, 0);

    let sample_count = mp4.sample_count(2).unwrap();
    assert_eq!(sample_count, 3);
    let sample1 = mp4.read_sample(2, 1).unwrap().unwrap();
    assert_eq!(sample1.bytes.len(), 179);
    assert_eq!(sample1, mp4::Mp4Sample {
        start_time: 0,
        duration: 1024,
        rendering_offset: 0,
        is_sync: true,
        bytes: mp4::Bytes::from(vec![0x0u8; 179]),
    });

    let sample2 = mp4.read_sample(2, 2).unwrap().unwrap();
    assert_eq!(sample2, mp4::Mp4Sample {
        start_time: 1024,
        duration: 1024,
        rendering_offset: 0,
        is_sync: true,
        bytes: mp4::Bytes::from(vec![0x0u8; 180]),
    });

    let sample3 = mp4.read_sample(2, 3).unwrap().unwrap();
    assert_eq!(sample3, mp4::Mp4Sample {
        start_time: 2048,
        duration: 896,
        rendering_offset: 0,
        is_sync: true,
        bytes: mp4::Bytes::from(vec![0x0u8; 160]),
    });

    let eos = mp4.read_sample(2, 4).unwrap();
    assert!(eos.is_none());
}
