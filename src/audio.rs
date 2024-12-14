#[cfg(test)]
mod audio_tests {
    use std::{thread, time::Duration};

    use rodio;

    #[test]
    fn play_sound() {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let source = rodio::source::SineWave::new(440.0);
        let result = stream_handle.play_raw(source);
        match result {
            Ok(_) => println!("Sound played successfully"),
            Err(err) => println!("Error playing sound: {}", err),
        }
        thread::sleep(Duration::from_secs(5));
    }
}
