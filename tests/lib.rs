use mp4::{
    AudioObjectType, AvcProfile, ChannelConfig, MediaType, Mp4Reader, SampleFreqIndex, TrackType,
};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

#[test]
fn test_read_mp4() {
    let mut mp4 = get_reader("tests/samples/minimal.mp4");

    assert_eq!(2591, mp4.size());

    // ftyp.
    assert_eq!(4, mp4.compatible_brands().len());

    // Check compatible_brands.
    let brands = vec![
        String::from("isom"),
        String::from("iso2"),
        String::from("avc1"),
        String::from("mp41"),
    ];

    for b in brands {
        let t = mp4.compatible_brands().iter().any(|x| x.to_string() == b);
        assert!(t);
    }

    assert_eq!(mp4.duration(), Duration::from_millis(62));
    assert_eq!(mp4.timescale(), 1000);
    assert_eq!(mp4.tracks().len(), 2);

    let sample_count = mp4.sample_count(1).unwrap();
    assert_eq!(sample_count, 1);
    let sample_1_1 = mp4.read_sample(1, 1).unwrap().unwrap();
    assert_eq!(sample_1_1.bytes.len(), 751);
    assert_eq!(
        sample_1_1,
        mp4::Mp4Sample {
            start_time: 0,
            duration: 512,
            rendering_offset: 0,
            is_sync: true,
            bytes: mp4::Bytes::from(vec![0x0u8; 751]),
        }
    );
    let eos = mp4.read_sample(1, 2).unwrap();
    assert!(eos.is_none());

    let sample_count = mp4.sample_count(2).unwrap();
    assert_eq!(sample_count, 3);
    let sample_2_1 = mp4.read_sample(2, 1).unwrap().unwrap();
    assert_eq!(sample_2_1.bytes.len(), 179);
    assert_eq!(
        sample_2_1,
        mp4::Mp4Sample {
            start_time: 0,
            duration: 1024,
            rendering_offset: 0,
            is_sync: true,
            bytes: mp4::Bytes::from(vec![0x0u8; 179]),
        }
    );

    let sample_2_2 = mp4.read_sample(2, 2).unwrap().unwrap();
    assert_eq!(
        sample_2_2,
        mp4::Mp4Sample {
            start_time: 1024,
            duration: 1024,
            rendering_offset: 0,
            is_sync: true,
            bytes: mp4::Bytes::from(vec![0x0u8; 180]),
        }
    );

    let sample_2_3 = mp4.read_sample(2, 3).unwrap().unwrap();
    assert_eq!(
        sample_2_3,
        mp4::Mp4Sample {
            start_time: 2048,
            duration: 896,
            rendering_offset: 0,
            is_sync: true,
            bytes: mp4::Bytes::from(vec![0x0u8; 160]),
        }
    );

    let eos = mp4.read_sample(2, 4).unwrap();
    assert!(eos.is_none());

    // track #1
    let track1 = mp4.tracks().get(&1).unwrap();
    assert_eq!(track1.track_id(), 1);
    assert_eq!(track1.track_type().unwrap(), TrackType::Video);
    assert_eq!(track1.media_type().unwrap(), MediaType::H264);
    assert_eq!(track1.video_profile().unwrap(), AvcProfile::AvcHigh);
    assert_eq!(track1.width(), 320);
    assert_eq!(track1.height(), 240);
    assert_eq!(track1.bitrate(), 0); // XXX
    assert_eq!(track1.frame_rate(), 25.00); // XXX

    // track #2
    let track2 = mp4.tracks().get(&2).unwrap();
    assert_eq!(track2.track_type().unwrap(), TrackType::Audio);
    assert_eq!(track2.media_type().unwrap(), MediaType::AAC);
    assert_eq!(
        track2.audio_profile().unwrap(),
        AudioObjectType::AacLowComplexity
    );
    assert_eq!(
        track2.sample_freq_index().unwrap(),
        SampleFreqIndex::Freq48000
    );
    assert_eq!(track2.channel_config().unwrap(), ChannelConfig::Mono);
    assert_eq!(track2.bitrate(), 67695);
}

#[test]
fn test_read_extended_audio_object_type() {
    // Extended audio object type and sample rate index of 15
    let mp4 = get_reader("tests/samples/extended_audio_object_type.mp4");

    let track = mp4.tracks().get(&1).unwrap();
    assert_eq!(track.track_type().unwrap(), TrackType::Audio);
    assert_eq!(track.media_type().unwrap(), MediaType::AAC);
    assert_eq!(
        track.audio_profile().unwrap(),
        AudioObjectType::AudioLosslessCoding
    );
    assert_eq!(
        track
            .trak
            .mdia
            .minf
            .stbl
            .stsd
            .mp4a
            .as_ref()
            .unwrap()
            .esds
            .as_ref()
            .unwrap()
            .es_desc
            .dec_config
            .dec_specific
            .freq_index,
        15
    );
    assert_eq!(track.channel_config().unwrap(), ChannelConfig::Stereo);
    assert_eq!(track.bitrate(), 839250);
}

fn get_reader(path: &str) -> Mp4Reader<BufReader<File>> {
    let f = File::open(path).unwrap();
    let f_size = f.metadata().unwrap().len();
    let reader = BufReader::new(f);

    mp4::Mp4Reader::read_header(reader, f_size).unwrap()
}
