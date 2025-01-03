use ffmpeg_next as ffmpeg;
use ffmpeg_next::ffi::{avcodec_alloc_context3, AVInputFormat, AVPixelFormat};
use ffmpeg_next::sys::av_find_input_format;
use ffmpeg_next::{Dictionary, Format};
use std::ffi::{c_int, CString};
use std::path::Path;
use ffmpeg_next::format::Pixel;

fn find_screen_grab_input_format() -> ffmpeg::format::Input {
    unsafe {
        #[cfg(target_os = "linux")]
        let fmt_type = CString::new("x11grab").unwrap();
        #[cfg(target_os = "windows")]
        let fmt_type = CString::new("gdigrab").unwrap();

        let fmt = av_find_input_format(fmt_type.as_ptr());
        ffmpeg::format::Input::wrap(fmt as *mut AVInputFormat)
    }
}

fn get_screen_grab_input_context(
    input_format: ffmpeg::format::Input,
    options: Option<Dictionary>,
) -> ffmpeg::format::context::Context {
    #[cfg(target_os = "linux")]
    return ffmpeg::format::open_with(
        ":0.0",
        &Format::Input(input_format),
        options.unwrap_or(Dictionary::new()),
    )
    .unwrap();
    #[cfg(target_os = "windows")]
    return open_with(
        "desktop",
        &Format::Input(input_format),
        options.unwrap_or(Dictionary::new()),
    )
    .unwrap();
}

fn print_stream_parameters(stream: &ffmpeg::Stream) {
    let params = unsafe { *stream.parameters().as_ptr() };
    println!("Stream parameters for: {:?}", stream);
    println!("  Codec type: {:?}", params.codec_type);
    println!("  Codec ID: {:?}", params.codec_id);
    println!("  Bit rate: {:?}", params.bit_rate);
    println!("  Width: {:?}", params.width);
    println!("  Height: {:?}", params.height);
    println!("  Sample rate: {:?}", params.sample_rate);
    println!("  Channels: {:?}", params.channels);
    println!("  Frame rate: {:?}", params.framerate);
    println!("  Pixel format: {:?}", unsafe { std::mem::transmute::<c_int, AVPixelFormat>(params.format) });
    println!();
}

fn record_screen

fn record_screen(output_file: &str, duration: u64) -> Result<(), ffmpeg::Error> {
    // Initialize FFmpeg
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Verbose);

    // Define the input format (e.g., gdigrab for Windows, x11grab for Linux)
    let input_format = find_screen_grab_input_format();

    let mut options = Dictionary::new();
    options.set("probesize", "5000000");
    options.set("analyzeduration", "5000000");
    options.set("framerate", "30");
    options.set("video_size", "1920x1080");

    // Set up the input context
    let ictx = get_screen_grab_input_context(input_format, Some(options));
    let mut input = ictx.input();

    println!(
        "Available input streams: {:?}",
        input.streams().collect::<Vec<_>>()
    );
    let mut stream = input.streams().best(ffmpeg::media::Type::Video).unwrap();
    println!("Best video stream: {:?}", stream);
    print_stream_parameters(&stream);

    // Setup the output context
    let mut octx = ffmpeg::format::output(Path::new(output_file)).unwrap();
    ffmpeg::format::context::output::dump(&octx, 0, Some(&output_file));

    // Create encoder
    let encoder = ffmpeg::encoder::find(ffmpeg::codec::Id::H264).unwrap();
    let mut encoder_ctx =
        ffmpeg::codec::context::Context::new_with_codec(encoder)
            .encoder()
            .video()?;
    let stream_params = unsafe { *stream.parameters().as_ptr() };
    encoder_ctx.set_width(stream_params.width as u32);
    encoder_ctx.set_height(stream_params.height as u32);
    encoder_ctx.set_time_base(stream.time_base());
    encoder_ctx.set_format(unsafe { Pixel::from(std::mem::transmute::<c_int, AVPixelFormat>(stream_params.format)) });
    
    let opened_encoder_ctx = encoder_ctx.open().unwrap();
    
    //Create decoder
    // let decoder = ffmpeg::decoder::find(ffmpeg::codec::Id::H264).unwrap();
    // let mut decoder_ctx =
    //     ffmpeg::codec::context::Context::new_with_codec(decoder)
    //         .decoder()
    //         .video()?;
    // decoder_ctx.
    // decoder_ctx.set_width(stream_params.width as u32);
    // decoder_ctx.set_height(stream_params.height as u32);
    // decoder_ctx.set_time_base(stream.time_base());
    // decoder_ctx.set_format(unsafe { Pixel::from(std::mem::transmute::<c_int, AVPixelFormat>(stream_params.format)) });


    let mut packet = ffmpeg::Packet::empty();
    let mut frame = ffmpeg::frame::Video::empty();
    let start_time = ffmpeg::time::relative();

    // Capture and encode frames
    packet.read(&mut input).expect("Should be able to read packet");
    

    // Write the trailer and clean up
    // octx.write_trailer()?;
    Ok(())
}

fn main() {
    let output_file = "screen_record.bmp";
    let duration = 10; // Record for 10 seconds

    match record_screen(output_file, duration) {
        Ok(_) => println!("Screen recording saved to {}", output_file),
        Err(e) => eprintln!("Error recording screen: {:?}", e),
    }
}
