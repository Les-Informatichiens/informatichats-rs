mod ffmpeg;
mod informatichat;

use eframe::egui;
use crate::informatichat::Informatichat;

fn main() -> eframe::Result {
    let encoder_codec = ffmpeg_sys_next::AVCodecID::AV_CODEC_ID_BMP;
    let decoder_codec = ffmpeg_sys_next::AVCodecID::AV_CODEC_ID_BMP;

    let context = unsafe { ffmpeg::Context::new(encoder_codec, decoder_codec) };

    unsafe {
        context.grab_screen();
    }

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Informatichat",
        options,
        Box::new(|_cc| Ok(Box::new(Informatichat::new()))),
    )
}
