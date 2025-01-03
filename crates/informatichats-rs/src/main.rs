use ffmpeg_next as ffmpeg;
use ffmpeg_next::ffi::{avcodec_alloc_context3, AVInputFormat};
use ffmpeg_next::format::open_with;
use ffmpeg_next::sys::av_find_input_format;
use ffmpeg_next::{Dictionary, Format};
use std::ffi::CString;
use std::path::Path;

fn record_screen(output_file: &str, duration: u64) -> Result<(), ffmpeg::Error> {
    // Initialize FFmpeg
    ffmpeg::init()?;

    // Define the input format (e.g., gdigrab for Windows, x11grab for Linux)
    let input_format = unsafe {
        #[cfg(target_os = "linux")]
        let fmt_type = CString::new("x11grab").unwrap();
        #[cfg(target_os = "windows")]
        let fmt_type = CString::new("gdigrab").unwrap();

        let fmt = av_find_input_format(fmt_type.as_ptr());
        ffmpeg::format::Input::wrap(fmt as *mut AVInputFormat)
    };

    let mut options = Dictionary::new();
    options.set("probesize", "5000000"); // Increase probesize
    options.set("analyzeduration", "5000000"); // Increase analyzeduration

    // Set up the input context
    #[cfg(target_os = "linux")]
    let ictx = open_with(":0.0", &Format::Input(input_format), options).unwrap();
    #[cfg(target_os = "windows")]
    let ictx = open_with("desktop", &Format::Input(input_format), options).unwrap();

    let mut octx = ffmpeg::format::output(Path::new(output_file)).unwrap();
    ffmpeg::format::context::output::dump(&octx, 0, Some(&output_file));
    // octx.write_header().unwrap();

    let mut input = ictx.input();

    ffmpeg::log::set_level(ffmpeg::log::Level::Verbose);

    // Set up the output context
    // let mut output = octx.format();

    // Add a video stream to the output
    let codec = ffmpeg::codec::encoder::find(ffmpeg::codec::Id::H264).unwrap();
    let global_header = octx.format().flags().contains(ffmpeg::format::flag::Flags::GLOBAL_HEADER);

    // let decoder = ffmpeg::codec::decoder::find(ffmpeg::codec::Id::H264).unwrap().decoder().unwrap().video().unwrap();
    let mut stream = input.streams().best(ffmpeg::media::Type::Video).unwrap();
    let video_stream_index = stream.index();

    let enc = ffmpeg::codec::encoder::find(ffmpeg::codec::Id::H264).unwrap();
    let mut ost = octx.add_stream(enc)?;
    // let mut _ = octx.add_stream(codec)?;
    let mut decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters()).unwrap()
        .decoder()
        .video().unwrap();

    let mut encoder =
        ffmpeg::codec::context::Context::new_with_codec(enc)
            .encoder()
            .video()?;
    encoder.set_width(1920);
    encoder.set_height(1080);
    encoder.set_time_base((1, 30));
    encoder.set_frame_rate(Some((30, 1)));
    encoder.set_bit_rate(4000 * 1000);
    encoder.set_format(ffmpeg::format::Pixel::YUV420P);
    if global_header {
        encoder.set_flags(ffmpeg::codec::flag::Flags::GLOBAL_HEADER);
    }
         let opened_encoder = encoder.open().unwrap();
    ost.set_parameters(&opened_encoder);


    // ost.set_time_base((1, 30));

    // let mut encoder = ost.codec().encoder().video()?;
    // encoder.set_height(720);
    // encoder.set_width(1280);
    // encoder.set_format(ffmpeg::format::Pixel::YUV420P);
    // encoder.set_time_base((1, 30));

    let mut packet = ffmpeg::Packet::empty();
    let mut frame = ffmpeg::frame::Video::empty();
    let start_time = ffmpeg::time::relative();

    // Capture and encode frames
    while ffmpeg::time::relative() - start_time < duration as i64 {
        if let Ok(()) = packet.read(&mut input) {
           packet.write(&mut octx).unwrap();

        }
    }

    // Write the trailer and clean up
    // octx.write_trailer()?;
    Ok(())
}

fn main() {
    let output_file = "screen_record.mp4";
    let duration = 10; // Record for 10 seconds

    match record_screen(output_file, duration) {
        Ok(_) => println!("Screen recording saved to {}", output_file),
        Err(e) => eprintln!("Error recording screen: {:?}", e),
    }
}
