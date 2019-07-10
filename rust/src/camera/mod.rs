use rscam;
use std::fs::File;
use std::io::prelude::*;

pub fn capture() {
    let mut camera = rscam::new("/dev/video0").unwrap();

    camera.start(&rscam::Config {
        interval: (1, 30),      // 30 fps.
        resolution: (1280, 720),
        format: b"MJPG",
        ..Default::default()
    }).unwrap();

    for i in 0..10 {
        let frame = camera.capture().unwrap();
        let mut file = File::create(&format!("/home/ubuntu/frame-{}.jpg", i)).unwrap();
        file.write_all(&frame[..]).unwrap();
    }
}
